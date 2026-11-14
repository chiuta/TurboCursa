use turbocursa::audio::{AudioParams, Synth};

const SR: f32 = 44_100.0;

fn render(p: &AudioParams, secs: f32) -> Vec<f32> {
    let mut s = Synth::new(SR, 5);
    let mut out = vec![0.0f32; (SR * secs) as usize * 2];
    for ch in out.chunks_mut(2048) {
        s.render_stereo(ch, p);
    }
    out
}

fn rms(v: &[f32], ch: usize) -> f32 {
    let n = v.len() / 2;
    (v.chunks(2).map(|f| f[ch] * f[ch]).sum::<f32>() / n as f32).sqrt()
}

fn driving() -> AudioParams {
    AudioParams {
        speed: 0.8,
        playing: true,
        weather: 2,
        daylight: 1.0,
        ..AudioParams::default()
    }
}

#[test]
fn the_two_channels_differ_and_stay_in_range() {
    let o = render(&driving(), 3.0);
    assert!(o.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
    let (mut dot, mut ll, mut rr) = (0.0f64, 0.0f64, 0.0f64);
    for f in o.chunks(2) {
        dot += (f[0] * f[1]) as f64;
        ll += (f[0] * f[0]) as f64;
        rr += (f[1] * f[1]) as f64;
    }
    let corr = dot / (ll.sqrt() * rr.sqrt());
    assert!(
        corr < 0.95,
        "real stereo, not a doubled mono signal (corr {corr})"
    );
    assert!(rms(&o, 0) > 0.01 && rms(&o, 1) > 0.01);
}

#[test]
fn mono_is_the_average_of_the_stereo_pair() {
    let p = driving();
    let st = render(&p, 1.0);
    let mut s = Synth::new(SR, 5);
    let mut mono = vec![0.0f32; st.len() / 2];
    for ch in mono.chunks_mut(2048) {
        s.render(ch, &p);
    }
    for (i, m) in mono.iter().enumerate() {
        let avg = (st[2 * i] + st[2 * i + 1]) * 0.5;
        assert!((m - avg).abs() < 1e-5, "sample {i}");
    }
}

#[test]
fn an_overtaking_ambulance_sits_on_its_side() {
    let at = |pan: f32| AudioParams {
        emerg: 1.0,
        emerg_kind: 1,
        emerg_pan: pan,
        ..AudioParams::default()
    };
    let (l, r) = (render(&at(-0.9), 1.5), render(&at(0.9), 1.5));
    assert!(
        rms(&l, 0) > rms(&l, 1) * 2.0,
        "left side is louder in the left ear"
    );
    assert!(
        rms(&r, 1) > rms(&r, 0) * 2.0,
        "right side is louder in the right ear"
    );
}

#[test]
fn police_ambulance_and_fire_engine_sound_different() {
    let own = |k: u8| {
        render(
            &AudioParams {
                own_siren: k,
                ..AudioParams::default()
            },
            2.0,
        )
    };
    let (a, b, c) = (own(1), own(2), own(3));
    let dist = |x: &[f32], y: &[f32]| {
        x.iter().zip(y).map(|(p, q)| (p - q).abs()).sum::<f32>() / x.len() as f32
    };
    for (x, y) in [(&a, &b), (&b, &c), (&a, &c)] {
        assert!(dist(x, y) > 0.005, "two sirens render the same");
    }
    // an overtaking fire engine is not an overtaking ambulance either
    let em = |k: u8| {
        render(
            &AudioParams {
                emerg: 1.0,
                emerg_kind: k,
                ..AudioParams::default()
            },
            2.0,
        )
    };
    assert!(dist(&em(1), &em(2)) > 0.005);
    // it is quieter far away and its pitch drops after it has passed
    let far = render(
        &AudioParams {
            emerg: 0.1,
            emerg_kind: 1,
            ..AudioParams::default()
        },
        1.0,
    );
    assert!(rms(&far, 0) < rms(&em(1), 0));
    let gone = render(
        &AudioParams {
            emerg: 1.0,
            emerg_kind: 1,
            emerg_passed: true,
            ..AudioParams::default()
        },
        2.0,
    );
    assert!(dist(&em(1), &gone) > 0.005);
}

#[test]
fn a_helicopter_has_a_rotor_sound() {
    let heli = |h: u8| {
        render(
            &AudioParams {
                heli: h,
                playing: true,
                ..AudioParams::default()
            },
            2.0,
        )
    };
    let (none, idle, fly) = (heli(0), heli(1), heli(2));
    assert!(rms(&idle, 0) > rms(&none, 0));
    assert!(rms(&fly, 0) > rms(&idle, 0), "flying is louder than idling");
    // the blade slap pulses: the level swings a lot within a tenth of a second
    let win: Vec<f32> = fly
        .chunks(2)
        .map(|f| f[0].abs())
        .collect::<Vec<_>>()
        .chunks(441)
        .map(|c| c.iter().sum::<f32>() / c.len() as f32)
        .collect();
    let (mn, mx) = win
        .iter()
        .fold((f32::MAX, 0.0f32), |(a, b), &v| (a.min(v), b.max(v)));
    assert!(mx > mn * 1.5, "the rotor beats ({mn} .. {mx})");
}
