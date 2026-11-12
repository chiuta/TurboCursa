//! Daily challenge: one stage, one vehicle, one set of conditions for everybody on a given UTC day.
//!
//! The first run of the day counts and earns a bonus; later runs are practice. The result code lets
//! friends compare. The check digits only catch typing mistakes - this is not anti-cheat.
use crate::career::splitmix;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Daily {
    pub day: u32,
    pub seed: u64,
    /// game mode index (see `game::Mode::from_index`)
    pub mode: u32,
    pub class: usize,
    /// difficulty level 1..=3
    pub level: usize,
    /// menu indices: 0 = automatic, k = item k - 1
    pub weather: usize,
    pub surface: usize,
    /// 1 dawn, 2 day, 3 dusk, 4 night
    pub time: usize,
}

/// Modes that make a fair one-attempt challenge.
const MODES: [u32; 5] = [0, 1, 3, 4, 5];

pub fn for_day(day: u32) -> Daily {
    let h = splitmix(day as u64 ^ 0xDA11_7C4A_11E5);
    let pick = |salt: u64, n: usize| (splitmix(h ^ salt) % n as u64) as usize;
    Daily {
        day,
        seed: 100_000 + day as u64,
        mode: MODES[pick(1, MODES.len())],
        class: pick(2, crate::cars::CLASS_COUNT),
        level: 1 + pick(3, 3),
        weather: if pick(4, 3) == 0 {
            0
        } else {
            1 + pick(5, crate::env::WEATHER_COUNT)
        },
        surface: if pick(6, 3) == 0 {
            0
        } else {
            1 + pick(7, crate::env::SURFACE_COUNT)
        },
        time: 1 + pick(8, 4),
    }
}

fn check(day: u32, score: u32, secs10: u32, finished: bool) -> u32 {
    (splitmix(
        ((day as u64) << 40)
            ^ ((score as u64) << 8)
            ^ (secs10 as u64) << 1
            ^ finished as u64
            ^ 0xC0DE,
    ) & 0xFFF) as u32
}

/// "TC-<day>-<score>-<seconds*10>-<F|T>-<check>"
pub fn code(day: u32, score: u32, secs: f32, finished: bool) -> String {
    let s10 = if secs.is_finite() {
        (secs.max(0.0) * 10.0).min(9_999_999.0) as u32
    } else {
        0
    };
    format!(
        "TC-{}-{}-{}-{}-{:03X}",
        day,
        score,
        s10,
        if finished { 'F' } else { 'T' },
        check(day, score, s10, finished)
    )
}

/// (day, score, seconds, finished) when the code is well formed and its check digits match.
pub fn parse_code(s: &str) -> Option<(u32, u32, f32, bool)> {
    let s = s.trim();
    if s.len() > 64 {
        return None;
    }
    let p: Vec<&str> = s.split('-').collect();
    if p.len() != 6 || p[0] != "TC" {
        return None;
    }
    let day: u32 = p[1].parse().ok()?;
    let score: u32 = p[2].parse().ok()?;
    let s10: u32 = p[3].parse().ok()?;
    let fin = match p[4] {
        "F" => true,
        "T" => false,
        _ => return None,
    };
    let c = u32::from_str_radix(p[5], 16).ok()?;
    (c == check(day, score, s10, fin)).then_some((day, score, s10 as f32 / 10.0, fin))
}

/// Weekly challenge: the same for everybody during one week (UTC, weeks counted from the epoch).
/// Each week applies one modifier on top of a seeded stage; the modifiers rotate.
pub const WEEKLY_MODIFIERS: usize = 6;

pub fn week_of(day: u32) -> u32 {
    day / 7
}

/// Index of the modifier of a week: 0 night and fog, 1 black ice, 2 heavy vehicle, 3 insane
/// difficulty, 4 mirrored steering, 5 blackout.
pub fn weekly_modifier(week: u32) -> usize {
    (splitmix(week as u64 ^ 0x77EE_C1A5) % WEEKLY_MODIFIERS as u64) as usize
}

pub fn weekly(week: u32) -> Daily {
    let mut d = for_day(week.wrapping_mul(7));
    d.seed = 700_000 + week as u64;
    match weekly_modifier(week) {
        0 => {
            d.time = 4;
            d.weather = 1 + 3; // fog
        }
        1 => {
            d.weather = 1 + 8; // black ice
            d.surface = 0;
        }
        2 => {
            d.class = [3, 4, 15][(week % 3) as usize];
            d.level = d.level.max(2);
        }
        3 => d.level = 4,
        4 => d.mode = 6,
        _ => d.mode = 7,
    }
    d.weather = d.weather.min(crate::env::WEATHER_COUNT);
    d.level = d.level.min(crate::game::LEVEL_COUNT - 1);
    d.class = d.class.min(crate::cars::CLASS_COUNT - 1);
    d
}

/// "TW-<week>-<score>-<seconds*10>-<F|T>-<check>": same format as the daily code, own prefix.
pub fn weekly_code(week: u32, score: u32, secs: f32, finished: bool) -> String {
    code(week, score, secs, finished).replacen("TC-", "TW-", 1)
}

pub fn parse_weekly_code(s: &str) -> Option<(u32, u32, f32, bool)> {
    let t = s.trim();
    let rest = t.strip_prefix("TW-")?;
    parse_code(&format!("TC-{rest}"))
}
