//! Ghost replay: the best run of a stage, replayed beside you. A ghost is a text code that can be
//! stored in a file / localStorage or shared with a friend who plays the same seed.
//!
//! The ghost stores the position along the road and the lateral position every `STEP` seconds; the
//! replay interpolates between samples.

/// Seconds between two samples.
pub const STEP: f32 = 0.2;
/// Longest ghost kept (samples): ten minutes.
pub const MAX_SAMPLES: usize = 3000;

#[derive(Clone, Debug, PartialEq)]
pub struct Ghost {
    pub seed: u64,
    pub mode: u8,
    pub class: u8,
    pub secs: f32,
    pub score: u32,
    /// (distance along the road, lateral position -2.2..2.2)
    pub pts: Vec<(f32, f32)>,
}

impl Ghost {
    /// Position of the ghost `t` seconds after the start (it waits at its last sample).
    pub fn at(&self, t: f32) -> Option<(f32, f32)> {
        let n = self.pts.len();
        if n == 0 || !t.is_finite() {
            return None;
        }
        let f = (t.max(0.0) / STEP).min((n - 1) as f32);
        let i = f as usize;
        let j = (i + 1).min(n - 1);
        let u = f - i as f32;
        let (a, b) = (self.pts[i], self.pts[j]);
        Some((a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u))
    }

    /// True when `self` is a faster (or equal-time, higher-scoring) run than `other`.
    pub fn beats(&self, other: &Ghost) -> bool {
        self.secs < other.secs || (self.secs == other.secs && self.score > other.score)
    }

    /// "TG2;seed;mode;class;secs*10;score;payload". The payload holds, per sample, the change of
    /// the position step and the change of the lateral position, each zigzag-coded and written as
    /// 6-bit symbols (5 data bits + a continuation bit): about 2.5 characters per sample instead
    /// of the 7 of the old `TG1` format, which is still read.
    pub fn encode(&self) -> String {
        let mut out = String::with_capacity(self.pts.len() * 3);
        let (mut prev, mut pd, mut px) = (0u32, 0i32, 0i32);
        for &(p, x) in &self.pts {
            let pos = p.max(0.0) as u32;
            let d = pos.saturating_sub(prev).min(0xFFFF);
            prev += d;
            let xi = (x.clamp(-2.2, 2.2) * 900.0).round() as i32;
            put(&mut out, zig(d as i32 - pd));
            put(&mut out, zig(xi - px));
            pd = d as i32;
            px = xi;
        }
        format!(
            "TG2;{};{};{};{};{};{}",
            self.seed,
            self.mode,
            self.class,
            (self.secs.clamp(0.0, 99_999.0) * 10.0) as u32,
            self.score,
            out
        )
    }

    /// Hostile-safe parser: anything malformed gives `None`.
    pub fn decode(s: &str) -> Option<Ghost> {
        let s = s.trim();
        if s.len() > 64 + MAX_SAMPLES * 7 {
            return None;
        }
        if s.starts_with("TG2;") {
            return Ghost::decode_v2(s);
        }
        if !s.starts_with("TG1;") {
            return None;
        }
        let p: Vec<&str> = s.split(';').collect();
        if p.len() != 7 {
            return None;
        }
        let seed: u64 = p[1].parse().ok()?;
        let mode: u8 = p[2].parse().ok()?;
        let class: u8 = p[3].parse().ok()?;
        let s10: u32 = p[4].parse().ok()?;
        let score: u32 = p[5].parse().ok()?;
        let hex = p[6].as_bytes();
        if !hex.len().is_multiple_of(7)
            || hex.len() / 7 > MAX_SAMPLES
            || !hex.iter().all(u8::is_ascii_hexdigit)
        {
            return None;
        }
        let mut pts = Vec::with_capacity(hex.len() / 7);
        let mut pos = 0u32;
        for c in hex.chunks(7) {
            let t = std::str::from_utf8(c).ok()?;
            pos = pos.saturating_add(u32::from_str_radix(&t[..4], 16).ok()?);
            let xi = i32::from_str_radix(&t[4..], 16).ok()? - 2048;
            pts.push((pos as f32, (xi as f32 / 900.0).clamp(-2.2, 2.2)));
        }
        Some(Ghost {
            seed,
            mode,
            class,
            secs: s10.min(999_990) as f32 / 10.0,
            score,
            pts,
        })
    }

    fn decode_v2(s: &str) -> Option<Ghost> {
        let p: Vec<&str> = s.split(';').collect();
        if p.len() != 7 {
            return None;
        }
        let seed: u64 = p[1].parse().ok()?;
        let mode: u8 = p[2].parse().ok()?;
        let class: u8 = p[3].parse().ok()?;
        let s10: u32 = p[4].parse().ok()?;
        let score: u32 = p[5].parse().ok()?;
        let mut it = p[6].bytes();
        let mut pts = Vec::new();
        let (mut pos, mut pd, mut px) = (0i64, 0i64, 0i64);
        while pts.len() < MAX_SAMPLES {
            let Some(a) = take(&mut it)? else { break };
            let b = take(&mut it)??;
            pd = (pd + unzig(a)).clamp(0, 0xFFFF);
            px = (px + unzig(b)).clamp(-1980, 1980);
            pos += pd;
            pts.push((pos as f32, (px as f32 / 900.0).clamp(-2.2, 2.2)));
        }
        if it.next().is_some() {
            return None; // longer than the limit
        }
        Some(Ghost {
            seed,
            mode,
            class,
            secs: s10.min(999_990) as f32 / 10.0,
            score,
            pts,
        })
    }
}

const ALPHA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn zig(v: i32) -> u32 {
    ((v << 1) ^ (v >> 31)) as u32
}

fn unzig(u: u32) -> i64 {
    ((u >> 1) as i64) ^ -((u & 1) as i64)
}

fn put(out: &mut String, mut v: u32) {
    loop {
        let low = (v & 31) as usize;
        v >>= 5;
        if v == 0 {
            out.push(ALPHA[low] as char);
            return;
        }
        out.push(ALPHA[low | 32] as char);
    }
}

/// Next value of the payload: `Some(None)` at a clean end, `None` when the text is malformed.
fn take(it: &mut std::str::Bytes) -> Option<Option<u32>> {
    let mut v = 0u32;
    for k in 0..4 {
        let Some(c) = it.next() else {
            return if k == 0 { Some(None) } else { None };
        };
        let sym = ALPHA.iter().position(|&a| a == c)? as u32;
        v |= (sym & 31) << (5 * k);
        if sym & 32 == 0 {
            return Some(Some(v));
        }
    }
    None
}
