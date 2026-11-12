use turbocursa::game::*;
use turbocursa::multi::*;
use turbocursa::{H, W};

#[test]
fn three_and_four_players_with_cpu_fill() {
    for (players, humans) in [(3usize, 1usize), (3, 3), (4, 2), (4, 4), (9, 0)] {
        let mut m = Multi::new(5, players, humans);
        assert_eq!(m.p.len(), players.clamp(3, 4));
        assert!(m.humans >= 1 && m.humans <= m.p.len());
        let mut fb = vec![0u32; W * H];
        let go = Input {
            accel: true,
            ..Default::default()
        };
        let inputs = [go, Input::default(), Input::default(), Input::default()];
        for _ in 0..1500 {
            m.update(0.02, &inputs);
        }
        m.render(&mut fb);
        assert!(fb.iter().any(|&c| c != 0x0A0A14), "something was drawn");
        let mut order = m.order();
        order.sort();
        assert_eq!(order, (0..m.p.len()).collect::<Vec<_>>());
        for i in 0..m.p.len() {
            assert!((1..=m.p.len()).contains(&m.place(i)));
            assert!(m.p[i].x.is_finite() && m.p[i].speed.is_finite());
        }
        // the autopilot cars really drive
        if m.humans < m.p.len() {
            assert!(m.p[m.p.len() - 1].pos > 1000.0, "the CPU car must move");
        }
    }
}

#[test]
fn a_winner_ends_the_round() {
    let mut m = Multi::new(2, 4, 1);
    let end = m.p[0].track.finish_seg as f32 * turbocursa::track::SEG_LEN;
    m.p[2].pos = end - 800.0;
    let none = Input::default();
    for _ in 0..800 {
        m.update(0.02, &[none; 4]);
        if m.winner.is_some() {
            break;
        }
    }
    assert!(m.winner.is_some());
    assert_eq!(m.place(m.winner.unwrap()), 1);
}
