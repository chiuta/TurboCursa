//! Stage codes and the designer through the browser API (`--features web`).
#![cfg(feature = "web")]
use turbocursa::share::StageCode;
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

#[test]
fn stage_codes_and_designer_work_through_the_api() {
    init(1, 0);
    let code = StageCode {
        seed: 4242,
        mode: 1,
        class: 11,
        level: 3,
        weather: 3,
        surface: 2,
        time: 4,
        ..StageCode::default()
    };
    let txt = code.encode();
    assert_eq!(code_load(put(&txt)), 1);
    assert_eq!(code_seed(), 4242);
    assert_eq!(code_mode(), 1);
    init(code_seed(), code_mode());
    for _ in 0..200 {
        set_input(4);
        frame(0.016);
    }
    let n = code_text();
    assert!(n > 0);
    assert_eq!(read_buf(n), txt);
    // hostile / corrupted codes are refused and change nothing
    for bad in [
        "",
        "TCS1-",
        "TCS1-00000-00000-00000-00000-00000-00000",
        &txt.replace('A', "B").replace('4', "5"),
        "zzzz",
        &"9".repeat(500),
    ] {
        let ok = code_load(put(bad));
        assert_eq!(ok, 0, "{bad:?}");
    }
    // a normal init forgets the code
    init(5, 0);
    assert_eq!(code_text(), 0);
    // designer mode: Enter plays the designed stage
    init(9, 13);
    for _ in 0..20 {
        set_input(0);
        frame(0.016);
    }
    set_input(32);
    frame(0.016);
    set_input(0);
    for _ in 0..30 {
        set_input(4);
        frame(0.016);
    }
    assert!(code_text() > 0);
    assert_eq!(state_code(), 0);
}
