use turbocursa::champ::*;
use turbocursa::daily::*;

#[test]
fn champ_code_roundtrip_and_typo() {
    for kind in 0..4u8 {
        let c = ChampCode::from_seed(42 + kind as u64, kind, 7, 2);
        let s = c.encode();
        assert!(s.starts_with("TCC1-"));
        assert_eq!(ChampCode::decode(&s), Some(c));
        assert_eq!(ChampCode::decode(&s.to_lowercase()), Some(c));
        assert_eq!(c.app().n, KINDS[kind as usize]);
        // any single changed digit is caught
        let b: Vec<char> = s.chars().collect();
        for i in 5..b.len() {
            if b[i] == '-' {
                continue;
            }
            let mut m = b.clone();
            m[i] = if b[i] == '0' { '1' } else { '0' };
            assert_eq!(
                ChampCode::decode(&m.iter().collect::<String>()),
                None,
                "{s} @{i}"
            );
        }
    }
    assert_eq!(ChampCode::decode("TCC1-"), None);
    assert_eq!(ChampCode::decode(&"A".repeat(500)), None);
    assert_eq!(ChampCode::decode("TCS1-00000"), None);
}

#[test]
fn tour_has_a_country_and_stages_are_distinct() {
    let c = ChampCode::from_seed(5, TOUR_KIND, 0, 1);
    let app = c.app();
    assert_eq!(app.n, 8);
    assert!(app.country.is_some());
    let mut seeds = std::collections::HashSet::new();
    for i in 0..app.n {
        assert!(seeds.insert(turbocursa::rally::plan_n(app.seed, i, app.n).seed));
    }
}

#[test]
fn long_championship_runs_to_the_end() {
    use turbocursa::game::Input;
    let mut app = ChampCode::from_seed(9, 0, 0, 0).app();
    let go = Input {
        accel: true,
        ..Default::default()
    };
    let mut guard = 0u64;
    while !app.finished() && guard < 2_000_000 {
        guard += 1;
        app.update(0.05, &go, guard.is_multiple_of(40), false);
    }
    assert!(app.finished(), "7 stages must end");
    assert_eq!(app.results.len(), 7);
}

#[test]
fn weekly_is_stable_and_valid() {
    let mut mods = std::collections::HashSet::new();
    for w in 0..400u32 {
        let d = weekly(w);
        assert_eq!(d, weekly(w));
        assert!(d.class < turbocursa::cars::CLASS_COUNT);
        assert!(d.level < turbocursa::game::LEVEL_COUNT);
        assert!(d.weather <= turbocursa::env::WEATHER_COUNT);
        assert!((1..=4).contains(&d.time));
        mods.insert(weekly_modifier(w));
        let c = weekly_code(w, 1234, 56.7, true);
        assert_eq!(parse_weekly_code(&c), Some((w, 1234, 56.7, true)));
        assert_eq!(parse_code(&c), None);
    }
    assert_eq!(mods.len(), WEEKLY_MODIFIERS);
    assert_eq!(week_of(13), 1);
}

#[test]
fn ghost_import_is_hostile_safe() {
    use turbocursa::ghost::Ghost;
    assert!(Ghost::decode("TG1;1;0;0;10;5;zz").is_none());
    assert!(Ghost::decode("").is_none());
    let g = Ghost {
        seed: 3,
        mode: 1,
        class: 2,
        secs: 12.3,
        score: 99,
        pts: vec![(0.0, 0.0), (100.0, 0.5)],
    };
    assert_eq!(Ghost::decode(&g.encode()).map(|x| x.pts.len()), Some(2));
}
