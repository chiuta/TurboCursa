use turbocursa::game::Input;
use turbocursa::garage::Garage;
use turbocursa::launch::*;
use turbocursa::pick::*;

#[test]
fn split_screen_never_hides_the_career_the_rally_or_a_championship() {
    // (players, career, champ, rally, versus)
    assert_eq!(decide(4, true, false, false, false), Launch::Career);
    assert_eq!(decide(3, false, true, false, false), Launch::Champ);
    assert_eq!(decide(3, false, false, true, false), Launch::Rally);
    assert_eq!(decide(3, false, false, false, false), Launch::Multi);
    assert_eq!(decide(4, false, false, false, true), Launch::Multi);
    assert_eq!(decide(0, false, false, false, true), Launch::Versus);
    assert_eq!(decide(0, false, false, false, false), Launch::Single);
    // every combination: the three special screens always win over split screen
    for bits in 0..16u8 {
        let (c, ch, r, v) = (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let k = decide(4, c, ch, r, v);
        if c || ch || r {
            assert!(
                !matches!(k, Launch::Multi | Launch::Versus | Launch::Single),
                "{bits}"
            );
        }
    }
}

fn tap(p: &mut PickApp, inp: Input, enter: bool) {
    p.update(0.02, &Input::default(), false);
    p.update(0.02, &inp, enter);
}

#[test]
fn extras_are_honest_about_the_platform_and_the_stock_modes() {
    let down = Input {
        brake: true,
        ..Default::default()
    };
    let right = Input {
        right: true,
        ..Default::default()
    };
    let mut p = PickApp::new(0, 1).with_extras(Extras {
        web: true,
        stock: true,
        garage: Garage {
            coins: 500,
            ..Default::default()
        },
        ..Default::default()
    });
    p.back();
    tap(&mut p, down, false);
    tap(&mut p, down, false); // row 2: keys (no layouts in the browser)
    tap(&mut p, right, false);
    assert!(p.take_extras().is_none(), "the key layout is native only");
    for _ in 0..5 {
        tap(&mut p, down, false); // rows 3..7 (7 = the engine)
    }
    tap(&mut p, Input::default(), true); // buy the engine
    assert!(p.take_extras().is_none(), "no upgrades in a stock-car mode");
}

#[test]
fn rally_coins_add_up_and_never_overflow() {
    let mut r = turbocursa::rally::RallyApp::new(1, 0, 1);
    r.add_coins(40);
    r.add_coins(u32::MAX);
    assert_eq!(r.coins_stage, u32::MAX);
    assert_eq!(r.coins_total, 9_999_999);
}
