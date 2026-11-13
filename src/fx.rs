#![allow(non_snake_case)]
//! Software post-processing: haze, radial speed blur, bloom, chromatic aberration,
//! vignette, filmic tonemapping, per-biome grading and film grain.
//! Written for speed: integer blur, LUT tonemap, precomputed per-column tables.
use std::sync::OnceLock;

pub struct FxParams {
    pub w: usize,
    pub h: usize,
    pub speed: f32,
    pub turbo: bool,
    pub shake: f32,
    pub biome_w: [f32; crate::NB],
    pub t: f32,
    pub haze: u32,
}

#[inline]
pub fn hash(mut x: u32) -> u32 {
    x = (x ^ 61) ^ (x >> 16);
    x = x.wrapping_mul(9);
    x ^= x >> 4;
    x = x.wrapping_mul(0x27d4_eb2d);
    x ^ (x >> 15)
}

fn aces(x: f32) -> f32 {
    ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0)
}

fn noise() -> &'static Vec<i32> {
    static N: OnceLock<Vec<i32>> = OnceLock::new();
    N.get_or_init(|| {
        (0..65536u32)
            .map(|i| noise_i((hash(i ^ 0xA53C) & 255) as f32 / 255.0 - 0.5))
            .collect()
    })
}

fn box_blur(
    buf: &mut Vec<[f32; 3]>,
    tmp: &mut Vec<[f32; 3]>,
    horizontal: bool,
    r: i32,
    bw: usize,
    bh: usize,
) {
    let (w, h) = (bw as i32, bh as i32);
    let n = (2 * r + 1) as f32;
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 3];
            for d in -r..=r {
                let (sx, sy) = if horizontal {
                    ((x + d).clamp(0, w - 1), y)
                } else {
                    (x, (y + d).clamp(0, h - 1))
                };
                let s = buf[(sy * w + sx) as usize];
                acc[0] += s[0];
                acc[1] += s[1];
                acc[2] += s[2];
            }
            tmp[(y * w + x) as usize] = [acc[0] / n, acc[1] / n, acc[2] / n];
        }
    }
    std::mem::swap(buf, tmp);
}

pub fn post(fb: &mut [u32], p: &FxParams) {
    let (W, H) = (p.w, p.h);
    let K = (W as f32 / 320.0).min(H as f32 / 200.0);
    let (BW, BH) = (W / 4, H / 4);
    let horizon = H / 2;
    let sp = p.speed.min(1.3);

    // 1. atmospheric haze near the horizon
    let hz = [(p.haze >> 16) & 255, (p.haze >> 8) & 255, p.haze & 255];
    let span = H as f32 * 0.09;
    for y in horizon..H {
        let f = (1.0 - (y - horizon) as f32 / span).clamp(0.0, 1.0);
        let f = f.powf(1.5) * 0.36;
        if f < 0.01 {
            break;
        }
        let fi = (f * 256.0) as i32;
        for px in &mut fb[y * W..(y + 1) * W] {
            let c = *px;
            let (r, g, b) = (
                ((c >> 16) & 255) as i32,
                ((c >> 8) & 255) as i32,
                (c & 255) as i32,
            );
            let r = r + (((hz[0] as i32 - r) * fi) >> 8);
            let g = g + (((hz[1] as i32 - g) * fi) >> 8);
            let b = b + (((hz[2] as i32 - b) * fi) >> 8);
            *px = ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
        }
    }

    // column / row tables
    let cx = W as f32 / 2.0;
    let cy = H as f32 * 0.55;
    let colx2: Vec<f32> = (0..W).map(|x| ((x as f32 - cx) / cx).powi(2)).collect();
    let rowy2: Vec<f32> = (0..H).map(|y| ((y as f32 - cy) / cy).powi(2)).collect();

    // 2. radial speed blur (integer, 4 taps toward the vanishing point)
    let kb = (sp - 0.6).max(0.0) * 0.012 + if p.turbo { 0.016 } else { 0.0 };
    if kb > 0.002 {
        let src = fb.to_vec();
        let mut sxs = vec![vec![0usize; W]; 4];
        let mut sys = vec![vec![0usize; H]; 4];
        for t in 0..4 {
            let s = kb * (t + 1) as f32;
            for x in 0..W {
                sxs[t][x] =
                    ((cx + (x as f32 - cx) * (1.0 - s)) as i32).clamp(0, W as i32 - 1) as usize;
            }
            for y in 0..H {
                sys[t][y] = (((cy + (y as f32 - cy) * (1.0 - s)) as i32).clamp(0, H as i32 - 1)
                    as usize)
                    * W;
            }
        }
        for y in 0..H {
            for x in 0..W {
                let r2 = colx2[x] + rowy2[y];
                if r2 < 0.1 {
                    continue;
                }
                let m = (((r2 - 0.1) * 0.7).clamp(0.0, 0.6) * 256.0) as i32;
                let o = src[y * W + x];
                let (or, og, ob) = (
                    ((o >> 16) & 255) as i32,
                    ((o >> 8) & 255) as i32,
                    (o & 255) as i32,
                );
                let (mut sr, mut sg, mut sb) = (or, og, ob);
                for t in 0..4 {
                    let c = src[sys[t][y] + sxs[t][x]];
                    sr += ((c >> 16) & 255) as i32;
                    sg += ((c >> 8) & 255) as i32;
                    sb += (c & 255) as i32;
                }
                let (ar, ag, ab) = ((sr * 51) >> 8, (sg * 51) >> 8, (sb * 51) >> 8);
                let r = or + (((ar - or) * m) >> 8);
                let g = og + (((ag - og) * m) >> 8);
                let b = ob + (((ab - ob) * m) >> 8);
                fb[y * W + x] = ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
            }
        }
    }

    // 3. bloom: bright pass at 1/4 resolution (4 taps per cell), blurred twice
    let mut bloom = vec![[0.0f32; 3]; BW * BH];
    for by in 0..BH {
        for bx in 0..BW {
            let mut acc = [0.0f32; 3];
            for (ox, oy) in [(1usize, 1usize), (3, 1), (1, 3), (3, 3)] {
                let c = fb[(by * 4 + oy) * W + bx * 4 + ox];
                let (r, g, b) = (
                    ((c >> 16) & 255) as f32,
                    ((c >> 8) & 255) as f32,
                    (c & 255) as f32,
                );
                let l = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
                let w = ((l - 0.78) / 0.22).max(0.0) / 255.0;
                acc[0] += r * w;
                acc[1] += g * w;
                acc[2] += b * w;
            }
            bloom[by * BW + bx] = [acc[0] * 0.25, acc[1] * 0.25, acc[2] * 0.25];
        }
    }
    let mut tmp = vec![[0.0f32; 3]; BW * BH];
    for _ in 0..2 {
        box_blur(&mut bloom, &mut tmp, true, 3, BW, BH);
        box_blur(&mut bloom, &mut tmp, false, 3, BW, BH);
    }
    // 4. composite (all-integer inner loop: tonemap LUTs per channel, fixed-point bloom and vignette)
    let src = fb.to_vec();
    let noise = noise();
    let tints = [
        [1.0f32, 1.02, 1.05],
        [1.07, 1.0, 0.90],
        [0.93, 0.97, 1.12],
        [0.97, 1.04, 1.05],
        [1.10, 0.94, 1.02],
        [1.06, 1.04, 0.96],
        [0.88, 1.0, 1.15],
        [0.95, 1.03, 1.10],
        [1.15, 0.90, 0.85],
        [0.85, 1.02, 1.10],
        [1.0, 0.92, 1.15],
        [1.10, 0.92, 1.14],
        [0.92, 1.08, 1.05],
        [0.96, 0.92, 1.14],
        [1.04, 1.04, 1.04],
        [1.12, 1.02, 0.86],
        [1.0, 1.04, 1.1],
        [1.08, 0.9, 0.95],
        [1.14, 0.96, 0.84],
    ];
    let wsum = p.biome_w.iter().sum::<f32>().max(1e-3);
    let mut tint = [0.0f32; 3];
    for (i, t) in tints.iter().enumerate() {
        for q in 0..3 {
            tint[q] += t[q] * p.biome_w[i] / wsum;
        }
    }
    // tone-map table per channel: input 0..1023 (8-bit value plus bloom), output 0..1023
    let mut tmap = vec![[0u16; 1024]; 3];
    for q in 0..3 {
        let e = 1.12 * tint[q];
        for (v, o) in tmap[q].iter_mut().enumerate() {
            *o = (aces(v as f32 / 255.0 * e) * 1023.0) as u16;
        }
    }
    // vignette table indexed by r2 in 1/256 steps (r2 < 2.1)
    let ti = (p.t * 60.0) as u32;
    let h0 = hash(ti);
    let (shx, shy) = if p.shake > 0.0 && !crate::access::calm() {
        (
            ((h0 & 7) as i32 - 3) * K as i32 / 2,
            (((h0 >> 3) & 7) as i32 - 3) * K as i32 / 2,
        )
    } else {
        (0, 0)
    };
    let (nox, noy) = ((h0 >> 8) as usize & 255, (h0 >> 16) as usize & 255);
    let ca_amt = 1.1 * K * sp;
    let wi = W as i32;
    let vig_k = 0.28 * (1.0 + 0.4 * sp);
    let vig: Vec<i32> = (0..540)
        .map(|i| {
            let r2 = i as f32 / 256.0;
            ((1.0 - vig_k * r2 * (1.0 + 0.2 * r2)).max(0.0) * 1024.0) as i32
        })
        .collect();
    let bstrength = 0.9
        + 0.6 * (p.biome_w[2] + p.biome_w[6])
        + 0.3
            * (p.biome_w[4]
                + p.biome_w[10]
                + p.biome_w[11]
                + p.biome_w[12]
                + p.biome_w[13]
                + p.biome_w[14]
                + p.biome_w[17])
        + if p.turbo { 0.35 } else { 0.0 };
    // bloom in 8-bit units times 16, with the strength folded in
    let bq: Vec<[i32; 3]> = bloom
        .iter()
        .map(|b| {
            [
                (b[0] * bstrength * 16.0) as i32,
                (b[1] * bstrength * 16.0) as i32,
                (b[2] * bstrength * 16.0) as i32,
            ]
        })
        .collect();
    let bx0s: Vec<usize> = (0..W)
        .map(|x| (((x as f32 + 0.5) / 4.0 - 0.5).clamp(0.0, BW as f32 - 1.001)) as usize)
        .collect();
    let fxs: Vec<i32> = (0..W)
        .map(|x| {
            ((((x as f32 + 0.5) / 4.0 - 0.5).clamp(0.0, BW as f32 - 1.001)).fract() * 256.0) as i32
        })
        .collect();
    // r2 = nx^2 + ny^2 in 1/256 steps, split into a column and a row part
    let cxr: Vec<i32> = (0..W)
        .map(|x| {
            let nx = (x as f32 / W as f32 - 0.5) * 2.0;
            (nx * nx * 256.0) as i32
        })
        .collect();
    let off_x: Vec<i32> = (0..W)
        .map(|x| {
            let nx = (x as f32 / W as f32 - 0.5) * 2.0;
            (nx * ca_amt) as i32
        })
        .collect();
    let mut brow = vec![[0i32; 3]; BW];
    for y in 0..H {
        let ny = (y as f32 / H as f32 - 0.5) * 2.0;
        let ry = (ny * ny * 256.0) as i32;
        let sy = (y as i32 + shy).clamp(0, H as i32 - 1) as usize;
        let byf = ((y as f32 + 0.5) / 4.0 - 0.5).clamp(0.0, BH as f32 - 1.001);
        let (by0, fy) = (byf as usize, (byf.fract() * 256.0) as i32);
        for i in 0..BW {
            let a = bq[by0 * BW + i];
            let b = bq[(by0 + 1) * BW + i];
            brow[i] = [
                a[0] + (((b[0] - a[0]) * fy) >> 8),
                a[1] + (((b[1] - a[1]) * fy) >> 8),
                a[2] + (((b[2] - a[2]) * fy) >> 8),
            ];
        }
        let row = &src[sy * W..(sy + 1) * W];
        let nrow = &noise[((y + noy) & 255) * 256..((y + noy) & 255) * 256 + 256];
        let out = &mut fb[y * W..(y + 1) * W];
        for x in 0..W {
            let sx = (x as i32 + shx).clamp(0, wi - 1);
            let off = off_x[x];
            let (cr, cg, cb) = if off == 0 {
                let c = row[sx as usize];
                (c, c, c)
            } else {
                (
                    row[(sx + off).clamp(0, wi - 1) as usize],
                    row[sx as usize],
                    row[(sx - off).clamp(0, wi - 1) as usize],
                )
            };
            let (b0, fx) = (bx0s[x], fxs[x]);
            let (s0, s1) = (brow[b0], brow[b0 + 1]);
            let ir = (((cr >> 16) & 255) as i32 + ((s0[0] + (((s1[0] - s0[0]) * fx) >> 8)) >> 4))
                .clamp(0, 1023);
            let ig = (((cg >> 8) & 255) as i32 + ((s0[1] + (((s1[1] - s0[1]) * fx) >> 8)) >> 4))
                .clamp(0, 1023);
            let ib =
                ((cb & 255) as i32 + ((s0[2] + (((s1[2] - s0[2]) * fx) >> 8)) >> 4)).clamp(0, 1023);
            let v = vig[((cxr[x] + ry) as usize).min(539)];
            let or = (tmap[0][ir as usize] as i32 * v) >> 10;
            let og = (tmap[1][ig as usize] as i32 * v) >> 10;
            let ob = (tmap[2][ib as usize] as i32 * v) >> 10;
            let lum = (77 * or + 150 * og + 29 * ob) >> 8;
            let g = nrow[(x + nox) & 255];
            let sat = |o: i32| ((lum + (((o - lum) * 287) >> 8) + g).clamp(0, 1023)) >> 2;
            out[x] = ((sat(or) as u32) << 16) | ((sat(og) as u32) << 8) | sat(ob) as u32;
        }
    }
}

/// Film grain in 1/1023 units (noise is -0.5..0.5, amplitude 0.02).
#[inline]
fn noise_i(n: f32) -> i32 {
    (n * 0.02 * 1023.0) as i32
}
