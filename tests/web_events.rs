//! Championship codes, weekly challenge, garage, split screen and options through the browser API.
#![cfg(feature = "web")]
use turbocursa::champ::ChampCode;
use turbocursa::web::*;

fn put(txt: &str) -> u32 {
    let p = buf_ptr(txt.len() as u32);
    unsafe { std::ptr::copy_nonoverlapping(txt.as_ptr(), p, txt.len()) };
    txt.len() as u32
}

fn read_buf(n: u32) -> String {
    let p = buf_ptr(n);
    unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(p, n as usize)).to_string() }
}

// the web state is process-wide, so everything runs in one test
#[test]
fn events_through_the_api() {
    init(1, 0);
    // a championship code is accepted by the same box as stage codes and plays in mode 12
    let c = ChampCode::from_seed(77, 1, 2, 1);
    assert_eq!(code_load(put(&c.encode())), 1);
    assert_eq!(code_mode(), 12);
    assert_eq!(code_seed(), c.seed);
    init(code_seed(), code_mode());
    for _ in 0..120 {
        set_input(0);
        frame(0.016);
    }
    let n = code_text();
    assert_eq!(read_buf(n), c.encode());
    assert_eq!(code_load(put("TCC1-00000-00000-00000")), 0);
    // a stage code afterwards replaces the championship
    let s = turbocursa::share::StageCode::default().encode();
    assert_eq!(code_load(put(&s)), 1);
    assert_ne!(code_mode(), 12);

    // weekly challenge: stable, valid
    let m = weekly_setup(20_000);
    assert!(m <= 8);
    assert_eq!(weekly_seed(20_000), weekly_seed(20_001));
    assert_ne!(weekly_seed(20_000), weekly_seed(20_007));
    assert!(weekly_modifier(20_000) < 6);
    init(weekly_seed(20_000), m);
    for _ in 0..60 {
        frame(0.016);
    }

    // options
    set_music(2);
    set_music(99);
    set_mph(1);
    set_mph(0);
    assert_eq!(secret_load(put("not a code")), 0);
    assert_eq!(secret_load(put("gold rush")), 1);

    // garage: no coins -> no purchase; coins load from text
    assert_eq!(garage_load(put("TG2;c=500;u=0000")), 500);
    assert_eq!(garage_buy(0), 1);
    assert_eq!(garage_info(0), 460);
    assert_eq!(garage_info(1), 1);
    assert!(garage_save() > 0);
    assert_eq!(garage_buy(9), 0);
    assert_eq!(garage_load(put("junk")), 0);

    // split screen for 3 and 4, pack of rivals
    set_players(4);
    init(3, 10);
    for _ in 0..200 {
        set_input(1);
        frame(0.016);
    }
    set_players(0);
    set_pack(12, 0);
    init(4, 0);
    for _ in 0..200 {
        set_input(1);
        frame(0.016);
    }
    set_pack(0, 0);
}
