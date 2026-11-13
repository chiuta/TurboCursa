use turbocursa::game::{Game, Input, Mode, State};
use turbocursa::i18n::{self, Lang, S};

fn driving() -> Game {
    let mut g = Game::new_mode(7, Mode::Campaign);
    let go = Input {
        accel: true,
        ..Input::default()
    };
    for _ in 0..240 {
        g.update(1.0 / 60.0, &go);
    }
    g
}

#[test]
fn pause_freezes_the_stage_and_resume_continues() {
    let mut g = driving();
    assert_eq!(g.state, State::Playing);
    let go = Input {
        accel: true,
        ..Input::default()
    };
    g.toggle_pause();
    assert!(g.paused);
    let (t, z, v) = (g.elapsed, g.pos, g.speed);
    for _ in 0..600 {
        g.update(1.0 / 60.0, &go);
    }
    assert_eq!(
        (g.elapsed, g.pos, g.speed),
        (t, z, v),
        "nothing moves while paused"
    );
    g.toggle_pause();
    assert!(!g.paused);
    for _ in 0..60 {
        g.update(1.0 / 60.0, &go);
    }
    assert!(
        g.elapsed > t && g.pos > z,
        "the stage runs again after resume"
    );
}

#[test]
fn pause_is_ignored_outside_a_running_stage() {
    let mut g = Game::new_mode(3, Mode::TimeTrial);
    g.state = State::Finished;
    g.toggle_pause();
    assert!(!g.paused, "a finished stage cannot be paused");
}

#[test]
fn photo_mode_hands_over_to_pause() {
    let mut g = driving();
    g.photo_toggle();
    assert!(g.photo);
    g.toggle_pause();
    assert!(g.paused && !g.photo);
}

#[test]
fn pause_overlay_is_drawn_and_translated() {
    let mut g = driving();
    let mut a = vec![0u32; turbocursa::W * turbocursa::H];
    turbocursa::render::render(&g, &mut a);
    g.toggle_pause();
    let mut b = vec![0u32; turbocursa::W * turbocursa::H];
    turbocursa::render::render(&g, &mut b);
    assert_ne!(a, b);
    let lit = |f: &[u32]| f.iter().map(|&p| (p >> 16 & 255) as u64).sum::<u64>();
    assert!(lit(&b) < lit(&a), "the paused picture is dimmed");
    let en = i18n::tr_in(Lang::En, S::Paused);
    for i in 0..8u32 {
        let l = Lang::from_index(i);
        assert!(!i18n::tr_in(l, S::Paused).is_empty());
        assert!(!i18n::tr_in(l, S::PauseHint).is_empty());
        if i > 0 {
            assert_ne!(
                i18n::tr_in(l, S::PauseHint),
                i18n::tr_in(Lang::En, S::PauseHint)
            );
        }
    }
    assert_eq!(en, "PAUSED");
}

#[test]
fn calm_mode_removes_the_screen_shake() {
    let mut g = driving();
    let draw = |g: &Game| {
        let mut f = vec![0u32; turbocursa::W * turbocursa::H];
        turbocursa::render::render(g, &mut f);
        f
    };
    let still = draw(&g);
    g.shake = 1.0;
    turbocursa::access::set_calm(false);
    let shaken = draw(&g);
    turbocursa::access::set_calm(true);
    let calm = draw(&g);
    turbocursa::access::set_calm(false);
    assert_ne!(still, shaken, "the shake moves the picture");
    assert_eq!(still, calm, "calm mode draws the same picture as no shake");
}
