//! High-resolution output: the 640x400 picture is scaled with a "sharp bilinear" filter to the full
//! height of a 16:9 frame (Full HD, 2K, 3K, 4K or 8K) and centred, with black side bars. Pixel edges
//! stay crisp (the blend is confined to about one output pixel around each source edge) and nothing
//! shimmers while the road scrolls. The game is still rendered at 640x400; this is an upscale.

use crate::{H, W};

/// Output resolution. The frame is 16:9, the picture (1.6:1) fills its height.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res {
    /// 1920x1080
    Hd,
    /// 2560x1440
    K2,
    /// 3200x1800
    K3,
    /// 3840x2160
    K4,
    /// 7680x4320
    K8,
}

impl Res {
    pub const ALL: [Res; 5] = [Res::Hd, Res::K2, Res::K3, Res::K4, Res::K8];

    /// Frame size in pixels.
    pub fn frame(self) -> (usize, usize) {
        match self {
            Res::Hd => (1920, 1080),
            Res::K2 => (2560, 1440),
            Res::K3 => (3200, 1800),
            Res::K4 => (3840, 2160),
            Res::K8 => (7680, 4320),
        }
    }

    /// Size of the scaled picture inside the frame.
    pub fn pic(self) -> (usize, usize) {
        let h = self.frame().1;
        (h * 8 / 5, h)
    }

    pub fn label(self) -> &'static str {
        match self {
            Res::Hd => "Full HD",
            Res::K2 => "2K",
            Res::K3 => "3K",
            Res::K4 => "4K",
            Res::K8 => "8K",
        }
    }

    /// Parses `hd`, `fullhd`, `1080`, `2k`, `1440`, `3k`, `4k`, `2160`, `8k`, `4320` (any case).
    pub fn parse(s: &str) -> Option<Res> {
        match s.to_ascii_lowercase().as_str() {
            "hd" | "fhd" | "fullhd" | "full-hd" | "1080" | "1080p" => Some(Res::Hd),
            "2k" | "qhd" | "1440" | "1440p" => Some(Res::K2),
            "3k" | "1800" | "1800p" => Some(Res::K3),
            "4k" | "uhd" | "2160" | "2160p" => Some(Res::K4),
            "8k" | "4320" | "4320p" => Some(Res::K8),
            _ => None,
        }
    }
}

// Kept for code written against the first Full HD version.
pub const HD_W: usize = 1920;
pub const HD_H: usize = 1080;
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
    res: Res,
    xs: Vec<(u32, u32, u32)>,
    ys: Vec<(u32, u32, u32)>,
    rows: Vec<u32>,
}

impl Default for Upscaler {
    fn default() -> Self {
        Self::new(Res::Hd)
    }
}

fn threads() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        1
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::available_parallelism()
            .map_or(1, |n| n.get())
            .clamp(1, 8)
    }
}

impl Upscaler {
    pub fn new(res: Res) -> Upscaler {
        let (pw, ph) = res.pic();
        Upscaler {
            res,
            xs: axis(W, pw),
            ys: axis(H, ph),
            rows: vec![0; H * pw],
        }
    }

    pub fn res(&self) -> Res {
        self.res
    }

    /// Scales the 640x400 `src` into `dst` (a frame of `res().frame()` pixels, black side bars).
    pub fn run(&mut self, src: &[u32], dst: &mut [u32]) {
        let (fw, fh) = self.res.frame();
        let (pw, ph) = self.res.pic();
        if src.len() < W * H || dst.len() < fw * fh {
            return;
        }
        let nt = threads();
        let xs = &self.xs;
        // horizontal pass: 400 rows of the picture width
        let band = H.div_ceil(nt);
        std::thread::scope(|sc| {
            for (ci, chunk) in self.rows.chunks_mut(band * pw).enumerate() {
                let mut job = move || {
                    for (r, out) in chunk.chunks_mut(pw).enumerate() {
                        let y = ci * band + r;
                        let row = &src[y * W..(y + 1) * W];
                        for (o, &(a, b, w)) in out.iter_mut().zip(xs) {
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
                };
                if nt == 1 {
                    job();
                } else {
                    sc.spawn(job);
                }
            }
        });
        // vertical pass: every output line blends two rows; threads take bands of output lines
        let bar = (fw - pw) / 2;
        let (rows, ys) = (&self.rows, &self.ys);
        let band = ph.div_ceil(nt);
        std::thread::scope(|sc| {
            for (ci, chunk) in dst[..fw * fh].chunks_mut(band * fw).enumerate() {
                let mut job = move || {
                    for (r, line) in chunk.chunks_mut(fw).enumerate() {
                        let (a, b, w) = ys[ci * band + r];
                        line[..bar].fill(0);
                        line[bar + pw..].fill(0);
                        let (ra, rb) = (a as usize * pw, b as usize * pw);
                        let pic = &mut line[bar..bar + pw];
                        if w == 0 {
                            pic.copy_from_slice(&rows[ra..ra + pw]);
                        } else if w == 256 {
                            pic.copy_from_slice(&rows[rb..rb + pw]);
                        } else {
                            for (i, o) in pic.iter_mut().enumerate() {
                                *o = lerp(rows[ra + i], rows[rb + i], w);
                            }
                        }
                    }
                };
                if nt == 1 {
                    job();
                } else {
                    sc.spawn(job);
                }
            }
        });
    }
}
