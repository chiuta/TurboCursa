use crate::city;
use crate::fractal::{fbm, Assets};
use crate::rng::Rng;
use crate::track::{
    self, feat_limit, Track, F_BRIDGE, F_NONE, F_PASS, F_TUNNEL, F_VIADUCT, SEG_LEN,
};
use crate::{H, W};
use std::sync::Arc;

pub const MAX_SPEED: f32 = SEG_LEN * 60.0;
pub const ACCEL: f32 = MAX_SPEED / 5.0;
pub const BRAKE: f32 = MAX_SPEED;
pub const DECEL: f32 = MAX_SPEED / 5.0;
pub const OFFROAD_DECEL: f32 = MAX_SPEED / 2.0;
pub const OFFROAD_LIMIT: f32 = MAX_SPEED / 4.0;
pub const CENTRIFUGAL: f32 = 0.3;
pub const CAM_H: f32 = 1000.0;
pub const CAM_DEPTH: f32 = 0.84;
pub const PLAYER_Z: f32 = CAM_H * CAM_DEPTH;
pub const CAR_W: f32 = 520.0;
pub const START_TIME: f32 = 38.0;
pub const CP_BONUS: f32 = 30.0;
const LANES: [f32; 3] = [-0.66, 0.0, 0.66];

/// Game modes: classic ones and some unusual ones. (Two-player and versus-CPU live in `versus.rs`.)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// beat the clock through the four landscapes of a stage
    Campaign,
    /// no traffic, the clock counts up: finish as fast as you can
    TimeTrial,
    /// laps forever: every lap is a brand-new generated stage
    Endless,
    /// triple traffic; every overtake buys time
    Rush,
    /// flowing fractal trails of orbs to collect for time
    Collector,
    /// rings to thread, combos multiply the score
    Slalom,
    /// steering flips to its mirror image in periodic bursts
    Mirror,
    /// darkness: only the headlight cone is visible
    Blackout,
    /// no clock, no traffic, no crashes: just drive through endless landscapes
    Zen,
}

pub const MODE_COUNT: usize = 9;

impl Mode {
    pub fn from_index(i: u32) -> Mode {
        [
            Mode::Campaign,
            Mode::TimeTrial,
            Mode::Endless,
            Mode::Rush,
            Mode::Collector,
            Mode::Slalom,
            Mode::Mirror,
            Mode::Blackout,
            Mode::Zen,
        ][(i as usize).min(MODE_COUNT - 1)]
    }
    pub fn index(self) -> u32 {
        self as u32
    }
    /// Accepts an index ("3") or a name ("rush").
    pub fn parse(s: &str) -> Option<Mode> {
        let t = s.trim().to_ascii_lowercase();
        if let Ok(i) = t.parse::<u32>() {
            return (i as usize).lt(&MODE_COUNT).then(|| Mode::from_index(i));
        }
        let names = [
            "campaign",
            "timetrial",
            "endless",
            "rush",
            "collector",
            "slalom",
            "mirror",
            "blackout",
            "zen",
        ];
        names
            .iter()
            .position(|n| *n == t.replace(['-', '_', ' '], ""))
            .map(|i| Mode::from_index(i as u32))
    }
}

/// A collectable orb (kind 0) or a slalom ring (kind 1) lying on the road.
#[derive(Clone, Copy, Debug)]
pub struct Orb {
    pub z: f32,
    pub x: f32,
    pub kind: u8,
    pub taken: bool,
}

/// Roadside items: obstacles (0..=5) and bonuses (6..=12) lying on the road.
pub const I_CONE: u8 = 0;
pub const I_OIL: u8 = 1;
pub const I_BARRIER: u8 = 2;
pub const I_POTHOLE: u8 = 3;
pub const I_PUDDLE: u8 = 4;
pub const I_MUD: u8 = 5;
pub const I_TIME: u8 = 6;
pub const I_TURBO: u8 = 7;
pub const I_BOOST: u8 = 8;
pub const I_SHIELD: u8 = 9;
pub const I_COIN: u8 = 10;
pub const I_FREEZE: u8 = 11;
pub const I_DOUBLE: u8 = 12;
pub const I_CRACK: u8 = 13;
pub const I_MAGMA: u8 = 14;
pub const I_RAMP_S: u8 = 15;
pub const I_RAMP_M: u8 = 16;
pub const I_RAMP_L: u8 = 17;
pub const I_PAD: u8 = 18;
pub const ITEM_KINDS: u8 = 19;

pub fn is_ramp(k: u8) -> bool {
    (I_RAMP_S..=I_RAMP_L).contains(&k)
}

/// Wild animals that cross the road: (speed in road half-widths per second, half hit width).
pub const SPECIES_COUNT: u8 = 8;
pub const SPECIES_NAMES: [&str; 8] = [
    "deer", "moose", "fox", "bear", "camel", "penguin", "boar", "kangaroo",
];
pub fn species_params(s: u8) -> (f32, f32) {
    match s % SPECIES_COUNT {
        0 => (1.0, 0.11),
        1 => (0.45, 0.20),
        2 => (1.4, 0.07),
        3 => (0.55, 0.17),
        4 => (0.5, 0.17),
        5 => (0.45, 0.07),
        6 => (0.85, 0.10),
        _ => (1.2, 0.10),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Animal {
    pub z: f32,
    pub x: f32,
    /// -1 walks left, +1 walks right
    pub dir: f32,
    pub species: u8,
    /// it sets off when the player is closer than speed × lead seconds
    pub lead: f32,
    pub active: bool,
    pub done: bool,
    pub hit: bool,
    pub passed: bool,
    pub hit_t: f32,
}

/// Time of day presets for the menus: 0 = a running cycle, then dawn, day, dusk, night.
pub const TOD_COUNT: usize = 4;
pub const DAY_LEN: f32 = 240.0;

/// A railway level crossing: a train passes every `period` seconds and occupies it for `dur`.
#[derive(Clone, Copy, Debug)]
pub struct Crossing {
    pub z: f32,
    pub phase: f32,
    pub period: f32,
    pub dur: f32,
}

impl Crossing {
    fn t(&self, elapsed: f32) -> f32 {
        (elapsed + self.phase).rem_euclid(self.period)
    }
    /// The train is on the crossing.
    pub fn train_on(&self, elapsed: f32) -> bool {
        self.t(elapsed) < self.dur
    }
    /// 0..1 progress of the train across the road while it is on the crossing.
    pub fn progress(&self, elapsed: f32) -> f32 {
        (self.t(elapsed) / self.dur).clamp(0.0, 1.0)
    }
    /// Lights flash and the barriers close from 4 s before the train until it has left.
    pub fn warning(&self, elapsed: f32) -> bool {
        let t = self.t(elapsed);
        t < self.dur + 0.3 || t > self.period - 4.0
    }
}

/// An ambulance (kind 0) or fire engine (kind 1) coming up from behind with its siren on.
#[derive(Clone, Copy, Debug)]
pub struct Emerg {
    pub kind: u8,
    pub z: f32,
    pub x: f32,
    pub speed: f32,
    target: f32,
    pub passed: bool,
    warned: bool,
}

/// A police car chasing the player.
#[derive(Clone, Copy, Debug)]
pub struct Cop {
    pub z: f32,
    pub x: f32,
    pub speed: f32,
    pub age: f32,
    /// 0 chasing from behind, 1 overtaken and blocking the lane
    pub mode: u8,
}
pub const K_POLICE: u8 = 14;

/// (first appearance in seconds, gap between cars, at most at once); Easy has no police.
pub fn cop_params(level: usize) -> (f32, f32, usize) {
    match level {
        0 => (f32::MAX, f32::MAX, 0),
        1 => (55.0, 45.0, 1),
        2 => (40.0, 35.0, 1),
        3 => (30.0, 28.0, 2),
        _ => (22.0, 20.0, 2),
    }
}
pub fn tod_preset(i: usize) -> f32 {
    [0.25, 0.5, 0.75, 0.93][i.min(3)]
}

pub fn is_bonus(k: u8) -> bool {
    (I_TIME..=I_DOUBLE).contains(&k)
}

#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub z: f32,
    pub x: f32,
    pub kind: u8,
    pub taken: bool,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub accel: bool,
    pub brake: bool,
    pub turbo: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Playing,
    TimeUp,
    Finished,
    /// caught by the police
    Disqualified,
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max: f32,
    /// 0 dust, 1 spark, 2 flame
    pub kind: u8,
    pub size: f32,
}

/// Difficulty level: scales the clock, traffic volume and speed and how hard crashes bite.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Expert,
    Insane,
}

pub const LEVEL_COUNT: usize = 5;

impl Difficulty {
    pub fn from_index(i: usize) -> Difficulty {
        [
            Difficulty::Easy,
            Difficulty::Normal,
            Difficulty::Hard,
            Difficulty::Expert,
            Difficulty::Insane,
        ][i.min(LEVEL_COUNT - 1)]
    }
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn parse(s: &str) -> Option<Difficulty> {
        let t = s.trim().to_ascii_lowercase();
        if let Ok(i) = t.parse::<usize>() {
            return (i < LEVEL_COUNT).then(|| Self::from_index(i));
        }
        ["easy", "normal", "hard", "expert", "insane"]
            .iter()
            .position(|n| *n == t)
            .map(Self::from_index)
    }
    /// (clock multiplier, traffic count, traffic speed, crash speed retention)
    pub fn params(self) -> (f32, f32, f32, f32) {
        match self {
            Difficulty::Easy => (2.0, 0.6, 0.85, 1.25),
            Difficulty::Normal => (1.9, 1.0, 1.0, 1.0),
            Difficulty::Hard => (1.8, 1.25, 1.08, 0.9),
            Difficulty::Expert => (1.7, 1.5, 1.15, 0.8),
            Difficulty::Insane => (1.6, 1.8, 1.22, 0.7),
        }
    }
    /// Shift applied to simulated rivals' strength in career mode.
    pub fn rival_shift(self) -> f32 {
        [-0.10, 0.0, 0.05, 0.10, 0.16][self.index()]
    }
}

/// Traffic vehicle kinds.
pub const K_CAR: u8 = 0;
pub const K_TRUCK: u8 = 1;
pub const K_SEMI: u8 = 2;
pub const K_MICRO: u8 = 3;
pub const K_BIKE: u8 = 4;
pub const K_BUS: u8 = 5;
pub const K_TRIKE: u8 = 6;
pub const K_TRACTOR: u8 = 7;
// sprite-only kinds (player vehicles)
pub const K_BUGGY: u8 = 8;
pub const K_FORMULA: u8 = 9;
pub const K_MONSTER: u8 = 10;
pub const K_TUKTUK: u8 = 11;
pub const K_QUAD: u8 = 12;
pub const K_LIMO: u8 = 13;

/// Half collision width of a traffic vehicle (a normal car is 0.13).
pub fn kind_half(k: u8) -> f32 {
    match k {
        K_MICRO => 0.10,
        K_TRUCK => 0.17,
        K_SEMI => 0.22,
        K_BIKE => 0.06,
        K_BUS => 0.22,
        K_TRIKE => 0.11,
        K_TRACTOR => 0.16,
        _ => 0.13,
    }
}

#[derive(Clone)]
pub struct Car {
    pub kind: u8,
    pub z: f32,
    pub x: f32,
    pub speed: f32,
    pub base: f32,
    pub color: u8,
    prev_dz: f32,
}

/// Largest single chain bonus (a checkpoint is worth 1000, so the chain adds spice, not the score).
pub const CHAIN_CAP: u32 = 2000;

#[derive(Clone)]
pub struct Game {
    pub track: Track,
    pub pos: f32,
    pub x: f32,
    pub speed: f32,
    pub time: f32,
    pub cars: Vec<Car>,
    pub state: State,
    pub score: u32,
    pub turbo: u8,
    pub turbo_t: f32,
    pub overtaken: u32,
    /// overtake chain: consecutive overtakes (outside cities) double the bonus, 200 up to `CHAIN_CAP`
    pub chain: u32,
    pub chain_best: u32,
    /// garage coins this finished game brought in (set by the caller, shown on the end screen)
    pub coins_earned: u32,
    chain_t: f32,
    chain_seen: u32,
    pub shake: f32,
    pub invuln: f32,
    pub next_cp: usize,
    pub banner: &'static str,
    pub banner_t: f32,
    /// seconds shown after a CHECKPOINT banner
    pub banner_bonus: u32,
    pub sky_off: f32,
    pub biome_w: [f32; crate::NB],
    pub steer: f32,
    pub elapsed: f32,
    pub seed: u64,
    pub particles: Vec<Particle>,
    pub assets: Arc<Assets>,
    pub vw: usize,
    pub vh: usize,
    pub crashes: u32,
    /// demo assist (only the walkthrough sets it): the car is shielded, uses no fuel and ignores jam queues, the clock
    /// stops at 2 s and the police cannot bust it, so the AI pilot always gets through every stage
    pub demo: bool,
    pub cps: u32,
    pub turbos: u32,
    pub mode: Mode,
    pub lap: u32,
    pub orbs: Vec<Orb>,
    pub combo: u32,
    pub collected: u32,
    pub inverted: bool,
    pub car: crate::cars::CarSpec,
    pub level: Difficulty,
    pub items: Vec<Item>,
    pub weather: crate::env::Weather,
    pub surface: crate::env::Surface,
    forced_weather: Option<crate::env::Weather>,
    forced_surface: Option<crate::env::Surface>,
    /// current wind push (signed, in lane widths per second before vehicle sensitivity)
    pub wind: f32,
    wx_t: f32,
    lat_v: f32,
    /// seconds spent nearly stopped on thin ice (it breaks at 1.5)
    pub ice_t: f32,
    /// z ranges of thin ice (frozen-lake stages)
    pub thin: Vec<(f32, f32)>,
    /// remaining seconds of: shield, boost, traffic freeze, double score, oil slip, wet road, mud
    pub shield_t: f32,
    pub boost_t: f32,
    /// driving assist 0..=3: 1 the car holds bends, 2 it also drifts back to the lane centre,
    /// 3 walls and traffic cost less speed
    pub assist: u8,
    /// top speed scale 0.4..=1.0 (reduced-speed mode)
    pub speed_cap: f32,
    pub freeze_t: f32,
    pub x2_t: f32,
    pub slip_t: f32,
    pub wet_t: f32,
    pub mud_t: f32,
    /// bonuses picked up / obstacles hit
    pub bonuses: u32,
    pub hits: u32,
    /// animals crossing the road
    pub animals: Vec<Animal>,
    pub animals_hit: u32,
    pub animals_dodged: u32,
    /// fuel stations as z ranges
    pub stations: Vec<(f32, f32)>,
    /// tank, 1 = full
    pub fuel: f32,
    /// photo mode: the game is paused, the camera can be moved
    pub photo: bool,
    /// pause: nothing moves, an overlay offers resume / restart
    pub paused: bool,
    pub photo_dx: f32,
    pub photo_dy: f32,
    pub photo_hud: bool,
    /// the tank ran empty at some point of this game
    pub ran_dry: bool,
    /// stage-designer layout used by every track this game builds
    pub layout: crate::track::Layout,
    /// the ghost to race against and the recording of this run
    pub ghost: Option<crate::ghost::Ghost>,
    ghost_rec: Vec<(f32, f32)>,
    ghost_acc: f32,
    /// condition of the body and of the engine, 1.0 = perfect; repaired at fuel stations
    pub body: f32,
    pub engine: f32,
    /// the fuel rule can be switched off (tests, practice)
    pub fuel_rule: bool,
    pub refueling: bool,
    /// seconds spent stopped with an empty tank (a tow truck brings fuel after a while)
    dry_t: f32,
    fuel_warned: bool,
    /// height above the road in car heights, vertical speed, size of the current jump (1..3)
    pub air: f32,
    pub air_v: f32,
    pub air_size: u8,
    pub jumps: u32,
    /// turbo pads driven over
    pub pads: u32,
    wall_cd: f32,
    /// gearbox: 1..=5, manual or automatic, shift cut timer
    pub gear: u8,
    pub manual: bool,
    pub shift_t: f32,
    pub rpm: f32,
    /// time of day in [0,1): 0 midnight, 0.25 dawn, 0.5 noon, 0.75 dusk
    pub tod: f32,
    pub tod_frozen: bool,
    /// level 0..=999 from the player's profile: more traffic, hazards, police and a tighter clock
    pub rank: u16,
    /// tyre set fitted, its wear (1 new .. 0 bald) and what the player asked for at service (None = auto)
    pub tyre: crate::env::Tyre,
    pub tyre_wear: f32,
    pub tyre_pref: Option<crate::env::Tyre>,
    pub setup: crate::env::Setup,
    /// level crossings with trains
    pub crossings: Vec<Crossing>,
    pub close_calls: u32,
    pub cops: Vec<Cop>,
    /// 0..1: at 1 the police disqualify you
    pub bust: f32,
    next_cop: f32,
    /// cities (lights, crosswalks, speed limit) and traffic jams of this stage
    pub city: crate::city::CityPlan,
    /// the standing queues of the jams
    pub jam_cars: Vec<Car>,
    /// index of the jam whose detour the player is on
    pub detour: Option<usize>,
    detour_orig: Vec<f32>,
    pub violations: u32,
    pub peds_hit: u32,
    pub peds_yielded: u32,
    pub detours: u32,
    /// consecutive good acts (stopping at red, yielding, clean cities); an offence resets it
    /// delivery mission of this stage and its progress
    /// announced bonus areas (proving ground, parking lots) and their state: 0 new, 1 announced, 2 done
    pub areas: Vec<crate::bonus::Area>,
    pub a_state: Vec<u8>,
    a_dwell: f32,
    pub parked: u32,
    pub tracks_clear: u32,
    pub mission: Option<crate::mission::Mission>,
    /// the fugitive of a police pursuit (police car only) and the number of arrests
    pub pursuit: Option<crate::pursuit::Pursuit>,
    pursuit_done: bool,
    pub arrests: u32,
    /// the visible pack of rivals (empty = none), its grid configuration and the race position
    pub pack: Vec<crate::pack::Rival>,
    pub pack_cfg: Option<crate::pack::PackCfg>,
    pub place: usize,
    /// route fork: (first segment, length); 0 = not chosen yet, 1 = quick route, 2 = scenic route
    pub fork: Option<(usize, usize)>,
    pub route: u8,
    fork_warn: bool,
    route_paid: bool,
    pub m_state: u8,
    /// seconds left to deliver (can go negative: a late delivery earns no tip)
    pub m_t: f32,
    m_dwell: f32,
    m_body0: f32,
    m_viol0: u32,
    m_crash0: u32,
    m_hint: u8,
    pub missions_done: u32,
    pub emerg: Vec<Emerg>,
    next_emerg: f32,
    block_t: f32,
    pub emerg_passed: u32,
    scene_warn: usize,
    pub civic: u32,
    pub good_acts: u32,
    pub clean_cities: u32,
    zone_clean: bool,
    waited: bool,
    viol_cd: f32,
    speeding_t: f32,
    prev_zone: Option<usize>,
    jam_warn: usize,
    fx_rng: Rng,
    prev_turbo: bool,
}

fn wrap(d: f32, len: f32) -> f32 {
    let mut d = d;
    if d > len / 2.0 {
        d -= len;
    }
    if d < -len / 2.0 {
        d += len;
    }
    d
}

mod effects;
mod events;
mod urban;
mod world;

impl Game {
    pub fn new(seed: u64) -> Self {
        Self::with_assets(seed, Assets::global(seed))
    }

    pub fn new_mode(seed: u64, mode: Mode) -> Self {
        Self::with_mode(seed, Assets::global(seed), mode)
    }

    pub fn with_assets(seed: u64, assets: Arc<Assets>) -> Self {
        Self::with_mode(seed, assets, Mode::Campaign)
    }

    /// 1 in full daylight, 0 at night, smooth through dawn and dusk.
    pub fn daylight(&self) -> f32 {
        let t = self.tod;
        let ss = |x: f32| {
            let x = x.clamp(0.0, 1.0);
            x * x * (3.0 - 2.0 * x)
        };
        if !(0.2..0.8).contains(&t) {
            0.0
        } else if t < 0.3 {
            ss((t - 0.2) / 0.1)
        } else if t < 0.7 {
            1.0
        } else {
            1.0 - ss((t - 0.7) / 0.1)
        }
    }

    /// Warm twilight strength: peaks at dawn (0.25) and dusk (0.75).
    pub fn sunset(&self) -> f32 {
        let d = (self.tod - 0.25).abs().min((self.tod - 0.75).abs());
        (1.0 - d / 0.09).clamp(0.0, 1.0)
    }

    /// Clock reading of the in-game day, in minutes after midnight.
    pub fn clock_minutes(&self) -> u32 {
        ((self.tod.rem_euclid(1.0)) * 1440.0) as u32 % 1440
    }

    /// 0 = a running cycle; 1..=4 = frozen dawn, day, dusk, night.
    pub fn set_time(&mut self, i: usize) {
        if (1..=TOD_COUNT).contains(&i) {
            self.tod = tod_preset(i - 1);
            self.tod_frozen = true;
        } else {
            self.tod_frozen = false;
        }
    }

    /// Practice mode: no hazards, animals, police or fuel rule (used by balance tests and tools).
    pub fn practice(&mut self) {
        self.items.clear();
        self.animals.clear();
        self.cops.clear();
        self.next_cop = f32::MAX;
        self.fuel_rule = false;
    }

    /// Service area: choose the tyre set (None = what the conditions ask for) and the set-up.
    pub fn set_equipment(&mut self, tyre: Option<crate::env::Tyre>, setup: crate::env::Setup) {
        self.tyre_pref = tyre;
        self.setup = setup;
        self.tyre = tyre.unwrap_or_else(|| crate::env::Tyre::best_for(self.weather, self.surface));
        self.tyre_wear = 1.0;
        self.reset_clock();
    }

    /// Rank from the player's profile (0..=999); regenerates traffic, hazards and animals before the start.
    /// Accessibility: assist level 0..=3 and the top-speed scale (0.4..=1.0).
    pub fn set_access(&mut self, assist: u8, speed_cap: f32) {
        self.assist = assist.min(crate::access::ASSIST_MAX);
        self.speed_cap = if speed_cap.is_finite() {
            speed_cap.clamp(0.4, 1.0)
        } else {
            1.0
        };
    }

    pub fn set_rank(&mut self, rank: u16) {
        self.rank = rank.min(999);
        if self.pos < 1.0 && self.elapsed < 0.01 {
            self.cars = Self::make_cars(&self.track, self.seed, self.mode, self.level, self.rank);
            self.items = Self::make_items(
                &self.track,
                self.seed,
                self.mode,
                self.level,
                self.surface,
                self.rank,
            );
            self.animals = Self::make_animals(
                &self.track,
                self.seed,
                self.mode,
                self.level,
                self.surface,
                self.rank,
            );
            self.city_animals();
            let (first, _, _) = self.cop_params_r();
            self.next_cop = first;
            self.reset_clock();
        }
    }

    /// Police timing from level and rank.
    fn cop_params_r(&self) -> (f32, f32, usize) {
        let (first, gap, maxc) = cop_params(self.level.index());
        let rf = self.rank as f32 / 999.0;
        (
            first * (1.0 - 0.45 * rf),
            gap * (1.0 - 0.45 * rf),
            maxc + (self.rank >= 600 && maxc > 0) as usize,
        )
    }

    /// Distance a full tank lasts: more than the gap between two stations at every level, a little
    /// less on harder levels, with a heavy set-up and at a high rank.
    fn fuel_range(&self) -> f32 {
        let li = self.level.index();
        self.track.length
            * 0.62
            * [1.3, 1.0, 0.9, 0.8, 0.7][li]
            * self.setup.mods().3
            * (1.0 - 0.15 * self.rank as f32 / 999.0)
    }

    /// Fuel level at which the warning shows (and the autopilot heads for a station): the tank must
    /// still reach the next station (stations are 16 % of the stage apart) with a margin.
    pub fn low_fuel_level(&self) -> f32 {
        (0.20 * self.track.length / self.fuel_range().max(1.0)).clamp(0.2, 0.9)
    }

    pub fn shift_up(&mut self) {
        if self.manual && self.gear < 5 {
            self.gear += 1;
        }
    }

    pub fn shift_down(&mut self) {
        if self.manual && self.gear > 1 {
            self.gear -= 1;
        }
    }

    pub fn toggle_manual(&mut self) {
        self.manual = !self.manual;
    }

    fn launch(&mut self, kind: u8) {
        if self.air > 0.0 {
            return;
        }
        let sp = (self.speed / MAX_SPEED).min(1.3);
        if sp < 0.25 {
            return;
        }
        let sz = (kind - I_RAMP_S) as usize;
        self.air_v = [3.6, 6.4, 10.0][sz] * (0.5 + 0.5 * sp.min(1.0));
        self.air = 0.001;
        self.air_size = sz as u8 + 1;
        self.jumps += 1;
        self.banner = if sz == 2 { "BIG AIR" } else { "JUMP" };
        self.banner_t = 1.0;
    }

    /// Structure under the player (F_*).
    pub fn feat_here(&self) -> u8 {
        self.track.seg_at(self.pos + PLAYER_Z).feat
    }

    /// Forced environment as menu indices (0 = automatic, k = type k-1).
    pub fn forced_env(&self) -> (usize, usize) {
        (
            self.forced_weather.map_or(0, |w| w.index() + 1),
            self.forced_surface.map_or(0, |s| s.index() + 1),
        )
    }

    /// Apply menu indices (0 = automatic). The surface goes first so the weather can follow it.
    pub fn set_env(&mut self, weather: usize, surface: usize) {
        use crate::env::{Surface, Weather, SURFACE_COUNT, WEATHER_COUNT};
        self.set_surface(
            (1..=SURFACE_COUNT)
                .contains(&surface)
                .then(|| Surface::from_index(surface - 1)),
        );
        self.set_weather(
            (1..=WEATHER_COUNT)
                .contains(&weather)
                .then(|| Weather::from_index(weather - 1)),
        );
    }

    /// Force a weather (None = chosen from the stage seed). Before the start only.
    pub fn set_weather(&mut self, w: Option<crate::env::Weather>) {
        self.forced_weather = w;
        self.weather = w.unwrap_or_else(|| crate::env::auto_weather(self.seed, self.surface));
        self.refit_tyre();
    }

    /// Before the start the tyres follow the conditions (unless a set was chosen).
    fn refit_tyre(&mut self) {
        if self.pos < 1.0 && self.elapsed < 0.01 {
            self.tyre = self
                .tyre_pref
                .unwrap_or_else(|| crate::env::Tyre::best_for(self.weather, self.surface));
            self.tyre_wear = 1.0;
        }
    }

    /// Force a road surface (None = from the seed). Regenerates hazards; before the start only.
    pub fn set_surface(&mut self, sf: Option<crate::env::Surface>) {
        self.forced_surface = sf;
        self.surface = sf.unwrap_or_else(|| crate::env::auto_surface(self.seed));
        if self.forced_weather.is_none() {
            self.weather = crate::env::auto_weather(self.seed, self.surface);
        }
        if self.pos < 1.0 && self.elapsed < 0.01 {
            self.items = Self::make_items(
                &self.track,
                self.seed,
                self.mode,
                self.level,
                self.surface,
                self.rank,
            );
            self.thin = Self::make_thin(&self.track, self.seed, self.surface);
            self.animals = Self::make_animals(
                &self.track,
                self.seed,
                self.mode,
                self.level,
                self.surface,
                self.rank,
            );
            self.city_animals();
        }
        self.refit_tyre();
    }

    /// Combined tyre grip of weather x surface for the current vehicle (0.25..=1).
    pub fn env_grip(&self) -> f32 {
        let g = self.weather.params().grip * self.surface.params().0;
        let base = g + (1.0 - g) * self.car.traction;
        let tyre = self.tyre.grip(self.weather, self.surface)
            * (0.72 + 0.28 * self.tyre_wear.clamp(0.0, 1.0));
        (base * tyre * self.setup.mods().2).clamp(0.25, 1.15)
    }

    fn make_orbs(track: &Track, seed: u64, mode: Mode) -> Vec<Orb> {
        let s32 = (seed as u32).wrapping_mul(2_246_822_519) ^ 0x1B87_3593;
        let end = track.finish_seg as f32 * SEG_LEN - 2000.0;
        let mut v = Vec::new();
        match mode {
            Mode::Collector => {
                let n = 150;
                for i in 0..n {
                    // fBm-driven flowing trails: neighbouring orbs form smooth fractal curves across the lanes
                    let x = ((fbm(i as f32 * 0.09, 3.3, s32, 4, 0) - 0.5) * 4.2).clamp(-0.95, 0.95);
                    v.push(Orb {
                        z: 3000.0 + (end - 3000.0) * i as f32 / n as f32,
                        x,
                        kind: 0,
                        taken: false,
                    });
                }
            }
            Mode::Slalom => {
                let mut z = 3200.0;
                let mut i = 0;
                while z < end {
                    let x = ((fbm(i as f32 * 0.33, 7.7, s32, 3, 0) - 0.5) * 3.2).clamp(-0.8, 0.8);
                    v.push(Orb {
                        z,
                        x,
                        kind: 1,
                        taken: false,
                    });
                    z += SEG_LEN * (11.0 + 6.0 * fbm(i as f32 * 0.7, 1.1, s32, 2, 0));
                    i += 1;
                }
            }
            _ => {}
        }
        v
    }

    pub fn with_mode(seed: u64, assets: Arc<Assets>, mode: Mode) -> Self {
        let track = track::build(seed);
        let cars = Self::make_cars(&track, seed, mode, Difficulty::Normal, 0);
        let surface = crate::env::auto_surface(seed);
        let weather = crate::env::auto_weather(seed, surface);
        let items = Self::make_items(&track, seed, mode, Difficulty::Normal, surface, 0);
        let thin = Self::make_thin(&track, seed, surface);
        let orbs = Self::make_orbs(&track, seed, mode);
        let animals = Self::make_animals(&track, seed, mode, Difficulty::Normal, surface, 0);
        let stations = Self::make_stations(&track, seed);
        let crossings = Self::make_crossings(&track, seed, &stations);
        let mut g = Game {
            pos: 0.0,
            x: 0.0,
            speed: 0.0,
            time: Self::base_time(mode),
            cars,
            state: State::Playing,
            score: 0,
            overtaken: 0,
            chain: 0,
            chain_best: 0,
            coins_earned: 0,
            chain_t: 0.0,
            chain_seen: 0,
            shake: 0.0,
            invuln: 0.0,
            next_cp: 0,
            banner: "GO!",
            banner_t: 1.5,
            banner_bonus: 0,
            sky_off: 0.0,
            biome_w: {
                let mut w = [0.0; crate::NB];
                w[track.segs[0].biome as usize] = 1.0;
                w
            },
            turbo: 3,
            turbo_t: 0.0,
            steer: 0.0,
            elapsed: 0.0,
            seed,
            particles: Vec::new(),
            assets,
            vw: W,
            vh: H,
            crashes: 0,
            demo: false,
            cps: 0,
            turbos: 0,
            mode,
            lap: 1,
            orbs,
            combo: 0,
            collected: 0,
            inverted: false,
            car: crate::cars::CarSpec::default(),
            level: Difficulty::Normal,
            items,
            weather,
            surface,
            forced_weather: None,
            forced_surface: None,
            wind: 0.0,
            wx_t: 0.0,
            lat_v: 0.0,
            ice_t: 0.0,
            thin,
            shield_t: 0.0,
            boost_t: 0.0,
            assist: 0,
            speed_cap: 1.0,
            freeze_t: 0.0,
            x2_t: 0.0,
            slip_t: 0.0,
            wet_t: 0.0,
            mud_t: 0.0,
            bonuses: 0,
            hits: 0,
            animals,
            animals_hit: 0,
            animals_dodged: 0,
            stations,
            fuel: 1.0,
            photo: false,
            paused: false,
            photo_dx: 0.0,
            photo_dy: 0.0,
            photo_hud: false,
            ran_dry: false,
            layout: crate::track::Layout::default(),
            ghost: None,
            ghost_rec: Vec::new(),
            ghost_acc: 0.0,
            body: 1.0,
            engine: 1.0,
            fuel_rule: true,
            refueling: false,
            dry_t: 0.0,
            fuel_warned: false,
            air: 0.0,
            air_v: 0.0,
            air_size: 0,
            jumps: 0,
            pads: 0,
            wall_cd: 0.0,
            gear: 1,
            manual: false,
            shift_t: 0.0,
            rpm: 0.0,
            tod: {
                let h = (seed as u32).wrapping_mul(2_654_435_761) ^ 0x7D0D;
                (h >> 8) as f32 / (1u32 << 24) as f32
            },
            tod_frozen: false,
            rank: 0,
            tyre: crate::env::Tyre::best_for(weather, surface),
            tyre_wear: 1.0,
            tyre_pref: None,
            setup: crate::env::Setup::Balanced,
            crossings,
            close_calls: 0,
            cops: Vec::new(),
            bust: 0.0,
            next_cop: cop_params(1).0,
            fx_rng: Rng::new(seed ^ 0x77),
            prev_turbo: false,
            city: crate::city::CityPlan::default(),
            jam_cars: Vec::new(),
            detour: None,
            detour_orig: Vec::new(),
            violations: 0,
            peds_hit: 0,
            peds_yielded: 0,
            detours: 0,
            areas: Vec::new(),
            a_state: Vec::new(),
            a_dwell: 0.0,
            parked: 0,
            tracks_clear: 0,
            mission: None,
            pursuit: None,
            pursuit_done: false,
            arrests: 0,
            pack: Vec::new(),
            pack_cfg: None,
            place: 0,
            fork: None,
            route: 0,
            fork_warn: false,
            route_paid: false,
            m_state: 0,
            m_t: 0.0,
            m_dwell: 0.0,
            m_body0: 1.0,
            m_viol0: 0,
            m_crash0: 0,
            m_hint: 0,
            missions_done: 0,
            emerg: Vec::new(),
            next_emerg: 28.0,
            block_t: 0.0,
            emerg_passed: 0,
            scene_warn: usize::MAX,
            civic: 0,
            good_acts: 0,
            clean_cities: 0,
            zone_clean: false,
            waited: false,
            viol_cd: 0.0,
            speeding_t: 0.0,
            prev_zone: None,
            jam_warn: usize::MAX,
            track,
        };
        g.setup_city(seed);
        g
    }

    /// Endless / Zen: a fresh generated stage starts when the finish line is crossed.
    fn next_lap(&mut self) {
        self.lap += 1;
        let s2 = self.seed ^ (self.lap as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.track = track::with_layout(self.layout, || track::build(s2));
        self.cars = Self::make_cars(&self.track, s2, self.mode, self.level, self.rank);
        self.surface = self
            .forced_surface
            .unwrap_or_else(|| crate::env::auto_surface(s2));
        self.weather = self
            .forced_weather
            .unwrap_or_else(|| crate::env::auto_weather(s2, self.surface));
        self.items = Self::make_items(
            &self.track,
            s2,
            self.mode,
            self.level,
            self.surface,
            self.rank,
        );
        self.thin = Self::make_thin(&self.track, s2, self.surface);
        self.animals = Self::make_animals(
            &self.track,
            s2,
            self.mode,
            self.level,
            self.surface,
            self.rank,
        );
        self.city_animals();
        self.stations = Self::make_stations(&self.track, s2);
        self.crossings = Self::make_crossings(&self.track, s2, &self.stations);
        self.setup_city(s2);
        self.ice_t = 0.0;
        self.orbs = Self::make_orbs(&self.track, s2, self.mode);
        self.inject_areas();
        self.pos = 0.0;
        self.next_cp = 0;
        if let Some(c) = self.pack_cfg {
            self.set_pack(Some(c));
        }
        self.score += 2000;
        if self.mode == Mode::Endless {
            self.time += 25.0;
        }
        self.banner = "LAP";
        self.banner_t = 2.5;
    }

    /// Effect of driving over an obstacle or collecting a bonus.
    fn apply_item(&mut self, kind: u8) {
        if is_ramp(kind) {
            self.launch(kind);
            return;
        }
        if kind == I_PAD {
            self.turbo_t = self.turbo_t.max(2.0);
            self.speed = (self.speed + MAX_SPEED * 0.12).min(MAX_SPEED * 1.3);
            self.pads += 1;
            self.banner = "TURBO PAD";
            self.banner_t = 1.0;
            return;
        }
        let protect = self.shield_t > 0.0 || self.invuln > 0.0;
        let say = |g: &mut Game, b: &'static str| {
            g.banner = b;
            g.banner_t = 1.2;
        };
        if is_bonus(kind) {
            self.bonuses += 1;
        } else if !protect {
            self.hits += 1;
        }
        match kind {
            I_CONE if !protect => {
                self.speed *= 0.93;
                self.shake = 0.15;
            }
            I_OIL if !protect => {
                self.slip_t = 1.4;
                say(self, "OIL!");
            }
            I_BARRIER if !protect => {
                self.speed = (self.speed * 0.35 * self.car.crash_keep * self.level.params().3)
                    .min(self.speed * 0.95);
                self.shake = 0.5;
                self.invuln = 0.6;
                self.crashes += 1;
                self.hurt(0.12, 0.05);
                self.burst_sparks();
                say(self, "CRASH");
            }
            I_POTHOLE if !protect => {
                self.speed *= 0.85;
                self.shake = 0.35;
                say(self, "HOLE");
            }
            I_PUDDLE if !protect => {
                self.wet_t = 2.0;
                self.speed *= 0.97;
                say(self, "WET");
            }
            I_CRACK if !protect => {
                self.speed *= 0.7;
                self.shake = 0.5;
                self.slip_t = 0.8;
                self.x += if self.x >= 0.0 { 0.25 } else { -0.25 };
                say(self, "CRACK");
            }
            I_MAGMA if !protect => {
                self.speed *= 0.55;
                self.shake = 0.5;
                if self.mode != Mode::Zen {
                    self.time -= 1.5;
                }
                say(self, "MAGMA");
            }
            I_MUD if !protect => {
                self.mud_t = 1.5;
                say(self, "MUD");
            }
            I_TIME => {
                if self.mode == Mode::TimeTrial {
                    self.score += 300;
                } else {
                    self.time += 4.0;
                }
                say(self, "+4 S");
            }
            I_TURBO => {
                self.turbo = (self.turbo + 1).min(9);
                say(self, "TURBO +1");
            }
            I_BOOST => {
                self.boost_t = 3.0;
                say(self, "BOOST");
            }
            I_SHIELD => {
                self.shield_t = 6.0;
                say(self, "SHIELD");
            }
            I_COIN => {
                self.score += if self.x2_t > 0.0 { 1000 } else { 500 };
                say(self, "+500");
            }
            I_FREEZE => {
                self.freeze_t = 5.0;
                say(self, "FREEZE");
            }
            I_DOUBLE => {
                self.x2_t = 12.0;
                say(self, "X2");
            }
            _ => {}
        }
    }

    fn base_time(mode: Mode) -> f32 {
        match mode {
            Mode::TimeTrial => 0.0,
            Mode::Zen => 99.0,
            Mode::Collector => 32.0,
            Mode::Slalom => 36.0,
            _ => START_TIME,
        }
    }

    /// Clock multiplier from difficulty and vehicle class.
    pub fn clock_mult(&self) -> f32 {
        self.level.params().0
            * self.car.time_mult
            * (1.0 - 0.30 * self.rank.min(999) as f32 / 999.0)
    }

    /// Before the start only: recompute the starting clock.
    fn reset_clock(&mut self) {
        if self.pos < 1.0
            && self.elapsed < 0.01
            && !matches!(self.mode, Mode::TimeTrial | Mode::Zen)
        {
            self.time = Self::base_time(self.mode) * self.clock_mult();
        }
    }

    /// Choose the difficulty level (before the start): regenerates the traffic and the clock.
    pub fn set_level(&mut self, level: Difficulty) {
        self.level = level;
        if self.pos < 1.0 && self.elapsed < 0.01 {
            self.cars = Self::make_cars(&self.track, self.seed, self.mode, level, self.rank);
            self.items = Self::make_items(
                &self.track,
                self.seed,
                self.mode,
                level,
                self.surface,
                self.rank,
            );
            self.animals = Self::make_animals(
                &self.track,
                self.seed,
                self.mode,
                level,
                self.surface,
                self.rank,
            );
            self.city_animals();
            self.next_cop = self.cop_params_r().0;
            self.reset_clock();
        }
    }

    /// Fit a country's car: livery, handling profile and number of turbo charges.
    pub fn set_car(&mut self, car: crate::cars::CarSpec) {
        // hostile or corrupt specs are clamped to sane handling
        let sane = |v: f32, lo: f32, hi: f32| if v.is_finite() { v.clamp(lo, hi) } else { 1.0 };
        let mut car = crate::garage::upgrade(car);
        car.top = sane(car.top, 0.5, 1.3);
        car.accel = sane(car.accel, 0.5, 1.3);
        car.grip = sane(car.grip, 0.5, 1.3);
        car.turbos = car.turbos.clamp(1, 6);
        car.width = sane(car.width, 0.4, 1.5);
        car.crash_keep = sane(car.crash_keep, 0.5, 1.6);
        car.prop_keep = sane(car.prop_keep, 0.5, 2.0);
        car.time_mult = sane(car.time_mult, 0.8, 1.4);
        car.traction = if car.traction.is_finite() {
            car.traction.clamp(-0.5, 1.0)
        } else {
            0.0
        };
        if let Some((b, c2, c3)) = crate::settings::livery_colours(crate::settings::livery()) {
            if !car.class.is_special() && car.body == 0xD61C24 {
                (car.body, car.c2, car.c3) = (b, c2, c3);
            }
        }
        self.turbo = car.turbos;
        self.car = car;
        self.reset_clock();
    }

    /// A hit: wears the body and the engine (harder levels hurt a little more).
    pub fn hurt(&mut self, body: f32, engine: f32) {
        let k = 0.8 + 0.1 * self.level.index() as f32;
        self.body = (self.body - body * k).max(0.0);
        self.engine = (self.engine - engine * k).max(0.0);
    }

    /// Race against a ghost (only when it belongs to the same stage and mode).
    pub fn set_ghost(&mut self, g: Option<crate::ghost::Ghost>) {
        self.ghost =
            g.filter(|g| g.seed == self.seed && g.mode == self.mode as u8 && !g.pts.is_empty());
    }

    /// The ghost's position (distance, lateral) at the player's current time.
    pub fn ghost_now(&self) -> Option<(f32, f32)> {
        self.ghost.as_ref()?.at(self.elapsed)
    }

    /// Seconds the player is ahead (+) or behind (-) of the ghost, estimated from the distance and speed.
    pub fn ghost_gap(&self) -> Option<f32> {
        let (gp, _) = self.ghost_now()?;
        Some((self.pos - gp) / self.speed.max(MAX_SPEED * 0.25))
    }

    /// The recording of a finished run as a ghost (None until the game is finished).
    pub fn make_ghost(&self) -> Option<crate::ghost::Ghost> {
        if self.state != State::Finished || self.ghost_rec.is_empty() {
            return None;
        }
        let mut pts = self.ghost_rec.clone();
        pts.push((self.pos, self.x));
        Some(crate::ghost::Ghost {
            seed: self.seed,
            mode: self.mode as u8,
            class: self.car.class.index() as u8,
            secs: self.elapsed,
            score: self.score,
            pts,
        })
    }

    pub fn kmh(&self) -> i32 {
        (self.speed / MAX_SPEED * 260.0).round() as i32
    }

    /// Enter / leave photo mode (only while the game is running: a finished game stays as it is).
    /// Pauses a running stage, or resumes a paused one; menus and finished stages ignore it.
    pub fn toggle_pause(&mut self) {
        if self.paused {
            self.paused = false;
        } else if self.state == State::Playing {
            self.paused = true;
            if self.photo {
                self.photo_toggle();
            }
        }
    }

    pub fn photo_toggle(&mut self) {
        if self.photo {
            self.photo = false;
            self.photo_dx = 0.0;
            self.photo_dy = 0.0;
        } else if self.state == State::Playing {
            self.photo = true;
            self.photo_hud = false;
        }
    }

    pub fn photo_hud_toggle(&mut self) {
        if self.photo {
            self.photo_hud = !self.photo_hud;
        }
    }

    /// Photo mode controls: left/right move the camera sideways, accel/brake scrub along the road
    /// (with turbo held: raise / lower the camera).
    fn photo_update(&mut self, dt: f32, inp: &Input) {
        let side = (inp.right as i32 - inp.left as i32) as f32;
        self.photo_dx = (self.photo_dx + side * dt * 1.2).clamp(-1.6, 1.6);
        let v = (inp.accel as i32 - inp.brake as i32) as f32;
        if inp.turbo {
            self.photo_dy = (self.photo_dy + v * dt * 900.0).clamp(-600.0, 2600.0);
        } else {
            let end = self.track.finish_seg as f32 * crate::track::SEG_LEN;
            self.pos = (self.pos + v * dt * MAX_SPEED * 0.4).clamp(0.0, end);
        }
    }

    pub fn update(&mut self, dt: f32, inp: &Input) {
        let dt = if dt.is_finite() {
            dt.clamp(0.0, 0.05)
        } else {
            0.0
        };
        if self.paused {
            return;
        }
        if self.photo {
            self.photo_update(dt, inp);
            return;
        }
        self.elapsed += dt;
        self.banner_t = (self.banner_t - dt).max(0.0);
        self.shake = (self.shake - dt).max(0.0);
        self.invuln = (self.invuln - dt).max(0.0);
        for t in [
            &mut self.shield_t,
            &mut self.boost_t,
            &mut self.freeze_t,
            &mut self.x2_t,
            &mut self.slip_t,
            &mut self.wet_t,
            &mut self.mud_t,
        ] {
            *t = (*t - dt).max(0.0);
        }
        self.step_particles(dt);
        if self.demo {
            // demo assist: the car is always shielded
            self.shield_t = self.shield_t.max(1.0);
        }
        self.wall_cd = (self.wall_cd - dt).max(0.0);
        self.shift_t = (self.shift_t - dt).max(0.0);
        if !self.tod_frozen {
            self.tod = (self.tod + dt / DAY_LEN).rem_euclid(1.0);
        }
        let n = self.track.segs.len();
        let len = self.track.length;

        // scenery colour blending follows the player's current biome
        let biome = self.track.seg_at(self.pos + PLAYER_Z).biome as usize;
        for (i, w) in self.biome_w.iter_mut().enumerate() {
            let t = if i == biome { 1.0 } else { 0.0 };
            *w += (t - *w) * (dt * 1.2).min(1.0);
        }

        if self.state != State::Playing {
            self.speed = (self.speed - DECEL * 2.0 * dt).max(0.0);
            return;
        }

        let zen = self.mode == Mode::Zen;
        // Mirror mode: controls flip for six seconds out of every fourteen
        let was_inv = self.inverted;
        self.inverted = self.mode == Mode::Mirror && self.elapsed % 14.0 > 8.0;
        if self.inverted != was_inv {
            self.banner = if self.inverted { "INVERT" } else { "NORMAL" };
            self.banner_t = 1.8;
        }
        let (il, ir) = if self.inverted {
            (inp.right, inp.left)
        } else {
            (inp.left, inp.right)
        };

        let old_pos = self.pos;
        let sp = (self.speed / MAX_SPEED).min(1.4);
        let cur = self.track.seg_at(self.pos + PLAYER_Z).curve;
        let wp = self.weather.params();
        let sp_env = self.surface.params();
        let feat = self.track.seg_at(self.pos + PLAYER_Z).feat;
        let eg = self.env_grip();
        // inside a tunnel the road is dry
        let eg = if feat == F_TUNNEL {
            eg + (1.0 - eg) * 0.6
        } else {
            eg
        };
        // gusts: two summed sine waves, felt most by narrow and light vehicles; bridges and
        // viaducts are exposed, tunnels and passes shelter
        self.wx_t += dt;
        let (expo, wf) = match feat {
            F_BRIDGE => (0.12, 1.8),
            F_VIADUCT => (0.12, 2.2),
            F_TUNNEL => (0.0, 0.0),
            F_PASS => (0.0, 0.4),
            _ => (0.0, 1.0),
        };
        self.wind =
            (wp.wind + expo) * wf * ((self.wx_t * 0.7).sin() + 0.6 * (self.wx_t * 1.9 + 1.3).sin());
        let wsens = (1.0 / self.car.width.clamp(0.5, 1.4)).clamp(0.7, 2.0);
        self.x += self.wind * dt * 0.3 * wsens * sp.clamp(0.25, 1.0);
        let slick = if self.slip_t > 0.0 {
            0.35
        } else if self.wet_t > 0.0 {
            0.65
        } else {
            1.0
        };
        let dx = dt * 2.0 * sp.min(1.0) * self.car.grip * slick;
        if self.slip_t > 0.0 {
            // oil: the car slides sideways
            self.x += (self.elapsed * 11.0).sin() * dt * 0.9 * sp.min(1.0);
        }

        // steering: on slippery ground the lateral speed follows the wheel with inertia (the car keeps sliding)
        let dir = (ir as i32 - il as i32) as f32;
        let airf = if self.air > 0.0 { 0.3 } else { 1.0 };
        let vt = dir
            * 2.0
            * sp.min(1.0)
            * self.car.grip
            * slick
            * eg.sqrt()
            * airf
            * (0.85 + 0.15 * self.body);
        if eg >= 0.995 {
            self.lat_v = vt;
        } else {
            self.lat_v += (vt - self.lat_v) * (dt * 40.0 * eg * eg).min(1.0);
        }
        self.x += self.lat_v * dt;
        let hold = if self.assist >= 1 { 0.4 } else { 1.0 };
        self.x -= dx * sp * cur * CENTRIFUGAL * hold / (self.car.grip * self.car.grip * eg.sqrt());
        if self.assist >= 2 && dir == 0.0 && self.air <= 0.0 {
            self.x -= self.x.clamp(-1.0, 1.0) * dt * 0.7 * sp.min(1.0);
        }
        let target = (ir as i32 - il as i32) as f32;
        self.steer += (target - self.steer) * (dt * 8.0).min(1.0);

        let turbo_on = self.turbo_t > 0.0;
        let top = self.car.top.clamp(0.5, 1.3)
            * if turbo_on {
                MAX_SPEED * 1.3
            } else if zen {
                MAX_SPEED * 0.7
            } else {
                MAX_SPEED
            };
        let top = top * wp.top * sp_env.1 * self.setup.mods().1;
        let top = if self.boost_t > 0.0 { top * 1.18 } else { top };
        let top = if self.mud_t > 0.0 { top * 0.55 } else { top };
        let fuel_on = self.fuel_rule && !zen;
        let empty = fuel_on && self.fuel <= 0.0;
        if empty {
            self.ran_dry = true;
        }
        let top = if empty { top * 0.18 } else { top };
        let top = top * (0.7 + 0.3 * self.engine);
        let top = top * self.speed_cap;
        // gearbox: five gears over the speed range; manual rewards good shifts
        let top_ref = MAX_SPEED * self.car.top.clamp(0.5, 1.3);
        let band = (self.speed / top_ref * 5.0).max(0.0);
        let gfac = if self.manual {
            let rpm = band - (self.gear as f32 - 1.0);
            self.rpm = rpm.clamp(-0.2, 1.4);
            let redline = if self.gear == 5 { 1.5 } else { 1.05 };
            if rpm > redline {
                0.12
            } else if rpm < 0.0 {
                0.55
            } else {
                0.7 + 0.7 * rpm.min(1.0)
            }
        } else {
            let want = ((band as usize) + 1).clamp(1, 5) as u8;
            if want != self.gear
                && self.shift_t <= 0.0
                && (want > self.gear || band < self.gear as f32 - 1.0 - 0.12)
            {
                self.gear = want;
                self.shift_t = 0.16;
            }
            self.rpm = (band - (self.gear as f32 - 1.0)).clamp(0.0, 1.4);
            if self.shift_t > 0.0 {
                0.4
            } else {
                1.0
            }
        };
        let acc = ACCEL
            * self.car.accel.clamp(0.5, 1.3)
            * self.setup.mods().0
            * if self.boost_t > 0.0 { 1.6 } else { 1.0 }
            * gfac
            * (0.6 + 0.4 * self.engine)
            * if empty { 0.0 } else { 1.0 };
        if self.air > 0.0 {
            self.speed -= DECEL * 0.15 * dt;
        } else if turbo_on {
            self.speed += acc * 2.5 * dt;
        } else if inp.accel {
            self.speed += acc * dt;
        } else if inp.brake {
            self.speed -= BRAKE * (0.25 + 0.75 * eg) * dt;
        } else {
            self.speed -= DECEL * dt;
        }
        if inp.turbo
            && !self.prev_turbo
            && self.turbo > 0
            && !turbo_on
            && self.speed > MAX_SPEED * 0.3
        {
            self.turbo -= 1;
            self.turbo_t = 2.5;
            self.turbos += 1;
        }
        self.prev_turbo = inp.turbo;
        // heat: the turbo overheats and runs out sooner
        let burn = if self.weather == crate::env::Weather::Heatwave {
            1.4
        } else {
            1.0
        };
        self.turbo_t = (self.turbo_t - dt * burn).max(0.0);

        // walls, rails and rock faces of structures
        let lim = feat_limit(feat);
        if lim < 5.0 && self.air < 0.3 {
            let m = lim - 0.10 * self.car.width.min(1.3);
            if self.x.abs() > m {
                let side = self.x.signum();
                self.x = side * m;
                self.lat_v = -side * self.lat_v.abs() * 0.3;
                let prot = self.shield_t > 0.0 || self.invuln > 0.0;
                if !prot && !zen {
                    if self.wall_cd <= 0.0 {
                        self.wall_cd = 0.45;
                        let hard = self.speed > MAX_SPEED * 0.5;
                        self.speed *= if hard { 0.78 } else { 0.9 };
                        self.shake = 0.4;
                        if hard {
                            self.crashes += 1;
                            self.hurt(0.06, 0.0);
                            self.banner = "SCRAPE";
                            self.banner_t = 0.8;
                        }
                        self.burst_sparks();
                    } else {
                        self.speed = (self.speed - MAX_SPEED * 0.3 * dt).max(0.0);
                    }
                }
            }
        }
        if (self.x < -1.0 || self.x > 1.0)
            && self.speed > OFFROAD_LIMIT
            && !self.in_lot()
            && !self.car.class.is_heli()
        {
            self.speed -= OFFROAD_DECEL * dt;
        }
        self.x = self.x.clamp(-2.2, 2.2);
        self.speed = self.speed.clamp(0.0, top);

        self.pos += self.speed * dt;
        // ghost recording: one sample every STEP seconds, the first one at the start
        if self.ghost_rec.is_empty() {
            self.ghost_rec.push((old_pos, self.x));
        }
        self.ghost_acc += dt;
        while self.ghost_acc >= crate::ghost::STEP {
            self.ghost_acc -= crate::ghost::STEP;
            if self.ghost_rec.len() < crate::ghost::MAX_SAMPLES - 1 {
                self.ghost_rec.push((self.pos, self.x));
            }
        }
        // tyre wear: faster in the wrong set and in bends
        if !zen {
            let mism = if self.tyre == crate::env::Tyre::best_for(self.weather, self.surface) {
                1.0
            } else {
                1.6
            };
            self.tyre_wear =
                (self.tyre_wear - self.speed * dt / (self.track.length * 1.4) * mism).max(0.0);
        }
        // level crossings: a train blocks the road for a few seconds
        {
            let (old_pz, new_pz) = (old_pos + PLAYER_Z, self.pos + PLAYER_Z);
            let prot = self.shield_t > 0.0 || self.invuln > 0.0;
            for i in 0..self.crossings.len() {
                let c = self.crossings[i];
                let dz = c.z - new_pz;
                if dz > 0.0
                    && dz < 9000.0
                    && c.warning(self.elapsed)
                    && self.banner_t < 0.2
                    && self.air == 0.0
                {
                    self.banner = "TRAIN!";
                    self.banner_t = 0.6;
                }
                if c.z >= old_pz && c.z < new_pz.max(old_pz + 1.0) && self.low() {
                    if c.train_on(self.elapsed) {
                        if !prot && !zen {
                            self.speed = 0.0;
                            self.time -= 4.0;
                            self.shake = 1.0;
                            self.invuln = 1.5;
                            self.crashes += 1;
                            self.hurt(0.35, 0.25);
                            self.burst_sparks();
                            self.banner = "CRASH";
                            self.banner_t = 1.4;
                        }
                    } else if c.t(self.elapsed) > c.period - 1.0 {
                        self.close_calls += 1;
                        self.score += 300;
                        self.banner = "CLOSE CALL";
                        self.banner_t = 1.0;
                    }
                }
            }
        }
        self.city_update(old_pos, dt, zen);
        self.emerg_update(old_pos, dt, zen);
        self.mission_update(dt);
        self.pursuit_update(dt);
        self.pack_update(dt);
        self.area_update(old_pos, dt, zen);
        self.x = self.x.clamp(-2.2, 2.2);
        // fuel
        if fuel_on {
            let range = self.fuel_range();
            let low = self.low_fuel_level();
            let burn = if turbo_on { 1.6 } else { 1.0 };
            if !self.demo {
                self.fuel = (self.fuel - self.speed * dt / range * burn).max(0.0);
            }
            let pzf = self.pos + PLAYER_Z;
            let in_st = self.stations.iter().any(|&(a, b)| pzf >= a && pzf < b);
            self.refueling =
                in_st && self.speed < MAX_SPEED * 0.3 && self.x > 0.2 && self.air == 0.0;
            if self.refueling {
                self.fuel = (self.fuel + dt * 0.30).min(1.0);
                if self.body < 1.0 || self.engine < 1.0 {
                    self.body = (self.body + dt * 0.12).min(1.0);
                    self.engine = (self.engine + dt * 0.12).min(1.0);
                    if self.banner_t < 0.2 {
                        self.banner = "REPAIR";
                        self.banner_t = 0.4;
                    }
                }
                // service: fresh tyres of the chosen (or the best) kind
                let want = self
                    .tyre_pref
                    .unwrap_or_else(|| crate::env::Tyre::best_for(self.weather, self.surface));
                if self.tyre != want || self.tyre_wear < 0.95 {
                    self.tyre = want;
                    self.tyre_wear = 1.0;
                    self.banner = "SERVICE";
                    self.banner_t = 1.0;
                }
                if self.banner_t < 0.2 {
                    self.banner = "REFUEL";
                    self.banner_t = 0.4;
                }
            }
            if self.fuel < low && !self.fuel_warned && self.fuel > 0.0 {
                self.fuel_warned = true;
                self.banner = "LOW FUEL";
                self.banner_t = 2.0;
            }
            if self.fuel > low + 0.2 {
                self.fuel_warned = false;
            }
            if self.fuel <= 0.0 && self.banner_t < 0.2 {
                self.banner = "NO FUEL";
                self.banner_t = 1.0;
            }
            // stranded: after a few seconds a tow truck brings a quarter of a tank, at a cost in time
            if self.fuel <= 0.0 && self.speed < MAX_SPEED * 0.02 {
                self.dry_t += dt;
                if self.dry_t > 6.0 {
                    self.dry_t = 0.0;
                    self.fuel = 0.25;
                    match self.mode {
                        Mode::TimeTrial => self.time += 20.0,
                        _ => self.time -= 8.0,
                    }
                    self.banner = "REFUEL";
                    self.banner_t = 1.2;
                }
            } else {
                self.dry_t = 0.0;
            }
        }
        // jumps
        if self.air > 0.0 || self.air_v > 0.0 {
            self.lat_v = 0.0;
            self.air_v -= 14.0 * dt;
            self.air += self.air_v * dt;
            if self.air <= 0.0 {
                let imp = -self.air_v;
                let sz = self.air_size as u32;
                self.air = 0.0;
                self.air_v = 0.0;
                self.air_size = 0;
                self.score += 100 * sz * if self.x2_t > 0.0 { 2 } else { 1 };
                if imp > 9.0 && self.shield_t <= 0.0 {
                    self.speed *= 0.88;
                    self.shake = 0.5;
                }
                if sz >= 2 {
                    self.banner = "LAND";
                    self.banner_t = 0.8;
                }
            }
        }
        self.score += (self.speed * dt * 0.01) as u32;
        self.sky_off += cur * (self.pos - old_pos) / SEG_LEN * 0.25;

        // roadside collisions over every segment crossed this frame
        let mut spark = false;
        if self.invuln <= 0.0 && self.shield_t <= 0.0 && !zen && self.low() {
            let a = ((old_pos + PLAYER_Z) / SEG_LEN) as usize;
            let b = ((self.pos + PLAYER_Z) / SEG_LEN) as usize;
            'hit: for k in a..=b {
                for p in &self.track.segs[k % n].props {
                    if p.kind.solid() && (self.x - p.offset).abs() < 0.10 + 0.13 * self.car.width {
                        self.speed = self
                            .speed
                            .min(MAX_SPEED * 0.12 * self.car.prop_keep * self.level.params().3);
                        self.shake = 0.5;
                        self.invuln = 0.7;
                        self.x += if self.x >= p.offset { 0.25 } else { -0.25 };
                        spark = true;
                        break 'hit;
                    }
                }
            }
        }

        // traffic
        let pz = self.pos + PLAYER_Z;
        let snapshot: Vec<(f32, f32, f32)> =
            self.cars.iter().map(|c| (c.z, c.x, c.speed)).collect();
        let priv_ = self.privileged();
        let low = self.low();
        let px = self.x;
        for i in 0..self.cars.len() {
            let mut spd = self.cars[i].base * if self.freeze_t > 0.0 { 0.25 } else { 1.0 };
            for (j, &(zj, xj, sj)) in snapshot.iter().enumerate() {
                if i == j {
                    continue;
                }
                let d = wrap(zj - self.cars[i].z, len);
                if d > 0.0 && d < SEG_LEN * 6.0 && (xj - self.cars[i].x).abs() < 0.3 && sj < spd {
                    spd = sj;
                }
            }
            let car = &mut self.cars[i];
            car.speed = spd;
            car.z = (car.z + spd * dt).rem_euclid(len);
            // traffic gives way to emergency vehicles
            if priv_ {
                let d = wrap(car.z - pz, len);
                if d > 0.0 && d < 6000.0 && (car.x - px).abs() < 0.55 {
                    let dir = if car.x >= px { 1.0 } else { -1.0 };
                    car.x = (car.x + dir * 1.2 * dt).clamp(-1.0, 1.0);
                }
            }
            // regular traffic takes other roads around a jam
            if let Some(ji) = self.city.jam_at((car.z / SEG_LEN) as usize) {
                car.z = (self.city.jams[ji].b as f32 * SEG_LEN + 600.0).rem_euclid(len);
            }

            let dz = wrap(car.z - pz, len);
            if car.prev_dz > 0.0 && dz <= 0.0 && car.prev_dz.abs() < len / 4.0 {
                self.overtaken += 1;
                self.score += if self.x2_t > 0.0 { 200 } else { 100 };
                if !self.city.in_city((self.pos / SEG_LEN) as usize) && self.mode != Mode::Zen {
                    self.chain = (self.chain + 1).min(7);
                    self.chain_t = 5.0;
                    self.chain_best = self.chain_best.max(self.chain);
                    let bonus = (100u32 << self.chain).min(CHAIN_CAP);
                    self.score += if self.x2_t > 0.0 { bonus * 2 } else { bonus };
                    if self.chain >= 3 && self.banner_t <= 0.0 {
                        self.banner = "CHAIN";
                        self.banner_t = 0.8;
                    }
                }
                if self.mode == Mode::Rush {
                    self.time += 0.8;
                }
            }
            car.prev_dz = dz;
            if self.invuln <= 0.0
                && self.shield_t <= 0.0
                && low
                && dz.abs() < SEG_LEN * 0.8
                && (self.x - car.x).abs() < kind_half(car.kind) + 0.13 * self.car.width
                && self.speed > car.speed
            {
                self.speed = (car.speed * 0.6 * self.car.crash_keep * self.level.params().3)
                    .min(self.speed * 0.95);
                self.shake = 0.5;
                self.invuln = 0.6;
                self.x += if self.x >= car.x { 0.2 } else { -0.2 };
                spark = true;
            }
        }
        if self.crashes != self.chain_seen {
            self.chain_seen = self.crashes;
            self.chain = 0;
        }
        self.chain_t -= dt;
        if self.chain_t <= 0.0 {
            self.chain = 0;
        }
        // collision bumps must not push the car beyond the playable width
        self.x = self.x.clamp(-2.2, 2.2);
        if spark {
            self.crashes += 1;
            self.hurt(0.1, 0.06);
            self.burst_sparks();
        }
        self.spawn_fx();

        // animals crossing the road
        {
            let old_pz = old_pos + PLAYER_Z;
            let prot = self.shield_t > 0.0 || self.invuln > 0.0;
            for i in 0..self.animals.len() {
                let mut a = self.animals[i];
                if a.done {
                    continue;
                }
                let (vx, half) = species_params(a.species);
                let dz = a.z - pz;
                if a.hit {
                    a.hit_t += dt;
                    a.x += a.dir * 2.5 * dt;
                    if a.hit_t > 0.9 {
                        a.done = true;
                    }
                    self.animals[i] = a;
                    continue;
                }
                if !a.active && dz > 0.0 && dz < self.speed.max(MAX_SPEED * 0.45) * a.lead {
                    a.active = true;
                    if self.banner_t < 0.3 && self.air == 0.0 {
                        self.banner = "ANIMAL!";
                        self.banner_t = 1.0;
                    }
                }
                if a.active {
                    a.x += a.dir * vx * dt;
                    if a.x.abs() > 1.55 {
                        a.done = true;
                    }
                }
                if a.active
                    && !a.passed
                    && a.z >= old_pz - SEG_LEN * 0.5
                    && a.z <= pz + SEG_LEN * 0.5
                {
                    a.passed = true;
                    if self.low()
                        && !prot
                        && !zen
                        && (self.x - a.x).abs() < half + 0.09 * self.car.width
                    {
                        a.hit = true;
                        a.hit_t = 0.0;
                        self.animals_hit += 1;
                        self.speed =
                            (self.speed * 0.25 * self.car.crash_keep * self.level.params().3)
                                .min(self.speed * 0.95);
                        self.time -= 3.0;
                        self.shake = 0.9;
                        self.invuln = 0.8;
                        self.crashes += 1;
                        self.hurt(0.12, 0.04);
                        self.burst_sparks();
                        self.banner = "CRASH";
                        self.banner_t = 1.2;
                    } else if !a.done {
                        self.animals_dodged += 1;
                        self.score += if self.x2_t > 0.0 { 400 } else { 200 };
                    }
                }
                self.animals[i] = a;
            }
        }

        // police: chase, ram, overtake and block; being caught means disqualification
        if !zen && self.mode != Mode::Zen && !self.privileged() {
            let li = self.level.index();
            let (_, gap, maxc) = self.cop_params_r();
            if self.cops.len() < maxc && self.elapsed >= self.next_cop {
                if pz < 9500.0 {
                    self.next_cop = self.elapsed + 3.0;
                } else {
                    self.cops.push(Cop {
                        z: pz - 9000.0,
                        x: 0.0,
                        speed: self.speed,
                        age: 0.0,
                        mode: 0,
                    });
                    self.next_cop = self.elapsed + gap;
                    self.banner = "POLICE!";
                    self.banner_t = 1.6;
                }
            }
            let slow = if self.freeze_t > 0.0 { 0.25 } else { 1.0 };
            let cap = MAX_SPEED * 0.97 * (1.0 + 0.03 * li as f32);
            let mut pressure = 0.0f32;
            let mut contact = None;
            for c in &mut self.cops {
                c.age += dt;
                let dz = c.z - pz;
                let want = if c.mode == 0 {
                    if dz < -2500.0 {
                        self.speed + 0.25 * MAX_SPEED
                    } else if dz < -400.0 {
                        self.speed + 0.12 * MAX_SPEED
                    } else {
                        self.speed + 0.03 * MAX_SPEED
                    }
                } else if dz < 2600.0 {
                    self.speed + 0.2 * MAX_SPEED
                } else {
                    self.speed * 0.55
                };
                let want = want.min(cap).max(0.0) * slow;
                c.speed += (want - c.speed) * (dt * 2.0).min(1.0);
                let rate = if c.mode == 0 { 0.9 } else { 0.8 };
                c.x += (self.x - c.x).clamp(-1.0, 1.0) * rate * dt;
                c.z += c.speed * dt;
                if c.mode == 0 && c.age > 14.0 && dz > -900.0 {
                    c.mode = 1;
                }
                let d = (c.z - pz).abs();
                if d < SEG_LEN * 10.0 && (c.x - self.x).abs() < 0.40 {
                    pressure += 0.04 + 0.08 * (1.0 - d / (SEG_LEN * 10.0));
                }
                if d < SEG_LEN * 0.9 && (c.x - self.x).abs() < 0.17 + 0.1 * self.car.width {
                    contact = Some(c.x);
                }
            }
            self.cops.retain(|c| {
                let dz = c.z - pz;
                c.age < 34.0 && dz > -16000.0 && dz < 16000.0
            });
            if let Some(cx) = contact {
                if self.invuln <= 0.0 && self.shield_t <= 0.0 && self.low() {
                    self.speed *= 0.82;
                    self.x = (self.x + if self.x >= cx { 0.2 } else { -0.2 }).clamp(-2.2, 2.2);
                    self.shake = 0.6;
                    self.invuln = 0.5;
                    self.bust += 0.15 * self.level.params().3;
                    self.crashes += 1;
                    self.hurt(0.08, 0.03);
                    self.burst_sparks();
                }
            }
            if self.shield_t > 0.0 || !self.low() {
                pressure *= 0.5;
            }
            // the police are slower to close in on the easier levels
            pressure *= 0.5 * self.level.params().3;
            if pressure > 0.0 {
                self.bust += pressure * dt;
            } else {
                self.bust -= 0.25 * dt;
            }
            self.bust = self.bust.clamp(0.0, if self.demo { 0.95 } else { 1.0 });
            if self.bust >= 1.0 {
                self.state = State::Disqualified;
                self.banner = "BUSTED";
                self.banner_t = 3.0;
                self.cops.clear();
            }
        }

        // orbs and slalom rings crossed this frame
        if !self.orbs.is_empty() {
            let (za, zb) = (old_pos + PLAYER_Z, self.pos + PLAYER_Z);
            for i in 0..self.orbs.len() {
                let o = self.orbs[i];
                if o.taken || o.z < za || o.z >= zb.max(za + 1.0) {
                    continue;
                }
                self.orbs[i].taken = true;
                if o.kind == 0 {
                    if (self.x - o.x).abs() < 0.3 {
                        self.collected += 1;
                        self.combo += 1;
                        self.time += 0.7;
                        self.score += 250 + 25 * self.combo.min(20);
                    } else {
                        self.combo = 0;
                    }
                } else if (self.x - o.x).abs() < 0.42 {
                    self.combo += 1;
                    self.collected += 1;
                    self.time += 0.5;
                    self.score += 200 * self.combo.min(10);
                    self.banner = "RING";
                    self.banner_t = 0.6;
                } else {
                    self.combo = 0;
                    self.time -= 1.0;
                }
            }
        }

        // obstacles and bonuses crossed this frame
        if !self.items.is_empty() {
            let (za, zb) = (old_pos + PLAYER_Z, self.pos + PLAYER_Z);
            let zb = zb.max(za + 1.0);
            let first = self.items.partition_point(|i| i.z < za);
            let half = 0.065 * self.car.width;
            for i in first..self.items.len() {
                let it = self.items[i];
                if it.z >= zb {
                    break;
                }
                if it.taken || (self.air > 0.45 && !is_bonus(it.kind)) {
                    continue;
                }
                let hw = match it.kind {
                    I_BARRIER | I_CRACK | I_MAGMA | I_RAMP_S | I_RAMP_M | I_RAMP_L | I_PAD => 0.30,
                    I_OIL | I_PUDDLE | I_MUD => 0.26,
                    k if is_bonus(k) => 0.26,
                    _ => 0.17,
                };
                if (self.x - it.x).abs() < hw + half {
                    self.items[i].taken = true;
                    self.apply_item(it.kind);
                }
            }
        }

        // thin ice: keep moving or it breaks
        if self.surface == crate::env::Surface::IceLake && !zen && !self.thin.is_empty() {
            let pzn = self.pos + PLAYER_Z;
            let on = self.thin.iter().any(|&(a, b)| pzn >= a && pzn < b);
            if on && self.speed < MAX_SPEED * 0.35 && self.shield_t <= 0.0 {
                self.ice_t += dt;
                if self.ice_t > 0.6 {
                    self.banner = "KEEP MOVING";
                    self.banner_t = 0.3;
                }
                if self.ice_t > 1.5 {
                    self.ice_t = 0.0;
                    self.speed = 0.0;
                    self.time -= 4.0;
                    self.shake = 0.9;
                    self.invuln = 1.2;
                    self.crashes += 1;
                    self.hurt(0.15, 0.1);
                    self.burst_sparks();
                    self.banner = "ICE BREAKS";
                    self.banner_t = 1.6;
                }
            } else {
                self.ice_t = (self.ice_t - dt * 2.0).max(0.0);
            }
        } else {
            self.ice_t = 0.0;
        }

        // checkpoints, finish, clock
        if let Some(&cp) = self.track.checkpoints.get(self.next_cp) {
            if self.pos >= cp as f32 * SEG_LEN {
                if !zen {
                    self.time += CP_BONUS * self.clock_mult();
                }
                self.next_cp += 1;
                self.cps += 1;
                self.banner = "CHECKPOINT";
                self.banner_bonus = if zen {
                    0
                } else {
                    (CP_BONUS * self.clock_mult()).round() as u32
                };
                self.banner_t = 2.0;
                self.score += 1000;
            }
        }
        if self.pos >= self.track.finish_seg as f32 * SEG_LEN {
            if matches!(self.mode, Mode::Endless | Mode::Zen) {
                self.next_lap();
                return;
            }
            self.state = State::Finished;
            self.score += if self.mode == Mode::TimeTrial {
                ((300.0 - self.time).max(0.0) * 100.0) as u32
            } else {
                (self.time.max(0.0) * 100.0) as u32
            };
            self.banner = "STAGE CLEAR!";
            self.banner_t = 99.0;
            if !self.pack.is_empty() {
                self.place = crate::pack::position(&self.pack, self.pos + PLAYER_Z);
                let b = crate::pack::podium_bonus(self.place);
                if b > 0 {
                    self.score += b;
                    self.banner = "PODIUM!";
                }
            }
            return;
        }
        match self.mode {
            Mode::TimeTrial => self.time += dt,
            Mode::Zen => {}
            _ => {
                // the countdown waits while the car is being serviced at a station
                let servicing = self.refueling
                    && (self.fuel < 0.995 || self.body < 0.995 || self.engine < 0.995);
                if !servicing {
                    self.time -= dt;
                }
                if self.demo && self.time < 2.0 {
                    self.time = 2.0;
                }
                if self.time <= 0.0 {
                    self.time = 0.0;
                    self.state = State::TimeUp;
                }
            }
        }
    }

    /// Fuel stop planning for the demo pilot. Returns the input to use while approaching or standing in a
    /// station it has decided to use, `None` when no stop is needed (or not yet).
    fn ace_refuel(&self, pz: f32, brake_traffic: bool) -> Option<Input> {
        let dist_per_fuel = self.fuel_range() / 1.8;
        let finish = self.track.finish_seg as f32 * SEG_LEN;
        let k = self.stations.iter().position(|&(_, b)| b > pz)?;
        let (a, b) = self.stations[k];
        let next_after = self.stations.get(k + 1).map_or(finish, |n| n.0);
        let inside = pz >= a;
        // fill to what the rest of the stage needs, or to the brim when more stops will follow
        let want = if (finish - a) / dist_per_fuel + 0.06 < 1.0 {
            ((finish - a) / dist_per_fuel + 0.06).min(1.0)
        } else {
            1.0
        };
        let reach = self.fuel * dist_per_fuel;
        let must = reach < (next_after - pz) + 4500.0;
        if inside {
            if self.fuel >= want - 0.005 || (!must && self.fuel > 0.5) {
                return None;
            }
        } else {
            if !must || a - pz > 26000.0 {
                return None;
            }
            // brake in time to enter the station at refuelling speed
            let v_in = MAX_SPEED * 0.27;
            let d = a - pz;
            let need =
                ((self.speed * self.speed - v_in * v_in).max(0.0)) / (2.0 * BRAKE * 0.6) + 500.0;
            let target = 0.55;
            let err = target - self.x;
            return Some(Input {
                left: err < -0.04,
                right: err > 0.04,
                accel: if d > need {
                    !brake_traffic
                } else {
                    self.speed < v_in * 0.85
                },
                brake: d <= need && self.speed > v_in,
                turbo: false,
            });
        }
        // inside the zone: roll at the speed that tops the tank up by the end of it
        let t_need = ((want - self.fuel) / 0.30).max(0.05);
        let v_des = ((b - pz) / t_need).clamp(300.0, MAX_SPEED * 0.27);
        let err = 0.55 - self.x;
        Some(Input {
            left: err < -0.04,
            right: err > 0.04,
            accel: self.speed
                < if self.x < 0.3 {
                    v_des.max(2200.0)
                } else {
                    v_des * 0.9
                },
            brake: self.speed
                > if self.x < 0.3 {
                    v_des.max(2600.0)
                } else {
                    v_des
                },
            turbo: false,
        })
    }

    /// True while the demo pilot is on a fuel stop (approaching or standing in a station).
    pub(crate) fn ace_refuelling(&self) -> bool {
        self.fuel_rule
            && self.mode != Mode::Zen
            && self.ace_refuel(self.pos + PLAYER_Z, false).is_some()
    }

    /// The banner text with its extra (the checkpoint's bonus seconds).
    pub fn banner_text(&self) -> String {
        let b = crate::i18n::banner(self.banner);
        if self.banner == "CHECKPOINT" && self.banner_bonus > 0 {
            format!("{b} +{}", self.banner_bonus)
        } else {
            b.to_string()
        }
    }

    /// Simple autopilot (steers around traffic, brakes behind slow cars); used by the CPU opponent and the tools.
    pub fn autopilot(&self) -> Input {
        self.drive(None, true, false)
    }

    /// The autopilot's rule set with the lane choice exposed: `lane` fixes the lateral target (None = the
    /// built-in traffic dodging), `yield_traffic` lets it brake behind slower cars, `ace` drives at the
    /// limit (full-throttle turbo use, tighter margins in cities). Used by the demo pilot, which picks lanes by
    /// simulating a few seconds ahead.
    pub fn drive(&self, lane: Option<f32>, yield_traffic: bool, ace: bool) -> Input {
        let mut i = self.drive_raw(lane, yield_traffic, ace);
        // Mirror mode swaps left and right for part of every cycle: steer the other way then
        if self.mode == Mode::Mirror && (self.elapsed + 1.0 / 60.0) % 14.0 > 8.0 {
            std::mem::swap(&mut i.left, &mut i.right);
        }
        i
    }

    fn drive_raw(&self, lane: Option<f32>, yield_traffic: bool, ace: bool) -> Input {
        let mut target = lane.unwrap_or(0.0);
        let pz = self.pos + PLAYER_Z;
        let len = self.track.length;
        let mut brake = false;
        for c in &self.cars {
            let d = (c.z - pz).rem_euclid(len);
            if d < 4000.0 && (c.x - self.x).abs() < 0.4 {
                if lane.is_none() {
                    target = if c.x > 0.0 { -0.66 } else { 0.66 };
                }
                if d < 1200.0 && self.speed > c.speed * 1.1 && (lane.is_none() || yield_traffic) {
                    brake = true;
                }
            }
        }
        // the demo pilot plans its fuel stops: as few as possible, at the last station it can reach
        if ace && self.fuel_rule && self.mode != Mode::Zen {
            if let Some(inp) = self.ace_refuel(pz, brake) {
                return inp;
            }
        }
        // low on fuel: pull into the next station and wait until the tank is nearly full
        if !ace
            && self.fuel_rule
            && self.mode != Mode::Zen
            && self.fuel < self.low_fuel_level() + 0.05
        {
            if let Some(&(a, b)) = self.stations.iter().find(|&&(_, b)| b > pz) {
                let d = a - pz;
                if d < 9000.0 {
                    target = 0.5;
                    if (d < 2500.0 && self.speed > MAX_SPEED * 0.28)
                        || (pz >= a && pz < b && self.fuel < 0.95 && self.speed > 400.0)
                    {
                        brake = true;
                    }
                    if pz >= a && pz < b && self.fuel < 0.95 {
                        let err = target - self.x;
                        return Input {
                            left: err < -0.04,
                            right: err > 0.04,
                            accel: false,
                            brake,
                            turbo: false,
                        };
                    }
                }
            }
        }
        // cities: keep to the limit, stop at red lights and for pedestrians
        let seg = (pz / SEG_LEN) as usize % self.track.segs.len();
        let mut hold = false;
        if self.mode != Mode::Zen {
            let lim = MAX_SPEED * city::LIMIT * if ace { 0.985 } else { 0.93 };
            let soon = self
                .city
                .zones
                .iter()
                .any(|z| seg < z.a && (z.a - seg) as f32 * SEG_LEN < 14000.0);
            if (self.city.in_city(seg) || soon) && self.speed > lim {
                brake = true;
            }
            if let Some(l) = self
                .city
                .lights
                .iter()
                .find(|l| l.z > pz - 150.0 && l.z - pz < 6000.0)
            {
                let d = l.z - pz;
                let stop = l.ped_x(self.elapsed).is_some()
                    || (l.lit && (l.state(self.elapsed) == 2 || (l.state(self.elapsed) == 1 && d > 1800.0)))
                    // light about to turn: do not enter on a late green
                    || (l.lit && l.state(self.elapsed) == 0 && d > 2500.0 && {
                        let t = (self.elapsed + l.phase).rem_euclid(city::CYCLE);
                        city::GREEN - t < d / self.speed.max(2500.0)
                    });
                if stop {
                    let v_ok = ((d - 250.0).max(0.0)) * 2.2;
                    if self.speed > v_ok {
                        brake = true;
                    } else {
                        hold = true;
                    }
                }
            }
        }
        // a jam ahead: take the exit
        let mut exit_x = None;
        if self.detour.is_none() {
            if let Some((_, j)) = self.city.next_jam(seg) {
                let ea = j.exit_a() as f32 * SEG_LEN;
                if pz > ea - 5000.0 - if ace { self.speed * 0.9 } else { 0.0 }
                    && seg < j.a
                    && self.detour.is_none()
                    && !self.demo
                {
                    exit_x = Some(1.4);
                }
            }
        }
        // keep the road centred through the generated curves
        if lane.is_none() {
            let cur = self.track.seg_at(self.pos + PLAYER_Z * 3.0).curve;
            target = (target - cur * 0.05).clamp(-0.8, 0.8);
        }
        if let Some(x) = exit_x {
            target = x;
        }
        let err = target - self.x;
        Input {
            left: err < -0.04,
            right: err > 0.04,
            accel: !brake && !hold,
            brake,
            turbo: self.turbo > 0
                && self.pos > 20000.0
                && (ace || self.elapsed as i32 % 14 == 6)
                && !self.city.in_city(seg)
                && self
                    .city
                    .next_jam(seg)
                    .is_none_or(|(_, j)| j.exit_a() > seg + 400),
        }
    }
}
