//! Player settings that are not part of the profile: music style, speed unit (km/h or mph), the
//! keyboard layout of the native build and the secret paint jobs unlocked by codes.
//! Stored as one short line, `TS1;m=0;u=0;k=0;l=0`; every value is clamped on load.
use std::sync::atomic::{AtomicU8, Ordering};

pub const KEY_SCHEMES: usize = 4;
pub const LIVERIES: usize = 6;

static MPH: AtomicU8 = AtomicU8::new(0);
static LIVERY: AtomicU8 = AtomicU8::new(0);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// 0 normal, 1 calm, 2 drive, 3 off
    pub music: u8,
    pub mph: bool,
    /// 0 arrows + WASD, 1 WASD + Space, 2 IJKL + Space, 3 numeric keypad 8/4/6/2 + 0
    pub keys: u8,
    /// 0 none, 1 gold, 2 neon, 3 chrome, 4 midnight, 5 sunset
    pub livery: u8,
}

impl Settings {
    pub fn sanitized(mut self) -> Settings {
        self.music = self.music.min(crate::audio::STYLE_COUNT - 1);
        self.keys = self.keys.min(KEY_SCHEMES as u8 - 1);
        self.livery = self.livery.min(LIVERIES as u8 - 1);
        self
    }

    pub fn encode(&self) -> String {
        let s = self.sanitized();
        format!(
            "TS1;m={};u={};k={};l={}",
            s.music, s.mph as u8, s.keys, s.livery
        )
    }

    /// Hostile-safe: unknown keys are ignored, bad numbers fall back to the default.
    pub fn decode(txt: &str) -> Settings {
        let mut s = Settings::default();
        let txt = txt.trim();
        if txt.len() > 80 || !txt.starts_with("TS1;") {
            return s;
        }
        for kv in txt.split(';').skip(1) {
            let Some((k, v)) = kv.split_once('=') else {
                continue;
            };
            let n: u8 = v.parse().unwrap_or(0);
            match k {
                "m" => s.music = n,
                "u" => s.mph = n != 0,
                "k" => s.keys = n,
                "l" => s.livery = n,
                _ => {}
            }
        }
        s.sanitized()
    }

    /// Makes the settings take effect everywhere (audio, HUD, car paint).
    pub fn apply(&self) {
        let s = self.sanitized();
        crate::audio::set_music_style(s.music);
        MPH.store(s.mph as u8, Ordering::Relaxed);
        LIVERY.store(s.livery, Ordering::Relaxed);
    }
}

pub fn mph() -> bool {
    MPH.load(Ordering::Relaxed) != 0
}

/// Speed as shown on the HUD in the chosen unit.
pub fn display_speed(kmh: i32) -> i32 {
    if mph() {
        (kmh as f32 * 0.621_371).round() as i32
    } else {
        kmh
    }
}

pub fn livery() -> u8 {
    LIVERY.load(Ordering::Relaxed)
}

/// Body and two accent colours of a paint job (None = the car keeps its own colours).
pub fn livery_colours(l: u8) -> Option<(u32, u32, u32)> {
    match l {
        1 => Some((0xE8B923, 0xFFE27A, 0x8A6A00)),
        2 => Some((0x18F0C8, 0xFF2BD6, 0x101030)),
        3 => Some((0xC9D1D9, 0x7D8790, 0xF5F8FA)),
        4 => Some((0x141B4D, 0x3A4BC8, 0xC8D0F0)),
        5 => Some((0xFF6A3C, 0xFFB347, 0x8E2A5A)),
        _ => None,
    }
}

/// Secret codes ("cheat codes" for paint only: they change colours and nothing else).
pub fn secret_code(s: &str) -> Option<u8> {
    let norm: String = s
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    match norm.as_str() {
        "GOLDRUSH" => Some(1),
        "NEONNIGHT" => Some(2),
        "CHROMEDREAM" => Some(3),
        "MIDNIGHTRUN" => Some(4),
        "SUNSETDRIVE" => Some(5),
        _ => None,
    }
}

/// Key scheme -> (left, right, accel, brake, turbo) names for the help line.
pub fn scheme_name(k: u8) -> &'static str {
    ["ARROWS+WASD", "WASD+SPACE", "IJKL+SPACE", "KEYPAD 8462+0"][(k as usize).min(KEY_SCHEMES - 1)]
}
