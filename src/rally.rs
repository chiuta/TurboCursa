//! Rally: a multi-stage event with changing weather, surface and time of day. Damage, fuel and
//! tyre wear carry over from stage to stage; between stages a service park lets you repair, change
//! tyres and refuel - every minute of service is added to your total time. The final position is
//! worked out against a simulated field of 15 crews.
use crate::career::splitmix;
use crate::career_ui::{fill, fmt_time, tx, txc};
use crate::cars::{CarSpec, VehicleClass};
use crate::env::{Setup, Tyre, SURFACE_COUNT, TYRE_COUNT, WEATHER_COUNT};
use crate::game::*;
use crate::i18n_career::{t, C};
use crate::pick::{SF, WX};
use crate::profile::GameResult;
use crate::{render, H, W};

pub const STAGES: usize = 5;
pub const FIELD: usize = 15;
/// Seconds added to the total time of a stage that is not finished.
pub const DNF_PENALTY: f32 = 90.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StagePlan {
    pub seed: u64,
    /// 0-based weather / surface indices
    pub weather: usize,
    pub surface: usize,
    /// 1 dawn, 2 day, 3 dusk, 4 night
    pub time: usize,
}

/// The plan of stage `i`: the day runs from dawn to night, the weather and surface change.
pub fn plan(seed: u64, i: usize) -> StagePlan {
    plan_n(seed, i, STAGES)
}

/// The plan of stage `i` in an event of `n` stages (championships and tours).
pub fn plan_n(seed: u64, i: usize, n: usize) -> StagePlan {
    let n = n.clamp(1, crate::champ::MAX_STAGES);
    let i = i.min(n - 1);
    let h = splitmix(seed ^ ((i as u64 + 1) * 0x9E37_79B9));
    let time = 1 + (i * 4 / n).min(3);
    // the first stage is on tarmac in fair weather, the later ones get wilder
    let surface = if i == 0 {
        0
    } else {
        (splitmix(h ^ 1) % SURFACE_COUNT as u64) as usize
    };
    let weather = if i == 0 {
        0
    } else {
        (splitmix(h ^ 2) % WEATHER_COUNT as u64) as usize
    };
    StagePlan {
        seed: seed.wrapping_add(i as u64 * 7919),
        weather,
        surface,
        time,
    }
}

/// Reference time of a stage (a calm, clean drive) used to scale the simulated field.
pub fn reference_time(g: &Game) -> f32 {
    g.track.finish_seg as f32 * crate::track::SEG_LEN / (MAX_SPEED * 0.5)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Briefing,
    Racing,
    StageEnd,
    Service,
    Final,
}

/// Service park choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Service {
    /// 0 none, 1 half, 2 full
    pub repair: u8,
    /// 0 = automatic (the best for the next stage), k = tyre set k-1
    pub tyre: usize,
    pub refuel: bool,
}

pub struct RallyApp {
    pub seed: u64,
    pub class: usize,
    pub level: usize,
    /// number of stages of this event
    pub n: usize,
    /// a country tour (index into `champ::COUNTRIES`)
    pub country: Option<usize>,
    pub phase: Phase,
    pub stage: usize,
    pub game: Option<Box<Game>>,
    pub stage_times: Vec<f32>,
    pub service_time: f32,
    pub penalty: f32,
    pub body: f32,
    pub engine: f32,
    pub fuel: f32,
    pub tyre_wear: f32,
    pub tyre: Tyre,
    pub svc: Service,
    cursor: usize,
    rival_skill: [f32; FIELD],
    rival_total: [f32; FIELD],
    pub position: usize,
    pub results: Vec<GameResult>,
    /// garage coins of the last stage and of the whole rally (set by the host, shown on the end screens)
    pub coins_stage: u32,
    pub coins_total: u32,
    pending: Option<GameResult>,
    prev: Input,
    t: f32,
}

fn skills(seed: u64) -> [f32; FIELD] {
    let mut s = [1.0; FIELD];
    for (i, v) in s.iter_mut().enumerate() {
        *v = 0.88 + (splitmix(seed ^ ((i as u64 + 1) * 0xABCD)) % 1000) as f32 / 1000.0 * 0.30;
    }
    s
}

impl RallyApp {
    pub fn new(seed: u64, class: usize, level: usize) -> RallyApp {
        RallyApp::with_stages(seed, class, level, STAGES)
    }

    /// An event of `n` stages (1..=15).
    pub fn with_stages(seed: u64, class: usize, level: usize, n: usize) -> RallyApp {
        RallyApp {
            n: n.clamp(1, crate::champ::MAX_STAGES),
            country: None,
            seed,
            class: class.min(crate::cars::CLASS_COUNT - 1),
            level: level.min(LEVEL_COUNT - 1),
            coins_stage: 0,
            coins_total: 0,
            phase: Phase::Briefing,
            stage: 0,
            game: None,
            stage_times: Vec::new(),
            service_time: 0.0,
            penalty: 0.0,
            body: 1.0,
            engine: 1.0,
            fuel: 1.0,
            tyre_wear: 1.0,
            tyre: Tyre::Road,
            svc: Service::default(),
            cursor: 0,
            rival_skill: skills(seed),
            rival_total: [0.0; FIELD],
            position: FIELD + 1,
            results: Vec::new(),
            pending: None,
            prev: Input::default(),
            t: 0.0,
        }
    }

    pub fn plan(&self) -> StagePlan {
        plan_n(self.seed, self.stage.min(self.n - 1), self.n)
    }

    /// Total time so far: stages + service + penalties.
    pub fn total(&self) -> f32 {
        self.stage_times.iter().sum::<f32>() + self.service_time + self.penalty
    }

    /// Seconds a service choice costs, from the carried condition of the car.
    pub fn service_cost(&self, s: Service) -> f32 {
        let missing = ((1.0 - self.body) + (1.0 - self.engine)) * 0.5;
        let repair = match s.repair {
            0 => 0.0,
            1 => missing * 0.5 * 40.0,
            _ => missing * 40.0,
        };
        let fuel = if s.refuel {
            (1.0 - self.fuel) * 14.0
        } else {
            0.0
        };
        let want = self.next_tyre(s);
        let tyres = if want != self.tyre || self.tyre_wear < 0.6 {
            9.0
        } else {
            0.0
        };
        repair + fuel + tyres
    }

    fn next_tyre(&self, s: Service) -> Tyre {
        if s.tyre == 0 {
            let p = plan_n(self.seed, (self.stage + 1).min(self.n - 1), self.n);
            Tyre::best_for(
                crate::env::Weather::from_index(p.weather),
                crate::env::Surface::from_index(p.surface),
            )
        } else {
            Tyre::from_index(s.tyre - 1)
        }
    }

    fn start_stage(&mut self) {
        let p = self.plan();
        let mut g = Game::new_mode(p.seed, Mode::TimeTrial);
        g.set_car(CarSpec::default().with_class(VehicleClass::from_index(self.class)));
        g.set_level(Difficulty::from_index(self.level));
        g.set_env(p.weather + 1, p.surface + 1);
        g.set_time(p.time);
        g.set_equipment(Some(self.tyre), Setup::Balanced);
        g.body = self.body;
        g.engine = self.engine;
        g.fuel = self.fuel;
        g.tyre_wear = self.tyre_wear;
        self.game = Some(Box::new(g));
        self.phase = Phase::Racing;
    }

    fn end_stage(&mut self) {
        let Some(g) = self.game.as_ref() else { return };
        let finished = g.state == State::Finished;
        let reference = reference_time(g);
        let time = if finished { g.elapsed } else { reference * 1.6 };
        let pen = if finished { 0.0 } else { DNF_PENALTY };
        self.body = g.body;
        self.engine = g.engine;
        self.fuel = g.fuel.max(0.15); // the tow truck brings a little fuel
        self.tyre_wear = g.tyre_wear;
        self.tyre = g.tyre;
        self.stage_times.push(time);
        self.penalty += pen;
        self.pending = Some(GameResult::from_game(g));
        if let Some(r) = self.pending {
            self.results.push(r);
        }
        let i = self.stage as u64;
        for (k, tot) in self.rival_total.iter_mut().enumerate() {
            let noise = (splitmix(self.seed ^ (i * 131) ^ (k as u64) << 20) % 1000) as f32 / 1000.0
                * 0.06
                - 0.03;
            *tot += reference * self.rival_skill[k] * (1.0 + noise);
        }
        let mine = self.total();
        self.position = 1 + self.rival_total.iter().filter(|&&r| r < mine).count();
        self.phase = Phase::StageEnd;
    }

    /// The host books the coins of the stage that just ended.
    pub fn add_coins(&mut self, c: u32) {
        self.coins_stage = c;
        self.coins_total = self.coins_total.saturating_add(c).min(9_999_999);
    }

    /// The result of the stage that just ended, once (for the XP booking of the front-end).
    pub fn take_result(&mut self) -> Option<GameResult> {
        self.pending.take()
    }

    pub fn in_race(&self) -> bool {
        self.phase == Phase::Racing
    }

    pub fn audio_params(&self) -> crate::audio::AudioParams {
        match (&self.game, self.phase) {
            (Some(g), Phase::Racing) => g.audio_params(),
            _ => Default::default(),
        }
    }

    pub fn score(&self) -> u32 {
        (self.results.len() * 100) as u32
    }

    fn apply_service(&mut self) {
        let s = self.svc;
        self.service_time += self.service_cost(s);
        let missing_b = 1.0 - self.body;
        let missing_e = 1.0 - self.engine;
        match s.repair {
            1 => {
                self.body += missing_b * 0.5;
                self.engine += missing_e * 0.5;
            }
            2 => {
                self.body = 1.0;
                self.engine = 1.0;
            }
            _ => {}
        }
        let want = self.next_tyre(s);
        if want != self.tyre || self.tyre_wear < 0.6 {
            self.tyre = want;
            self.tyre_wear = 1.0;
        }
        if s.refuel {
            self.fuel = 1.0;
        }
    }

    pub fn update(&mut self, dt: f32, inp: &Input, enter: bool, _back: bool) {
        let dt = if dt.is_finite() {
            dt.clamp(0.0, 0.05)
        } else {
            0.0
        };
        self.t += dt;
        let press = |now: bool, before: bool| now && !before;
        let (l, r, u, d) = (
            press(inp.left, self.prev.left),
            press(inp.right, self.prev.right),
            press(inp.accel, self.prev.accel),
            press(inp.brake, self.prev.brake),
        );
        self.prev = *inp;
        match self.phase {
            Phase::Briefing => {
                if enter {
                    self.start_stage();
                }
            }
            Phase::Racing => {
                if let Some(g) = self.game.as_mut() {
                    g.update(dt, inp);
                    let done = g.state != State::Playing && g.elapsed > 0.5;
                    // a finished or failed stage waits for Enter before the summary
                    if done && enter {
                        self.end_stage();
                    }
                }
            }
            Phase::StageEnd => {
                if enter {
                    if self.stage + 1 >= self.n {
                        self.phase = Phase::Final;
                    } else {
                        self.svc = Service::default();
                        self.cursor = 0;
                        self.phase = Phase::Service;
                    }
                }
            }
            Phase::Service => {
                if u {
                    self.cursor = (self.cursor + 2) % 3;
                }
                if d {
                    self.cursor = (self.cursor + 1) % 3;
                }
                let dir = r as i32 - l as i32;
                if dir != 0 {
                    match self.cursor {
                        0 => self.svc.repair = ((self.svc.repair as i32 + dir).rem_euclid(3)) as u8,
                        1 => {
                            self.svc.tyre = ((self.svc.tyre as i32 + dir)
                                .rem_euclid(TYRE_COUNT as i32 + 1))
                                as usize
                        }
                        _ => self.svc.refuel = !self.svc.refuel,
                    }
                }
                if enter {
                    self.apply_service();
                    self.stage += 1;
                    self.phase = Phase::Briefing;
                }
            }
            Phase::Final => {}
        }
    }

    pub fn finished(&self) -> bool {
        self.phase == Phase::Final
    }

    pub fn render(&mut self, fb: &mut [u32]) {
        if let (Some(g), Phase::Racing) = (self.game.as_ref(), self.phase) {
            render::render(g, fb);
            if g.state != State::Playing {
                txc(fb, 330, t(C::GoService), 1, 0xFFD23C);
            }
            return;
        }
        for y in 0..H {
            let k = y as f32 / H as f32;
            let c = ((16.0 + 14.0 * k) as u32) << 16
                | ((18.0 + 20.0 * k) as u32) << 8
                | (30.0 + 30.0 * k) as u32;
            fb[y * W..(y + 1) * W].fill(c);
        }
        let stage_line = t(C::StageOf)
            .replacen("{}", &(self.stage + 1).min(self.n).to_string(), 1)
            .replacen("{}", &self.n.to_string(), 1);
        txc(fb, 14, t(C::RallyT), 3, 0xFFD23C);
        if let Some(c) = self.country {
            txc(
                fb,
                46,
                crate::i18n::banner(
                    crate::champ::COUNTRIES[c.min(crate::champ::COUNTRIES.len() - 1)],
                ),
                1,
                0x8FD8FF,
            );
        }
        match self.phase {
            Phase::Briefing => {
                let p = self.plan();
                txc(fb, 70, &stage_line, 2, 0xFFFFFF);
                txc(
                    fb,
                    110,
                    &format!("{} / {}", t(WX[p.weather]), t(SF[p.surface])),
                    2,
                    0x8FD8FF,
                );
                let clock = ["", "05:30", "12:00", "18:30", "23:00"][p.time];
                txc(fb, 140, clock, 2, 0xC8D0F0);
                self.status_lines(fb, 180);
                txc(fb, 350, t(C::GoStage), 1, 0x9AA3C7);
            }
            Phase::StageEnd => {
                txc(fb, 70, &stage_line, 2, 0xFFFFFF);
                let last = self.stage_times.last().copied();
                let fin = self.results.last().is_some_and(|r| r.finished);
                if fin {
                    txc(
                        fb,
                        110,
                        &format!("{} {}", t(C::StageTime), fmt_time(last)),
                        2,
                        0x3CDC78,
                    );
                } else {
                    txc(fb, 110, t(C::Dnf), 2, 0xFF6A3C);
                }
                txc(
                    fb,
                    150,
                    &format!("{} {}", t(C::Position), self.position),
                    2,
                    0xFFD23C,
                );
                txc(
                    fb,
                    180,
                    &format!("{} {}", t(C::TotalTime), fmt_time(Some(self.total()))),
                    1,
                    0xC8D0F0,
                );
                if self.coins_stage > 0 {
                    let c = format!("+{} {}", self.coins_stage, crate::i18n::banner("COINS"));
                    txc(fb, 198, &c, 1, 0xFFD23C);
                }
                self.status_lines(fb, 220);
                txc(
                    fb,
                    350,
                    if self.stage + 1 >= self.n {
                        t(C::RallyOver)
                    } else {
                        t(C::GoService)
                    },
                    1,
                    0x9AA3C7,
                );
            }
            Phase::Service => {
                txc(fb, 60, t(C::Service), 2, 0xFFFFFF);
                let names = [t(C::RNone), t(C::RHalf), t(C::RFull)];
                let tn = [C::Auto, C::TyRoad, C::TyRain, C::TySnow, C::TyOff];
                let rows = [
                    format!("{}: {}", t(C::Repairs), names[self.svc.repair as usize]),
                    format!("{}: {}", t(C::Tyres), t(tn[self.svc.tyre.min(TYRE_COUNT)])),
                    format!(
                        "{}: {}",
                        t(C::Fuel),
                        if self.svc.refuel {
                            t(C::RFull)
                        } else {
                            t(C::RNone)
                        }
                    ),
                ];
                for (i, row) in rows.iter().enumerate() {
                    let y = 120 + i as i32 * 32;
                    fill(
                        fb,
                        120,
                        y - 6,
                        520,
                        y + 18,
                        if i == self.cursor { 0xFF6A3C } else { 0x2A3156 },
                    );
                    tx(fb, 134, y, row, 2, 0xFFFFFF);
                }
                txc(
                    fb,
                    230,
                    &format!("{}: {:.0}S", t(C::ServiceCost), self.service_cost(self.svc)),
                    2,
                    0xFFD23C,
                );
                self.status_lines(fb, 270);
                txc(fb, 350, t(C::RallyHint), 1, 0x9AA3C7);
            }
            Phase::Final => {
                txc(fb, 70, t(C::RallyOver), 3, 0xFFFFFF);
                txc(
                    fb,
                    130,
                    &format!("{} {}", t(C::Position), self.position),
                    3,
                    0xFFD23C,
                );
                txc(
                    fb,
                    180,
                    &format!("{} {}", t(C::TotalTime), fmt_time(Some(self.total()))),
                    2,
                    0xC8D0F0,
                );
                txc(
                    fb,
                    210,
                    &format!("{} {:.0}S", t(C::Penalty), self.penalty + self.service_time),
                    1,
                    0x9AA3C7,
                );
                if self.coins_total > 0 {
                    let c = format!("+{} {}", self.coins_total, crate::i18n::banner("COINS"));
                    txc(fb, 236, &c, 2, 0xFFD23C);
                }
            }
            Phase::Racing => {}
        }
    }

    fn status_lines(&self, fb: &mut [u32], y: i32) {
        let bar = |fb: &mut [u32], y: i32, label: &str, v: f32, col: u32| {
            tx(fb, 150, y, label, 1, 0xC8D0F0);
            fill(fb, 270, y, 270 + 220, y + 8, 0x1C2444);
            fill(
                fb,
                270,
                y,
                270 + (220.0 * v.clamp(0.0, 1.0)) as i32,
                y + 8,
                col,
            );
        };
        bar(fb, y, t(C::Damage), self.body.min(self.engine), 0x8FD8FF);
        bar(fb, y + 16, t(C::Fuel), self.fuel, 0x3CDC78);
        bar(fb, y + 32, t(C::Tyres), self.tyre_wear, 0xFFD23C);
    }
}
