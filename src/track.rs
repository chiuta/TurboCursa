use crate::fractal::fbm;
use crate::rng::Rng;

pub const SEG_LEN: f32 = 200.0;
/// Road half-width in world units.
pub const ROAD_W: f32 = 2000.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Prop {
    Pine,
    Round,
    Rock,
    Cactus,
    Dead,
    Sign,
    Lamp,
    Gate,
    Finish,
    /// colossal world-tree trunk with a cloud collar
    Giant,
    /// glowing flower / fern cluster
    Bloom,
    /// red mesa butte
    Spire,
    /// sage bush
    Scrub,
    /// hovering rock chunk
    Hover,
    /// glowing crystal cluster
    Crystal,
    /// black volcanic glass with glowing seams
    Obsidian,
    /// swaying kelp strands
    Kelp,
    /// recursively branching coral
    Coral,
    /// neon Sierpinski pyramid
    Sierpinski,
    /// neon Menger-style carpet block
    Menger,
    /// giant glowing mushroom
    Mushroom,
}

impl Prop {
    pub fn solid(self) -> bool {
        !matches!(
            self,
            Prop::Gate
                | Prop::Finish
                | Prop::Bloom
                | Prop::Scrub
                | Prop::Hover
                | Prop::Giant
                | Prop::Kelp
        )
    }
}

#[derive(Clone, Copy)]
pub struct Sprite {
    pub kind: Prop,
    /// Lateral offset in road half-widths (±1 is the road edge).
    pub offset: f32,
    pub variant: u8,
}

/// Road structures: what the road runs through or over.
pub const F_NONE: u8 = 0;
pub const F_BRIDGE: u8 = 1;
pub const F_TUNNEL: u8 = 2;
pub const F_VIADUCT: u8 = 3;
pub const F_PASS: u8 = 4;
pub const FEAT_COUNT: u8 = 5;

/// Playable half-width (in road half-widths) while on a structure; the walls or rails start there.
pub fn feat_limit(f: u8) -> f32 {
    match f {
        F_BRIDGE | F_VIADUCT => 1.0,
        F_TUNNEL => 1.0,
        F_PASS => 0.82,
        _ => 99.0,
    }
}

#[derive(Clone)]
pub struct Segment {
    /// structure this segment belongs to (F_*)
    pub feat: u8,
    pub y0: f32,
    pub y1: f32,
    pub curve: f32,
    pub biome: u8,
    /// fBm-derived terrain tone in \[0,1\], used for patchy ground colour
    pub tone: f32,
    pub props: Vec<Sprite>,
}

#[derive(Clone)]
pub struct Track {
    pub segs: Vec<Segment>,
    /// Segment indices of mid-stage checkpoints.
    pub checkpoints: Vec<usize>,
    pub finish_seg: usize,
    pub length: f32,
    /// (first segment, one past the last, F_*) of every bridge, tunnel, viaduct and mountain pass
    pub features: Vec<(usize, usize, u8)>,
}

impl Track {
    pub fn seg_at(&self, z: f32) -> &Segment {
        let n = self.segs.len();
        &self.segs[(z / SEG_LEN) as usize % n]
    }
}

fn ease_in(a: f32, b: f32, p: f32) -> f32 {
    a + (b - a) * p * p
}
fn ease_io(a: f32, b: f32, p: f32) -> f32 {
    a + (b - a) * (0.5 - (p * std::f32::consts::PI).cos() * 0.5)
}

struct Builder {
    segs: Vec<Segment>,
    last_y: f32,
    biome: u8,
}

impl Builder {
    fn add(&mut self, curve: f32, y: f32) {
        self.segs.push(Segment {
            y0: self.last_y,
            y1: y,
            curve,
            feat: F_NONE,
            biome: self.biome,
            tone: 0.5,
            props: Vec::new(),
        });
        self.last_y = y;
    }

    fn road(&mut self, enter: usize, hold: usize, leave: usize, curve: f32, hill: f32) {
        let start = self.last_y;
        let end = start + hill * SEG_LEN;
        let total = (enter + hold + leave) as f32;
        for n in 0..enter {
            let p = n as f32 / enter as f32;
            self.add(
                ease_in(0.0, curve, p),
                ease_io(start, end, (n + 1) as f32 / total),
            );
        }
        for n in 0..hold {
            self.add(curve, ease_io(start, end, (enter + n + 1) as f32 / total));
        }
        for n in 0..leave {
            let p = n as f32 / leave as f32;
            self.add(
                ease_io(curve, 0.0, p),
                ease_io(start, end, (enter + hold + n + 1) as f32 / total),
            );
        }
    }
}

/// (enter, hold, leave, curve, hill)
type Piece = (usize, usize, usize, f32, f32);

/// Fractal stage layout: curve and hill profiles are fBm samples, so every seed is a new road.
fn gen_script(
    rng: &mut Rng,
    seed: u32,
    biome: u32,
    count: usize,
    twist: f32,
    hills: f32,
) -> Vec<Piece> {
    let mut v: Vec<Piece> = Vec::new();
    for i in 0..count {
        let t = biome as f32 * 40.0 + i as f32 * 0.9;
        let c = (fbm(t, 1.7, seed, 4, 0) - 0.5) * 3.8;
        let curve = if rng.f() < 0.22 {
            0.0
        } else {
            (c * 7.0 * twist).clamp(-6.5, 6.5)
        };
        let hill = (fbm(t, 9.3, seed.wrapping_add(7), 4, 0) - 0.5) * 3.8 * 38.0 * hills;
        let e = 20 + rng.below(12) as usize;
        let h = 25 + rng.below(50) as usize;
        let l = 20 + rng.below(12) as usize;
        v.push((e, h, l, curve, hill));
    }
    // zero net elevation per biome so the loop closes without a visible seam
    let mean = v.iter().map(|p| p.4).sum::<f32>() / v.len() as f32;
    for p in &mut v {
        p.4 = (p.4 - mean).clamp(-22.0, 22.0);
    }
    // clamping can unbalance the sum; re-centre so the biome returns exactly to its start height
    let mean = v.iter().map(|p| p.4).sum::<f32>() / v.len() as f32;
    for p in &mut v {
        p.4 -= mean;
    }
    v
}

/// Stage designer settings. The default layout is exactly the classic seeded stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    /// the four landscapes in driving order; `AUTO` = the seeded choice
    pub biomes: [u8; 4],
    /// 0..=3: how twisty the road is (0.4x, 1x, 1.5x, 2x)
    pub twist: u8,
    /// 0..=3: how hilly (0.3x, 1x, 1.6x, 2.2x)
    pub hills: u8,
    /// 0..=2: pieces per landscape (6, 9, 12)
    pub length: u8,
}

impl Layout {
    pub const AUTO: u8 = 255;
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            biomes: [Layout::AUTO; 4],
            twist: 1,
            hills: 1,
            length: 1,
        }
    }
}

impl Layout {
    /// Clamps every field to its valid range (hostile codes).
    pub fn sanitized(mut self) -> Layout {
        for b in &mut self.biomes {
            if *b as usize >= crate::NB {
                *b = Layout::AUTO;
            }
        }
        self.twist = self.twist.min(3);
        self.hills = self.hills.min(3);
        self.length = self.length.min(2);
        self
    }
}

thread_local! {
    static LAYOUT: std::cell::Cell<Layout> = const { std::cell::Cell::new(Layout { biomes: [Layout::AUTO; 4], twist: 1, hills: 1, length: 1 }) };
}

/// Runs `f` with `layout` as the layout every `build` call uses (restored afterwards).
pub fn with_layout<R>(layout: Layout, f: impl FnOnce() -> R) -> R {
    let old = LAYOUT.with(|l| l.replace(layout.sanitized()));
    let r = f();
    LAYOUT.with(|l| l.set(old));
    r
}

pub fn build(seed: u64) -> Track {
    let layout = LAYOUT.with(|l| l.get());
    let twist = [0.4, 1.0, 1.5, 2.0][layout.twist as usize];
    let hills = [0.3, 1.0, 1.6, 2.2][layout.hills as usize];
    let count = [6usize, 9, 12][layout.length as usize];
    let s32 = (seed as u32).wrapping_mul(2_654_435_761) ^ 0x5bd1_e995;
    let mut rng = Rng::new(seed ^ 0xC0FFEE);
    let mut b = Builder {
        segs: Vec::new(),
        last_y: 0.0,
        biome: 0,
    };
    let mut checkpoints = Vec::new();
    // route: one classic biome plus three of the fifteen newer landscapes, in a seeded order
    let mut news = [3u8, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];
    for i in (1..news.len()).rev() {
        news.swap(i, rng.below(i as u32 + 1) as usize);
    }
    let classic = [0u8, 1, 2, 7][rng.below(4) as usize];
    let mut order = [classic, news[0], news[1], news[2]];
    for i in (1..order.len()).rev() {
        order.swap(i, rng.below(i as u32 + 1) as usize);
    }
    for (i, &b) in layout.biomes.iter().enumerate() {
        if b != Layout::AUTO {
            order[i] = b;
        }
    }
    for (stage, &biome) in order.iter().enumerate() {
        if stage > 0 {
            checkpoints.push(b.segs.len());
        }
        b.biome = biome;
        let mut script = gen_script(&mut rng, s32, biome as u32, count, twist, hills);
        if stage == 0 {
            script[0] = (20, 40, 20, 0.0, 0.0);
            // the flat start piece removed some elevation: re-balance the rest so the loop still closes
            let m = script[1..].iter().map(|p| p.4).sum::<f32>() / (script.len() - 1) as f32;
            for p in &mut script[1..] {
                p.4 -= m;
            }
        }
        for (e, h, l, c, hill) in script {
            b.road(e, h, l, c, hill);
        }
    }
    // run-out straight, then close the loop at ground level
    b.road(10, 40, 10, 0.0, 0.0);
    let n = b.segs.len();
    b.segs[n - 1].y1 = 0.0;
    let finish_seg = n - 30;

    // scenery
    let mut rng = Rng::new(seed ^ 0xA5A5);
    for i in 0..n {
        let curve = b.segs[i].curve;
        let biome = b.segs[i].biome;
        for side in [-1.0f32, 1.0] {
            let (p, kinds): (f32, &[Prop]) = match biome {
                0 => (0.30, &[Prop::Pine, Prop::Pine, Prop::Round]),
                1 => (0.14, &[Prop::Cactus, Prop::Rock, Prop::Dead, Prop::Cactus]),
                2 => (0.24, &[Prop::Pine, Prop::Rock, Prop::Dead, Prop::Pine]),
                3 => (
                    0.30,
                    &[
                        Prop::Pine,
                        Prop::Round,
                        Prop::Round,
                        Prop::Rock,
                        Prop::Bloom,
                        Prop::Bloom,
                    ],
                ),
                4 => (
                    0.18,
                    &[
                        Prop::Spire,
                        Prop::Scrub,
                        Prop::Scrub,
                        Prop::Rock,
                        Prop::Dead,
                        Prop::Cactus,
                    ],
                ),
                5 => (
                    0.22,
                    &[
                        Prop::Hover,
                        Prop::Hover,
                        Prop::Round,
                        Prop::Bloom,
                        Prop::Rock,
                        Prop::Pine,
                    ],
                ),
                6 => (
                    0.24,
                    &[Prop::Crystal, Prop::Crystal, Prop::Rock, Prop::Crystal],
                ),
                7 => (
                    0.26,
                    &[
                        Prop::Pine,
                        Prop::Pine,
                        Prop::Pine,
                        Prop::Rock,
                        Prop::Crystal,
                    ],
                ),
                8 => (
                    0.24,
                    &[Prop::Obsidian, Prop::Obsidian, Prop::Rock, Prop::Dead],
                ),
                9 => (
                    0.30,
                    &[
                        Prop::Kelp,
                        Prop::Kelp,
                        Prop::Coral,
                        Prop::Coral,
                        Prop::Rock,
                        Prop::Bloom,
                    ],
                ),
                10 => (
                    0.20,
                    &[
                        Prop::Crystal,
                        Prop::Hover,
                        Prop::Hover,
                        Prop::Crystal,
                        Prop::Rock,
                    ],
                ),
                11 => (
                    0.30,
                    &[
                        Prop::Sierpinski,
                        Prop::Menger,
                        Prop::Menger,
                        Prop::Sierpinski,
                        Prop::Crystal,
                    ],
                ),
                13 => (
                    0.26,
                    &[
                        Prop::Spire,
                        Prop::Hover,
                        Prop::Crystal,
                        Prop::Dead,
                        Prop::Rock,
                    ],
                ),
                14 => (
                    0.30,
                    &[
                        Prop::Crystal,
                        Prop::Crystal,
                        Prop::Crystal,
                        Prop::Spire,
                        Prop::Rock,
                    ],
                ),
                15 => (
                    0.28,
                    &[
                        Prop::Menger,
                        Prop::Menger,
                        Prop::Lamp,
                        Prop::Obsidian,
                        Prop::Rock,
                    ],
                ),
                16 => (
                    0.22,
                    &[
                        Prop::Hover,
                        Prop::Bloom,
                        Prop::Round,
                        Prop::Hover,
                        Prop::Spire,
                    ],
                ),
                17 => (
                    0.18,
                    &[Prop::Obsidian, Prop::Hover, Prop::Crystal, Prop::Sierpinski],
                ),
                18 => (
                    0.24,
                    &[
                        Prop::Spire,
                        Prop::Spire,
                        Prop::Rock,
                        Prop::Cactus,
                        Prop::Scrub,
                        Prop::Sierpinski,
                    ],
                ),
                _ => (
                    0.32,
                    &[
                        Prop::Mushroom,
                        Prop::Mushroom,
                        Prop::Mushroom,
                        Prop::Bloom,
                        Prop::Rock,
                    ],
                ),
            };
            if rng.f() < p {
                let kind = kinds[rng.below(kinds.len() as u32) as usize];
                let off = side * rng.range(1.25, 3.4);
                let variant = rng.below(8) as u8;
                b.segs[i].props.push(Sprite {
                    kind,
                    offset: off,
                    variant,
                });
            }
        }
        // colossal world-trees stand far off the road
        if biome == 3 && rng.f() < 0.07 {
            let side = if rng.f() < 0.5 { -1.0 } else { 1.0 };
            let variant = rng.below(8) as u8;
            b.segs[i].props.push(Sprite {
                kind: Prop::Giant,
                offset: side * rng.range(3.9, 6.6),
                variant,
            });
        }
        // warning sign ahead of sharp bends
        if curve.abs() >= 4.0 && i >= 18 && b.segs[i - 1].curve.abs() < 4.0 {
            let side = if curve > 0.0 { -1.0 } else { 1.0 };
            b.segs[i - 18].props.push(Sprite {
                kind: Prop::Sign,
                offset: side * 1.25,
                variant: 0,
            });
        }
    }
    for i in (0..n).step_by(6) {
        if b.segs[i].biome == 2 {
            let side = if (i / 6) % 2 == 0 { -1.0 } else { 1.0 };
            b.segs[i].props.push(Sprite {
                kind: Prop::Lamp,
                offset: side * 1.18,
                variant: 0,
            });
        }
    }
    for (gi, &c) in checkpoints.iter().enumerate() {
        b.segs[c].props.push(Sprite {
            kind: Prop::Gate,
            offset: 0.0,
            variant: gi as u8,
        });
    }
    for (i, s) in b.segs.iter_mut().enumerate() {
        s.tone = fbm(i as f32 * 0.03, 5.5, s32, 4, 0);
    }
    b.segs[finish_seg].props.push(Sprite {
        kind: Prop::Finish,
        offset: 0.0,
        variant: 0,
    });

    // structures: one candidate window per slice of the stage, clear of start, checkpoints and finish
    let mut frng = Rng::new(seed ^ 0x57A71C);
    let mut features: Vec<(usize, usize, u8)> = Vec::new();
    let slices = 7usize;
    let first = 70usize;
    let last = finish_seg.saturating_sub(40);
    if last > first + 200 {
        let w = (last - first) / slices;
        for s in 0..slices {
            let kind = 1 + frng.below(4) as u8;
            let len = match kind {
                F_BRIDGE => 30 + frng.below(30),
                F_TUNNEL => 55 + frng.below(60),
                F_VIADUCT => 50 + frng.below(40),
                _ => 45 + frng.below(40),
            } as usize;
            let len = len.min(w.saturating_sub(24));
            if len < 20 {
                continue;
            }
            let a = first + s * w + 6 + frng.below((w - len - 12).max(1) as u32) as usize;
            let e = a + len;
            let near_cp = checkpoints.iter().any(|&c| c + 10 > a && c < e + 10);
            if near_cp || e + 10 >= finish_seg {
                continue;
            }
            for sg in &mut b.segs[a..e] {
                sg.feat = kind;
                sg.props.clear();
            }
            features.push((a, e, kind));
        }
    }

    Track {
        features,
        length: n as f32 * SEG_LEN,
        segs: b.segs,
        checkpoints,
        finish_seg,
    }
}
