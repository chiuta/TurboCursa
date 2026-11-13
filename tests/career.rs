use turbocursa::career::*;
use turbocursa::career_ui::CareerApp;
use turbocursa::game::Input;
use turbocursa::{font, i18n, i18n_career};

fn play_cup(kind: CupKind, div: u8, seed: u64, factor: f32) -> (bool, bool) {
    let mut c = Cup::new(kind, seed, div);
    let mut guard = 0;
    while !c.is_over() {
        c.advance(Some(100.0 * factor), 100.0).unwrap();
        guard += 1;
        assert!(guard < 40, "cup never ends");
    }
    (c.phase == Phase::Won, c.final_reached)
}

#[test]
fn balance_curve() {
    let win = |k: CupKind, div: u8, f: f32| -> f32 {
        (0..300u64).filter(|&s| play_cup(k, div, s, f).0).count() as f32 / 300.0
    };
    let easy = win(CupKind::National(0), 0, 1.0);
    let slow = win(CupKind::National(0), 0, 1.5);
    let hard = win(CupKind::World, 4, 1.0);
    let top = win(CupKind::World, 4, 0.9);
    let mid = win(CupKind::World, 4, 0.95);
    eprintln!("bronze national @1.0 {easy:.2}, @1.5 {slow:.2}; diamond world @1.0 {hard:.2}, @0.95 {mid:.2}, @0.9 {top:.2}");
    assert!(
        mid > hard && mid < top + 1e-6,
        "win rate must grow smoothly with pace"
    );
    assert!(
        easy > 0.85,
        "a par-pace driver should rule the bronze national cup"
    );
    assert!(slow < 0.05, "a 50% slower driver must not win");
    assert!(
        hard < 0.6,
        "the diamond world cup must be hard even at par pace"
    );
    assert!(top > hard, "being faster must help");
}

#[test]
fn repechage_gives_second_chances() {
    // a driver who is a bit slow gets eliminated, then can come back through the repechage
    let mut revived = 0;
    for s in 0..400u64 {
        let mut c = Cup::new(CupKind::Continental(0), s, 1);
        let mut guard = 0;
        while !c.is_over() && guard < 30 {
            let r = c.advance(Some(112.0), 100.0).unwrap();
            if r.outcome == Outcome::Revived {
                revived += 1;
            }
            guard += 1;
        }
        assert!(c.lives <= 2);
    }
    assert!(revived > 10, "repechage should happen sometimes: {revived}");
}

#[test]
fn qualification_chain_and_locks() {
    let mut c = Career::new(0, 9);
    assert_eq!(
        c.start_cup(CupKind::Continental(0)),
        Err(Locked::NotQualified)
    );
    assert_eq!(c.start_cup(CupKind::World), Err(Locked::NotQualified));
    // a fast driver wins the home cup and unlocks the continental one
    c.start_cup(CupKind::National(0)).unwrap();
    assert_eq!(
        c.start_cup(CupKind::National(1)),
        Err(Locked::CupInProgress)
    );
    while c.cup.is_some() {
        c.run_cup(Some(80.0), 100.0).unwrap();
    }
    assert!(c.q[0] && c.trophies.national[0] == 1);
    // a guest cup in another country never unlocks anything
    let q = c.q;
    c.start_cup(CupKind::National(20)).unwrap();
    while c.cup.is_some() {
        c.run_cup(Some(80.0), 100.0).unwrap();
    }
    assert_eq!(c.q, q);
    assert_eq!(c.trophies.national[20], 1);
    // wrong continent is locked
    assert_eq!(
        c.start_cup(CupKind::Continental(3)),
        Err(Locked::NotQualified)
    );
    c.start_cup(CupKind::Continental(0)).unwrap();
    while c.cup.is_some() {
        c.run_cup(Some(80.0), 100.0).unwrap();
    }
    assert!(c.q[1] && c.trophies.cont[0] == 1);
    c.start_cup(CupKind::Inter).unwrap();
    while c.cup.is_some() {
        c.run_cup(Some(80.0), 100.0).unwrap();
    }
    assert!(c.q[2] && c.trophies.inter == 1);
    c.start_cup(CupKind::World).unwrap();
    while c.cup.is_some() {
        c.run_cup(Some(80.0), 100.0).unwrap();
    }
    assert_eq!(c.trophies.world, 1);
    assert!(c.rating > 1000.0);
}

#[test]
fn league_season_promotes_and_relegates() {
    let mut c = Career::new(3, 5);
    for _ in 0..LEAGUE_DAYS {
        c.run_league(Some(80.0), 100.0);
    }
    assert_eq!((c.div, c.season, c.day), (1, 2, 0));
    // a driver that never finishes is relegated, but never below bronze or above diamond
    for _ in 0..LEAGUE_DAYS {
        c.run_league(None, 100.0);
    }
    assert_eq!(c.div, 0);
    for _ in 0..(LEAGUE_DAYS as usize * 12) {
        c.run_league(Some(1.0e-3 + 60.0), 100.0);
    }
    assert!((c.div as usize) < DIVS);
    assert!(
        c.trophies.league >= 1,
        "a dominant driver wins the diamond league"
    );
}

#[test]
fn hostile_inputs_never_panic() {
    let mut c = Career::new(200, 1); // country index clamps
    assert!((c.country as usize) < ncountries());
    c.start_cup(CupKind::National(0)).unwrap();
    for t in [f32::NAN, f32::INFINITY, -5.0, 0.0, 1e30] {
        for p in [f32::NAN, -1.0, 0.0, 1e30, f32::INFINITY] {
            let mut d = c.clone();
            for _ in 0..8 {
                d.run_cup(Some(t), p);
                d.run_league(Some(t), p);
            }
            assert!(d.rating.is_finite());
        }
    }
    // saves: garbage, truncation and bit flips never panic and never yield out-of-range state
    let good = {
        let mut c = Career::new(7, 123);
        c.start_cup(CupKind::World).ok();
        c.q = [true; 3];
        c.start_cup(CupKind::World).unwrap();
        c.run_cup(Some(300.0), 100.0);
        c.to_save()
    };
    assert!(Career::from_save(&good).is_some());
    let mut x = 0x1234_5678_9ABCu64;
    for i in 0..4000 {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut b = good.clone().into_bytes();
        match i % 4 {
            0 => b.truncate((x >> 33) as usize % b.len()),
            1 => {
                let k = (x >> 33) as usize % b.len();
                b[k] = (x >> 20) as u8;
            }
            2 => b.extend((0..(x >> 60)).map(|j| (x >> j) as u8)),
            _ => b = (0..(x >> 54)).map(|j| (x >> (j % 56)) as u8).collect(),
        }
        if let Ok(s) = String::from_utf8(b) {
            if let Some(c) = Career::from_save(&s) {
                assert!(
                    (c.country as usize) < ncountries()
                        && (c.div as usize) < DIVS
                        && c.day < LEAGUE_DAYS
                );
                if let Some(cup) = &c.cup {
                    assert!(cup.round < cup.rounds());
                }
                let mut c = c;
                let _ = c.run_cup(Some(100.0), 100.0);
                let _ = c.to_save();
            }
        }
    }
    for s in [
        "",
        "TC1",
        "TC1;",
        "TC1;c=x",
        "TC1;cup=1",
        "TC1;cup=0,0,1,0,9,9,9,9|1|2",
        "XX1;c=1",
        "TC1;lp=1,2,3,4,5,6,7,8,9,10,11,12,13",
    ] {
        let _ = Career::from_save(s);
    }
}

#[test]
fn rankings_are_consistent() {
    let c = Career::new(0, 77);
    let d = c.drivers_ranking();
    assert_eq!(d.len(), 301);
    assert!(d.windows(2).all(|w| w[0].2 >= w[1].2));
    assert_eq!(d.iter().filter(|x| x.3).count(), 1);
    let n = c.nations_ranking();
    assert_eq!(n.len(), ncountries());
    assert!(n.windows(2).all(|w| w[0].1 >= w[1].1));
    assert_eq!(c.league_table().len(), LEAGUE_SIZE);
}

#[test]
fn every_country_is_unique_and_listed() {
    let mut codes: Vec<&str> = COUNTRIES.iter().map(|c| c.code).collect();
    codes.sort();
    codes.dedup();
    assert_eq!(codes.len(), COUNTRIES.len());
    for k in 0..NCONT as u8 {
        assert!(
            countries_of(k).len() >= 5,
            "continent {k} too small for a cup field"
        );
    }
    assert!(COUNTRIES.len() >= 100);
}

#[test]
fn career_text_is_drawable_in_all_languages() {
    for s in i18n_career::all().chain(COUNTRIES.iter().map(|c| c.name)) {
        for ch in s.chars().filter(|c| *c != '{' && *c != '}') {
            assert!(
                font::glyph(ch).is_some() || ch == ' ',
                "no glyph for {ch:?} in {s:?}"
            );
        }
    }
}

#[test]
fn ui_walkthrough_renders_every_screen() {
    i18n::set_lang(i18n::Lang::Ro);
    let mut app = CareerApp::new(5, None);
    let mut fb = vec![0u32; 640 * 400];
    let none = Input::default();
    let tap = |app: &mut CareerApp, i: Input, e: bool, b: bool, fb: &mut Vec<u32>| {
        app.update(0.016, &i, e, b);
        app.render(fb);
        app.update(0.016, &none, false, false);
        app.render(fb);
    };
    let up = Input {
        accel: true,
        ..Input::default()
    };
    let down = Input {
        brake: true,
        ..Input::default()
    };
    let right = Input {
        right: true,
        ..Input::default()
    };
    // pick a country (move around, switch continent), confirm
    for _ in 0..3 {
        tap(&mut app, down, false, false, &mut fb);
    }
    tap(&mut app, right, false, false, &mut fb);
    tap(&mut app, none, true, false, &mut fb);
    assert!(app.take_save().is_some());
    // hub: rankings (all tabs), trophies, abroad picker, locked cups -> toast
    for item in [6usize, 7, 5] {
        for _ in 0..item {
            tap(&mut app, down, false, false, &mut fb);
        }
        tap(&mut app, none, true, false, &mut fb);
        for _ in 0..4 {
            tap(&mut app, right, false, false, &mut fb);
        }
        tap(&mut app, none, false, true, &mut fb);
        for _ in 0..item {
            tap(&mut app, up, false, false, &mut fb);
        }
    }
    tap(&mut app, down, false, false, &mut fb);
    tap(&mut app, down, false, false, &mut fb);
    tap(&mut app, none, true, false, &mut fb); // continental: locked
                                               // a league race: hold the throttle until the clock runs out (DNF), then continue
    for _ in 0..2 {
        tap(&mut app, up, false, false, &mut fb);
    }
    tap(&mut app, none, true, false, &mut fb);
    assert!(app.in_race());
    let gas = Input {
        accel: true,
        ..Input::default()
    };
    for _ in 0..30_000 {
        app.update(0.016, &gas, false, false);
    }
    app.render(&mut fb);
    for _ in 0..90 {
        app.update(0.016, &none, false, false);
    }
    app.update(0.016, &none, true, false);
    app.render(&mut fb);
    assert!(!app.in_race());
    assert_eq!(app.career.stats.races, 1);
    assert!(app.take_save().unwrap().starts_with("TC1;"));
    // everything on screen stays in range
    assert!(fb.iter().all(|&p| p <= 0xFFFFFF));
}
