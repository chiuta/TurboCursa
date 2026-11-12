//! Career front-end drawn straight into the framebuffer, so the native and the WASM builds share it.
//! Menus, cup brackets, result tables, rankings and the trophy cabinet; races run in a regular `Game`.
use crate::audio::AudioParams;
use crate::career::*;
use crate::cars::{Archetype, CarSpec, VehicleClass};
use crate::fractal;
use crate::game::{Difficulty, Game, Input, Mode, State};
use crate::i18n_career::{continent, division, t, tn, C};
use crate::{render, H, W};

const WHITE: u32 = 0xFFFFFF;
const GOLD: u32 = 0xFFD23C;
const GREY: u32 = 0x9AA3B8;
const GREEN: u32 = 0x6CFF8C;
const RED: u32 = 0xFF6B6B;
const CYAN: u32 = 0x7FE8FF;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Screen {
    Pick,
    Hub,
    CupInfo,
    Race,
    Result,
    Rank,
    Trophies,
    Abroad,
    Garage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RaceFor {
    League,
    Cup,
}

struct Picker {
    cont: u8,
    idx: usize,
}

pub struct CareerApp {
    pub career: Career,
    has_career: bool,
    screen: Screen,
    game: Option<Box<Game>>,
    race_for: RaceFor,
    par: f32,
    cursor: usize,
    tab: usize,
    garage: usize,
    picker: Picker,
    report: Option<Report>,
    notes: Vec<String>,
    my_time: Option<f32>,
    result_title: String,
    prev: [bool; 4],
    confirm: bool,
    toast: Option<(String, f32)>,
    end_t: f32,
    t: f32,
    seed: u64,
    dirty: bool,
}

/// Reference time of a stage: the autopilot's finish time (+0.5 s per crash, like the player).
pub fn par_time(seed: u64) -> f32 {
    let mut g = Game::new_mode(seed, Mode::Campaign);
    for _ in 0..(400 * 60) {
        let i = g.autopilot();
        g.update(1.0 / 60.0, &i);
        if g.state != State::Playing {
            break;
        }
    }
    if g.state == State::Finished {
        g.elapsed + 0.5 * g.crashes as f32
    } else {
        120.0
    }
}

pub fn fmt_time(t: Option<f32>) -> String {
    match t {
        Some(t) if t.is_finite() && t >= 0.0 => format!("{}:{:05.2}", (t / 60.0) as u32, t % 60.0),
        _ => "DNF".into(),
    }
}

pub(crate) fn tx(fb: &mut [u32], x: i32, y: i32, s: &str, sc: i32, c: u32) {
    render::overlay_text_at(fb, W, H, x, y, s, sc, c);
}
pub(crate) fn txc(fb: &mut [u32], y: i32, s: &str, sc: i32, c: u32) {
    render::overlay_text(fb, W, H, y, s, sc, c);
}
pub(crate) fn fill(fb: &mut [u32], x0: i32, y0: i32, x1: i32, y1: i32, c: u32) {
    for y in y0.max(0)..y1.min(H as i32) {
        for x in x0.max(0)..x1.min(W as i32) {
            fb[y as usize * W + x as usize] = c;
        }
    }
}

pub(crate) fn level_name(l: u8) -> &'static str {
    use C::*;
    t([Lv0, Lv1, Lv2, Lv3, Lv4][(l as usize).min(4)])
}

pub(crate) fn class_name(c: usize) -> &'static str {
    use C::*;
    const CL: [C; 23] = [
        Cl0, Cl1, Cl2, Cl3, Cl4, Cl5, Cl6, Cl7, Cl8, Cl9, Cl10, Cl11, Cl12, Cl13, Cl14, Cl15, Cl16,
        Cl17, Cl18, Cl19, Cl20, Cl21, Cl22,
    ];
    t(CL[c.min(22)])
}

impl CareerApp {
    /// `saved`: a save string from storage (None or malformed starts the country picker).
    pub fn new(seed: u64, saved: Option<&str>) -> CareerApp {
        let career = saved.and_then(Career::from_save);
        let has = career.is_some();
        let c = career.unwrap_or_else(|| Career::new(0, seed));
        CareerApp {
            picker: Picker {
                cont: c.continent(),
                idx: 0,
            },
            career: c,
            has_career: has,
            screen: if has { Screen::Hub } else { Screen::Pick },
            game: None,
            race_for: RaceFor::League,
            par: 100.0,
            cursor: 0,
            tab: 0,
            garage: 0,
            report: None,
            notes: Vec::new(),
            my_time: None,
            result_title: String::new(),
            prev: [false; 4],
            confirm: false,
            toast: None,
            end_t: 0.0,
            t: 0.0,
            seed,
            dirty: false,
        }
    }

    /// Save string if anything changed since the last call.
    pub fn take_save(&mut self) -> Option<String> {
        if self.dirty && self.has_career {
            self.dirty = false;
            Some(self.career.to_save())
        } else {
            None
        }
    }

    pub fn in_race(&self) -> bool {
        self.screen == Screen::Race && self.game.is_some()
    }

    pub fn audio_params(&self) -> AudioParams {
        match &self.game {
            Some(g) if self.screen == Screen::Race => g.audio_params(),
            _ => AudioParams::default(),
        }
    }

    pub fn score(&self) -> u32 {
        self.career.rating as u32
    }

    fn start_race(&mut self, rf: RaceFor) {
        let seed = match rf {
            RaceFor::League => self.career.league_seed(),
            RaceFor::Cup => match &self.career.cup {
                Some(c) => c.race_seed(),
                None => return,
            },
        };
        self.par = par_time(seed);
        self.race_for = rf;
        let mut g = Game::new_mode(seed, Mode::Campaign);
        g.set_car(
            CarSpec::for_country(self.career.country as usize)
                .with_class(VehicleClass::from_index(self.career.class as usize)),
        );
        g.set_level(Difficulty::from_index(self.career.level as usize));
        self.game = Some(Box::new(g));
        self.end_t = 0.0;
        self.screen = Screen::Race;
    }

    /// Cycle the difficulty (item 10) or the vehicle class (item 11).
    fn cycle(&mut self, item: usize, d: i32) {
        let (n, v) = if item == 10 {
            (crate::game::LEVEL_COUNT, &mut self.career.level)
        } else {
            (crate::cars::CLASS_COUNT, &mut self.career.class)
        };
        *v = ((*v as i32 + d).rem_euclid(n as i32)) as u8;
        self.dirty = true;
    }

    fn finish_race(&mut self, forfeit: bool) {
        let g = match self.game.take() {
            Some(g) => g,
            None => return,
        };
        let time = if !forfeit && g.state == State::Finished {
            Some(g.elapsed + 0.5 * g.crashes as f32)
        } else {
            None
        };
        self.result_title = self.race_title().0;
        let q0 = self.career.q;
        let kind = self.career.cup.as_ref().map(|c| c.kind);
        let rep = match self.race_for {
            RaceFor::League => Some(self.career.run_league(time, self.par)),
            RaceFor::Cup => self.career.run_cup(time, self.par),
        };
        self.notes.clear();
        let q1 = self.career.q;
        let names = [C::ContCup, C::InterCup, C::WorldCup];
        for i in 0..3 {
            if !q0[i] && q1[i] {
                self.notes
                    .push(format!("{} {}", t(C::Qualified), t(names[i])));
            }
        }
        if let (Some(r), Some(_)) = (&rep, kind) {
            if r.outcome == Outcome::Champion {
                self.notes.push(t(C::Champion).into());
            }
        }
        self.my_time = time;
        self.report = rep;
        self.dirty = true;
        self.screen = Screen::Result;
    }

    fn cup_for_item(&self, item: usize) -> CupKind {
        match item {
            1 => CupKind::National(self.career.country),
            2 => CupKind::Continental(self.career.continent()),
            3 => CupKind::Inter,
            _ => CupKind::World,
        }
    }

    fn toast(&mut self, s: &str) {
        self.toast = Some((s.to_string(), 3.0));
    }

    /// Advance one frame. `inp`: arrows (accel = up, brake = down); `enter` / `back` are one-shot.
    pub fn update(&mut self, dt: f32, inp: &Input, enter: bool, back: bool) {
        let dt = if dt.is_finite() {
            dt.clamp(0.0, 0.05)
        } else {
            0.0
        };
        self.t += dt;
        if let Some((_, tt)) = &mut self.toast {
            *tt -= dt;
            if *tt <= 0.0 {
                self.toast = None;
            }
        }
        let now = [inp.accel, inp.brake, inp.left, inp.right];
        let edge = |i: usize| now[i] && !self.prev[i];
        let (up, down, left, right) = (edge(0), edge(1), edge(2), edge(3));
        self.prev = now;

        match self.screen {
            Screen::Race => {
                let mut finished = false;
                if let Some(g) = &mut self.game {
                    g.update(dt, inp);
                    if g.state != State::Playing {
                        self.end_t += dt;
                        finished = enter && self.end_t > 0.8;
                    }
                }
                if finished {
                    self.finish_race(false);
                } else if back {
                    self.finish_race(true);
                }
            }
            Screen::Pick | Screen::Abroad => {
                let list = countries_of(self.picker.cont);
                if left || right {
                    let d = if right { 1 } else { NCONT as i32 - 1 };
                    self.picker.cont = ((self.picker.cont as i32 + d) % NCONT as i32) as u8;
                    self.picker.idx = 0;
                }
                let list = if left || right {
                    countries_of(self.picker.cont)
                } else {
                    list
                };
                if up {
                    self.picker.idx = (self.picker.idx + list.len() - 1) % list.len();
                }
                if down {
                    self.picker.idx = (self.picker.idx + 1) % list.len();
                }
                let country = list[self.picker.idx.min(list.len() - 1)];
                if enter {
                    if self.screen == Screen::Pick {
                        self.career = Career::new(country, self.seed);
                        self.has_career = true;
                        self.dirty = true;
                        self.screen = Screen::Hub;
                    } else {
                        match self.career.start_cup(CupKind::National(country)) {
                            Ok(()) => {
                                self.dirty = true;
                                self.screen = Screen::CupInfo;
                            }
                            Err(_) => {
                                self.toast(t(C::Resume));
                                self.screen = Screen::Hub;
                            }
                        }
                    }
                } else if back && self.screen == Screen::Abroad {
                    self.screen = Screen::Hub;
                }
            }
            Screen::Hub => {
                const N: usize = 12;
                if up {
                    self.cursor = (self.cursor + N - 1) % N;
                    self.confirm = false;
                }
                if down {
                    self.cursor = (self.cursor + 1) % N;
                    self.confirm = false;
                }
                if (left || right) && self.cursor >= 10 {
                    self.cycle(self.cursor, if right { 1 } else { -1 });
                }
                if enter {
                    match self.cursor {
                        0 => self.start_race(RaceFor::League),
                        1..=4 => {
                            if self.career.cup.is_some() {
                                self.screen = Screen::CupInfo;
                            } else {
                                let k = self.cup_for_item(self.cursor);
                                match self.career.start_cup(k) {
                                    Ok(()) => {
                                        self.dirty = true;
                                        self.screen = Screen::CupInfo;
                                    }
                                    Err(_) => self.toast(t(C::NeedFinal)),
                                }
                            }
                        }
                        5 => {
                            if self.career.cup.is_some() {
                                self.screen = Screen::CupInfo;
                            } else {
                                self.picker = Picker {
                                    cont: self.career.continent(),
                                    idx: 0,
                                };
                                self.screen = Screen::Abroad;
                            }
                        }
                        6 => {
                            self.tab = 0;
                            self.screen = Screen::Rank;
                        }
                        7 => {
                            self.garage = self.career.country as usize;
                            self.screen = Screen::Garage;
                        }
                        8 => self.screen = Screen::Trophies,
                        10 | 11 => self.cycle(self.cursor, 1),
                        _ => {
                            if self.confirm {
                                self.confirm = false;
                                self.has_career = false;
                                self.screen = Screen::Pick;
                            } else {
                                self.confirm = true;
                            }
                        }
                    }
                }
            }
            Screen::CupInfo => {
                if enter {
                    self.confirm = false;
                    if self.career.cup.is_some() {
                        self.start_race(RaceFor::Cup);
                    } else {
                        self.screen = Screen::Hub;
                    }
                } else if down {
                    if self.confirm {
                        self.career.abandon_cup();
                        self.dirty = true;
                        self.confirm = false;
                        self.screen = Screen::Hub;
                    } else {
                        self.confirm = true;
                    }
                } else if back || up {
                    self.confirm = false;
                    self.screen = Screen::Hub;
                }
            }
            Screen::Result => {
                if enter || back {
                    self.report = None;
                    self.screen = if self.race_for == RaceFor::Cup && self.career.cup.is_some() {
                        Screen::CupInfo
                    } else {
                        Screen::Hub
                    };
                }
            }
            Screen::Rank => {
                if left {
                    self.tab = (self.tab + 3) % 4;
                }
                if right {
                    self.tab = (self.tab + 1) % 4;
                }
                if enter || back {
                    self.screen = Screen::Hub;
                }
            }
            Screen::Trophies => {
                if enter || back {
                    self.screen = Screen::Hub;
                }
            }
            Screen::Garage => {
                let n = COUNTRIES.len();
                if right {
                    self.garage = (self.garage + 1) % n;
                }
                if left {
                    self.garage = (self.garage + n - 1) % n;
                }
                if down {
                    self.cycle(11, 1);
                }
                if up {
                    self.cycle(11, -1);
                }
                if enter || back {
                    self.screen = Screen::Hub;
                }
            }
        }
    }

    pub fn render(&mut self, fb: &mut [u32]) {
        if fb.len() != W * H {
            return;
        }
        if self.screen == Screen::Race {
            if let Some(g) = &self.game {
                render::render(g, fb);
                let (name, sub) = self.race_title();
                tx(fb, 8, 40, &name, 1, WHITE);
                tx(fb, 8, 54, &sub, 1, GREY);
                if g.state != State::Playing && self.end_t > 0.8 {
                    txc(fb, 300, t(C::Continue), 1, WHITE);
                }
                return;
            }
        }
        self.backdrop(fb);
        match self.screen {
            Screen::Pick => self.draw_picker(fb, t(C::ChooseCountry)),
            Screen::Abroad => self.draw_picker(fb, t(C::Abroad)),
            Screen::Hub => self.draw_hub(fb),
            Screen::CupInfo => self.draw_cup(fb),
            Screen::Result => self.draw_result(fb),
            Screen::Rank => self.draw_rank(fb),
            Screen::Trophies => self.draw_trophies(fb),
            Screen::Garage => self.draw_garage(fb),
            Screen::Race => {}
        }
        if let Some((s, _)) = &self.toast {
            fill(fb, 40, 360, W as i32 - 40, 384, 0x401020);
            txc(fb, 366, s, 1, RED);
        }
    }

    fn race_title(&self) -> (String, String) {
        match (self.race_for, &self.career.cup) {
            (RaceFor::Cup, Some(c)) => (cup_name(c.kind), round_name(c)),
            _ => (
                t(C::League).to_string(),
                format!("{} {}/{}", t(C::Stage), self.career.day + 1, LEAGUE_DAYS),
            ),
        }
    }

    fn backdrop(&self, fb: &mut [u32]) {
        for y in 0..H {
            let f = y as f32 / H as f32;
            let c = ((8.0 + 30.0 * f) as u32) << 16
                | ((10.0 + 24.0 * f) as u32) << 8
                | (34.0 + 50.0 * f) as u32;
            fill(fb, 0, y as i32, W as i32, y as i32 + 1, c);
        }
        for x in 0..W {
            let h = 150.0 + 70.0 * fractal::fbm(x as f32 * 0.012 + self.t * 0.02, 3.0, 7, 4, 0);
            fill(fb, x as i32, h as i32, x as i32 + 1, H as i32, 0x0C1030);
        }
        for i in 0..90u32 {
            let h = crate::fx::hash(i.wrapping_mul(7919)) as usize;
            let (x, y) = (h % W, (h >> 10) % 140);
            if (self.t * 2.0 + i as f32).sin() > -0.6 {
                fb[y * W + x] = 0xBFD0FF;
            }
        }
    }

    fn header(&self, fb: &mut [u32], title: &str) {
        txc(fb, 8, title, 2, GOLD);
        let c = &self.career;
        let line = format!(
            "{} {}  {} {}  {} {}  {}",
            t(C::Season),
            c.season,
            t(C::Division),
            division(c.div),
            t(C::Rating),
            c.rating as i32,
            COUNTRIES[c.country as usize].name
        );
        txc(fb, 34, &line, 1, CYAN);
    }

    fn draw_picker(&self, fb: &mut [u32], title: &str) {
        txc(fb, 10, title, 2, GOLD);
        let cont = self.picker.cont;
        txc(fb, 40, &format!("< {} >", continent(cont)), 1, WHITE);
        let list = countries_of(cont);
        let idx = self.picker.idx.min(list.len() - 1);
        let first = idx.saturating_sub(7).min(list.len().saturating_sub(15));
        for (row, &ci) in list.iter().enumerate().skip(first).take(15) {
            let y = 66 + (row - first) as i32 * 16;
            let sel = row == idx;
            if sel {
                fill(fb, 180, y - 3, 460, y + 13, 0x203060);
            }
            let c = &COUNTRIES[ci as usize];
            let won = self.career.trophies.national[ci as usize];
            let s = format!("{}  {}{}", c.code, c.name, if won > 0 { " *" } else { "" });
            tx(fb, 190, y, &s, 1, if sel { WHITE } else { GREY });
        }
        txc(fb, 372, t(C::Hint), 1, GREY);
    }

    fn hub_label(&self, i: usize) -> (String, bool) {
        let c = &self.career;
        let busy = c.cup.is_some();
        let cup_line = |k: CupKind, name: String| -> (String, bool) {
            match c.cup.as_ref().map(|x| x.kind) {
                Some(ck) if ck == k => (format!("{} - {}", t(C::Resume), name), true),
                _ => {
                    let ok = c.can_start(k).is_ok();
                    (
                        if ok {
                            name
                        } else {
                            format!("{} [{}]", name, t(C::Locked))
                        },
                        ok && !busy || ok,
                    )
                }
            }
        };
        match i {
            0 => (
                format!(
                    "{}  {} {}/{}",
                    t(C::League),
                    t(C::Stage),
                    c.day + 1,
                    LEAGUE_DAYS
                ),
                true,
            ),
            1 => cup_line(
                CupKind::National(c.country),
                cup_name(CupKind::National(c.country)),
            ),
            2 => cup_line(
                CupKind::Continental(c.continent()),
                cup_name(CupKind::Continental(c.continent())),
            ),
            3 => cup_line(CupKind::Inter, cup_name(CupKind::Inter)),
            4 => cup_line(CupKind::World, cup_name(CupKind::World)),
            5 => (t(C::Abroad).into(), true),
            6 => (t(C::Rankings).into(), true),
            7 => (t(C::Garage).into(), true),
            8 => (t(C::Trophies).into(), true),
            10 => (format!("{}: {}", t(C::Level), level_name(c.level)), true),
            11 => (
                format!("{}: {}", t(C::Vehicle), class_name(c.class as usize)),
                true,
            ),
            _ => (
                if self.confirm {
                    t(C::Confirm).into()
                } else {
                    t(C::NewCareer).into()
                },
                true,
            ),
        }
    }

    fn draw_hub(&self, fb: &mut [u32]) {
        self.header(fb, t(C::Career));
        for i in 0..12 {
            let y = 66 + i as i32 * 22;
            let (label, ok) = self.hub_label(i);
            let sel = i == self.cursor;
            if sel {
                fill(fb, 90, y - 6, W as i32 - 90, y + 18, 0x24366E);
            }
            let col = if !ok {
                0x5A6078
            } else if sel {
                WHITE
            } else {
                GREY
            };
            tx(fb, 104, y, &label, 1, col);
        }
        let s = &self.career.stats;
        let line = format!(
            "{} {}  {} {}  {} {}",
            t(C::Races),
            s.races,
            t(C::Wins),
            s.wins,
            t(C::Podiums),
            s.podiums
        );
        txc(fb, 340, &line, 1, GREY);
        txc(fb, 372, t(C::Hint), 1, GREY);
    }

    fn draw_cup(&self, fb: &mut [u32]) {
        let c = match &self.career.cup {
            Some(c) => c,
            None => return,
        };
        self.header(fb, &cup_name(c.kind));
        let plan = c.kind.plan();
        // bracket: one box per round
        let bw = 96;
        let x0 = (W as i32 - plan.len() as i32 * (bw + 8)) / 2;
        for (i, (n, adv)) in plan.iter().enumerate() {
            let x = x0 + i as i32 * (bw + 8);
            let cur = i == c.round && matches!(c.phase, Phase::Round | Phase::Repechage);
            let done = i < c.round;
            let col = if cur {
                0x3A5CC8
            } else if done {
                0x1F6A3A
            } else {
                0x1C2444
            };
            fill(fb, x, 70, x + bw, 130, col);
            let label = if *adv == 1 {
                t(C::Final).to_string()
            } else {
                tn(C::RoundOf, *n)
            };
            tx(fb, x + 4, 76, &label, 1, WHITE);
            tx(fb, x + 4, 94, &tn(C::Entrants, *n), 1, GREY);
            if *adv > 1 {
                tx(fb, x + 4, 110, &format!("TOP {}", adv), 1, GREY);
            }
        }
        let mut y = 160;
        txc(fb, y, &round_name(c), 2, WHITE);
        y += 40;
        let ent = c.entrants();
        txc(fb, y, &format!("{}: {}", t(C::Lives), c.lives), 1, GOLD);
        y += 22;
        if c.phase == Phase::Repechage {
            txc(
                fb,
                y,
                &format!("{}: {}", t(C::Repechage), tn(C::TopAdvance, 2)),
                1,
                CYAN,
            );
        } else if let Some((_, adv)) = plan.get(c.round) {
            txc(fb, y, &tn(C::TopAdvance, *adv), 1, CYAN);
        }
        y += 22;
        txc(fb, y, &tn(C::Entrants, ent.len()), 1, GREY);
        y += 22;
        let names: Vec<String> = ent
            .iter()
            .filter(|&&i| i != 0)
            .take(6)
            .map(|&i| c.rival(i).name)
            .collect();
        txc(fb, y, &names.join("  "), 1, GREY);
        txc(fb, 330, t(C::Race), 2, GREEN);
        if self.confirm {
            txc(fb, 356, t(C::Confirm), 1, RED);
        } else {
            txc(fb, 356, t(C::Abandon), 1, GREY);
        }
        txc(fb, 372, t(C::Back), 1, GREY);
    }

    fn draw_result(&self, fb: &mut [u32]) {
        let rep = match &self.report {
            Some(r) => r,
            None => return,
        };
        let title = self.result_title.clone();
        self.header(fb, &title);
        let (msg, col) = match rep.outcome {
            Outcome::Advanced => (t(C::Advance), GREEN),
            Outcome::Eliminated { chance } => {
                if chance {
                    (t(C::Repechage), CYAN)
                } else {
                    (t(C::Eliminated), RED)
                }
            }
            Outcome::Revived => (t(C::Revived), GREEN),
            Outcome::Champion => (t(C::Champion), GOLD),
            Outcome::FinalPlace => (t(C::FinalLost), RED),
        };
        let msg = if self.race_for == RaceFor::League {
            ""
        } else {
            msg
        };
        txc(
            fb,
            54,
            &format!(
                "{}  {} {}  {}",
                msg,
                t(C::Pos),
                rep.player_pos,
                fmt_time(self.my_time)
            ),
            1,
            col,
        );
        let n = rep.table.len();
        let show: Vec<usize> = if n <= 16 {
            (0..n).collect()
        } else {
            let mut v: Vec<usize> = (0..12).collect();
            let p = rep.player_pos.saturating_sub(1);
            if p >= 12 {
                v.push(p);
            }
            v
        };
        let mut y = 76;
        for &i in &show {
            let r = &rep.table[i];
            let name = if r.id == 0 {
                t(C::You).to_string()
            } else {
                r.name.clone()
            };
            let cc = COUNTRIES[(if r.id == 0 {
                self.career.country
            } else {
                r.country
            }) as usize]
                .code;
            let line = format!("{:>2}  {:<14} {}  {}", i + 1, name, cc, fmt_time(r.time));
            if r.id == 0 {
                fill(fb, 150, y - 2, 490, y + 13, 0x3A3A10);
            }
            tx(fb, 160, y, &line, 1, if r.id == 0 { GOLD } else { WHITE });
            y += 15;
        }
        for (k, note) in self.notes.iter().enumerate() {
            txc(fb, 310 + k as i32 * 14, note, 1, GOLD);
        }
        if let Some((o, nw, pl)) = rep.season_end {
            let m = if nw > o {
                t(C::Promoted)
            } else if nw < o {
                t(C::Relegated)
            } else {
                ""
            };
            txc(
                fb,
                334,
                &format!(
                    "{} {}  {} {}  {} {}",
                    t(C::SeasonOver),
                    "",
                    t(C::Pos),
                    pl,
                    division(nw),
                    m
                ),
                1,
                CYAN,
            );
        }
        txc(fb, 372, t(C::Continue), 1, GREY);
    }

    fn draw_rank(&self, fb: &mut [u32]) {
        let tabs = [C::Drivers, C::Nations, C::League, C::Rating];
        self.header(fb, t(C::Rankings));
        let mut x = 80;
        for (i, k) in tabs.iter().enumerate() {
            let s = t(*k);
            if i == self.tab {
                fill(
                    fb,
                    x - 6,
                    54,
                    x + s.chars().count() as i32 * 8 + 6,
                    70,
                    0x24366E,
                );
            }
            tx(fb, x, 58, s, 1, if i == self.tab { WHITE } else { GREY });
            x += s.chars().count() as i32 * 8 + 24;
        }
        let mut y = 80;
        let c = &self.career;
        match self.tab {
            0 => {
                let v = c.drivers_ranking();
                let me = v.iter().position(|d| d.3).unwrap_or(0);
                for (i, d) in v.iter().enumerate().take(14) {
                    self.rank_row(
                        fb,
                        &mut y,
                        i + 1,
                        &d.0,
                        COUNTRIES[d.1 as usize].code,
                        d.2,
                        d.3,
                    );
                }
                if me >= 14 {
                    let d = &v[me];
                    y += 6;
                    self.rank_row(
                        fb,
                        &mut y,
                        me + 1,
                        &d.0,
                        COUNTRIES[d.1 as usize].code,
                        d.2,
                        true,
                    );
                }
            }
            1 => {
                let v = c.nations_ranking();
                let me = v.iter().position(|d| d.0 == c.country).unwrap_or(0);
                for (i, d) in v.iter().enumerate().take(14) {
                    let n = &COUNTRIES[d.0 as usize];
                    self.rank_row(fb, &mut y, i + 1, n.name, n.code, d.1, d.0 == c.country);
                }
                if me >= 14 {
                    let d = v[me];
                    y += 6;
                    let n = &COUNTRIES[d.0 as usize];
                    self.rank_row(fb, &mut y, me + 1, n.name, n.code, d.1, true);
                }
            }
            2 => {
                for (i, d) in c.league_table().iter().enumerate() {
                    let n = if d.3 {
                        t(C::You).to_string()
                    } else {
                        d.0.clone()
                    };
                    self.rank_row(
                        fb,
                        &mut y,
                        i + 1,
                        &n,
                        COUNTRIES[d.1 as usize].code,
                        d.2 as i32,
                        d.3,
                    );
                }
            }
            _ => {
                let s = &c.stats;
                for line in [
                    format!("{}: {}", t(C::Rating), c.rating as i32),
                    format!("{}: {}", t(C::Races), s.races),
                    format!("{}: {}", t(C::Wins), s.wins),
                    format!("{}: {}", t(C::Podiums), s.podiums),
                    format!("{}: {}", t(C::Champions), c.trophies.league),
                ] {
                    tx(fb, 200, y, &line, 1, WHITE);
                    y += 20;
                }
            }
        }
        txc(fb, 372, t(C::Hint), 1, GREY);
    }

    #[allow(clippy::too_many_arguments)]
    fn rank_row(
        &self,
        fb: &mut [u32],
        y: &mut i32,
        pos: usize,
        name: &str,
        cc: &str,
        val: i32,
        me: bool,
    ) {
        if me {
            fill(fb, 130, *y - 2, 510, *y + 13, 0x3A3A10);
        }
        let n: String = name.chars().take(18).collect();
        tx(
            fb,
            140,
            *y,
            &format!("{:>3}  {:<18} {}  {:>5}", pos, n, cc, val),
            1,
            if me { GOLD } else { WHITE },
        );
        *y += 16;
    }

    fn draw_garage(&self, fb: &mut [u32]) {
        self.header(fb, t(C::Garage));
        let car = CarSpec::for_country(self.garage)
            .with_class(VehicleClass::from_index(self.career.class as usize));
        let c = &COUNTRIES[self.garage.min(COUNTRIES.len() - 1)];
        let mine = self.garage == self.career.country as usize;
        txc(
            fb,
            54,
            &format!("< {} {} >", c.code, c.name),
            1,
            if mine { GOLD } else { WHITE },
        );
        fill(fb, 150, 80, 490, 250, 0x10163A);
        fill(fb, 150, 238, 490, 250, 0x1A1F44);
        render::draw_car_preview(fb, W, H, 320.0, 236.0, 220.0, &car);
        let arch = match car.archetype {
            Archetype::Balanced => C::ArBalanced,
            Archetype::TopSpeed => C::ArTop,
            Archetype::Dragster => C::ArDrag,
            Archetype::Grip => C::ArGrip,
            Archetype::Turbo => C::ArTurbo,
        };
        txc(fb, 262, &format!("{}  {}", car.model(), t(arch)), 1, CYAN);
        txc(fb, 76, class_name(car.class.index()), 1, WHITE);
        let bars = [
            (C::TopSpeed, car.top, 0.08),
            (C::Accel, car.accel, 0.12),
            (C::Grip, car.grip, 0.10),
        ];
        for (i, (k, v, span)) in bars.iter().enumerate() {
            let y = 286 + i as i32 * 18;
            tx(fb, 170, y, t(*k), 1, GREY);
            let f = (0.5 + (v - 1.0) / (2.0 * span)).clamp(0.05, 1.0);
            fill(fb, 330, y, 470, y + 9, 0x1C2444);
            fill(fb, 330, y, 330 + (140.0 * f) as i32, y + 9, GREEN);
        }
        let y = 340;
        tx(fb, 170, y, t(C::Turbos), 1, GREY);
        for i in 0..car.turbos as i32 {
            fill(fb, 330 + i * 22, y, 346 + i * 22, y + 9, 0xFF961E);
        }
        txc(fb, 372, t(C::Hint), 1, GREY);
    }

    fn draw_trophies(&self, fb: &mut [u32]) {
        self.header(fb, t(C::Trophies));
        let tr = &self.career.trophies;
        let nat = tr.national.iter().filter(|&&x| x > 0).count();
        txc(
            fb,
            56,
            &format!(
                "{} {}/{}   {} {}   {} {}   {} {}",
                t(C::NatCup),
                nat,
                COUNTRIES.len(),
                t(C::InterCup),
                tr.inter,
                t(C::WorldCup),
                tr.world,
                t(C::League),
                tr.league
            ),
            1,
            GOLD,
        );
        let conts: Vec<String> = (0..NCONT)
            .map(|i| format!("{} {}", continent(i as u8), tr.cont[i]))
            .collect();
        txc(fb, 72, &conts[..3].join("   "), 1, CYAN);
        txc(fb, 86, &conts[3..].join("   "), 1, CYAN);
        let cols = 21;
        for (i, c) in COUNTRIES.iter().enumerate() {
            let (cx, cy) = (22 + (i % cols) as i32 * 29, 110 + (i / cols) as i32 * 21);
            let won = tr.national[i] > 0;
            fill(
                fb,
                cx - 2,
                cy - 3,
                cx + 25,
                cy + 15,
                if won { 0x7A5A00 } else { 0x141A38 },
            );
            tx(
                fb,
                cx + 1,
                cy + 1,
                c.code,
                1,
                if won { GOLD } else { 0x5A6078 },
            );
        }
        txc(fb, 372, t(C::Hint), 1, GREY);
    }
}

pub fn cup_name(k: CupKind) -> String {
    match k {
        CupKind::National(c) => format!(
            "{} - {}",
            t(C::NatCup),
            COUNTRIES[c as usize % COUNTRIES.len()].name
        ),
        CupKind::Continental(c) => format!("{} - {}", t(C::ContCup), continent(c)),
        CupKind::Inter => t(C::InterCup).into(),
        CupKind::World => t(C::WorldCup).into(),
    }
}

pub fn round_name(c: &Cup) -> String {
    let plan = c.kind.plan();
    let (n, adv) = plan[c.round.min(plan.len() - 1)];
    let base = if adv == 1 {
        t(C::Final).to_string()
    } else {
        tn(C::RoundOf, n)
    };
    if c.phase == Phase::Repechage {
        format!("{} ({})", t(C::Repechage), base)
    } else {
        base
    }
}
