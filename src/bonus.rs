//! Announced bonus areas along a stage: a proving ground (slalom, slope, rings, ramp) and parking
//! lots beside the road. Derived from the seed; nothing here changes stage codes.
use crate::city::CityPlan;
use crate::mission::Mission;
use crate::track::{Track, F_NONE, SEG_LEN};

pub const PROVING: u8 = 0;
pub const PARKING: u8 = 1;

pub const PROVING_LEN: usize = 150;
pub const PARKING_LEN: usize = 44;
/// Rings to collect for the "track clear" bonus.
pub const RINGS_NEEDED: u32 = 8;
/// Height of the hill in the proving ground, in world units, and its length in segments.
pub const HUMP_H: f32 = 1400.0;
pub const HUMP_FROM: usize = 34;
pub const HUMP_LEN: usize = 80;
/// Lateral centres of the two columns of parking slots, and the lot's inner edge.
pub const SLOTS: [f32; 2] = [1.40, 1.90];
pub const LOT_EDGE: f32 = 1.12;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    pub kind: u8,
    pub a: usize,
    pub b: usize,
}

impl Area {
    pub fn az(&self) -> f32 {
        self.a as f32 * SEG_LEN
    }
    pub fn bz(&self) -> f32 {
        self.b as f32 * SEG_LEN
    }
}

pub fn plan(
    track: &Track,
    seed: u64,
    city: &CityPlan,
    stations: &[(f32, f32)],
    crossings: &[f32],
    mission: Option<&Mission>,
) -> Vec<Area> {
    let total = track.finish_seg;
    let mut rng = crate::rng::Rng::new(seed ^ 0xB0175);
    let mut out: Vec<Area> = Vec::new();
    let mut want = vec![
        (PROVING, 0.46f32, PROVING_LEN),
        (PARKING, 0.12, PARKING_LEN),
        (PARKING, 0.80, PARKING_LEN),
    ];
    for (kind, f, len) in want.drain(..) {
        let f = f + rng.range(-0.03, 0.03);
        let c = (total as f32 * f) as i64;
        'search: for step in 0..500i64 {
            for sgn in [1i64, -1] {
                let a = c + sgn * step * 5;
                if a < 150 {
                    continue;
                }
                let (a, b) = (a as usize, a as usize + len);
                if b + 60 >= total {
                    continue;
                }
                let za = a as f32 * SEG_LEN;
                let zb = b as f32 * SEG_LEN;
                let ok = (a.saturating_sub(12)..b + 12)
                    .all(|i| track.segs[i % track.segs.len()].feat == F_NONE)
                    && track.checkpoints.iter().all(|&k| k + 10 < a || k > b + 10)
                    && stations
                        .iter()
                        .all(|&(s0, s1)| zb + 4000.0 < s0 || za > s1 + 4000.0)
                    && crossings
                        .iter()
                        .all(|&k| k + 3000.0 < za || k - 3000.0 > zb)
                    && !city.zones.iter().any(|z| b + 20 >= z.a && a <= z.b + 20)
                    && !city
                        .jams
                        .iter()
                        .any(|j| b + 20 >= j.exit_a() && a <= j.b + 40)
                    && city
                        .scenes
                        .iter()
                        .all(|&s| zb + 3000.0 < s || za > s + crate::city::SCENE_LEN + 3000.0)
                    && mission.is_none_or(|m| b + 20 < m.pick || a > m.pick + 20)
                    && mission.is_none_or(|m| b + 20 < m.drop || a > m.drop + 20)
                    && out.iter().all(|o| b + 30 < o.a || a > o.b + 30);
                if ok {
                    out.push(Area { kind, a, b });
                    break 'search;
                }
            }
        }
    }
    out.sort_by_key(|o| o.a);
    out
}

/// Height added to the road at the segment boundary `i` of a proving ground starting at `a`.
pub fn hump(a: usize, i: usize) -> f32 {
    let from = a + HUMP_FROM;
    if i <= from || i >= from + HUMP_LEN {
        return 0.0;
    }
    let t = (i - from) as f32 / HUMP_LEN as f32;
    HUMP_H * (std::f32::consts::PI * t).sin().powi(2)
}
