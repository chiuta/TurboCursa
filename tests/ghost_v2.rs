use turbocursa::ghost::*;

fn sample(n: usize) -> Ghost {
    let mut pos = 0.0f32;
    let pts = (0..n)
        .map(|i| {
            pos += 900.0 + 300.0 * (i as f32 * 0.05).sin() + (i % 7) as f32;
            (pos.floor(), 1.6 * (i as f32 * 0.03).sin())
        })
        .collect();
    Ghost {
        seed: 42,
        mode: 1,
        class: 3,
        secs: 61.5,
        score: 12345,
        pts,
    }
}

#[test]
fn the_compact_format_round_trips_and_is_much_shorter() {
    let g = sample(300);
    let t = g.encode();
    assert!(t.starts_with("TG2;"));
    let back = Ghost::decode(&t).expect("decodes");
    assert_eq!(
        (back.seed, back.mode, back.class, back.score),
        (42, 1, 3, 12345)
    );
    assert_eq!(back.pts.len(), g.pts.len());
    for (a, b) in g.pts.iter().zip(&back.pts) {
        assert!((a.0 - b.0).abs() <= 1.0, "position {} vs {}", a.0, b.0);
        assert!((a.1 - b.1).abs() < 0.002, "lateral {} vs {}", a.1, b.1);
    }
    let per_sample = (t.len() - 30) as f32 / 300.0;
    assert!(per_sample < 4.0, "{per_sample} characters per sample");
    // a re-encoded ghost is stable
    assert_eq!(Ghost::decode(&back.encode()).unwrap().pts, back.pts);
}

#[test]
fn old_codes_still_load_and_hostile_codes_do_not() {
    let old = "TG1;1;0;0;10;5;0100800";
    assert!(Ghost::decode(old).is_some());
    for bad in [
        "TG2;1;0;0;10;5;A",        // odd number of values
        "TG2;1;0;0;10;5;gggggggg", // continuation forever
        "TG2;1;0;0;10;5;!!",       // not in the alphabet
        "TG2;x;0;0;10;5;AA",
        "TG2;1;0;0;10;5",
        &format!("TG2;1;0;0;10;5;{}", "AA".repeat(MAX_SAMPLES + 5)),
    ] {
        assert!(Ghost::decode(bad).is_none(), "{bad}");
    }
    assert!(Ghost::decode("TG2;1;0;0;10;5;").is_some_and(|g| g.pts.is_empty()));
}
