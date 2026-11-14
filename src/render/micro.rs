//! Fractal micro-texture for the road and the ground.
//!
//! The road and verge are flat colours per segment; this pass lays tileable fractal detail
//! over them (asphalt grain, pebbles, crack networks and repair patches; multi-scale
//! domain-warped soil and turf). Texture coordinates are anchored to the track (lateral
//! position in road half-widths, distance in segment units), so the detail rides with the
//! world instead of swimming on the screen. Each texture has a mip chain and a row picks
//! its level from the pixel footprint; the detail fades out before it would shimmer.
use super::{vh, vw};
use crate::fx::hash;
use crate::track::{ROAD_W, SEG_LEN};
use std::sync::OnceLock;

/// One tileable texture with a mip chain; each texel is (brightness, warm tint), signed.
struct Tex {
    mips: Vec<(usize, Vec<[i8; 2]>)>,
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn h2(ix: i32, iy: i32, seed: u32) -> f32 {
    let h = hash(
        (ix as u32).wrapping_mul(0x9E37_79B1)
            ^ (iy as u32).wrapping_mul(0x85EB_CA6B)
            ^ seed.wrapping_mul(0xC2B2_AE35),
    );
    (h & 0xFF_FFFF) as f32 / 16_777_215.0
}

/// Value noise in [0,1], periodic in both axes with `per` lattice cells.
fn tn(x: f32, y: f32, seed: u32, per: i32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (smooth(x - ix as f32), smooth(y - iy as f32));
    let w = |i: i32| i.rem_euclid(per);
    let (x0, x1, y0, y1) = (w(ix), w(ix + 1), w(iy), w(iy + 1));
    let a = h2(x0, y0, seed);
    let b = h2(x1, y0, seed);
    let c = h2(x0, y1, seed);
    let d = h2(x1, y1, seed);
    let top = a + (b - a) * fx;
    let bot = c + (d - c) * fx;
    top + (bot - top) * fy
}

/// Tileable fBm in [0,1]; `cells` lattice cells across the tile at the first octave.
fn tfbm(u: f32, v: f32, seed: u32, cells: i32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut norm) = (0.0, 0.5, 0.0);
    for o in 0..octaves {
        let per = cells << o;
        let f = per as f32;
        sum += amp * tn(u * f, v * f, seed.wrapping_add(o * 31), per);
        norm += amp;
        amp *= 0.5;
    }
    sum / norm
}

fn q(v: f32) -> i8 {
    (v.clamp(-1.0, 1.0) * 127.0) as i8
}

fn build(size: usize, f: &dyn Fn(f32, f32) -> (f32, f32)) -> Tex {
    let mut base = vec![[0i8; 2]; size * size];
    for y in 0..size {
        for x in 0..size {
            let (l, t) = f(x as f32 / size as f32, y as f32 / size as f32);
            base[y * size + x] = [q(l), q(t)];
        }
    }
    let mut mips = vec![(size, base)];
    while mips.last().unwrap().0 > size / 16 {
        let (s, src) = mips.last().unwrap();
        let ns = s / 2;
        let mut dst = vec![[0i8; 2]; ns * ns];
        for y in 0..ns {
            for x in 0..ns {
                let mut acc = [0i32; 2];
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let t = src[(y * 2 + dy) * s + x * 2 + dx];
                    acc[0] += t[0] as i32;
                    acc[1] += t[1] as i32;
                }
                dst[y * ns + x] = [(acc[0] / 4) as i8, (acc[1] / 4) as i8];
            }
        }
        mips.push((ns, dst));
    }
    Tex { mips }
}

struct Set {
    /// texel density multiplier (2 for the 1024-pixel set used at high resolutions)
    kmul: f32,
    facade: Tex,
    asphalt: Tex,
    soil: Tex,
    macro_: Tex,
}

/// Value noise with separate periods per axis (for stretched streaks).
fn tn2(x: f32, y: f32, seed: u32, px: i32, py: i32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (smooth(x - ix as f32), smooth(y - iy as f32));
    let (x0, x1, y0, y1) = (
        ix.rem_euclid(px),
        (ix + 1).rem_euclid(px),
        iy.rem_euclid(py),
        (iy + 1).rem_euclid(py),
    );
    let (a, b) = (h2(x0, y0, seed), h2(x1, y0, seed));
    let (c, d) = (h2(x0, y1, seed), h2(x1, y1, seed));
    let top = a + (b - a) * fx;
    let bot = c + (d - c) * fx;
    top + (bot - top) * fy
}

fn make_set(size: usize, kmul: f32, ex: u32) -> Set {
    let sz = size as f32;
    Set {
        kmul,
        // building facade: concrete grain, panel joints, rain streaks, stains and chips
        facade: build(size, &|u, v| {
            let grain = tfbm(u, v, 71, 24, 5 + ex) * 2.0 - 1.0;
            let fine = h2((u * sz) as i32, (v * sz) as i32, 13) * 2.0 - 1.0;
            let streak =
                tn2(u * 96.0, v * 6.0, 72, 96, 6) * 0.6 + tn2(u * 40.0, v * 3.0, 73, 40, 3) * 0.4;
            let streak = ((streak - 0.5) * 3.0).clamp(-1.0, 1.0);
            let stain = tfbm(u, v, 74, 4, 4) * 2.0 - 1.0;
            let fu = (u * 8.0).fract();
            let fv = (v * 8.0).fract();
            let joint =
                (!(0.012..=0.988).contains(&fu) || !(0.012..=0.988).contains(&fv)) as i32 as f32;
            let chip = ((tfbm(u, v, 75, 40, 2) - 0.78) * 14.0).clamp(0.0, 1.0);
            let l = 0.28 * grain + 0.12 * fine + 0.34 * streak + 0.30 * stain
                - 0.55 * joint
                - 0.35 * chip;
            let t = 0.5 * (tfbm(u, v, 76, 3, 3) * 2.0 - 1.0);
            (l, t)
        }),
        // asphalt: aggregate grain, pebbles, a fractal crack network and repair patches
        asphalt: build(size, &|u, v| {
            let grain = tfbm(u, v, 11, 24, 6 + ex) * 2.0 - 1.0;
            let fine = h2((u * sz) as i32, (v * sz) as i32, 5);
            let pebble = if fine > 0.965 {
                1.0
            } else if fine < 0.04 {
                -1.0
            } else {
                0.0
            };
            // cracks: thin valleys of a domain-warped noise, branching through octaves
            let wu = u + 0.04 * (tfbm(u, v, 21, 4, 3) - 0.5);
            let wv = v + 0.04 * (tfbm(u, v, 22, 4, 3) - 0.5);
            let c1 = (tfbm(wu, wv, 31, 5, 3) - 0.5).abs();
            let c2 = (tfbm(wu, wv, 32, 11, 2) - 0.5).abs();
            let crack =
                (1.0 - c1 / 0.012).clamp(0.0, 1.0) + 0.6 * (1.0 - c2 / 0.009).clamp(0.0, 1.0);
            let patch = tfbm(u, v, 41, 3, 3) - 0.5;
            let patch_edge = ((tfbm(u, v, 42, 3, 2) - 0.58) * 24.0).clamp(0.0, 1.0);
            let l = 0.36 * grain + 0.30 * pebble - 0.85 * crack.min(1.0) + 0.45 * patch
                - 0.28 * patch_edge;
            (l, 0.0)
        }),
        // soil and turf: warped multi-octave fractal with fine blades
        soil: build(size, &|u, v| {
            let wu = u + 0.08 * (tfbm(u, v, 51, 6, 4) - 0.5);
            let wv = v + 0.08 * (tfbm(u, v, 52, 6, 4) - 0.5);
            let big = tfbm(wu, wv, 53, 8, 6 + ex) * 2.0 - 1.0;
            let blade = h2((u * sz) as i32, (v * sz) as i32, 9) * 2.0 - 1.0;
            let tuft = ((tfbm(u, v, 54, 64, 2) - 0.55) * 8.0).clamp(-1.0, 1.0);
            let tint = tfbm(u, v, 55, 5, 3) * 2.0 - 1.0;
            (0.55 * big + 0.30 * blade + 0.35 * tuft, 0.4 * tint)
        }),
        // large-scale mottling that survives to the far distance
        macro_: build(size / 2, &|u, v| {
            let a = tfbm(u, v, 61, 3, 6) * 2.0 - 1.0;
            let t = tfbm(u, v, 62, 2, 4) * 2.0 - 1.0;
            (0.8 * a, 0.7 * t)
        }),
    }
}

fn set() -> &'static Set {
    static S: OnceLock<Set> = OnceLock::new();
    S.get_or_init(|| make_set(512, 1.0, 0))
}

/// Finer set (1024-pixel tiles, twice the texel density, one more octave); built on first use
/// by renders at high native resolution.
fn hi() -> &'static Set {
    static S: OnceLock<Set> = OnceLock::new();
    S.get_or_init(|| make_set(1024, 2.0, 1))
}

fn pick() -> &'static Set {
    if super::vk() >= 2.0 {
        hi()
    } else {
        set()
    }
}
/// Texels per road half-width (lateral) for each texture.
const K_ASPHALT: f32 = 192.0;
const K_SOIL: f32 = 96.0;
const K_MACRO: f32 = 9.0;
/// Segment length in road half-widths: converts lateral density to along-track density.
const SEG_U: f32 = SEG_LEN / ROAD_W;

fn fade(m: f32, from: f32) -> f32 {
    ((from + 1.6 - m) / 1.6).clamp(0.0, 1.0)
}

#[derive(Clone, Copy)]
struct Look {
    mip: usize,
    amp: i32,
}

fn look(tex: &Tex, k: f32, w: f32, rowh: f32, strength: i32) -> Option<Look> {
    let tpu = k / w;
    let tpv = k * SEG_U / rowh;
    let m = tpu.max(tpv).max(1.0).log2();
    let top = (tex.mips.len() - 1) as f32;
    let a = fade(m, top - 1.0);
    if a <= 0.0 {
        return None;
    }
    Some(Look {
        mip: (m as usize).min(tex.mips.len() - 1),
        amp: (strength as f32 * a) as i32,
    })
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Ground {
    None,
    Lum,
    LumTint,
}

/// One texture layer prepared for a row: a fixed-point (16.16) walk along a texel row.
struct Lay<'a> {
    row: &'a [[i8; 2]],
    mask: i32,
    su: i32,
    ds: i32,
    amp: i32,
}

impl<'a> Lay<'a> {
    fn new(tex: &'a Tex, l: Look, k: f32, w: f32, xc: f32, x0: i32, v: f32) -> Self {
        let (size, d) = &tex.mips[l.mip];
        let sc = 1.0 / (1 << l.mip) as f32;
        let ty = ((v * k * SEG_U * sc).floor() as i32) & (*size as i32 - 1);
        Lay {
            row: &d[ty as usize * size..(ty as usize + 1) * size],
            mask: *size as i32 - 1,
            su: (((x0 as f32 + 0.5 - xc) / w) * k * sc * 65536.0) as i32,
            ds: (k / w * sc * 65536.0) as i32,
            amp: l.amp,
        }
    }
    #[inline(always)]
    fn next(&mut self) -> [i8; 2] {
        let t = self.row[((self.su >> 16) & self.mask) as usize];
        self.su = self.su.wrapping_add(self.ds);
        t
    }
}

/// Lay the texture over one segment strip (`y2` far edge, `y1` near edge).
#[allow(clippy::too_many_arguments)]
pub(super) fn strip(
    fb: &mut [u32],
    (y2, y1): (f32, f32),
    (x2, w2): (f32, f32),
    (x1, w1): (f32, f32),
    idx: usize,
    maxy: f32,
    road: bool,
    ground: Ground,
) {
    if y1 - y2 <= 0.0 || (!road && ground == Ground::None) {
        return;
    }
    let s = pick();
    let (ka, ks) = (K_ASPHALT * s.kmul, K_SOIL * s.kmul);
    let rowh = (y1 - y2).max(1.0);
    let ya = y2.round() as i32;
    let ye = y1.min(maxy).round() as i32;
    let (vw, vh) = (vw() as i32, vh() as i32);
    for y in ya.max(0)..ye.min(vh) {
        let t = (((y as f32 + 0.5) - y2) / (y1 - y2)).clamp(0.0, 1.0);
        let xc = x2 + (x1 - x2) * t;
        let w = w2 + (w1 - w2) * t;
        if w < 2.5 {
            continue;
        }
        let v = idx as f32 + 1.0 - t;
        let row = &mut fb[y as usize * vw as usize..(y as usize + 1) * vw as usize];
        let xl = ((xc - w).round() as i32).clamp(0, vw);
        let xr = ((xc + w).round() as i32).clamp(0, vw);
        if road {
            if let Some(l) = look(&s.asphalt, ka, w, rowh, 118) {
                let mut ly = Lay::new(&s.asphalt, l, ka, w, xc, xl, v);
                for px in &mut row[xl as usize..xr as usize] {
                    let tx = ly.next();
                    *px = apply(*px, (tx[0] as i32 * ly.amp) >> 7, 0);
                }
            }
        }
        if ground != Ground::None {
            let micro = look(&s.soil, ks, w, rowh, 84);
            let mac = look(&s.macro_, K_MACRO, w, rowh, 60);
            let tint_k = if ground == Ground::LumTint { 24 } else { 0 };
            for (a, b) in [(0, xl), (xr, vw)] {
                if a >= b {
                    continue;
                }
                let mut mi = micro.map(|l| Lay::new(&s.soil, l, ks, w, xc, a, v));
                let mut ma = mac.map(|l| Lay::new(&s.macro_, l, K_MACRO, w, xc, a, v));
                for px in &mut row[a as usize..b as usize] {
                    let (mut lum, mut tint) = (0i32, 0i32);
                    if let Some(ly) = mi.as_mut() {
                        let tx = ly.next();
                        lum += (tx[0] as i32 * ly.amp) >> 7;
                        tint += (tx[1] as i32 * tint_k) >> 7;
                    }
                    if let Some(ly) = ma.as_mut() {
                        let tx = ly.next();
                        lum += (tx[0] as i32 * ly.amp) >> 7;
                        tint += (tx[1] as i32 * tint_k * 2) >> 7;
                    }
                    *px = apply(*px, lum, tint);
                }
            }
        }
    }
}

#[inline]
fn apply(c: u32, lum: i32, tint: i32) -> u32 {
    let f = 256 + lum;
    let add = lum >> 3;
    let r = ((((c >> 16) & 255) as i32 * f) >> 8) + tint + add;
    let g = ((((c >> 8) & 255) as i32 * f) >> 8) + (tint >> 2) + add;
    let b = (((c & 255) as i32 * f) >> 8) - tint + add;
    ((r.clamp(0, 255) as u32) << 16) | ((g.clamp(0, 255) as u32) << 8) | b.clamp(0, 255) as u32
}

const K_FACADE: f32 = 128.0;

/// Lay the facade texture over a building wall: a quad with vertical sides at `xa` and `xb`,
/// bottom edge from y1 (near) to y2 (far), height `hb` road half-widths.
#[allow(clippy::too_many_arguments)]
pub(super) fn wall(
    fb: &mut [u32],
    (xa, xb): (f32, f32),
    (y1, y2): (f32, f32),
    (w1, w2): (f32, f32),
    hb: f32,
    idx: usize,
    clip: f32,
    fog: f32,
) {
    let (lo, hi) = if xa < xb { (xa, xb) } else { (xb, xa) };
    let span = (xb - xa).abs();
    if span < 1.0 || fog > 0.85 {
        return;
    }
    let s = pick();
    let kf = K_FACADE * s.kmul;
    let (vwi, vhi) = (vw() as i32, vh() as i32);
    let tex = &s.facade;
    let tpu = kf * SEG_U / span;
    let x0 = (lo.round() as i32).max(0);
    let x1 = (hi.round() as i32).min(vwi);
    for x in x0..x1 {
        let t = ((x as f32 + 0.5 - xa) / (xb - xa)).clamp(0.0, 1.0);
        let bottom = y1 + (y2 - y1) * t;
        let wpx = w1 + (w2 - w1) * t;
        if wpx < 2.0 {
            continue;
        }
        let top = bottom - hb * wpx;
        let m = tpu.max(kf / wpx).max(1.0).log2();
        let amp_f = fade(m, (tex.mips.len() - 1) as f32 - 1.0) * (1.0 - fog);
        if amp_f <= 0.0 {
            continue;
        }
        let amp = (96.0 * amp_f) as i32;
        let mi = (m as usize).min(tex.mips.len() - 1);
        let (size, d) = &tex.mips[mi];
        let mask = *size as i32 - 1;
        let sc = 1.0 / (1 << mi) as f32;
        let tx = (((idx as f32 + t) * SEG_U * kf * sc).floor() as i32) & mask;
        let ya = (top.round() as i32).max(0);
        let yb = (bottom.round() as i32).min(vhi).min(clip as i32);
        for y in ya..yb {
            let f = (bottom - (y as f32 + 0.5)) / wpx;
            let ty = ((f * kf * sc).floor() as i32) & mask;
            let tv = d[ty as usize * size + tx as usize];
            let i = y as usize * vwi as usize + x as usize;
            fb[i] = apply(fb[i], (tv[0] as i32 * amp) >> 7, (tv[1] as i32 * amp) >> 9);
        }
    }
}
