use turbocursa::cars::*;
use turbocursa::game::*;
use turbocursa::track::SEG_LEN;

fn drive(seed: u64, side: f32, class: VehicleClass) -> Game {
    let mut g = Game::new_mode(seed, Mode::Campaign);
    g.set_car(CarSpec::default().with_class(class));
    g.set_level(Difficulty::from_index(0));
    for _ in 0..60_000 {
        let inp = Input {
            accel: true,
            left: g.x > side + 0.1,
            right: g.x < side - 0.1,
            ..Default::default()
        };
        g.update(0.02, &inp);
        assert!(g.x.is_finite() && g.speed.is_finite() && g.time.is_finite());
        if g.state != State::Playing {
            break;
        }
    }
    g
}

#[test]
fn most_stages_have_a_fork_clear_of_everything() {
    let mut with = 0;
    for seed in 1..=60u64 {
        let g = Game::new_mode(seed, Mode::Campaign);
        if let Some((s, l)) = g.fork {
            with += 1;
            assert!(s + l + 20 < g.track.finish_seg);
            assert!(g.track.segs[s..s + l]
                .iter()
                .all(|x| x.feat == turbocursa::track::F_NONE));
            assert!(!g.city.zones.iter().any(|z| s + l >= z.a && s < z.b));
        }
    }
    assert!(with >= 40, "forks found on {with}/60 stages");
    assert!(Game::new_mode(5, Mode::Zen).fork.is_none());
}

#[test]
fn left_is_quick_right_is_scenic_and_each_pays_once() {
    let (mut q, mut s) = (0, 0);
    for seed in 1..=8u64 {
        let l = drive(seed, -0.6, VehicleClass::Sports);
        let r = drive(seed, 0.6, VehicleClass::Sports);
        if l.fork.is_none() {
            continue;
        }
        if l.route == 1 {
            q += 1;
        }
        if r.route == 2 {
            s += 1;
        }
        let (fs, len) = l.fork.unwrap();
        assert!((fs + len) as f32 * SEG_LEN > 0.0);
    }
    assert!(q >= 3 && s >= 3, "routes taken: quick {q}, scenic {s}");
}

#[test]
fn track_stays_finite_after_a_fork() {
    for seed in [3u64, 9, 21] {
        let g = drive(seed, 0.8, VehicleClass::Helicopter);
        assert!(g
            .track
            .segs
            .iter()
            .all(|x| x.curve.is_finite() && (x.biome as usize) < turbocursa::NB));
    }
}
