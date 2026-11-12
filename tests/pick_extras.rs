use turbocursa::game::Input;
use turbocursa::garage::Garage;
use turbocursa::pick::*;

fn press(p: &mut PickApp, inp: Input, enter: bool) {
    p.update(0.02, &Input::default(), false); // release
    p.update(0.02, &inp, enter);
}

#[test]
fn the_options_page_changes_buys_and_reports_once() {
    let mut p = PickApp::new(0, 1).with_extras(Extras {
        garage: Garage {
            coins: 100,
            ..Default::default()
        },
        ..Default::default()
    });
    assert_eq!(p.page, 0);
    p.back();
    assert_eq!(p.page, 2, "Backspace on the vehicle page opens the options");
    let right = Input {
        right: true,
        ..Default::default()
    };
    let down = Input {
        brake: true,
        ..Default::default()
    };
    press(&mut p, right, false); // row 0: music
    let e = p.take_extras().expect("changed");
    assert_eq!(e.music, 1);
    assert!(p.take_extras().is_none(), "reported once");
    press(&mut p, down, false); // row 1: speed unit
    press(&mut p, right, false);
    assert!(p.take_extras().unwrap().mph);
    for _ in 0..2 {
        press(&mut p, down, false);
    }
    press(&mut p, right, false); // row 3: pack off -> 8
    assert_eq!(p.take_extras().unwrap().pack, 8);
    for _ in 0..2 {
        press(&mut p, down, false);
    }
    press(&mut p, right, false); // row 5: players 3
    assert_eq!(p.take_extras().unwrap().players, 3);
    press(&mut p, down, false); // row 6: the code field
    press(&mut p, down, false); // row 7: engine
    press(&mut p, Input::default(), true); // buy
    let e = p.take_extras().unwrap();
    assert_eq!((e.garage.lv[0], e.garage.coins), (1, 60));
    // too expensive: the second level costs 110, nothing changes
    press(&mut p, Input::default(), true);
    assert!(p.take_extras().is_none());
    // Backspace returns to the vehicle page and the start still works
    p.back();
    assert_eq!(p.page, 0);
    let mut buf = vec![0u32; turbocursa::W * turbocursa::H];
    p.back();
    p.render(&mut buf);
    assert!(buf.iter().any(|&c| c != 0));
    // hostile values are clamped
    let hostile = Extras {
        music: 99,
        keys: 99,
        pack: 200,
        players: 77,
        ..Default::default()
    }
    .sanitized();
    assert_eq!(
        (hostile.music, hostile.keys, hostile.pack, hostile.players),
        (3, 3, 16, 4)
    );
}
