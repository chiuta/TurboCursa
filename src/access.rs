//! Accessibility: colour-vision filters, high contrast and driving assists.
//!
//! The filters work on the finished frame (`apply`), so every screen gets them. The colour-blind
//! modes are "daltonisation": the colour information a viewer cannot separate is shifted into
//! channels they can.
use std::cell::Cell;

/// 0 off, 1 protanopia, 2 deuteranopia, 3 tritanopia, 4 high contrast
pub const FILTER_COUNT: usize = 5;
pub const ASSIST_MAX: u8 = 3;

thread_local! {
    static FILTER: Cell<u8> = const { Cell::new(0) };
    static CALM: Cell<bool> = const { Cell::new(false) };
}

/// Calm mode (reduced motion): no screen shake, no strobing police text, soft lightning flashes.
pub fn set_calm(on: bool) {
    CALM.with(|c| c.set(on));
}

pub fn calm() -> bool {
    CALM.with(|c| c.get())
}

pub fn set_filter(i: usize) {
    FILTER.with(|f| f.set(i.min(FILTER_COUNT - 1) as u8));
}

pub fn filter() -> usize {
    FILTER.with(|f| f.get()) as usize
}

pub fn parse_filter(s: &str) -> Option<usize> {
    match s.to_ascii_lowercase().as_str() {
        "off" | "none" | "0" => Some(0),
        "protan" | "protanopia" | "1" => Some(1),
        "deutan" | "deuteranopia" | "2" => Some(2),
        "tritan" | "tritanopia" | "3" => Some(3),
        "contrast" | "highcontrast" | "4" => Some(4),
        _ => None,
    }
}

#[inline]
fn daltonize(c: u32, sim: [[f32; 3]; 3]) -> u32 {
    let (r, g, b) = (
        ((c >> 16) & 255) as f32,
        ((c >> 8) & 255) as f32,
        (c & 255) as f32,
    );
    let s = [
        sim[0][0] * r + sim[0][1] * g + sim[0][2] * b,
        sim[1][0] * r + sim[1][1] * g + sim[1][2] * b,
        sim[2][0] * r + sim[2][1] * g + sim[2][2] * b,
    ];
    let e = [r - s[0], g - s[1], b - s[2]];
    let o = [r, g + 0.7 * e[0] + e[1], b + 0.7 * e[0] + e[2]];
    pack(o)
}

#[inline]
fn pack(o: [f32; 3]) -> u32 {
    let q = |v: f32| v.clamp(0.0, 255.0) as u32;
    q(o[0]) << 16 | q(o[1]) << 8 | q(o[2])
}

#[inline]
fn contrast(c: u32) -> u32 {
    let ch = |v: u32| {
        let x = v as f32 / 255.0;
        // S-curve: darker darks, brighter lights, then a small saturation lift
        let y = x * x * (3.0 - 2.0 * x);
        (x * 0.35 + y * 0.65) * 255.0
    };
    let (r, g, b) = (ch((c >> 16) & 255), ch((c >> 8) & 255), ch(c & 255));
    let l = 0.3 * r + 0.59 * g + 0.11 * b;
    pack([l + (r - l) * 1.25, l + (g - l) * 1.25, l + (b - l) * 1.25])
}

/// Applies the current filter to a finished frame.
pub fn apply(fb: &mut [u32]) {
    let f = filter();
    if f == 0 {
        return;
    }
    const PROTAN: [[f32; 3]; 3] = [
        [0.567, 0.433, 0.0],
        [0.558, 0.442, 0.0],
        [0.0, 0.242, 0.758],
    ];
    const DEUTAN: [[f32; 3]; 3] = [[0.625, 0.375, 0.0], [0.7, 0.3, 0.0], [0.0, 0.3, 0.7]];
    const TRITAN: [[f32; 3]; 3] = [[0.95, 0.05, 0.0], [0.0, 0.433, 0.567], [0.0, 0.475, 0.525]];
    match f {
        1 => fb.iter_mut().for_each(|p| *p = daltonize(*p, PROTAN)),
        2 => fb.iter_mut().for_each(|p| *p = daltonize(*p, DEUTAN)),
        3 => fb.iter_mut().for_each(|p| *p = daltonize(*p, TRITAN)),
        _ => fb.iter_mut().for_each(|p| *p = contrast(*p)),
    }
}
