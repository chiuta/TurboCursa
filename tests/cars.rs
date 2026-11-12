use turbocursa::career::{ncountries, COUNTRIES};
use turbocursa::cars::*;
use turbocursa::game::{Game, Input, MAX_SPEED};
use turbocursa::{render, H, W};

#[test]
fn every_country_has_a_flag_and_a_balanced_car() {
    assert!(ncountries() >= 195, "all UN members (193) plus observers");
    let mut seen = std::collections::HashSet::new();
    for (i, c) in COUNTRIES.iter().enumerate() {
        assert!(has_flag(c.code), "no flag colours for {}", c.code);
        let car = CarSpec::for_country(i);
        assert!(
            (0.92..=1.08).contains(&car.top),
            "{} top {}",
            c.code,
            car.top
        );
        assert!((0.88..=1.12).contains(&car.accel));
        assert!((0.92..=1.08).contains(&car.grip));
        assert!((2..=4).contains(&car.turbos));
        // zero-sum budget: each trait's deviation, measured in units of its own range, sums to zero
        // (a turbo charge above/below three is paid for with 3% top speed)
        let tpay = (car.turbos as f32 - 3.0) * 0.03;
        let budget =
            (car.top - 1.0 + tpay) / 0.05 + (car.accel - 1.0) / 0.10 + (car.grip - 1.0) / 0.08;
        assert!(budget.abs() < 0.01, "{} unbalanced: {budget}", c.code);
        seen.insert((
            (car.top * 1e4) as i32,
            (car.accel * 1e4) as i32,
            (car.grip * 1e4) as i32,
        ));
        assert!(car.model().starts_with(c.code));
    }
    assert!(
        seen.len() >= 190,
        "profiles must differ per country: {}",
        seen.len()
    );
}

#[test]
fn liveries_render_and_differ() {
    let mut sums = std::collections::HashSet::new();
    let mut fb = vec![0u32; W * H];
    for i in 0..COUNTRIES.len() {
        let mut g = Game::new(3);
        g.set_car(CarSpec::for_country(i));
        render::render(&g, &mut fb);
        // the car sits at the bottom centre: hash that region
        let mut h = 0u64;
        for y in (H - 60)..(H - 8) {
            for x in (W / 2 - 50)..(W / 2 + 50) {
                h = h.wrapping_mul(31).wrapping_add(fb[y * W + x] as u64);
            }
        }
        sums.insert(h);
    }
    assert!(
        sums.len() >= 150,
        "liveries too similar: {} distinct of {}",
        sums.len(),
        COUNTRIES.len()
    );
}

#[test]
fn handling_profile_changes_the_physics() {
    let run = |car: CarSpec| {
        let mut g = Game::new(5);
        g.practice(); // obstacles would add noise to a pure handling comparison
        g.set_car(car);
        let gas = Input {
            accel: true,
            ..Input::default()
        };
        let mut t_half = None;
        for i in 0..1200 {
            g.update(1.0 / 60.0, &gas);
            if t_half.is_none() && g.speed >= MAX_SPEED * 0.5 {
                t_half = Some(i);
            }
        }
        (g.speed, t_half.unwrap())
    };
    let fast = CarSpec {
        top: 1.07,
        accel: 0.9,
        ..CarSpec::default()
    };
    let quick = CarSpec {
        top: 0.95,
        accel: 1.1,
        ..CarSpec::default()
    };
    let (top_fast, t_fast) = run(fast);
    let (top_quick, t_quick) = run(quick);
    assert!(top_fast > top_quick, "top speed must differ");
    assert!(t_quick < t_fast, "acceleration must differ");
    let mut g = Game::new(1);
    g.set_car(CarSpec {
        turbos: 4,
        ..CarSpec::default()
    });
    assert_eq!(g.turbo, 4);
}

#[test]
fn hostile_specs_cannot_break_the_game() {
    let mut g = Game::new(2);
    g.set_car(CarSpec {
        top: 1e9,
        accel: -5.0,
        grip: f32::NAN,
        turbos: 200,
        ..CarSpec::default()
    });
    let gas = Input {
        accel: true,
        right: true,
        ..Input::default()
    };
    for _ in 0..600 {
        g.update(1.0 / 60.0, &gas);
        assert!(g.speed.is_finite() && g.speed >= 0.0 && g.x.is_finite());
    }
    let mut fb = vec![0u32; W * H];
    render::render(&g, &mut fb);
}
