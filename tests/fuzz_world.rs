//! Adversarial world fuzz: every vehicle class x mode x seed under random inputs, random teleports,
//! hostile state values and mid-game setter calls. Invariants must hold and nothing may panic.
use turbocursa::cars::*;
use turbocursa::game::*;
use turbocursa::{render, H, W};

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
    fn f(&mut self) -> f32 {
        self.next() as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
    fn input(&mut self) -> Input {
        let r = self.next();
        Input {
            left: r & 1 != 0,
            right: r & 2 != 0,
            accel: r & 12 != 0,
            brake: r & 16 != 0 && r & 32 != 0,
            turbo: r & 64 != 0,
        }
    }
}

fn check(g: &Game, tag: &str) {
    let fin = |v: f32| v.is_finite();
    assert!(
        fin(g.pos) && fin(g.x) && fin(g.speed) && fin(g.time),
        "{tag}: non-finite core"
    );
    assert!(
        fin(g.steer) && fin(g.elapsed) && fin(g.fuel),
        "{tag}: non-finite aux"
    );
    assert!(g.x.abs() <= 2.21, "{tag}: x {}", g.x);
    assert!(
        g.speed >= -1.0 && g.speed <= MAX_SPEED * 2.1,
        "{tag}: speed {}",
        g.speed
    );
    assert!(g.pos >= -1.0, "{tag}: pos {}", g.pos);
    for c in &g.cars {
        assert!(fin(c.z) && fin(c.x) && fin(c.speed), "{tag}: car");
    }
    for c in &g.cops {
        assert!(fin(c.z) && fin(c.x), "{tag}: cop");
    }
    assert!(g.civic_mult() >= 1 && g.civic_mult() <= 3, "{tag}: civic");
}

/// Quick set (every class and mode, one seed) for every run; the full set of four seeds is the
/// ignored nightly test: `cargo test --release --test fuzz_world -- --ignored`.
#[test]
fn every_class_and_mode_survives_random_play() {
    random_play(&[7]);
}

#[test]
#[ignore = "nightly: four seeds per class and mode"]
fn every_class_mode_and_seed_survives_random_play_full() {
    random_play(&[1, 7, 23, 99]);
}

fn random_play(seeds: &[u64]) {
    let mut r = Lcg(0xC0FFEE);
    let mut fb = vec![0u32; W * H];
    for class in 0..CLASS_TOTAL {
        for mode in 0..MODE_COUNT as u32 {
            for &seed in seeds {
                let tag = format!("class {class} mode {mode} seed {seed}");
                let mut g = Game::new_mode(seed, Mode::from_index(mode));
                g.set_car(CarSpec::default().with_class(VehicleClass::from_index(class)));
                g.set_rank((r.next() % 1000) as u16);
                g.state = State::Playing;
                for f in 0..900 {
                    let mut inp = r.input();
                    // teleports into every kind of place: cities, jams, scenes, bays, lots, hill
                    if f % 150 == 0 {
                        let len = g.track.length;
                        g.pos = (r.next() as f32 / u32::MAX as f32 * len).min(len - 1.0);
                        g.x = r.f() * 2.0;
                        g.speed = (r.f().abs()) * MAX_SPEED;
                        if r.next().is_multiple_of(4) {
                            g.speed = 0.0;
                        }
                    }
                    // hostile dt
                    let dt = match r.next() % 12 {
                        0 => 0.0,
                        1 => 0.5,
                        2 => 1e-5,
                        _ => 1.0 / 60.0,
                    };
                    if r.next().is_multiple_of(90) {
                        g.set_weather(None);
                        g.set_env((r.next() % 12) as usize, (r.next() % 7) as usize);
                        g.set_level(Difficulty::from_index((r.next() % 5) as usize));
                    }
                    if f % 200 == 199 {
                        inp = g.autopilot();
                    }
                    g.update(dt, &inp);
                    check(&g, &tag);
                    if f % 45 == 0 {
                        render::render(&g, &mut fb);
                    }
                    if g.state != State::Playing {
                        g.state = State::Playing;
                    }
                }
            }
        }
    }
}

#[test]
fn hostile_car_specs_never_break_the_game() {
    for class in 0..CLASS_TOTAL {
        for bad in [f32::NAN, f32::INFINITY, -1e9, 1e9, 0.0] {
            let mut c = CarSpec::default().with_class(VehicleClass::from_index(class));
            c.top = bad;
            c.accel = bad;
            c.grip = bad;
            c.width = bad;
            c.crash_keep = bad;
            c.prop_keep = bad;
            c.time_mult = bad;
            c.traction = bad;
            let mut g = Game::new_mode(5, Mode::Campaign);
            g.set_car(c);
            g.state = State::Playing;
            for _ in 0..200 {
                g.update(
                    1.0 / 60.0,
                    &Input {
                        accel: true,
                        ..Default::default()
                    },
                );
            }
            check(&g, &format!("class {class} bad {bad}"));
            let mut fb = vec![0u32; W * H];
            render::render(&g, &mut fb);
        }
    }
}

#[test]
fn hostile_values_injected_into_runtime_state_never_panic() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let bads = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1e30, -1e30];
    for (i, bad) in bads.into_iter().enumerate() {
        for field in 0..4 {
            let r = catch_unwind(AssertUnwindSafe(|| {
                let mut fb = vec![0u32; W * H];
                let mut g = Game::new_mode(11 + i as u64, Mode::Campaign);
                g.state = State::Playing;
                g.set_car(CarSpec::default().with_class(VehicleClass::Helicopter));
                match field {
                    0 => g.pos = bad,
                    1 => g.x = bad,
                    2 => g.speed = bad,
                    _ => g.air = bad,
                }
                for _ in 0..5 {
                    g.update(
                        1.0 / 60.0,
                        &Input {
                            accel: true,
                            ..Default::default()
                        },
                    );
                    render::render(&g, &mut fb);
                }
            }));
            assert!(r.is_ok(), "panic with {bad} in field {field}");
        }
    }
}
