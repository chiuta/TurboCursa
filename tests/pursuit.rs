use turbocursa::cars::*;
use turbocursa::game::*;
use turbocursa::pursuit::*;

fn police(seed: u64, level: usize) -> Game {
    let mut g = Game::new_mode(seed, Mode::Campaign);
    g.set_car(CarSpec::default().with_class(VehicleClass::Police));
    g.set_level(Difficulty::from_index(level));
    g
}

#[test]
fn only_the_police_car_gets_a_fugitive() {
    let mut g = Game::new_mode(3, Mode::Campaign);
    let go = Input {
        accel: true,
        ..Default::default()
    };
    for _ in 0..1500 {
        g.update(0.02, &go);
    }
    assert!(g.pursuit.is_none() && g.arrests == 0);
    let mut p = police(3, 1);
    for _ in 0..400 {
        p.update(0.02, &go);
    }
    assert!(p.pursuit.is_some(), "the police car must get a target");
}

#[test]
fn a_chasing_driver_ends_the_pursuit_and_nothing_breaks() {
    let mut ended = 0;
    for seed in 1..=6u64 {
        let mut g = police(seed, 1);
        let mut seen = false;
        for _ in 0..40_000 {
            let inp = match &g.pursuit {
                Some(p) => {
                    seen = true;
                    Input {
                        accel: true,
                        left: p.x < g.x - 0.05,
                        right: p.x > g.x + 0.05,
                        ..Default::default()
                    }
                }
                None => Input {
                    accel: true,
                    ..Default::default()
                },
            };
            g.update(0.02, &inp);
            assert!(g.score < 100_000_000 && g.time.is_finite() && g.x.is_finite());
            if g.state != State::Playing {
                break;
            }
        }
        if seen && g.pursuit.is_none() {
            ended += 1;
        }
    }
    assert!(ended >= 3, "pursuits must reach an outcome, got {ended}");
}

#[test]
fn pursuit_logic_is_bounded() {
    let mut p = Pursuit::new(9, 0.0, 7000.0, 0);
    // never reaches him: the catch phase times out
    for _ in 0..4000 {
        p.update(0.02, 0.0, 0.0, 0.0, 0, 1e9);
    }
    assert_eq!(p.phase, Phase::Escaped);
    // three crashes
    let mut q = Pursuit::new(9, 0.0, 7000.0, 2);
    q.update(0.02, 0.0, 0.0, 0.0, 5, 1e9);
    assert_eq!(q.phase, Phase::Escaped);
    // hostile input never panics or goes non-finite
    let mut r = Pursuit::new(1, f32::NAN, f32::INFINITY, 0);
    for _ in 0..100 {
        r.update(f32::NAN, f32::NAN, f32::NAN, f32::NAN, 0, f32::NAN);
        r.update(0.02, 0.0, 0.0, 0.0, 0, 1e9);
    }
    assert!(r.hp.is_finite() && r.left.is_finite());
}

#[test]
#[ignore]
fn probe_arrests() {
    let mut a = 0;
    for seed in 1..=10u64 {
        let mut g = police(seed, 1);
        for _ in 0..40_000 {
            let inp = match &g.pursuit {
                Some(p) => Input {
                    accel: true,
                    left: p.x < g.x - 0.05,
                    right: p.x > g.x + 0.05,
                    ..Default::default()
                },
                None => Input {
                    accel: true,
                    ..Default::default()
                },
            };
            g.update(0.02, &inp);
            if g.state != State::Playing {
                break;
            }
        }
        a += g.arrests;
    }
    println!("arrests {a}/10");
}
