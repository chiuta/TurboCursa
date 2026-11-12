use turbocursa::champ::ChampCode;
use turbocursa::codes::*;
use turbocursa::ghost::Ghost;

#[test]
fn every_kind_of_code_is_recognised_and_hostile_text_is_not() {
    let champ = ChampCode::from_seed(12345, 1, 2, 1).encode();
    assert!(matches!(parse(&champ), Some(Code::Champ(_))), "{champ}");
    assert!(
        matches!(parse(&champ.to_lowercase()), Some(Code::Champ(_))),
        "lower case"
    );
    assert_eq!(parse(&champ).unwrap().kind(), 1);
    let g = Ghost {
        seed: 7,
        mode: 0,
        class: 0,
        secs: 10.0,
        score: 1,
        pts: vec![(0.0, 0.0), (500.0, 0.3)],
    };
    assert!(matches!(parse(&g.encode()), Some(Code::Ghost(_))));
    assert_eq!(parse(" goldrush ").unwrap(), Code::Paint(1));
    assert_eq!(parse("daily"), Some(Code::Daily));
    assert_eq!(parse("Weekly"), Some(Code::Weekly));
    assert!(parse(&champ).unwrap().launch_args(&champ).is_some());
    assert!(parse("GOLDRUSH").unwrap().launch_args("GOLDRUSH").is_none());
    for bad in [
        "",
        "   ",
        "hello",
        "TCC1-",
        "TCC1-ZZZZZ-ZZZZZ",
        "TCS1-",
        "TG2;",
        "\u{0}",
        "'; DROP TABLE",
    ] {
        assert!(parse(bad).is_none(), "{bad:?}");
    }
    assert!(parse(&"A".repeat(MAX_LEN + 1)).is_none());
}

#[test]
fn typing_a_code_on_the_options_page_submits_it_once() {
    use turbocursa::game::Input;
    use turbocursa::pick::*;
    let mut p = PickApp::new(0, 1);
    p.back(); // options page
    let down = Input {
        brake: true,
        ..Default::default()
    };
    for _ in 0..6 {
        p.update(0.02, &Input::default(), false);
        p.update(0.02, &down, false);
    }
    p.update(0.02, &Input::default(), true); // Enter on the code row: start typing
    assert!(p.is_typing());
    for c in "goldrush".chars() {
        p.type_char(c);
    }
    // letters and arrows must not move anything while typing
    p.update(
        0.02,
        &Input {
            left: true,
            accel: true,
            ..Default::default()
        },
        false,
    );
    p.back(); // erases the "h"
    p.type_char('h');
    p.type_char('\n'); // control characters are ignored
    p.update(0.02, &Input::default(), true);
    assert!(!p.is_typing());
    assert_eq!(p.take_code().as_deref(), Some("goldrush"));
    assert!(p.take_code().is_none(), "once");
    // an empty line cancels with Backspace
    p.update(0.02, &Input::default(), true);
    assert!(p.is_typing());
    p.back();
    assert!(!p.is_typing());
    assert_eq!(p.page, 2, "cancelling stays on the options page");
}
