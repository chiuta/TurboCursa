//! A visible pack of rivals on the road: 8 to 16 cars on a start grid (normal, inverted or from a
//! qualifying lap), a live race position and a podium bonus. The rivals are simple: each cruises at
//! its own pace, slows in bends and keeps within sight of the player (rubber band).
use crate::career::splitmix;
use crate::track::{Track, SEG_LEN};

pub const MIN_CARS: usize = 8;
pub const MAX_CARS: usize = 16;
const ROW: f32 = 700.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackCfg {
    /// cars on the grid including the player (8..=16)
    pub n: usize,
    /// the player starts in front, the pack behind
    pub inverted: bool,
    /// grid slot of the player, 0 = pole (from qualifying); `None` = the back (or the front when inverted)
    pub slot: Option<usize>,
}

impl PackCfg {
    pub fn sanitized(mut self) -> PackCfg {
        self.n = self.n.clamp(MIN_CARS, MAX_CARS);
        self.slot = self.slot.map(|s| s.min(self.n - 1));
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rival {
    pub z: f32,
    pub x: f32,
    pub speed: f32,
    pub skill: f32,
    pub color: u8,
    phase: f32,
}

/// Pace of rival `i` (fraction of the top speed), spread between 0.50 and 0.68. A stage's
/// reference pace is half the top speed and a careful driver averages 0.45-0.55 of it (measured with
/// the autopilot), so the slowest rivals are within reach and the fastest need a clean, quick run.
pub fn skill(seed: u64, i: usize) -> f32 {
    0.50 + (splitmix(seed ^ ((i as u64 + 1) * 0x51_7C_C1)) % 1000) as f32 / 1000.0 * 0.18
}

/// Qualifying lap of rival `i` for a stage whose reference time is `reference` seconds.
pub fn qual_time(seed: u64, i: usize, reference: f32) -> f32 {
    // the rival's average pace is about 0.9 of its cruising pace (bends); the reference is 0.5
    reference * 0.5 / (skill(seed, i) * 0.9)
}

/// Grid slot (0 = pole) the player earns with a qualifying lap of `secs`.
pub fn slot_for(seed: u64, n: usize, secs: f32, reference: f32) -> usize {
    let n = n.clamp(MIN_CARS, MAX_CARS);
    if !secs.is_finite() {
        return n - 1;
    }
    (0..n - 1)
        .filter(|&i| qual_time(seed, i, reference) < secs)
        .count()
}

/// The rivals on the grid; the player is at `player_z`. Slot k sits k rows from pole (two cars per row).
pub fn grid(seed: u64, cfg: PackCfg, player_z: f32) -> Vec<Rival> {
    let cfg = cfg.sanitized();
    let me = cfg.slot.unwrap_or(if cfg.inverted { 0 } else { cfg.n - 1 });
    let mut v = Vec::with_capacity(cfg.n - 1);
    let mut r = 0usize;
    for slot in 0..cfg.n {
        if slot == me {
            continue;
        }
        // pole is ahead of the player when the player is further back, the grid is two columns wide
        let rows = (slot as f32 / 2.0).floor() - (me as f32 / 2.0).floor();
        let col = if slot % 2 == 0 { -0.4 } else { 0.4 };
        let side = if rows == 0.0 {
            if slot < me {
                350.0
            } else {
                -350.0
            }
        } else {
            0.0
        };
        let dz = -rows * ROW + side;
        let sk = skill(seed, r);
        v.push(Rival {
            z: player_z + dz,
            x: col,
            speed: 0.0,
            skill: sk,
            color: (splitmix(seed ^ r as u64) % 7) as u8,
            phase: (splitmix(seed ^ (r as u64) << 8) % 628) as f32 / 100.0,
        });
        r += 1;
    }
    v
}

impl Rival {
    /// One step: cruise, slow in bends, and stay within sight of the player.
    pub fn update(&mut self, dt: f32, t: f32, track: &Track, player_z: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let dt = dt.min(0.1);
        let seg = (self.z.max(0.0) / SEG_LEN) as usize % track.segs.len().max(1);
        let bend = track.segs.get(seg).map_or(0.0, |s| s.curve.abs());
        let mut want = self.skill * crate::game::MAX_SPEED * (1.0 - (bend * 0.07).min(0.28));
        let gap = self.z - player_z;
        if gap < -14000.0 {
            want *= 1.10;
        } else if gap > 14000.0 {
            want *= 0.92;
        }
        self.speed += (want - self.speed) * (dt * 0.8).min(1.0);
        self.z += self.speed * dt;
        self.x = (if self.x < 0.0 { -0.4 } else { 0.4 } + 0.3 * (t * 0.5 + self.phase).sin())
            .clamp(-0.85, 0.85);
    }
}

/// Race position: 1 + the rivals in front of `player_z`.
pub fn position(rivals: &[Rival], player_z: f32) -> usize {
    1 + rivals.iter().filter(|r| r.z > player_z).count()
}

/// Score for the final place.
pub fn podium_bonus(pos: usize) -> u32 {
    match pos {
        1 => 5000,
        2 => 3000,
        3 => 1500,
        _ => 0,
    }
}
