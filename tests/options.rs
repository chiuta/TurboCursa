use turbocursa::audio::*;
use turbocursa::settings::*;

fn energy(style: u8, own: u8) -> f32 {
    set_music_style(style);
    let mut s = Synth::new(48000.0, 5);
    let p = AudioParams {
        playing: true,
        speed: 0.5,
        own_siren: own,
        ..Default::default()
    };
    let mut buf = vec![0.0f32; 2 * 4800];
    let mut e = 0.0;
    for _ in 0..40 {
        s.render(&mut buf, &p);
        assert!(buf.iter().all(|v| v.is_finite() && v.abs() <= 1.5));
        e += buf.iter().map(|v| v * v).sum::<f32>();
    }
    set_music_style(0);
    e
}

#[test]
fn music_styles_and_sirens() {
    let normal = energy(0, 0);
    let calm = energy(1, 0);
    let off = energy(3, 0);
    assert!(normal > calm, "calm must be quieter: {normal} vs {calm}");
    assert!(calm >= off);
    for sir in 1..=3u8 {
        assert!(
            energy(3, sir) > off * 1.5 + 0.01,
            "siren {sir} must be audible"
        );
    }
}

#[test]
fn settings_roundtrip_and_hostile() {
    let s = Settings {
        music: 2,
        mph: true,
        keys: 3,
        livery: 4,
    };
    assert_eq!(Settings::decode(&s.encode()), s);
    assert_eq!(Settings::decode("garbage"), Settings::default());
    assert_eq!(
        Settings::decode("TS1;m=9;k=99;l=200;u=1;x=")
            .sanitized()
            .music,
        3
    );
    let big = Settings::decode(&format!("TS1;{}", "m=1;".repeat(100)));
    assert_eq!(big, Settings::default());
}

#[test]
fn secrets_and_units() {
    assert_eq!(secret_code(" gold-rush "), Some(1));
    assert_eq!(secret_code("nothing"), None);
    for l in 1..LIVERIES as u8 {
        assert!(livery_colours(l).is_some());
    }
    assert_eq!(display_speed(100), 100);
    Settings {
        mph: true,
        ..Default::default()
    }
    .apply();
    assert_eq!(display_speed(100), 62);
    Settings::default().apply();
}

#[test]
fn overtake_chain_is_bounded_and_resets() {
    use turbocursa::game::*;
    let mut g = Game::new_mode(11, Mode::Rush);
    let go = Input {
        accel: true,
        ..Default::default()
    };
    let mut best = 0;
    for i in 0..60_000 {
        g.update(0.02, &go);
        best = best.max(g.chain);
        assert!(g.chain <= 7);
        if g.state != State::Playing || i > 30_000 {
            break;
        }
    }
    assert_eq!(g.chain_best, best);
}
