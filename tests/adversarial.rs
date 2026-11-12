//! Adversarial tests: hostile inputs, odd sizes, many seeds, invariants.
use std::panic::{catch_unwind, AssertUnwindSafe};
use turbocursa::audio::{AudioParams, Synth};
use turbocursa::fractal::Assets;
use turbocursa::game::*;
use turbocursa::track;
use turbocursa::versus::Versus;
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

fn fnv(h: &mut u64, v: u64) {
    *h ^= v;
    *h = h.wrapping_mul(0x100000001b3);
}

#[test]
fn track_geometry_is_sane_for_many_seeds() {
    let mut bad = Vec::new();
    for seed in 0..300u64 {
        let t = track::build(seed);
        let n = t.segs.len();
        let mut max_abs_y = 0f32;
        let mut max_slope = 0f32;
        for (i, s) in t.segs.iter().enumerate() {
            assert!(
                s.y0.is_finite() && s.y1.is_finite() && s.curve.is_finite() && s.tone.is_finite(),
                "non-finite segment {i} seed {seed}"
            );
            max_abs_y = max_abs_y.max(s.y0.abs());
            // slope between segment start and end, ignoring nothing: the seam must be smooth too
            max_slope = max_slope.max((s.y1 - s.y0).abs());
        }
        // consecutive segments must join without a jump
        let mut max_join = 0f32;
        for w in t.segs.windows(2) {
            max_join = max_join.max((w[0].y1 - w[1].y0).abs());
        }
        max_join = max_join.max((t.segs[n - 1].y1 - t.segs[0].y0).abs());
        if max_abs_y > 25000.0 || max_slope > 300.0 || max_join > 1.0 {
            bad.push(format!("seed {seed}: max|y| {max_abs_y:.0}, max slope/seg {max_slope:.0}, worst join {max_join:.0}"));
        }
    }
    assert!(
        bad.is_empty(),
        "{} bad tracks, e.g.:\n{}",
        bad.len(),
        bad.iter().take(5).cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn render_never_panics_at_odd_view_sizes() {
    let sizes = [
        (640, 400),
        (640, 200),
        (320, 200),
        (321, 201),
        (800, 600),
        (100, 60),
        (64, 40),
        (32, 20),
        (16, 12),
        (8, 8),
    ];
    let mut failures = Vec::new();
    for &(w, h) in &sizes {
        let r = catch_unwind(AssertUnwindSafe(|| {
            let mut g = Game::new(2);
            g.set_view(w, h);
            let mut fb = vec![0u32; w * h];
            for i in 0..90 {
                g.update(
                    1.0 / 60.0,
                    &Input {
                        accel: true,
                        turbo: i == 30,
                        ..Default::default()
                    },
                );
                render::render(&g, &mut fb);
            }
        }));
        if r.is_err() {
            failures.push(format!("{w}x{h}"));
        }
    }
    assert!(
        failures.is_empty(),
        "render panicked at sizes: {failures:?}"
    );
}

#[test]
fn hostile_dt_never_corrupts_state() {
    let dts = [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -1.0,
        -0.002,
        0.0,
        1e-30,
        0.016,
        0.05,
        1.0,
        1e9,
    ];
    for &dt in &dts {
        let mut g = Game::new(3);
        let mut rng = Lcg(7);
        let mut prev_pos = 0.0f32;
        for step in 0..600 {
            let inp = rng.input();
            g.update(dt, &inp);
            assert!(
                g.pos.is_finite() && g.x.is_finite() && g.speed.is_finite() && g.time.is_finite(),
                "dt={dt} step={step}: non-finite state"
            );
            assert!(
                g.pos >= prev_pos - 1e-3,
                "dt={dt} step={step}: car moved backwards ({} -> {})",
                prev_pos,
                g.pos
            );
            assert!(g.speed >= 0.0 && g.x.abs() <= 2.21);
            prev_pos = g.pos;
        }
        let mut fb = vec![0u32; W * H];
        render::render(&g, &mut fb);
    }
}

#[test]
fn synth_survives_hostile_sample_rates_and_params() {
    let rates = [
        0.0f32,
        -5.0,
        1.0,
        100.0,
        3999.0,
        8000.0,
        44100.0,
        48000.0,
        96000.0,
        384000.0,
        f32::NAN,
    ];
    let speeds = [0.0f32, 0.5, 1.0, 1.3, 50.0, -3.0, f32::NAN, f32::INFINITY];
    let mut failures = Vec::new();
    for &sr in &rates {
        let r = catch_unwind(AssertUnwindSafe(|| {
            let mut s = Synth::new(sr, 5);
            let mut buf = vec![0.0f32; 2048];
            let mut bad_samples = 0usize;
            for (i, &sp) in speeds.iter().enumerate() {
                let p = AudioParams {
                    speed: sp,
                    playing: i % 2 == 0,
                    turbo: i % 3 == 0,
                    offroad: i % 4 == 0,
                    crashes: i as u32,
                    cps: i as u32 / 2,
                    biome: (i % 5) as u8,
                    time_low: true,
                    ..Default::default()
                };
                for _ in 0..6 {
                    s.render(&mut buf, &p);
                    bad_samples += buf
                        .iter()
                        .filter(|x| !x.is_finite() || x.abs() > 1.0)
                        .count();
                }
            }
            // after hostile params a sane call must recover
            let mut buf2 = vec![0.0f32; 4096];
            s.render(
                &mut buf2,
                &AudioParams {
                    speed: 0.6,
                    playing: true,
                    ..Default::default()
                },
            );
            bad_samples += buf2.iter().filter(|x| !x.is_finite()).count();
            assert_eq!(bad_samples, 0, "non-finite or out-of-range samples");
        }));
        if r.is_err() {
            failures.push(sr);
        }
    }
    assert!(
        failures.is_empty(),
        "synth failed for sample rates: {failures:?}"
    );
}

#[test]
fn simulation_is_bit_for_bit_deterministic() {
    let run = || {
        let mut g = Game::new(9);
        let mut rng = Lcg(42);
        let mut h = 0xcbf29ce484222325u64;
        let mut fb = vec![0u32; W * H];
        for i in 0..1500 {
            let inp = rng.input();
            g.update(1.0 / 60.0, &inp);
            fnv(&mut h, g.pos.to_bits() as u64);
            fnv(&mut h, g.speed.to_bits() as u64);
            fnv(&mut h, g.score as u64);
            fnv(&mut h, g.particles.len() as u64);
            if i % 100 == 0 {
                render::render(&g, &mut fb);
                for &c in fb.iter().step_by(97) {
                    fnv(&mut h, c as u64);
                }
            }
        }
        h
    };
    assert_eq!(run(), run());
}

#[test]
fn threaded_asset_generation_is_deterministic() {
    let sum = |a: &Assets| {
        let mut h = 0xcbf29ce484222325u64;
        for img in a
            .broadleaf
            .iter()
            .chain(a.conifer.iter().flatten())
            .chain(a.dead.iter().flatten())
            .chain(a.rocks.iter().flatten())
            .chain(a.gate.iter())
        {
            for t in &img.mips {
                for &p in &t.px {
                    fnv(&mut h, p as u64);
                }
            }
        }
        for &p in a.clouds.px.iter().chain(a.nebula.px.iter()) {
            fnv(&mut h, p as u64);
        }
        for r in &a.ridges {
            for &v in r {
                fnv(&mut h, v.to_bits() as u64);
            }
        }
        h
    };
    assert_eq!(sum(&Assets::generate(5)), sum(&Assets::generate(5)));
    assert_ne!(
        sum(&Assets::generate(5)),
        sum(&Assets::generate(6)),
        "different seeds must differ"
    );
}

#[test]
fn render_fuzz_over_positions_states_and_seeds() {
    let mut fb = vec![0u32; W * H];
    let mut worst_black = 0.0f64;
    for seed in 0..10u64 {
        let mut g = Game::new(seed);
        let len = g.track.length;
        for &frac in &[
            0.0f32, 0.0005, 0.1, 0.33, 0.34, 0.5, 0.66, 0.67, 0.9, 0.97, 0.99, 0.9999,
        ] {
            for &x in &[-2.2f32, -1.0, 0.0, 1.0, 2.2] {
                for (sp, tt, st) in [
                    (0.0, 0.0, State::Playing),
                    (MAX_SPEED, 0.0, State::Playing),
                    (MAX_SPEED * 1.3, 2.0, State::Playing),
                    (0.0, 0.0, State::TimeUp),
                    (0.0, 0.0, State::Finished),
                ] {
                    g.pos = (len * frac).min(len - 1.0);
                    g.x = x;
                    g.speed = sp;
                    g.turbo_t = tt;
                    g.state = st;
                    g.shake = if x > 0.0 { 0.4 } else { 0.0 };
                    render::render(&g, &mut fb);
                    let black = fb.iter().filter(|&&c| c == 0).count() as f64 / fb.len() as f64;
                    worst_black = worst_black.max(black);
                    assert!(
                        !(0..H).any(|y| fb[y * W..(y + 1) * W].iter().all(|&c| c == 0)),
                        "fully black row: seed {seed} frac {frac} x {x}"
                    );
                }
            }
        }
    }
    assert!(
        worst_black < 0.01,
        "pure-black (unfilled) pixels: {:.3}%",
        worst_black * 100.0
    );
}

#[test]
fn versus_invariants_hold_under_random_play() {
    for seed in 0..6u64 {
        let mut v = Versus::new(seed);
        let mut rng = Lcg(seed + 100);
        let n = v.p[0].track.segs.len();
        assert_eq!(
            n,
            v.p[1].track.segs.len(),
            "both players need the same stage"
        );
        for i in 0..v.p[0].track.segs.len().min(200) {
            assert_eq!(
                v.p[0].track.segs[i].y0.to_bits(),
                v.p[1].track.segs[i].y0.to_bits()
            );
        }
        let mut frozen_at = None;
        let mut out = vec![0u32; W * H];
        for step in 0..6000 {
            // player 1 mostly drives well, player 2 randomly
            let a = Input {
                accel: true,
                left: rng.next().is_multiple_of(40),
                right: rng.next().is_multiple_of(40),
                ..Default::default()
            };
            let b = rng.input();
            let before = (v.p[0].pos, v.p[1].pos);
            let was_frozen = v.frozen;
            v.update(1.0 / 60.0, &[a, b]);
            if let Some(w) = v.winner {
                assert!(
                    v.p[w].state == State::Finished
                        || v.p[1 - w].state == State::Finished
                        || v.p[1 - w].state == State::Disqualified,
                    "winner must have finished or the rival was disqualified"
                );
            }
            if v.frozen {
                frozen_at.get_or_insert(step);
            }
            if was_frozen {
                assert_eq!(
                    (v.p[0].pos, v.p[1].pos),
                    before,
                    "state changed while frozen"
                );
            }
            if step % 400 == 0 {
                v.render(&mut out);
            }
        }
        v.render(&mut out);
    }
}

#[test]
fn checkpoints_and_finish_pay_out_exactly_once() {
    for seed in 0..8u64 {
        let mut g = Game::new(seed);
        let ncp = g.track.checkpoints.len() as u32;
        let mut finish_scores = 0;
        let mut last_score = 0;
        for _ in 0..(60 * 120) {
            let before = g.state;
            g.update(
                1.0 / 60.0,
                &Input {
                    accel: true,
                    ..Default::default()
                },
            );
            // no steering: car will bump into things, which is fine
            assert!(g.cps <= ncp, "checkpoint counted more than once");
            if before == State::Playing && g.state == State::Finished {
                finish_scores += 1;
            }
            assert!(g.score >= last_score, "score went down");
            last_score = g.score;
        }
        assert!(finish_scores <= 1);
    }
}

#[test]
fn all_seven_landscapes_appear_and_render_in_every_blend() {
    use turbocursa::NB;
    let mut seen = [0usize; NB];
    for seed in 0..120u64 {
        let t = track::build(seed);
        let mut present = [false; NB];
        for s in &t.segs {
            assert!((s.biome as usize) < NB);
            present[s.biome as usize] = true;
        }
        for (i, &p) in present.iter().enumerate() {
            seen[i] += p as usize;
        }
        assert_eq!(
            present.iter().filter(|&&x| x).count(),
            4,
            "seed {seed}: a stage has exactly four landscapes"
        );
    }
    assert!(
        seen.iter().all(|&n| n > 10),
        "every landscape must occur regularly: {seen:?}"
    );
    // render every biome one-hot and every pair 50/50, at several positions, at full and tiny size
    let mut fb = vec![0u32; W * H];
    let mut small = vec![0u32; 64 * 40];
    let mut g = Game::new(4);
    for a in 0..NB {
        for b in 0..NB {
            let mut w = [0.0f32; NB];
            w[a] += 0.5;
            w[b] += 0.5;
            g.biome_w = w;
            for frac in [0.05f32, 0.5, 0.95] {
                g.pos = g.track.length * frac;
                g.speed = MAX_SPEED;
                render::render(&g, &mut fb);
                assert!(
                    fb.iter().step_by(131).all(|&c| c <= 0xFFFFFF),
                    "stray alpha bits"
                );
            }
            let mut g2 = Game::new(4);
            g2.set_view(64, 40);
            g2.biome_w = w;
            render::render(&g2, &mut small);
        }
    }
}
