//! The demo pilot: drives each stage as fast as the rules allow.
//!
//! Every few frames it clones the game, plays a handful of candidate plans (a lane to hold, with or
//! without yielding to traffic) a couple of seconds into the future with the real simulation, scores
//! them (distance gained, crashes, disqualification, time-up) and follows the best one. The city
//! rules (limits, lights, pedestrians), fuel stops and jam exits come from [`Game::drive`].
use crate::game::{Game, Input, Mode, State};

/// seconds after which an Endless demo stage is considered done even when the lap is not
pub const ENDLESS_CAP: f32 = 150.0;
const COLLECT: f32 = 12.0;
const DT: f32 = 1.0 / 60.0;
/// frames between two plans
const REPLAN: u32 = 4;
/// frames each candidate is simulated
const HORIZON: u32 = 110;
const LANES: [f32; 5] = [-0.75, -0.4, 0.0, 0.4, 0.75];
/// frames of braking at the start of a plan (0 = flat out)
const BRAKES: [u32; 3] = [0, 18, 40];

#[derive(Clone, Copy, PartialEq, Debug)]
struct Plan {
    lane: f32,
    yield_traffic: bool,
    brake: u32,
}

pub struct Ace {
    plan: Plan,
    left: u32,
    age: u32,
}

impl Default for Ace {
    fn default() -> Self {
        Self::new()
    }
}

impl Ace {
    pub fn new() -> Ace {
        Ace {
            plan: Plan {
                lane: 0.0,
                yield_traffic: true,
                brake: 0,
            },
            left: 0,
            age: 0,
        }
    }

    /// The input for this frame.
    pub fn input(&mut self, g: &Game) -> Input {
        // a jam ahead: leave by the slip road, straight to the outer lane (the planner would shy away from it)
        let seg = ((g.pos + crate::game::PLAYER_Z) / crate::track::SEG_LEN) as usize;
        if g.detour.is_none() && !g.demo {
            if let Some((_, j)) = g.city.next_jam(seg) {
                if seg + 110 >= j.exit_a() && seg < j.a {
                    self.left = 0;
                    let lane = if seg + 12 >= j.exit_a() { 1.45 } else { 0.9 };
                    let mut i = g.drive(Some(lane), true, true);
                    i.turbo = false;
                    if g.speed < 1500.0 {
                        // never wait for a car standing in the slip road
                        i.brake = false;
                        i.accel = true;
                    }
                    return i;
                }
            }
        }
        if self.left == 0 {
            self.plan = Self::best(g, self.plan, self.age);
            self.left = REPLAN;
            self.age = 0;
        }
        self.left -= 1;
        let i = Self::apply(g, self.plan, self.age);
        self.age += 1;
        i
    }

    /// The plan's input `age` frames after it started.
    fn apply(g: &Game, p: Plan, age: u32) -> Input {
        let mut i = g.drive(Some(p.lane), p.yield_traffic, true);
        if age < p.brake && g.speed > 1500.0 && !g.ace_refuelling() {
            i.accel = false;
            i.brake = true;
            i.turbo = false;
        }
        i
    }

    fn best(g: &Game, cur: Plan, age: u32) -> Plan {
        let mut best = (f32::NEG_INFINITY, cur);
        // yielding only matters with a car close ahead: otherwise both variants play out the same
        let pz = g.pos + crate::game::PLAYER_Z;
        let len = g.track.length;
        let traffic = g.cars.iter().any(|c| (c.z - pz).rem_euclid(len) < 4000.0);
        let yields: &[bool] = if traffic { &[false, true] } else { &[false] };
        for &lane in &LANES {
            for &yield_traffic in yields {
                for brake in BRAKES {
                    let p = Plan {
                        lane,
                        yield_traffic,
                        brake,
                    };
                    let mut sc = Self::score(g, p) - p.brake as f32 * 8.0; // braking only when it pays
                                                                           // hysteresis: do not dither between equal plans
                    if p.lane == cur.lane
                        && p.yield_traffic == cur.yield_traffic
                        && (p.brake == 0) == (age >= cur.brake)
                    {
                        sc += 400.0;
                    }
                    if sc > best.0 {
                        best = (sc, p);
                    }
                }
            }
        }
        best.1
    }

    fn score(g: &Game, p: Plan) -> f32 {
        let mut s = g.clone();
        let crashes0 = s.crashes;
        let mut frames = 0;
        let mut worst = 0.0f32;
        for k in 0..HORIZON {
            let i = Self::apply(&s, p, k);
            s.update(DT, &i);
            worst = worst.max(s.x.abs());
            frames += 1;
            if s.state != State::Playing {
                break;
            }
        }
        let mut sc = s.pos - g.pos - 40000.0 * (s.crashes - crashes0) as f32;
        match s.state {
            State::Playing => {}
            State::Finished => sc += 1.0e6 + (HORIZON - frames) as f32 * 1000.0,
            _ => sc -= 1.0e7,
        }
        sc += s.speed * 2.0;
        if matches!(g.mode, Mode::Collector | Mode::Slalom) {
            // pickups and gates add clock in these modes
            sc += (s.score as f32 - g.score as f32) * COLLECT;
        }
        let seg = ((g.pos + crate::game::PLAYER_Z) / crate::track::SEG_LEN) as usize;
        let exiting = g
            .city
            .next_jam(seg)
            .is_some_and(|(_, j)| seg + 140 > j.exit_a() && seg < j.a + 4);
        if !exiting {
            sc -= (worst - 1.0).max(0.0) * 40000.0;
        }
        sc
    }
}

/// Demo walkthrough: every finite mode on every difficulty, one stage after the other, driven by [`Ace`].
pub struct Demo {
    list: Vec<crate::share::StageCode>,
    pub idx: usize,
    ace: Ace,
    wait: f32,
    lap0: Option<u32>,
}

impl Default for Demo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo {
    /// Modes 0..=7 (Endless is one lap, Zen has no clock) x levels 0..=4; a class with a roomier clock on the
    /// two hardest levels.
    pub fn list() -> Vec<crate::share::StageCode> {
        let mut v = Vec::new();
        for level in 0..5u8 {
            for mode in 0..8u8 {
                let class = 0;
                v.push(crate::share::StageCode {
                    seed: 1 + mode as u32 * 7 + level as u32,
                    mode,
                    class,
                    level,
                    ..Default::default()
                });
            }
        }
        v
    }
    pub fn new() -> Demo {
        Demo {
            list: Self::list(),
            idx: 0,
            ace: Ace::new(),
            wait: 0.0,
            lap0: None,
        }
    }
    pub fn total(&self) -> usize {
        self.list.len()
    }
    /// The stage to start with (and after it, the next ones).
    pub fn current(&self) -> Game {
        let mut g = self.list[self.idx % self.list.len()].game();
        g.demo = true;
        g
    }
    /// Drive one frame; returns the input and, when a finished stage has been shown long enough, the next game.
    pub fn step(&mut self, g: &Game, dt: f32) -> (Input, Option<Game>) {
        let playing = g.state == State::Playing;
        // an endless stage is one lap
        let lap0 = *self.lap0.get_or_insert(g.lap);
        let lap_done = g.mode == Mode::Endless && (g.lap > lap0 || g.elapsed > ENDLESS_CAP);
        if playing && !lap_done {
            self.wait = 0.0;
            return (self.ace.input(g), None);
        }
        self.wait += dt;
        if self.wait > 2.5 {
            self.wait = 0.0;
            self.idx += 1;
            self.ace = Ace::new();
            self.lap0 = None;
            return (Input::default(), Some(self.current()));
        }
        (Input::default(), None)
    }
}
