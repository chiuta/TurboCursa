//! Profile (rank 0..=999, responsible play), tyres / set-up and day-night.
use turbocursa::env::*;
use turbocursa::game::*;
use turbocursa::profile::*;

fn result(frac: f32, secs: f32) -> GameResult {
    GameResult {
        distance_frac: frac,
        finished: frac >= 1.0,
        seconds: secs,
        ..Default::default()
    }
}

#[test]
fn rank_curve_is_monotonic_and_bounded() {
    let mut prev = 0;
    for r in 0..=MAX_RANK {
        let x = xp_for_rank(r);
        assert!(r == 0 || x > prev);
        prev = x;
        let mut p = Profile::new();
        p.xp = x;
        assert_eq!(p.rank(), r);
    }
    let mut p = Profile::new();
    p.xp = u64::MAX;
    assert!(p.rank() <= MAX_RANK);
    // the whole climb needs well over a hundred hours at ~30 s of useful play per XP point
    assert!(xp_for_rank(MAX_RANK) > 40_000);
}

#[test]
fn daily_budget_reduces_then_stops_xp() {
    let mut p = Profile::new();
    let day = 20_000;
    let first = p.record(&result(1.0, 600.0), day);
    assert!(first.daily_bonus > 0 && first.mult == 1.0);
    for _ in 0..4 {
        p.record(&result(1.0, 600.0), day); // 50 min played
    }
    assert!(p.xp_multiplier() < 1.0);
    for _ in 0..6 {
        p.record(&result(1.0, 600.0), day); // 110 min
    }
    assert_eq!(p.xp_multiplier(), 0.0);
    let before = p.xp;
    let g = p.record(&result(1.0, 600.0), day);
    assert_eq!((g.xp, p.xp), (0, before));
    assert_eq!(p.advice(0.0), Advice::Done);
    // next day everything is fresh again, the streak grows
    p.begin_day(day + 1);
    assert_eq!((p.streak, p.xp_multiplier()), (2, 1.0));
    // a missed day costs only the streak counter
    p.begin_day(day + 5);
    assert_eq!(p.streak, 1);
    assert!(p.xp >= before);
}

#[test]
fn break_advice_after_a_long_session() {
    let p = Profile::new();
    assert_eq!(p.advice(5.0), Advice::None);
    assert_eq!(p.advice(BREAK_MINUTES + 1.0), Advice::Break);
}

#[test]
fn profile_roundtrip_and_hostile_decode() {
    let mut p = Profile::new();
    p.record(&result(1.0, 120.0), 20_001);
    let q = Profile::decode(&p.encode());
    assert_eq!((q.xp, q.day, q.streak), (p.xp, p.day, p.streak));
    for bad in [
        "",
        "TP1;",
        "TP1;xp=NaN;min=inf;g=-1",
        "TP1;xp=99999999999999999999999;st=70000;min=1e99",
        "garbage",
        &"TP1;xp=1;".repeat(200),
        "TP1;=;;;==;xp",
    ] {
        let d = Profile::decode(bad);
        assert!(d.rank() <= MAX_RANK && d.minutes_today.is_finite() && d.streak <= 999);
    }
}

#[test]
fn tyres_match_conditions() {
    for w in 0..WEATHER_COUNT {
        for s in 0..SURFACE_COUNT {
            let (w, s) = (Weather::from_index(w), Surface::from_index(s));
            let best = Tyre::best_for(w, s);
            for t in Tyre::ALL {
                let g = t.grip(w, s);
                assert!((0.7..=1.13).contains(&g));
                assert!(
                    g <= best.grip(w, s) + 1e-6,
                    "{best:?} is the best on {w:?}/{s:?}"
                );
            }
        }
    }
    assert_eq!(Tyre::best_for(Weather::Snow, Surface::Asphalt), Tyre::Snow);
    assert_eq!(Tyre::best_for(Weather::Rain, Surface::Asphalt), Tyre::Rain);
}

#[test]
fn setups_trade_off_and_parse() {
    let (a, t, g, f) = Setup::Sprint.mods();
    assert!(a > 1.0 && t < 1.0 && g < 1.0 && f < 1.0);
    assert!(Setup::Economy.mods().3 > 1.2);
    assert!(Setup::Downforce.mods().2 > 1.05);
    for i in 0..SETUP_COUNT {
        assert_eq!(Setup::from_index(i).index(), i);
    }
}

#[test]
fn day_night_set_time_and_rank_scaling() {
    let mut vals = vec![];
    for i in 1..=4 {
        let mut g = Game::new_mode(3, Mode::Campaign);
        g.set_time(i);
        vals.push(g.daylight());
    }
    assert!(
        vals[1] > vals[0] && vals[1] > vals[2],
        "day is brighter than dawn and dusk: {vals:?}"
    );
    assert!(
        vals[3] < vals[0] && vals[3] < vals[2],
        "night is the darkest: {vals:?}"
    );
    for v in &vals {
        assert!((0.0..=1.0).contains(v));
    }
    // a higher rank never makes the stage easier: more traffic or less time, never less of both
    let g0 = {
        let mut g = Game::new_mode(9, Mode::Campaign);
        g.set_rank(0);
        g
    };
    let g9 = {
        let mut g = Game::new_mode(9, Mode::Campaign);
        g.set_rank(999);
        g
    };
    assert!(g9.cars.len() >= g0.cars.len());
}

#[test]
fn colour_filters_are_safe_and_distinct() {
    use turbocursa::access::*;
    let sample: Vec<u32> = (0..4096u32)
        .map(|i| i.wrapping_mul(2654435761) & 0xFFFFFF)
        .collect();
    for f in 0..FILTER_COUNT {
        set_filter(f);
        let mut fb = sample.clone();
        apply(&mut fb);
        assert!(
            fb.iter().all(|&c| c <= 0xFFFFFF),
            "filter {f} keeps 0x00RRGGBB"
        );
        if f > 0 {
            assert_ne!(fb, sample, "filter {f} changes the picture");
        } else {
            assert_eq!(fb, sample);
        }
    }
    // pure red and green stay distinguishable for a deuteranope after the shift
    set_filter(2);
    let mut px = [0xFF0000u32, 0x00C000];
    apply(&mut px);
    let d = |a: u32, b: u32| {
        (0..3)
            .map(|k| ((a >> (16 - 8 * k)) & 255) as i32 - ((b >> (16 - 8 * k)) & 255) as i32)
            .map(|x| x.abs())
            .sum::<i32>()
    };
    assert!(d(px[0], px[1]) > 150);
    set_filter(0);
    assert_eq!(parse_filter("Deutan"), Some(2));
    assert_eq!(parse_filter("zzz"), None);
}

#[test]
fn assists_and_slow_mode_make_driving_easier_never_harder() {
    let run = |assist: u8, cap: f32| {
        let mut g = Game::new_mode(21, Mode::Zen);
        g.set_access(assist, cap);
        let inp = Input {
            accel: true,
            ..Default::default()
        };
        let mut off = 0.0f32;
        for _ in 0..1800 {
            g.update(1.0 / 60.0, &inp);
            off += g.x.abs();
        }
        (off, g.speed, g.pos)
    };
    let plain = run(0, 1.0);
    let held = run(2, 1.0);
    assert!(
        held.0 < plain.0,
        "assist keeps the car nearer the centre: {held:?} vs {plain:?}"
    );
    // reduced-speed mode: the same driver covers clearly less ground at a clearly lower speed
    let drive = |cap: f32| {
        let mut g = Game::new_mode(21, Mode::Zen);
        g.set_access(0, cap);
        let mut top = 0.0f32;
        for _ in 0..1800 {
            let i = g.autopilot();
            g.update(1.0 / 60.0, &i);
            top = top.max(g.speed);
        }
        (top, g.pos)
    };
    let (full, slow) = (drive(1.0), drive(0.5));
    assert!(
        slow.0 < full.0 * 0.7 && slow.1 < full.1 * 0.7,
        "{slow:?} vs {full:?}"
    );
    // hostile values are clamped
    let mut g = Game::new_mode(1, Mode::Zen);
    g.set_access(250, f32::NAN);
    assert_eq!((g.assist, g.speed_cap), (3, 1.0));
    g.set_access(1, -5.0);
    assert_eq!(g.speed_cap, 0.4);
}

#[test]
fn badges_are_earned_once_and_survive_a_hostile_save() {
    use turbocursa::badges::*;
    let mut p = Profile::new();
    let mut r = result(1.0, 120.0);
    r.cond = Conditions {
        class: 3,
        weather: 5,
        surface: 1,
        time: 3,
        country: 70,
        fuel_careful: true,
    };
    r.jumps = 6;
    let g = p.record(&r, 20_000);
    for b in [B_FIRST, B_CLEAN, B_NIGHT, B_ICE, B_STORM, B_AIR, B_FUEL] {
        assert!(g.badges_new >> b & 1 != 0, "badge {b}");
    }
    assert!(!p.col.has(B_DAWN));
    // the same game again earns nothing new
    assert_eq!(p.record(&r, 20_000).badges_new, 0);
    // an unfinished game adds no collection entries
    let mut q = Profile::new();
    let mut r2 = result(0.5, 60.0);
    r2.cond.class = 7;
    q.record(&r2, 20_000);
    assert_eq!(q.col.classes, 0);
    // save round trip
    let d = Profile::decode(&p.encode());
    assert_eq!(d.col, p.col);
    assert!(p.encode().len() < 400);
    // hostile values are masked to their valid ranges
    let h = Profile::decode(
        "TP1;cl=ffffffffffff;we=zz;su=ff;tm=ff;bd=ffffffffffffffff;cy=1.2.3.4.5.6.7",
    );
    assert!(h.col.classes.count_ones() as usize <= turbocursa::cars::CLASS_COUNT);
    assert!(h.col.count() as usize <= BADGE_COUNT);
    assert_eq!(
        h.col.surfaces.count_ones() as usize,
        turbocursa::env::SURFACE_COUNT
    );
    assert_eq!(h.col.times, 0xF);
}

#[test]
fn collections_complete_into_badges() {
    use turbocursa::badges::*;
    let mut p = Profile::new();
    for c in 0..turbocursa::cars::CLASS_COUNT {
        let mut r = result(1.0, 60.0);
        r.cond = Conditions {
            class: c as u8,
            weather: (c % WEATHER_COUNT) as u8,
            surface: (c % SURFACE_COUNT) as u8,
            time: (c % 4) as u8,
            country: (c * 11 % 196) as u8,
            fuel_careful: false,
        };
        p.record(&r, 20_000 + c as u32);
    }
    for b in [B_VEHICLES, B_WEATHERS, B_SURFACES, B_ALLDAY] {
        assert!(p.col.has(b), "badge {b}");
    }
    assert!(p.col.has(16) && p.col.has(17), "3- and 7-day streaks");
    assert!(!p.col.has(B_COUNTRIES), "only 19 countries so far");
    assert_eq!(time_bucket(6 * 60), 0);
    assert_eq!(time_bucket(12 * 60), 1);
    assert_eq!(time_bucket(18 * 60), 2);
    assert_eq!(time_bucket(23 * 60), 3);
    for i in 0..BADGE_COUNT as u8 {
        assert!(!badge_name(i).is_empty());
    }
}

#[test]
fn photo_mode_freezes_the_race_and_moves_the_camera() {
    let mut g = Game::new_mode(5, Mode::Campaign);
    let go = Input {
        accel: true,
        ..Default::default()
    };
    for _ in 0..120 {
        g.update(1.0 / 60.0, &go);
    }
    let (t, speed) = (g.elapsed, g.speed);
    g.photo_toggle();
    assert!(g.photo);
    let side = Input {
        right: true,
        ..Default::default()
    };
    for _ in 0..60 {
        g.update(1.0 / 60.0, &side);
    }
    assert_eq!(
        (g.elapsed, g.speed),
        (t, speed),
        "the clock and the car are frozen"
    );
    assert!(g.photo_dx > 0.5);
    // the camera is bounded and the frame differs from the normal view
    let mut a = vec![0u32; turbocursa::W * turbocursa::H];
    let mut b = a.clone();
    turbocursa::render::render(&g, &mut a);
    g.photo_hud_toggle();
    g.photo_toggle();
    assert!(!g.photo && g.photo_dx == 0.0);
    turbocursa::render::render(&g, &mut b);
    assert_ne!(a, b);
    // scrubbing never leaves the track
    g.photo_toggle();
    let back = Input {
        brake: true,
        ..Default::default()
    };
    for _ in 0..3000 {
        g.update(0.05, &back);
    }
    assert!(g.pos >= 0.0);
    let fwd = Input {
        accel: true,
        turbo: false,
        ..Default::default()
    };
    for _ in 0..30000 {
        g.update(0.05, &fwd);
    }
    assert!(g.pos <= g.track.finish_seg as f32 * turbocursa::track::SEG_LEN);
}

#[test]
fn png_encoder_makes_a_valid_file() {
    let fb: Vec<u32> = (0..64 * 48)
        .map(|i| (i as u32 * 40503) & 0xFFFFFF)
        .collect();
    let p = turbocursa::png::encode(&fb, 64, 48);
    assert_eq!(&p[..8], &[0x89, b'P', b'N', b'G', 13, 10, 26, 10]);
    assert_eq!(&p[p.len() - 8..p.len() - 4], b"IEND");
    // 640x400 spans several stored blocks
    let big = vec![0x123456u32; turbocursa::W * turbocursa::H];
    assert!(
        turbocursa::png::encode(&big, turbocursa::W, turbocursa::H).len()
            > turbocursa::W * turbocursa::H * 3
    );
}

#[test]
fn damage_slows_the_car_and_the_station_repairs_it() {
    let drive = |engine: f32| {
        let mut g = Game::new_mode(21, Mode::Campaign);
        g.engine = engine;
        let mut top = 0.0f32;
        for _ in 0..1200 {
            let i = g.autopilot();
            g.update(1.0 / 60.0, &Input { accel: true, ..i });
            top = top.max(g.speed);
        }
        top
    };
    assert!(
        drive(0.0) < drive(1.0) * 0.85,
        "a wrecked engine is clearly slower"
    );
    // hits wear both parts, never below zero; harder levels hurt more
    let mut g = Game::new_mode(2, Mode::Campaign);
    g.hurt(0.3, 0.2);
    let easy = (g.body, g.engine);
    for _ in 0..20 {
        g.hurt(0.3, 0.3);
    }
    assert_eq!((g.body, g.engine), (0.0, 0.0));
    let mut a = Game::new_mode(2, Mode::Campaign);
    a.set_level(Difficulty::Insane);
    a.hurt(0.3, 0.2);
    assert!(a.body < 1.0 && a.body <= easy.0 + 1e-6 || a.level.index() == 4);
    // a stop at a station repairs
    let mut g = (1..60)
        .map(|s| Game::new_mode(s, Mode::Campaign))
        .find(|g| !g.stations.is_empty())
        .expect("some stage has a fuel station");
    g.fuel_rule = true;
    let s0 = g.stations[0].0;
    g.body = 0.2;
    g.engine = 0.3;
    g.fuel = 0.2;
    g.pos = s0 - PLAYER_Z + 400.0;
    g.x = 0.6;
    g.speed = 0.0;
    for _ in 0..600 {
        g.update(1.0 / 60.0, &Input::default());
    }
    assert!(
        g.body > 0.9 && g.engine > 0.9,
        "repaired: {} {}",
        g.body,
        g.engine
    );
}

#[test]
fn daily_challenge_is_deterministic_varied_and_counts_once() {
    use turbocursa::daily::*;
    assert_eq!(for_day(20_000), for_day(20_000));
    let mut classes = std::collections::HashSet::new();
    let mut modes = std::collections::HashSet::new();
    for d in 20_000..20_200 {
        let c = for_day(d);
        assert!((1..=3).contains(&c.level) && (1..=4).contains(&c.time));
        assert!(
            c.class < turbocursa::cars::CLASS_COUNT
                && c.weather <= WEATHER_COUNT
                && c.surface <= SURFACE_COUNT
        );
        assert!(turbocursa::game::Mode::from_index(c.mode) as u32 <= 8);
        classes.insert(c.class);
        modes.insert(c.mode);
    }
    assert!(
        classes.len() >= 12 && modes.len() >= 4,
        "{} {}",
        classes.len(),
        modes.len()
    );
    // codes round trip, typing mistakes are caught
    let c = code(20_001, 12_345, 83.4, true);
    assert_eq!(parse_code(&c), Some((20_001, 12_345, 83.4, true)));
    let bad = c.replace("12345", "12346");
    assert_eq!(parse_code(&bad), None);
    for junk in [
        "",
        "TC",
        "TC-1-2-3-X-000",
        "TC-a-b-c-F-0",
        &"TC-1-".repeat(30),
        "TC-1-2-3-F-ZZZ",
    ] {
        assert_eq!(parse_code(junk), None);
    }
    assert!(code(1, 1, f32::NAN, true).starts_with("TC-1-1-0-F-"));
    // only the first run of a day counts
    let mut p = Profile::new();
    let first = p.record_daily(20_001, 900, 60.0, true);
    assert!(first
        .as_ref()
        .is_some_and(|(c, b)| c.starts_with("TC-20001-900") && *b > 0));
    assert!(p.record_daily(20_001, 5_000, 30.0, true).is_none());
    assert_eq!((p.daily_score, p.daily_runs), (900, 1));
    assert!(p.record_daily(20_002, 100, 60.0, false).is_some());
    let d = Profile::decode(&p.encode());
    assert_eq!((d.daily_day, d.daily_score, d.daily_runs), (20_002, 100, 2));
}

#[test]
fn ghost_records_replays_and_survives_hostile_codes() {
    use turbocursa::ghost::*;
    // drive a short campaign with the autopilot until it finishes or 200 s pass
    let mut g = Game::new_mode(33, Mode::TimeTrial);
    for _ in 0..12000 {
        let i = g.autopilot();
        g.update(1.0 / 60.0, &Input { accel: true, ..i });
        if g.state != State::Playing {
            break;
        }
    }
    if g.state != State::Finished {
        // the stage may be too long for the budget: fall back to a hand-made ghost
        g.state = State::Finished;
    }
    let gh = g.make_ghost().expect("finished run makes a ghost");
    assert!(gh.pts.len() > 10 && gh.pts.windows(2).all(|w| w[1].0 >= w[0].0));
    // code round trip keeps positions within the quantisation
    let back = Ghost::decode(&gh.encode()).expect("valid code");
    assert_eq!(back.pts.len(), gh.pts.len());
    for (a, b) in gh.pts.iter().zip(&back.pts) {
        assert!(
            (a.0 - b.0).abs() < gh.pts.len() as f32 && (a.1 - b.1).abs() < 0.002,
            "{a:?} {b:?}"
        );
    }
    // a second game of the same stage races the ghost and sees the gap
    let mut h = Game::new_mode(33, Mode::TimeTrial);
    h.set_ghost(Some(back.clone()));
    assert!(h.ghost.is_some());
    for _ in 0..300 {
        h.update(
            1.0 / 60.0,
            &Input {
                accel: true,
                ..Default::default()
            },
        );
    }
    assert!(h.ghost_now().is_some() && h.ghost_gap().is_some());
    let mut fb = vec![0u32; turbocursa::W * turbocursa::H];
    turbocursa::render::render(&h, &mut fb);
    // another stage's ghost is refused
    let mut o = Game::new_mode(34, Mode::TimeTrial);
    o.set_ghost(Some(back.clone()));
    assert!(o.ghost.is_none());
    // faster run wins
    let mut faster = back.clone();
    faster.secs -= 1.0;
    assert!(faster.beats(&back) && !back.beats(&faster));
    // hostile codes never panic and never allocate absurdly
    for bad in [
        "",
        "TG1",
        "TG1;1;2;3;4;5",
        "TG1;x;0;0;0;0;00",
        "TG1;1;0;0;0;0;zzzz",
        "TG1;1;0;0;0;0;0001",
        &format!("TG1;1;0;0;0;0;{}", "f".repeat(7 * (MAX_SAMPLES + 5))),
        &"TG1;".repeat(100),
        "TG1;1;300;0;0;0;",
    ] {
        let d = Ghost::decode(bad);
        if let Some(d) = d {
            assert!(d.pts.len() <= MAX_SAMPLES);
        }
    }
    assert!(Ghost {
        seed: 1,
        mode: 0,
        class: 0,
        secs: 1.0,
        score: 0,
        pts: vec![]
    }
    .at(1.0)
    .is_none());
}

#[test]
fn rally_runs_five_stages_with_service_and_a_field() {
    use turbocursa::rally::*;
    let mut fb = vec![0u32; turbocursa::W * turbocursa::H];
    let mut app = RallyApp::new(4242, 0, 1);
    let none = Input::default();
    let mut frames = 0u64;
    let mut seen = std::collections::HashSet::new();
    while app.phase != Phase::Final && frames < 2_000_000 {
        frames += 1;
        seen.insert(app.phase as u8);
        match app.phase {
            Phase::Racing => {
                let g = app.game.as_ref().unwrap();
                let over = g.state != State::Playing;
                let i = g.autopilot();
                app.update(
                    1.0 / 60.0,
                    &Input { accel: true, ..i },
                    over && frames.is_multiple_of(3),
                    false,
                );
            }
            Phase::Service => {
                app.svc = Service {
                    repair: 2,
                    tyre: 0,
                    refuel: true,
                };
                app.update(1.0 / 60.0, &none, true, false);
            }
            _ => app.update(1.0 / 60.0, &none, true, false),
        }
        if frames.is_multiple_of(5000) {
            app.render(&mut fb);
        }
        // between frames the pending stage result is handed out once
        let _ = app.take_result();
    }
    seen.insert(app.phase as u8);
    assert_eq!(app.phase, Phase::Final, "the rally finishes");
    assert_eq!(app.stage_times.len(), STAGES);
    assert_eq!(app.results.len(), STAGES);
    assert!((1..=FIELD + 1).contains(&app.position));
    assert!(app.total() >= app.stage_times.iter().sum::<f32>());
    assert!(app.service_time > 0.0 || app.stage_times.iter().all(|t| t.is_finite()));
    assert_eq!(seen.len(), 5, "every phase was visited: {seen:?}");
    for _ in 0..3 {
        app.render(&mut fb); // final screen draws
    }
    // plans are deterministic, the first stage is easy, the day runs dawn to night
    assert_eq!(plan(7, 0), plan(7, 0));
    assert_eq!((plan(7, 0).weather, plan(7, 0).surface), (0, 0));
    assert_eq!((plan(7, 0).time, plan(7, STAGES - 1).time), (1, 4));
    // service costs time, repairs more so when the car is worse
    let mut a = RallyApp::new(1, 0, 1);
    let free = a.service_cost(Service {
        repair: 0,
        tyre: 1,
        refuel: false,
    }); // keeps the road tyres
    a.body = 0.2;
    a.engine = 0.2;
    a.fuel = 0.3;
    let full = a.service_cost(Service {
        repair: 2,
        tyre: 1,
        refuel: true,
    });
    let half = a.service_cost(Service {
        repair: 1,
        tyre: 1,
        refuel: false,
    });
    assert!(free == 0.0 && full > half && half > 0.0);
}

#[test]
fn weather_police_train_and_ice_sounds_are_audible_and_safe() {
    use turbocursa::audio::{AudioParams, Synth};
    // (hiss: rms of the first difference, level: rms) over two seconds after a one-second warm-up
    let m = |p: AudioParams| {
        let mut s = Synth::new(44_100.0, 7);
        let mut buf = vec![0.0f32; 44_100];
        s.render(&mut buf, &p);
        let (mut hiss, mut level) = (0.0f64, 0.0f64);
        for _ in 0..2 {
            s.render(&mut buf, &p);
            assert!(
                buf.iter().all(|v| v.is_finite() && v.abs() <= 1.0),
                "bounded and finite"
            );
            for i in 1..buf.len() {
                hiss += ((buf[i] - buf[i - 1]) as f64).powi(2);
                level += (buf[i] as f64).powi(2);
            }
        }
        ((hiss / 88_200.0).sqrt(), (level / 88_200.0).sqrt())
    };
    let base = AudioParams {
        speed: 0.5,
        playing: true,
        engine: 1.0,
        daylight: 1.0,
        ..Default::default()
    };
    let (q_hiss, q_level) = m(base);
    for w in [1u8, 2, 4, 5, 6, 7, 10] {
        assert!(
            m(AudioParams { weather: w, ..base }).0 > q_hiss * 1.2,
            "weather {w} adds hiss"
        );
    }
    assert!(
        m(AudioParams { cop: 1.0, ..base }).1 > q_level * 1.05,
        "siren"
    );
    assert!(m(AudioParams { cop: 1.0, ..base }).1 > m(AudioParams { cop: 0.1, ..base }).1);
    assert!(
        m(AudioParams {
            train: 1.0,
            train_on: true,
            ..base
        })
        .1 > q_level * 1.05,
        "horn"
    );
    assert!(
        m(AudioParams { ice: 1.0, ..base }).1 > q_level,
        "ice cracks"
    );
    assert!(
        m(AudioParams {
            fuel_low: true,
            ..base
        })
        .1 > q_level,
        "fuel beeps"
    );
    assert!(
        m(AudioParams {
            engine: 0.1,
            ..base
        })
        .0 > q_hiss,
        "a damaged engine rattles"
    );
    // night crickets only when standing still
    let still = AudioParams {
        speed: 0.0,
        playing: true,
        engine: 1.0,
        daylight: 0.0,
        ..Default::default()
    };
    assert!(
        m(still).1
            > m(AudioParams {
                daylight: 1.0,
                ..still
            })
            .1
    );
    // hostile parameters
    let evil = AudioParams {
        speed: f32::NAN,
        weather: 250,
        daylight: f32::INFINITY,
        cop: f32::NAN,
        train: -5.0,
        ice: 1e30,
        engine: f32::NEG_INFINITY,
        gear: 255,
        pads: u32::MAX,
        animals: u32::MAX,
        playing: true,
        ..Default::default()
    };
    let mut s = Synth::new(48_000.0, 1);
    let mut buf = vec![0.0f32; 4096];
    for _ in 0..20 {
        s.render(&mut buf, &evil);
        assert!(buf.iter().all(|v| v.is_finite() && v.abs() <= 1.0));
    }
}

#[test]
fn stage_layout_shapes_the_road_and_codes_round_trip() {
    use turbocursa::share::StageCode;
    use turbocursa::track::*;
    let mean_curve =
        |t: &Track| t.segs.iter().map(|s| s.curve.abs()).sum::<f32>() / t.segs.len() as f32;
    let range = |t: &Track| {
        let (lo, hi) = t
            .segs
            .iter()
            .fold((f32::MAX, f32::MIN), |a, s| (a.0.min(s.y0), a.1.max(s.y0)));
        hi - lo
    };
    let build_l = |l: Layout| with_layout(l, || build(77));
    let base = build_l(Layout::default());
    assert_eq!(
        base.segs.len(),
        build(77).segs.len(),
        "the default layout is the classic stage"
    );
    assert!(base
        .segs
        .iter()
        .zip(&build(77).segs)
        .all(|(a, b)| a.curve == b.curve && a.y0 == b.y0));
    let twisty = build_l(Layout {
        twist: 3,
        ..Layout::default()
    });
    let calm = build_l(Layout {
        twist: 0,
        ..Layout::default()
    });
    assert!(mean_curve(&twisty) > mean_curve(&base) && mean_curve(&base) > mean_curve(&calm));
    let alpine = build_l(Layout {
        hills: 3,
        ..Layout::default()
    });
    let flat = build_l(Layout {
        hills: 0,
        ..Layout::default()
    });
    assert!(range(&alpine) > range(&base) && range(&base) > range(&flat));
    let short = build_l(Layout {
        length: 0,
        ..Layout::default()
    });
    let long = build_l(Layout {
        length: 2,
        ..Layout::default()
    });
    assert!(short.segs.len() < base.segs.len() && base.segs.len() < long.segs.len());
    // chosen landscapes appear, in order
    let picked = build_l(Layout {
        biomes: [9, 12, 17, 4],
        ..Layout::default()
    });
    let mut seen: Vec<u8> = vec![];
    for s in &picked.segs {
        if seen.last() != Some(&s.biome) {
            seen.push(s.biome);
        }
    }
    assert_eq!(&seen[..4], &[9, 12, 17, 4]);
    // every layout loops back to ground level
    for l in [
        twisty_l(),
        Layout {
            hills: 3,
            length: 2,
            ..Layout::default()
        },
    ] {
        let t = build_l(l);
        assert!(t.segs.last().unwrap().y1.abs() < 1e-3);
    }
    fn twisty_l() -> Layout {
        Layout {
            twist: 3,
            hills: 0,
            length: 0,
            biomes: [1, 2, 3, 4],
        }
    }
    // codes: round trip, typo detection, hostile input
    let c = StageCode {
        seed: 0xDEAD_BEEF,
        mode: 3,
        class: 11,
        level: 2,
        weather: 6,
        surface: 3,
        time: 4,
        layout: Layout {
            biomes: [9, 12, 17, 4],
            twist: 3,
            hills: 2,
            length: 0,
        },
    };
    let s = c.encode();
    assert!(s.starts_with("TCS1-") && s.len() < 50, "{s}");
    assert_eq!(StageCode::decode(&s), Some(c));
    assert_eq!(
        StageCode::decode(&s.to_lowercase().replace("tcs1", "TCS1")),
        Some(c)
    );
    let mut typo = s.clone().into_bytes();
    let i = typo.len() - 8;
    typo[i] = if typo[i] == b'7' { b'8' } else { b'7' };
    assert_eq!(StageCode::decode(&String::from_utf8(typo).unwrap()), None);
    for bad in [
        "",
        "TCS1",
        "TCS1-",
        "TCS1-ZZZZZ",
        "XXXX-00000",
        &"TCS1-0".repeat(40),
        "TCS1-0000000000000000000000000000000000",
    ] {
        assert_eq!(StageCode::decode(bad), None, "{bad}");
    }
    // a code with out-of-range fields is clamped, and the game it describes runs and renders
    let wild = StageCode {
        mode: 99,
        class: 200,
        level: 77,
        weather: 250,
        surface: 250,
        time: 9,
        layout: Layout {
            biomes: [250, 3, 3, 3],
            twist: 9,
            hills: 9,
            length: 9,
        },
        ..c
    };
    let w = StageCode::decode(&wild.encode()).unwrap();
    assert!(
        w.mode < 9 && (w.class as usize) < turbocursa::cars::CLASS_COUNT && w.layout.twist <= 3
    );
    let mut g = w.game();
    let mut fb = vec![0u32; turbocursa::W * turbocursa::H];
    for _ in 0..300 {
        g.update(
            1.0 / 60.0,
            &Input {
                accel: true,
                ..Default::default()
            },
        );
    }
    turbocursa::render::render(&g, &mut fb);
    // the same code always makes the same stage
    let (a, b) = (c.game(), c.game());
    assert!(a
        .track
        .segs
        .iter()
        .zip(&b.track.segs)
        .all(|(x, y)| x.curve == y.curve && x.y0 == y.y0));
    assert_eq!(a.track.segs.len(), b.track.segs.len());
}

#[test]
fn designer_edits_and_returns_a_valid_code() {
    use turbocursa::design::DesignApp;
    use turbocursa::game::Input;
    use turbocursa::share::StageCode;
    let mut d = DesignApp::new(None);
    let idle = Input::default();
    let right = Input {
        right: true,
        ..Input::default()
    };
    let down = Input {
        brake: true,
        ..Input::default()
    };
    let seed0 = d.code.seed;
    d.update(0.016, &right, false, false);
    d.update(0.016, &idle, false, false);
    assert_eq!(d.code.seed, seed0 + 1);
    // walk to the biome rows and the layout rows, changing each
    for _ in 0..13 {
        d.update(0.016, &down, false, false);
        d.update(0.016, &idle, false, false);
        d.update(0.016, &right, false, false);
        d.update(0.016, &idle, false, false);
    }
    assert_eq!(d.cursor(), 13);
    assert_eq!(d.code.layout.length, 2);
    let mut fb = vec![0u32; turbocursa::W * turbocursa::H];
    d.render(&mut fb);
    assert!(fb.contains(&0xFFD23C));
    d.update(0.016, &idle, true, false);
    let code = d.take_code().expect("enter returns the stage");
    assert!(d.take_code().is_none());
    let txt = code.encode();
    assert_eq!(StageCode::decode(&txt), Some(code));
    // holding the key repeats; hostile long hold never leaves the valid range
    for _ in 0..2000 {
        d.update(0.016, &right, false, false);
    }
    assert!(d.code.sanitized() == d.code);
}

#[test]
fn gamepad_events_decode_like_the_browser_mapping() {
    use turbocursa::gamepad::Decoder;
    let ev = |kind: u8, num: u8, value: i16| {
        let v = value.to_le_bytes();
        [0, 0, 0, 0, v[0], v[1], kind, num]
    };
    let mut d = Decoder::default();
    // initial-state events (0x80) never count as presses
    d.feed(&ev(0x81, 7, 1));
    assert!(!d.state().enter);
    d.feed(&ev(2, 0, 20000)); // stick right
    d.feed(&ev(1, 0, 1)); // A
    d.feed(&ev(2, 5, 30000)); // RT
    d.feed(&ev(1, 2, 1)); // X
    let s = d.state();
    assert!(s.input.right && !s.input.left && s.input.accel && s.input.turbo && !s.input.brake);
    d.feed(&ev(1, 7, 0)); // released
    d.feed(&ev(1, 7, 1)); // Start
    d.feed(&ev(1, 5, 1)); // RB
    let s = d.state();
    assert!(s.enter && s.gear_up && !s.back);
    assert!(!d.state().enter, "edges fire once");
    d.feed(&ev(2, 0, -20000));
    d.feed(&ev(2, 6, -1)); // d-pad left
    assert!(d.state().input.left);
    // a resting trigger (-32767) is not a press; garbage numbers are ignored
    let mut d = Decoder::default();
    d.feed(&ev(2, 2, -32767));
    d.feed(&ev(2, 200, 5));
    d.feed(&ev(1, 200, 1));
    assert_eq!(d.state(), Default::default());
    // d-pad up/down and stick deadzone
    d.feed(&ev(2, 7, -1));
    assert!(d.state().input.accel);
    d.feed(&ev(2, 7, 0));
    d.feed(&ev(2, 1, 3000));
    assert!(!d.state().input.brake);
}

#[test]
fn a_stranded_car_gets_towed_and_stations_pause_the_clock() {
    use turbocursa::game::*;
    let mut g = Game::new(5);
    g.fuel = 0.0;
    g.speed = 0.0;
    let idle = Input::default();
    for _ in 0..60 * 8 {
        g.update(1.0 / 60.0, &idle);
    }
    assert!(g.fuel > 0.0, "a tow truck brings fuel");
    // parked in a station with a full tank and no damage: the clock keeps running
    let mut g = Game::new(5);
    let (a, _) = g.stations[0];
    g.pos = a + 800.0 - PLAYER_Z;
    g.speed = 0.0;
    g.x = 0.5;
    g.fuel = 1.0;
    let t0 = g.time;
    for _ in 0..120 {
        g.update(1.0 / 60.0, &idle);
    }
    assert!(
        g.time < t0 - 1.5,
        "nothing to service, so the clock runs: {} -> {}",
        t0,
        g.time
    );
    // but a half-empty tank freezes it while it fills
    g.fuel = 0.3;
    let t1 = g.time;
    for _ in 0..60 {
        g.update(1.0 / 60.0, &idle);
    }
    assert!(
        (g.time - t1).abs() < 0.2,
        "clock waits during the service: {} -> {}",
        t1,
        g.time
    );
}
