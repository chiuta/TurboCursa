//! Garage: coins earned by driving buy permanent upgrades for every vehicle. Four upgrades, three
//! levels each. The daily and the weekly challenge and shared stage codes always use the stock car.
use std::sync::atomic::{AtomicU8, Ordering};

pub const UPGRADES: usize = 4;
pub const MAX_LEVEL: u8 = 3;
/// Engine (top speed), Gearbox (acceleration), Tyres (grip), Turbo (charges).
pub const NAMES: [&str; UPGRADES] = ["ENGINE", "GEARBOX", "TYRES", "TURBO"];
const COSTS: [u32; MAX_LEVEL as usize] = [40, 110, 260];

static LEVELS: [AtomicU8; UPGRADES] = [
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
];
static STOCK: AtomicU8 = AtomicU8::new(0);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Garage {
    pub coins: u32,
    pub lv: [u8; UPGRADES],
}

impl Garage {
    pub fn sanitized(mut self) -> Garage {
        self.coins = self.coins.min(9_999_999);
        for l in &mut self.lv {
            *l = (*l).min(MAX_LEVEL);
        }
        self
    }

    /// Price of the next level of upgrade `i`, `None` when it is maxed out (or `i` is invalid).
    pub fn price(&self, i: usize) -> Option<u32> {
        let l = *self.lv.get(i)? as usize;
        COSTS.get(l).copied()
    }

    /// Buys the next level of upgrade `i`; false when it is maxed out or too expensive.
    pub fn buy(&mut self, i: usize) -> bool {
        match self.price(i) {
            Some(p) if p <= self.coins => {
                self.coins -= p;
                self.lv[i] += 1;
                true
            }
            _ => false,
        }
    }

    /// Coins a finished game brings in: distance, overtakes, bonuses and a finish prize, scaled by
    /// the difficulty. A disqualified game brings nothing.
    pub fn coins_for(r: &crate::profile::GameResult) -> u32 {
        if r.disqualified {
            return 0;
        }
        let base = r.distance_frac.clamp(0.0, 1.0) * 10.0
            + r.overtaken.min(40) as f32 / 4.0
            + r.bonuses.min(20) as f32 / 2.0
            + if r.finished { 12.0 } else { 0.0 };
        (base * 2.0 * (1.0 + r.level.min(4) as f32 * 0.25)) as u32
    }

    pub fn earn(&mut self, r: &crate::profile::GameResult) -> u32 {
        let c = Garage::coins_for(r);
        self.coins = (self.coins + c).min(9_999_999);
        c
    }

    pub fn encode(&self) -> String {
        let g = self.sanitized();
        format!(
            "TG2;c={};u={}{}{}{}",
            g.coins, g.lv[0], g.lv[1], g.lv[2], g.lv[3]
        )
    }

    pub fn decode(s: &str) -> Garage {
        let mut g = Garage::default();
        let s = s.trim();
        if s.len() > 60 || !s.starts_with("TG2;") {
            return g;
        }
        for kv in s.split(';').skip(1) {
            match kv.split_once('=') {
                Some(("c", v)) => g.coins = v.parse().unwrap_or(0),
                Some(("u", v)) => {
                    for (i, ch) in v.chars().take(UPGRADES).enumerate() {
                        g.lv[i] = ch.to_digit(10).unwrap_or(0) as u8;
                    }
                }
                _ => {}
            }
        }
        g.sanitized()
    }

    /// Makes the upgrades take effect in `Game::set_car`.
    pub fn apply(&self) {
        for (i, l) in self.sanitized().lv.iter().enumerate() {
            LEVELS[i].store(*l, Ordering::Relaxed);
        }
    }

    pub fn status(&self) -> String {
        (0..UPGRADES)
            .map(|i| match self.price(i) {
                Some(p) => format!(
                    "{} {}/{} (next {} coins)",
                    NAMES[i], self.lv[i], MAX_LEVEL, p
                ),
                None => format!("{} MAX", NAMES[i]),
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

/// Stock mode: the upgrades are ignored (daily, weekly, shared stage codes).
pub fn set_stock(on: bool) {
    STOCK.store(on as u8, Ordering::Relaxed);
}

pub fn level(i: usize) -> u8 {
    if STOCK.load(Ordering::Relaxed) != 0 {
        return 0;
    }
    LEVELS.get(i).map_or(0, |l| l.load(Ordering::Relaxed))
}

/// Adds the installed upgrades to a car's multipliers (before `set_car` clamps them).
pub fn upgrade(mut c: crate::cars::CarSpec) -> crate::cars::CarSpec {
    if c.tuned {
        return c;
    }
    c.tuned = true;
    c.top *= 1.0 + 0.03 * level(0) as f32;
    c.accel *= 1.0 + 0.05 * level(1) as f32;
    c.grip *= 1.0 + 0.04 * level(2) as f32;
    c.turbos = c
        .turbos
        .saturating_add(level(3) / 2 + (level(3) == 1) as u8);
    c
}
