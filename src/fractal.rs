//! Generative fractal content: periodic value-noise fBm, ridged multifractals, recursive
//! branching trees, Mandelbrot ribbons, domain-warped nebulae. Everything is produced from
//! the stage seed at start-up; no external assets.
use crate::fx::hash;
use crate::rng::Rng;
use std::sync::OnceLock;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    smooth(((x - a) / (b - a)).clamp(0.0, 1.0))
}

fn hash2(ix: i32, iy: i32, seed: u32) -> f32 {
    let h = hash(
        (ix as u32).wrapping_mul(0x9E37_79B1)
            ^ (iy as u32).wrapping_mul(0x85EB_CA6B)
            ^ seed.wrapping_mul(0xC2B2_AE35),
    );
    (h & 0xFF_FFFF) as f32 / 16_777_215.0
}

/// Value noise in \[0,1\]; periodic in x with period `px` lattice cells (0 = aperiodic).
pub fn noise(x: f32, y: f32, seed: u32, px: i32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (smooth(x - ix as f32), smooth(y - iy as f32));
    let wx = |i: i32| if px > 0 { i.rem_euclid(px) } else { i };
    let a = hash2(wx(ix), iy, seed);
    let b = hash2(wx(ix + 1), iy, seed);
    let c = hash2(wx(ix), iy + 1, seed);
    let d = hash2(wx(ix + 1), iy + 1, seed);
    lerp(lerp(a, b, fx), lerp(c, d, fx), fy)
}

/// Fractional Brownian motion, roughly in \[0,1\].
pub fn fbm(x: f32, y: f32, seed: u32, octaves: u32, px: i32) -> f32 {
    let (mut sum, mut amp, mut norm) = (0.0, 0.5, 0.0);
    for o in 0..octaves {
        let f = (1u32 << o) as f32;
        sum += amp * noise(x * f, y * f, seed.wrapping_add(o * 31), px * (1 << o));
        norm += amp;
        amp *= 0.5;
    }
    sum / norm
}

/// Ridged multifractal in \[0,1\] (sharp crests).
pub fn ridged(x: f32, y: f32, seed: u32, octaves: u32, px: i32) -> f32 {
    let (mut sum, mut amp, mut norm, mut weight) = (0.0, 0.5, 0.0, 1.0f32);
    for o in 0..octaves {
        let f = (1u32 << o) as f32;
        let n =
            1.0 - (2.0 * noise(x * f, y * f, seed.wrapping_add(o * 57), px * (1 << o)) - 1.0).abs();
        let n = n * n * weight;
        weight = (n * 2.0).clamp(0.0, 1.0);
        sum += amp * n;
        norm += amp;
        amp *= 0.5;
    }
    sum / norm
}

pub fn ridge_table(seed: u32) -> Vec<f32> {
    (0..1024)
        .map(|x| (ridged(x as f32 / 128.0, 0.5, seed, 5, 8) * 1.25 - 0.1).clamp(0.0, 1.0))
        .collect()
}

// ---------------------------------------------------------------- textures

#[derive(Clone)]
pub struct Tex {
    pub w: usize,
    pub h: usize,
    /// ARGB, straight (non-premultiplied) alpha
    pub px: Vec<u32>,
}

pub struct Img {
    pub mips: Vec<Tex>,
}

fn over(dst: u32, src: u32) -> u32 {
    let sa = (src >> 24) as f32 / 255.0;
    if sa <= 0.0 {
        return dst;
    }
    let da = (dst >> 24) as f32 / 255.0;
    let oa = sa + da * (1.0 - sa);
    let ch = |sh: u32| -> u32 {
        let s = ((src >> sh) & 255) as f32;
        let d = ((dst >> sh) & 255) as f32;
        (((s * sa + d * da * (1.0 - sa)) / oa) + 0.5).min(255.0) as u32
    };
    (((oa * 255.0 + 0.5) as u32) << 24) | (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

pub fn mix_rgb(a: u32, b: u32, t: f32) -> u32 {
    let ch = |sh: u32| -> u32 {
        let (x, y) = (((a >> sh) & 255) as f32, ((b >> sh) & 255) as f32);
        (x + (y - x) * t) as u32
    };
    (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

impl Tex {
    pub fn new(w: usize, h: usize) -> Self {
        Tex {
            w,
            h,
            px: vec![0; w * h],
        }
    }

    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, rgb: u32, a: f32) {
        let x0 = ((cx - r - 1.0).floor() as i32).max(0);
        let x1 = ((cx + r + 1.0).ceil() as i32).min(self.w as i32);
        let y0 = ((cy - r - 1.0).floor() as i32).max(0);
        let y1 = ((cy + r + 1.0).ceil() as i32).min(self.h as i32);
        for y in y0..y1 {
            for x in x0..x1 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let cov = (r - d + 0.5).clamp(0.0, 1.0) * a;
                if cov > 0.0 {
                    let src = (((cov * 255.0) as u32) << 24) | (rgb & 0xFF_FFFF);
                    let i = y as usize * self.w + x as usize;
                    self.px[i] = over(self.px[i], src);
                }
            }
        }
    }

    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, w0: f32, w1: f32, rgb: u32) {
        let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        let steps = (len * 1.2).ceil().max(1.0) as i32;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            self.disc(
                lerp(x0, x1, t),
                lerp(y0, y1, t),
                lerp(w0, w1, t) * 0.5,
                rgb,
                1.0,
            );
        }
    }

    pub fn into_img(self) -> Img {
        let mut mips = vec![self];
        while mips.last().unwrap().w > 16 && mips.last().unwrap().h > 16 {
            let n = {
                let p = mips.last().unwrap();
                let (nw, nh) = (p.w / 2, p.h / 2);
                let mut n = Tex::new(nw, nh);
                for y in 0..nh {
                    for x in 0..nw {
                        let (mut a, mut r, mut g, mut b) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
                        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                            let c = p.px[(2 * y + dy) * p.w + 2 * x + dx];
                            let al = (c >> 24) as f32;
                            a += al;
                            r += ((c >> 16) & 255) as f32 * al;
                            g += ((c >> 8) & 255) as f32 * al;
                            b += (c & 255) as f32 * al;
                        }
                        if a > 0.0 {
                            r /= a;
                            g /= a;
                            b /= a;
                        }
                        n.px[y * nw + x] = (((a / 4.0) as u32) << 24)
                            | ((r as u32) << 16)
                            | ((g as u32) << 8)
                            | b as u32;
                    }
                }
                n
            };
            mips.push(n);
        }
        Img { mips }
    }
}

// ---------------------------------------------------------------- trees

struct Pal {
    bark: u32,
    leaf1: u32,
    leaf2: u32,
}

fn leaves(t: &mut Tex, rng: &mut Rng, x: f32, y: f32, pal: &Pal, count: u32, spread: f32) {
    for _ in 0..count {
        let dx = rng.range(-spread, spread);
        let dy = rng.range(-spread, spread);
        let r = rng.range(2.2, 4.8);
        let c = mix_rgb(pal.leaf1, pal.leaf2, rng.f());
        t.disc(x + dx, y + dy, r, c, 1.0);
    }
}

/// Recursive binary/ternary branching with decaying length and width.
#[allow(clippy::too_many_arguments)]
fn branch(
    t: &mut Tex,
    rng: &mut Rng,
    x: f32,
    y: f32,
    ang: f32,
    len: f32,
    wd: f32,
    depth: u32,
    pal: &Pal,
    spread: f32,
    leafy: bool,
) {
    let (x1, y1) = (x + ang.sin() * len, y - ang.cos() * len);
    t.line(x, y, x1, y1, wd, wd * 0.72, pal.bark);
    if depth == 0 || len < 3.0 {
        if leafy {
            leaves(t, rng, x1, y1, pal, 12, 9.0);
        } else {
            t.disc(x1, y1, 1.0, pal.bark, 1.0);
        }
        return;
    }
    if leafy && depth <= 3 {
        leaves(t, rng, (x + x1) * 0.5, (y + y1) * 0.5, pal, 3, 5.0);
    }
    let n = if depth > 4 { 2 } else { 2 + rng.below(2) };
    for i in 0..n {
        let k = if n > 1 {
            i as f32 / (n - 1) as f32 - 0.5
        } else {
            0.0
        };
        let a = ang + k * spread * 2.0 + rng.range(-0.25, 0.25);
        let nl = len * rng.range(0.68, 0.8);
        branch(
            t,
            rng,
            x1,
            y1,
            a,
            nl,
            wd * 0.68,
            depth - 1,
            pal,
            spread,
            leafy,
        );
    }
}

pub fn broadleaf(seed: u64, bark: u32, l1: u32, l2: u32) -> Img {
    let mut t = Tex::new(192, 256);
    let mut rng = Rng::new(seed);
    let pal = Pal {
        bark,
        leaf1: l1,
        leaf2: l2,
    };
    let (a0, l0) = (rng.range(-0.08, 0.08), rng.range(48.0, 58.0));
    branch(
        &mut t, &mut rng, 96.0, 252.0, a0, l0, 8.0, 7, &pal, 0.62, true,
    );
    t.into_img()
}

pub fn dead_tree(seed: u64, bark: u32, glow: u32) -> Img {
    let mut t = Tex::new(192, 256);
    let mut rng = Rng::new(seed);
    let pal = Pal {
        bark,
        leaf1: glow,
        leaf2: glow,
    };
    let (a0, l0) = (rng.range(-0.1, 0.1), rng.range(46.0, 56.0));
    branch(
        &mut t,
        &mut rng,
        96.0,
        252.0,
        a0,
        l0,
        8.0 - 1.0,
        8,
        &pal,
        0.85,
        false,
    );
    for _ in 0..10 {
        let (x, y) = (rng.range(40.0, 150.0), rng.range(40.0, 200.0));
        t.disc(x, y, 1.6, glow, 0.9);
    }
    t.into_img()
}

#[allow(clippy::too_many_arguments)]
fn frond(
    t: &mut Tex,
    rng: &mut Rng,
    x0: f32,
    y0: f32,
    side: f32,
    len: f32,
    droop: f32,
    bark: u32,
    n1: u32,
    n2: u32,
) {
    let (x1, y1) = (x0 + side * len, y0 + len * droop);
    t.line(x0, y0, x1, y1, 2.4, 0.9, mix_rgb(bark, n1, 0.45));
    let nt = ((len / 5.0) as i32).max(3);
    for j in 1..=nt {
        let u = j as f32 / nt as f32;
        let (bx, by) = (lerp(x0, x1, u), lerp(y0, y1, u));
        let tl = len * 0.32 * (1.0 - u * 0.65);
        // downward twig, plus a smaller upward one: the self-similar part
        let (dx, dy) = (side * tl * 0.28, tl * 0.95);
        let c = mix_rgb(n1, n2, rng.f());
        t.line(bx, by, bx + dx, by + dy, 1.6, 0.6, c);
        for k in 0..4 {
            let f = (k as f32 + 1.0) / 4.0;
            t.disc(
                bx + dx * f,
                by + dy * f,
                rng.range(1.6, 3.0),
                mix_rgb(n1, n2, rng.f()),
                1.0,
            );
        }
        t.line(bx, by, bx + side * tl * 0.2, by - tl * 0.3, 1.2, 0.5, c);
        t.disc(bx + side * tl * 0.2, by - tl * 0.3, 1.8, c, 1.0);
    }
    t.disc(x1, y1, 2.6, n2, 1.0);
}

pub fn conifer(seed: u64, bark: u32, n1: u32, n2: u32) -> Img {
    let mut t = Tex::new(192, 256);
    let mut rng = Rng::new(seed);
    let top_x = 96.0 + rng.range(-3.0, 3.0);
    t.line(96.0, 252.0, top_x, 16.0, 7.0, 1.5, bark);
    let whorls = 26;
    for i in 0..whorls {
        let f = i as f32 / whorls as f32;
        let y = 234.0 - f * 205.0;
        let l = (1.0 - f).powf(0.85) * 76.0 + 8.0;
        for side in [-1.0f32, 1.0] {
            for _ in 0..2 {
                let y0 = y + rng.range(-3.0, 3.0);
                let (fl, dr) = (l * rng.range(0.8, 1.05), rng.range(0.25, 0.45));
                frond(&mut t, &mut rng, 96.0, y0, side, fl, dr, bark, n1, n2);
            }
        }
    }
    t.disc(top_x, 16.0, 3.0, n2, 1.0);
    t.into_img()
}

pub fn rock(seed: u64, base: u32) -> Img {
    let (w, h) = (128usize, 96usize);
    let mut t = Tex::new(w, h);
    let s = seed as u32;
    let (cx, cy) = (w as f32 / 2.0, h as f32 * 0.86);
    let light = (-0.5f32, 0.6f32, 0.62f32);
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let ang = dy.atan2(dx);
            let rb_x =
                56.0 * (0.62 + 0.55 * fbm(ang.cos() * 1.6 + 4.0, ang.sin() * 1.6 + 4.0, s, 4, 0));
            let rb_y =
                70.0 * (0.62 + 0.55 * fbm(ang.cos() * 1.6 + 4.0, ang.sin() * 1.6 + 4.0, s, 4, 0));
            let d = ((dx / rb_x).powi(2) + (dy / rb_y).powi(2)).sqrt();
            if d >= 1.0 || dy > 0.0 {
                if dy > 0.0 && dy < 6.0 && d < 1.0 {
                    t.px[y * w + x] = 0xFF00_0000 | mix_rgb(base, 0x000000, 0.6);
                }
                continue;
            }
            let nz = (1.0 - d * d).sqrt();
            let (nx, ny) = (dx / rb_x, dy / rb_y);
            let lam = (nx * light.0 + -ny * light.1 + nz * light.2).clamp(0.0, 1.0);
            let tex = fbm(x as f32 * 0.09, y as f32 * 0.09, s ^ 99, 5, 0);
            let k = (0.35 + 0.85 * lam) * (0.7 + 0.6 * tex);
            let c = mix_rgb(0x000000, base, (k).min(1.3).min(1.0));
            let c = if k > 1.0 {
                mix_rgb(c, 0xFFFFFF, (k - 1.0).min(0.5))
            } else {
                c
            };
            let edge = ((1.0 - d) * 18.0).clamp(0.0, 1.0);
            t.px[y * w + x] = (((edge * 255.0) as u32) << 24) | c;
        }
    }
    t.into_img()
}

// ---------------------------------------------------------------- Mandelbrot ribbon

/// Smooth-coloured Mandelbrot window; used for checkpoint banners.
pub fn ribbon(w: usize, h: usize, cx: f32, cy: f32, width: f32, iters: u32, phase: f32) -> Img {
    let mut t = Tex::new(w, h);
    let hh = width * h as f32 / w as f32;
    for y in 0..h {
        for x in 0..w {
            let cr = cx as f64 + ((x as f64 + 0.5) / w as f64 - 0.5) * width as f64;
            let ci = cy as f64 + ((y as f64 + 0.5) / h as f64 - 0.5) * hh as f64;
            let (mut zr, mut zi) = (0.0f64, 0.0f64);
            let mut it = 0;
            while it < iters && zr * zr + zi * zi < 65536.0 {
                let nr = zr * zr - zi * zi + cr;
                zi = 2.0 * zr * zi + ci;
                zr = nr;
                it += 1;
            }
            let col = if it >= iters {
                0x08_0418
            } else {
                let m = (zr * zr + zi * zi).sqrt().ln().max(1e-6).log2();
                let mu = it as f64 + 1.0 - m;
                let tt = (mu * 0.045) as f32 + phase;
                let c = |o: f32| {
                    ((0.5 + 0.5 * (std::f32::consts::TAU * (tt + o)).cos()).powf(0.8) * 255.0)
                        as u32
                };
                (c(0.0) << 16) | (c(0.33) << 8) | c(0.67)
            };
            t.px[y * w + x] = 0xFF00_0000 | col;
        }
    }
    t.into_img()
}

// ---------------------------------------------------------------- sky textures

/// Horizontally periodic domain-warped fBm nebula, additive-blended in the night sky.
/// Julia set rendered with smooth colouring; the interior stays transparent so it works as a sky layer.
pub fn julia(w: usize, h: usize, seed: u32) -> Tex {
    let mut t = Tex::new(w, h);
    let a = (seed % 628) as f32 / 100.0;
    let (cr, ci) = (0.7885 * a.cos(), 0.7885 * a.sin());
    let aspect = w as f32 / h as f32;
    for y in 0..h {
        for x in 0..w {
            let mut zr = (x as f32 / w as f32 - 0.5) * 3.2 * aspect.min(2.0) / 2.0 * 2.0;
            let mut zi = (y as f32 / h as f32 - 0.5) * 3.2;
            let mut i = 0u32;
            while i < 140 && zr * zr + zi * zi < 256.0 {
                let t = zr * zr - zi * zi + cr;
                zi = 2.0 * zr * zi + ci;
                zr = t;
                i += 1;
            }
            if i >= 140 {
                continue;
            }
            let r2 = (zr * zr + zi * zi).max(1.0001);
            let sm = i as f32 + 1.0 - r2.ln().ln() / std::f32::consts::LN_2;
            let u = (sm * 0.045).fract();
            let c = mix_rgb(
                mix_rgb(
                    0x2A0A70,
                    0x18D0FF,
                    (u * std::f32::consts::TAU).sin() * 0.5 + 0.5,
                ),
                0xFF70D0,
                (u * 12.0).cos() * 0.5 + 0.5,
            );
            let fade = smoothstep(0.0, 30.0, sm).min(1.0);
            t.px[y * w + x] = (((fade * 215.0) as u32) << 24) | c;
        }
    }
    t
}

pub fn nebula(w: usize, h: usize, seed: u32) -> Tex {
    let mut t = Tex::new(w, h);
    let per = (w / 128) as i32;
    for y in 0..h {
        let v = y as f32 / 80.0;
        let m = smoothstep(0.0, 0.35, y as f32 / h as f32)
            * (1.0 - smoothstep(0.7, 1.0, y as f32 / h as f32));
        for x in 0..w {
            let u = x as f32 / 128.0;
            let qx = fbm(u, v, seed, 4, per);
            let qy = fbm(u + 5.2, v + 1.3, seed ^ 0x55, 4, per);
            let val = fbm(
                u + 2.6 * (qx - 0.5),
                v + 2.6 * (qy - 0.5),
                seed ^ 0xAA,
                5,
                per,
            );
            let b = smoothstep(0.40, 0.78, val) * m;
            let col = mix_rgb(0x5A1EA0, 0x28C8FF, qy);
            let hot = mix_rgb(col, 0xFFB0E0, smoothstep(0.7, 0.95, val));
            t.px[y * w + x] = (((b * 255.0) as u32) << 24) | hot;
        }
    }
    t
}

/// Horizontally periodic soft clouds with baked top-left lighting.
pub fn clouds(w: usize, h: usize, seed: u32) -> Tex {
    let mut t = Tex::new(w, h);
    let per = (w / 128) as i32;
    for y in 0..h {
        let bias = 0.14 - ((y as f32 / h as f32) - 0.5).abs() * 0.5;
        for x in 0..w {
            let (u, v) = (x as f32 / 128.0, y as f32 / 40.0);
            let d = fbm(u, v, seed, 6, per) - 0.5 + bias;
            let d2 = fbm(u + 0.06, v + 0.12, seed, 6, per) - 0.5 + bias;
            let dens = smoothstep(0.02, 0.26, d);
            let s = (0.5 + (d - d2) * 9.0).clamp(0.0, 1.0);
            let c = mix_rgb(0x96A0B9, 0xFFFFFF, s);
            t.px[y * w + x] = (((dens * 255.0) as u32) << 24) | c;
        }
    }
    t
}

// ---------------------------------------------------------------- asset bundle

pub struct Assets {
    /// Julia-set backdrop for the cosmos landscape
    pub julia: Tex,
    pub broadleaf: Vec<Img>,
    pub conifer: [Vec<Img>; 3],
    pub dead: [Vec<Img>; 3],
    pub rocks: [Vec<Img>; 3],
    pub gate: Vec<Img>,
    pub clouds: Tex,
    pub nebula: Tex,
    pub ridges: [Vec<f32>; 2],
}

fn variants(s: u64, n: u64, f: &dyn Fn(u64) -> Img, count: u64) -> Vec<Img> {
    (0..count)
        .map(|i| f(s.wrapping_add(n * 1000 + i * 77)))
        .collect()
}

type Trio = [Vec<Img>; 3];

fn gen_trees(s: u64) -> (Vec<Img>, Trio) {
    (
        variants(s, 1, &|k| broadleaf(k, 0x5A3A22, 0x288C32, 0x5FAF46), 4),
        [
            variants(s, 2, &|k| conifer(k, 0x4A2F1C, 0x166428, 0x28823A), 3),
            variants(s, 3, &|k| conifer(k, 0x5A4A2A, 0x5A7832, 0x829646), 2),
            variants(s, 4, &|k| conifer(k, 0x2A2438, 0x1E6E6E, 0x3CA0AA), 3),
        ],
    )
}

fn gen_dead_rocks(s: u64) -> (Trio, Trio) {
    (
        [
            variants(s, 5, &|k| dead_tree(k, 0x5A4636, 0x78B450), 2),
            variants(s, 6, &|k| dead_tree(k, 0x7A5A40, 0xC8A050), 3),
            variants(s, 7, &|k| dead_tree(k, 0x463C5A, 0x50E6FF), 3),
        ],
        [
            variants(s, 8, &|k| rock(k, 0x6E6C64), 3),
            variants(s, 9, &|k| rock(k, 0xA88260), 3),
            variants(s, 10, &|k| rock(k, 0x46445A), 3),
        ],
    )
}

fn gen_sky(s: u64) -> (Tex, Tex, [Vec<f32>; 2], Tex) {
    (
        clouds(1024, 120, s as u32 ^ 0x1234),
        nebula(1024, 200, s as u32 ^ 0x4321),
        [ridge_table(s as u32 ^ 0x77), ridge_table(s as u32 ^ 0x99)],
        julia(512, 200, s as u32 ^ 0x3C3C),
    )
}

fn gen_gates() -> Vec<Img> {
    vec![
        ribbon(512, 64, -0.7453, 0.1127, 0.016, 260, 0.05),
        ribbon(512, 64, -0.1592, 1.0317, 0.012, 260, 0.35),
        ribbon(512, 64, -1.7686, 0.0, 0.02, 260, 0.6),
    ]
}

impl Assets {
    /// Generates everything; uses one thread per asset family where threads exist.
    pub fn generate(seed: u64) -> Assets {
        let s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xFEED;
        #[cfg(not(target_arch = "wasm32"))]
        let ((broadleaf, conifer), (dead, rocks), (clouds, nebula, ridges, julia), gate) =
            std::thread::scope(|sc| {
                let t = sc.spawn(|| gen_trees(s));
                let d = sc.spawn(|| gen_dead_rocks(s));
                let k = sc.spawn(|| gen_sky(s));
                let g = sc.spawn(gen_gates);
                (
                    t.join().unwrap(),
                    d.join().unwrap(),
                    k.join().unwrap(),
                    g.join().unwrap(),
                )
            });
        #[cfg(target_arch = "wasm32")]
        let ((broadleaf, conifer), (dead, rocks), (clouds, nebula, ridges, julia), gate) =
            (gen_trees(s), gen_dead_rocks(s), gen_sky(s), gen_gates());
        Assets {
            julia,
            broadleaf,
            conifer,
            dead,
            rocks,
            gate,
            clouds,
            nebula,
            ridges,
        }
    }

    /// Process-wide cache: generated once (from the first seed), reused by every retry.
    pub fn global(seed: u64) -> std::sync::Arc<Assets> {
        static G: OnceLock<std::sync::Arc<Assets>> = OnceLock::new();
        G.get_or_init(|| std::sync::Arc::new(Assets::generate(seed)))
            .clone()
    }
}
