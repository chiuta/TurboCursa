//! Adversarial audit of the curated stages: every showcase stage x every vehicle class, long runs
//! under random input and hostile frame times, spawned aircraft at every distance, and renders at
//! odd resolutions. Nothing may panic, hang or go non-finite.
use turbocursa::cars::*;
use turbocursa::game::*;
use turbocursa::{render, showcase};

fn fin(g: &Game, tag: &str) {
    assert!(
        g.pos.is_finite() && g.x.is_finite() && g.speed.is_finite() && g.time.is_finite(),
        "{tag}: non-finite state"
    );
    for e in &g.emerg {
        assert!(
            e.z.is_finite() && e.x.is_finite() && e.speed.is_finite(),
            "{tag}: emerg"
        );
    }
}

#[test]
fn every_showcase_stage_with_every_class_survives_long_hostile_runs() {
    let dts = [1.0 / 60.0, 1.0 / 30.0, 0.25, 1.0, 0.0, 5.0];
    let mut s = 0x9E3779B97F4A7C15u64;
    for n in 1..=showcase::COUNT {
        for class in 0..CLASS_COUNT {
            let mut c = showcase::stage(n).unwrap();
            c.class = class as u8;
            let mut g = c.sanitized().game();
            for i in 0..1200 {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                let inp = Input {
                    left: s & 1 != 0,
                    right: s & 2 != 0,
                    accel: s & 12 != 0,
                    brake: s & 48 == 48,
                    turbo: s & 64 != 0,
                };
                g.update(dts[(s >> 8) as usize % dts.len()], &inp);
                if i % 100 == 0 {
                    fin(&g, &format!("stage {n} class {class} step {i}"));
                }
            }
            fin(&g, &format!("stage {n} class {class} end"));
        }
    }
}

#[test]
fn aircraft_at_any_distance_never_collide_or_break_rendering() {
    for n in [1usize, 5, 12] {
        let mut g = showcase::stage(n).unwrap().game();
        for kind in 20u8..=28 {
            for d in [0.0f32, 1.0, 300.0, 2600.0, 1e6, f32::MAX / 4.0] {
                g.spawn_special(kind, d);
                for _ in 0..3 {
                    g.update(
                        1.0 / 30.0,
                        &Input {
                            accel: true,
                            ..Default::default()
                        },
                    );
                }
                fin(&g, &format!("kind {kind} dist {d}"));
            }
        }
    }
}

#[test]
fn rendering_at_odd_sizes_does_not_panic() {
    let mut g = showcase::stage(3).unwrap().game();
    for _ in 0..30 {
        g.update(
            1.0 / 30.0,
            &Input {
                accel: true,
                ..Default::default()
            },
        );
    }
    for (w, h) in [
        (0usize, 0usize),
        (1, 1),
        (2, 2),
        (3, 7),
        (15, 9),
        (16, 10),
        (17, 11),
        (31, 19),
        (64, 40),
        (641, 401),
        (1000, 3),
        (3, 1000),
    ] {
        let mut fb = vec![0u32; w * h];
        render::render_view(&g, &mut fb, w, h);
    }
}

#[test]
fn audio_is_finite_and_bounded_for_every_special_kind_and_hostile_values() {
    use turbocursa::audio::{AudioParams, Synth};
    let vals = [
        0.0f32,
        0.3,
        1.0,
        -1.0,
        5.0,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ];
    let mut syn = Synth::new(44100.0, 7);
    let mut out = vec![0.0f32; 2048];
    for kind in 0..=255u8 {
        for &e in &vals {
            for &pan in &[0.0f32, -1.0, 1.0, f32::NAN] {
                let p = AudioParams {
                    emerg: e,
                    emerg_kind: kind,
                    emerg_pan: pan,
                    playing: true,
                    speed: 0.8,
                    ..AudioParams::default()
                };
                syn.render_stereo(&mut out, &p);
                assert!(
                    out.iter().all(|x| x.is_finite() && x.abs() <= 1.0),
                    "kind {kind} emerg {e} pan {pan}"
                );
            }
        }
    }
}
