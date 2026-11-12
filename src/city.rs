//! City stretches (boulevards and streets with traffic lights, crosswalks, pedestrians and a speed
//! limit) and traffic jams with a detour exit. Everything is derived from the stage seed, so a stage
//! code always yields the same cities and jams; nothing here changes the track itself.
use crate::rng::Rng;
use crate::track::{Track, F_NONE, SEG_LEN};

/// Traffic-light cycle in seconds: green, yellow, red.
pub const GREEN: f32 = 5.5;
pub const YELLOW: f32 = 1.5;
pub const RED: f32 = 4.0;
pub const CYCLE: f32 = GREEN + YELLOW + RED;
/// Pedestrians walk during the red phase, after this delay and for this long.
const WALK_DELAY: f32 = 0.5;
const WALK_DUR: f32 = 3.0;

/// City speed limit as a fraction of the top speed.
pub const LIMIT: f32 = 0.50;
/// Average speed of the queue on the main road, in units per second.
pub const JAM_CRAWL: f32 = 1500.0;
/// Speed cap on the detour, as a fraction of the top speed.
pub const DETOUR_CAP: f32 = 0.72;
/// Segments of slip road before the jam where the exit can be taken.
pub const EXIT_LEN: usize = 60;

#[derive(Clone, Copy, Debug)]
pub struct Zone {
    pub a: usize,
    pub b: usize,
}

/// A crosswalk, with or without a traffic light.
#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub z: f32,
    pub phase: f32,
    /// true: signalled crossing; false: plain zebra, pedestrians cross on their own timing
    pub lit: bool,
    pub dir: f32,
    pub period: f32,
}

impl Light {
    fn t(&self, elapsed: f32) -> f32 {
        (elapsed + self.phase).rem_euclid(if self.lit { CYCLE } else { self.period })
    }
    /// 0 green, 1 yellow, 2 red (zebra crossings are always 0).
    pub fn state(&self, elapsed: f32) -> u8 {
        if !self.lit {
            return 0;
        }
        let t = self.t(elapsed);
        if t < GREEN {
            0
        } else if t < GREEN + YELLOW {
            1
        } else {
            2
        }
    }
    /// Pedestrians' lateral position (road half-widths) while they are on the road or waiting at
    /// the kerb; `None` when nobody is there.
    pub fn ped_x(&self, elapsed: f32) -> Option<f32> {
        let t = self.t(elapsed);
        let (walk_t, dur) = if self.lit {
            (t - (GREEN + YELLOW + WALK_DELAY), WALK_DUR)
        } else {
            (t, WALK_DUR)
        };
        if walk_t < 0.0 {
            // waiting at the kerb shortly before the walk
            return if walk_t > -2.5 {
                Some(-self.dir * 1.32)
            } else {
                None
            };
        }
        if walk_t > dur {
            return None;
        }
        let p = walk_t / dur;
        Some(-self.dir * 1.32 + self.dir * 2.64 * p)
    }
    /// A pedestrian is on the carriageway.
    pub fn ped_on_road(&self, elapsed: f32) -> bool {
        self.ped_x(elapsed).is_some_and(|x| x.abs() < 1.1)
    }
}

/// A traffic jam on the main road with an exit onto a free parallel street.
#[derive(Clone, Copy, Debug)]
pub struct Jam {
    pub a: usize,
    pub b: usize,
}

impl Jam {
    /// First segment of the slip road.
    pub fn exit_a(&self) -> usize {
        self.a.saturating_sub(EXIT_LEN)
    }
    pub fn len_units(&self) -> f32 {
        (self.b - self.a) as f32 * SEG_LEN
    }
    /// Seconds to cross the jam on the main road.
    pub fn eta_main(&self) -> f32 {
        self.len_units() / JAM_CRAWL
    }
    /// Seconds to cross the same stretch on the detour.
    pub fn eta_exit(&self) -> f32 {
        self.len_units() / (crate::game::MAX_SPEED * DETOUR_CAP) + 1.5
    }
}

#[derive(Clone, Debug, Default)]
pub struct CityPlan {
    pub zones: Vec<Zone>,
    pub lights: Vec<Light>,
    pub jams: Vec<Jam>,
    /// accident scenes (z of the first cone): a helicopter has landed on the road, with an
    /// ambulance and a fire engine; the left lane stays open
    pub scenes: Vec<f32>,
}

/// Objects of an accident scene, relative to its z: (dz, x, half-width, kind) with kind 0 landed
/// helicopter, 1 ambulance, 2 fire engine.
pub const SCENE: [(f32, f32, f32, u8); 3] = [
    (500.0, 0.70, 0.24, 0),
    (1300.0, 0.93, 0.14, 1),
    (1800.0, 0.56, 0.2, 2),
];
pub const SCENE_LEN: f32 = 2600.0;

impl CityPlan {
    pub fn zone_at(&self, seg: usize) -> Option<usize> {
        self.zones.iter().position(|z| seg >= z.a && seg < z.b)
    }
    pub fn in_city(&self, seg: usize) -> bool {
        self.zone_at(seg).is_some()
    }
    pub fn jam_at(&self, seg: usize) -> Option<usize> {
        self.jams.iter().position(|j| seg >= j.a && seg < j.b)
    }
    /// Slip road in front of a jam.
    pub fn exit_at(&self, seg: usize) -> Option<usize> {
        self.jams
            .iter()
            .position(|j| seg >= j.exit_a() && seg < j.a)
    }
    /// The next jam whose end is still ahead of this segment.
    pub fn next_jam(&self, seg: usize) -> Option<(usize, &Jam)> {
        self.jams.iter().enumerate().find(|(_, j)| j.b > seg)
    }
}

/// Where the cities and jams go: clear of structures, checkpoints, fuel stations and railway
/// crossings, and clear of each other.
pub fn plan(track: &Track, seed: u64, stations: &[(f32, f32)], crossings: &[f32]) -> CityPlan {
    let mut rng = Rng::new(seed ^ 0xC17E);
    let total = track.finish_seg;
    let mut plan = CityPlan::default();
    let fits = |a: usize, b: usize, plan: &CityPlan| -> bool {
        if b + 30 >= total || a < 120 {
            return false;
        }
        (a.saturating_sub(4)..b + 24).all(|i| track.segs[i % track.segs.len()].feat == F_NONE)
            && track.checkpoints.iter().all(|&c| c + 6 < a || c > b + 6)
            && stations.iter().all(|&(s0, s1)| {
                (b as f32 + 20.0) * SEG_LEN < s0 || (a as f32 - 20.0) * SEG_LEN > s1
            })
            && crossings
                .iter()
                .all(|&z| z < (a as f32 - 10.0) * SEG_LEN || z > (b as f32 + 10.0) * SEG_LEN)
            && plan.zones.iter().all(|z| b + 60 < z.a || a > z.b + 60)
            && plan
                .jams
                .iter()
                .all(|j| b + 90 < j.exit_a() || a.saturating_sub(EXIT_LEN) > j.b + 60)
    };
    // a spot near the wished fraction: search outwards, earlier and later
    let find = |plan: &CityPlan, f: f32, len: usize| -> Option<usize> {
        let c = (total as f32 * f) as usize;
        for step in 0..260usize {
            for sgn in [1i64, -1] {
                let a = c as i64 + sgn * (step as i64 * 7);
                if a > 0 && fits(a as usize, a as usize + len, plan) {
                    return Some(a as usize);
                }
            }
        }
        None
    };
    // two cities
    for f in [0.30f32, 0.68] {
        let f = f + rng.range(-0.03, 0.03);
        for len in [280usize, 220, 170] {
            if let Some(a) = find(&plan, f, len) {
                plan.zones.push(Zone { a, b: a + len });
                break;
            }
        }
    }
    // one or two jams
    let count = 1 + (rng.f() < 0.55) as usize;
    let fr: [f32; 3] = [0.14, 0.49, 0.85];
    let mut order = [0usize, 1, 2];
    for i in (1..3).rev() {
        order.swap(i, rng.below(i as u32 + 1) as usize);
    }
    for &k in order.iter() {
        if plan.jams.len() >= count {
            break;
        }
        let len = 150usize + rng.below(40) as usize;
        let f = fr[k] + rng.range(-0.02, 0.02);
        if let Some(a) = find(&plan, f, len) {
            plan.jams.push(Jam { a, b: a + len });
        }
    }
    // accident scenes: one at the head of each jam, one more clear of everything else
    for j in plan.jams.clone() {
        plan.scenes.push((j.b + 6) as f32 * SEG_LEN);
    }
    if plan.scenes.len() < 2 {
        let f = 0.58 + rng.range(-0.03, 0.03);
        if let Some(a) = find(&plan, f, 24) {
            plan.scenes.push(a as f32 * SEG_LEN);
        }
    }
    plan.scenes.sort_by(|a, b| a.total_cmp(b));
    plan.jams.sort_by_key(|j| j.a);
    plan.zones.sort_by_key(|z| z.a);
    // crosswalks: alternate signalled and plain ones along each city street
    for z in plan.zones.clone() {
        let mut s = z.a + 38;
        let mut i = 0;
        while s + 20 < z.b {
            plan.lights.push(Light {
                z: s as f32 * SEG_LEN,
                phase: rng.range(0.0, CYCLE),
                lit: i % 3 != 2,
                dir: if rng.f() < 0.5 { -1.0 } else { 1.0 },
                period: rng.range(8.0, 11.0),
            });
            s += 52;
            i += 1;
        }
    }
    plan
}

/// Cars of the standing queue: rows of vehicles on five lanes with a winding free corridor, so a
/// careful driver can always creep through. Returns (z, x, kind, color) tuples.
pub fn jam_cars(track_len: f32, jam: &Jam, seed: u64) -> Vec<(f32, f32, u8, u8)> {
    let mut rng = Rng::new(seed ^ 0x7A3 ^ (jam.a as u64) << 8);
    let lanes = [-0.8f32, -0.4, 0.0, 0.4, 0.8];
    let mut corridor = 2i32;
    let mut v = Vec::new();
    let mut s = jam.a as f32 + 2.0;
    while s < jam.b as f32 - 2.0 {
        let r = rng.f();
        if r < 0.25 {
            corridor = (corridor - 1).max(0);
        } else if r < 0.5 {
            corridor = (corridor + 1).min(4);
        }
        for (li, &x) in lanes.iter().enumerate() {
            if li as i32 == corridor || rng.f() < 0.30 {
                continue;
            }
            let kind = if rng.f() < 0.16 {
                crate::game::K_BIKE
            } else if rng.f() < 0.12 {
                crate::game::K_MICRO
            } else {
                crate::game::K_CAR
            };
            let z = (s + rng.range(-0.4, 0.4)) * SEG_LEN;
            if z < track_len {
                v.push((z, x, kind, rng.below(7) as u8));
            }
        }
        s += 2.6 + rng.range(0.0, 0.8);
    }
    v
}
