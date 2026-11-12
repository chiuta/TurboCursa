//! Player profile: experience, rank 0..=999, a daily streak and gentle, honest play-time guidance.
//!
//! Design rules (deliberately not a slot machine): rank grows slowly and smoothly; XP is worth less
//! after half an hour in one day and nothing after ninety minutes; the streak is a small bonus, a
//! missed day costs nothing but the streak counter; the game suggests a break every 25 minutes.
use crate::game::Game;

pub const MAX_RANK: u16 = 999;
/// Minutes of play after which a break is suggested.
pub const BREAK_MINUTES: f32 = 25.0;
/// Daily minutes at which XP is worth 60 %, 30 % and finally nothing.
pub const SOFT_LIMITS: [f32; 3] = [30.0, 60.0, 90.0];

/// XP needed to climb from rank `r` to `r + 1`.
pub fn xp_to_climb(r: u16) -> u64 {
    40 + r as u64 / 2
}

/// Total XP needed to reach rank `r`.
pub fn xp_for_rank(r: u16) -> u64 {
    let r = r.min(MAX_RANK) as u64;
    // sum of 40 + i/2 for i in 0..r
    40 * r + (0..r).map(|i| i / 2).sum::<u64>()
}

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub xp: u64,
    /// day number of the last play (days since 1970-01-01)
    pub day: u32,
    pub minutes_today: f32,
    pub games_today: u16,
    pub streak: u16,
    pub best_streak: u16,
    pub col: crate::badges::Collection,
    /// the day of the last counted daily challenge, its score and the number of counted days
    pub daily_day: u32,
    pub daily_score: u32,
    pub daily_runs: u16,
    /// lifetime counters that unlock the service vehicles: clean cities, deliveries, track/parking bonuses
    pub clean_cities: u16,
    pub deliveries: u16,
    pub skills: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Advice {
    None,
    /// a quarter of an hour of play in one go: stretch, drink some water
    Break,
    /// the daily play budget is used up: come back tomorrow
    Done,
}

/// What one finished game brings in.
#[derive(Clone, Copy, Debug, Default)]
pub struct GameResult {
    pub distance_frac: f32,
    pub finished: bool,
    pub disqualified: bool,
    pub overtaken: u32,
    pub dodged: u32,
    pub jumps: u32,
    pub pads: u32,
    pub close_calls: u32,
    pub bonuses: u32,
    pub crashes: u32,
    pub level: usize,
    pub seconds: f32,
    pub cond: crate::badges::Conditions,
    pub clean_cities: u32,
    pub deliveries: u32,
    pub skills: u32,
}

impl GameResult {
    pub fn from_game(g: &Game) -> GameResult {
        use crate::game::State;
        GameResult {
            distance_frac: (g.pos / (g.track.finish_seg as f32 * crate::track::SEG_LEN))
                .clamp(0.0, 1.0),
            finished: g.state == State::Finished,
            disqualified: g.state == State::Disqualified,
            overtaken: g.overtaken,
            dodged: g.animals_dodged,
            jumps: g.jumps,
            pads: g.pads,
            close_calls: g.close_calls,
            bonuses: g.bonuses,
            crashes: g.crashes,
            level: g.level.index(),
            seconds: g.elapsed,
            clean_cities: g.clean_cities,
            deliveries: g.missions_done,
            skills: (g.tracks_clear + g.parked),
            cond: crate::badges::Conditions {
                class: g.car.class.index() as u8,
                weather: g.weather.index() as u8,
                surface: g.surface.index() as u8,
                time: crate::badges::time_bucket(g.clock_minutes()),
                country: g.car.country,
                fuel_careful: g.fuel_rule && g.fuel > 0.0 && !g.ran_dry,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gain {
    pub xp: u64,
    pub raw: u64,
    /// ranks gained by this game
    pub ranks_up: u16,
    /// multiplier applied by the daily play budget
    pub mult: f32,
    /// the welcome-back bonus of the first game of a day
    pub daily_bonus: u64,
    /// bit mask of the badges earned by this game
    pub badges_new: u32,
}

impl Default for Profile {
    fn default() -> Self {
        Profile::new()
    }
}

impl Profile {
    pub fn new() -> Profile {
        Profile {
            xp: 0,
            day: 0,
            minutes_today: 0.0,
            games_today: 0,
            streak: 0,
            best_streak: 0,
            col: crate::badges::Collection::default(),
            daily_day: 0,
            daily_score: 0,
            daily_runs: 0,
            clean_cities: 0,
            deliveries: 0,
            skills: 0,
        }
    }

    pub fn rank(&self) -> u16 {
        // binary search over the cumulative table
        let (mut lo, mut hi) = (0u16, MAX_RANK);
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            if xp_for_rank(mid) <= self.xp {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        lo
    }

    /// (XP earned inside the current rank, XP the rank needs); the second is 0 at the top.
    pub fn progress(&self) -> (u64, u64) {
        let r = self.rank();
        if r >= MAX_RANK {
            return (0, 0);
        }
        (self.xp - xp_for_rank(r), xp_to_climb(r))
    }

    /// Call when the game starts or a stage ends: rolls the daily counters over.
    pub fn begin_day(&mut self, today: u32) {
        if today == self.day {
            return;
        }
        if self.day != 0 && today == self.day + 1 {
            self.streak = self.streak.saturating_add(1).min(999);
        } else {
            self.streak = 1;
        }
        self.best_streak = self.best_streak.max(self.streak);
        self.day = today;
        self.minutes_today = 0.0;
        self.games_today = 0;
    }

    /// XP multiplier from the minutes already played today.
    pub fn xp_multiplier(&self) -> f32 {
        let m = self.minutes_today;
        if m < SOFT_LIMITS[0] {
            1.0
        } else if m < SOFT_LIMITS[1] {
            0.6
        } else if m < SOFT_LIMITS[2] {
            0.3
        } else {
            0.0
        }
    }

    /// Is the vehicle class free to pick? Base classes always are; the service vehicles are earned.
    pub fn unlocked(&self, class: usize) -> bool {
        let police = self.clean_cities >= 10;
        let amb = self.deliveries >= 8;
        let fire = self.daily_runs >= 3 && self.skills >= 4;
        match class {
            19 => police,
            20 => fire,
            21 => amb,
            22 => police && amb && fire && self.daily_runs >= 7,
            _ => true,
        }
    }

    /// Bit mask of unlocked service vehicles (bit 0 police .. bit 3 helicopter).
    pub fn specials_mask(&self) -> u8 {
        (0..4).fold(0, |m, i| m | (u8::from(self.unlocked(19 + i)) << i))
    }

    /// What the start screen shows.
    pub fn info(&self, session_minutes: f32) -> crate::pick::ProfileInfo {
        let c = &self.col;
        let (xp, next) = self.progress();
        crate::pick::ProfileInfo {
            rank: self.rank(),
            xp,
            next,
            streak: self.streak,
            badges: c.count() as u16,
            classes: c.classes.count_ones() as u8,
            weathers: c.weathers.count_ones() as u8,
            surfaces: c.surfaces.count_ones() as u8,
            countries: c.countries_seen().min(255) as u8,
            specials: self.specials_mask(),
            advice: match self.advice(session_minutes) {
                Advice::None => 0,
                Advice::Break => 1,
                Advice::Done => 2,
            },
        }
    }

    /// Break suggestion for the current uninterrupted session.
    pub fn advice(&self, session_minutes: f32) -> Advice {
        if self.minutes_today >= SOFT_LIMITS[2] {
            Advice::Done
        } else if session_minutes >= BREAK_MINUTES {
            Advice::Break
        } else {
            Advice::None
        }
    }

    /// Books a finished game and returns what it earned.
    pub fn record(&mut self, r: &GameResult, today: u32) -> Gain {
        self.begin_day(today);
        let before = self.rank();
        let mult = self.xp_multiplier();
        let mut raw = 40.0 * r.distance_frac
            + if r.finished { 120.0 } else { 0.0 }
            + 2.0 * r.overtaken.min(40) as f32
            + 6.0 * r.dodged.min(30) as f32
            + 4.0 * r.jumps.min(10) as f32
            + 3.0 * r.pads.min(10) as f32
            + 20.0 * r.close_calls.min(5) as f32
            + 2.0 * r.bonuses.min(30) as f32
            - 10.0 * r.crashes.min(10) as f32;
        raw *= 1.0 + 0.15 * r.level.min(4) as f32;
        if r.disqualified {
            raw *= 0.5;
        }
        let raw = if r.distance_frac > 0.05 {
            raw.max(5.0) as u64
        } else {
            0
        };
        let mut gain = (raw as f32 * mult) as u64;
        let mut daily_bonus = 0;
        if self.games_today == 0 && raw > 0 {
            daily_bonus = 30 + 10 * self.streak.min(7) as u64;
            gain += daily_bonus;
        }
        self.clean_cities = self
            .clean_cities
            .saturating_add(r.clean_cities.min(20) as u16);
        self.deliveries = self.deliveries.saturating_add(r.deliveries.min(20) as u16);
        self.skills = self.skills.saturating_add(r.skills.min(20) as u16);
        self.games_today = self.games_today.saturating_add(1);
        self.minutes_today = (self.minutes_today + r.seconds.max(0.0) / 60.0).min(1440.0);
        self.xp = (self.xp + gain).min(xp_for_rank(MAX_RANK));
        Gain {
            xp: gain,
            raw,
            ranks_up: self.rank() - before,
            mult,
            daily_bonus,
            badges_new: self.col.record(r, self.rank(), self.streak),
        }
    }

    /// Books a daily-challenge run. Only the first run of a day counts: it earns a bonus (scaled by
    /// the daily play budget like every XP) and returns the shareable code; later runs return `None`.
    pub fn record_daily(
        &mut self,
        today: u32,
        score: u32,
        secs: f32,
        finished: bool,
    ) -> Option<(String, u64)> {
        self.begin_day(today);
        if self.daily_day == today {
            return None;
        }
        self.daily_day = today;
        self.daily_score = score;
        self.daily_runs = self.daily_runs.saturating_add(1);
        let bonus = ((if finished { 90.0 } else { 30.0 }) * self.xp_multiplier()) as u64;
        self.xp = (self.xp + bonus).min(xp_for_rank(MAX_RANK));
        Some((crate::daily::code(today, score, secs, finished), bonus))
    }

    pub fn encode(&self) -> String {
        format!(
            "TP1;xp={};day={};min={:.1};g={};st={};bs={};dd={};ds={};dn={};cc={};dv={};sk={};{}",
            self.xp,
            self.day,
            self.minutes_today,
            self.games_today,
            self.streak,
            self.best_streak,
            self.daily_day,
            self.daily_score,
            self.daily_runs,
            self.clean_cities,
            self.deliveries,
            self.skills,
            self.col.encode()
        )
    }

    /// Hostile-safe parser: unknown keys are ignored, every value is bounded.
    pub fn decode(s: &str) -> Profile {
        let mut p = Profile::new();
        let s = s.trim();
        if !s.starts_with("TP1;") || s.len() > 512 {
            return p;
        }
        for kv in s.split(';').skip(1) {
            let Some((k, v)) = kv.split_once('=') else {
                continue;
            };
            match k {
                "xp" => p.xp = v.parse::<u64>().unwrap_or(0).min(xp_for_rank(MAX_RANK)),
                "day" => p.day = v.parse::<u32>().unwrap_or(0).min(4_000_000),
                "min" => {
                    let m = v.parse::<f32>().unwrap_or(0.0);
                    p.minutes_today = if m.is_finite() {
                        m.clamp(0.0, 1440.0)
                    } else {
                        0.0
                    };
                }
                "g" => p.games_today = v.parse::<u16>().unwrap_or(0).min(5000),
                "st" => p.streak = v.parse::<u16>().unwrap_or(0).min(999),
                "bs" => p.best_streak = v.parse::<u16>().unwrap_or(0).min(999),
                "dd" => p.daily_day = v.parse::<u32>().unwrap_or(0).min(4_000_000),
                "ds" => p.daily_score = v.parse::<u32>().unwrap_or(0).min(100_000_000),
                "dn" => p.daily_runs = v.parse::<u16>().unwrap_or(0).min(60_000),
                "cc" => p.clean_cities = v.parse::<u16>().unwrap_or(0).min(60_000),
                "dv" => p.deliveries = v.parse::<u16>().unwrap_or(0).min(60_000),
                "sk" => p.skills = v.parse::<u16>().unwrap_or(0).min(60_000),
                _ => p.col.decode_kv(k, v),
            }
        }
        p.best_streak = p.best_streak.max(p.streak);
        p
    }
}

/// Today's day number from the system clock (UTC days since 1970-01-01).
#[cfg(not(target_arch = "wasm32"))]
pub fn today() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(1, |d| (d.as_secs() / 86_400) as u32)
        .max(1)
}
