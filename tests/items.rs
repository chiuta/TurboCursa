use turbocursa::game::*;

/// A stage in neutral conditions, so item effects are not mixed with weather or surface effects.
fn game(mode: Mode) -> Game {
    let mut g = Game::new_mode(9, mode);
    g.set_weather(Some(turbocursa::env::Weather::Clear));
    g.set_surface(Some(turbocursa::env::Surface::Asphalt));
    g
}

/// Puts one item directly in front of the player and drives over it.
fn drive_over(g: &mut Game, kind: u8) {
    g.items.clear();
    g.cars.clear();
    g.speed = MAX_SPEED * 0.6;
    g.x = 0.0;
    let z = g.pos + PLAYER_Z + 900.0;
    g.items.push(Item {
        z,
        x: 0.0,
        kind,
        taken: false,
    });
    let inp = Input {
        accel: true,
        ..Input::default()
    };
    for _ in 0..60 {
        g.update(0.016, &inp);
    }
}

#[test]
fn items_are_sorted_deterministic_and_absent_in_zen() {
    let a = game(Mode::Campaign);
    let b = game(Mode::Campaign);
    assert!(!a.items.is_empty());
    assert_eq!(a.items.len(), b.items.len());
    assert!(a.items.windows(2).all(|w| w[0].z <= w[1].z));
    assert!(a
        .items
        .iter()
        .zip(&b.items)
        .all(|(x, y)| x.z == y.z && x.kind == y.kind && x.x == y.x));
    assert!(game(Mode::Zen).items.is_empty());
    assert!(a
        .items
        .iter()
        .all(|i| i.kind < ITEM_KINDS && i.x.abs() <= 0.9));
    assert!(a.items.iter().any(|i| is_bonus(i.kind)) && a.items.iter().any(|i| !is_bonus(i.kind)));
}

#[test]
fn harder_levels_have_more_obstacles_and_fewer_bonuses() {
    let count = |lv, bonus: bool| {
        let mut g = game(Mode::Campaign);
        g.set_level(lv);
        g.items.iter().filter(|i| is_bonus(i.kind) == bonus).count()
    };
    assert!(count(Difficulty::Insane, false) > count(Difficulty::Easy, false));
    assert!(count(Difficulty::Insane, true) < count(Difficulty::Easy, true));
}

#[test]
fn every_item_kind_is_generated_somewhere() {
    let mut seen = std::collections::HashSet::new();
    for seed in 1..8 {
        for sf in 0..turbocursa::env::SURFACE_COUNT {
            let mut g = Game::new_mode(seed, Mode::Rush);
            g.set_surface(Some(turbocursa::env::Surface::from_index(sf)));
            seen.extend(g.items.iter().map(|i| i.kind));
        }
    }
    for k in 0..ITEM_KINDS {
        assert!(seen.contains(&k), "kind {k} never appears");
    }
}

#[test]
fn bonuses_do_what_they_say() {
    let mut g = game(Mode::Campaign);
    let t0 = g.time;
    drive_over(&mut g, I_TIME);
    assert!(g.time > t0 + 3.0 - 60.0 * 0.016 && g.bonuses == 1);

    let mut g = game(Mode::Campaign);
    let tb = g.turbo;
    drive_over(&mut g, I_TURBO);
    assert_eq!(g.turbo, tb + 1);

    let mut g = game(Mode::Campaign);
    drive_over(&mut g, I_SHIELD);
    assert!(g.shield_t > 4.0);
    let mut g = game(Mode::Campaign);
    drive_over(&mut g, I_BOOST);
    assert!(g.boost_t > 0.0 && g.speed > MAX_SPEED * 0.6);
    let mut g = game(Mode::Campaign);
    let s0 = g.score;
    drive_over(&mut g, I_COIN);
    assert!(g.score >= s0 + 500);
    let mut g = game(Mode::Campaign);
    drive_over(&mut g, I_FREEZE);
    assert!(g.freeze_t > 0.0);
    let mut g = game(Mode::Campaign);
    drive_over(&mut g, I_DOUBLE);
    assert!(g.x2_t > 5.0);
}

#[test]
fn time_bonus_is_score_in_time_trial() {
    let mut g = game(Mode::TimeTrial);
    drive_over(&mut g, I_TIME);
    assert!(g.time > 0.0); // clock still counts up
    assert!(g.score >= 300);
}

#[test]
fn obstacles_hurt_and_the_shield_protects() {
    for k in [I_BARRIER, I_POTHOLE, I_CONE] {
        let mut g = game(Mode::Campaign);
        g.set_level(Difficulty::Normal);
        drive_over(&mut g, k);
        assert!(g.hits == 1, "kind {k} not registered");
        // with a shield the same obstacle is harmless
        let mut s = game(Mode::Campaign);
        s.shield_t = 10.0;
        drive_over(&mut s, k);
        assert_eq!(s.hits, 0);
        assert!(s.speed > g.speed);
    }
    let mut g = game(Mode::Campaign);
    drive_over(&mut g, I_OIL);
    assert!(g.hits == 1);
    let mut g = game(Mode::Campaign);
    drive_over(&mut g, I_MUD);
    assert!(g.hits == 1);
    let mut g = game(Mode::Campaign);
    drive_over(&mut g, I_PUDDLE);
    assert!(g.hits == 1);
}

#[test]
fn missing_the_item_costs_nothing() {
    let mut g = game(Mode::Campaign);
    g.items.clear();
    g.cars.clear();
    g.speed = MAX_SPEED * 0.6;
    let z = g.pos + PLAYER_Z + 900.0;
    g.items.push(Item {
        z,
        x: 0.85,
        kind: I_BARRIER,
        taken: false,
    });
    g.x = -0.6;
    let inp = Input {
        accel: true,
        ..Input::default()
    };
    for _ in 0..60 {
        g.update(0.016, &inp);
    }
    assert_eq!(g.hits, 0);
}

#[test]
fn freeze_slows_traffic_and_boost_raises_top_speed() {
    let mut g = game(Mode::Rush);
    g.freeze_t = 3.0;
    let before: Vec<f32> = g.cars.iter().map(|c| c.base).collect();
    g.update(0.016, &Input::default());
    for (c, b) in g.cars.iter().zip(before) {
        assert!(c.speed <= b * 0.26 + 1e-3);
    }
    let top = |boost: bool| {
        let mut g = game(Mode::TimeTrial);
        g.cars.clear();
        g.items.clear();
        if boost {
            g.boost_t = 99.0;
        }
        let inp = Input {
            accel: true,
            ..Input::default()
        };
        let mut best = 0.0f32;
        for _ in 0..1500 {
            g.update(0.016, &inp);
            g.boost_t = if boost { 99.0 } else { 0.0 };
            best = best.max(g.speed);
        }
        best
    };
    assert!(top(true) > top(false));
}

#[test]
fn hostile_play_with_items_never_breaks_state() {
    for mode in [
        Mode::Campaign,
        Mode::Rush,
        Mode::Collector,
        Mode::Slalom,
        Mode::Mirror,
    ] {
        let mut g = Game::new_mode(77, mode);
        g.set_level(Difficulty::Insane);
        let mut r = 12345u64;
        for i in 0..6000 {
            r = r
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let inp = Input {
                accel: r >> 60 != 0,
                left: r >> 33 & 1 == 1,
                right: r >> 34 & 1 == 1,
                brake: r >> 35 & 7 == 0,
                turbo: i % 200 == 0,
            };
            g.update(0.016, &inp);
            assert!(g.x.abs() <= 2.21 && g.speed.is_finite() && g.time.is_finite());
            assert!(g.shield_t >= 0.0 && g.boost_t >= 0.0 && g.slip_t >= 0.0 && g.turbo <= 9);
        }
    }
}

#[test]
fn pick_screen_navigation() {
    use turbocursa::cars::CLASS_TOTAL;
    use turbocursa::pick::PickApp;
    let none = Input::default();
    let info = turbocursa::pick::ProfileInfo {
        specials: 0b1111,
        ..Default::default()
    };
    let mut p = PickApp::new(0, 1).with_profile(info);
    p.update(0.016, &none, false); // release: a key held at start is ignored
    let press = |p: &mut PickApp, i: Input| {
        p.update(0.016, &i, false);
        p.update(0.016, &none, false)
    };
    press(
        &mut p,
        Input {
            right: true,
            ..none
        },
    );
    assert_eq!(p.class, 1);
    press(&mut p, Input { left: true, ..none });
    press(&mut p, Input { left: true, ..none });
    assert_eq!(p.class, CLASS_TOTAL - 1, "wraps around");
    press(
        &mut p,
        Input {
            brake: true,
            ..none
        },
    );
    assert_eq!(p.level, 2);
    press(
        &mut p,
        Input {
            accel: true,
            ..none
        },
    );
    press(
        &mut p,
        Input {
            accel: true,
            ..none
        },
    );
    assert_eq!(p.level, 0);
    // first Enter opens the service page, the second one starts
    assert!(p.update(0.016, &none, true).is_none());
    assert_eq!(p.page, 1);
    press(
        &mut p,
        Input {
            right: true,
            ..none
        },
    );
    press(
        &mut p,
        Input {
            brake: true,
            ..none
        },
    );
    assert_eq!((p.tyre, p.setup), (1, 1));
    p.back();
    assert_eq!(p.page, 0);
    assert!(p.update(0.016, &none, true).is_none());
    let ch = p.update(0.016, &none, true).unwrap();
    assert_eq!((ch.class, ch.level), (CLASS_TOTAL - 1, 0));
    assert_eq!((ch.tyre, ch.setup), (1, 1));
    // rendering every class and both pages never panics
    let mut fb = vec![0u32; turbocursa::W * turbocursa::H];
    for c in 0..CLASS_TOTAL {
        let mut q = PickApp::new(c, c % 5).with_forecast(c as u64 * 7);
        q.render(&mut fb);
        q.update(0.016, &none, true);
        q.render(&mut fb);
    }
}
