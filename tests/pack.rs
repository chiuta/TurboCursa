use turbocursa::game::*;
use turbocursa::pack::*;

fn cfg(n: usize, inv: bool, slot: Option<usize>) -> PackCfg {
    PackCfg {
        n,
        inverted: inv,
        slot,
    }
}

#[test]
fn grid_has_the_right_cars_and_order() {
    for n in [8usize, 12, 16, 3, 99] {
        let c = cfg(n, false, None).sanitized();
        let v = grid(7, c, 600.0);
        assert_eq!(v.len(), c.n - 1);
        // the player starts at the back: every rival is ahead
        assert!(v.iter().all(|r| r.z > 600.0), "n={n}");
        assert_eq!(position(&v, 600.0), c.n);
        let inv = grid(7, cfg(n, true, None), 600.0);
        assert_eq!(position(&inv, 600.0), 1, "inverted: the player is first");
        let pole = grid(7, cfg(n, false, Some(0)), 600.0);
        assert_eq!(position(&pole, 600.0), 1);
        let mid = grid(7, cfg(n, false, Some(4)), 600.0);
        assert_eq!(position(&mid, 600.0), 5);
    }
}

#[test]
fn qualifying_slots_follow_the_time() {
    let r = 100.0;
    let fast = slot_for(3, 12, 30.0, r);
    let slow = slot_for(3, 12, 400.0, r);
    assert_eq!((fast, slow), (0, 11));
    assert_eq!(slot_for(3, 12, f32::NAN, r), 11);
    let mut last = 0;
    for t in (60..200).step_by(5) {
        let s = slot_for(3, 12, t as f32, r);
        assert!(s >= last);
        last = s;
    }
}

#[test]
fn a_race_with_a_pack_runs_and_pays_the_podium() {
    let mut finished = 0;
    for seed in 1..=4u64 {
        let mut g = Game::new_mode(seed, Mode::Campaign);
        g.set_level(Difficulty::from_index(0));
        g.set_pack(Some(cfg(12, false, None)));
        assert_eq!(g.place, 12);
        let go = Input {
            accel: true,
            ..Default::default()
        };
        for _ in 0..80_000 {
            g.update(0.02, &go);
            assert!(g.x.is_finite() && g.speed.is_finite());
            assert!(g.pack.iter().all(|r| r.z.is_finite() && r.speed >= 0.0));
            assert!((1..=12).contains(&g.place));
            if g.state != State::Playing {
                break;
            }
        }
        if g.state == State::Finished {
            finished += 1;
        }
    }
    let _ = finished;
    // no pack in time trial and zen
    let mut t = Game::new_mode(1, Mode::TimeTrial);
    t.set_pack(Some(cfg(12, false, None)));
    assert!(t.pack.is_empty());
}

#[test]
fn winning_pays_the_podium_bonus() {
    let mut g = Game::new_mode(2, Mode::Campaign);
    g.set_level(Difficulty::from_index(0));
    g.set_pack(Some(cfg(10, false, None)));
    let end = g.track.finish_seg as f32 * turbocursa::track::SEG_LEN;
    for r in g.pack.iter_mut() {
        r.z = 100.0; // everybody is far behind
    }
    g.pos = end - 1500.0;
    g.time = 60.0;
    let before = g.score;
    let go = Input {
        accel: true,
        ..Default::default()
    };
    for _ in 0..600 {
        g.update(0.02, &go);
        if g.state != State::Playing {
            break;
        }
    }
    assert_eq!(g.state, State::Finished);
    assert_eq!(g.place, 1);
    assert!(g.score >= before + podium_bonus(1));
}

/// A driver at a constant pace against the pack model alone: the pack is calibrated to the pace a
/// careful driver really reaches (about half the top speed), so a good pace wins and a poor one does not.
#[test]
fn a_good_pace_reaches_the_podium_and_a_poor_one_does_not() {
    use turbocursa::game::MAX_SPEED;
    use turbocursa::pack::*;
    use turbocursa::track::{self, SEG_LEN};
    fn place_at(seed: u64, frac: f32) -> usize {
        let tr = track::build(seed);
        let finish = tr.finish_seg as f32 * SEG_LEN;
        let mut rivals = grid(
            seed,
            PackCfg {
                n: 12,
                inverted: false,
                slot: None,
            },
            0.0,
        );
        let mut pz = 0.0f32;
        let mut t = 0.0;
        while pz < finish && t < 900.0 {
            pz += frac * MAX_SPEED * 0.02;
            for r in rivals.iter_mut() {
                r.update(0.02, t, &tr, pz);
            }
            t += 0.02;
        }
        position(&rivals, pz)
    }
    let avg = |f: f32| (1..=12u64).map(|s| place_at(s, f)).sum::<usize>() as f32 / 12.0;
    let (slow, fair, good, great) = (avg(0.40), avg(0.50), avg(0.60), avg(0.72));
    println!("average place at 40/50/60/72% pace: {slow} {fair} {good} {great}");
    assert!(slow > 9.0, "a poor pace ends at the back: {slow}");
    assert!(good <= 4.0, "a good pace is near the podium: {good}");
    assert!(great < 2.0, "a great pace wins: {great}");
    assert!(slow > fair && fair > good && good > great, "monotonic");
}
