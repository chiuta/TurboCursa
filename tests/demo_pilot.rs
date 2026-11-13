use turbocursa::ace::{Ace, Demo};
use turbocursa::game::{Mode, State};

fn run(idx: usize, frames: usize, assist: bool) -> turbocursa::game::Game {
    let mut g = Demo::list()[idx].game();
    g.demo = assist;
    let mut ace = Ace::new();
    for _ in 0..frames {
        let i = ace.input(&g);
        g.update(1.0 / 60.0, &i);
        if g.state != State::Playing {
            break;
        }
    }
    g
}

#[test]
fn playlist_covers_every_finite_mode_and_level() {
    let l = Demo::list();
    assert_eq!(l.len(), 40);
    for level in 0..5u8 {
        for mode in 0..8u8 {
            assert!(
                l.iter().any(|c| c.mode == mode && c.level == level),
                "mode {mode} level {level}"
            );
        }
    }
    assert!(
        !l.iter().any(|c| c.mode == 8),
        "Zen has no goal, so it is not in the walkthrough"
    );
}

#[test]
fn a_normal_game_has_no_demo_assist() {
    let g = Demo::list()[0].game();
    assert!(!g.demo);
    // Demo::current() is the one that switches the assist on
    assert!(Demo::new().current().demo);
}

#[test]
fn pilot_is_deterministic_and_drives() {
    let a = run(3, 900, false);
    let b = run(3, 900, false);
    assert_eq!(a.pos.to_bits(), b.pos.to_bits());
    assert_eq!(a.crashes, b.crashes);
    assert!(
        a.pos > 20_000.0,
        "the pilot should have covered ground in 15 s, got {}",
        a.pos
    );
}

#[test]
fn demo_assist_keeps_the_car_running() {
    // the assist: shielded, no fuel use, the clock cannot run out
    let g = run(4, 1200, true);
    assert!(g.fuel > 0.99);
    assert!(g.time >= 2.0 || g.state == State::Finished);
    assert_ne!(g.state, State::TimeUp);
}

#[test]
fn checkpoint_banner_shows_the_real_bonus() {
    let mut g = Demo::list()[0].game();
    g.banner = "CHECKPOINT";
    g.banner_bonus = 57;
    assert!(g.banner_text().ends_with("+57"), "{}", g.banner_text());
    g.banner = "GO!";
    assert!(!g.banner_text().contains('+'));
}

/// The whole walkthrough, one stage per mode on the hardest and easiest level (about a minute).
#[test]
#[ignore = "slow: plays complete stages"]
fn pilot_finishes_whole_stages() {
    for (i, c) in Demo::list().iter().enumerate() {
        if c.level != 0 && c.level != 4 {
            continue;
        }
        let g = run(i, 300 * 60, true);
        let lap = g.mode == Mode::Endless && g.lap > 0;
        assert!(
            g.state == State::Finished || lap || g.mode == Mode::Endless,
            "stage {i} ended {:?}",
            g.state
        );
    }
}
