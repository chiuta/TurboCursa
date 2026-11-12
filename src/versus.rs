//! Two-player split-screen race: two cars on the same stage, top and bottom halves.
use crate::fractal::Assets;
use crate::game::*;
use crate::i18n::{self, S};
use crate::render;
use crate::{H, W};

pub struct Versus {
    pub p: [Game; 2],
    pub winner: Option<usize>,
    pub frozen: bool,
    pub seed: u64,
    /// player 2 is driven by the autopilot
    pub cpu: bool,
    over_t: f32,
    bufs: [Vec<u32>; 2],
}

const P_COL: [u32; 2] = [0xFF5A5A, 0x5AAAFF];

impl Versus {
    pub fn new(seed: u64) -> Self {
        let assets = Assets::global(seed);
        let mut p = [
            Game::with_assets(seed, assets.clone()),
            Game::with_assets(seed, assets),
        ];
        for g in &mut p {
            g.set_view(W, H / 2);
        }
        Versus {
            p,
            winner: None,
            frozen: false,
            seed,
            cpu: false,
            over_t: 0.0,
            bufs: [vec![0; W * H / 2], vec![0; W * H / 2]],
        }
    }

    pub fn new_cpu(seed: u64) -> Self {
        let mut v = Self::new(seed);
        v.cpu = true;
        v
    }

    /// Fit cars (country liveries and handling profiles) to the two players.
    pub fn set_cars(&mut self, a: crate::cars::CarSpec, b: crate::cars::CarSpec) {
        self.p[0].set_car(a);
        self.p[1].set_car(b);
    }

    /// Same difficulty for both players (before the start).
    pub fn set_env(&mut self, weather: usize, surface: usize) {
        self.p[0].set_env(weather, surface);
        self.p[1].set_env(weather, surface);
    }

    /// Time of day (0 = automatic, 1 dawn, 2 day, 3 dusk, 4 night), same for both players.
    pub fn set_time(&mut self, i: usize) {
        self.p[0].set_time(i);
        self.p[1].set_time(i);
    }

    /// Tyres and set-up, same for both players.
    pub fn set_equipment(&mut self, t: Option<crate::env::Tyre>, s: crate::env::Setup) {
        self.p[0].set_equipment(t, s);
        self.p[1].set_equipment(t, s);
    }

    pub fn set_access(&mut self, assist: u8, speed_cap: f32) {
        self.p[0].set_access(assist, speed_cap);
        self.p[1].set_access(assist, speed_cap);
    }

    pub fn set_rank(&mut self, r: u16) {
        self.p[0].set_rank(r);
        self.p[1].set_rank(r);
    }

    pub fn set_level(&mut self, l: crate::game::Difficulty) {
        self.p[0].set_level(l);
        self.p[1].set_level(l);
    }

    /// True when the round is finished and ENTER should start a new one.
    pub fn is_over(&self) -> bool {
        self.frozen || self.p.iter().all(|g| g.state != State::Playing)
    }

    pub fn update(&mut self, dt: f32, inp: &[Input; 2]) {
        if self.frozen {
            return;
        }
        let mut inp = *inp;
        if self.cpu {
            inp[1] = self.p[1].autopilot();
        }
        for i in 0..2 {
            self.p[i].update(dt, &inp[i]);
        }
        if self.winner.is_none() {
            let f = [
                self.p[0].state == State::Finished,
                self.p[1].state == State::Finished,
            ];
            if f[0] || f[1] {
                self.winner = Some(if f[0] && f[1] {
                    if self.p[0].pos >= self.p[1].pos {
                        0
                    } else {
                        1
                    }
                } else if f[0] {
                    0
                } else {
                    1
                });
            }
        }
        if self.winner.is_none() {
            // the police take a player out: the other one wins
            for i in 0..2 {
                if self.p[i].state == State::Disqualified
                    && self.p[1 - i].state != State::Disqualified
                {
                    self.winner = Some(1 - i);
                }
            }
        }
        if self.winner.is_some() {
            self.over_t += dt;
            let loser_done = self.p.iter().all(|g| g.state != State::Playing);
            if loser_done || self.over_t > 4.0 {
                self.frozen = true;
            }
        }
    }

    /// 0-based rank of player `i` by distance covered.
    fn rank(&self, i: usize) -> usize {
        if let Some(w) = self.winner {
            return (i != w) as usize;
        }
        (self.p[i].pos < self.p[1 - i].pos) as usize
    }

    pub fn render(&mut self, out: &mut [u32]) {
        assert_eq!(out.len(), W * H);
        let half = H / 2;
        for i in 0..2 {
            let rank = self.rank(i);
            {
                let (g, buf) = (&self.p[i], &mut self.bufs[i]);
                render::render_view(g, buf, W, half);
            }
            let who = if self.cpu && i == 1 {
                "CPU".to_string()
            } else {
                format!("P{}", i + 1)
            };
            let tag = format!("{} {}", who, i18n::rank_tag(rank));
            render::overlay_text_at(&mut self.bufs[i], W, half, 6, 38, &tag, 2, P_COL[i]);
            if let Some(w) = self.winner {
                let (msg, col) = if w == i {
                    (i18n::tr(S::Winner), 0xFFD83C)
                } else {
                    (i18n::tr(S::SecondPlace), 0xDDDDEE)
                };
                render::overlay_text(
                    &mut self.bufs[i],
                    W,
                    half,
                    (half as i32) / 2 - 12,
                    msg,
                    3,
                    col,
                );
            } else if self
                .p
                .iter()
                .all(|g| matches!(g.state, State::TimeUp | State::Disqualified))
            {
                render::overlay_text(
                    &mut self.bufs[i],
                    W,
                    half,
                    (half as i32) / 2 - 12,
                    i18n::tr(S::Draw),
                    3,
                    0xDDDDEE,
                );
            }
            out[i * half * W..(i + 1) * half * W].copy_from_slice(&self.bufs[i]);
        }
        // divider
        for y in half - 1..half + 1 {
            for x in 0..W {
                out[y * W + x] = 0x0A0A14;
            }
        }
        if self.is_over() {
            render::overlay_text_at(
                out,
                W,
                H,
                (W as i32 - crate::font::text_width(i18n::tr(S::Rematch), 1)) / 2,
                H as i32 / 2 - 4,
                i18n::tr(S::Rematch),
                1,
                0xFFFFFF,
            );
        }
    }
}
