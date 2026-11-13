//! Language tests live in their own binary: the language is a process-wide setting.
use turbocursa::game::*;
use turbocursa::versus::Versus;
use turbocursa::{render, H, W};

#[test]
fn every_language_renders_and_fits() {
    use turbocursa::font;
    use turbocursa::i18n::{self, Lang, LANGS, S};
    let keys = [
        S::Time,
        S::Kmh,
        S::Turbo,
        S::TimeUp,
        S::Score,
        S::PressEnter,
        S::StageClear,
        S::Passed,
        S::Winner,
        S::SecondPlace,
        S::Draw,
        S::Rematch,
        S::First,
        S::Second,
        S::Go,
        S::Checkpoint,
        S::StageClearBang,
    ];
    let mut fb = vec![0u32; W * H];
    for &(l, code, _) in LANGS.iter() {
        assert_eq!(Lang::parse(code), Some(l));
        for &k in &keys {
            let s = i18n::tr_in(l, k);
            assert!(!s.is_empty());
            // every character must be drawable (or a space)
            for ch in s
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '{' && *c != '}')
            {
                assert!(
                    font::glyph(ch).is_some(),
                    "{code}: no glyph for {ch:?} in {s:?}"
                );
            }
        }
        for m in 0..11 {
            let n = i18n::mode_name_in(l, m);
            for ch in n.chars().filter(|c| !c.is_whitespace()) {
                assert!(
                    font::glyph(ch).is_some(),
                    "{code}: mode name {n:?} has no glyph for {ch:?}"
                );
            }
            assert!(
                font::text_width(n, 2) + 200 < W as i32,
                "{code}: mode name too wide"
            );
        }
        // widest end-screen strings must fit the 640px view at their draw scale (4 for TIME UP, 3 for the rest, vk = 2)
        let w = |k: S, sc: i32| font::text_width(i18n::tr_in(l, k), sc * 2);
        assert!(w(S::TimeUp, 4) < W as i32, "{code}: TIME UP too wide");
        assert!(
            w(S::StageClear, 3) < W as i32,
            "{code}: STAGE CLEAR too wide"
        );
        assert!(
            w(S::Winner, 3) < W as i32
                && w(S::SecondPlace, 3) < W as i32
                && w(S::Draw, 3) < W as i32,
            "{code}: versus text too wide"
        );
        assert!(
            w(S::Checkpoint, 2) + 40 < W as i32,
            "{code}: banner too wide"
        );
        i18n::set_lang(l);
        let mut g = Game::new(1);
        for i in 0..200 {
            g.update(
                1.0 / 60.0,
                &Input {
                    accel: true,
                    ..Default::default()
                },
            );
            if i % 50 == 0 {
                render::render(&g, &mut fb);
            }
        }
        g.state = State::TimeUp;
        render::render(&g, &mut fb);
        g.state = State::Finished;
        render::render(&g, &mut fb);
        let mut v = Versus::new(2);
        v.p[0].state = State::Finished;
        v.winner = Some(0);
        v.frozen = true;
        let mut out = vec![0u32; W * H];
        v.render(&mut out);
    }
    i18n::set_lang(Lang::En);
}

#[test]
fn every_banner_in_game_code_is_translated_into_all_languages() {
    use turbocursa::i18n::{item_banner_in, Lang};
    let src = [
        include_str!("../src/game/mod.rs"),
        include_str!("../src/game/world.rs"),
        include_str!("../src/game/events.rs"),
        include_str!("../src/game/urban.rs"),
        include_str!("../src/game/effects.rs"),
    ]
    .join("\n");
    let mut keys = std::collections::BTreeSet::new();
    for l in src.lines() {
        if let Some(i) = l.find("banner = \"") {
            let rest = &l[i + 10..];
            if let Some(j) = rest.find('"') {
                keys.insert(rest[..j].to_string());
            }
        }
    }
    assert!(keys.len() > 30, "found only {} banners", keys.len());
    // words that read the same in every supported language
    let same_everywhere = ["TURBO +1", "TURBO", "ICE", "OK"];
    // translated through the string tables instead of the item table
    let in_tables = [
        "GO!",
        "CHECKPOINT",
        "STAGE CLEAR!",
        "LAP",
        "INVERT",
        "NORMAL",
        "RING",
    ];
    let mut missing = Vec::new();
    for k in &keys {
        if same_everywhere.contains(&k.as_str()) || in_tables.contains(&k.as_str()) {
            continue;
        }
        for lang in [
            Lang::Ro,
            Lang::Fr,
            Lang::It,
            Lang::Es,
            Lang::Pt,
            Lang::De,
            Lang::Hu,
        ] {
            if item_banner_in(lang, k).is_none_or(|v| v == k.as_str()) {
                missing.push(format!("{k} / {lang:?}"));
            }
        }
    }
    // genuine cognates: the same word is correct in that language
    let cognates = [
        "ACCIDENT / Ro",
        "ACCIDENT / Fr",
        "ANIMAL! / Ro",
        "PODIUM! / Ro",
        "ANIMAL! / Es",
        "ANIMAL! / Pt",
        "CRASH / De",
        "SERVICE / Ro",
        "SERVICE / Fr",
    ];
    missing.retain(|m| !cognates.contains(&m.as_str()));
    assert!(missing.is_empty(), "untranslated: {missing:?}");
}
