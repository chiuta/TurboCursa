use super::*;

impl Game {
    pub(super) fn make_cars(
        track: &Track,
        seed: u64,
        mode: Mode,
        level: Difficulty,
        rank: u16,
    ) -> Vec<Car> {
        let (n, lo, hi) = match mode {
            Mode::TimeTrial | Mode::Zen => (0usize, 0.0, 0.0),
            Mode::Rush => (96, 0.34, 0.70),
            _ => (34, 0.28, 0.66),
        };
        let rf = rank.min(999) as f32 / 999.0;
        let (_, nmul, smul, _) = level.params();
        let (nmul, smul) = (nmul * (1.0 + 0.8 * rf), smul * (1.0 + 0.10 * rf));
        let n = (n as f32 * nmul).round() as usize;
        let hi = (hi * smul).min(0.80);
        let lo = (lo * smul).min(0.75);
        let mut rng = Rng::new(seed);
        let mut cars = Vec::new();
        for i in 0..n {
            let span = track.length - 6000.0;
            let z = 4000.0 + span * (i as f32 + rng.f() * 0.7) / n as f32;
            // vehicle mix: harder levels bring more heavy traffic
            let r = rng.f();
            let h = level.index() as f32 * 0.03;
            let cum = [
                (K_SEMI, 0.07 + h),
                (K_TRUCK, 0.17 + 1.5 * h),
                (K_BUS, 0.23 + 1.5 * h),
                (K_TRACTOR, 0.28 + 1.5 * h),
                (K_MICRO, 0.40 + 1.5 * h),
                (K_BIKE, 0.52 + 1.5 * h),
                (K_TRIKE, 0.58 + 1.5 * h),
            ];
            let kind = cum.iter().find(|(_, t)| r < *t).map_or(K_CAR, |(k, _)| *k);
            let km = match kind {
                K_SEMI => 0.78,
                K_TRUCK => 0.9,
                K_BUS => 0.85,
                K_TRACTOR => 0.55,
                K_TRIKE => 0.8,
                K_BIKE => 1.08,
                _ => 1.0,
            };
            let base = MAX_SPEED * rng.range(lo, hi) * km;
            cars.push(Car {
                kind,
                prev_dz: wrap(z - PLAYER_Z, track.length),
                z,
                x: LANES[rng.below(3) as usize],
                speed: base,
                base,
                color: rng.below(7) as u8,
            });
        }
        cars
    }

    /// Obstacles and bonuses scattered along the stage (none in Zen).
    pub(super) fn make_items(
        track: &Track,
        seed: u64,
        mode: Mode,
        level: Difficulty,
        surface: crate::env::Surface,
        rank: u16,
    ) -> Vec<Item> {
        if mode == Mode::Zen {
            return Vec::new();
        }
        let li = level.index();
        let omul = [0.5, 1.0, 1.3, 1.6, 2.0][li] * (1.0 + 0.9 * rank.min(999) as f32 / 999.0);
        let bmul = [1.4, 1.0, 0.85, 0.7, 0.55][li];
        let (no, nb) = match mode {
            Mode::Rush => (70.0, 30.0),
            Mode::TimeTrial => (40.0, 16.0),
            Mode::Collector | Mode::Slalom => (22.0, 24.0),
            _ => (46.0, 26.0),
        };
        let (no, nb) = ((no * omul) as usize, (nb * bmul) as usize);
        let mut rng = Rng::new(seed ^ 0xB0B0_1234);
        let span = track.length - 7000.0;
        let mut v = Vec::with_capacity(no + nb);
        let pick = |rng: &mut Rng, w: &[(u8, f32)]| {
            let tot: f32 = w.iter().map(|x| x.1).sum();
            let mut r = rng.f() * tot;
            for (k, wt) in w {
                if r < *wt {
                    return *k;
                }
                r -= wt;
            }
            w[0].0
        };
        let obst = [
            (I_CONE, 0.25),
            (I_OIL, 0.18),
            (I_BARRIER, 0.15),
            (I_POTHOLE, 0.17),
            (I_PUDDLE, 0.13),
            (I_MUD, 0.12),
        ];
        let bon = [
            (I_TIME, 0.20),
            (I_TURBO, 0.12),
            (I_BOOST, 0.20),
            (I_SHIELD, 0.12),
            (I_COIN, 0.22),
            (I_FREEZE, 0.07),
            (I_DOUBLE, 0.07),
        ];
        for i in 0..no {
            let z = 3500.0 + span * (i as f32 + rng.f() * 0.8) / no.max(1) as f32;
            let mut kind = pick(&mut rng, &obst);
            // the surface decides which hazards are typical
            use crate::env::Surface as S;
            let r = rng.f();
            kind = match (surface, kind) {
                (S::IceLake, I_OIL | I_PUDDLE | I_MUD) => I_CRACK,
                (S::IceLake, _) if r < 0.30 => I_CRACK,
                (S::Lava, I_PUDDLE | I_OIL) => I_MAGMA,
                (S::Lava, _) if r < 0.30 => I_MAGMA,
                (S::SaltFlat, _) if r < 0.30 => I_PUDDLE,
                (S::Dunes | S::Bog, _) if r < 0.30 => I_MUD,
                (_, k) => k,
            };
            let x = if kind == I_BARRIER {
                LANES[rng.below(3) as usize] * 0.8
            } else {
                rng.range(-0.85, 0.85)
            };
            v.push(Item {
                z,
                x,
                kind,
                taken: false,
            });
        }
        for i in 0..nb {
            let z = 3900.0 + span * (i as f32 + rng.f() * 0.8) / nb.max(1) as f32;
            let kind = pick(&mut rng, &bon);
            v.push(Item {
                z,
                x: rng.range(-0.8, 0.8),
                kind,
                taken: false,
            });
        }
        // turbo pads: drive over one for a free turbo
        let np = (6.0 + 4.0 * bmul) as usize;
        for i in 0..np {
            let z = 5200.0 + span * (i as f32 + rng.f() * 0.7) / np as f32;
            if matches!(track.seg_at(z).feat, F_TUNNEL) {
                continue;
            }
            v.push(Item {
                z,
                x: rng.range(-0.5, 0.5),
                kind: I_PAD,
                taken: false,
            });
        }
        // jump ramps in three sizes, only on open road
        let nr = (7.0 + 3.0 * bmul) as usize;
        for i in 0..nr {
            let z = 4200.0 + span * (i as f32 + rng.f() * 0.7) / nr as f32;
            if track.seg_at(z).feat != F_NONE {
                continue;
            }
            let r = rng.f();
            let kind = if r < 0.5 {
                I_RAMP_S
            } else if r < 0.83 {
                I_RAMP_M
            } else {
                I_RAMP_L
            };
            v.push(Item {
                z,
                x: rng.range(-0.55, 0.55),
                kind,
                taken: false,
            });
        }
        // inside a mountain pass everything must lie between the walls
        for it in &mut v {
            let lim = feat_limit(track.seg_at(it.z).feat);
            if lim < 5.0 {
                it.x = it.x.clamp(-(lim - 0.25), lim - 0.25);
            }
        }
        v.sort_by(|a, b| a.z.total_cmp(&b.z));
        v
    }

    /// Animals crossing the road; none in Zen, on bridges, viaducts or in tunnels.
    pub(super) fn make_animals(
        track: &Track,
        seed: u64,
        mode: Mode,
        level: Difficulty,
        surface: crate::env::Surface,
        rank: u16,
    ) -> Vec<Animal> {
        if mode == Mode::Zen {
            return Vec::new();
        }
        let omul = [0.5, 1.0, 1.3, 1.6, 2.0][level.index()] * (1.0 + rank.min(999) as f32 / 999.0);
        let n = (16.0 * omul) as usize;
        let mut rng = Rng::new(seed ^ 0xA11AA);
        let span = track.length - 8000.0;
        let mut v = Vec::with_capacity(n);
        use crate::env::Surface as S;
        for i in 0..n {
            let z = 4400.0 + span * (i as f32 + rng.f() * 0.7) / n as f32;
            let seg = track.seg_at(z);
            if matches!(seg.feat, F_TUNNEL | F_BRIDGE | F_VIADUCT) {
                continue;
            }
            let pool: &[u8] = match (surface, seg.biome) {
                (S::IceLake, _) => &[5, 5, 2, 3],
                (S::Dunes | S::SaltFlat, _) => &[4, 4, 2, 7],
                (S::Bog, _) => &[6, 1, 0, 3],
                (S::Lava, _) => &[2, 6],
                (_, 1 | 4 | 18) => &[4, 2, 7, 6],
                (_, 0 | 2 | 3 | 7) => &[0, 1, 6, 3, 2],
                _ => &[0, 2, 6, 7, 1],
            };
            let species = pool[rng.below(pool.len() as u32) as usize];
            let side = if rng.f() < 0.5 { -1.0 } else { 1.0 };
            v.push(Animal {
                z,
                x: side * 1.4,
                dir: -side,
                species,
                lead: 1.3 + rng.f() * 1.3,
                active: false,
                done: false,
                hit: false,
                passed: false,
                hit_t: 0.0,
            });
        }
        v
    }

    /// Railway crossings: a few per stage, clear of structures, stations and checkpoints.
    pub(super) fn make_crossings(
        track: &Track,
        seed: u64,
        stations: &[(f32, f32)],
    ) -> Vec<Crossing> {
        let mut rng = Rng::new(seed ^ 0x7A11);
        let total = track.finish_seg;
        let mut v = Vec::new();
        for f in [0.22f32, 0.40, 0.58, 0.74, 0.88] {
            let mut s = (total as f32 * (f + rng.range(-0.02, 0.02))) as usize;
            let mut guard = 0;
            loop {
                let z = s as f32 * SEG_LEN;
                let ok = (s.saturating_sub(10)..s + 10)
                    .all(|i| track.segs[i % track.segs.len()].feat == F_NONE)
                    && track.checkpoints.iter().all(|&c| c + 8 < s || c > s + 8)
                    && stations
                        .iter()
                        .all(|&(a, b)| z + 6000.0 < a || z > b + 3000.0);
                if ok || guard > 300 || s + 40 >= total {
                    break;
                }
                s += 7;
                guard += 1;
            }
            if s + 40 < total {
                v.push(Crossing {
                    z: s as f32 * SEG_LEN,
                    phase: rng.range(0.0, 16.0),
                    period: rng.range(13.0, 18.0),
                    dur: 3.2,
                });
            }
        }
        v
    }

    /// Fuel stations, spread along the stage (a tank does not last for all of it).
    pub(super) fn make_stations(track: &Track, seed: u64) -> Vec<(f32, f32)> {
        let mut rng = Rng::new(seed ^ 0xF0E1);
        let total = track.finish_seg;
        let mut v: Vec<(f32, f32)> = Vec::new();
        for f in [0.16f32, 0.32, 0.48, 0.64, 0.80] {
            let mut s = (total as f32 * (f + rng.range(-0.02, 0.02))) as usize;
            let mut guard = 0;
            loop {
                let ok = (s..s + 16).all(|i| track.segs[i % track.segs.len()].feat == F_NONE)
                    && track.checkpoints.iter().all(|&c| c + 14 < s || c > s + 30);
                if ok || guard > 300 || s + 40 >= total {
                    break;
                }
                s += 8;
                guard += 1;
            }
            if s + 40 < total && v.last().is_none_or(|l| (s as f32 * SEG_LEN) > l.1 + 6000.0) {
                v.push((s as f32 * SEG_LEN, (s + 14) as f32 * SEG_LEN));
            }
        }
        v
    }

    /// Thin-ice zones of a frozen-lake stage: stopping on them breaks the ice.
    pub(super) fn make_thin(
        track: &Track,
        seed: u64,
        surface: crate::env::Surface,
    ) -> Vec<(f32, f32)> {
        if surface != crate::env::Surface::IceLake {
            return Vec::new();
        }
        let mut rng = Rng::new(seed ^ 0x1CE_1CE);
        let n = 6;
        let span = track.length - 9000.0;
        (0..n)
            .map(|i| {
                let a = 4500.0 + span * (i as f32 + rng.f() * 0.45) / n as f32;
                (a, a + rng.range(5000.0, 10500.0))
            })
            .collect()
    }
}
