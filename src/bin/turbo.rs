//! Native window front-end (TURBOCURSA 1.6).
//!   cargo run --release --features native --bin turbo [-- --versus | --cpu | --career | --demo | --mode NAME | --car RO] [--class sports|city|truck|bus|moped|...] [--level easy|normal|hard|expert|insane] [--weather clear|rain|storm|fog|snow|blizzard|sandstorm|hail|glaze|heatwave|ash|auto] [--surface asphalt|icelake|saltflat|lava|dunes|bog|auto] [--time auto|dawn|day|dusk|night] [--tyre auto|road|rain|snow|offroad] [--setup balanced|sprint|downforce|economy] [--rank N] [--colorblind protan|deutan|tritan|contrast] [--assist 0-3] [--slow 40-100] [--large] [--rally] [--champ TCC1-...|7|10|15] [--tour] [--ghost FILE|TG1;...] [--ghost-export SEED] [--pack 8-16 [--inverted] [--qualify]] [--players 3|4 [--humans N]] [--garage] [--buy 0-3] [--weekly] [--music 0-3] [--mph] [--keys 0-3] [--secret CODE] [--daily] [--design] [--code TCS1-...] [--seed N] [--lang xx]
//!   modes: campaign timetrial endless rush collector slalom mirror blackout zen (or 0-8)
//! Photo mode: P on/off (arrows move the camera, Up/Down scrub, Space+Up/Down height), H shows the HUD, C saves a PNG.
//! Ghost: the best finished run of a stage is saved (~/.turbocursa_ghost_<mode>_<seed>) and raced next time.
//! Gamepad (Linux, /dev/input/js0): sticks or d-pad steer, RT/A gas, LT/B brake, X turbo, Start enter, Select back, LB/RB gears, Y manual.
//! Gears: E up, Q down, T manual/automatic. Backspace: back to the vehicle / service screens.
//! Stage designer: --design (Enter drives the stage, Backspace edits it); --code shares a stage as a short typo-checked code.
//! The profile (rank 0-999, streak, daily budget) lives in ~/.turbocursa_profile.
//! Add `--features audio` for sound (cpal). P1: arrows + Space; P2 (versus): WASD + Left Shift.
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use turbocursa::career_ui::CareerApp;
use turbocursa::cars::CarSpec;
use turbocursa::game::*;
use turbocursa::profile::GameResult;
use turbocursa::versus::Versus;
use turbocursa::{render, H, W};

#[cfg(feature = "audio")]
mod native_audio {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::{Arc, Mutex};
    use turbocursa::audio::{AudioParams, Synth};

    pub struct AudioOut {
        shared: Arc<Mutex<(Synth, AudioParams)>>,
        _stream: cpal::Stream,
    }

    impl AudioOut {
        pub fn new(seed: u64) -> Option<AudioOut> {
            let host = cpal::default_host();
            let dev = host.default_output_device()?;
            let cfg = dev.default_output_config().ok()?;
            if cfg.sample_format() != cpal::SampleFormat::F32 {
                return None;
            }
            let sr = cfg.sample_rate().0 as f32;
            let ch = cfg.channels() as usize;
            let shared = Arc::new(Mutex::new((Synth::new(sr, seed), AudioParams::default())));
            let s2 = shared.clone();
            let mut mono: Vec<f32> = Vec::new();
            let stream = dev
                .build_output_stream(
                    &cfg.into(),
                    move |data: &mut [f32], _| {
                        let frames = data.len() / ch;
                        if mono.len() < frames {
                            mono.resize(frames, 0.0);
                        }
                        let mut g = s2.lock().unwrap();
                        let p = g.1;
                        g.0.render(&mut mono[..frames], &p);
                        for (i, f) in data.chunks_mut(ch).enumerate() {
                            f.fill(mono[i]);
                        }
                    },
                    |e| eprintln!("audio error: {e}"),
                    None,
                )
                .ok()?;
            stream.play().ok()?;
            Some(AudioOut {
                shared,
                _stream: stream,
            })
        }

        pub fn set(&self, p: AudioParams) {
            self.shared.lock().unwrap().1 = p;
        }
    }
}

/// Reads a small text file (saves, ghosts, settings); anything over 2 MB is refused instead of loaded.
fn read_small<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<String> {
    use std::io::Read;
    let mut s = String::new();
    std::fs::File::open(path)?
        .take(2 << 20)
        .read_to_string(&mut s)?;
    Ok(s)
}

/// Writes through a temporary file and renames it over the target, so a crash or a full disk
/// never leaves a half-written save behind.
fn write_atomic<P: AsRef<std::path::Path>, C: AsRef<[u8]>>(
    path: P,
    data: C,
) -> std::io::Result<()> {
    let path = path.as_ref();
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

fn ghost_file(g: &Game) -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
        .join(format!(".turbocursa_ghost_{}_{}", g.mode as u8, g.seed))
}

fn load_ghost(g: &mut Game) {
    let txt = read_small(ghost_file(g)).unwrap_or_default();
    g.set_ghost(turbocursa::ghost::Ghost::decode(&txt));
}

/// Keeps the faster of the stored ghost and the finished run.
fn save_ghost(g: &Game) {
    let Some(new) = g.make_ghost() else { return };
    let old = read_small(ghost_file(g))
        .ok()
        .and_then(|t| turbocursa::ghost::Ghost::decode(&t));
    if old.as_ref().is_none_or(|o| new.beats(o)) {
        let _ = write_atomic(ghost_file(g), new.encode());
    }
}

/// Characters typed in the window (the code field of the options page).
struct CharSink(std::rc::Rc<std::cell::RefCell<Vec<char>>>);

impl minifb::InputCallback for CharSink {
    fn add_char(&mut self, c: u32) {
        if let Some(c) = char::from_u32(c) {
            let mut v = self.0.borrow_mut();
            if v.len() < 256 {
                v.push(c);
            }
        }
    }
}

/// Stores an imported ghost when it beats the one already kept for that stage.
fn import_ghost(gh: &turbocursa::ghost::Ghost) {
    let path = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
        .join(format!(".turbocursa_ghost_{}_{}", gh.mode, gh.seed));
    let old = read_small(&path)
        .ok()
        .and_then(|t| turbocursa::ghost::Ghost::decode(&t));
    if old.as_ref().is_none_or(|o| gh.beats(o)) {
        let _ = write_atomic(&path, gh.encode());
    }
}

enum Mode {
    Single(Box<Game>),
    Versus(Box<Versus>),
    Multi(Box<turbocursa::multi::Multi>),
    Career(Box<CareerApp>),
    Pick(Box<turbocursa::pick::PickApp>),
    Rally(Box<turbocursa::rally::RallyApp>),
    Design(Box<turbocursa::design::DesignApp>),
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cpu = args.iter().any(|a| a == "--cpu");
    let career = args.iter().any(|a| a == "--career");
    // --car RO | 17 | random : fit a country's car (livery + handling profile)
    let car_arg = args
        .iter()
        .position(|a| a == "--car")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let ncty = turbocursa::career::ncountries();
    let arg = |name: &str| -> Option<String> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    // settings: --music 0-3 (normal, calm, drive, off), --mph, --keys 0-3, --secret CODE (paint jobs)
    let set_path = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
        .join(".turbocursa_settings");
    let mut settings =
        turbocursa::settings::Settings::decode(&read_small(&set_path).unwrap_or_default());
    if let Some(m) = arg("--music").and_then(|s| s.parse().ok()) {
        settings.music = m;
    }
    if args.iter().any(|a| a == "--mph") {
        settings.mph = true;
    }
    if args.iter().any(|a| a == "--kmh") {
        settings.mph = false;
    }
    if let Some(k) = arg("--keys").and_then(|s| s.parse().ok()) {
        settings.keys = k;
    }
    if let Some(code) = arg("--secret") {
        match turbocursa::settings::secret_code(&code) {
            Some(l) => {
                settings.livery = l;
                println!("secret paint unlocked");
            }
            None => eprintln!("unknown secret code"),
        }
    }
    let mut settings = settings.sanitized();
    settings.apply();
    let _ = write_atomic(&set_path, settings.encode());
    // garage: coins from play buy upgrades (--garage shows them, --buy 0-3 buys the next level)
    let gar_path = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
        .join(".turbocursa_garage");
    let mut garage = turbocursa::garage::Garage::decode(&read_small(&gar_path).unwrap_or_default());
    if let Some(i) = arg("--buy").and_then(|s| s.parse::<usize>().ok()) {
        if garage.buy(i) {
            println!("bought: {}", turbocursa::garage::NAMES[i]);
            let _ = write_atomic(&gar_path, garage.encode());
        } else {
            eprintln!("cannot buy that upgrade (not enough coins or already maxed)");
        }
    }
    if args.iter().any(|a| a == "--garage") || arg("--buy").is_some() {
        println!("coins {} | {}", garage.coins, garage.status());
        return;
    }
    garage.apply();
    let stock = args
        .iter()
        .any(|a| a == "--daily" || a == "--weekly" || a == "--code");
    turbocursa::garage::set_stock(stock);
    let weekly_on = args.iter().any(|a| a == "--weekly");
    let daily_cfg = if args.iter().any(|a| a == "--daily") {
        Some(turbocursa::daily::for_day(turbocursa::profile::today()))
    } else if weekly_on {
        Some(turbocursa::daily::weekly(turbocursa::daily::week_of(
            turbocursa::profile::today(),
        )))
    } else {
        None
    };
    // vehicle class and difficulty: --class / --level skip the selection screen; --nopick too
    let class_i = std::cell::Cell::new(
        arg("--class")
            .and_then(|s| turbocursa::cars::VehicleClass::parse(&s))
            .map_or(daily_cfg.map_or(0, |d| d.class), |c| c.index()),
    );
    let level_i = std::cell::Cell::new(
        arg("--level")
            .and_then(|s| turbocursa::game::Difficulty::parse(&s))
            .map_or(daily_cfg.map_or(1, |d| d.level), |l| l.index()),
    );
    // weather / surface menu indices: 0 = automatic
    let env_i = std::cell::Cell::new((
        arg("--weather")
            .and_then(|s| turbocursa::env::Weather::parse(&s))
            .map_or(daily_cfg.map_or(0, |d| d.weather), |w| w.index() + 1),
        arg("--surface")
            .and_then(|s| turbocursa::env::Surface::parse(&s))
            .map_or(daily_cfg.map_or(0, |d| d.surface), |s| s.index() + 1),
    ));
    // equipment (service) and time of day: 0 = automatic
    let equip = std::cell::Cell::new((
        arg("--tyre")
            .and_then(|s| turbocursa::env::Tyre::parse(&s).map(|t| t.index() + 1))
            .unwrap_or(0),
        arg("--setup")
            .and_then(|s| turbocursa::env::Setup::parse(&s))
            .map_or(0, |s| s.index()),
    ));
    let time_i = arg("--time")
        .and_then(|s| match s.as_str() {
            "dawn" => Some(1),
            "day" => Some(2),
            "dusk" => Some(3),
            "night" => Some(4),
            _ => None,
        })
        .unwrap_or(daily_cfg.map_or(0, |d| d.time));
    // accessibility
    if let Some(f) = arg("--colorblind").and_then(|s| turbocursa::access::parse_filter(&s)) {
        turbocursa::access::set_filter(f);
    }
    let assist: u8 = arg("--assist").and_then(|s| s.parse().ok()).unwrap_or(0);
    let speed_cap: f32 = arg("--slow")
        .and_then(|s| s.parse::<f32>().ok())
        .map_or(1.0, |p| p / 100.0);
    let large = args.iter().any(|a| a == "--large");
    let pick_on = daily_cfg.is_none()
        && !args.iter().any(|a| {
            a == "--career"
                || a == "--nopick"
                || a == "--class"
                || a == "--level"
                || a == "--weather"
                || a == "--surface"
        });
    let car1 = |seed: u64| -> CarSpec {
        let c = match car_arg.as_deref() {
            None => CarSpec::default(),
            Some("random") => CarSpec::for_country(
                (turbocursa::career::splitmix(seed ^ 0xC4) % ncty as u64) as usize,
            ),
            Some(c) => {
                let up = c.to_ascii_uppercase();
                let idx = turbocursa::career::COUNTRIES
                    .iter()
                    .position(|x| x.code == up)
                    .or_else(|| c.parse::<usize>().ok().filter(|&i| i < ncty));
                idx.map(CarSpec::for_country).unwrap_or_default()
            }
        };
        c.with_class(turbocursa::cars::VehicleClass::from_index(class_i.get()))
    };
    let level = || turbocursa::game::Difficulty::from_index(level_i.get());
    let car2 = |seed: u64| {
        CarSpec::for_country((turbocursa::career::splitmix(seed) % ncty as u64) as usize)
    };
    // --ghost FILE|TG1;...: import a friend's ghost (it becomes the ghost of that stage if faster);
    // --ghost-export seed [mode]: print the stored ghost of a stage as a shareable code
    if let Some(src) = arg("--ghost") {
        let txt = read_small(&src).unwrap_or(src);
        match turbocursa::ghost::Ghost::decode(&txt) {
            Some(gh) => {
                let path = std::env::var("HOME")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_default()
                    .join(format!(".turbocursa_ghost_{}_{}", gh.mode, gh.seed));
                let old = read_small(&path)
                    .ok()
                    .and_then(|t| turbocursa::ghost::Ghost::decode(&t));
                if old.as_ref().is_none_or(|o| gh.beats(o)) {
                    let _ = write_atomic(&path, gh.encode());
                    println!(
                        "ghost imported: stage {} mode {} ({:.1} s)",
                        gh.seed, gh.mode, gh.secs
                    );
                } else {
                    println!("you already have a faster ghost for that stage");
                }
            }
            None => eprintln!("invalid ghost code"),
        }
        return;
    }
    if let Some(sd) = arg("--ghost-export") {
        let mode = arg("--mode-index").unwrap_or_else(|| "0".into());
        let path = std::env::var("HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_default()
            .join(format!(".turbocursa_ghost_{mode}_{sd}"));
        match read_small(path) {
            Ok(t) => println!("{}", t.trim()),
            Err(_) => eprintln!("no ghost stored for that stage"),
        }
        return;
    }
    let rally = args.iter().any(|a| a == "--rally");
    // --champ TCC1-... | 7 | 10 | 15 : a long championship; --tour [N]: the eight-stage tour of a country
    let champ_cfg: Option<turbocursa::champ::ChampCode> = {
        let seed0 = arg("--seed")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1);
        let fresh = |kind: u8| {
            turbocursa::champ::ChampCode::from_seed(seed0, kind, class_i.get(), level_i.get())
        };
        match arg("--champ") {
            Some(c) => match turbocursa::champ::ChampCode::decode(&c) {
                Some(cc) => Some(cc),
                None => match c.as_str() {
                    "7" => Some(fresh(0)),
                    "10" => Some(fresh(1)),
                    "15" => Some(fresh(2)),
                    _ => {
                        eprintln!("invalid championship code (TCC1-... or 7 / 10 / 15)");
                        None
                    }
                },
            },
            None if args.iter().any(|a| a == "--tour") => Some(fresh(turbocursa::champ::TOUR_KIND)),
            None => None,
        }
    };
    if let Some(c) = champ_cfg {
        println!("championship code: {} ({} stages)", c.encode(), c.stages());
    }
    let versus = args.iter().any(|a| a == "--versus") || cpu;
    let gmode = args
        .iter()
        .position(|a| a == "--mode")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| turbocursa::game::Mode::parse(s))
        .unwrap_or_else(|| {
            daily_cfg.map_or(turbocursa::game::Mode::Campaign, |d| {
                turbocursa::game::Mode::from_index(d.mode)
            })
        });
    // interface language: --lang xx, else $LC_ALL / $LANG, else English
    let lang = args
        .iter()
        .position(|a| a == "--lang")
        .and_then(|i| args.get(i + 1).cloned())
        .or_else(|| std::env::var("LC_ALL").ok())
        .or_else(|| std::env::var("LANG").ok())
        .and_then(|s| turbocursa::i18n::Lang::parse(&s))
        .unwrap_or(turbocursa::i18n::Lang::En);
    turbocursa::i18n::set_lang(lang);
    // --daily: the stage of the day, with the same vehicle, level and conditions for everybody
    let mut seed: u64 = if let Some(d) = daily_cfg {
        d.seed
    } else {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(1)
            % 100_000
    };
    if let Some(i) = args.iter().position(|a| a == "--seed") {
        seed = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(seed);
    }
    let title = if career {
        "TURBOCURSA - career"
    } else if versus {
        "TURBOCURSA - versus"
    } else {
        "TURBOCURSA"
    };
    let mut win = Window::new(
        title,
        W,
        H,
        WindowOptions {
            scale: if large { Scale::X4 } else { Scale::X2 },
            ..WindowOptions::default()
        },
    )
    .expect("window");
    win.set_target_fps(60);
    let typed = std::rc::Rc::new(std::cell::RefCell::new(Vec::<char>::new()));
    win.set_input_callback(Box::new(CharSink(typed.clone())));
    let quit = std::cell::Cell::new(false);
    let save_path = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".turbocursa_career"))
        .unwrap_or_else(|_| std::path::PathBuf::from("turbocursa_career.sav"));
    let prof_path = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".turbocursa_profile"))
        .unwrap_or_else(|_| std::path::PathBuf::from("turbocursa_profile.sav"));
    let mut profile =
        turbocursa::profile::Profile::decode(&read_small(&prof_path).unwrap_or_default());
    profile.begin_day(turbocursa::profile::today());
    if !profile.unlocked(class_i.get()) {
        eprintln!(
            "vehicle still locked: earn it with clean cities, deliveries and daily challenges"
        );
        class_i.set(0);
    }
    let rank_override: Option<u16> = arg("--rank").and_then(|s| s.parse().ok());
    let rank_now =
        |p: &turbocursa::profile::Profile| rank_override.unwrap_or_else(|| p.rank()).min(999);
    let mut session = 0.0f32; // seconds of play since the last break
    let mut photo_n = 1u32;
    let mut last_score: Option<(u32, f32)> = None;
    let mut counted = false; // the current game is already booked
    let tyre_of = |i: usize| (i > 0).then(|| turbocursa::env::Tyre::from_index(i - 1));
    // --pack N [--inverted] [--qualify]: a visible pack of N cars (8-16) on the grid; --qualify
    // first drives a timed lap whose time decides the starting slot
    let pack_n = std::cell::Cell::new(arg("--pack").and_then(|s| s.parse::<usize>().ok()));
    let pack_inv = std::cell::Cell::new(args.iter().any(|a| a == "--inverted"));
    let qualify = pack_n.get().is_some() && args.iter().any(|a| a == "--qualify");
    let qual_slot = std::cell::Cell::new(None::<usize>);
    let pack_cfg = || {
        pack_n.get().map(|n| turbocursa::pack::PackCfg {
            n,
            inverted: pack_inv.get(),
            slot: qual_slot.get(),
        })
    };
    // --players 3|4 [--humans N]: 2x2 split screen, the cars beyond the humans are driven by the autopilot
    let players = std::cell::Cell::new(
        arg("--players")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0),
    );
    let humans: usize = arg("--humans").and_then(|s| s.parse().ok()).unwrap_or(2);
    let make = |s: u64, rank: u16| {
        use turbocursa::launch::{decide, Launch};
        let kind = decide(players.get(), career, champ_cfg.is_some(), rally, versus);
        if kind == Launch::Multi {
            let mut m = turbocursa::multi::Multi::new(s, players.get(), humans);
            m.set_cars(&[car1(s), car1(s + 1), car1(s + 2), car1(s + 3)]);
            m.set_level(level());
            m.set_env(env_i.get().0, env_i.get().1);
            m.set_time(time_i);
            m.set_rank(rank);
            Mode::Multi(Box::new(m))
        } else if kind == Launch::Career {
            let saved = read_small(&save_path).ok();
            Mode::Career(Box::new(CareerApp::new(s, saved.as_deref())))
        } else if let (Launch::Champ, Some(c)) = (kind, champ_cfg) {
            Mode::Rally(Box::new(c.app()))
        } else if kind == Launch::Rally {
            Mode::Rally(Box::new(turbocursa::rally::RallyApp::new(
                s,
                class_i.get(),
                level_i.get(),
            )))
        } else if kind == Launch::Versus {
            let mut v = if cpu {
                Versus::new_cpu(s)
            } else {
                Versus::new(s)
            };
            v.set_cars(car1(s), car2(s));
            v.set_level(level());
            v.set_env(env_i.get().0, env_i.get().1);
            v.set_time(time_i);
            v.set_equipment(
                tyre_of(equip.get().0),
                turbocursa::env::Setup::from_index(equip.get().1),
            );
            v.set_rank(rank);
            v.set_access(assist, speed_cap);
            Mode::Versus(Box::new(v))
        } else {
            let qualifying = qualify && qual_slot.get().is_none();
            let mut g = Game::new_mode(
                s,
                if qualifying {
                    turbocursa::game::Mode::TimeTrial
                } else {
                    gmode
                },
            );
            g.set_car(car1(s));
            g.set_level(level());
            g.set_env(env_i.get().0, env_i.get().1);
            g.set_time(time_i);
            g.set_equipment(
                tyre_of(equip.get().0),
                turbocursa::env::Setup::from_index(equip.get().1),
            );
            g.set_rank(rank);
            g.set_access(assist, speed_cap);
            load_ghost(&mut g);
            if !qualifying {
                g.set_pack(pack_cfg());
            }
            Mode::Single(Box::new(g))
        }
    };
    let extras_now = |st: &turbocursa::settings::Settings, g: &turbocursa::garage::Garage| {
        turbocursa::pick::Extras {
            music: st.music,
            mph: st.mph,
            keys: st.keys,
            pack: pack_n.get().unwrap_or(0) as u8,
            inverted: pack_inv.get(),
            players: players.get() as u8,
            garage: *g,
            web: false,
            stock,
        }
    };
    let new_pick = |seed: u64,
                    profile: &turbocursa::profile::Profile,
                    session: f32,
                    ex: turbocursa::pick::Extras| {
        Mode::Pick(Box::new(
            turbocursa::pick::PickApp::new(class_i.get(), level_i.get())
                .with_forecast(seed)
                .with_profile(profile.info(session / 60.0))
                .with_extras(ex),
        ))
    };
    // --code TCS1-...: drive a shared stage; --design: build one (Enter drives it, Backspace edits)
    let design = args.iter().any(|a| a == "--design");
    let mut stage_code: Option<turbocursa::share::StageCode> =
        arg("--code").and_then(|c| turbocursa::share::StageCode::decode(&c));
    if arg("--code").is_some() && stage_code.is_none() {
        eprintln!("invalid stage code");
    }
    let mut demo = args
        .iter()
        .any(|a| a == "--demo")
        .then(turbocursa::ace::Demo::new);
    let mut mode = if let Some(d) = &demo {
        println!("demo: {} stages", d.total());
        Mode::Single(Box::new(d.current()))
    } else if design {
        Mode::Design(Box::new(turbocursa::design::DesignApp::new(stage_code)))
    } else if let Some(c) = stage_code {
        let mut g = c.game();
        g.set_rank(rank_now(&profile));
        g.set_access(assist, speed_cap);
        Mode::Single(Box::new(g))
    } else if pick_on {
        new_pick(seed, &profile, session, extras_now(&settings, &garage))
    } else {
        make(seed, rank_now(&profile))
    };
    #[cfg(feature = "audio")]
    let audio = native_audio::AudioOut::new(seed);
    let mut fb = vec![0u32; W * H];
    let mut last = Instant::now();
    let mut pad = turbocursa::gamepad::Gamepad::new();
    while win.is_open() && !win.is_key_down(Key::Escape) && !quit.get() {
        let now = Instant::now();
        let dt = (now - last).as_secs_f32();
        last = now;
        let ps = pad.poll(dt);
        let dn = |k: Key| win.is_key_down(k);
        let p1 = match (settings.keys, versus) {
            (2, false) => Input {
                left: dn(Key::J) || dn(Key::Left),
                right: dn(Key::L) || dn(Key::Right),
                accel: dn(Key::I) || dn(Key::Up),
                brake: dn(Key::K) || dn(Key::Down),
                turbo: dn(Key::Space),
            },
            (3, false) => Input {
                left: dn(Key::NumPad4) || dn(Key::Left),
                right: dn(Key::NumPad6) || dn(Key::Right),
                accel: dn(Key::NumPad8) || dn(Key::Up),
                brake: dn(Key::NumPad2) || dn(Key::Down),
                turbo: dn(Key::NumPad0) || dn(Key::Space),
            },
            (1, false) => Input {
                left: dn(Key::A) || dn(Key::Q),
                right: dn(Key::D),
                accel: dn(Key::W) || dn(Key::Z),
                brake: dn(Key::S),
                turbo: dn(Key::Space),
            },
            _ => Input {
                left: dn(Key::Left) || (!versus && dn(Key::A)),
                right: dn(Key::Right) || (!versus && dn(Key::D)),
                accel: dn(Key::Up) || (!versus && (dn(Key::W) || dn(Key::Z))),
                brake: dn(Key::Down) || (!versus && dn(Key::S)),
                turbo: dn(Key::Space),
            },
        };
        let p1 = Input {
            left: p1.left || ps.input.left,
            right: p1.right || ps.input.right,
            accel: p1.accel || ps.input.accel,
            brake: p1.brake || ps.input.brake,
            turbo: p1.turbo || ps.input.turbo,
        };
        let p2 = Input {
            left: win.is_key_down(Key::A),
            right: win.is_key_down(Key::D),
            accel: win.is_key_down(Key::W),
            brake: win.is_key_down(Key::S),
            turbo: win.is_key_down(Key::LeftShift),
        };
        let enter = win.is_key_pressed(Key::Enter, KeyRepeat::No) || ps.enter;
        let back = win.is_key_pressed(Key::Backspace, KeyRepeat::No) || ps.back;
        // the demo plays by itself: Enter / Backspace must not start a normal game under the pilot
        let (enter, back) = if demo.is_some() {
            (false, false)
        } else {
            (enter, back)
        };
        if back && design && matches!(mode, Mode::Single(_)) {
            mode = Mode::Design(Box::new(turbocursa::design::DesignApp::new(stage_code)));
        } else if back && pick_on && !design && matches!(mode, Mode::Single(_) | Mode::Versus(_)) {
            mode = new_pick(seed, &profile, session, extras_now(&settings, &garage));
        }
        let mut chosen = false;
        let mut finished_game: Option<GameResult> = None;
        match &mut mode {
            Mode::Pick(p) => {
                if back {
                    p.back();
                }
                for c in typed.borrow_mut().drain(..) {
                    p.type_char(c);
                }
                let ch = p.update(dt, &p1, enter);
                if let Some(raw) = p.take_code() {
                    match turbocursa::codes::parse(&raw) {
                        Some(code) => {
                            match &code {
                                turbocursa::codes::Code::Paint(l) => {
                                    settings.livery = *l;
                                    settings.apply();
                                    let _ = write_atomic(&set_path, settings.encode());
                                }
                                turbocursa::codes::Code::Ghost(g) => import_ghost(g),
                                _ => {}
                            }
                            if let Some(args) = code.launch_args(&raw) {
                                // the stage, the championship and the challenges set up the whole run: start afresh
                                let started = std::env::current_exe().and_then(|exe| {
                                    std::process::Command::new(exe).args(args).spawn()
                                });
                                if started.is_ok() {
                                    quit.set(true);
                                } else {
                                    p.set_note("CODE INVALID");
                                }
                            } else {
                                p.set_note("CODE OK");
                            }
                        }
                        None => p.set_note("CODE INVALID"),
                    }
                }
                if let Some(ex) = p.take_extras() {
                    settings = ex.settings();
                    settings.apply();
                    let _ = write_atomic(&set_path, settings.encode());
                    pack_n.set((ex.pack > 0).then_some(ex.pack as usize));
                    pack_inv.set(ex.inverted);
                    players.set(ex.players as usize);
                    if ex.garage != garage {
                        garage = ex.garage;
                        garage.apply();
                        let _ = write_atomic(&gar_path, garage.encode());
                    }
                }
                if let Some(ch) = ch {
                    equip.set((ch.tyre, ch.setup));
                    class_i.set(ch.class);
                    level_i.set(ch.level);
                    env_i.set((ch.weather, ch.surface));
                    chosen = true;
                }
                p.render(&mut fb);
            }
            Mode::Design(d) => {
                d.update(dt, &p1, enter, back);
                d.render(&mut fb);
                if let Some(c) = d.take_code() {
                    eprintln!("{}", c.encode());
                    stage_code = Some(c);
                    let mut g = c.game();
                    g.set_rank(rank_now(&profile));
                    g.set_access(assist, speed_cap);
                    mode = Mode::Single(Box::new(g));
                }
            }
            Mode::Rally(r) => {
                r.update(dt, &p1, enter, back);
                r.render(&mut fb);
                if r.in_race() {
                    session += dt;
                }
                if let Some(res) = r.take_result() {
                    finished_game = Some(res);
                }
                #[cfg(feature = "audio")]
                if let Some(a) = &audio {
                    a.set(r.audio_params());
                }
            }
            Mode::Career(c) => {
                c.update(dt, &p1, enter, back);
                c.render(&mut fb);
                if let Some(s) = c.take_save() {
                    let _ = write_atomic(&save_path, s);
                }
                #[cfg(feature = "audio")]
                if let Some(a) = &audio {
                    a.set(c.audio_params());
                }
            }
            Mode::Single(g) => {
                if let (true, Some(c)) = (enter && g.state != State::Playing, stage_code) {
                    // a shared stage is retried as it is
                    **g = c.game();
                    g.set_rank(rank_now(&profile));
                    g.set_access(assist, speed_cap);
                    counted = false;
                } else if enter && g.state != State::Playing {
                    let was_qualifying = qualify && qual_slot.get().is_none();
                    if was_qualifying {
                        // the timed lap decides the grid slot; the race is on the same stage
                        let secs = if g.state == State::Finished {
                            g.elapsed
                        } else {
                            f32::MAX
                        };
                        let n = pack_n.get().unwrap_or(12);
                        qual_slot.set(Some(turbocursa::pack::slot_for(
                            g.seed,
                            n,
                            secs,
                            g.reference_secs(),
                        )));
                        println!("qualified: grid slot {}", qual_slot.get().unwrap_or(0) + 1);
                    } else {
                        seed += 1;
                    }
                    let (car, lv) = (g.car, g.level);
                    let next_mode = if was_qualifying { gmode } else { g.mode };
                    **g = Game::new_mode(seed, next_mode);
                    g.set_car(car);
                    g.set_level(lv);
                    g.set_env(env_i.get().0, env_i.get().1);
                    g.set_time(time_i);
                    g.set_equipment(
                        tyre_of(equip.get().0),
                        turbocursa::env::Setup::from_index(equip.get().1),
                    );
                    g.set_rank(rank_now(&profile));
                    g.set_access(assist, speed_cap);
                    load_ghost(g);
                    g.set_pack(pack_cfg());
                    counted = false;
                }
                if win.is_key_pressed(Key::P, KeyRepeat::No) {
                    g.photo_toggle();
                }
                if win.is_key_pressed(Key::H, KeyRepeat::No) {
                    g.photo_hud_toggle();
                }
                if win.is_key_pressed(Key::E, KeyRepeat::No) || ps.gear_up {
                    g.shift_up();
                }
                if win.is_key_pressed(Key::Q, KeyRepeat::No) || ps.gear_down {
                    g.shift_down();
                }
                if win.is_key_pressed(Key::T, KeyRepeat::No) || ps.manual {
                    g.toggle_manual();
                }
                let p1 = if let Some(d) = demo.as_mut() {
                    let (i, next) = d.step(g, dt);
                    if let Some(n) = next {
                        **g = n;
                        println!("demo: stage {}/{}", d.idx % d.total() + 1, d.total());
                    }
                    i
                } else {
                    p1
                };
                g.update(dt, &p1);
                if g.state == State::Playing {
                    session += dt;
                } else if !counted && demo.is_none() {
                    // (a demo run is never recorded: no XP, coins, records or ghosts)
                    counted = true;
                    finished_game = Some(GameResult::from_game(g));
                    last_score = Some((g.score, g.elapsed));
                    save_ghost(g);
                }
                render::render(g, &mut fb);
                #[cfg(feature = "audio")]
                if let Some(a) = &audio {
                    a.set(g.audio_params());
                }
            }
            Mode::Multi(m) => {
                if enter && m.is_over() {
                    seed += 1;
                    let rk = rank_now(&profile);
                    if let Mode::Multi(nm) = make(seed, rk) {
                        *m = nm;
                    }
                }
                let key = |ks: [Key; 5]| Input {
                    left: dn(ks[0]),
                    right: dn(ks[1]),
                    accel: dn(ks[2]),
                    brake: dn(ks[3]),
                    turbo: dn(ks[4]),
                };
                let inputs = [
                    key([Key::Left, Key::Right, Key::Up, Key::Down, Key::Space]),
                    key([Key::A, Key::D, Key::W, Key::S, Key::LeftShift]),
                    key([Key::J, Key::L, Key::I, Key::K, Key::U]),
                    key([
                        Key::NumPad4,
                        Key::NumPad6,
                        Key::NumPad8,
                        Key::NumPad2,
                        Key::NumPad0,
                    ]),
                ];
                m.update(dt, &inputs);
                m.render(&mut fb);
                #[cfg(feature = "audio")]
                if let Some(a) = &audio {
                    a.set(m.p[0].audio_params());
                }
            }
            Mode::Versus(v) => {
                if enter && v.is_over() {
                    seed += 1;
                    let cpu_now = v.cpu;
                    **v = if cpu_now {
                        Versus::new_cpu(seed)
                    } else {
                        Versus::new(seed)
                    };
                    v.set_cars(car1(seed), car2(seed));
                    v.set_level(level());
                    v.set_env(env_i.get().0, env_i.get().1);
                    v.set_time(time_i);
                    v.set_equipment(
                        tyre_of(equip.get().0),
                        turbocursa::env::Setup::from_index(equip.get().1),
                    );
                    v.set_rank(rank_now(&profile));
                    v.set_access(assist, speed_cap);
                }
                v.update(dt, &[p1, p2]);
                v.render(&mut fb);
                #[cfg(feature = "audio")]
                if let Some(a) = &audio {
                    a.set(v.p[0].audio_params());
                }
            }
        }
        if let Some(r) = finished_game {
            let today = turbocursa::profile::today();
            let gain = profile.record(&r, today);
            if !stock {
                let c = garage.earn(&r);
                match &mut mode {
                    Mode::Single(g) => g.coins_earned = c,
                    Mode::Rally(r) => r.add_coins(c),
                    _ => {}
                }
                if c > 0 {
                    println!("+{c} coins (total {})", garage.coins);
                    let _ = write_atomic(&gar_path, garage.encode());
                }
            }
            let mut daily_msg = String::new();
            if let (Some(d), Some((score, secs)), true) = (daily_cfg, last_score, weekly_on) {
                let code = turbocursa::daily::weekly_code(
                    turbocursa::daily::week_of(today),
                    score,
                    secs,
                    r.finished,
                );
                daily_msg = format!("  WEEKLY {code}");
                println!("weekly result: {code}");
                let _ = d;
            } else if let (Some(d), Some((score, secs))) = (daily_cfg, last_score) {
                if let Some((code, bonus)) = profile.record_daily(d.day, score, secs, r.finished) {
                    daily_msg = format!("  DAILY +{bonus} XP {code}");
                    println!("daily result: {code}");
                } else {
                    daily_msg = "  (daily: practice run)".into();
                }
            }
            let _ = write_atomic(&prof_path, profile.encode());
            win.set_title(&format!(
                "TURBOCURSA  +{} XP  rank {}{}{}{}",
                gain.xp,
                profile.rank(),
                if gain.ranks_up > 0 { "  RANK UP!" } else { "" },
                if gain.badges_new != 0 {
                    format!("  *{}*", turbocursa::badges::names_of(gain.badges_new))
                } else {
                    String::new()
                },
                daily_msg
            ));
        }
        if chosen {
            if profile.advice(session / 60.0) == turbocursa::profile::Advice::Break {
                session = 0.0; // the reminder was shown on the pick screen
            }
            mode = make(seed, rank_now(&profile));
        }
        turbocursa::access::apply(&mut fb);
        let typing_code = matches!(&mode, Mode::Pick(p) if p.is_typing());
        if !typing_code && win.is_key_pressed(Key::C, KeyRepeat::No) {
            let name = format!("turbocursa_photo_{}.png", photo_n);
            photo_n += 1;
            match write_atomic(&name, turbocursa::png::encode(&fb, W, H)) {
                Ok(()) => win.set_title(&format!("TURBOCURSA  saved {name}")),
                Err(e) => win.set_title(&format!("TURBOCURSA  photo failed: {e}")),
            }
        }
        win.update_with_buffer(&fb, W, H).unwrap();
    }
}
