//! Stage designer: pick the seed, vehicle, conditions, the four landscapes, how twisty and hilly
//! the road is and how long; a profile chart previews the built road and a short, typo-checked
//! code (`TCS1-...`) lets anyone else drive exactly the same stage.
use crate::career_ui::{class_name, fill, level_name, tx, txc};
use crate::game::*;
use crate::i18n_career::{t, C};
use crate::pick::{SF, WX};
use crate::share::StageCode;
use crate::track::{self, Layout};
use crate::{H, NB, W};

const BIOME_NAMES: [C; NB] = [
    C::Bm0,
    C::Bm1,
    C::Bm2,
    C::Bm3,
    C::Bm4,
    C::Bm5,
    C::Bm6,
    C::Bm7,
    C::Bm8,
    C::Bm9,
    C::Bm10,
    C::Bm11,
    C::Bm12,
    C::Bm13,
    C::Bm14,
    C::Bm15,
    C::Bm16,
    C::Bm17,
    C::Bm18,
];
pub const ROWS: usize = 14;
const TIMES: [&str; 5] = ["AUTO", "06:00", "12:00", "18:00", "00:00"];

pub struct DesignApp {
    pub code: StageCode,
    cursor: usize,
    prev: Input,
    hold: f32,
    chart: Vec<(f32, f32, u8)>,
    done: bool,
    t: f32,
}

impl DesignApp {
    pub fn new(code: Option<StageCode>) -> DesignApp {
        let mut d = DesignApp {
            code: code.unwrap_or_default().sanitized(),
            cursor: 0,
            prev: Input::default(),
            hold: 0.0,
            chart: Vec::new(),
            done: false,
            t: 0.0,
        };
        d.rebuild();
        d
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// The finished stage (after Enter), once.
    pub fn take_code(&mut self) -> Option<StageCode> {
        if self.done {
            self.done = false;
            Some(self.code.sanitized())
        } else {
            None
        }
    }

    fn rebuild(&mut self) {
        let c = self.code.sanitized();
        let tr = track::with_layout(c.layout, || track::build(c.seed as u64));
        let n = tr.segs.len().max(1);
        let step = (n / 280).max(1);
        let mut v = Vec::new();
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for s in tr.segs.iter().step_by(step) {
            lo = lo.min(s.y1);
            hi = hi.max(s.y1);
        }
        let span = (hi - lo).max(1.0);
        for s in tr.segs.iter().step_by(step) {
            v.push(((s.y1 - lo) / span, s.curve, s.biome));
        }
        self.chart = v;
    }

    fn change(&mut self, dir: i32, fast: bool) {
        let c = &mut self.code;
        let wrap = |v: i32, n: i32| v.rem_euclid(n);
        match self.cursor {
            0 => {
                let st: i64 = if fast { 1000 } else { 1 };
                c.seed = (c.seed as i64 + st * dir as i64).rem_euclid(1 << 32) as u32;
            }
            1 => c.mode = wrap(c.mode as i32 + dir, MODE_COUNT as i32) as u8,
            2 => c.class = wrap(c.class as i32 + dir, crate::cars::CLASS_COUNT as i32) as u8,
            3 => c.level = wrap(c.level as i32 + dir, LEVEL_COUNT as i32) as u8,
            4 => {
                c.weather = wrap(c.weather as i32 + dir, crate::env::WEATHER_COUNT as i32 + 1) as u8
            }
            5 => {
                c.surface = wrap(c.surface as i32 + dir, crate::env::SURFACE_COUNT as i32 + 1) as u8
            }
            6 => c.time = wrap(c.time as i32 + dir, 5) as u8,
            7..=10 => {
                let b = &mut c.layout.biomes[self.cursor - 7];
                // AUTO is stored as 255; map it to slot NB for stepping
                let cur = if *b as usize >= NB {
                    NB as i32
                } else {
                    *b as i32
                };
                let n = wrap(cur + dir, NB as i32 + 1);
                *b = if n as usize == NB {
                    Layout::AUTO
                } else {
                    n as u8
                };
            }
            11 => c.layout.twist = wrap(c.layout.twist as i32 + dir, 4) as u8,
            12 => c.layout.hills = wrap(c.layout.hills as i32 + dir, 4) as u8,
            _ => c.layout.length = wrap(c.layout.length as i32 + dir, 3) as u8,
        }
        self.code = self.code.sanitized();
        self.rebuild();
    }

    pub fn update(&mut self, dt: f32, inp: &Input, enter: bool, _back: bool) {
        let dt = if dt.is_finite() {
            dt.clamp(0.0, 0.1)
        } else {
            0.0
        };
        self.t += dt;
        let press = |now: bool, before: bool| now && !before;
        if press(inp.accel, self.prev.accel) {
            self.cursor = (self.cursor + ROWS - 1) % ROWS;
        }
        if press(inp.brake, self.prev.brake) {
            self.cursor = (self.cursor + 1) % ROWS;
        }
        let dir = inp.right as i32 - inp.left as i32;
        let was = self.prev.right as i32 - self.prev.left as i32;
        if dir != 0 {
            if dir != was {
                self.hold = -0.35;
                self.change(dir, inp.turbo);
            } else {
                self.hold += dt;
                if self.hold >= 0.06 {
                    self.hold = 0.0;
                    self.change(dir, inp.turbo);
                }
            }
        }
        self.prev = *inp;
        if enter {
            self.done = true;
        }
    }

    /// Text of row `i`: (label, value).
    pub fn row(&self, i: usize) -> (&'static str, String) {
        let c = &self.code;
        let auto = || t(C::DsAuto).to_string();
        let menu = |v: u8, names: &[C]| {
            if v == 0 {
                auto()
            } else {
                t(names[(v as usize - 1).min(names.len() - 1)]).to_string()
            }
        };
        match i {
            0 => (t(C::DsSeed), c.seed.to_string()),
            1 => (
                t(C::DsMode),
                crate::i18n::mode_name(c.mode as usize).to_string(),
            ),
            2 => (t(C::DsVehicle), class_name(c.class as usize).to_string()),
            3 => (t(C::DsLevel), level_name(c.level).to_string()),
            4 => (t(C::Weather), menu(c.weather, &WX)),
            5 => (t(C::Surface), menu(c.surface, &SF)),
            6 => (t(C::DsTime), TIMES[c.time as usize % 5].to_string()),
            7..=10 => {
                let b = c.layout.biomes[i - 7];
                let v = if b as usize >= NB {
                    auto()
                } else {
                    t(BIOME_NAMES[b as usize]).to_string()
                };
                (t(C::DsLandscape), format!("{} {}", i - 6, v))
            }
            11 => (t(C::DsTwist), (c.layout.twist + 1).to_string()),
            12 => (t(C::DsHills), (c.layout.hills + 1).to_string()),
            _ => (t(C::DsLength), (c.layout.length + 1).to_string()),
        }
    }

    pub fn render(&mut self, fb: &mut [u32]) {
        for y in 0..H {
            let k = y as f32 / H as f32;
            let col = ((14.0 + 12.0 * k) as u32) << 16
                | ((20.0 + 18.0 * k) as u32) << 8
                | (34.0 + 30.0 * k) as u32;
            fb[y * W..(y + 1) * W].fill(col);
        }
        txc(fb, 10, t(C::DsTitle), 3, 0xFFD23C);
        for i in 0..ROWS {
            let y = 42 + i as i32 * 24;
            let sel = i == self.cursor;
            if sel {
                fill(fb, 14, y - 5, 350, y + 17, 0x2A3F66);
            }
            let (lab, val) = self.row(i);
            tx(fb, 22, y, lab, 2, if sel { 0xFFFFFF } else { 0x9AB0D0 });
            let sc = if val.chars().count() > 11 { 1 } else { 2 };
            tx(
                fb,
                172,
                y + if sc == 1 { 4 } else { 0 },
                &val,
                sc,
                if sel { 0xFFD23C } else { 0xE0E8F5 },
            );
        }
        // profile chart
        let (x0, x1) = (360i32, 626i32);
        fill(fb, x0 - 4, 44, x1 + 4, 360, 0x10182A);
        let n = self.chart.len().max(1);
        for (i, (h, cv, b)) in self.chart.iter().enumerate() {
            let x = x0 + (i * (x1 - x0) as usize / n) as i32;
            let w = ((x1 - x0) as usize / n).max(1) as i32;
            let hue = (*b as u32 * 53) % 360;
            let col = hsv(hue);
            fill(fb, x, 60 + 6, x + w, 70, col);
            let yh = 190 - (h * 100.0) as i32;
            fill(fb, x, yh, x + w, 192, 0x3C6EA8);
            fill(fb, x, yh, x + w, yh + 2, 0xBFE0FF);
            let cm = 260 + (cv.clamp(-6.0, 6.0) * 6.0) as i32;
            fill(fb, x, cm.min(260), x + w, cm.max(260) + 1, 0xE8A040);
        }
        fill(fb, x0, 260, x1, 261, 0x506080);
        txc(
            fb,
            376,
            &format!("{} {}", t(C::DsCode), self.code.encode()),
            1,
            0xFFFFFF,
        );
        txc(fb, 388, t(C::DsHint), 1, 0x9AB0D0);
    }
}

fn hsv(h: u32) -> u32 {
    let k = |n: f32| {
        let kk = (n + h as f32 / 60.0) % 6.0;
        (0.35 + 0.55 * (1.0 - kk.min(4.0 - kk).clamp(0.0, 1.0))).clamp(0.0, 1.0)
    };
    let (r, g, b) = (k(5.0), k(3.0), k(1.0));
    ((r * 255.0) as u32) << 16 | ((g * 255.0) as u32) << 8 | (b * 255.0) as u32
}
