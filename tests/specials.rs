//! Unlockable service vehicles: police, fire engine, ambulance, helicopter.
use turbocursa::cars::*;
use turbocursa::game::*;
use turbocursa::pick::PickApp;
use turbocursa::profile::Profile;

fn game_as(seed: u64, class: VehicleClass) -> Game {
    let mut g = Game::new_mode(seed, Mode::Campaign);
    g.set_car(CarSpec::default().with_class(class));
    g.cars.clear();
    g.jam_cars.clear();
    g
}

#[test]
fn counts_and_parsing() {
    assert_eq!(CLASS_COUNT, 19);
    assert_eq!(CLASS_TOTAL, 23);
    assert_eq!(VehicleClass::parse("19"), None);
    assert_eq!(VehicleClass::parse("police"), Some(VehicleClass::Police));
    assert_eq!(
        VehicleClass::parse("helicopter"),
        Some(VehicleClass::Helicopter)
    );
    for i in CLASS_COUNT..CLASS_TOTAL {
        let c = VehicleClass::from_index(i);
        assert!(c.is_special());
        assert_eq!(VehicleClass::parse(c.name()), Some(c));
    }
    assert!(!VehicleClass::Monster.is_special());
}

#[test]
fn unlock_rules() {
    let mut p = Profile::new();
    assert_eq!(p.specials_mask(), 0);
    assert!(p.unlocked(0) && p.unlocked(18));
    p.clean_cities = 10;
    assert!(p.unlocked(19) && !p.unlocked(21));
    p.deliveries = 8;
    assert!(p.unlocked(21) && !p.unlocked(20));
    p.daily_runs = 3;
    p.skills = 4;
    assert!(p.unlocked(20));
    assert!(!p.unlocked(22), "the helicopter needs seven daily runs");
    p.daily_runs = 7;
    assert!(p.unlocked(22));
    assert_eq!(p.specials_mask(), 0b1111);
}

#[test]
fn profile_roundtrip_and_hostile_input() {
    let mut p = Profile::new();
    p.clean_cities = 12;
    p.deliveries = 9;
    p.skills = 5;
    p.daily_runs = 3;
    let s = p.encode();
    assert!(s.len() <= 512, "{}", s.len());
    let q = Profile::decode(&s);
    assert_eq!((q.clean_cities, q.deliveries, q.skills), (12, 9, 5));
    let h = Profile::decode("TP1;cc=99999999999;dv=-5;sk=abc");
    assert_eq!((h.clean_cities, h.deliveries, h.skills), (0, 0, 0));
}

#[test]
fn pick_screen_blocks_locked_vehicles() {
    let mut a = PickApp::new(19, 1);
    assert!(a.locked(19));
    assert!(!a.locked(3));
    a = PickApp::new(19, 1).with_profile(Default::default());
    assert_eq!(
        a.class, 0,
        "a locked class at the start falls back to the first"
    );
    let mut info = turbocursa::pick::ProfileInfo::default();
    info.specials = 0b0001;
    let a = PickApp::new(19, 1).with_profile(info);
    assert_eq!(a.class, 19);
    assert!(!a.locked(19) && a.locked(20));
}

#[test]
fn service_vehicles_are_not_fined_or_chased() {
    for class in [
        VehicleClass::Police,
        VehicleClass::Fire,
        VehicleClass::Ambulance,
    ] {
        let mut g = game_as(7, class);
        g.state = State::Playing;
        let before = g.violations;
        for _ in 0..600 {
            g.update(
                1.0 / 60.0,
                &Input {
                    accel: true,
                    ..Default::default()
                },
            );
        }
        assert_eq!(g.violations, before, "{class:?}");
        assert!(g.cops.is_empty(), "{class:?}");
    }
}

#[test]
fn helicopter_flies_over_traffic_and_lands_when_slow() {
    let mut g = game_as(3, VehicleClass::Helicopter);
    g.state = State::Playing;
    g.speed = 6000.0;
    assert!(!g.low(), "fast helicopter is airborne");
    g.speed = 500.0;
    assert!(g.low(), "slow helicopter is on the ground");
    let mut s = game_as(3, VehicleClass::Sports);
    s.speed = 6000.0;
    assert!(s.low());
}

#[test]
fn traffic_gives_way_to_emergency_vehicles() {
    let mut g = Game::new_mode(5, Mode::Campaign);
    g.set_car(CarSpec::default().with_class(VehicleClass::Ambulance));
    g.jam_cars.clear();
    g.cars.truncate(1);
    g.state = State::Playing;
    let len = g.track.length;
    let pz = g.pos + PLAYER_Z;
    g.cars[0].z = (pz + 3000.0).rem_euclid(len);
    g.cars[0].x = g.x + 0.1;
    g.cars[0].base = 0.0;
    for _ in 0..30 {
        g.update(1.0 / 60.0, &Input::default());
    }
    assert!(
        (g.cars[0].x - g.x).abs() > 0.3,
        "the car moved aside: {}",
        (g.cars[0].x - g.x).abs()
    );
}

#[test]
fn every_special_renders_in_preview() {
    for i in CLASS_COUNT..CLASS_TOTAL {
        let car = CarSpec::default().with_class(VehicleClass::from_index(i));
        let mut fb = vec![0u32; 640 * 400];
        turbocursa::render::draw_car_preview(&mut fb, 640, 400, 320.0, 216.0, 210.0, &car);
        assert!(fb.iter().any(|&p| p != 0), "class {i} drew nothing");
    }
}

#[test]
fn service_vehicles_cannot_farm_the_clean_city_unlock() {
    for seed in 1..6u64 {
        let mut g = game_as(seed, VehicleClass::Ambulance);
        g.practice();
        g.state = State::Playing;
        g.time = 9999.0;
        for _ in 0..6000 {
            let inp = g.autopilot();
            g.update(1.0 / 60.0, &inp);
            if g.state != State::Playing {
                break;
            }
        }
        assert_eq!(g.clean_cities, 0, "seed {seed}");
    }
}

#[test]
fn a_maxed_out_profile_still_fits_the_save_limit() {
    let mut p = Profile::new();
    p.xp = u64::MAX;
    p.day = 4_000_000;
    p.minutes_today = 1440.0;
    p.games_today = 5000;
    p.streak = 999;
    p.best_streak = 999;
    p.daily_day = 4_000_000;
    p.daily_score = 100_000_000;
    p.daily_runs = 60_000;
    p.clean_cities = 60_000;
    p.deliveries = 60_000;
    p.skills = 60_000;
    p.col.classes = u32::MAX;
    let s = p.encode();
    assert!(
        s.len() <= 512,
        "a full save is {} bytes: it would be discarded on load",
        s.len()
    );
}
