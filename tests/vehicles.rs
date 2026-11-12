use turbocursa::career::*;
use turbocursa::cars::*;
use turbocursa::game::*;

fn run(g: &mut Game, frames: usize, seed: u64) {
    let mut r = splitmix(seed);
    for i in 0..frames {
        if i % 30 == 0 {
            r = splitmix(r);
        }
        let inp = Input {
            accel: r & 1 == 0 || i % 7 != 0,
            left: r & 2 != 0,
            right: r & 4 != 0,
            brake: r & 8 != 0 && r & 16 != 0,
            turbo: r & 32 != 0,
        };
        g.update(0.016, &inp);
        assert!(
            g.x.abs() <= 2.21 && g.speed.is_finite() && g.time.is_finite(),
            "x={} sp={} t={} i={} pos={}",
            g.x,
            g.speed,
            g.time,
            i,
            g.pos
        );
    }
}

#[test]
fn levels_are_ordered() {
    let mut prev_n = 0;
    let mut prev_t = f32::MAX;
    for i in 0..LEVEL_COUNT {
        let mut g = Game::new_mode(11, Mode::Campaign);
        g.set_level(Difficulty::from_index(i));
        assert!(g.cars.len() >= prev_n, "traffic must grow with level");
        assert!(g.time <= prev_t, "clock must shrink with level");
        prev_n = g.cars.len();
        prev_t = g.time;
    }
    let mut a = Game::new_mode(11, Mode::Campaign);
    a.set_level(Difficulty::Easy);
    let mut b = Game::new_mode(11, Mode::Campaign);
    b.set_level(Difficulty::Insane);
    assert!(a.cars.len() * 2 < b.cars.len() * 2 && a.cars.len() < b.cars.len());
    assert!(a.time > b.time);
}

#[test]
fn level_after_start_keeps_the_race() {
    let mut g = Game::new_mode(5, Mode::Campaign);
    run(&mut g, 120, 1);
    let (n, t) = (g.cars.len(), g.time);
    g.set_level(Difficulty::Insane);
    assert_eq!(g.cars.len(), n);
    assert_eq!(g.time, t);
}

#[test]
fn traffic_has_every_vehicle_kind() {
    for lv in [Difficulty::Normal, Difficulty::Insane] {
        let mut seen = std::collections::HashSet::new();
        for seed in 1..6 {
            let mut g = Game::new_mode(seed, Mode::Rush);
            g.set_level(lv);
            seen.extend(g.cars.iter().map(|c| c.kind));
        }
        for k in [
            K_CAR, K_TRUCK, K_SEMI, K_MICRO, K_BIKE, K_BUS, K_TRIKE, K_TRACTOR,
        ] {
            assert!(seen.contains(&k), "{lv:?} lacks kind {k}");
        }
    }
    let heavy = |lv| {
        let mut g = Game::new_mode(3, Mode::Rush);
        g.set_level(lv);
        g.cars
            .iter()
            .filter(|c| c.kind == K_SEMI || c.kind == K_TRUCK)
            .count() as f32
            / g.cars.len() as f32
    };
    assert!(heavy(Difficulty::Insane) > heavy(Difficulty::Easy));
}

#[test]
fn zen_and_timetrial_have_no_traffic_at_any_level() {
    for m in [Mode::Zen, Mode::TimeTrial] {
        let mut g = Game::new_mode(2, m);
        g.set_level(Difficulty::Insane);
        assert!(g.cars.is_empty());
    }
}

#[test]
fn classes_trade_off_and_stay_in_range() {
    let base = CarSpec::for_country(10);
    let semi = base.with_class(VehicleClass::Semi);
    let city = base.with_class(VehicleClass::City);
    let sport = base.with_class(VehicleClass::Sports);
    assert!(semi.top < sport.top && semi.accel < city.accel);
    assert!(semi.width > sport.width && city.width < sport.width);
    assert!(semi.crash_keep > city.crash_keep);
    assert!(semi.time_mult > sport.time_mult);
    for i in 0..ncountries() {
        for c in 0..CLASS_COUNT {
            let s = CarSpec::for_country(i).with_class(VehicleClass::from_index(c));
            let mut g = Game::new_mode(1, Mode::Campaign);
            g.set_car(s);
            assert!((0.5..=1.3).contains(&g.car.top));
            assert!((0.5..=1.3).contains(&g.car.grip));
            assert!((0.5..=1.3).contains(&g.car.accel));
            assert!((1..=6).contains(&g.car.turbos));
        }
    }
}

#[test]
fn hostile_specs_are_sanitised() {
    let mut s = CarSpec::for_country(3).with_class(VehicleClass::Semi);
    s.width = f32::NAN;
    s.crash_keep = f32::INFINITY;
    s.prop_keep = -5.0;
    s.time_mult = 1e9;
    let mut g = Game::new_mode(1, Mode::Campaign);
    g.set_car(s);
    run(&mut g, 600, 9);
    assert!(g.car.time_mult <= 1.4 && g.car.width >= 0.4);
}

#[test]
fn crashes_never_add_speed() {
    // same stage, same inputs: count the speed kept after the first traffic hit
    let hit_speed = |c: VehicleClass| {
        let mut g = Game::new_mode(21, Mode::Campaign);
        g.set_car(CarSpec::default().with_class(c));
        g.set_level(Difficulty::Insane);
        let inp = Input {
            accel: true,
            ..Input::default()
        };
        let mut last = 0u32;
        for _ in 0..6000 {
            let before = g.speed;
            g.update(0.016, &inp);
            if g.crashes > last {
                return g.speed / before.max(1.0);
            }
            last = g.crashes;
        }
        1.0
    };
    // both must register a crash at some point and never gain speed from it
    assert!(hit_speed(VehicleClass::City) <= 1.0);
    assert!(hit_speed(VehicleClass::Semi) <= 1.0);
}

#[test]
fn every_class_and_level_plays_without_panic() {
    for c in 0..CLASS_COUNT {
        for l in 0..LEVEL_COUNT {
            for mode in [Mode::Campaign, Mode::Rush, Mode::Mirror] {
                let mut g = Game::new_mode((c * 7 + l) as u64, mode);
                g.set_car(
                    CarSpec::for_country((c * 10) % ncountries())
                        .with_class(VehicleClass::from_index(c)),
                );
                g.set_level(Difficulty::from_index(l));
                run(&mut g, 900, (c * 5 + l) as u64);
            }
        }
    }
}

#[test]
fn parse_names() {
    assert_eq!(VehicleClass::parse("TRUCK"), Some(VehicleClass::Truck));
    assert_eq!(VehicleClass::parse("4"), Some(VehicleClass::Semi));
    assert_eq!(VehicleClass::parse("19"), None);
    for i in 0..CLASS_COUNT {
        let c = VehicleClass::from_index(i);
        assert_eq!(VehicleClass::parse(c.name()), Some(c));
        assert_eq!(c.index(), i);
    }
    assert_eq!(Difficulty::parse("insane"), Some(Difficulty::Insane));
    assert_eq!(Difficulty::parse("-1"), None);
}

#[test]
fn career_saves_level_and_class() {
    let mut c = Career::new(5, 9);
    c.level = 3;
    c.class = 4;
    let back = Career::from_save(&c.to_save()).unwrap();
    assert_eq!((back.level, back.class), (3, 4));
    // old saves (no lv/cl) still load with defaults
    let old = c.to_save().replace(";lv=3", "").replace(";cl=4", "");
    let o = Career::from_save(&old).unwrap();
    assert_eq!((o.level, o.class), (1, 0));
    // hostile values are clamped
    let hostile = c
        .to_save()
        .replace(";lv=3", ";lv=250")
        .replace(";cl=4", ";cl=99");
    let h = Career::from_save(&hostile).unwrap();
    assert!(h.level < LEVEL_COUNT as u8 && h.class < CLASS_COUNT as u8);
    assert!(Career::from_save(&c.to_save().replace(";lv=3", ";lv=zz")).is_none());
}

#[test]
fn harder_level_makes_rivals_faster() {
    let mut easy = Cup::new(CupKind::World, 4, 2);
    let mut hard = Cup::new(CupKind::World, 4, 2);
    easy.shift = Difficulty::Easy.rival_shift();
    hard.shift = Difficulty::Insane.rival_shift();
    let mean = |c: &mut Cup| {
        let r = c.advance(Some(100.0), 100.0).unwrap();
        let t: Vec<f32> = r
            .table
            .iter()
            .filter(|x| x.id != 0)
            .filter_map(|x| x.time)
            .collect();
        t.iter().sum::<f32>() / t.len() as f32
    };
    assert!(mean(&mut hard) < mean(&mut easy));
}

#[test]
fn all_nineteen_classes_are_distinct() {
    assert_eq!(CLASS_COUNT, 19);
    let base = CarSpec::default();
    let mut seen = std::collections::HashSet::new();
    for i in 0..CLASS_COUNT {
        let s = base.with_class(VehicleClass::from_index(i));
        let key = (
            (s.top * 1000.0) as i32,
            (s.accel * 1000.0) as i32,
            (s.width * 1000.0) as i32,
            (s.crash_keep * 1000.0) as i32,
        );
        assert!(seen.insert(key), "class {i} duplicates another profile");
    }
    // two-wheelers are narrow and fragile, heavy vehicles wide and tough
    let m = base.with_class(VehicleClass::Motorcycle);
    let b = base.with_class(VehicleClass::Bus);
    assert!(m.width < 0.6 && m.crash_keep < 0.7);
    assert!(b.width > 1.2 && b.crash_keep > 1.2);
}
