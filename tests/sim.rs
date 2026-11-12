use turbocursa::game::*;
use turbocursa::{render, H, W};

#[test]
fn deterministic() {
    let run = || {
        let mut g = Game::new(7);
        let i = Input {
            accel: true,
            ..Default::default()
        };
        for _ in 0..600 {
            g.update(1.0 / 60.0, &i);
        }
        (g.pos, g.score, g.overtaken)
    };
    assert_eq!(run(), run());
}

#[test]
fn time_runs_out_when_idle() {
    let mut g = Game::new(1);
    for _ in 0..(60 * 45) {
        g.update(1.0 / 60.0, &Input::default());
    }
    assert_eq!(g.state, State::TimeUp);
}

#[test]
fn track_closes_and_renders_everywhere() {
    let g = Game::new(3);
    let n = g.track.segs.len();
    assert!(g.track.segs[n - 1].y1.abs() < 1.0);
    let mut fb = vec![0u32; W * H];
    let mut g = g;
    for step in 0..40 {
        g.pos = g.track.length * step as f32 / 40.0 * 0.99;
        render::render(&g, &mut fb);
    }
}

#[test]
fn offroad_slows_down() {
    let mut g = Game::new(1);
    g.speed = MAX_SPEED;
    g.x = 1.8;
    g.update(
        0.5,
        &Input {
            accel: true,
            ..Default::default()
        },
    );
    assert!(g.speed < MAX_SPEED);
}

#[test]
fn effects_do_not_panic() {
    let mut g = Game::new(5);
    let mut fb = vec![0u32; W * H];
    for i in 0..900 {
        let inp = Input {
            accel: true,
            turbo: i % 120 == 5,
            left: (i / 60) % 4 == 0,
            right: (i / 60) % 4 == 2,
            ..Default::default()
        };
        g.update(1.0 / 60.0, &inp);
        if i % 3 == 0 {
            render::render(&g, &mut fb);
        }
    }
    g.x = 2.1;
    g.speed = MAX_SPEED;
    for _ in 0..30 {
        g.update(
            1.0 / 60.0,
            &Input {
                accel: true,
                ..Default::default()
            },
        );
        render::render(&g, &mut fb);
    }
    assert!(g.particles.len() < 700);
}

#[test]
fn versus_runs_and_renders() {
    use turbocursa::versus::Versus;
    let mut v = Versus::new(11);
    let mut out = vec![0u32; W * H];
    for i in 0..600 {
        let a = Input {
            accel: true,
            right: i % 90 < 5,
            ..Default::default()
        };
        let b = Input {
            accel: i % 3 != 0,
            left: i % 70 < 4,
            ..Default::default()
        };
        v.update(1.0 / 60.0, &[a, b]);
        if i % 50 == 0 {
            v.render(&mut out);
        }
    }
    assert!(v.p[0].pos > v.p[1].pos, "flat-out driver should lead");
    assert!(out.iter().any(|&c| c != 0));
}

#[test]
fn audio_is_finite_bounded_and_reacts() {
    use turbocursa::audio::{AudioParams, Synth};
    let mut s = Synth::new(44100.0, 3);
    let mut quiet = vec![0.0f32; 44100];
    s.render(
        &mut quiet,
        &AudioParams {
            playing: true,
            ..Default::default()
        },
    );
    let mut loud = vec![0.0f32; 44100];
    let fast = AudioParams {
        playing: true,
        speed: 1.0,
        turbo: true,
        crashes: 1,
        cps: 1,
        ..Default::default()
    };
    s.render(&mut loud, &fast);
    let rms = |v: &[f32]| (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt();
    assert!(quiet
        .iter()
        .chain(&loud)
        .all(|x| x.is_finite() && x.abs() <= 1.0));
    assert!(
        rms(&loud) > rms(&quiet),
        "speed/turbo/crash must raise the level"
    );
}

#[test]
fn daily_seed_is_stable_within_a_day() {
    assert_eq!(
        turbocursa::daily_seed(86_400 * 20_000 + 5),
        turbocursa::daily_seed(86_400 * 20_000 + 86_000)
    );
    assert_ne!(
        turbocursa::daily_seed(86_400 * 20_000),
        turbocursa::daily_seed(86_400 * 20_001)
    );
}
