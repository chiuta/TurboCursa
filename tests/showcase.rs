use turbocursa::share::StageCode;
use turbocursa::showcase;

#[test]
fn twelve_stages_round_trip_and_are_distinct() {
    let mut seen = std::collections::HashSet::new();
    for n in 1..=showcase::COUNT {
        let c = showcase::stage(n).expect("stage");
        let code = c.encode();
        assert_eq!(StageCode::decode(&code), Some(c), "stage {n}");
        assert!(seen.insert(code), "stage {n} repeats another code");
    }
    assert!(showcase::stage(0).is_none() && showcase::stage(showcase::COUNT + 1).is_none());
}

#[test]
fn stages_build_and_run() {
    for n in 1..=showcase::COUNT {
        let mut g = showcase::stage(n).unwrap().game();
        for _ in 0..90 {
            g.update(
                1.0 / 30.0,
                &turbocursa::game::Input {
                    accel: true,
                    ..Default::default()
                },
            );
        }
        assert!(g.track.segs.len() > 100, "stage {n} has a road");
    }
}
