//! Delivery missions: collect a parcel or a passenger at a lay-by on the right and take it to the
//! destination further along the stage, within a time limit and with care. Derived from the seed.
use crate::city::{CityPlan, EXIT_LEN, SCENE_LEN};
use crate::track::{Track, F_NONE, SEG_LEN};

pub const PARCEL: u8 = 0;
pub const PASSENGER: u8 = 1;

/// Longest speed at which the vehicle counts as stopped for loading and unloading.
pub const STOP_SPEED: f32 = 2200.0;
/// Lateral position of the lay-by (must be this far to the right) and the half-length of the bay.
pub const BAY_X: f32 = 0.25;
pub const BAY_LEN: f32 = 1500.0;
/// Seconds spent stopped in the bay to load or unload.
pub const DWELL: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mission {
    pub kind: u8,
    /// segment of the pick-up bay and of the drop-off bay
    pub pick: usize,
    pub drop: usize,
    /// seconds allowed between loading and unloading
    pub limit: f32,
}

impl Mission {
    pub fn pick_z(&self) -> f32 {
        self.pick as f32 * SEG_LEN
    }
    pub fn drop_z(&self) -> f32 {
        self.drop as f32 * SEG_LEN
    }
}

/// Phases of a mission.
pub const WAITING: u8 = 0;
pub const CARRYING: u8 = 1;
pub const DONE: u8 = 2;
pub const FAILED: u8 = 3;

pub fn plan(
    track: &Track,
    seed: u64,
    city: &CityPlan,
    stations: &[(f32, f32)],
    crossings: &[f32],
) -> Option<Mission> {
    let total = track.finish_seg;
    let mut rng = crate::rng::Rng::new(seed ^ 0xD3117E5);
    let clear = |s: usize| -> bool {
        if s < 150 || s + 60 >= total {
            return false;
        }
        let z = s as f32 * SEG_LEN;
        (s.saturating_sub(12)..s + 12).all(|i| track.segs[i % track.segs.len()].feat == F_NONE)
            && track.checkpoints.iter().all(|&c| c + 10 < s || c > s + 10)
            && stations
                .iter()
                .all(|&(a, b)| z + 4000.0 < a || z > b + 4000.0)
            && crossings.iter().all(|&c| (c - z).abs() > 4000.0)
            && !city.zones.iter().any(|zn| s + 15 >= zn.a && s < zn.b + 15)
            && !city
                .jams
                .iter()
                .any(|j| s + 15 >= j.exit_a() && s < j.b + 15 + EXIT_LEN / 4)
            && city
                .scenes
                .iter()
                .all(|&c| z + 3000.0 < c || z > c + SCENE_LEN + 3000.0)
    };
    let find = |f: f32| -> Option<usize> {
        let c = (total as f32 * f) as i64;
        for step in 0..400i64 {
            for sgn in [1i64, -1] {
                let s = c + sgn * step * 5;
                if s > 0 && clear(s as usize) {
                    return Some(s as usize);
                }
            }
        }
        None
    };
    let pick = find(0.18 + rng.range(-0.03, 0.05))?;
    let drop = find(0.66 + rng.range(-0.04, 0.08))?;
    if drop < pick + 600 {
        return None;
    }
    let dist = (drop - pick) as f32 * SEG_LEN;
    let kind = if rng.f() < 0.5 { PARCEL } else { PASSENGER };
    // a brisk but fair pace (about 55 % of the top speed) plus a margin for the city and the jams
    let limit = (dist / (crate::game::MAX_SPEED * 0.55) + 16.0).round();
    Some(Mission {
        kind,
        pick,
        drop,
        limit,
    })
}
