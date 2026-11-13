use crate::font;
use crate::fractal::{self, Assets, Img};
use crate::fx::{self, FxParams};
use crate::game::*;
use crate::i18n::{self, S};
use crate::track::*;
use std::cell::Cell;

mod city;
mod landscapes;
mod scenery;
mod sprites;
mod weather;
use self::city::*;
pub use self::landscapes::draw_car_preview;
use self::landscapes::*;
use self::scenery::*;
use self::sprites::*;
use self::weather::*;

const DRAW_DIST: usize = 220;
thread_local! {
    /// Livery of the player's car while it is being drawn (None for traffic).
    static LIVERY: Cell<Option<crate::cars::CarSpec>> = const { Cell::new(None) };
    static DIM: Cell<(usize, usize)> = const { Cell::new((crate::W, crate::H)) };
}
fn vw() -> usize {
    DIM.with(|d| d.get().0)
}
fn vh() -> usize {
    DIM.with(|d| d.get().1)
}
/// Design-grid scale: 2.0 at 640x400, 1.0 at 640x200 (split-screen halves).
fn vk() -> f32 {
    (vw() as f32 / 320.0).min(vh() as f32 / 200.0)
}
fn horizon() -> i32 {
    (vh() / 2) as i32
}

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}
fn unpack(c: u32) -> [f32; 3] {
    [
        ((c >> 16) & 255) as f32,
        ((c >> 8) & 255) as f32,
        (c & 255) as f32,
    ]
}
fn mix(a: u32, b: u32, t: f32) -> u32 {
    let (a, b) = (unpack(a), unpack(b));
    rgb(
        (a[0] + (b[0] - a[0]) * t) as u8,
        (a[1] + (b[1] - a[1]) * t) as u8,
        (a[2] + (b[2] - a[2]) * t) as u8,
    )
}
fn shade(c: u32, k: f32) -> u32 {
    let a = unpack(c);
    rgb(
        (a[0] * k).min(255.0) as u8,
        (a[1] * k).min(255.0) as u8,
        (a[2] * k).min(255.0) as u8,
    )
}
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

struct Pal {
    sky_top: u32,
    sky_hor: u32,
    mount: u32,
    grass: [u32; 2],
    road: [u32; 2],
    rumble: [u32; 2],
    lane: u32,
}

fn pal(biome: usize) -> Pal {
    match biome {
        0 => Pal {
            sky_top: rgb(70, 140, 230),
            sky_hor: rgb(190, 225, 255),
            mount: rgb(90, 130, 150),
            grass: [rgb(16, 170, 16), rgb(0, 150, 0)],
            road: [rgb(107, 107, 107), rgb(100, 100, 104)],
            rumble: [rgb(240, 240, 240), rgb(200, 20, 20)],
            lane: rgb(220, 220, 220),
        },
        1 => Pal {
            sky_top: rgb(250, 170, 90),
            sky_hor: rgb(255, 236, 190),
            mount: rgb(190, 120, 80),
            grass: [rgb(224, 186, 112), rgb(212, 172, 96)],
            road: [rgb(112, 106, 100), rgb(104, 98, 94)],
            rumble: [rgb(240, 240, 240), rgb(210, 90, 20)],
            lane: rgb(235, 225, 200),
        },
        3 => Pal {
            sky_top: rgb(58, 128, 236),
            sky_hor: rgb(218, 236, 255),
            mount: rgb(104, 136, 170),
            grass: [rgb(40, 164, 62), rgb(30, 148, 54)],
            road: [rgb(112, 112, 112), rgb(104, 104, 108)],
            rumble: [rgb(240, 240, 240), rgb(56, 156, 92)],
            lane: rgb(235, 235, 235),
        },
        4 => Pal {
            sky_top: rgb(34, 46, 138),
            sky_hor: rgb(228, 142, 172),
            mount: rgb(158, 66, 46),
            grass: [rgb(178, 76, 46), rgb(164, 66, 40)],
            road: [rgb(122, 102, 98), rgb(112, 94, 92)],
            rumble: [rgb(240, 230, 220), rgb(200, 70, 40)],
            lane: rgb(242, 226, 210),
        },
        5 => Pal {
            sky_top: rgb(64, 168, 214),
            sky_hor: rgb(255, 226, 172),
            mount: rgb(120, 164, 176),
            grass: [rgb(72, 192, 150), rgb(58, 178, 140)],
            road: [rgb(122, 120, 124), rgb(112, 110, 116)],
            rumble: [rgb(250, 250, 250), rgb(232, 150, 40)],
            lane: rgb(255, 240, 200),
        },
        6 => Pal {
            sky_top: rgb(8, 12, 48),
            sky_hor: rgb(92, 204, 224),
            mount: rgb(30, 54, 104),
            grass: [rgb(40, 112, 144), rgb(34, 100, 134)],
            road: [rgb(60, 64, 84), rgb(54, 58, 78)],
            rumble: [rgb(220, 240, 255), rgb(40, 200, 220)],
            lane: rgb(200, 240, 255),
        },
        7 => Pal {
            sky_top: rgb(30, 74, 124),
            sky_hor: rgb(196, 228, 242),
            mount: rgb(150, 176, 204),
            grass: [rgb(228, 238, 248), rgb(214, 227, 242)],
            road: [rgb(116, 120, 128), rgb(108, 112, 122)],
            rumble: [rgb(250, 250, 255), rgb(60, 120, 200)],
            lane: rgb(240, 245, 255),
        },
        8 => Pal {
            sky_top: rgb(28, 8, 12),
            sky_hor: rgb(232, 92, 40),
            mount: rgb(40, 24, 26),
            grass: [rgb(52, 36, 36), rgb(42, 30, 32)],
            road: [rgb(72, 66, 68), rgb(64, 60, 62)],
            rumble: [rgb(236, 200, 160), rgb(240, 90, 20)],
            lane: rgb(255, 200, 140),
        },
        9 => Pal {
            sky_top: rgb(4, 40, 92),
            sky_hor: rgb(64, 192, 204),
            mount: rgb(10, 60, 104),
            grass: [rgb(198, 178, 122), rgb(186, 166, 112)],
            road: [rgb(82, 98, 112), rgb(74, 90, 106)],
            rumble: [rgb(222, 246, 250), rgb(255, 120, 140)],
            lane: rgb(210, 240, 250),
        },
        10 => Pal {
            sky_top: rgb(6, 4, 24),
            sky_hor: rgb(124, 64, 184),
            mount: rgb(40, 30, 92),
            grass: [rgb(20, 14, 52), rgb(14, 10, 42)],
            road: [rgb(52, 48, 92), rgb(46, 42, 86)],
            rumble: [rgb(255, 255, 255), rgb(255, 80, 200)],
            lane: rgb(232, 222, 255),
        },
        11 => Pal {
            sky_top: rgb(40, 8, 92),
            sky_hor: rgb(255, 112, 170),
            mount: rgb(60, 20, 100),
            grass: [rgb(20, 10, 42), rgb(26, 14, 54)],
            road: [rgb(42, 38, 72), rgb(36, 32, 66)],
            rumble: [rgb(80, 240, 255), rgb(255, 60, 200)],
            lane: rgb(255, 240, 120),
        },
        12 => Pal {
            sky_top: rgb(6, 26, 40),
            sky_hor: rgb(60, 172, 152),
            mount: rgb(20, 60, 72),
            grass: [rgb(62, 42, 92), rgb(52, 34, 82)],
            road: [rgb(72, 68, 88), rgb(64, 60, 82)],
            rumble: [rgb(222, 255, 242), rgb(120, 255, 180)],
            lane: rgb(200, 255, 230),
        },
        13 => Pal {
            sky_top: rgb(40, 14, 70),
            sky_hor: rgb(236, 150, 120),
            mount: rgb(90, 50, 110),
            grass: [rgb(54, 90, 120), rgb(46, 78, 108)],
            road: [rgb(96, 88, 110), rgb(88, 80, 104)],
            rumble: [rgb(255, 230, 200), rgb(120, 60, 200)],
            lane: rgb(255, 225, 190),
        },
        14 => Pal {
            sky_top: rgb(20, 20, 60),
            sky_hor: rgb(250, 235, 255),
            mount: rgb(150, 160, 210),
            grass: [rgb(236, 226, 244), rgb(222, 214, 236)],
            road: [rgb(120, 124, 150), rgb(112, 116, 142)],
            rumble: [rgb(255, 255, 255), rgb(120, 210, 255)],
            lane: rgb(255, 255, 255),
        },
        15 => Pal {
            sky_top: rgb(46, 28, 16),
            sky_hor: rgb(214, 160, 84),
            mount: rgb(100, 70, 40),
            grass: [rgb(120, 86, 48), rgb(108, 76, 42)],
            road: [rgb(74, 66, 60), rgb(66, 58, 54)],
            rumble: [rgb(230, 190, 90), rgb(120, 80, 40)],
            lane: rgb(255, 220, 140),
        },
        16 => Pal {
            sky_top: rgb(60, 130, 230),
            sky_hor: rgb(255, 214, 190),
            mount: rgb(190, 190, 230),
            grass: [rgb(246, 246, 252), rgb(232, 234, 246)],
            road: [rgb(150, 154, 176), rgb(140, 144, 168)],
            rumble: [rgb(255, 255, 255), rgb(255, 150, 120)],
            lane: rgb(255, 255, 255),
        },
        17 => Pal {
            sky_top: rgb(2, 2, 8),
            sky_hor: rgb(70, 24, 20),
            mount: rgb(30, 10, 24),
            grass: [rgb(18, 14, 26), rgb(12, 10, 20)],
            road: [rgb(48, 44, 60), rgb(40, 36, 52)],
            rumble: [rgb(255, 170, 60), rgb(110, 40, 20)],
            lane: rgb(255, 200, 120),
        },
        18 => Pal {
            sky_top: rgb(120, 30, 20),
            sky_hor: rgb(255, 196, 96),
            mount: rgb(160, 70, 40),
            grass: [rgb(176, 86, 44), rgb(160, 74, 38)],
            road: [rgb(104, 84, 78), rgb(96, 76, 72)],
            rumble: [rgb(255, 240, 200), rgb(200, 60, 30)],
            lane: rgb(255, 230, 170),
        },
        _ => Pal {
            sky_top: rgb(24, 24, 80),
            sky_hor: rgb(232, 112, 96),
            mount: rgb(50, 36, 80),
            grass: [rgb(30, 110, 92), rgb(20, 96, 80)],
            road: [rgb(84, 84, 96), rgb(76, 76, 90)],
            rumble: [rgb(230, 230, 240), rgb(150, 30, 120)],
            lane: rgb(200, 200, 220),
        },
    }
}

fn blend_pal(w: [f32; crate::NB]) -> Pal {
    let ps: [Pal; crate::NB] = std::array::from_fn(pal);
    let sum = w.iter().sum::<f32>().max(1e-3);
    let m = |f: &dyn Fn(&Pal) -> u32| -> u32 {
        let mut acc = [0.0f32; 3];
        for i in 0..crate::NB {
            let c = unpack(f(&ps[i]));
            for k in 0..3 {
                acc[k] += c[k] * w[i] / sum;
            }
        }
        rgb(acc[0] as u8, acc[1] as u8, acc[2] as u8)
    };
    Pal {
        sky_top: m(&|p| p.sky_top),
        sky_hor: m(&|p| p.sky_hor),
        mount: m(&|p| p.mount),
        grass: [0, 1].map(|i| m(&|p| p.grass[i])),
        road: [0, 1].map(|i| m(&|p| p.road[i])),
        rumble: [0, 1].map(|i| m(&|p| p.rumble[i])),
        lane: m(&|p| p.lane),
    }
}

// ---------- raster helpers ----------

fn hline(fb: &mut [u32], y: i32, x0: i32, x1: i32, c: u32) {
    if y < 0 || y >= vh() as i32 {
        return;
    }
    let a = x0.max(0);
    let b = x1.min(vw() as i32);
    if a >= b {
        return;
    }
    let row = y as usize * vw();
    for x in a..b {
        fb[row + x as usize] = c;
    }
}

fn rect(fb: &mut [u32], x0: f32, y0: f32, x1: f32, y1: f32, c: u32, clip: f32) {
    let ya = y0.round() as i32;
    let yb = y1.min(clip).round() as i32;
    for y in ya..yb {
        hline(fb, y, x0.round() as i32, x1.round() as i32, c);
    }
}

#[allow(clippy::too_many_arguments)]
fn quad(
    fb: &mut [u32],
    yt: f32,
    xlt: f32,
    xrt: f32,
    yb: f32,
    xlb: f32,
    xrb: f32,
    clip: f32,
    c: u32,
) {
    if yb - yt <= 0.0 {
        return;
    }
    let ya = yt.round() as i32;
    let ye = yb.min(clip).round() as i32;
    for y in ya.max(0)..ye.min(vh() as i32) {
        let t = (((y as f32 + 0.5) - yt) / (yb - yt)).clamp(0.0, 1.0);
        hline(
            fb,
            y,
            lerp(xlt, xlb, t).round() as i32,
            lerp(xrt, xrb, t).round() as i32,
            c,
        );
    }
}

fn tri(fb: &mut [u32], cx: f32, ytop: f32, ybot: f32, half: f32, c: u32, clip: f32) {
    quad(fb, ytop, cx, cx, ybot, cx - half, cx + half, clip, c);
}

fn ellipse(fb: &mut [u32], cx: f32, cy: f32, rx: f32, ry: f32, c: u32, clip: f32) {
    if rx < 0.3 || ry < 0.3 {
        return;
    }
    let ya = (cy - ry).floor() as i32;
    let yb = (cy + ry).ceil() as i32;
    for y in ya..yb {
        if (y as f32) >= clip {
            break;
        }
        let t = ((y as f32 + 0.5) - cy) / ry;
        if t.abs() >= 1.0 {
            continue;
        }
        let w = rx * (1.0 - t * t).sqrt();
        hline(fb, y, (cx - w).round() as i32, (cx + w).round() as i32, c);
    }
}

fn blend_px(fb: &mut [u32], x: i32, y: i32, c: u32, a: f32) {
    if x < 0 || y < 0 || x >= vw() as i32 || y >= vh() as i32 {
        return;
    }
    let i = y as usize * vw() + x as usize;
    fb[i] = mix(fb[i], c, a);
}

fn add_px(fb: &mut [u32], x: i32, y: i32, c: [f32; 3], v: f32) {
    if x < 0 || y < 0 || x >= vw() as i32 || y >= vh() as i32 {
        return;
    }
    let i = y as usize * vw() + x as usize;
    let o = unpack(fb[i]);
    fb[i] = rgb(
        (o[0] + c[0] * v).clamp(0.0, 255.0) as u8,
        (o[1] + c[1] * v).clamp(0.0, 255.0) as u8,
        (o[2] + c[2] * v).clamp(0.0, 255.0) as u8,
    );
}

/// Additive radial glow with quadratic falloff.
fn glow(fb: &mut [u32], cx: f32, cy: f32, rx: f32, ry: f32, col: u32, strength: f32, clip: f32) {
    let rx = rx.min(240.0 * vk());
    let ry = ry.min(240.0 * vk());
    if rx < 1.0 || ry < 1.0 || strength <= 0.0 {
        return;
    }
    let c = unpack(col);
    let y0 = ((cy - ry).floor() as i32).max(0);
    let y1 = ((cy + ry).ceil() as i32)
        .min(vh() as i32)
        .min(clip.ceil() as i32);
    let x0 = ((cx - rx).floor() as i32).max(0);
    let x1 = ((cx + rx).ceil() as i32).min(vw() as i32);
    let w = vw();
    let (kx, ky) = (1.0 / rx, 1.0 / ry);
    let cs = [c[0] * strength, c[1] * strength, c[2] * strength];
    for y in y0..y1 {
        let dy = ((y as f32 + 0.5) - cy) * ky;
        let dy2 = dy * dy;
        if dy2 >= 1.0 {
            continue;
        }
        // only the span of this row that lies inside the ellipse
        let half = (1.0 - dy2).sqrt() * rx;
        let xa = ((cx - half).floor() as i32).max(x0);
        let xb = ((cx + half).ceil() as i32).min(x1);
        let row = &mut fb[y as usize * w..(y as usize + 1) * w];
        for x in xa..xb {
            let dx = ((x as f32 + 0.5) - cx) * kx;
            let d = dx * dx + dy2;
            if d < 1.0 {
                let v = 1.0 - d;
                let v = v * v;
                let px = &mut row[x as usize];
                let o = *px;
                let r = (((o >> 16) & 255) as f32 + cs[0] * v).min(255.0) as u32;
                let g = (((o >> 8) & 255) as f32 + cs[1] * v).min(255.0) as u32;
                let b = ((o & 255) as f32 + cs[2] * v).min(255.0) as u32;
                *px = (r << 16) | (g << 8) | b;
            }
        }
    }
}

fn rect_a(fb: &mut [u32], x0: f32, y0: f32, x1: f32, y1: f32, c: u32, a: f32, clip: f32) {
    for y in (y0.round() as i32)..(y1.min(clip).round() as i32) {
        for x in (x0.round() as i32)..(x1.round() as i32) {
            blend_px(fb, x, y, c, a);
        }
    }
}

fn ellipse_a(fb: &mut [u32], cx: f32, cy: f32, rx: f32, ry: f32, c: u32, a: f32, clip: f32) {
    if rx < 0.3 || ry < 0.3 {
        return;
    }
    for y in ((cy - ry).floor() as i32)..((cy + ry).ceil() as i32) {
        if (y as f32) >= clip {
            break;
        }
        let t = ((y as f32 + 0.5) - cy) / ry;
        if t.abs() >= 1.0 {
            continue;
        }
        let w = rx * (1.0 - t * t).sqrt();
        for x in ((cx - w).round() as i32)..((cx + w).round() as i32) {
            blend_px(fb, x, y, c, a);
        }
    }
}

fn vrect(fb: &mut [u32], x0: f32, y0: f32, x1: f32, y1: f32, ct: u32, cb: u32, clip: f32) {
    let span = (y1 - y0).max(1.0);
    for y in (y0.round() as i32)..(y1.min(clip).round() as i32) {
        let t = ((y as f32 - y0) / span).clamp(0.0, 1.0);
        hline(fb, y, x0.round() as i32, x1.round() as i32, mix(ct, cb, t));
    }
}

// ---------- text ----------

pub fn text(fb: &mut [u32], x: i32, y: i32, s: &str, scale: i32, c: u32) {
    let scale = scale * vk() as i32;
    let mut cx = x;
    for ch in s.chars() {
        if let Some(g) = font::glyph(ch) {
            if font::fold(ch).1 {
                // accent mark: one pixel above the middle column
                for dy in 0..scale {
                    hline(fb, y - scale + dy, cx + scale, cx + 2 * scale, c);
                }
            }
            let g = g.as_bytes();
            for (i, &b) in g.iter().enumerate() {
                if b == b'#' {
                    let (gx, gy) = ((i % 3) as i32, (i / 3) as i32);
                    for dy in 0..scale {
                        hline(
                            fb,
                            y + gy * scale + dy,
                            cx + gx * scale,
                            cx + gx * scale + scale,
                            c,
                        );
                    }
                }
            }
        }
        cx += 4 * scale;
    }
}

fn text_shadow(fb: &mut [u32], x: i32, y: i32, s: &str, scale: i32, c: u32) {
    text(
        fb,
        x + vk() as i32,
        y + vk() as i32,
        s,
        scale,
        rgb(10, 10, 20),
    );
    text(fb, x, y, s, scale, c);
}

fn text_center(fb: &mut [u32], y: i32, s: &str, scale: i32, c: u32) {
    let w = font::text_width(s, scale * vk() as i32);
    text_shadow(fb, (vw() as i32 - w) / 2, y, s, scale, c);
}

/// Scaled blit of a mip-mapped fractal sprite (nearest sampling from the best mip level).
#[allow(clippy::too_many_arguments)]
fn blit(
    fb: &mut [u32],
    img: &Img,
    src: (f32, f32, f32, f32),
    left: f32,
    top: f32,
    dw: f32,
    dh: f32,
    clip: f32,
    fog: (u32, f32),
) {
    if dw < 1.0 || dh < 1.0 || dw > 6.0 * vw() as f32 {
        return;
    }
    let ratio = img.mips[0].w as f32 * (src.2 - src.0) / dw;
    let lvl = if ratio > 1.0 {
        (ratio.log2().floor() as usize).min(img.mips.len() - 1)
    } else {
        0
    };
    let t = &img.mips[lvl];
    let y0 = (top.floor() as i32).max(0);
    let y1 = ((top + dh).ceil() as i32)
        .min(vh() as i32)
        .min(clip.ceil() as i32);
    let x0 = (left.floor() as i32).max(0);
    let x1 = ((left + dw).ceil() as i32).min(vw() as i32);
    for y in y0..y1 {
        let v = src.1 + ((y as f32 + 0.5 - top) / dh) * (src.3 - src.1);
        let ty = ((v * t.h as f32) as usize).min(t.h - 1);
        let row = ty * t.w;
        for x in x0..x1 {
            let u = src.0 + ((x as f32 + 0.5 - left) / dw) * (src.2 - src.0);
            let c = t.px[row + ((u * t.w as f32) as usize).min(t.w - 1)];
            let a = c >> 24;
            if a < 8 {
                continue;
            }
            let mut col = c & 0xFF_FFFF;
            if fog.1 > 0.0 {
                col = mix(col, fog.0, fog.1);
            }
            let i = y as usize * vw() + x as usize;
            fb[i] = if a > 247 {
                col
            } else {
                mix(fb[i], col, a as f32 / 255.0)
            };
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn sprite_tree(
    fb: &mut [u32],
    img: &Img,
    cx: f32,
    by: f32,
    ux: f32,
    world_h: f32,
    clip: f32,
    fog: (u32, f32),
) {
    let dh = world_h * ux;
    let dw = dh * img.mips[0].w as f32 / img.mips[0].h as f32;
    blit(
        fb,
        img,
        (0.0, 0.0, 1.0, 1.0),
        cx - dw / 2.0,
        by - dh * (250.0 / 256.0),
        dw,
        dh,
        clip,
        fog,
    );
}

// ---------- main render ----------

#[derive(Clone, Copy, Default)]
struct SegProj {
    ok: bool,
    s1: f32,
    x1: f32,
    y1: f32,
    s2: f32,
    x2: f32,
    y2: f32,
    clip: f32,
}

fn project(wx: f32, wy: f32, wz: f32, cx: f32, cy: f32, cz: f32) -> (f32, f32, f32, f32) {
    let (tx, ty, tz) = (wx - cx, wy - cy, wz - cz);
    let s = CAM_DEPTH / tz;
    (
        s,
        vw() as f32 / 2.0 + s * tx * vw() as f32 / 2.0,
        vh() as f32 / 2.0 - s * ty * vh() as f32 / 2.0,
        s * ROAD_W * vw() as f32 / 2.0,
    )
}

/// Tiny per-variant phase used to make hovering rocks bob out of step with each other.
fn g_t_hover(variant: u8) -> f32 {
    variant as f32 * 1.7
}

fn draw_sky(fb: &mut [u32], g: &Game, p: &Pal) {
    let hz = horizon() as f32;
    for y in 0..horizon() {
        let t = y as f32 / hz;
        hline(
            fb,
            y,
            0,
            vw() as i32,
            mix(p.sky_top, p.sky_hor, t.powf(1.7)),
        );
    }
    let mut w = g.biome_w;
    let ws = w.iter().sum::<f32>().max(1e-3);
    for v in w.iter_mut() {
        *v /= ws;
    }
    // how "night-like" the current sky is: dusk, mirror flats and (partly) the violet mesa twilight
    let nw = (w[2]
        + w[6]
        + 0.55 * w[4]
        + 0.6 * w[7]
        + 0.4 * w[8]
        + w[10]
        + 0.7 * w[11]
        + 0.5 * w[12]
        + 0.6 * w[13]
        + 0.3 * w[14]
        + 0.3 * w[15]
        + w[17]
        + 0.4 * w[18]
        + 0.6 * w[13]
        + 0.3 * w[14]
        + 0.3 * w[15]
        + w[17]
        + 0.4 * w[18])
        .min(1.0);

    // stars (dusk)
    if nw > 0.05 {
        for i in 0..160u32 {
            let h = fx::hash(i.wrapping_mul(7919));
            let sx = ((h % 1000) as f32 / 1000.0 * vw() as f32 - g.sky_off * 0.08 * vk())
                .rem_euclid(vw() as f32);
            let sy = ((h >> 10) % 1000) as f32 / 1000.0 * hz * 0.8;
            let tw = 0.6 + 0.4 * (g.elapsed * 3.0 + i as f32).sin();
            let a = nw * tw * (1.0 - sy / (hz * 0.8) * 0.6);
            add_px(fb, sx as i32, sy as i32, [255.0, 255.0, 255.0], a);
            if i % 9 == 0 {
                add_px(fb, sx as i32 + 1, sy as i32, [200.0, 210.0, 255.0], a * 0.6);
                add_px(fb, sx as i32, sy as i32 + 1, [200.0, 210.0, 255.0], a * 0.6);
            }
        }
    }

    // sun / moon
    let sun_x = (vw() as f32 * 0.72 - g.sky_off * 0.15 * vk())
        .rem_euclid(vw() as f32 + 240.0 * vk())
        - 120.0 * vk();
    let sun_y = hz
        * (1.0
            - (0.34 * w[0]
                + 0.10 * w[1]
                + 0.34 * w[2]
                + 0.38 * w[3]
                + 0.42 * w[4]
                + 0.30 * w[5]
                + 0.40 * w[6]
                + 0.22 * w[7]
                + 0.12 * w[8]
                + 0.45 * w[9]
                + 0.50 * w[10]
                + 0.40 * w[12]
                + 0.55 * w[13]
                + 0.35 * w[14]
                + 0.30 * w[15]
                + 0.20 * w[16]
                + 0.60 * w[17]
                + 0.22 * w[18]));
    let sr = vk()
        * (9.0 * w[0]
            + 24.0 * w[1]
            + 8.0 * w[2]
            + 10.0 * w[3]
            + 17.0 * w[4]
            + 11.0 * w[5]
            + 13.0 * w[6]
            + 12.0 * w[7]
            + 14.0 * w[8]
            + 9.0 * w[9]
            + 18.0 * w[10]
            + 9.0 * w[12]
            + 20.0 * w[13]
            + 11.0 * w[14]
            + 10.0 * w[15]
            + 16.0 * w[16]
            + 6.0 * w[17]
            + 20.0 * w[18]);
    let c = mix(rgb(255, 246, 214), rgb(255, 150, 60), w[1]);
    let sun_col = mix(
        mix(c, rgb(225, 235, 255), w[2] + w[6]),
        rgb(252, 248, 232),
        w[4],
    );
    let sun_col = mix(
        mix(sun_col, rgb(255, 70, 30), w[8]),
        rgb(180, 255, 230),
        w[12],
    );
    glow(fb, sun_x, sun_y, sr * 5.0, sr * 5.0, sun_col, 0.35, hz);
    ellipse(fb, sun_x, sun_y, sr, sr, sun_col, hz);

    // fractal clouds (periodic fBm texture, scrolled with parallax)
    let ca = ((1.0 - (w[2] + w[6]) * 0.75 - w[4] * 0.45) * 0.8 + w[3] * 0.35
        - 0.9 * (w[9] + w[10] + w[12] + w[17])
        - 0.6 * (w[8] + w[11] + w[13] + w[15] + w[18])
        - 0.5 * w[16])
        .max(0.0);
    if ca > 0.02 {
        let ct = &g.assets.clouds;
        let off = (g.sky_off * 0.35 * vk() + g.elapsed * 4.0 * vk()) as i32;
        let y_top = (6.0 * vk()) as i32;
        for row in 0..ct.h.min(horizon() as usize - y_top as usize) {
            let yy = y_top + row as i32;
            for x in 0..vw() as i32 {
                let c = ct.px[row * ct.w + ((x + off).rem_euclid(ct.w as i32)) as usize];
                let a = (c >> 24) as f32 / 255.0 * ca;
                if a > 0.01 {
                    blend_px(fb, x, yy, mix(c & 0xFF_FFFF, p.sky_hor, 0.25), a);
                }
            }
        }
    }

    // twin moon over the mirror flats
    if w[6] > 0.04 {
        let mx = (vw() as f32 * 0.28 - g.sky_off * 0.1 * vk())
            .rem_euclid(vw() as f32 + 160.0 * vk())
            - 80.0 * vk();
        let my = hz * 0.38;
        glow(
            fb,
            mx,
            my,
            40.0 * vk(),
            40.0 * vk(),
            rgb(255, 170, 220),
            0.3 * w[6],
            hz,
        );
        ellipse_a(
            fb,
            mx,
            my,
            7.0 * vk(),
            7.0 * vk(),
            rgb(255, 205, 235),
            w[6],
            hz,
        );
    }

    // domain-warped nebula + aurora (night)
    if nw > 0.03 {
        let nt = &g.assets.nebula;
        let off = (g.sky_off * 0.1 * vk() + g.elapsed * 1.5) as i32;
        for y in 0..horizon().min(nt.h as i32) {
            for x in 0..vw() as i32 {
                let c = nt.px[y as usize * nt.w + ((x + off).rem_euclid(nt.w as i32)) as usize];
                let a = (c >> 24) as f32 / 255.0 * nw * 0.8;
                if a > 0.01 {
                    add_px(fb, x, y, unpack(c & 0xFF_FFFF), a);
                }
            }
        }
        for x in 0..vw() as i32 {
            let u = (x as f32 / vk() + g.sky_off * 0.1) / 70.0;
            let t = g.elapsed * 0.12;
            let base = hz * (0.30 + 0.20 * fractal::fbm(u, t, 77, 4, 0));
            let hgt = hz * (0.10 + 0.38 * fractal::fbm(u * 1.7 + 9.0, t * 1.3, 91, 3, 0));
            let amp = (fractal::fbm(u * 3.0, t * 2.0, 13, 3, 0) - 0.38).max(0.0) * 2.4 * nw;
            if amp <= 0.0 {
                continue;
            }
            let top = (base - hgt) as i32;
            for y in top.max(0)..base as i32 {
                let f = (y - top) as f32 / hgt.max(1.0);
                let a = f.powf(2.2) * (1.0 - f).powf(0.3) * amp * 0.9;
                let c = [
                    60.0 + (1.0 - f) * 140.0,
                    255.0 - (1.0 - f) * 175.0,
                    160.0 + (1.0 - f) * 95.0,
                ];
                add_px(fb, x, y, c, a);
            }
        }
    }

    // ridged-multifractal mountain ranges with snow caps
    let snowk = w[0] + 0.8 * w[2] + w[3] + 0.5 * w[6] + 1.2 * w[7];
    for (layer, kk, amp, far) in [(0usize, 0.5f32, 36.0f32, true), (1, 1.0, 20.0, false)] {
        let base_col = if far {
            mix(p.mount, p.sky_hor, 0.45)
        } else {
            p.mount
        };
        let off = (g.sky_off * kk * vk()) as i32 + layer as i32 * 311;
        let table = &g.assets.ridges[layer];
        for x in 0..vw() as i32 {
            let tt = table[((x + off).rem_euclid(1024)) as usize];
            let h = amp * vk() * (0.12 + 0.95 * tt);
            let top = horizon() - h as i32;
            for y in top..horizon() {
                let t = (y - top) as f32 / h.max(1.0);
                let mut col = mix(shade(base_col, 1.18), mix(base_col, p.sky_hor, 0.25), t);
                let snow =
                    ((tt - 0.62) * 4.0).clamp(0.0, 1.0) * (1.0 - t * 4.0).clamp(0.0, 1.0) * snowk;
                if snow > 0.0 {
                    col = mix(col, rgb(236, 242, 252), snow * 0.85);
                }
                fb[y as usize * vw() + x as usize] = col;
            }
        }
    }
    sky_volcano(fb, g, p, w[8]);
    sky_abyss(fb, g, p, w[9]);
    sky_cosmos(fb, g, p, w[10]);
    sky_citadel(fb, g, p, w[11]);
    sky_myco(fb, g, p, w[12]);
    sky_inverted(fb, g, p, w[13]);
    sky_prism(fb, g, p, w[14]);
    sky_clockwork(fb, g, p, w[15]);
    sky_cloudsea(fb, g, p, w[16]);
    sky_blackhole(fb, g, p, w[17]);
    sky_canyon(fb, g, p, w[18]);
    sky_islands(fb, g, p, w[5]);
    sky_mesas(fb, g, p, w[4]);
    sky_world_trees(fb, g, p, w[3]);
    sky_crystals(fb, g, p, w[6]);
    for y in horizon()..vh() as i32 {
        hline(fb, y, 0, vw() as i32, p.grass[0]);
    }
}

/// Deterministic pseudo-random value in 0..1 for index `i` and a salt.
fn h01(i: u32, salt: u32) -> f32 {
    (fx::hash(i.wrapping_mul(2_654_435_761).wrapping_add(salt)) % 10_000) as f32 / 10_000.0
}

/// Distant world-trees: colossal trunks rising through collars of cloud, canopies peeking above them.
fn sky_world_trees(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    let span = vw() as f32 + 300.0 * k;
    for i in 0..4u32 {
        let depth = 0.2 + 0.8 * h01(i, 23);
        let cx =
            (h01(i, 11) * span - g.sky_off * (0.22 + 0.2 * depth) * k).rem_euclid(span) - 150.0 * k;
        let base = (20.0 + 20.0 * depth) * k;
        let h = (56.0 + 52.0 * depth) * k;
        let haze = 0.55 - 0.4 * depth;
        for y in ((hz - h).max(0.0) as i32)..hz as i32 {
            let t = ((hz - y as f32) / h).clamp(0.0, 1.0);
            let hw = base * (1.0 - 0.5 * t) + base * 0.9 * (1.0 - t).powi(7);
            for x in ((cx - hw) as i32).max(0)..((cx + hw) as i32).min(vw() as i32) {
                let dxn = (x as f32 - cx) / hw;
                let gr = fractal::fbm(dxn * 4.0 + i as f32 * 5.0, y as f32 * 0.04, 6, 2, 0);
                let col = shade(
                    rgb(100, 82, 66),
                    (0.6 + 0.8 * gr) * (1.1 - 0.4 * (dxn * 0.5 + 0.5)),
                );
                blend_px(fb, x, y, mix(col, p.sky_hor, haze), a);
            }
        }
        // cloud collar around the trunk and a green crown above it
        let collar = hz - h * 0.55;
        for j in 0..6 {
            let f = j as f32 / 5.0 - 0.5;
            ellipse_a(
                fb,
                cx + f * base * 4.2,
                collar + (h01(i * 9 + j, 5) - 0.5) * 6.0 * k,
                base * 1.1,
                7.0 * k,
                rgb(252, 253, 255),
                0.9 * a,
                hz,
            );
        }
        let ctop = hz - h;
        if ctop > 2.0 {
            for j in 0..6 {
                let f = j as f32 / 5.0 - 0.5;
                ellipse_a(
                    fb,
                    cx + f * base * 4.0,
                    ctop + (h01(i * 5 + j, 8) - 0.4) * 8.0 * k,
                    base * 1.3,
                    9.0 * k,
                    mix(rgb(54, 128, 60), p.sky_hor, haze),
                    0.9 * a,
                    hz,
                );
            }
        }
    }
}

/// Stratified red mesas and buttes in two parallax layers.
fn sky_mesas(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    let span = vw() as f32 + 300.0 * k;
    for layer in 0..2u32 {
        let (n, par, hbase, haze) = if layer == 0 {
            (6u32, 0.45f32, 34.0f32, 0.5f32)
        } else {
            (4, 0.8, 52.0, 0.12)
        };
        for i in 0..n {
            let id = i + layer * 16;
            let cx = (h01(id, 31) * span - g.sky_off * par * k).rem_euclid(span) - 150.0 * k;
            let wt = (26.0 + 46.0 * h01(id, 41)) * k;
            let h = hbase * (0.7 + 0.6 * h01(id, 51)) * k;
            let thumb = h01(id, 61) > 0.6;
            for y in ((hz - h * 1.12).max(0.0) as i32)..hz as i32 {
                let f = ((hz - y as f32) / h).clamp(0.0, 1.2);
                let hw = if f < 0.34 {
                    lerp(wt + h * 0.9, wt * 1.12, f / 0.34)
                } else if f <= 1.0 {
                    wt * (1.0 + 0.07 * (f * 37.0 + id as f32).sin())
                } else if thumb {
                    wt * 0.22
                } else {
                    continue;
                };
                let band = fractal::fbm(0.0, f * 26.0 + id as f32, 21, 2, 0);
                for x in ((cx - hw) as i32).max(0)..((cx + hw) as i32).min(vw() as i32) {
                    let dxn = (x as f32 - cx) / hw;
                    let lit = 1.14 - 0.4 * (dxn * 0.5 + 0.5);
                    let base = if f < 0.34 {
                        rgb(176, 84, 56)
                    } else {
                        rgb(150, 60, 42)
                    };
                    let col = shade(base, (0.78 + 0.5 * band) * lit);
                    blend_px(fb, x, y, mix(col, p.sky_hor, haze), a);
                }
            }
        }
    }
}

/// Floating islands (waterfalls, glowing undersides) and a huge ringed planet.
fn sky_islands(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    // ringed gas giant
    let (px, py, pr) = (
        vw() as f32 * 0.24 - g.sky_off * 0.03 * k,
        hz * 0.34,
        30.0 * k,
    );
    for pass in 0..2 {
        if pass == 1 {
            ellipse_a(fb, px, py, pr, pr, rgb(238, 176, 96), 0.92 * a, hz);
            ellipse_a(
                fb,
                px,
                py - pr * 0.25,
                pr * 0.9,
                pr * 0.1,
                rgb(200, 120, 70),
                0.6 * a,
                hz,
            );
            ellipse_a(
                fb,
                px,
                py + pr * 0.3,
                pr * 0.95,
                pr * 0.12,
                rgb(255, 214, 150),
                0.5 * a,
                hz,
            );
        }
        for i in 0..400 {
            let th = i as f32 / 400.0 * std::f32::consts::TAU;
            let front = th.sin() > 0.0;
            if (pass == 1) != front {
                continue;
            }
            let (x, y) = (px + th.cos() * pr * 2.0, py + th.sin() * pr * 0.34);
            for d in 0..3 {
                add_px(
                    fb,
                    x as i32,
                    y as i32 + d - 1,
                    [255.0, 220.0, 170.0],
                    0.55 * a,
                );
            }
        }
    }
    let span = vw() as f32 + 300.0 * k;
    for i in 0..6u32 {
        let near = h01(i, 71);
        let cx =
            (h01(i, 73) * span - g.sky_off * (0.14 + 0.1 * near) * k).rem_euclid(span) - 150.0 * k;
        let cy = hz * (0.30 + 0.4 * h01(i, 79));
        let rx = (22.0 + 30.0 * near) * k;
        let depth = (28.0 + 30.0 * near) * k;
        let haze = 0.4 - 0.3 * near;
        for r in 0..depth as i32 {
            let f = r as f32 / depth;
            let jag = 0.8 + 0.4 * fractal::fbm(f * 5.0, i as f32 * 3.0, 17, 3, 0);
            let hw = rx * (1.0 - f).powf(0.8) * jag;
            let col = mix(shade(rgb(118, 100, 96), 1.1 - 0.55 * f), p.sky_hor, haze);
            for x in ((cx - hw) as i32).max(0)..((cx + hw) as i32).min(vw() as i32) {
                blend_px(fb, x, cy as i32 + r, col, a);
            }
        }
        ellipse_a(
            fb,
            cx,
            cy,
            rx * 1.04,
            rx * 0.16,
            mix(rgb(96, 204, 124), p.sky_hor, haze),
            a,
            hz,
        );
        for t in 0..3 {
            let tx = cx + (t as f32 - 1.0) * rx * 0.5;
            tri(
                fb,
                tx,
                cy - 7.0 * k,
                cy,
                3.0 * k,
                mix(rgb(40, 120, 70), p.sky_hor, haze),
                hz,
            );
        }
        glow(
            fb,
            cx,
            cy + depth,
            rx * 1.1,
            depth * 0.5,
            rgb(130, 236, 255),
            0.35 * a,
            hz,
        );
        if near > 0.35 {
            // waterfall off the rim, shimmering
            let wx = cx + rx * 0.62;
            for y in (cy as i32)..((cy + depth * 1.4) as i32).min(hz as i32) {
                let sh = 0.5 + 0.5 * (g.elapsed * 6.0 - y as f32 * 0.5).sin();
                let fade = 1.0 - (y as f32 - cy) / (depth * 1.4);
                add_px(fb, wx as i32, y, [170.0, 230.0, 255.0], 0.7 * sh * fade * a);
            }
        }
    }
}

/// Glowing crystal spikes along the horizon of the mirror flats.
fn sky_crystals(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.04 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    let span = vw() as f32 + 200.0 * k;
    for i in 0..22u32 {
        let cx = (h01(i, 91) * span - g.sky_off * 0.6 * k).rem_euclid(span) - 100.0 * k;
        let h = (8.0 + 26.0 * h01(i, 93).powi(2)) * k;
        let hue = [rgb(70, 220, 240), rgb(200, 90, 240), rgb(110, 150, 255)][(i % 3) as usize];
        let c = mix(p.sky_hor, hue, 0.55 * a);
        tri(
            fb,
            cx,
            hz - h,
            hz,
            (2.5 + 3.0 * h01(i, 95)) * k,
            shade(c, 0.9),
            hz,
        );
        glow(fb, cx, hz - h * 0.4, 9.0 * k, h * 0.6, hue, 0.25 * a, hz);
    }
}

pub fn render(g: &Game, fb: &mut [u32]) {
    render_view(g, fb, g.vw, g.vh);
}

/// Draw text on a buffer of the given size (used by split-screen overlays).
pub fn overlay_text(fb: &mut [u32], w: usize, h: usize, y: i32, s: &str, scale: i32, c: u32) {
    DIM.with(|d| d.set((w, h)));
    text_center(fb, y, s, scale, c);
}

pub fn overlay_text_at(
    fb: &mut [u32],
    w: usize,
    h: usize,
    x: i32,
    y: i32,
    s: &str,
    scale: i32,
    c: u32,
) {
    DIM.with(|d| d.set((w, h)));
    text_shadow(fb, x, y, s, scale, c);
}

pub fn render_view(g: &Game, fb: &mut [u32], vw_: usize, vh_: usize) {
    DIM.with(|d| d.set((vw_, vh_)));
    assert_eq!(fb.len(), vw_ * vh_);
    let env = env_of(g);
    let mut p = blend_pal(g.biome_w);
    env.sky(&mut p);
    {
        // day and night: sky colours follow the time of day, warm at dawn and dusk
        let l = g.daylight();
        let ss = g.sunset();
        p.sky_top = mix(rgb(6, 8, 26), p.sky_top, l);
        p.sky_hor = mix(rgb(24, 30, 64), p.sky_hor, l);
        p.mount = mix(rgb(14, 16, 36), p.mount, l);
        p.sky_hor = mix(p.sky_hor, rgb(255, 128, 64), ss * 0.7);
        p.sky_top = mix(p.sky_top, rgb(120, 70, 130), ss * 0.35);
    }
    draw_sky(fb, g, &p);

    let n = g.track.segs.len();
    let base = (g.pos / SEG_LEN) as usize;
    let base_pct = (g.pos % SEG_LEN) / SEG_LEN;
    let pz = g.pos + PLAYER_Z;
    let ps = g.track.seg_at(pz);
    let player_y = lerp(ps.y0, ps.y1, (pz % SEG_LEN) / SEG_LEN);
    let cam_y = CAM_H + player_y + g.photo_dy;
    let cam_x = (g.x + g.photo_dx) * ROAD_W;
    let base_seg = &g.track.segs[base % n];

    let mut proj = vec![SegProj::default(); DRAW_DIST];
    let mut maxy = vh() as f32;
    let mut x = 0.0f32;
    let mut dx = -(base_seg.curve * base_pct);

    for k in 0..DRAW_DIST {
        let idx = base + k;
        let seg = &g.track.segs[idx % n];
        let z0 = idx as f32 * SEG_LEN;
        let z1 = z0 + SEG_LEN;
        let (s1, x1, y1, w1) = project(0.0, seg.y0, z0, cam_x - x, cam_y, g.pos);
        let (s2, x2, y2, w2) = project(0.0, seg.y1, z1, cam_x - x - dx, cam_y, g.pos);
        x += dx;
        dx += seg.curve;
        if z0 - g.pos <= CAM_DEPTH || y2 >= y1 || y2 >= maxy {
            continue;
        }
        proj[k] = SegProj {
            ok: true,
            s1,
            x1,
            y1,
            s2,
            x2,
            y2,
            clip: maxy,
        };

        let band = (idx / RUMBLE_LEN) % 2;
        let in_thin = g.surface == crate::env::Surface::IceLake
            && g.thin.iter().any(|&(a, b)| z0 >= a && z0 < b);
        let mut pp = if seg.feat == F_TUNNEL {
            // indoors: no snow, no puddles, dim concrete
            let mut q = pal(seg.biome as usize);
            q.road = q.road.map(|c| shade(c, 0.55));
            q.rumble = q.rumble.map(|c| shade(c, 0.5));
            q
        } else {
            env.apply(pal(seg.biome as usize))
        };
        if in_thin {
            // thin ice: darker, greenish-blue and webbed with cracks
            pp.road = pp.road.map(|c| mix(c, rgb(60, 112, 150), 0.5));
        }
        let tone = seg.tone;
        let ground = if seg.biome == 6 {
            // mirror lake: the ground reflects the sky, with faint ripple bands
            let hzf = horizon() as f32;
            let t = (((y1 + y2) * 0.5 - hzf) / (vh() as f32 - hzf)).clamp(0.0, 1.0);
            let sky = mix(p.sky_hor, p.sky_top, t.powf(0.7));
            shade(
                mix(sky, pp.grass[band], 0.35),
                0.8 + 0.1 * band as f32 + 0.1 * tone,
            )
        } else {
            ground_color(seg.biome, &pp, band, tone, idx, g.elapsed)
        };
        let ground = match seg.feat {
            F_BRIDGE => {
                let r = ((idx as f32 * 0.7 + g.elapsed * 2.0).sin() + 1.0) * 0.5;
                mix(
                    rgb(28, 88, 148),
                    rgb(110, 184, 226),
                    0.12 + 0.2 * r * (0.5 + band as f32 * 0.5),
                )
            }
            F_VIADUCT => mix(shade(pp.grass[band], 0.4), p.sky_hor, 0.35),
            F_TUNNEL => shade(rgb(40, 42, 50), 0.85 + 0.15 * band as f32),
            F_PASS => shade(rgb(104, 94, 86), 0.8 + 0.2 * band as f32 + 0.1 * tone),
            _ if city_like(g, idx % n) => shade(rgb(86, 88, 96), 0.88 + 0.12 * band as f32),
            _ => ground,
        };
        rect(fb, 0.0, y2, vw() as f32, y1, ground, maxy);
        let (rw1, rw2) = (w1 / 6.0, w2 / 6.0);
        quad(
            fb,
            y2,
            x2 - w2 - rw2,
            x2 + w2 + rw2,
            y1,
            x1 - w1 - rw1,
            x1 + w1 + rw1,
            maxy,
            pp.rumble[band],
        );
        // road with a subtle darker shoulder and lighter crown for depth
        quad(
            fb,
            y2,
            x2 - w2,
            x2 + w2,
            y1,
            x1 - w1,
            x1 + w1,
            maxy,
            shade(pp.road[band], 0.9 + 0.08 * tone),
        );
        quad(
            fb,
            y2,
            x2 - w2 * 0.8,
            x2 + w2 * 0.8,
            y1,
            x1 - w1 * 0.8,
            x1 + w1 * 0.8,
            maxy,
            pp.road[band],
        );
        quad(
            fb,
            y2,
            x2 - w2 * 0.35,
            x2 + w2 * 0.35,
            y1,
            x1 - w1 * 0.35,
            x1 + w1 * 0.35,
            maxy,
            shade(pp.road[band], 1.06),
        );
        // edge lines
        let e1 = w1 / 40.0;
        let e2 = w2 / 40.0;
        for sgn in [-1.0f32, 1.0] {
            quad(
                fb,
                y2,
                x2 + sgn * w2 * 0.94 - e2,
                x2 + sgn * w2 * 0.94 + e2,
                y1,
                x1 + sgn * w1 * 0.94 - e1,
                x1 + sgn * w1 * 0.94 + e1,
                maxy,
                pp.lane,
            );
        }
        if band == 0 {
            let (l1, l2) = (w1 / 32.0, w2 / 32.0);
            for lane in 1..3 {
                let t = lane as f32 / 3.0;
                let lx1 = x1 - w1 + w1 * 2.0 * t;
                let lx2 = x2 - w2 + w2 * 2.0 * t;
                quad(
                    fb,
                    y2,
                    lx2 - l2,
                    lx2 + l2,
                    y1,
                    lx1 - l1,
                    lx1 + l1,
                    maxy,
                    pp.lane,
                );
            }
        }
        if in_thin && idx.is_multiple_of(2) {
            // jagged crack lines across the ice
            let hsh = fx::hash(idx as u32 ^ 0xC4AC);
            for c in 0..2u32 {
                let a = ((hsh >> (c * 7)) % 100) as f32 / 100.0 * 1.6 - 0.8;
                let lean = (((hsh >> (c * 5 + 3)) % 100) as f32 / 100.0 - 0.5) * 0.7;
                let (e1, e2) = (w1 / 90.0, w2 / 90.0);
                quad(
                    fb,
                    y2,
                    x2 + w2 * (a + lean) - e2,
                    x2 + w2 * (a + lean) + e2,
                    y1,
                    x1 + w1 * a - e1,
                    x1 + w1 * a + e1,
                    maxy,
                    rgb(16, 34, 60),
                );
            }
        }
        maxy = y1;
    }

    // headlight pool on the road at night
    let night = (g.biome_w[2] + g.biome_w[6] + 0.4 * g.biome_w[4]).min(1.0);
    if night > 0.05 {
        glow(
            fb,
            vw() as f32 / 2.0 + g.steer * 6.0 * vk(),
            vh() as f32 * 0.8,
            170.0 * vk(),
            48.0 * vk(),
            rgb(255, 236, 190),
            0.26 * night,
            vh() as f32,
        );
    }

    // structures: tunnel linings, bridge and viaduct parapets, mountain-pass walls
    let in_tunnel = g.feat_here() == F_TUNNEL;
    {
        // everything above the nearest tunnel ring is ceiling
        if let Some(sp) = proj
            .iter()
            .enumerate()
            .find(|(k, s)| s.ok && g.track.segs[(base + k) % n].feat == F_TUNNEL)
        {
            let top = sp.1.y1 - sp.1.s1 * ROAD_W * vw() as f32 / 2.0 * 0.8;
            if top > 0.0 {
                rect(fb, 0.0, 0.0, vw() as f32, top, rgb(26, 27, 33), vh() as f32);
            }
        }
        for k in (0..DRAW_DIST).rev() {
            let sp = proj[k];
            let idx = base + k;
            let seg = &g.track.segs[idx % n];
            if !sp.ok || seg.feat == F_NONE {
                continue;
            }
            let (x1, y1, w1) = (sp.x1, sp.y1, sp.s1 * ROAD_W * vw() as f32 / 2.0);
            let (x2, y2, w2) = (sp.x2, sp.y2, sp.s2 * ROAD_W * vw() as f32 / 2.0);
            let band = (idx / RUMBLE_LEN) % 2;
            let dz = CAM_DEPTH / sp.s1;
            let dark = ((dz - 6000.0) / (DRAW_DIST as f32 * SEG_LEN - 6000.0))
                .clamp(0.0, 1.0)
                .powf(0.8);
            let clip = sp.clip;
            match seg.feat {
                F_TUNNEL => {
                    let (h1, h2) = (w1 * 0.8, w2 * 0.8);
                    let wall = mix(rgb(58, 60, 70), rgb(4, 4, 8), dark * 0.85);
                    let wall2 = mix(rgb(48, 50, 58), rgb(4, 4, 8), dark * 0.85);
                    let ceil = mix(rgb(40, 42, 50), rgb(4, 4, 8), dark * 0.85);
                    poly4(
                        fb,
                        [
                            (x1 - w1, y1 - h1),
                            (x1 + w1, y1 - h1),
                            (x2 + w2, y2 - h2),
                            (x2 - w2, y2 - h2),
                        ],
                        clip,
                        ceil,
                    );
                    for sgn in [-1.0f32, 1.0] {
                        poly4(
                            fb,
                            [
                                (x1 + sgn * w1, y1 - h1),
                                (x1 + sgn * w1, y1),
                                (x2 + sgn * w2, y2),
                                (x2 + sgn * w2, y2 - h2),
                            ],
                            clip,
                            if band == 0 { wall } else { wall2 },
                        );
                        // everything beyond the wall is solid
                        poly4(
                            fb,
                            [
                                (x1 + sgn * w1 * 6.0, y1 - h1 * 3.0),
                                (x1 + sgn * w1, y1 - h1 * 3.0),
                                (x1 + sgn * w1, y1),
                                (x1 + sgn * w1 * 6.0, y1),
                            ],
                            clip,
                            wall2,
                        );
                        // reflective stripe low on the wall
                        poly4(
                            fb,
                            [
                                (x1 + sgn * w1, y1 - h1 * 0.22),
                                (x1 + sgn * w1, y1 - h1 * 0.16),
                                (x2 + sgn * w2, y2 - h2 * 0.16),
                                (x2 + sgn * w2, y2 - h2 * 0.22),
                            ],
                            clip,
                            mix(rgb(255, 170, 40), rgb(4, 4, 8), dark * 0.8),
                        );
                    }
                    if idx.is_multiple_of(5) {
                        let lamp = mix(rgb(255, 240, 190), rgb(20, 18, 10), dark * 0.7);
                        poly4(
                            fb,
                            [
                                (x1 - w1 * 0.14, y1 - h1),
                                (x1 + w1 * 0.14, y1 - h1),
                                (x2 + w2 * 0.14, y2 - h2),
                                (x2 - w2 * 0.14, y2 - h2),
                            ],
                            clip,
                            lamp,
                        );
                        glow(
                            fb,
                            x1,
                            y1 - h1 * 0.95,
                            w1 * 0.5,
                            w1 * 0.22,
                            rgb(255, 226, 150),
                            0.35 * (1.0 - dark),
                            clip,
                        );
                    }
                }
                F_BRIDGE | F_VIADUCT => {
                    let tall = if seg.feat == F_VIADUCT { 0.16 } else { 0.10 };
                    let (h1, h2) = (w1 * tall, w2 * tall);
                    let face = if band == 0 {
                        rgb(204, 204, 210)
                    } else {
                        rgb(150, 154, 166)
                    };
                    let face = mix(face, p.sky_hor, dark * 0.5);
                    for sgn in [-1.0f32, 1.0] {
                        let (e1, e2) = (x1 + sgn * w1 * 1.03, x2 + sgn * w2 * 1.03);
                        poly4(
                            fb,
                            [(e1, y1 - h1), (e1, y1), (e2, y2), (e2, y2 - h2)],
                            clip,
                            face,
                        );
                        seg_line(
                            fb,
                            (e1, y1 - h1),
                            (e2, y2 - h2),
                            (w1 * 0.008).max(0.6),
                            clip,
                            rgb(70, 74, 86),
                        );
                        if idx.is_multiple_of(3) {
                            let pw = (w1 * 0.014).max(0.8);
                            poly4(
                                fb,
                                [
                                    (e1 - pw, y1 - h1 * 1.5),
                                    (e1 + pw, y1 - h1 * 1.5),
                                    (e1 + pw, y1),
                                    (e1 - pw, y1),
                                ],
                                clip,
                                rgb(90, 94, 108),
                            );
                        }
                        if seg.feat == F_VIADUCT && idx.is_multiple_of(40) {
                            // cable-stay pylon
                            let ht = w1 * 1.2;
                            let tw = (w1 * 0.03).max(1.0);
                            poly4(
                                fb,
                                [
                                    (e1 - tw, y1 - ht),
                                    (e1 + tw, y1 - ht),
                                    (e1 + tw, y1),
                                    (e1 - tw, y1),
                                ],
                                clip,
                                rgb(176, 176, 186),
                            );
                            for j in [5usize, 10, 15] {
                                if let Some(q) = proj.get(k + j).filter(|q| q.ok) {
                                    let qw = q.s1 * ROAD_W * vw() as f32 / 2.0;
                                    seg_line(
                                        fb,
                                        (e1, y1 - ht),
                                        (q.x1 + sgn * qw * 1.03, q.y1 - qw * tall),
                                        0.7,
                                        clip,
                                        rgb(200, 200, 208),
                                    );
                                }
                            }
                        }
                    }
                }
                F_PASS => {
                    let hv = 1.5 + 0.6 * h01(idx as u32, 77);
                    let hv2 = 1.5 + 0.6 * h01(idx as u32 + 1, 77);
                    let (h1, h2) = (w1 * hv, w2 * hv2);
                    let lim = feat_limit(F_PASS);
                    let rock = if band == 0 {
                        rgb(118, 106, 96)
                    } else {
                        rgb(98, 88, 80)
                    };
                    let rock = mix(rock, p.sky_hor, dark * 0.45);
                    for sgn in [-1.0f32, 1.0] {
                        let (e1, e2) = (x1 + sgn * w1 * lim, x2 + sgn * w2 * lim);
                        poly4(
                            fb,
                            [(e1, y1 - h1), (e1, y1), (e2, y2), (e2, y2 - h2)],
                            clip,
                            rock,
                        );
                        poly4(
                            fb,
                            [
                                (e1 + sgn * w1 * 6.0, y1 - h1),
                                (e1, y1 - h1),
                                (e1, y1),
                                (e1 + sgn * w1 * 6.0, y1),
                            ],
                            clip,
                            shade(rock, 0.82),
                        );
                        // strata
                        if idx.is_multiple_of(2) {
                            poly4(
                                fb,
                                [
                                    (e1, y1 - h1 * 0.55),
                                    (e1, y1 - h1 * 0.5),
                                    (e2, y2 - h2 * 0.5),
                                    (e2, y2 - h2 * 0.55),
                                ],
                                clip,
                                shade(rock, 0.7),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // fuel-station pads on the road
    for k in (0..DRAW_DIST).rev() {
        let sp = proj[k];
        if !sp.ok {
            continue;
        }
        let z0 = (base + k) as f32 * SEG_LEN;
        if g.stations.iter().any(|&(a, b)| z0 >= a && z0 < b) {
            let (w1, w2) = (
                sp.s1 * ROAD_W * vw() as f32 / 2.0,
                sp.s2 * ROAD_W * vw() as f32 / 2.0,
            );
            let alt = ((base + k) / 2).is_multiple_of(2);
            poly4(
                fb,
                [
                    (sp.x2 + w2 * 0.2, sp.y2),
                    (sp.x2 + w2 * 0.98, sp.y2),
                    (sp.x1 + w1 * 0.98, sp.y1),
                    (sp.x1 + w1 * 0.2, sp.y1),
                ],
                sp.clip,
                if alt {
                    rgb(250, 210, 40)
                } else {
                    rgb(40, 40, 46)
                },
            );
        }
    }

    // railway crossings: gravel bed and rails across the road
    for k in (0..DRAW_DIST).rev() {
        let sp = proj[k];
        if !sp.ok {
            continue;
        }
        let z0 = (base + k) as f32 * SEG_LEN;
        if g.crossings.iter().any(|c| c.z >= z0 && c.z < z0 + SEG_LEN) {
            let (w1, w2) = (
                sp.s1 * ROAD_W * vw() as f32 / 2.0,
                sp.s2 * ROAD_W * vw() as f32 / 2.0,
            );
            poly4(
                fb,
                [
                    (sp.x2 - w2 * 1.2, sp.y2),
                    (sp.x2 + w2 * 1.2, sp.y2),
                    (sp.x1 + w1 * 1.2, sp.y1),
                    (sp.x1 - w1 * 1.2, sp.y1),
                ],
                sp.clip,
                rgb(74, 66, 58),
            );
            for tt in [0.3f32, 0.7] {
                let y = lerp(sp.y2, sp.y1, tt);
                let (xc, w) = (lerp(sp.x2, sp.x1, tt), lerp(w2, w1, tt));
                let th = (w * 0.02).max(1.0);
                poly4(
                    fb,
                    [
                        (xc - w * 1.2, y),
                        (xc + w * 1.2, y),
                        (xc + w * 1.2, y + th),
                        (xc - w * 1.2, y + th),
                    ],
                    sp.clip,
                    rgb(196, 198, 206),
                );
            }
        }
    }

    // sprites, far to near
    for k in (0..DRAW_DIST).rev() {
        let sp = proj[k];
        if !sp.ok {
            continue;
        }
        let idx = base + k;
        let seg = &g.track.segs[idx % n];
        draw_city_seg(fb, g, &sp, idx, n, p.sky_hor);
        draw_emergency_seg(fb, g, &sp, idx, n);
        for pr in &seg.props {
            let ux = sp.s1 * vw() as f32 / 2.0;
            let cx = sp.x1 + sp.s1 * pr.offset * ROAD_W * vw() as f32 / 2.0;
            let dz = CAM_DEPTH / sp.s1;
            let fogamt = ((dz - 9000.0) / (DRAW_DIST as f32 * SEG_LEN - 9000.0))
                .clamp(0.0, 1.0)
                .powf(1.3)
                * 0.7;
            draw_prop(
                fb,
                pr.kind,
                cx,
                sp.y1,
                ux,
                sp.clip,
                seg.biome,
                pr.offset,
                &g.assets,
                pr.variant,
                (p.sky_hor, fogamt),
            );
        }
        for o in &g.orbs {
            if o.taken || (o.z / SEG_LEN) as usize != idx % n {
                continue;
            }
            let pct = (o.z % SEG_LEN) / SEG_LEN;
            let sc = lerp(sp.s1, sp.s2, pct);
            let ox = lerp(sp.x1, sp.x2, pct) + sc * o.x * ROAD_W * vw() as f32 / 2.0;
            let oy = lerp(sp.y1, sp.y2, pct);
            let u = sc * vw() as f32 / 2.0;
            let fogamt = (((CAM_DEPTH / sp.s1) - 9000.0) / (DRAW_DIST as f32 * SEG_LEN - 9000.0))
                .clamp(0.0, 1.0)
                * 0.6;
            if o.kind == 0 {
                let col = mix(
                    hsv(g.elapsed * 0.5 + o.z * 0.0003, 0.7, 1.0),
                    p.sky_hor,
                    fogamt,
                );
                let cy = oy - 420.0 * u;
                glow(fb, ox, cy, 520.0 * u, 520.0 * u, col, 0.6, sp.clip);
                ellipse(fb, ox, cy, 210.0 * u, 210.0 * u, col, sp.clip);
                ellipse(
                    fb,
                    ox - 60.0 * u,
                    cy - 70.0 * u,
                    70.0 * u,
                    60.0 * u,
                    rgb(255, 255, 255),
                    sp.clip,
                );
            } else {
                let half = 0.42 * ROAD_W * u;
                let col = mix(rgb(255, 80, 220), p.sky_hor, fogamt);
                let top = oy - 1500.0 * u;
                rect(
                    fb,
                    ox - half - 55.0 * u,
                    top,
                    ox - half + 55.0 * u,
                    oy,
                    col,
                    sp.clip,
                );
                rect(
                    fb,
                    ox + half - 55.0 * u,
                    top,
                    ox + half + 55.0 * u,
                    oy,
                    col,
                    sp.clip,
                );
                rect(fb, ox - half, top, ox + half, top + 110.0 * u, col, sp.clip);
                glow(
                    fb,
                    ox,
                    top + 700.0 * u,
                    half * 1.1,
                    760.0 * u,
                    rgb(255, 120, 240),
                    0.25,
                    sp.clip,
                );
            }
        }
        for it in &g.items {
            if it.taken || (it.z / SEG_LEN) as usize != idx % n {
                continue;
            }
            let pct = (it.z % SEG_LEN) / SEG_LEN;
            let sc = lerp(sp.s1, sp.s2, pct);
            let ix = lerp(sp.x1, sp.x2, pct) + sc * it.x * ROAD_W * vw() as f32 / 2.0;
            let iy = lerp(sp.y1, sp.y2, pct);
            let u = sc * vw() as f32 / 2.0;
            let fogamt = (((CAM_DEPTH / sp.s1) - 9000.0) / (DRAW_DIST as f32 * SEG_LEN - 9000.0))
                .clamp(0.0, 1.0)
                * 0.6;
            draw_item(fb, it, ix, iy, u, sp.clip, g.elapsed, (p.sky_hor, fogamt));
        }
        let fog_k = (((CAM_DEPTH / sp.s1) - 9000.0) / (DRAW_DIST as f32 * SEG_LEN - 9000.0))
            .clamp(0.0, 1.0)
            * 0.6;
        let fogf = |c: u32| mix(c, p.sky_hor, fog_k);
        for &(a, _) in &g.stations {
            let s0 = (a / SEG_LEN) as usize;
            if s0 % n == idx % n {
                let u = sp.s1 * vw() as f32 / 2.0;
                draw_station(
                    fb,
                    sp.x1 + sp.s1 * 1.55 * ROAD_W * vw() as f32 / 2.0,
                    sp.y1,
                    u,
                    sp.clip,
                    g.elapsed,
                    &fogf,
                );
            }
            if (s0 + n - 26) % n == idx % n && seg.feat == F_NONE {
                let u = sp.s1 * vw() as f32 / 2.0;
                draw_station_sign(
                    fb,
                    sp.x1 + sp.s1 * 1.25 * ROAD_W * vw() as f32 / 2.0,
                    sp.y1,
                    u,
                    sp.clip,
                    &fogf,
                );
            }
        }
        for c in &g.crossings {
            if (c.z / SEG_LEN) as usize != idx % n {
                continue;
            }
            let pct = (c.z % SEG_LEN) / SEG_LEN;
            let sc = lerp(sp.s1, sp.s2, pct);
            let u = sc * vw() as f32 / 2.0;
            let cx = lerp(sp.x1, sp.x2, pct);
            let by = lerp(sp.y1, sp.y2, pct);
            let warn = c.warning(g.elapsed);
            let blink = (g.elapsed * 4.0) as i32 % 2 == 0;
            for s in [-1.0f32, 1.0] {
                let px = cx + s * 2150.0 * u;
                rect(
                    fb,
                    px - 28.0 * u,
                    by - 900.0 * u,
                    px + 28.0 * u,
                    by,
                    fogf(rgb(220, 220, 226)),
                    sp.clip,
                );
                for (i, on) in [(0, blink), (1, !blink)] {
                    let lx = px + (i as f32 * 2.0 - 1.0) * 80.0 * u * -s;
                    let col = if warn && on {
                        rgb(255, 40, 40)
                    } else {
                        rgb(70, 24, 24)
                    };
                    if warn && on {
                        glow(
                            fb,
                            lx,
                            by - 820.0 * u,
                            200.0 * u,
                            200.0 * u,
                            rgb(255, 60, 60),
                            0.5,
                            sp.clip,
                        );
                    }
                    ellipse(fb, lx, by - 820.0 * u, 55.0 * u, 55.0 * u, col, sp.clip);
                }
                if warn {
                    // barrier arm across half of the road, red and white stripes
                    let x0 = px;
                    let x1 = cx + s * 80.0 * u;
                    for j in 0..8 {
                        let (a, b) = (
                            lerp(x0, x1, j as f32 / 8.0),
                            lerp(x0, x1, (j + 1) as f32 / 8.0),
                        );
                        let col = if j % 2 == 0 {
                            rgb(240, 240, 240)
                        } else {
                            rgb(220, 30, 30)
                        };
                        rect(
                            fb,
                            a.min(b),
                            by - 560.0 * u,
                            a.max(b),
                            by - 470.0 * u,
                            fogf(col),
                            sp.clip,
                        );
                    }
                } else {
                    seg_line(
                        fb,
                        (px, by - 560.0 * u),
                        (px, by - 1400.0 * u),
                        24.0 * u,
                        sp.clip,
                        fogf(rgb(230, 230, 230)),
                    );
                }
            }
            if c.train_on(g.elapsed) {
                let p01 = c.progress(g.elapsed);
                let (wl, gap, nw) = (1300.0f32, 90.0f32, 5usize);
                let ltr = nw as f32 * (wl + gap);
                let xa = cx + (-2300.0 - ltr + p01 * (4600.0 + ltr)) * u;
                for i in 0..nw {
                    let x0 = xa + i as f32 * (wl + gap) * u;
                    let col = if i == nw - 1 {
                        rgb(232, 190, 40)
                    } else if i % 2 == 0 {
                        rgb(56, 80, 120)
                    } else {
                        rgb(96, 70, 56)
                    };
                    rect(
                        fb,
                        x0,
                        by - 980.0 * u,
                        x0 + wl * u,
                        by - 140.0 * u,
                        fogf(col),
                        sp.clip,
                    );
                    rect(
                        fb,
                        x0,
                        by - 980.0 * u,
                        x0 + wl * u,
                        by - 900.0 * u,
                        fogf(shade(col, 0.7)),
                        sp.clip,
                    );
                    rect(
                        fb,
                        x0,
                        by - 160.0 * u,
                        x0 + wl * u,
                        by - 60.0 * u,
                        fogf(rgb(24, 24, 28)),
                        sp.clip,
                    );
                    for wdw in 0..3 {
                        let wx = x0 + (150.0 + wdw as f32 * 400.0) * u;
                        rect(
                            fb,
                            wx,
                            by - 700.0 * u,
                            wx + 200.0 * u,
                            by - 520.0 * u,
                            fogf(rgb(210, 230, 250)),
                            sp.clip,
                        );
                    }
                }
                glow(
                    fb,
                    xa + ltr * u,
                    by - 600.0 * u,
                    300.0 * u,
                    200.0 * u,
                    rgb(255, 240, 180),
                    0.5,
                    sp.clip,
                );
            }
        }
        for a in &g.animals {
            if a.done || (a.z / SEG_LEN) as usize != idx % n {
                continue;
            }
            let pct = (a.z % SEG_LEN) / SEG_LEN;
            let sc = lerp(sp.s1, sp.s2, pct);
            let ax = lerp(sp.x1, sp.x2, pct) + sc * a.x * ROAD_W * vw() as f32 / 2.0;
            let ay = lerp(sp.y1, sp.y2, pct);
            draw_animal(
                fb,
                a,
                ax,
                ay,
                sc * vw() as f32 / 2.0,
                sp.clip,
                g.elapsed,
                (p.sky_hor, fog_k),
            );
        }
        for c in &g.cops {
            if (c.z.rem_euclid(g.track.length) / SEG_LEN) as usize != idx % n {
                continue;
            }
            let pct = (c.z.rem_euclid(SEG_LEN)) / SEG_LEN;
            let s = lerp(sp.s1, sp.s2, pct);
            let cx = lerp(sp.x1, sp.x2, pct) + s * c.x * ROAD_W * vw() as f32 / 2.0;
            let by = lerp(sp.y1, sp.y2, pct);
            let w = s * CAR_W * vw() as f32 / 2.0;
            draw_police(fb, cx, by, w, g.elapsed, sp.clip);
        }
        for rv in &g.pack {
            if rv.z < 0.0 || (rv.z.rem_euclid(g.track.length) / SEG_LEN) as usize != idx % n {
                continue;
            }
            let pct = rv.z.rem_euclid(SEG_LEN) / SEG_LEN;
            let s = lerp(sp.s1, sp.s2, pct);
            let cx = lerp(sp.x1, sp.x2, pct) + s * rv.x * ROAD_W * vw() as f32 / 2.0;
            let by = lerp(sp.y1, sp.y2, pct);
            let w = s * CAR_W * vw() as f32 / 2.0;
            draw_vehicle(
                fb,
                cx,
                by,
                w,
                CAR_COLORS[rv.color as usize % 7],
                sp.clip,
                true,
                0,
            );
        }
        if let Some(p) = &g.pursuit {
            if (p.z.rem_euclid(g.track.length) / SEG_LEN) as usize == idx % n {
                let pct = p.z.rem_euclid(SEG_LEN) / SEG_LEN;
                let s = lerp(sp.s1, sp.s2, pct);
                let cx = lerp(sp.x1, sp.x2, pct) + s * p.x * ROAD_W * vw() as f32 / 2.0;
                let by = lerp(sp.y1, sp.y2, pct);
                let w = s * CAR_W * vw() as f32 / 2.0;
                draw_vehicle(fb, cx, by, w, rgb(20, 20, 24), sp.clip, false, 0);
            }
        }
        let jam_iter: &[crate::game::Car] = if g.detour.is_some() { &[] } else { &g.jam_cars };
        for car in g.cars.iter().chain(jam_iter.iter()) {
            if (car.z / SEG_LEN) as usize != idx % n {
                continue;
            }
            let pct = (car.z % SEG_LEN) / SEG_LEN;
            let s = lerp(sp.s1, sp.s2, pct);
            let cx = lerp(sp.x1, sp.x2, pct) + s * car.x * ROAD_W * vw() as f32 / 2.0;
            let by = lerp(sp.y1, sp.y2, pct);
            let w = s * CAR_W * vw() as f32 / 2.0;
            draw_vehicle(
                fb,
                cx,
                by,
                w,
                CAR_COLORS[car.color as usize % 7],
                sp.clip,
                false,
                car.kind,
            );
        }
    }

    draw_patrol(fb, g, &proj, base, n);
    if !in_tunnel {
        day_night(fb, g);
    }
    draw_ghost(fb, g, &proj);
    draw_player(fb, g);
    draw_particles(fb, g);
    if !in_tunnel {
        weather(fb, g);
        weather_fx(fb, g, &env);
    }
    if g.mode == Mode::Blackout {
        blackout(fb, g);
    }

    fx::post(
        fb,
        &FxParams {
            w: vw(),
            h: vh(),
            speed: g.speed / MAX_SPEED,
            turbo: g.turbo_t > 0.0,
            shake: g.shake,
            biome_w: g.biome_w,
            t: g.elapsed,
            haze: p.sky_hor,
        },
    );
    if !g.photo || g.photo_hud {
        draw_hud(fb, g);
        draw_hud_extra(fb, g);
        draw_radar(fb, g);
        draw_limit_sign(fb, g);
        draw_mission_hud(fb, g);
        draw_slope(fb, g);
    }
    if g.paused {
        draw_pause(fb, vh_);
    }
}

/// Dims the picture and shows the pause overlay.
fn draw_pause(fb: &mut [u32], h: usize) {
    for px in fb.iter_mut() {
        *px = shade(*px, 0.3);
    }
    let y = h as i32 / 2;
    text_center(fb, y - 30, i18n::tr(S::Paused), 4, rgb(255, 210, 63));
    text_center(fb, y + 14, i18n::tr(S::PauseHint), 1, rgb(255, 255, 255));
}
