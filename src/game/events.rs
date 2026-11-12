use super::*;

impl Game {
    /// A fork in the road about 40 % into the stage, away from cities, jams, bonus areas, the
    /// delivery bays and every structure. Left takes the quick route, right the scenic one.
    /// Puts a pack of rivals on the grid (not in Time Trial or Zen).
    pub fn set_pack(&mut self, cfg: Option<crate::pack::PackCfg>) {
        self.pack_cfg = cfg.map(|c| c.sanitized());
        self.pack.clear();
        self.place = 0;
        if let Some(c) = self.pack_cfg {
            if !matches!(self.mode, Mode::TimeTrial | Mode::Zen) {
                self.pack = crate::pack::grid(self.seed, c, self.pos + PLAYER_Z);
                // the player's own starting place in the standings
                self.place = crate::pack::position(&self.pack, self.pos + PLAYER_Z);
            }
        }
    }

    /// Reference time of the stage (what a calm clean drive takes), for qualifying.
    pub fn reference_secs(&self) -> f32 {
        self.track.finish_seg as f32 * SEG_LEN / (MAX_SPEED * 0.5)
    }

    pub(super) fn pack_update(&mut self, dt: f32) {
        if self.pack.is_empty() {
            return;
        }
        let pz = self.pos + PLAYER_Z;
        let t = self.elapsed;
        let prot = self.shield_t > 0.0 || self.invuln > 0.0;
        let low = self.low();
        let mut hit = false;
        for r in self.pack.iter_mut() {
            r.update(dt, t, &self.track, pz);
            let dz = r.z - pz;
            if !prot
                && low
                && dz.abs() < SEG_LEN * 0.8
                && (self.x - r.x).abs() < 0.3 + 0.13 * self.car.width
                && self.speed > r.speed
            {
                self.speed = (r.speed * 0.6 * self.car.crash_keep * self.level.params().3)
                    .min(self.speed * 0.95);
                self.shake = 0.5;
                self.invuln = 0.6;
                self.x += if self.x >= r.x { 0.2 } else { -0.2 };
                hit = true;
            }
        }
        if hit {
            self.crashes += 1;
            self.hurt(0.1, 0.06);
            self.burst_sparks();
        }
        self.place = crate::pack::position(&self.pack, pz);
    }

    pub(super) fn plan_fork(&mut self) {
        self.fork = None;
        self.pursuit = None;
        self.pursuit_done = false;
        self.route = 0;
        self.fork_warn = false;
        self.route_paid = false;
        if self.mode == Mode::Zen {
            return;
        }
        let total = self.track.finish_seg;
        let len = 70usize;
        let n = self.track.segs.len();
        let h = crate::career::splitmix(self.seed ^ 0xF0_4C);
        let free = |s: usize, t: &Game| -> bool {
            let (a, b) = (s.saturating_sub(40), s + len + 40);
            b + 20 < total
                && (a..b).all(|i| t.track.segs[i % n].feat == crate::track::F_NONE)
                && t.track
                    .checkpoints
                    .iter()
                    .all(|&c| c + 5 < s || c > s + len + 5)
                && !t.city.zones.iter().any(|z| b >= z.a && a < z.b)
                && !t.city.jams.iter().any(|j| b >= j.exit_a() && a < j.b + 20)
                && !t.areas.iter().any(|ar| b >= ar.a && a < ar.b)
                && !t
                    .mission
                    .iter()
                    .any(|m| (m.pick + 20 >= a && m.pick < b) || (m.drop + 20 >= a && m.drop < b))
                && !t.crossings.iter().any(|c| {
                    let cs = (c.z / SEG_LEN) as usize;
                    cs + 20 >= a && cs < b
                })
                && !t.stations.iter().any(|&(za, zb)| {
                    (zb / SEG_LEN) as usize + 20 >= a && ((za / SEG_LEN) as usize) < b
                })
        };
        let start = (total as f32 * (0.36 + (h % 100) as f32 / 1000.0)) as usize;
        for step in 0..300usize {
            for sgn in [1i64, -1] {
                let s = start as i64 + sgn * step as i64 * 4;
                if s > 200 && free(s as usize, self) {
                    self.fork = Some((s as usize, len));
                    return;
                }
            }
        }
    }

    /// Warns before the fork and applies the route chosen when the fork is crossed.
    pub(super) fn fork_update(&mut self, seg: usize, new_pz: f32) {
        let Some((fs, len)) = self.fork else { return };
        let n = self.track.segs.len();
        let d = fs as f32 * SEG_LEN - new_pz;
        if !self.fork_warn && d > 0.0 && d < 22000.0 {
            self.fork_warn = true;
            self.banner = "FORK AHEAD";
            self.banner_t = 2.5;
        }
        if self.route == 0 && seg >= fs && seg < fs + len {
            let quick = self.x < 0.0;
            self.route = if quick { 1 } else { 2 };
            for k in 0..len {
                let sg = &mut self.track.segs[(fs + k) % n];
                if quick {
                    sg.curve *= 0.5;
                } else {
                    let ph = k as f32 / len as f32 * std::f32::consts::TAU;
                    sg.curve = sg.curve * 1.2 + 0.5 * ph.sin();
                    sg.biome = (sg.biome + 1) % crate::NB as u8;
                }
            }
            self.banner = if quick { "QUICK ROUTE" } else { "SCENIC ROUTE" };
            self.banner_t = 2.0;
        }
        // the two routes end differently: the quick one saves time, the scenic one pays points
        if self.route != 0 && !self.route_paid && seg >= fs + len {
            self.route_paid = true;
            if self.route == 1 {
                self.time += 8.0;
            } else {
                self.score += 2000;
            }
        }
    }

    /// Parcels and passengers: load in the pick-up bay, unload in the drop-off bay.
    /// Police pursuit: only for the police car, never in Zen; one fugitive per stage.
    pub(super) fn pursuit_update(&mut self, dt: f32) {
        use crate::pursuit::Phase;
        if self.car.class != crate::cars::VehicleClass::Police
            || self.mode == Mode::Zen
            || self.state != State::Playing
        {
            return;
        }
        let pz = self.pos + PLAYER_Z;
        let finish = self.track.finish_seg as f32 * SEG_LEN;
        if self.pursuit.is_none() && !self.pursuit_done && self.elapsed > 3.0 && pz < finish * 0.6 {
            self.pursuit = Some(crate::pursuit::Pursuit::new(
                self.seed,
                pz,
                7000.0,
                self.crashes,
            ));
            self.banner = "CATCH UP";
            self.banner_t = 1.5;
        }
        let Some(mut p) = self.pursuit else { return };
        let before = p.phase;
        p.update(dt, pz, self.x, self.speed, self.crashes, finish);
        if p.phase != before {
            match p.phase {
                Phase::Ram => {
                    self.banner = "RAM HIM!";
                    self.banner_t = 1.5;
                }
                Phase::Caught => {
                    self.banner = "ARRESTED!";
                    self.banner_t = 2.0;
                    self.score += 5000;
                    self.time += 15.0;
                    self.arrests += 1;
                }
                Phase::Escaped => {
                    self.banner = "ESCAPED!";
                    self.banner_t = 2.0;
                }
                Phase::Catch => {}
            }
        }
        if p.over() {
            self.pursuit_done = true;
            self.pursuit = None;
        } else {
            self.pursuit = Some(p);
        }
    }
}
