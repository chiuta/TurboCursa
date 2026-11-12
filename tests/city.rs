//! Cities, traffic rules, jams with a detour, emergency vehicles and accident scenes.
use turbocursa::city::{self, CYCLE, GREEN, YELLOW};
use turbocursa::game::*;
use turbocursa::i18n::{self, LANGS};
use turbocursa::{font, track};

fn game(seed: u64, level: Difficulty) -> Game {
    let mut g = Game::new_mode(seed, Mode::Campaign);
    g.set_level(level);
    g.cars.clear();
    g.jam_cars.clear();
    g.invuln = 0.0;
    g
}

/// Put the player `d` units before world z, at lateral x and speed v.
fn place(g: &mut Game, z: f32, d: f32, x: f32, v: f32) {
    g.pos = z - PLAYER_Z - d;
    g.x = x;
    g.speed = v;
    g.state = State::Playing;
}

fn step(g: &mut Game, n: usize, accel: bool) {
    for _ in 0..n {
        g.update(
            1.0 / 60.0,
            &Input {
                accel,
                ..Input::default()
            },
        );
    }
}

fn lit_light(g: &Game) -> city::Light {
    *g.city
        .lights
        .iter()
        .find(|l| l.lit)
        .expect("a signalled crossing")
}

#[test]
fn every_stage_has_a_city_and_the_plan_is_deterministic_and_clear() {
    for seed in 1..60u64 {
        let a = Game::new_mode(seed, Mode::Campaign);
        let b = Game::new_mode(seed, Mode::Campaign);
        assert_eq!(a.city.zones.len(), b.city.zones.len());
        assert_eq!(a.city.lights.len(), b.city.lights.len());
        assert!(!a.city.zones.is_empty(), "seed {seed}: no city");
        assert!(!a.city.jams.is_empty(), "seed {seed}: no jam");
        assert!(!a.city.scenes.is_empty(), "seed {seed}: no accident scene");
        for z in &a.city.zones {
            assert!((z.a..z.b).all(|i| a.track.segs[i].feat == track::F_NONE));
            assert!((z.a..z.b).all(|i| a.track.segs[i].props.is_empty()));
            for j in &a.city.jams {
                assert!(z.b < j.exit_a() || z.a > j.b, "city and jam overlap");
            }
        }
        for c in &a.crossings {
            assert!(a.city.zones.iter().all(|z| {
                let s = (c.z / 200.0) as usize;
                s < z.a || s > z.b
            }));
        }
    }
}

#[test]
fn no_jams_without_traffic() {
    assert!(Game::new_mode(7, Mode::TimeTrial).city.jams.is_empty());
    assert!(Game::new_mode(7, Mode::Zen).jam_cars.is_empty());
}

#[test]
fn light_cycle_and_pedestrians_follow_the_phases() {
    let mut l = city::Light {
        z: 0.0,
        phase: 0.0,
        lit: true,
        dir: 1.0,
        period: 9.0,
    };
    assert_eq!(l.state(1.0), 0);
    assert_eq!(l.state(GREEN + 0.5), 1);
    assert_eq!(l.state(GREEN + YELLOW + 0.5), 2);
    assert!(l.ped_x(1.0).is_none());
    assert!(!l.ped_on_road(GREEN + YELLOW + 0.2));
    assert!(l.ped_on_road(GREEN + YELLOW + 2.0));
    assert!(l.ped_x(CYCLE - 0.1).is_none() || !l.ped_on_road(CYCLE - 0.1));
    l.lit = false;
    assert_eq!(l.state(3.0), 0);
}

#[test]
fn running_a_red_light_calls_the_police_or_fines_on_easy() {
    for (level, police) in [(Difficulty::Normal, true), (Difficulty::Easy, false)] {
        let mut g = game(4, level);
        let l = lit_light(&g);
        g.elapsed = (GREEN + YELLOW + 0.1 - l.phase).rem_euclid(CYCLE);
        place(&mut g, l.z, 400.0, 0.0, 6000.0);
        let t0 = g.time;
        step(&mut g, 12, true);
        assert_eq!(g.violations, 1, "{level:?}");
        if police {
            assert!(!g.cops.is_empty() || g.bust > 0.05);
        } else {
            assert!(g.cops.is_empty());
            assert_eq!(g.banner, "FINE");
            assert!(g.time < t0 - 2.0);
        }
        assert_eq!(g.civic, 0);
    }
}

#[test]
fn waiting_for_the_green_and_giving_way_pay_a_bonus() {
    let mut g = game(4, Difficulty::Normal);
    let l = lit_light(&g);
    g.elapsed = (GREEN + YELLOW + 0.1 - l.phase).rem_euclid(CYCLE);
    place(&mut g, l.z, 1500.0, 0.0, 0.0);
    step(&mut g, 6, false);
    assert_eq!(g.violations, 0);
    // the light turns green: drive off
    g.elapsed = (0.5 - l.phase).rem_euclid(CYCLE);
    place(&mut g, l.z, 600.0, 0.0, 1200.0);
    let (s0, t0) = (g.score, g.time);
    g.banner_t = 0.0;
    step(&mut g, 40, true);
    assert_eq!(g.violations, 0);
    assert!(g.good_acts >= 1 && g.civic >= 1);
    assert!(g.score > s0 && g.time > t0 - 1.0);
    assert_eq!(g.banner, "GOOD DRIVER");
}

#[test]
fn a_pedestrian_on_the_crossing_is_hit_at_speed_and_spared_at_a_crawl() {
    let mut g = game(4, Difficulty::Normal);
    let l = lit_light(&g);
    // mid-walk: the pedestrians are in the middle of the road
    g.elapsed = (GREEN + YELLOW + 0.5 + 1.5 - l.phase).rem_euclid(CYCLE);
    place(&mut g, l.z, 400.0, 0.0, 6000.0);
    step(&mut g, 8, true);
    assert_eq!(g.peds_hit, 1);
    assert!(g.violations >= 1);

    let mut g = game(4, Difficulty::Normal);
    g.elapsed = (GREEN + YELLOW + 0.5 + 1.5 - l.phase).rem_euclid(CYCLE);
    place(&mut g, l.z, 100.0, 0.0, 900.0);
    step(&mut g, 14, false);
    assert_eq!(g.peds_hit, 0);
    assert_eq!(g.violations, 0);
    assert!(g.peds_yielded >= 1 && g.good_acts >= 1);
}

#[test]
fn speeding_in_a_city_is_an_offence_but_not_outside() {
    let mut g = game(4, Difficulty::Normal);
    let z = g.city.zones[0];
    let mid = (z.a + 20) as f32 * 200.0;
    g.shield_t = 0.0;
    place(&mut g, mid, 0.0, 0.0, MAX_SPEED * 0.8);
    for _ in 0..240 {
        g.speed = MAX_SPEED * 0.8;
        step(&mut g, 1, true);
        // keep clear of crossings: the check is about the limit
        if g.pos > (z.b as f32 - 60.0) * 200.0 {
            break;
        }
    }
    assert!(g.violations >= 1);
    let mut g = game(4, Difficulty::Normal);
    let mut z = 30000.0;
    while g.city.in_city((z / 200.0) as usize)
        || g.city.jam_at((z / 200.0) as usize).is_some()
        || g.city.exit_at((z / 200.0) as usize).is_some()
    {
        z += 4000.0;
    }
    place(&mut g, z, 0.0, 0.0, MAX_SPEED * 0.8);
    assert!(!g.city.in_city((z / 200.0) as usize));
    step(&mut g, 60, true);
    assert_eq!(g.violations, 0);
}

#[test]
fn a_clean_city_pays_time_and_raises_the_multiplier() {
    let mut g = game(4, Difficulty::Normal);
    let z = g.city.zones[0];
    g.elapsed = 0.0;
    // all lights green: remove the crosswalks, then drive through at the limit
    g.city.lights.clear();
    place(&mut g, (z.a + 1) as f32 * 200.0, 0.0, 0.0, MAX_SPEED * 0.45);
    let t0 = g.time;
    let mut guard = 0;
    while g.pos < (z.b + 6) as f32 * 200.0 && guard < 4000 {
        g.speed = MAX_SPEED * 0.45;
        step(&mut g, 1, true);
        guard += 1;
    }
    assert_eq!(g.violations, 0);
    assert_eq!(g.clean_cities, 1);
    assert!(g.civic >= 2 && g.civic_mult() >= 1);
    assert!(g.time > t0 - 10.0, "the clean-city bonus offsets the drive");
}

#[test]
fn a_violation_resets_the_multiplier() {
    let mut g = game(4, Difficulty::Normal);
    g.civic = 4;
    assert_eq!(g.civic_mult(), 2);
    g.civic = 40;
    assert_eq!(g.civic_mult(), 3);
    let l = lit_light(&g);
    g.elapsed = (GREEN + YELLOW + 0.1 - l.phase).rem_euclid(CYCLE);
    place(&mut g, l.z, 400.0, 0.0, 6000.0);
    step(&mut g, 12, true);
    assert_eq!(g.civic, 0);
}

#[test]
fn the_exit_leads_around_the_jam_and_restores_the_road() {
    let seed = 4;
    let fresh = Game::new_mode(seed, Mode::Campaign);
    let mut g = game(seed, Difficulty::Normal);
    let j = g.city.jams[0];
    // the slip road: x beyond the right edge inside the window
    place(&mut g, (j.exit_a() + 8) as f32 * 200.0, 0.0, 1.25, 6000.0);
    step(&mut g, 2, true);
    assert_eq!(g.detour, Some(0));
    assert_eq!(g.detours, 1);
    let mid = j.a + (j.b - j.a) / 2;
    assert!(
        (g.track.segs[mid].curve - fresh.track.segs[mid].curve).abs() > 0.05
            || (g.track.segs[mid + 7].curve - fresh.track.segs[mid + 7].curve).abs() > 0.05
    );
    // the detour caps the speed and the street holds no queue
    for _ in 0..30 {
        g.speed = MAX_SPEED;
        step(&mut g, 1, true);
        assert!(g.speed <= MAX_SPEED * city::DETOUR_CAP + 1.0);
    }
    assert_eq!(g.crashes, 0);
    // back on the main road
    place(&mut g, (j.b + 2) as f32 * 200.0, 0.0, 0.0, 6000.0);
    step(&mut g, 2, true);
    assert_eq!(g.detour, None);
    for i in j.a..j.b {
        assert_eq!(g.track.segs[i].curve, fresh.track.segs[i].curve, "seg {i}");
    }
}

#[test]
fn the_exit_is_faster_than_the_queue_and_the_radar_says_so() {
    for seed in 1..30u64 {
        let g = Game::new_mode(seed, Mode::Campaign);
        for j in &g.city.jams {
            assert!(j.eta_exit() + 8.0 < j.eta_main(), "seed {seed}");
        }
    }
}

#[test]
fn the_queue_stops_a_fast_car_but_lets_a_careful_one_creep_through() {
    let mut g = game(4, Difficulty::Normal);
    let _ = &g;
    g = Game::new_mode(4, Mode::Campaign);
    g.cars.clear();
    g.invuln = 0.0;
    let c = g.jam_cars[3].clone();
    place(&mut g, c.z, 700.0, c.x, 9000.0);
    step(&mut g, 20, true);
    assert!(g.crashes >= 1, "driving into the queue is a crash");

    let mut g = Game::new_mode(4, Mode::Campaign);
    g.cars.clear();
    let c = g.jam_cars[3].clone();
    place(&mut g, c.z, 700.0, c.x, 700.0);
    step(&mut g, 60, false);
    assert_eq!(g.crashes, 0, "creeping in contact is only a nudge");
    assert!(g.speed <= 500.1);
}

#[test]
fn the_queue_always_leaves_a_corridor() {
    for seed in 1..30u64 {
        let g = Game::new_mode(seed, Mode::Campaign);
        for j in &g.city.jams {
            let cars = city::jam_cars(g.track.length, j, seed);
            // walk the road in rows: at every row some lateral position stays clear
            let mut s = j.a as f32 * 200.0;
            while s < j.b as f32 * 200.0 {
                let free = (-9..=9).map(|k| k as f32 * 0.1).any(|x| {
                    cars.iter()
                        .filter(|c| (c.0 - s).abs() < 200.0)
                        .all(|c| (c.1 - x).abs() > kind_half(c.2) + 0.13)
                });
                assert!(free, "seed {seed}: row at {s} is blocked");
                s += 200.0;
            }
        }
    }
}

#[test]
fn an_ambulance_overtakes_and_making_room_pays() {
    let mut g = game(5, Difficulty::Normal);
    // somewhere with no jam, scene or detour about
    let mut z = 20000.0;
    while g.city.jam_at((z / 200.0) as usize).is_some()
        || g.city.scenes.iter().any(|&s| (s - z).abs() < 60000.0)
        || g.city.exit_at((z / 200.0) as usize).is_some()
    {
        z += 5000.0;
    }
    g.elapsed = 30.0;
    place(&mut g, z, 0.0, 0.0, 6000.0);
    step(&mut g, 2, true);
    assert_eq!(g.emerg.len(), 1);
    let kind_ok = g.emerg[0].kind <= 1;
    assert!(kind_ok);
    let (s0, mult) = (g.score, g.civic_mult());
    for _ in 0..60 * 16 {
        g.speed = 6000.0;
        step(&mut g, 1, true);
        if g.emerg_passed > 0 {
            break;
        }
    }
    assert_eq!(g.emerg_passed, 1);
    assert!(g.score >= s0 + 500 * mult);
    assert!(g.good_acts >= 1);
}

#[test]
fn blocking_an_ambulance_is_an_offence() {
    let mut g = game(5, Difficulty::Normal);
    let mut z = 20000.0;
    while g.city.jam_at((z / 200.0) as usize).is_some()
        || g.city.scenes.iter().any(|&s| (s - z).abs() < 60000.0)
        || g.city.exit_at((z / 200.0) as usize).is_some()
    {
        z += 5000.0;
    }
    g.elapsed = 30.0;
    place(&mut g, z, 0.0, 0.0, 3000.0);
    step(&mut g, 2, true);
    assert_eq!(g.emerg.len(), 1);
    // the siren is right behind: stay in its lane at a lower speed
    let pz = g.pos + PLAYER_Z;
    for _ in 0..60 * 20 {
        g.speed = 3000.0;
        if let Some(e) = g.emerg.first().copied() {
            if pz < 0.0 || e.z > g.pos + PLAYER_Z - 2000.0 {
                g.x = e.x;
            }
        }
        step(&mut g, 1, true);
        if g.violations > 0 {
            break;
        }
    }
    assert!(g.violations > 0 || g.emerg.is_empty());
}

#[test]
fn accident_scenes_block_the_right_and_leave_the_left() {
    let mut g = game(4, Difficulty::Normal);
    let z0 = g.city.scenes[0];
    g.emerg.clear();
    place(
        &mut g,
        z0 + city::SCENE[1].0,
        300.0,
        city::SCENE[1].1,
        3000.0,
    );
    step(&mut g, 20, true);
    assert!(g.crashes >= 1, "driving into the ambulance");
    let mut g = game(4, Difficulty::Normal);
    place(&mut g, z0 + 900.0, 3000.0, -0.4, 6000.0);
    step(&mut g, 120, true);
    assert_eq!(g.crashes, 0);
}

#[test]
fn new_banners_exist_in_all_languages_and_are_drawable() {
    let keys = [
        "CITY",
        "SPEEDING!",
        "RED LIGHT!",
        "PEDESTRIAN!",
        "DETOUR",
        "JAM AHEAD",
        "FINE",
        "GOOD DRIVER",
        "CLEAN CITY",
        "EMERGENCY",
        "ACCIDENT",
        "MOVE OVER!",
    ];
    for &(l, code, _) in LANGS.iter() {
        for k in keys {
            let s = i18n::item_banner_in(l, k).unwrap_or_else(|| panic!("{code}: no banner {k}"));
            assert!(!s.is_empty());
            for ch in s.chars().filter(|c| !c.is_whitespace()) {
                assert!(
                    font::glyph(ch).is_some(),
                    "{code}: no glyph for {ch:?} in {s:?}"
                );
            }
        }
    }
}

#[test]
fn a_whole_stage_with_a_bot_still_ends_and_replays_the_same() {
    let run = || {
        let mut g = Game::new_mode(6, Mode::Campaign);
        g.practice();
        for _ in 0..60 * 200 {
            let i = g.autopilot();
            g.update(1.0 / 60.0, &i);
            if g.state != State::Playing {
                break;
            }
        }
        (g.state, g.score, g.violations, g.detours, g.pos as i64)
    };
    let (a, b) = (run(), run());
    assert_eq!(a, b, "deterministic");
    assert!(a.0 != State::Playing);
}
