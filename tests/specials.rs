use turbocursa::audio::*;
use turbocursa::game::*;
use turbocursa::share::StageCode;
use turbocursa::{H, W};

fn game() -> Game {
    StageCode {
        seed: 11,
        mode: 0,
        ..StageCode::default()
    }
    .game()
}

#[test]
fn every_special_vehicle_is_drawn_and_leaves_the_road() {
    for kind in 0..=28u8 {
        let mut g = game();
        let inp = Input::default();
        for _ in 0..120 {
            g.update(1.0 / 60.0, &inp);
        }
        let mut plain = vec![0u32; W * H];
        turbocursa::render::render(&g, &mut plain);
        g.spawn_special(kind, Emerg::view_dist(kind));
        let mut fb = vec![0u32; W * H];
        turbocursa::render::render(&g, &mut fb);
        assert_ne!(fb, plain, "kind {kind} must change the picture");
        // drive on at speed: nothing panics, and the vehicle is eventually removed
        let gas = Input {
            accel: true,
            ..Input::default()
        };
        for _ in 0..60 * 40 {
            g.update(1.0 / 60.0, &gas);
            assert!(g.x.is_finite() && g.speed.is_finite());
        }
        assert!(
            g.emerg.len() <= 1,
            "kind {kind}: vehicles are removed after they have passed"
        );
    }
}

#[test]
fn every_special_vehicle_has_its_own_audible_sound() {
    let energy = |kind: u8| {
        let mut s = Synth::new(48_000.0, 3);
        let p = AudioParams {
            playing: true,
            speed: 0.4,
            emerg: if kind == 255 { 0.0 } else { 1.0 },
            emerg_kind: if kind == 255 { 0 } else { kind + 1 },
            ..Default::default()
        };
        let mut buf = vec![0.0f32; 2 * 4800];
        let mut e = 0.0f32;
        let mut wave = Vec::new();
        for _ in 0..40 {
            s.render_stereo(&mut buf, &p);
            assert!(
                buf.iter().all(|v| v.is_finite() && v.abs() <= 1.0),
                "kind {kind}"
            );
            e += buf.iter().map(|v| v * v).sum::<f32>();
            wave.extend_from_slice(&buf[..64]);
        }
        (e, wave)
    };
    let (quiet, _) = energy(255);
    let mut seen: Vec<Vec<f32>> = Vec::new();
    for kind in 0..=28u8 {
        let (e, w) = energy(kind);
        assert!(
            e > quiet * 1.05,
            "kind {kind} must be audible: {e} vs {quiet}"
        );
        assert!(
            seen.iter().all(|o| o != &w),
            "kind {kind} must not sound exactly like another"
        );
        seen.push(w);
    }
}
