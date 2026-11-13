use super::*;

impl Game {
    /// Cities and jams of the current track: plan them, clear the scenery inside the cities, and
    /// line up the standing queues.
    pub(super) fn setup_city(&mut self, seed: u64) {
        self.end_detour();
        let cz: Vec<f32> = self.crossings.iter().map(|c| c.z).collect();
        self.city = crate::city::plan(&self.track, seed, &self.stations, &cz);
        if matches!(self.mode, Mode::TimeTrial | Mode::Zen) {
            self.city.jams.clear();
        }
        let n = self.track.segs.len();
        for z in &self.city.zones {
            for i in z.a..z.b.min(n) {
                self.track.segs[i].props.clear();
            }
        }
        for j in &self.city.jams {
            for i in j.exit_a()..j.b.min(n) {
                self.track.segs[i].props.clear();
            }
        }
        self.city_animals();
        let len = self.track.length;
        self.jam_cars.clear();
        for j in self.city.jams.clone() {
            for (z, x, kind, color) in crate::city::jam_cars(len, &j, seed) {
                self.jam_cars.push(Car {
                    kind,
                    z,
                    x,
                    speed: 0.0,
                    base: 0.0,
                    color,
                    prev_dz: 0.0,
                });
            }
        }
        self.prev_zone = None;
        self.jam_warn = usize::MAX;
        self.scene_warn = usize::MAX;
        self.emerg.clear();
        let cz: Vec<f32> = self.crossings.iter().map(|c| c.z).collect();
        self.mission = if self.mode == Mode::Zen {
            None
        } else {
            crate::mission::plan(&self.track, seed, &self.city, &self.stations, &cz)
        };
        self.m_state = crate::mission::WAITING;
        self.m_t = 0.0;
        self.m_dwell = 0.0;
        self.m_hint = 0;
        // bonus areas: the proving ground gets its hill, parking lots and hills lose their scenery
        self.areas = if self.mode == Mode::Zen {
            Vec::new()
        } else {
            crate::bonus::plan(
                &self.track,
                seed,
                &self.city,
                &self.stations,
                &cz,
                self.mission.as_ref(),
            )
        };
        self.a_state = vec![0; self.areas.len()];
        self.a_dwell = 0.0;
        for ar in self.areas.clone() {
            for i in ar.a.saturating_sub(10)..(ar.b + 10).min(n) {
                self.track.segs[i].props.clear();
            }
            if ar.kind == crate::bonus::PROVING {
                for i in ar.a..=(ar.b).min(n - 1) {
                    let (h0, h1) = (crate::bonus::hump(ar.a, i), crate::bonus::hump(ar.a, i + 1));
                    self.track.segs[i].y0 += h0;
                    self.track.segs[i].y1 += h1;
                }
            }
        }
        self.inject_areas();
        self.plan_fork();
    }

    /// The proving ground's cones, rings, pads, ramp and coins (idempotent: the area is cleared
    /// of earlier content first).
    pub(super) fn inject_areas(&mut self) {
        use crate::bonus as b;
        for ar in self.areas.clone() {
            if ar.kind != b::PROVING {
                continue;
            }
            let (za, zb) = (ar.az(), ar.bz());
            self.items.retain(|i| i.z < za || i.z > zb);
            self.orbs.retain(|o| o.z < za || o.z > zb);
            let seg = |k: usize| (ar.a + k) as f32 * SEG_LEN;
            let mut put = |k: usize, x: f32, kind: u8| {
                self.items.push(Item {
                    z: seg(k),
                    x,
                    kind,
                    taken: false,
                });
            };
            for i in 0..6 {
                put(8 + i * 4, if i % 2 == 0 { -0.5 } else { 0.5 }, I_CONE);
            }
            put(32, 0.0, I_PAD);
            for i in 0..10 {
                put(128 + i, 0.0, I_COIN);
            }
            put(124, 0.0, I_RAMP_L);
            for i in 0..12 {
                self.orbs.push(Orb {
                    z: seg(40 + i * 7),
                    x: 0.55 * (i as f32 * 1.3).sin(),
                    kind: 1,
                    taken: false,
                });
            }
        }
        self.items.sort_by(|a, b| a.z.total_cmp(&b.z));
    }

    /// Slope, proving-ground result and parking.
    pub(super) fn area_update(&mut self, old_pos: f32, dt: f32, zen: bool) {
        use crate::bonus as b;
        let pz = self.pos + PLAYER_Z;
        let n = self.track.segs.len();
        let seg = (pz / SEG_LEN) as usize % n;
        for i in 0..self.areas.len() {
            let ar = self.areas[i];
            let d = ar.az() - pz;
            if self.a_state[i] == 0 && d > 0.0 && d < 20000.0 && self.banner_t < 0.5 {
                self.a_state[i] = 1;
                self.banner = if ar.kind == b::PROVING {
                    "PROVING GROUND"
                } else {
                    "PARKING BONUS"
                };
                self.banner_t = 2.0;
            }
            if ar.kind == b::PROVING {
                if seg >= ar.a && seg < ar.b {
                    // the hill pulls and pushes the car
                    let sg = &self.track.segs[seg];
                    let grade = (sg.y1 - sg.y0) / SEG_LEN;
                    self.speed = (self.speed - grade * 5200.0 * dt).max(0.0);
                }
                if self.a_state[i] < 2 && pz > ar.bz() + SEG_LEN * 4.0 && old_pos + PLAYER_Z <= pz {
                    self.a_state[i] = 2;
                    let (za, zb) = (ar.az(), ar.bz());
                    let got = self
                        .orbs
                        .iter()
                        .filter(|o| o.taken && o.z >= za && o.z <= zb)
                        .count() as u32;
                    if got >= b::RINGS_NEEDED && !zen {
                        self.tracks_clear += 1;
                        self.score += 2500;
                        self.time += 6.0;
                        self.civic += 2;
                        self.banner = "TRACK CLEAR!";
                        self.banner_t = 2.0;
                    }
                }
            } else if self.a_state[i] < 2 && pz >= ar.az() && pz <= ar.bz() {
                if self.x > b::LOT_EDGE && self.speed < 700.0 && self.low() && !zen {
                    self.a_dwell += dt;
                    if self.a_dwell >= 1.5 {
                        self.a_state[i] = 2;
                        self.parked += 1;
                        self.score += 1000;
                        self.time += 4.0;
                        let off = b::SLOTS
                            .iter()
                            .map(|c| (self.x - c).abs())
                            .fold(f32::MAX, f32::min);
                        if off < 0.10 {
                            self.score += 1500;
                            self.civic += 2;
                            self.banner = "PERFECT PARKING!";
                        } else {
                            self.civic += 1;
                            self.banner = "PARKED!";
                        }
                        self.banner_t = 2.0;
                    }
                } else {
                    self.a_dwell = 0.0;
                }
            }
        }
    }

    /// In a parking lot the verge is tarmac: no off-road drag.
    pub(super) fn in_lot(&self) -> bool {
        let pz = self.pos + PLAYER_Z;
        self.areas
            .iter()
            .any(|a| a.kind == crate::bonus::PARKING && pz >= a.az() && pz <= a.bz())
    }

    /// Score multiplier for a run of responsible driving: x1 to x3.
    pub fn civic_mult(&self) -> u32 {
        1 + (self.civic.min(6) / 3)
    }

    /// A good act: stopping at a red light, giving way to a pedestrian, a clean city.
    pub(super) fn good_act(&mut self, pts: u32, secs: f32, label: &'static str) {
        self.civic += 1;
        self.good_acts += 1;
        self.score += pts * self.civic_mult();
        self.time += secs;
        if self.banner_t < 0.6 || label == "CLEAN CITY" {
            self.banner = label;
            self.banner_t = 1.5;
        }
    }

    /// On the ground (can collide, be fined, pick up cargo). A helicopter above walking speed flies.
    pub fn low(&self) -> bool {
        if self.car.class.is_heli() && self.speed >= 2200.0 {
            return false;
        }
        self.air < 0.5
    }

    /// Police, fire engine and ambulance may break the rules; a helicopter is above them.
    pub(super) fn privileged(&self) -> bool {
        self.car.class.is_special()
    }

    /// A traffic offence: the police are called (or, where there are none, a fine costs time).
    pub(super) fn violation(&mut self, label: &'static str) {
        if self.privileged() {
            return;
        }
        self.violations += 1;
        self.civic = 0;
        self.zone_clean = false;
        if self.viol_cd > 0.0 {
            return;
        }
        self.viol_cd = 3.0;
        let (_, _, maxc) = self.cop_params_r();
        let pz = self.pos + PLAYER_Z;
        if maxc == 0 {
            self.time -= 3.0;
            self.banner = "FINE";
            self.banner_t = 1.4;
        } else {
            if self.cops.len() < maxc + 1 {
                self.cops.push(Cop {
                    z: (pz - 6000.0).max(0.0),
                    x: 0.0,
                    speed: self.speed.max(MAX_SPEED * 0.4),
                    age: 0.0,
                    mode: 0,
                });
            }
            self.bust = (self.bust + 0.12).min(0.9);
            self.banner = label;
            self.banner_t = 1.4;
        }
    }

    /// Cities (limit, lights, crosswalks, pedestrians) and jams (queue, slip road, detour).
    pub(super) fn city_update(&mut self, old_pos: f32, dt: f32, zen: bool) {
        let n = self.track.segs.len();
        let (old_pz, new_pz) = (old_pos + PLAYER_Z, self.pos + PLAYER_Z);
        let seg = (new_pz / SEG_LEN) as usize % n;
        let prot = self.shield_t > 0.0 || self.invuln > 0.0;
        self.viol_cd = (self.viol_cd - dt).max(0.0);
        self.fork_update(seg, new_pz);

        // entering a city: the limit applies
        let zi = self.city.zone_at(seg);
        if zi != self.prev_zone {
            if self.prev_zone.is_some() && self.zone_clean && !zen {
                // emergency vehicles break no rules by privilege: a clean city proves nothing
                if !self.privileged() {
                    self.clean_cities += 1;
                }
                self.civic += 2;
                self.good_act(1000, 4.0, "CLEAN CITY");
            }
            if zi.is_some() {
                self.zone_clean = true;
                if self.banner_t < 0.3 {
                    self.banner = "CITY";
                    self.banner_t = 1.6;
                }
            }
            self.prev_zone = zi;
        }
        if zi.is_some() && !zen && self.low() {
            if self.speed > MAX_SPEED * city::LIMIT * 1.12 {
                self.speeding_t += dt;
                if self.speeding_t > 0.4 && self.banner_t < 0.2 {
                    self.banner = "SPEEDING!";
                    self.banner_t = 0.5;
                }
                if self.speeding_t > 1.8 {
                    self.speeding_t = 0.0;
                    self.violation("SPEEDING!");
                }
            } else {
                self.speeding_t = (self.speeding_t - 2.0 * dt).max(0.0);
            }
        } else {
            self.speeding_t = 0.0;
        }

        // crosswalks: red lights and pedestrians
        for i in 0..self.city.lights.len() {
            let l = self.city.lights[i];
            if !(l.z >= old_pz && l.z < new_pz.max(old_pz + 1.0)) || !self.low() {
                continue;
            }
            let on_road = l.ped_on_road(self.elapsed);
            let st = l.state(self.elapsed);
            let crawl = self.speed < 1500.0;
            if on_road && !zen {
                let px = l.ped_x(self.elapsed).unwrap_or(9.0);
                if !prot && (self.x - px).abs() < 0.16 + 0.09 * self.car.width && !crawl {
                    self.peds_hit += 1;
                    self.speed = (self.speed * 0.25 * self.car.crash_keep * self.level.params().3)
                        .min(self.speed * 0.95);
                    self.time -= 5.0;
                    self.shake = 1.0;
                    self.invuln = 1.0;
                    self.crashes += 1;
                    self.hurt(0.15, 0.05);
                    self.burst_sparks();
                    self.banner = "PEDESTRIAN!";
                    self.banner_t = 1.6;
                    self.viol_cd = 0.0;
                    self.violation("PEDESTRIAN!");
                } else if crawl {
                    // gave way and crept past
                    self.good_act(300, 2.0, "GOOD DRIVER");
                    self.peds_yielded += 1;
                } else {
                    self.violation("PEDESTRIAN!");
                }
            } else if !zen && l.lit && st == 2 && self.speed > 600.0 {
                self.violation("RED LIGHT!");
            } else if !zen {
                if l.lit && st == 0 && self.waited && self.speed > 300.0 {
                    // waited for the green
                    self.good_act(250, 1.5, "GOOD DRIVER");
                } else if self.speed > 600.0 && self.speed <= MAX_SPEED * city::LIMIT * 1.05 {
                    self.score += 40 * self.civic_mult();
                }
                if l.ped_x(self.elapsed).is_some() && self.speed <= 3500.0 {
                    self.peds_yielded += 1;
                    self.good_act(150, 0.0, "GOOD DRIVER");
                }
            }
            self.waited = false;
        }
        // stopping for a red light or a pedestrian is remembered until the crosswalk is passed
        if !zen && self.speed < 500.0 {
            let pz = new_pz;
            if self.city.lights.iter().any(|l| {
                l.z > pz
                    && l.z - pz < 2500.0
                    && (l.state(self.elapsed) == 2 || l.ped_x(self.elapsed).is_some())
            }) {
                self.waited = true;
            }
        }

        // jams: warning, slip road, detour
        if let Some((ji, j)) = self.city.next_jam(seg) {
            let j = *j;
            let ea = j.exit_a() as f32 * SEG_LEN;
            let d = ea - new_pz;
            if d > 0.0 && d < 20000.0 && self.jam_warn != ji && self.detour.is_none() {
                self.jam_warn = ji;
                self.banner = "JAM AHEAD";
                self.banner_t = 2.2;
            }
            if self.detour.is_none()
                && seg >= j.exit_a()
                && seg < j.a
                && self.x > 1.08
                && self.low()
            {
                self.detour = Some(ji);
                self.detours += 1;
                self.detour_orig = (j.a..j.b).map(|i| self.track.segs[i % n].curve).collect();
                let len = (j.b - j.a) as f32;
                for k in 0..(j.b - j.a) {
                    let ph = k as f32 / len * std::f32::consts::TAU * 2.0;
                    self.track.segs[(j.a + k) % n].curve += 2.2 * ph.sin();
                }
                self.banner = "DETOUR";
                self.banner_t = 1.8;
                self.score += 400;
            }
        }
        if let Some(ji) = self.detour {
            let j = self.city.jams[ji];
            self.speed = self.speed.min(MAX_SPEED * city::DETOUR_CAP);
            self.x = self.x.clamp(-1.0, 1.0);
            if seg >= j.b {
                self.end_detour();
            }
        } else if self.low() && !self.demo {
            // the queue: creeping through is fine, driving into it is not
            let pz = new_pz;
            let len = self.track.length;
            for i in 0..self.jam_cars.len() {
                let c = &self.jam_cars[i];
                let dz = wrap(c.z - pz, len);
                if dz.abs() > SEG_LEN * 0.8
                    || (self.x - c.x).abs() >= kind_half(c.kind) + 0.13 * self.car.width
                {
                    continue;
                }
                let side = if self.x >= c.x { 1.0 } else { -1.0 };
                if self.speed < 1800.0 || prot || zen {
                    self.speed = self.speed.min(500.0);
                    self.x += side * 6.0 * dt;
                } else if self.invuln <= 0.0 {
                    self.speed = (self.speed * 0.3 * self.car.crash_keep).min(self.speed * 0.9);
                    self.shake = 0.5;
                    self.invuln = 0.6;
                    self.x += side * 0.2;
                    self.crashes += 1;
                    self.hurt(0.08, 0.04);
                    self.burst_sparks();
                }
            }
        }
    }

    pub(super) fn mission_update(&mut self, dt: f32) {
        use crate::mission as m;
        let Some(ms) = self.mission else {
            return;
        };
        let pz = self.pos + PLAYER_Z;
        let in_bay = |z: f32| (pz - z).abs() < m::BAY_LEN;
        let stopped = self.speed < m::STOP_SPEED && self.x > m::BAY_X && self.low();
        let label = |kind: u8| {
            if kind == m::PARCEL {
                "PARCEL LOADED"
            } else {
                "PASSENGER ON BOARD"
            }
        };
        match self.m_state {
            m::WAITING => {
                let d = ms.pick_z() - pz;
                if d > 0.0 && d < 20000.0 && self.m_hint & 1 == 0 && self.banner_t < 0.5 {
                    self.m_hint |= 1;
                    self.banner = "PICK UP";
                    self.banner_t = 2.0;
                }
                if in_bay(ms.pick_z()) && stopped {
                    self.m_dwell += dt;
                    if self.m_dwell >= m::DWELL {
                        self.m_state = m::CARRYING;
                        self.m_t = ms.limit;
                        self.m_dwell = 0.0;
                        self.m_body0 = self.body;
                        self.m_viol0 = self.violations;
                        self.m_crash0 = self.crashes;
                        self.banner = label(ms.kind);
                        self.banner_t = 1.8;
                    }
                } else {
                    self.m_dwell = 0.0;
                }
                if pz > ms.pick_z() + m::BAY_LEN + 1000.0 {
                    self.m_state = m::FAILED;
                    self.banner = "MISSED";
                    self.banner_t = 1.6;
                }
            }
            m::CARRYING => {
                let was = self.m_t;
                self.m_t -= dt;
                if was > 0.0 && self.m_t <= 0.0 {
                    self.banner = "TOO LATE";
                    self.banner_t = 1.6;
                }
                let d = ms.drop_z() - pz;
                if d > 0.0 && d < 20000.0 && self.m_hint & 2 == 0 && self.banner_t < 0.5 {
                    self.m_hint |= 2;
                    self.banner = "DROP OFF";
                    self.banner_t = 2.0;
                }
                if in_bay(ms.drop_z()) && stopped {
                    self.m_dwell += dt;
                    if self.m_dwell >= m::DWELL {
                        self.m_state = m::DONE;
                        self.missions_done += 1;
                        let on_time = self.m_t > 0.0;
                        let careful = if ms.kind == m::PARCEL {
                            self.body >= self.m_body0 - 0.05
                        } else {
                            self.violations == self.m_viol0 && self.crashes == self.m_crash0
                        };
                        let mut pts = 1500 + if on_time { (self.m_t * 25.0) as u32 } else { 0 };
                        if careful {
                            pts += 1000;
                        }
                        self.score += pts;
                        self.time += if on_time { 10.0 } else { 3.0 };
                        self.civic += if careful { 2 } else { 1 };
                        self.banner = "DELIVERED!";
                        self.banner_t = 2.0;
                    }
                } else {
                    self.m_dwell = 0.0;
                }
                if pz > ms.drop_z() + m::BAY_LEN + 1000.0 {
                    self.m_state = m::FAILED;
                    self.banner = "MISSED";
                    self.banner_t = 1.6;
                }
            }
            _ => {}
        }
    }

    /// Ambulances and fire engines overtaking with their sirens on, and the accident scenes.
    pub(super) fn emerg_update(&mut self, old_pos: f32, dt: f32, zen: bool) {
        let len = self.track.length;
        let n = self.track.segs.len();
        let pz = self.pos + PLAYER_Z;
        let old_pz = old_pos + PLAYER_Z;
        let seg = (pz / SEG_LEN) as usize % n;
        let prot = self.shield_t > 0.0 || self.invuln > 0.0;
        // accident scenes: warning and solid objects on the right-hand side
        for i in 0..self.city.scenes.len() {
            let z0 = self.city.scenes[i];
            let d = z0 - pz;
            if d > 0.0 && d < 18000.0 && self.scene_warn != i && self.banner_t < 0.5 {
                self.scene_warn = i;
                self.banner = "ACCIDENT";
                self.banner_t = 1.8;
            }
            if zen || prot || !self.low() || self.detour.is_some() {
                continue;
            }
            for &(dz, x, half, _) in &city::SCENE {
                let (zo, hx) = (z0 + dz, half + 0.12 * self.car.width);
                if zo >= old_pz - SEG_LEN * 0.7
                    && zo <= pz + SEG_LEN * 0.7
                    && (self.x - x).abs() < hx
                    && self.invuln <= 0.0
                {
                    self.speed = (self.speed * 0.3 * self.car.crash_keep).min(self.speed * 0.9);
                    self.shake = 0.7;
                    self.invuln = 0.8;
                    self.x += if self.x >= x { 0.25 } else { -0.25 };
                    self.crashes += 1;
                    self.hurt(0.12, 0.05);
                    self.burst_sparks();
                    self.banner = "CRASH";
                    self.banner_t = 1.2;
                }
            }
        }
        // sirens from behind: make room
        let traffic = !matches!(self.mode, Mode::TimeTrial | Mode::Zen);
        let in_jam = self.city.jam_at(seg).is_some() || self.detour.is_some();
        if traffic
            && !zen
            && self.emerg.is_empty()
            && self.elapsed >= self.next_emerg
            && pz > 12000.0
            && !in_jam
            && self.city.scenes.iter().all(|&z| (z - pz).abs() > 12000.0)
        {
            let kind = (self.fx_rng.f() < 0.45) as u8;
            self.emerg.push(Emerg {
                kind,
                z: pz - 11000.0,
                x: 0.0,
                speed: (self.speed * 1.15).clamp(MAX_SPEED * 0.62, MAX_SPEED * 0.9),
                target: 0.0,
                passed: false,
                warned: false,
            });
            self.next_emerg = self.elapsed + 40.0 + self.fx_rng.f() * 25.0;
        }
        let cars: Vec<(f32, f32)> = self.cars.iter().map(|c| (c.z, c.x)).collect();
        let mut blocked = false;
        let mut i = 0;
        while i < self.emerg.len() {
            let mut e = self.emerg[i];
            let d = wrap(pz - e.z, len); // > 0: behind the player
            if d > 2500.0 {
                // pick a lane away from the player and free of traffic
                let mut best = (f32::MIN, 0.0f32);
                for lane in LANES {
                    let free = cars.iter().all(|&(cz, cx)| {
                        !((0.0..4500.0).contains(&wrap(cz - e.z, len)) && (cx - lane).abs() < 0.3)
                    });
                    let sc = (lane - self.x).abs() + if free { 2.0 } else { 0.0 };
                    if sc > best.0 {
                        best = (sc, lane);
                    }
                }
                e.target = best.1;
            }
            e.x += (e.target - e.x).clamp(-1.2 * dt, 1.2 * dt);
            e.z += e.speed * dt;
            let dz = wrap(e.z - pz, len);
            if !e.warned && d < 15000.0 && d > 0.0 {
                e.warned = true;
                if self.banner_t < 0.5 {
                    self.banner = "EMERGENCY";
                    self.banner_t = 1.8;
                }
            }
            if d > 0.0 && d < 2400.0 && (self.x - e.x).abs() < 0.32 && self.speed < e.speed {
                blocked = true;
            }
            if dz.abs() < SEG_LEN * 0.8
                && (self.x - e.x).abs() < 0.2 + 0.13 * self.car.width
                && !prot
                && !zen
                && self.invuln <= 0.0
                && self.low()
            {
                self.speed *= 0.6;
                self.shake = 0.6;
                self.invuln = 0.8;
                self.x += if self.x >= e.x { 0.22 } else { -0.22 };
                self.crashes += 1;
                self.hurt(0.1, 0.04);
                self.burst_sparks();
                e.passed = true;
            }
            if !e.passed && dz > SEG_LEN {
                e.passed = true;
                if !zen {
                    self.emerg_passed += 1;
                    self.good_act(500, 2.0, "GOOD DRIVER");
                }
            }
            if dz > 26000.0 {
                self.emerg.remove(i);
                continue;
            }
            self.emerg[i] = e;
            i += 1;
        }
        if blocked && !zen {
            self.block_t += dt;
            if self.block_t > 1.0 && self.banner_t < 0.3 {
                self.banner = "MOVE OVER!";
                self.banner_t = 0.6;
            }
            if self.block_t > 3.0 {
                self.block_t = 0.0;
                self.violation("MOVE OVER!");
            }
        } else {
            self.block_t = (self.block_t - dt).max(0.0);
        }
    }

    /// No animals wander through the cities; the proving ground's content is (re)placed.
    pub(super) fn city_animals(&mut self) {
        self.inject_areas();
        let zones = self.city.zones.clone();
        self.animals.retain(|a| {
            let sg = (a.z / SEG_LEN) as usize;
            !zones.iter().any(|z| sg + 12 >= z.a && sg < z.b + 12)
        });
    }

    /// Back on the main road: the original curves return.
    pub(super) fn end_detour(&mut self) {
        if let Some(ji) = self.detour.take() {
            if let Some(j) = self.city.jams.get(ji) {
                for (k, &c) in self.detour_orig.iter().enumerate() {
                    if let Some(sg) = self.track.segs.get_mut(j.a + k) {
                        sg.curve = c;
                    }
                }
            }
        }
        self.detour_orig.clear();
    }
}
