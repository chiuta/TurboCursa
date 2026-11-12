//! Minimal C-ABI surface for the browser (no wasm-bindgen needed).
use crate::audio::{AudioParams, Synth};
use crate::career_ui::CareerApp;
use crate::game::*;
use crate::profile::{GameResult, Profile};
use crate::versus::Versus;
use crate::{render, H, W};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

enum Mode {
    Single(Box<Game>),
    Versus(Box<Versus>),
    Multi(Box<crate::multi::Multi>),
    Career(Box<CareerApp>),
    Pick(Box<crate::pick::PickApp>),
    Rally(Box<crate::rally::RallyApp>),
    Design(Box<crate::design::DesignApp>),
}

struct App {
    mode: Mode,
    fb: Vec<u32>,
    rgba: Vec<u8>,
    inp: [Input; 2],
    restart: bool,
    back: bool,
    seed: u64,
    /// the `init` mode code, kept so the selection screen can start the right game
    code: u32,
    save_buf: Vec<u8>,
    synth: Option<Synth>,
    aparams: AudioParams,
    abuf: Vec<f32>,
    /// seconds of play since the last break
    session: f32,
    /// the running game is already booked into the profile
    counted: bool,
    /// previous gear bits (edge detection)
    gear_prev: u32,
    gear_bits: u32,
}

static APP: Mutex<Option<App>> = Mutex::new(None);
/// Chosen country car (index into the country table); u32::MAX = the neutral default car.
static CAR: AtomicU32 = AtomicU32::new(u32::MAX);

/// Chosen vehicle class (0..5) and difficulty level (0..5; default 1 = normal).
static CLASS: AtomicU32 = AtomicU32::new(0);
static LEVEL: AtomicU32 = AtomicU32::new(1);

fn level() -> crate::game::Difficulty {
    crate::game::Difficulty::from_index(LEVEL.load(Ordering::Relaxed) as usize)
}

static WX: AtomicU32 = AtomicU32::new(0);
static SF: AtomicU32 = AtomicU32::new(0);

/// Weather / surface as menu indices (0 = automatic).
fn env_sel() -> (usize, usize) {
    (
        WX.load(Ordering::Relaxed) as usize,
        SF.load(Ordering::Relaxed) as usize,
    )
}

/// Weather for the next `init`: 0..10 clear, rain, storm, fog, snow, blizzard, sandstorm, hail,
/// glaze, heatwave, ash; anything else = automatic.
#[no_mangle]
pub extern "C" fn set_weather(i: u32) {
    let v = if (i as usize) < crate::env::WEATHER_COUNT {
        i + 1
    } else {
        0
    };
    WX.store(v, Ordering::Relaxed);
}

/// Surface for the next `init`: 0..5 asphalt, frozen lake, salt flat, lava field, dunes, bog;
/// anything else = automatic.
#[no_mangle]
pub extern "C" fn set_surface(i: u32) {
    let v = if (i as usize) < crate::env::SURFACE_COUNT {
        i + 1
    } else {
        0
    };
    SF.store(v, Ordering::Relaxed);
}

static ASSIST: AtomicU32 = AtomicU32::new(0);
static SLOW: AtomicU32 = AtomicU32::new(100);

/// Colour filter: 0 off, 1 protanopia, 2 deuteranopia, 3 tritanopia, 4 high contrast.
#[no_mangle]
pub extern "C" fn set_filter(i: u32) {
    crate::access::set_filter(i as usize);
}

/// Driving assist 0..=3 for the next `init`.
#[no_mangle]
pub extern "C" fn set_assist(i: u32) {
    ASSIST.store(i.min(crate::access::ASSIST_MAX as u32), Ordering::Relaxed);
}

/// Top-speed percentage 40..=100 for the next `init`.
#[no_mangle]
pub extern "C" fn set_slow(pct: u32) {
    SLOW.store(pct.clamp(40, 100), Ordering::Relaxed);
}

fn access() -> (u8, f32) {
    (
        ASSIST.load(Ordering::Relaxed) as u8,
        SLOW.load(Ordering::Relaxed) as f32 / 100.0,
    )
}

static TIME: AtomicU32 = AtomicU32::new(0);
static TYRE: AtomicU32 = AtomicU32::new(0);
static SETUP: AtomicU32 = AtomicU32::new(0);
/// u32::MAX = the rank comes from the profile
static RANK: AtomicU32 = AtomicU32::new(u32::MAX);
static TODAY: AtomicU32 = AtomicU32::new(0);
/// Garage (coins and upgrades): the page stores `garage_save` text and gives it back by `garage_load`.
static GARAGE: Mutex<crate::garage::Garage> = Mutex::new(crate::garage::Garage {
    coins: 0,
    lv: [0; 4],
});
static GARAGE_DIRTY: AtomicU32 = AtomicU32::new(0);

/// Garage text in the scratch buffer (`n` bytes); returns the coins.
#[no_mangle]
pub extern "C" fn garage_load(n: u32) -> u32 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let n = (n as usize).min(a.save_buf.len());
    let g = crate::garage::Garage::decode(&String::from_utf8_lossy(&a.save_buf[..n]));
    g.apply();
    *GARAGE.lock().unwrap() = g;
    g.coins
}

/// The garage as text (empty and 0 when nothing changed since the last call).
#[no_mangle]
pub extern "C" fn garage_save() -> u32 {
    if GARAGE_DIRTY.swap(0, Ordering::Relaxed) == 0 {
        return 0;
    }
    let txt = GARAGE.lock().unwrap().encode();
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    a.save_buf = txt.into_bytes();
    a.save_buf.len() as u32
}

/// Buys the next level of upgrade `i` (0 engine, 1 gearbox, 2 tyres, 3 turbo): 1 when bought.
#[no_mangle]
pub extern "C" fn garage_buy(i: u32) -> u32 {
    let mut g = *GARAGE.lock().unwrap();
    let ok = g.buy(i as usize);
    if ok {
        g.apply();
        *GARAGE.lock().unwrap() = g;
        GARAGE_DIRTY.store(1, Ordering::Relaxed);
    }
    ok as u32
}

/// `what`: 0 coins, 1..=4 level of upgrade 0..3, 5..=8 price of the next level (0 = maxed).
#[no_mangle]
pub extern "C" fn garage_info(what: u32) -> u32 {
    let g = *GARAGE.lock().unwrap();
    match what {
        0 => g.coins,
        1..=4 => g.lv[what as usize - 1] as u32,
        5..=8 => g.price(what as usize - 5).unwrap_or(0),
        _ => 0,
    }
}

static PROFILE: Mutex<Option<Profile>> = Mutex::new(None);
static PROFILE_DIRTY: AtomicU32 = AtomicU32::new(0);
/// badges earned by the last finished game, not yet announced to the page
/// day number of the daily challenge being played (0 = none) and its result code
static DAILY: AtomicU32 = AtomicU32::new(0);
static DAILY_CODE: Mutex<String> = Mutex::new(String::new());

/// Sets up the daily challenge of `day` (vehicle, level, weather, surface, time of day) for the next
/// `init` and returns its mode code. The page passes `Math.floor(Date.now()/864e5)`.
#[no_mangle]
pub extern "C" fn daily_setup(day: u32) -> u32 {
    let d = crate::daily::for_day(day);
    DAILY.store(day, Ordering::Relaxed);
    CLASS.store(d.class as u32, Ordering::Relaxed);
    LEVEL.store(d.level as u32, Ordering::Relaxed);
    WX.store(d.weather as u32, Ordering::Relaxed);
    SF.store(d.surface as u32, Ordering::Relaxed);
    TIME.store(d.time as u32, Ordering::Relaxed);
    d.mode
}

/// Sets up the weekly challenge of the week containing `day` (a modifier on top of a seeded stage);
/// it does not count as the daily. Returns the mode code; `weekly_seed` gives the seed.
#[no_mangle]
pub extern "C" fn weekly_setup(day: u32) -> u32 {
    let d = crate::daily::weekly(crate::daily::week_of(day));
    CLASS.store(d.class as u32, Ordering::Relaxed);
    LEVEL.store(d.level as u32, Ordering::Relaxed);
    WX.store(d.weather as u32, Ordering::Relaxed);
    SF.store(d.surface as u32, Ordering::Relaxed);
    TIME.store(d.time as u32, Ordering::Relaxed);
    d.mode
}

#[no_mangle]
pub extern "C" fn weekly_seed(day: u32) -> u32 {
    crate::daily::weekly(crate::daily::week_of(day)).seed as u32
}

/// Index of this week's modifier (0 night+fog, 1 black ice, 2 heavy vehicle, 3 insane, 4 mirrored,
/// 5 blackout).
#[no_mangle]
pub extern "C" fn weekly_modifier(day: u32) -> u32 {
    crate::daily::weekly_modifier(crate::daily::week_of(day)) as u32
}

/// Music style: 0 normal, 1 calm, 2 drive, 3 off.
#[no_mangle]
pub extern "C" fn set_music(i: u32) {
    crate::audio::set_music_style(i.min(255) as u8);
}

/// 1 = show speed in mph.
#[no_mangle]
pub extern "C" fn set_mph(on: u32) {
    let mut s = crate::settings::Settings::default();
    s.mph = on != 0;
    s.music = crate::audio::music_style();
    s.livery = crate::settings::livery();
    s.apply();
}

/// Secret paint code in the scratch buffer (`n` bytes): returns the paint job (1..) or 0.
#[no_mangle]
pub extern "C" fn secret_load(n: u32) -> u32 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let n = (n as usize).min(a.save_buf.len());
    let txt = String::from_utf8_lossy(&a.save_buf[..n]).to_string();
    let Some(l) = crate::settings::secret_code(&txt) else {
        return 0;
    };
    let s = crate::settings::Settings {
        music: crate::audio::music_style(),
        mph: crate::settings::mph(),
        keys: 0,
        livery: l,
    };
    s.apply();
    l as u32
}

/// Seed of the daily challenge of `day`.
#[no_mangle]
pub extern "C" fn daily_seed(day: u32) -> u32 {
    crate::daily::for_day(day).seed as u32
}

/// The result code of the daily run that just ended (first run of the day only), written into the
/// scratch buffer; returns its length, 0 when there is none.
#[no_mangle]
pub extern "C" fn daily_code() -> u32 {
    let txt = std::mem::take(&mut *DAILY_CODE.lock().unwrap());
    if txt.is_empty() {
        return 0;
    }
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    a.save_buf = txt.into_bytes();
    a.save_buf.len() as u32
}

/// The shared stage being played (`code_load` / the designer); `STAGE_NEXT` arms it for the next `init`.
static STAGE: Mutex<Option<crate::share::StageCode>> = Mutex::new(None);
/// A championship / tour code loaded by `code_load` (played in mode 12).
static CHAMP: Mutex<Option<crate::champ::ChampCode>> = Mutex::new(None);
static STAGE_NEXT: AtomicU32 = AtomicU32::new(0);

/// Stage code: the page puts a `TCS1-...` code into the scratch buffer (`n` bytes). Returns 0 when it
/// is malformed; otherwise the stage's vehicle, level and conditions are set and the next `init`
/// (seed from `code_seed`, any single-player mode) plays exactly that stage.
#[no_mangle]
pub extern "C" fn code_load(n: u32) -> u32 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let n = (n as usize).min(a.save_buf.len());
    let txt = String::from_utf8_lossy(&a.save_buf[..n]).to_string();
    if let Some(cc) = crate::champ::ChampCode::decode(&txt) {
        CLASS.store(cc.class as u32, Ordering::Relaxed);
        LEVEL.store(cc.level as u32, Ordering::Relaxed);
        *STAGE.lock().unwrap() = None;
        *CHAMP.lock().unwrap() = Some(cc);
        STAGE_NEXT.store(1, Ordering::Relaxed);
        return 1;
    }
    let Some(c) = crate::share::StageCode::decode(&txt) else {
        return 0;
    };
    *CHAMP.lock().unwrap() = None;
    CLASS.store(c.class as u32, Ordering::Relaxed);
    LEVEL.store(c.level as u32, Ordering::Relaxed);
    WX.store(c.weather as u32, Ordering::Relaxed);
    SF.store(c.surface as u32, Ordering::Relaxed);
    TIME.store(c.time as u32, Ordering::Relaxed);
    *STAGE.lock().unwrap() = Some(c);
    STAGE_NEXT.store(1, Ordering::Relaxed);
    1
}

/// The current selection as the page shows it: 0 class, 1 level, 2 weather menu index (0 = auto),
/// 3 surface menu index, 4 time of day.
#[no_mangle]
pub extern "C" fn cfg_get(i: u32) -> u32 {
    match i {
        0 => CLASS.load(Ordering::Relaxed),
        1 => LEVEL.load(Ordering::Relaxed),
        2 => WX.load(Ordering::Relaxed),
        3 => SF.load(Ordering::Relaxed),
        _ => TIME.load(Ordering::Relaxed),
    }
}

/// Seed of the loaded stage code (0 when none).
#[no_mangle]
pub extern "C" fn code_seed() -> u32 {
    if let Some(c) = *CHAMP.lock().unwrap() {
        return c.seed;
    }
    STAGE.lock().unwrap().map_or(0, |c| c.seed)
}

/// Game mode of the loaded stage code.
#[no_mangle]
pub extern "C" fn code_mode() -> u32 {
    if CHAMP.lock().unwrap().is_some() {
        return 12;
    }
    STAGE.lock().unwrap().map_or(0, |c| c.mode as u32)
}

/// The code of the stage being played or designed, as text in the scratch buffer; returns its
/// length, 0 when the current game is not a coded stage.
#[no_mangle]
pub extern "C" fn code_text() -> u32 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let c = match &a.mode {
        Mode::Design(d) => Some(d.code),
        Mode::Single(_) => *STAGE.lock().unwrap(),
        _ => None,
    };
    if c.is_none() {
        if let (Mode::Rally(_), Some(cc)) = (&a.mode, *CHAMP.lock().unwrap()) {
            a.save_buf = cc.encode().into_bytes();
            return a.save_buf.len() as u32;
        }
    }
    let Some(c) = c else { return 0 };
    a.save_buf = c.encode().into_bytes();
    a.save_buf.len() as u32
}

fn stage_game(c: crate::share::StageCode) -> Box<Game> {
    let mut g = c.game();
    apply_equipment(&mut g);
    Box::new(g)
}

static GHOST: Mutex<Option<crate::ghost::Ghost>> = Mutex::new(None);
static NEW_GHOST: Mutex<Option<crate::ghost::Ghost>> = Mutex::new(None);
static NEW_BADGES: AtomicU32 = AtomicU32::new(0);

/// Time of day for the next `init`: 0 automatic, 1 dawn, 2 day, 3 dusk, 4 night.
#[no_mangle]
pub extern "C" fn set_time(i: u32) {
    TIME.store(i.min(4), Ordering::Relaxed);
}

/// Tyres for the next `init`: 0 automatic (best for the conditions), 1 road, 2 rain, 3 snow, 4 off-road.
#[no_mangle]
pub extern "C" fn set_tyre(i: u32) {
    TYRE.store(i.min(crate::env::TYRE_COUNT as u32), Ordering::Relaxed);
}

/// Set-up for the next `init`: 0 balanced, 1 sprint, 2 downforce, 3 economy.
#[no_mangle]
pub extern "C" fn set_setup(i: u32) {
    SETUP.store(i.min(crate::env::SETUP_COUNT as u32 - 1), Ordering::Relaxed);
}

/// Force the difficulty rank 0..=999 (testing); anything larger = take it from the profile.
#[no_mangle]
pub extern "C" fn set_rank(r: u32) {
    RANK.store(if r <= 999 { r } else { u32::MAX }, Ordering::Relaxed);
}

/// Today's day number (days since 1970-01-01); the page passes `Math.floor(Date.now()/864e5)`.
#[no_mangle]
pub extern "C" fn set_today(day: u32) {
    TODAY.store(day, Ordering::Relaxed);
    with_profile(|p| p.begin_day(day));
}

fn with_profile<R>(f: impl FnOnce(&mut Profile) -> R) -> R {
    let mut g = PROFILE.lock().unwrap();
    f(g.get_or_insert_with(Profile::new))
}

fn rank_now() -> u16 {
    match RANK.load(Ordering::Relaxed) {
        u32::MAX => with_profile(|p| p.rank()),
        r => r as u16,
    }
}

fn apply_equipment(g: &mut Game) {
    g.set_time(TIME.load(Ordering::Relaxed) as usize);
    let t = TYRE.load(Ordering::Relaxed) as usize;
    g.set_equipment(
        (t > 0).then(|| crate::env::Tyre::from_index(t - 1)),
        crate::env::Setup::from_index(SETUP.load(Ordering::Relaxed) as usize),
    );
    g.set_rank(rank_now());
    let (a, c) = access();
    g.set_access(a, c);
}

/// Profile: load the save string the page wrote into the scratch buffer (`n` bytes).
#[no_mangle]
pub extern "C" fn profile_load(n: u32) -> u32 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let n = (n as usize).min(a.save_buf.len());
    let txt = String::from_utf8_lossy(&a.save_buf[..n]).to_string();
    let mut p = Profile::decode(&txt);
    p.begin_day(TODAY.load(Ordering::Relaxed));
    *PROFILE.lock().unwrap() = Some(p);
    1
}

/// Profile: if it changed since the last call, writes the save string into the scratch buffer and
/// returns its length (else 0). The page keeps it in localStorage.
#[no_mangle]
pub extern "C" fn profile_save() -> u32 {
    if PROFILE_DIRTY.swap(0, Ordering::Relaxed) == 0 {
        return 0;
    }
    let txt = with_profile(|p| p.encode());
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    a.save_buf = txt.into_bytes();
    a.save_buf.len() as u32
}

/// Ghost: the page puts a ghost code into the scratch buffer (`n` bytes); it is used by the next
/// single-player game of the same stage. Returns 1 when the code is valid.
#[no_mangle]
pub extern "C" fn ghost_load(n: u32) -> u32 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let n = (n as usize).min(a.save_buf.len());
    let txt = String::from_utf8_lossy(&a.save_buf[..n]).to_string();
    let g = crate::ghost::Ghost::decode(&txt);
    let ok = g.is_some() as u32;
    if let Mode::Single(game) = &mut a.mode {
        game.set_ghost(g.clone());
    }
    *GHOST.lock().unwrap() = g;
    ok
}

/// Ghost of the run that just finished (single player), as a code in the scratch buffer; returns
/// its length, 0 when there is none. The page keeps the faster of old and new in localStorage.
#[no_mangle]
pub extern "C" fn ghost_save() -> u32 {
    let Some(g) = NEW_GHOST.lock().unwrap().take() else {
        return 0;
    };
    let txt = g.encode();
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    a.save_buf = txt.into_bytes();
    a.save_buf.len() as u32
}

/// Seconds of the stored ghost of the current stage x 10 (0 = none), for the page's label.
#[no_mangle]
pub extern "C" fn ghost_time() -> u32 {
    GHOST
        .lock()
        .unwrap()
        .as_ref()
        .map_or(0, |g| (g.secs * 10.0) as u32)
}

/// Names of the badges earned since the last call, in the current language, written into the
/// scratch buffer (returns the byte length, 0 when there are none).
#[no_mangle]
pub extern "C" fn badges_new() -> u32 {
    let mask = NEW_BADGES.swap(0, Ordering::Relaxed);
    if mask == 0 {
        return 0;
    }
    let txt = crate::badges::names_of(mask);
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    a.save_buf = txt.into_bytes();
    a.save_buf.len() as u32
}

/// Current rank 0..=999 of the profile.
#[no_mangle]
pub extern "C" fn profile_rank() -> u32 {
    with_profile(|p| p.rank()) as u32
}

fn car_p1() -> crate::cars::CarSpec {
    let base = match CAR.load(Ordering::Relaxed) {
        u32::MAX => crate::cars::CarSpec::default(),
        i => crate::cars::CarSpec::for_country(i as usize),
    };
    base.with_class(crate::cars::VehicleClass::from_index(
        CLASS.load(Ordering::Relaxed) as usize,
    ))
}

/// The rival's car: a different country, picked from the stage seed.
fn car_p2(seed: u64) -> crate::cars::CarSpec {
    let n = crate::career::ncountries() as u64;
    crate::cars::CarSpec::for_country((crate::career::splitmix(seed) % n) as usize)
}

fn new_versus(seed: u64, cpu: bool) -> Box<Versus> {
    let mut v = if cpu {
        Versus::new_cpu(seed)
    } else {
        Versus::new(seed)
    };
    v.set_cars(car_p1(), car_p2(seed));
    v.set_level(level());
    let (wx, sf) = env_sel();
    v.set_env(wx, sf);
    v.set_time(TIME.load(Ordering::Relaxed) as usize);
    let t = TYRE.load(Ordering::Relaxed) as usize;
    v.set_equipment(
        (t > 0).then(|| crate::env::Tyre::from_index(t - 1)),
        crate::env::Setup::from_index(SETUP.load(Ordering::Relaxed) as usize),
    );
    v.set_rank(rank_now());
    let (a, c) = access();
    v.set_access(a, c);
    Box::new(v)
}

fn new_game(seed: u64, mode: crate::game::Mode) -> Box<Game> {
    let mut g = Game::new_mode(seed, mode);
    g.set_car(car_p1());
    g.set_level(level());
    let (wx, sf) = env_sel();
    g.set_env(wx, sf);
    apply_equipment(&mut g);
    // a visible pack of rivals (not for the daily, the weekly or shared stage codes)
    let n = PACK_N.load(Ordering::Relaxed) as usize;
    if n > 0 && DAILY.load(Ordering::Relaxed) == 0 && STAGE.lock().unwrap().is_none() {
        g.set_pack(Some(crate::pack::PackCfg {
            n,
            inverted: PACK_INV.load(Ordering::Relaxed) != 0,
            slot: None,
        }));
    }
    Box::new(g)
}

static PLAYERS: AtomicU32 = AtomicU32::new(0);

/// 3 or 4 cars in a 2x2 split screen for the versus modes (0 = the usual two); modes 9 (one human
/// plus CPU cars) and 10 (two humans plus CPU cars).
#[no_mangle]
pub extern "C" fn set_players(n: u32) {
    PLAYERS.store(if n < 3 { 0 } else { n.min(4) }, Ordering::Relaxed);
}

fn new_multi(seed: u64, two_humans: bool) -> Box<crate::multi::Multi> {
    let n = PLAYERS.load(Ordering::Relaxed).clamp(3, 4) as usize;
    let mut m = crate::multi::Multi::new(seed, n, if two_humans { 2 } else { 1 });
    m.set_cars(&[car_p1(), car_p1(), car_p1(), car_p1()]);
    m.set_level(level());
    let (wx, sf) = env_sel();
    m.set_env(wx, sf);
    Box::new(m)
}

static PACK_N: AtomicU32 = AtomicU32::new(0);
static PACK_INV: AtomicU32 = AtomicU32::new(0);

/// Visible pack of rivals for the next game: `n` cars on the grid (8-16, 0 = off); `inverted` puts
/// the player in front with the pack behind.
#[no_mangle]
pub extern "C" fn set_pack(n: u32, inverted: u32) {
    PACK_N.store(if n == 0 { 0 } else { n.clamp(8, 16) }, Ordering::Relaxed);
    PACK_INV.store(inverted.min(1), Ordering::Relaxed);
}

static SETTINGS_DIRTY: AtomicU32 = AtomicU32::new(0);

/// The options of the start screen as they stand now.
fn web_extras() -> crate::pick::Extras {
    crate::pick::Extras {
        music: crate::audio::music_style(),
        mph: crate::settings::mph(),
        keys: 0,
        pack: PACK_N.load(Ordering::Relaxed) as u8,
        inverted: PACK_INV.load(Ordering::Relaxed) != 0,
        players: PLAYERS.load(Ordering::Relaxed) as u8,
        garage: *GARAGE.lock().unwrap(),
        web: true,
        stock: DAILY.load(Ordering::Relaxed) != 0 || STAGE.lock().unwrap().is_some(),
    }
    .sanitized()
}

/// Options changed on the start screen: make them take effect and ask the page to store them.
fn apply_extras(ex: crate::pick::Extras) {
    ex.settings().apply();
    PACK_N.store(ex.pack as u32, Ordering::Relaxed);
    PACK_INV.store(ex.inverted as u32, Ordering::Relaxed);
    PLAYERS.store(ex.players as u32, Ordering::Relaxed);
    let mut g = GARAGE.lock().unwrap();
    if *g != ex.garage {
        *g = ex.garage;
        g.apply();
        GARAGE_DIRTY.store(1, Ordering::Relaxed);
    }
    SETTINGS_DIRTY.store(1, Ordering::Relaxed);
}

/// Stored options (`TS1;...;p=pack;i=inverted;n=players`) in the scratch buffer (`n` bytes).
#[no_mangle]
pub extern "C" fn settings_load(n: u32) {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let n = (n as usize).min(a.save_buf.len());
    let txt = String::from_utf8_lossy(&a.save_buf[..n]).to_string();
    if txt.len() > 80 || !txt.starts_with("TS1;") {
        return;
    }
    let st = crate::settings::Settings::decode(&txt);
    // the paint job comes from secret codes only
    crate::settings::Settings {
        livery: crate::settings::livery(),
        ..st
    }
    .apply();
    for kv in txt.split(';').skip(1) {
        if let Some((k, v)) = kv.split_once('=') {
            let n: u32 = v.parse().unwrap_or(0);
            match k {
                "p" => PACK_N.store(if n == 0 { 0 } else { n.clamp(8, 16) }, Ordering::Relaxed),
                "i" => PACK_INV.store(n.min(1), Ordering::Relaxed),
                "n" => PLAYERS.store(if n < 3 { 0 } else { n.min(4) }, Ordering::Relaxed),
                _ => {}
            }
        }
    }
}

/// The options as text (0 when nothing changed since the last call).
#[no_mangle]
pub extern "C" fn settings_save() -> u32 {
    if SETTINGS_DIRTY.swap(0, Ordering::Relaxed) == 0 {
        return 0;
    }
    let ex = web_extras();
    let txt = format!(
        "{};p={};i={};n={}",
        ex.settings().encode(),
        ex.pack,
        ex.inverted as u8,
        ex.players
    );
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    a.save_buf = txt.into_bytes();
    a.save_buf.len() as u32
}

/// What a typed or pasted code is (scratch buffer, `n` bytes): 1 championship, 2 stage, 3 ghost, 4 paint
/// word, 5 daily, 6 weekly, 0 not a code.
#[no_mangle]
pub extern "C" fn code_kind(n: u32) -> u32 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let n = (n as usize).min(a.save_buf.len());
    let txt = String::from_utf8_lossy(&a.save_buf[..n]).to_string();
    crate::codes::parse(&txt).map_or(0, |c| c.kind())
}

/// Number of country cars.
#[no_mangle]
pub extern "C" fn country_count() -> u32 {
    crate::career::ncountries() as u32
}

/// ISO code of country `i`, packed as two ASCII bytes (first << 8 | second); 0 when out of range.
#[no_mangle]
pub extern "C" fn country_code(i: u32) -> u32 {
    match crate::career::COUNTRIES.get(i as usize) {
        Some(c) => {
            let b = c.code.as_bytes();
            (b[0] as u32) << 8 | b[1] as u32
        }
        None => 0,
    }
}

/// Choose the country car for the next `init` (index from `country_code`; anything else = default car).
#[no_mangle]
pub extern "C" fn set_car(i: u32) {
    let v = if (i as usize) < crate::career::ncountries() {
        i
    } else {
        u32::MAX
    };
    CAR.store(v, Ordering::Relaxed);
}

/// Vehicle class for the next `init`: 0 sports, 1 sedan, 2 city car, 3 truck, 4 semi-trailer.
#[no_mangle]
pub extern "C" fn set_class(i: u32) {
    CLASS.store(
        i.min(crate::cars::CLASS_COUNT as u32 - 1),
        Ordering::Relaxed,
    );
}

/// Difficulty for the next `init`: 0 easy, 1 normal, 2 hard, 3 expert, 4 insane.
#[no_mangle]
pub extern "C" fn set_level(i: u32) {
    LEVEL.store(
        i.min(crate::game::LEVEL_COUNT as u32 - 1),
        Ordering::Relaxed,
    );
}

/// 1 = show the vehicle / difficulty selection screen at the start of every game (default 0).
#[no_mangle]
pub extern "C" fn set_pick(on: u32) {
    PICK.store(on.min(1), Ordering::Relaxed);
}

static PICK: AtomicU32 = AtomicU32::new(0);

fn build(seed: u64, mode: u32) -> Mode {
    // the daily, the weekly and shared stage codes always use the stock car
    crate::garage::set_stock(DAILY.load(Ordering::Relaxed) != 0 || STAGE.lock().unwrap().is_some());
    if mode < 9 {
        if let Some(c) = *STAGE.lock().unwrap() {
            return Mode::Single(stage_game(c));
        }
    }
    match mode {
        13.. => Mode::Design(Box::new(crate::design::DesignApp::new(
            *STAGE.lock().unwrap(),
        ))),
        9 | 10 if PLAYERS.load(Ordering::Relaxed) >= 3 => Mode::Multi(new_multi(seed, mode == 10)),
        9 => Mode::Versus(new_versus(seed, true)),
        10 => Mode::Versus(new_versus(seed, false)),
        11 => Mode::Career(Box::new(CareerApp::new(seed, None))),
        12.. if CHAMP.lock().unwrap().is_some() => {
            Mode::Rally(Box::new(CHAMP.lock().unwrap().unwrap().app()))
        }
        12.. => Mode::Rally(Box::new(crate::rally::RallyApp::new(
            seed,
            CLASS.load(Ordering::Relaxed) as usize,
            LEVEL.load(Ordering::Relaxed) as usize,
        ))),
        m => Mode::Single(new_game(seed, crate::game::Mode::from_index(m))),
    }
}

fn pick_screen(seed: u64, session: f32) -> Mode {
    let info = with_profile(|p| p.info(session / 60.0));
    Mode::Pick(Box::new(
        crate::pick::PickApp::new(
            CLASS.load(Ordering::Relaxed) as usize,
            LEVEL.load(Ordering::Relaxed) as usize,
        )
        .with_forecast(seed)
        .with_profile(info)
        .with_extras(web_extras()),
    ))
}

#[no_mangle]
/// mode: 0-8 single-player modes (see `Mode`), 9 = race a CPU rival, 10 = two players on one screen, 11 = career,
/// 12 = rally (five stages).
pub extern "C" fn init(seed: u32, mode: u32) {
    let seed = seed as u64;
    if STAGE_NEXT.swap(0, Ordering::Relaxed) == 0 {
        *STAGE.lock().unwrap() = None;
    }
    *APP.lock().unwrap() = Some(App {
        mode: if PICK.load(Ordering::Relaxed) == 1 && mode != 11 && mode != 13 {
            pick_screen(seed, 0.0)
        } else {
            build(seed, mode)
        },
        fb: vec![0; W * H],
        rgba: vec![255; W * H * 4],
        inp: [Input::default(); 2],
        restart: false,
        back: false,
        seed,
        code: mode,
        save_buf: Vec::new(),
        synth: None,
        aparams: AudioParams::default(),
        abuf: Vec::new(),
        session: 0.0,
        counted: false,
        gear_prev: 0,
        gear_bits: 0,
    });
}

fn decode(bits: u32) -> Input {
    Input {
        left: bits & 1 != 0,
        right: bits & 2 != 0,
        accel: bits & 4 != 0,
        brake: bits & 8 != 0,
        turbo: bits & 16 != 0,
    }
}

/// bits 0-4: P1 left, right, accel, brake, turbo; bit 5: restart / Enter; bit 6: back (career menus);
/// bit 7: gear up, bit 14: gear down, bit 15: manual / automatic gearbox, bit 16: photo mode,
/// bit 17: photo-mode HUD (all edge-triggered).
/// bits 8-12: the same for P2 (versus mode); bit 13: restart.
#[no_mangle]
pub extern "C" fn set_input(bits: u32) {
    if let Some(a) = APP.lock().unwrap().as_mut() {
        a.inp = [decode(bits), decode(bits >> 8)];
        a.restart = bits & (32 | 8192) != 0;
        a.back = bits & 64 != 0;
        a.gear_bits = (bits >> 7 & 1) | (bits >> 14 & 3) << 1 | (bits >> 16 & 3) << 3;
    }
}

#[no_mangle]
pub extern "C" fn frame(dt: f32) -> *const u8 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let inp = a.inp;
    let mut next: Option<Mode> = None;
    if a.back
        && PICK.load(Ordering::Relaxed) == 1
        && matches!(a.mode, Mode::Single(_) | Mode::Versus(_) | Mode::Multi(_))
    {
        a.mode = pick_screen(a.seed, a.session);
    }
    let gear_edge = a.gear_bits & !a.gear_prev;
    a.gear_prev = a.gear_bits;
    match &mut a.mode {
        Mode::Pick(p) => {
            if a.back {
                p.back();
            }
            let ch = p.update(dt, &inp[0], a.restart);
            if let Some(ex) = p.take_extras() {
                apply_extras(ex);
            }
            if let Some(ch) = ch {
                CLASS.store(ch.class as u32, Ordering::Relaxed);
                LEVEL.store(ch.level as u32, Ordering::Relaxed);
                WX.store(ch.weather as u32, Ordering::Relaxed);
                SF.store(ch.surface as u32, Ordering::Relaxed);
                TYRE.store(ch.tyre as u32, Ordering::Relaxed);
                SETUP.store(ch.setup as u32, Ordering::Relaxed);
                if with_profile(|p| p.advice(a.session / 60.0)) == crate::profile::Advice::Break {
                    a.session = 0.0;
                }
                a.counted = false;
                next = Some(build(a.seed, a.code));
            }
            p.render(&mut a.fb);
            a.aparams = AudioParams::default();
        }
        Mode::Single(g) => {
            if a.restart && g.state != State::Playing {
                let staged = *STAGE.lock().unwrap();
                if let Some(c) = staged {
                    **g = *stage_game(c);
                } else {
                    a.seed += 1;
                    **g = *new_game(a.seed, g.mode);
                }
                g.set_ghost(GHOST.lock().unwrap().clone());
                a.counted = false;
            }
            if gear_edge & 1 != 0 {
                g.shift_up();
            }
            if gear_edge & 2 != 0 {
                g.shift_down();
            }
            if gear_edge & 4 != 0 {
                g.toggle_manual();
            }
            if gear_edge & 8 != 0 {
                g.photo_toggle();
            }
            if gear_edge & 16 != 0 {
                g.photo_hud_toggle();
            }
            g.update(dt, &inp[0]);
            if g.state == State::Playing {
                a.session += dt.clamp(0.0, 0.1);
            } else if !a.counted {
                a.counted = true;
                let r = GameResult::from_game(g);
                if let Some(ng) = g.make_ghost() {
                    let better = GHOST
                        .lock()
                        .unwrap()
                        .as_ref()
                        .is_none_or(|o| o.seed != ng.seed || o.mode != ng.mode || ng.beats(o));
                    if better {
                        *GHOST.lock().unwrap() = Some(ng.clone());
                        *NEW_GHOST.lock().unwrap() = Some(ng);
                    }
                }
                let gain = with_profile(|p| p.record(&r, TODAY.load(Ordering::Relaxed)));
                NEW_BADGES.fetch_or(gain.badges_new, Ordering::Relaxed);
                let day = DAILY.load(Ordering::Relaxed);
                if day != 0 {
                    let (score, secs) = (g.score, g.elapsed);
                    if let Some((code, _)) =
                        with_profile(|p| p.record_daily(day, score, secs, r.finished))
                    {
                        *DAILY_CODE.lock().unwrap() = code;
                    }
                }
                PROFILE_DIRTY.store(1, Ordering::Relaxed);
                if day == 0 && STAGE.lock().unwrap().is_none() {
                    let mut gr = *GARAGE.lock().unwrap();
                    g.coins_earned = gr.earn(&r);
                    *GARAGE.lock().unwrap() = gr;
                    GARAGE_DIRTY.store(1, Ordering::Relaxed);
                }
            }
            render::render(g, &mut a.fb);
            a.aparams = g.audio_params();
        }
        Mode::Design(d) => {
            d.update(dt, &inp[0], a.restart, a.back);
            d.render(&mut a.fb);
            a.aparams = AudioParams::default();
            if let Some(c) = d.take_code() {
                *STAGE.lock().unwrap() = Some(c);
                a.counted = false;
                next = Some(Mode::Single(stage_game(c)));
            }
        }
        Mode::Rally(r) => {
            r.update(dt, &inp[0], a.restart, a.back);
            r.render(&mut a.fb);
            a.aparams = r.audio_params();
            if r.in_race() {
                a.session += dt.clamp(0.0, 0.1);
            }
            if let Some(res) = r.take_result() {
                let gain = with_profile(|p| p.record(&res, TODAY.load(Ordering::Relaxed)));
                NEW_BADGES.fetch_or(gain.badges_new, Ordering::Relaxed);
                PROFILE_DIRTY.store(1, Ordering::Relaxed);
                // stages of a rally, a championship or a tour pay coins like any other race
                if DAILY.load(Ordering::Relaxed) == 0 && STAGE.lock().unwrap().is_none() {
                    let mut gr = *GARAGE.lock().unwrap();
                    r.add_coins(gr.earn(&res));
                    *GARAGE.lock().unwrap() = gr;
                    GARAGE_DIRTY.store(1, Ordering::Relaxed);
                }
            }
        }
        Mode::Career(c) => {
            c.update(dt, &inp[0], a.restart, a.back);
            c.render(&mut a.fb);
            a.aparams = c.audio_params();
        }
        Mode::Multi(m) => {
            if a.restart && m.is_over() {
                a.seed += 1;
                let two = m.humans >= 2;
                **m = *new_multi(a.seed, two);
                a.counted = false;
            }
            let inputs = [inp[0], inp[1], Input::default(), Input::default()];
            m.update(dt, &inputs);
            m.render(&mut a.fb);
            a.aparams = m.p[0].audio_params();
        }
        Mode::Versus(v) => {
            if a.restart && v.is_over() {
                a.seed += 1;
                **v = *new_versus(a.seed, v.cpu);
                a.counted = false;
            }
            v.update(dt, &inp);
            v.render(&mut a.fb);
            a.aparams = v.p[0].audio_params();
        }
    }
    if let Some(m) = next {
        a.mode = m;
    }
    a.restart = false;
    a.back = false; // one-shot: a stale flag must not restart the next round
    crate::access::apply(&mut a.fb);
    for (i, &c) in a.fb.iter().enumerate() {
        a.rgba[i * 4] = (c >> 16) as u8;
        a.rgba[i * 4 + 1] = (c >> 8) as u8;
        a.rgba[i * 4 + 2] = c as u8;
    }
    a.rgba.as_ptr()
}

#[no_mangle]
pub extern "C" fn width() -> u32 {
    W as u32
}
#[no_mangle]
pub extern "C" fn height() -> u32 {
    H as u32
}

#[no_mangle]
pub extern "C" fn audio_init(sample_rate: f32) {
    if let Some(a) = APP.lock().unwrap().as_mut() {
        a.synth = Some(Synth::new(sample_rate, a.seed));
    }
}

/// Renders `n` mono samples with the latest game parameters; returns a pointer into wasm memory.
#[no_mangle]
pub extern "C" fn audio_fill(n: u32) -> *const f32 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let n = (n as usize).min(1 << 16); // hostile sizes must not exhaust wasm memory
    if a.abuf.len() < n {
        a.abuf.resize(n, 0.0);
    }
    let p = a.aparams;
    match a.synth.as_mut() {
        Some(s) => s.render(&mut a.abuf[..n], &p),
        None => a.abuf[..n].fill(0.0),
    }
    a.abuf.as_ptr()
}

/// 0 playing, 1 time up, 2 finished (single mode; P1 in versus). Handy for tests and the page.
#[no_mangle]
pub extern "C" fn state_code() -> u32 {
    let guard = APP.lock().unwrap();
    let a = guard.as_ref().expect("init first");
    let s = match &a.mode {
        Mode::Single(g) => g.state,
        Mode::Versus(v) => v.p[0].state,
        Mode::Multi(m) => m.p[0].state,
        Mode::Career(_) | Mode::Pick(_) | Mode::Rally(_) | Mode::Design(_) => State::Playing,
    };
    match s {
        State::Playing => 0,
        State::TimeUp => 1,
        State::Finished => 2,
        State::Disqualified => 3,
    }
}

/// Interface language: 0 en, 1 ro, 2 fr, 3 it, 4 es, 5 pt, 6 de, 7 hu.
#[no_mangle]
pub extern "C" fn set_lang(i: u32) {
    crate::i18n::set_lang(crate::i18n::Lang::from_index(i));
}

/// Current score of player 1 (the page keeps the personal best in localStorage).
#[no_mangle]
pub extern "C" fn score() -> u32 {
    let guard = APP.lock().unwrap();
    let a = guard.as_ref().expect("init first");
    match &a.mode {
        Mode::Single(g) => g.score as u32,
        Mode::Versus(v) => v.p[0].score as u32,
        Mode::Multi(m) => m.p[0].score as u32,
        Mode::Career(c) => c.score(),
        Mode::Pick(_) | Mode::Design(_) => 0,
        Mode::Rally(r) => r.score(),
    }
}

/// Scratch buffer shared with the page: returns a pointer to at least `n` bytes.
#[no_mangle]
pub extern "C" fn buf_ptr(n: u32) -> *mut u8 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    a.save_buf.resize((n as usize).min(1 << 16), 0);
    a.save_buf.as_mut_ptr()
}

/// Career: load a save string the page wrote into the scratch buffer (`n` bytes). Returns 1 on success.
#[no_mangle]
pub extern "C" fn career_load(n: u32) -> u32 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let n = (n as usize).min(a.save_buf.len());
    let txt = String::from_utf8_lossy(&a.save_buf[..n]).to_string();
    if let Mode::Career(_) = a.mode {
        if crate::career::Career::from_save(&txt).is_some() {
            a.mode = Mode::Career(Box::new(CareerApp::new(a.seed, Some(&txt))));
            return 1;
        }
    }
    0
}

/// Career: if the state changed since the last call, writes the save string into the scratch buffer
/// and returns its length (else 0). The page stores it in localStorage.
#[no_mangle]
pub extern "C" fn career_save() -> u32 {
    let mut guard = APP.lock().unwrap();
    let a = guard.as_mut().expect("init first");
    let txt = match &mut a.mode {
        Mode::Career(c) => c.take_save(),
        _ => None,
    };
    match txt {
        Some(t) => {
            a.save_buf = t.into_bytes();
            a.save_buf.len() as u32
        }
        None => 0,
    }
}
