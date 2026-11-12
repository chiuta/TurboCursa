//! Three or four cars on the same stage in a 2x2 split screen. Players beyond the humans are driven
//! by the autopilot ("CPU fill"). First to the finish wins; the others are ranked by distance.
use crate::fractal::Assets;
use crate::game::*;
use crate::i18n::{self, S};
use crate::render;
use crate::{H, W};

pub const MAX_PLAYERS: usize = 4;
const P_COL: [u32; MAX_PLAYERS] = [0xFF5A5A, 0x5AAAFF, 0x6BE675, 0xFFD23C];

pub struct Multi {
    pub p: Vec<Game>,
    /// number of human players (the first ones); the rest is autopilot
    pub humans: usize,
    pub winner: Option<usize>,
    pub frozen: bool,
    pub seed: u64,
    over_t: f32,
    bufs: Vec<Vec<u32>>,
}

impl Multi {
    /// `players` 3..=4 cars of which `humans` (1..=players) are driven by people.
    pub fn new(seed: u64, players: usize, humans: usize) -> Multi {
        let n = players.clamp(3, MAX_PLAYERS);
        let assets = Assets::global(seed);
        let mut p: Vec<Game> = (0..n)
            .map(|_| Game::with_assets(seed, assets.clone()))
            .collect();
        for g in &mut p {
            g.set_view(W / 2, H / 2);
        }
        Multi {
            p,
            humans: humans.clamp(1, n),
            winner: None,
            frozen: false,
            seed,
            over_t: 0.0,
            bufs: (0..n).map(|_| vec![0; W / 2 * H / 2]).collect(),
        }
    }

    pub fn set_cars(&mut self, cars: &[crate::cars::CarSpec]) {
        for (g, c) in self.p.iter_mut().zip(cars.iter().cycle()) {
            g.set_car(*c);
        }
    }

    pub fn set_level(&mut self, l: Difficulty) {
        self.p.iter_mut().for_each(|g| g.set_level(l));
    }

    pub fn set_env(&mut self, weather: usize, surface: usize) {
        self.p.iter_mut().for_each(|g| g.set_env(weather, surface));
    }

    pub fn set_time(&mut self, i: usize) {
        self.p.iter_mut().for_each(|g| g.set_time(i));
    }

    pub fn set_rank(&mut self, r: u16) {
        self.p.iter_mut().for_each(|g| g.set_rank(r));
    }

    pub fn is_over(&self) -> bool {
        self.frozen || self.p.iter().all(|g| g.state != State::Playing)
    }

    /// Players still in the race, best distance first.
    pub fn order(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.p.len()).collect();
        v.sort_by(|&a, &b| {
            let key = |i: usize| {
                (
                    self.winner == Some(i),
                    self.p[i].state != State::Disqualified,
                    self.p[i].pos,
                )
            };
            key(b)
                .partial_cmp(&key(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v
    }

    pub fn place(&self, i: usize) -> usize {
        self.order().iter().position(|&x| x == i).unwrap_or(0) + 1
    }

    pub fn update(&mut self, dt: f32, inp: &[Input; MAX_PLAYERS]) {
        if self.frozen {
            return;
        }
        for i in 0..self.p.len() {
            let input = if i < self.humans {
                inp[i]
            } else {
                self.p[i].autopilot()
            };
            self.p[i].update(dt, &input);
        }
        if self.winner.is_none() {
            // first to finish; when several finish in one step the one furthest along wins
            self.winner = (0..self.p.len())
                .filter(|&i| self.p[i].state == State::Finished)
                .max_by(|&a, &b| {
                    self.p[a]
                        .pos
                        .partial_cmp(&self.p[b].pos)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
        }
        if self.winner.is_some() {
            self.over_t += dt;
            if self.p.iter().all(|g| g.state != State::Playing) || self.over_t > 4.0 {
                self.frozen = true;
            }
        }
    }

    pub fn render(&mut self, out: &mut [u32]) {
        assert_eq!(out.len(), W * H);
        let (qw, qh) = (W / 2, H / 2);
        out.fill(0x0A0A14);
        for i in 0..self.p.len() {
            render::render_view(&self.p[i], &mut self.bufs[i], qw, qh);
            let who = if i < self.humans {
                format!("P{}", i + 1)
            } else {
                "CPU".to_string()
            };
            let tag = format!("{} #{}", who, self.place(i));
            render::overlay_text_at(&mut self.bufs[i], qw, qh, 4, 30, &tag, 1, P_COL[i]);
            if let Some(w) = self.winner {
                if w == i {
                    render::overlay_text(
                        &mut self.bufs[i],
                        qw,
                        qh,
                        qh as i32 / 2 - 8,
                        i18n::tr(S::Winner),
                        2,
                        0xFFD83C,
                    );
                }
            }
            let (ox, oy) = ((i % 2) * qw, (i / 2) * qh);
            for y in 0..qh {
                let dst = (oy + y) * W + ox;
                out[dst..dst + qw].copy_from_slice(&self.bufs[i][y * qw..(y + 1) * qw]);
            }
        }
        // dividers
        for y in 0..H {
            out[y * W + qw - 1] = 0x0A0A14;
            out[y * W + qw] = 0x0A0A14;
        }
        for x in 0..W {
            out[(qh - 1) * W + x] = 0x0A0A14;
            out[qh * W + x] = 0x0A0A14;
        }
    }
}
