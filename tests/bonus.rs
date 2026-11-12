//! Proving ground (slalom rings, slope, ramp) and parking lots.
use turbocursa::bonus as b;
use turbocursa::game::*;
use turbocursa::track;

fn game(seed: u64) -> Game {
    let mut g = Game::new_mode(seed, Mode::Campaign);
    g.cars.clear();
    g.jam_cars.clear();
    g.invuln = 0.0;
    g.shield_t = 0.0;
    g
}

fn at(g: &mut Game, z: f32, x: f32, v: f32) {
    g.pos = z - PLAYER_Z;
    g.x = x;
    g.speed = v;
    g.state = State::Playing;
}

fn proving(g: &Game) -> b::Area {
    *g.areas
        .iter()
        .find(|a| a.kind == b::PROVING)
        .expect("a proving ground")
}

#[test]
fn every_stage_has_one_proving_ground_and_parking_lots_clear_of_everything() {
    for seed in 1..50u64 {
        let g = Game::new_mode(seed, Mode::Campaign);
        assert_eq!(
            g.areas.iter().filter(|a| a.kind == b::PROVING).count(),
            1,
            "seed {seed}"
        );
        assert!(
            g.areas.iter().filter(|a| a.kind == b::PARKING).count() >= 1,
            "seed {seed}"
        );
        for (i, a) in g.areas.iter().enumerate() {
            assert!((a.a..a.b).all(|s| g.track.segs[s].feat == track::F_NONE));
            assert!(g.city.zones.iter().all(|z| a.b < z.a || a.a > z.b));
            assert!(g.areas.iter().skip(i + 1).all(|o| a.b < o.a || a.a > o.b));
            let ms = g.mission.unwrap();
            assert!(a.b < ms.pick || a.a > ms.pick);
            assert!(a.b < ms.drop || a.a > ms.drop);
        }
    }
    assert!(Game::new_mode(3, Mode::Zen).areas.is_empty());
}

#[test]
fn the_hill_rises_and_falls_back_to_the_original_level() {
    let g = game(4);
    let fresh = track::build(4);
    let a = proving(&g);
    let mut peak = 0.0f32;
    for i in 1..g.track.segs.len() {
        assert!(
            (g.track.segs[i].y0 - g.track.segs[i - 1].y1).abs() < 0.01,
            "seg {i} is a step"
        );
    }
    for i in a.a..a.b {
        peak = peak.max(g.track.segs[i].y1 - fresh.segs[i].y1);
    }
    assert!(peak > b::HUMP_H * 0.95, "the hill is {peak} high");
    for i in a.b..a.b + 50 {
        assert!((g.track.segs[i].y1 - fresh.segs[i].y1).abs() < 0.01);
    }
    for i in (a.a - 20)..a.a + b::HUMP_FROM {
        assert!((g.track.segs[i].y1 - fresh.segs[i].y1).abs() < 0.01);
    }
}

#[test]
fn the_proving_ground_holds_cones_rings_a_pad_a_ramp_and_coins_once() {
    let mut g = game(4);
    let a = proving(&g);
    let count = |g: &Game, k: u8| {
        g.items
            .iter()
            .filter(|i| i.z >= a.az() && i.z <= a.bz() && i.kind == k)
            .count()
    };
    assert_eq!(count(&g, I_CONE), 6);
    assert_eq!(count(&g, I_RAMP_L), 1);
    assert_eq!(count(&g, I_PAD), 1);
    assert_eq!(count(&g, I_COIN), 10);
    let rings = |g: &Game| {
        g.orbs
            .iter()
            .filter(|o| o.z >= a.az() && o.z <= a.bz() && o.kind == 1)
            .count()
    };
    assert_eq!(rings(&g), 12);
    // changing the level regenerates the items: the area must not be doubled
    g.set_level(Difficulty::Hard);
    g.set_level(Difficulty::Expert);
    assert_eq!(count(&g, I_RAMP_L), 1);
    assert_eq!(count(&g, I_CONE), 6);
    assert_eq!(rings(&g), 12);
    assert!(
        g.items.windows(2).all(|w| w[0].z <= w[1].z),
        "items stay sorted"
    );
}

#[test]
fn slopes_slow_the_climb_and_speed_the_descent() {
    let g0 = game(4);
    let a = proving(&g0);
    let coast = |up: bool| {
        let mut g = game(4);
        g.items.clear();
        let seg = (a.a..a.b)
            .find(|&s| {
                let sg = &g.track.segs[s];
                let gr = (sg.y1 - sg.y0) / 200.0;
                if up {
                    gr > 0.12
                } else {
                    gr < -0.12
                }
            })
            .expect("a steep part");
        at(&mut g, seg as f32 * 200.0 + 100.0, 0.0, 7000.0);
        let v0 = g.speed;
        for _ in 0..10 {
            let v = g.speed;
            g.update(1.0 / 60.0, &Input::default());
            g.pos = (seg as f32 * 200.0 + 100.0) - PLAYER_Z; // stay on the slope
            g.speed = v + (g.speed - v);
        }
        g.speed - v0
    };
    assert!(
        coast(true) < coast(false) - 100.0,
        "uphill {} downhill {}",
        coast(true),
        coast(false)
    );
}

#[test]
fn finishing_the_proving_ground_with_enough_rings_pays() {
    let mut g = game(4);
    let a = proving(&g);
    let hit = |g: &mut Game, n: usize| {
        let mut k = 0;
        for o in g.orbs.iter_mut().filter(|o| o.z >= a.az() && o.z <= a.bz()) {
            o.taken = k < n;
            k += 1;
        }
    };
    hit(&mut g, 3);
    at(&mut g, a.bz() + 200.0 * 6.0, 0.0, 6000.0);
    let s0 = g.score;
    g.update(1.0 / 60.0, &Input::default());
    assert_eq!(g.tracks_clear, 0);
    assert!(g.score < s0 + 2500);
    let mut g = game(4);
    hit(&mut g, 10);
    at(&mut g, a.bz() + 200.0 * 6.0, 0.0, 6000.0);
    let (s0, t0) = (g.score, g.time);
    g.update(1.0 / 60.0, &Input::default());
    assert_eq!(g.tracks_clear, 1);
    assert!(g.score >= s0 + 2500 && g.time > t0 + 5.0);
}

fn park(g: &mut Game, lot: b::Area, x: f32, secs: f32) {
    for _ in 0..(secs * 60.0) as usize {
        at(g, lot.az() + 1500.0, x, 0.0);
        g.update(1.0 / 60.0, &Input::default());
    }
}

#[test]
fn parking_pays_and_a_centred_slot_pays_more() {
    let g0 = game(4);
    let lot = *g0.areas.iter().find(|a| a.kind == b::PARKING).unwrap();
    // in a slot
    let mut g = game(4);
    let (s0, t0) = (g.score, g.time);
    park(&mut g, lot, b::SLOTS[0], 1.8);
    assert_eq!(g.parked, 1);
    assert_eq!(g.banner, "PERFECT PARKING!");
    assert!(g.score >= s0 + 2500 && g.time > t0 + 2.0);
    // on the dividing line: parked, not perfect
    let mut g = game(4);
    let s0 = g.score;
    park(&mut g, lot, 1.65, 1.8);
    assert_eq!(g.parked, 1);
    assert_eq!(g.banner, "PARKED!");
    assert!(g.score >= s0 + 1000 && g.score < s0 + 2500);
    // on the road, or too briefly: nothing
    let mut g = game(4);
    park(&mut g, lot, 0.5, 3.0);
    park(&mut g, lot, 1.65, 0.8);
    assert_eq!(g.parked, 0);
    // only once per lot
    let mut g = game(4);
    park(&mut g, lot, b::SLOTS[1], 1.8);
    park(&mut g, lot, b::SLOTS[1], 3.0);
    assert_eq!(g.parked, 1);
}

#[test]
fn a_parking_lot_has_no_off_road_drag() {
    let g0 = game(4);
    let lot = *g0.areas.iter().find(|a| a.kind == b::PARKING).unwrap();
    let drive = |z: f32| {
        let mut g = game(4);
        g.shield_t = 0.0;
        at(&mut g, z, 1.5, 6000.0);
        for _ in 0..20 {
            g.update(
                1.0 / 60.0,
                &Input {
                    accel: false,
                    ..Input::default()
                },
            );
            g.pos = z - PLAYER_Z + g.pos.min(0.0) * 0.0 + (g.pos - (z - PLAYER_Z)).min(0.0);
            g.x = 1.5;
        }
        g.speed
    };
    // coast through the middle of the lot and on an ordinary verge elsewhere
    let (in_lot, outside) = (drive(lot.az() + 4000.0), drive(lot.bz() + 200.0 * 200.0));
    assert!(in_lot > outside + 200.0, "lot {in_lot} verge {outside}");
}

#[test]
fn the_new_banners_are_localised() {
    use turbocursa::i18n::{item_banner_in, LANGS};
    for k in [
        "PROVING GROUND",
        "PARKING BONUS",
        "PARKED!",
        "PERFECT PARKING!",
        "TRACK CLEAR!",
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
