//! Full HD output: the 640x400 picture is scaled to 1728x1080 with a "sharp bilinear" filter and
//! centred in a 1920x1080 frame. Pixel edges stay crisp (the blend is confined to about one output
//! pixel around each source edge), nothing shimmers while the road scrolls, and the whole pass costs
//! a few milliseconds, so the game keeps its 60 frames a second.

use crate::{H, W};

pub const HD_W: usize = 1920;
pub const HD_H: usize = 1080;
/// Size of the scaled picture inside the 1920x1080 frame (the rest is black side bars).
pub const PIC_W: usize = 1728;
pub const PIC_H: usize = 1080;

#[inline]
fn lerp(a: u32, b: u32, w: u32) -> u32 {
    let rb = (((a & 0x00FF_00FF) * (256 - w) + (b & 0x00FF_00FF) * w) >> 8) & 0x00FF_00FF;
    let g = (((a & 0x0000_FF00) * (256 - w) + (b & 0x0000_FF00) * w) >> 8) & 0x0000_FF00;
    rb | g
}

/// For each output coordinate: the two source pixels and the blend weight 0..=256.
fn axis(src: usize, dst: usize) -> Vec<(u32, u32, u32)> {
    let s = dst as f32 / src as f32;
    (0..dst)
        .map(|o| {
            let u = (o as f32 + 0.5) / s - 0.5; // position in source pixel centres
            let i = u.floor();
            let f = u - i;
            // sharpen: only the last 1/s of the way between two centres is blended
            let f = ((f - 0.5) * s + 0.5).clamp(0.0, 1.0);
            let i0 = (i as i32).clamp(0, src as i32 - 1) as u32;
            let i1 = (i as i32 + 1).clamp(0, src as i32 - 1) as u32;
            (i0, i1, (f * 256.0).round() as u32)
        })
        .collect()
}

pub struct Upscaler {
    xs: Vec<(u32, u32, u32)>,
    ys: Vec<(u32, u32, u32)>,
    rows: Vec<u32>,
}

impl Default for Upscaler {
    fn default() -> Self {
        Self::new()
    }
}

impl Upscaler {
    pub fn new() -> Upscaler {
        Upscaler {
            xs: axis(W, PIC_W),
            ys: axis(H, PIC_H),
            rows: vec![0; H * PIC_W],
        }
    }

    /// Scales the 640x400 `src` into the 1920x1080 `dst` (black side bars).
    pub fn run(&mut self, src: &[u32], dst: &mut [u32]) {
        if src.len() < W * H || dst.len() < HD_W * HD_H {
            return;
        }
        for y in 0..H {
            let row = &src[y * W..(y + 1) * W];
            let out = &mut self.rows[y * PIC_W..(y + 1) * PIC_W];
            for (o, &(a, b, w)) in out.iter_mut().zip(&self.xs) {
                let (pa, pb) = (row[a as usize], row[b as usize]);
                *o = if w == 0 || pa == pb {
                    pa
                } else if w == 256 {
                    pb
                } else {
                    lerp(pa, pb, w)
                };
            }
        }
        let bar = (HD_W - PIC_W) / 2;
        for (oy, &(a, b, w)) in self.ys.iter().enumerate() {
            let line = &mut dst[oy * HD_W..(oy + 1) * HD_W];
            line[..bar].fill(0);
            line[bar + PIC_W..].fill(0);
            let (ra, rb) = (a as usize * PIC_W, b as usize * PIC_W);
            let pic = &mut line[bar..bar + PIC_W];
            if w == 0 {
                pic.copy_from_slice(&self.rows[ra..ra + PIC_W]);
            } else if w == 256 {
                pic.copy_from_slice(&self.rows[rb..rb + PIC_W]);
            } else {
                for (i, o) in pic.iter_mut().enumerate() {
                    *o = lerp(self.rows[ra + i], self.rows[rb + i], w);
                }
            }
        }
    }
}
