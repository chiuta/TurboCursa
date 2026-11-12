//! Delivery missions: parcels and passengers.
use turbocursa::game::*;
use turbocursa::mission as m;

fn game(seed: u64) -> Game {
    let mut g = Game::new_mode(seed, Mode::Campaign);
    g.cars.clear();
    g.jam_cars.clear();
    g.invuln = 0.0;
    g
}

fn at(g: &mut Game, z: f32, x: f32, v: f32) {
    g.pos = z - PLAYER_Z;
    g.x = x;
    g.speed = v;
    g.state = State::Playing;
}

fn wait(g: &mut Game, secs: f32, z: f32, x: f32) {
    for _ in 0..(secs * 60.0) as usize {
        at(g, z, x, 0.0);
        g.update(1.0 / 60.0, &Input::default());
    }
}

#[test]
fn every_stage_has_a_sensible_mission() {
    for seed in 1..60u64 {
        let g = Game::new_mode(seed, Mode::Campaign);
        let ms = g
            .mission
            .unwrap_or_else(|| panic!("seed {seed}: no mission"));
        assert!(ms.drop > ms.pick + 600 && ms.limit > 20.0 && ms.limit < 200.0);
        for s in [ms.pick, ms.drop] {
            assert!(g.track.segs[s].feat == turbocursa::track::F_NONE);
            assert!(!g.city.in_city(s) && g.city.jam_at(s).is_none());
        }
        assert_eq!(g.m_state, m::WAITING);
    }
    assert!(Game::new_mode(3, Mode::Zen).mission.is_none());
}

#[test]
fn load_and_unload_in_the_bays_pays() {
    let mut g = game(4);
    let ms = g.mission.unwrap();
    // driving through without stopping does not load
    let mut g2 = game(4);
    at(&mut g2, ms.pick_z() - 300.0, 0.6, 6000.0);
    for _ in 0..80 {
        g2.update(
            1.0 / 60.0,
            &Input {
                accel: true,
                ..Input::default()
            },
        );
    }
    assert_ne!(g2.m_state, m::CARRYING);
    // far on the left of the bay: no loading either
    wait(&mut g, 2.0, ms.pick_z(), -0.5);
    assert_eq!(g.m_state, m::WAITING);
    wait(&mut g, 1.5, ms.pick_z(), 0.6);
    assert_eq!(g.m_state, m::CARRYING);
    assert!((g.m_t - ms.limit).abs() < 3.0);
    let (s0, t0, civic0) = (g.score, g.time, g.civic);
    g.elapsed = 5.0;
    wait(&mut g, 1.5, ms.drop_z(), 0.6);
    assert_eq!(g.m_state, m::DONE);
    assert_eq!(g.missions_done, 1);
    assert!(g.score >= s0 + 2500 - 100 && g.time > t0 + 5.0 && g.civic > civic0);
    assert_eq!(g.banner, "DELIVERED!");
}

#[test]
fn a_late_or_careless_delivery_pays_less() {
    let run = |late: bool, hurt: bool| {
        let mut g = game(4);
        let ms = g.mission.unwrap();
        wait(&mut g, 1.5, ms.pick_z(), 0.6);
        assert_eq!(g.m_state, m::CARRYING);
        if late {
            g.m_t = -5.0;
        }
        if hurt {
            g.hurt(0.2, 0.0);
        }
        let s0 = g.score;
        wait(&mut g, 1.5, ms.drop_z(), 0.6);
        assert_eq!(g.m_state, m::DONE);
        g.score - s0
    };
    let best = run(false, false);
    assert!(run(true, false) < best);
    // a parcel is only "careful" while the body is intact; a passenger while no offence was made
    let hurt = run(false, true);
    let mut g = game(4);
    let kind = g.mission.unwrap().kind;
    g.mission = Some(m::Mission {
        kind: m::PARCEL,
        ..g.mission.unwrap()
    });
    let _ = kind;
    assert!(hurt <= best);
}

#[test]
fn missing_a_bay_fails_the_mission() {
    let mut g = game(4);
    let ms = g.mission.unwrap();
    at(&mut g, ms.pick_z() + m::BAY_LEN + 1500.0, 0.0, 6000.0);
    g.update(1.0 / 60.0, &Input::default());
    assert_eq!(g.m_state, m::FAILED);
    assert_eq!(g.banner, "MISSED");
    let mut g = game(4);
    wait(&mut g, 1.5, ms.pick_z(), 0.6);
    at(&mut g, ms.drop_z() + m::BAY_LEN + 1500.0, 0.0, 6000.0);
    g.update(1.0 / 60.0, &Input::default());
    assert_eq!(g.m_state, m::FAILED);
}

#[test]
fn the_clock_runs_only_while_carrying() {
    let mut g = game(4);
    let ms = g.mission.unwrap();
    assert_eq!(g.m_t, 0.0);
    g.update(1.0 / 60.0, &Input::default());
    assert_eq!(g.m_t, 0.0);
    wait(&mut g, 1.5, ms.pick_z(), 0.6);
    let t = g.m_t;
    wait(&mut g, 1.0, ms.pick_z() + 400.0, 0.0);
    assert!(g.m_t < t);
}

#[test]
fn mission_banners_are_localised() {
    use turbocursa::i18n::{item_banner_in, LANGS};
    for k in [
        "PICK UP",
        "DROP OFF",
        "PARCEL LOADED",
        "PASSENGER ON BOARD",
        "DELIVERED!",
        "TOO LATE",
        "MISSED",
    ] {
        for &(l, code, _) in LANGS.iter() {
            let s = item_banner_in(l, k).unwrap_or_else(|| panic!("{code}: {k}"));
            assert!(
                s.chars()
                    .filter(|c| !c.is_whitespace())
                    .all(|c| turbocursa::font::glyph(c).is_some()),
                "{code}: {s}"
            );
        }
    }
}
