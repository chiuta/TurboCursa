//! Game-mode behaviour: every mode runs, obeys its own rules and survives random play.
use turbocursa::game::*;
use turbocursa::versus::Versus;
use turbocursa::{render, H, W};

fn drive(g: &mut Game, frames: usize, f: impl Fn(&Game) -> Input) {
    for _ in 0..frames {
        let inp = f(g);
        g.update(1.0 / 60.0, &inp);
    }
}

fn steer_to(g: &Game, tx: f32) -> Input {
    Input {
        accel: true,
        left: g.x > tx + 0.05,
        right: g.x < tx - 0.05,
        ..Default::default()
    }
}

#[test]
fn every_mode_survives_random_play() {
    let mut s = 12345u64;
    for m in 0..MODE_COUNT as u32 {
        let mode = Mode::from_index(m);
        let mut g = Game::new_mode(3, mode);
        let mut fb = vec![0u32; W * H];
        for i in 0..2400 {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (s >> 33) as u32;
            g.update(
                1.0 / 60.0,
                &Input {
                    left: r & 1 != 0,
                    right: r & 2 != 0,
                    accel: r & 12 != 0,
                    brake: r & 48 == 48,
                    turbo: r & 64 != 0,
                },
            );
            assert!(
                g.time.is_finite() && g.pos.is_finite() && g.x.is_finite(),
                "{mode:?} step {i}"
            );
            if i % 400 == 0 {
                render::render(&g, &mut fb);
            }
        }
    }
}

#[test]
fn mode_parse_accepts_names_and_indices() {
    assert_eq!(Mode::parse("rush"), Some(Mode::Rush));
    assert_eq!(Mode::parse("Time-Trial"), Some(Mode::TimeTrial));
    assert_eq!(Mode::parse("8"), Some(Mode::Zen));
    assert_eq!(Mode::parse("9"), None);
    assert_eq!(Mode::parse("nonsense"), None);
}

#[test]
fn time_trial_has_no_traffic_and_counts_up() {
    let mut g = Game::new_mode(4, Mode::TimeTrial);
    assert!(g.cars.is_empty());
    drive(&mut g, 600, |g| steer_to(g, 0.0));
    assert!(g.time > 9.0 && g.time < 11.0, "clock {}", g.time);
    assert_eq!(g.state, State::Playing);
}

#[test]
fn rush_has_triple_traffic_and_overtakes_pay_time() {
    let g = Game::new_mode(5, Mode::Rush);
    assert!(g.cars.len() >= 90);
    let mut g = Game::new_mode(5, Mode::Rush);
    let t0 = g.time;
    let mut gained = 0.0f32;
    // the bot keeps to the city limit and stops at red lights: give it time
    for _ in 0..6000 {
        let before = (g.overtaken, g.time);
        let inp = g.autopilot();
        g.update(1.0 / 60.0, &inp);
        if g.overtaken > before.0 && g.time > before.1 {
            gained += 1.0;
        }
        if g.state != State::Playing {
            break;
        }
    }
    assert!(g.overtaken > 3, "overtook {}", g.overtaken);
    assert!(gained > 0.0 && g.time > t0 - 50.0);
}

#[test]
fn collector_orbs_pay_when_followed() {
    let mut g = Game::new_mode(6, Mode::Collector);
    assert!(g.orbs.len() >= 100);
    drive(&mut g, 3600, |g| {
        let pz = g.pos + PLAYER_Z;
        let tx = g
            .orbs
            .iter()
            .find(|o| !o.taken && o.z > pz + 400.0)
            .map(|o| o.x)
            .unwrap_or(0.0);
        steer_to(g, tx)
    });
    assert!(g.collected > 30, "collected only {}", g.collected);
}

#[test]
fn slalom_rings_build_combos() {
    let mut g = Game::new_mode(7, Mode::Slalom);
    assert!(g.orbs.iter().all(|o| o.kind == 1) && g.orbs.len() > 20);
    let best = std::cell::Cell::new(0u32);
    drive(&mut g, 3600, |g| {
        best.set(best.get().max(g.combo));
        let pz = g.pos + PLAYER_Z;
        let tx = g
            .orbs
            .iter()
            .find(|o| !o.taken && o.z > pz + 300.0)
            .map(|o| o.x)
            .unwrap_or(0.0);
        steer_to(g, tx)
    });
    let best = best.get();
    assert!(
        g.collected > 10 && best > 8,
        "collected {} best combo {}",
        g.collected,
        best
    );
}

#[test]
fn mirror_flips_the_steering_for_a_while() {
    let mut g = Game::new_mode(8, Mode::Mirror);
    let mut saw_inverted = false;
    let mut flipped_ok = false;
    for _ in 0..1500 {
        let x0 = g.x;
        g.update(
            1.0 / 60.0,
            &Input {
                accel: true,
                left: true,
                ..Default::default()
            },
        );
        if g.inverted {
            saw_inverted = true;
            // holding LEFT while inverted must not move the car left (curve drift is far smaller than a steering step)
            if g.x > x0 {
                flipped_ok = true;
            }
        }
        g.x = g.x.clamp(-0.5, 0.5);
    }
    assert!(saw_inverted && flipped_ok);
}

#[test]
fn blackout_is_much_darker() {
    let mut fb1 = vec![0u32; W * H];
    let mut fb2 = vec![0u32; W * H];
    let mut a = Game::new_mode(9, Mode::Campaign);
    let mut b = Game::new_mode(9, Mode::Blackout);
    for _ in 0..30 {
        a.update(
            1.0 / 60.0,
            &Input {
                accel: true,
                ..Default::default()
            },
        );
        b.update(
            1.0 / 60.0,
            &Input {
                accel: true,
                ..Default::default()
            },
        );
    }
    render::render(&a, &mut fb1);
    render::render(&b, &mut fb2);
    let lum = |f: &[u32]| {
        f.iter()
            .step_by(7)
            .map(|&c| ((c >> 16) & 255) + ((c >> 8) & 255) + (c & 255))
            .sum::<u32>() as f64
    };
    assert!(
        lum(&fb2) < lum(&fb1) * 0.55,
        "blackout {} vs {}",
        lum(&fb2),
        lum(&fb1)
    );
}

#[test]
fn endless_and_zen_generate_new_laps_and_never_end() {
    for mode in [Mode::Endless, Mode::Zen] {
        let mut g = Game::new_mode(10, mode);
        g.practice();
        let first = g
            .track
            .segs
            .iter()
            .map(|s| s.y1.to_bits() as u64)
            .sum::<u64>();
        // the bot takes the detours and keeps to the city rules; a naive driver would ram the jams
        drive(&mut g, 60 * 300, |g| g.autopilot());
        assert!(g.lap >= 2, "{mode:?}: lap {}", g.lap);
        assert_eq!(g.state, State::Playing, "{mode:?} must not end");
        let now = g
            .track
            .segs
            .iter()
            .map(|s| s.y1.to_bits() as u64)
            .sum::<u64>();
        assert_ne!(first, now, "{mode:?}: the new lap must be a new stage");
        assert!(g.pos >= 0.0 && g.pos < g.track.length);
        if mode == Mode::Zen {
            assert!(g.cars.is_empty() && g.time >= 98.0 && g.crashes == 0);
        }
    }
}

#[test]
fn versus_cpu_rival_actually_drives() {
    let mut v = Versus::new_cpu(11);
    for _ in 0..60 * 40 {
        v.update(
            1.0 / 60.0,
            &[
                Input {
                    accel: true,
                    ..Default::default()
                },
                Input::default(),
            ],
        );
    }
    assert!(v.p[1].pos > 100_000.0, "cpu only reached {}", v.p[1].pos);
    let mut out = vec![0u32; W * H];
    v.render(&mut out);
}
