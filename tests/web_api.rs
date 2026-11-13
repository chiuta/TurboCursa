//! Fuzzes the C-ABI surface the browser calls (compiled natively with `--features web`).
#![cfg(feature = "web")]
use turbocursa::web::*;

/// The API keeps one global app, so the tests must not interleave.
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
}

#[test]
fn browser_api_survives_hostile_calls() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // class / level selectors clamp hostile values and the game still runs
    for (c, l) in [(0u32, 0u32), (4, 4), (u32::MAX, u32::MAX), (2, 9)] {
        set_class(c);
        set_level(l);
        for mode in [0u32, 3, 9, 10] {
            init(7, mode);
            set_input(4);
            for _ in 0..120 {
                let p = frame(0.016);
                assert!(!p.is_null());
            }
        }
    }
    // selection screen: shown first, Enter starts the chosen game
    set_pick(1);
    for mode in [0u32, 9, 10] {
        init(3, mode);
        assert_eq!(state_code(), 0);
        set_input(2); // right: next vehicle
        frame(0.016);
        set_input(0);
        frame(0.016);
        set_input(32); // Enter
        frame(0.016);
        set_input(0);
        for _ in 0..30 {
            assert!(!frame(0.016).is_null());
        }
        set_input(64); // back to the selection screen
        frame(0.016);
        set_input(0);
        frame(0.016);
    }
    set_pick(0);
    set_class(0);
    set_level(1);
    let mut r = Lcg(5);
    set_lang(9999); // out of range must clamp, not panic
    set_lang(0);
    assert_eq!((width(), height()), (640, 400));
    for versus in 0..=11u32 {
        init(r.next(), versus);
        // audio before audio_init, then hostile sample rates and sizes
        let _ = audio_fill(256);
        for sr in [0.0f32, f32::NAN, -1.0, 1.0, 44100.0, 1e9] {
            audio_init(sr);
            for n in [0u32, 1, 3, 2048, 1 << 20, u32::MAX] {
                let p = audio_fill(n);
                assert!(!p.is_null());
                let m = (n as usize).min(1 << 16);
                let s = unsafe { std::slice::from_raw_parts(p, m) };
                assert!(
                    s.iter().all(|x| x.is_finite() && x.abs() <= 1.0),
                    "bad samples sr={sr} n={n}"
                );
            }
        }
        let mut finished_seen = false;
        for i in 0..1500 {
            set_input(r.next() & if i % 7 == 0 { u32::MAX } else { 0x3F3F });
            let dt =
                [0.016f32, 0.0, -1.0, f32::NAN, f32::INFINITY, 5.0, 1e-9][(r.next() % 7) as usize];
            let p = frame(dt);
            assert!(!p.is_null());
            let px = unsafe { std::slice::from_raw_parts(p, 640 * 400 * 4) };
            assert!(
                px.chunks(4).step_by(997).all(|c| c[3] == 255),
                "alpha must stay opaque"
            );
            let st = state_code();
            assert!(st <= 2);
            finished_seen |= st == 2;
            if i % 500 == 0 {
                set_lang(r.next());
            }
            let _ = score();
        }
        let _ = finished_seen;
        // restart is one-shot: after one Enter press the next frames must not keep restarting
        init(1, versus);
        set_input(32 | 8192);
        frame(0.016);
        let s1 = score();
        frame(0.016);
        assert!(score() >= s1);
    }
    set_lang(0);
}

#[test]
fn career_save_and_load_through_the_browser_api() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    init(3, 11);
    // pick the first country (Enter), which creates the career and a save
    frame(0.016);
    set_input(32);
    frame(0.016);
    set_input(0);
    frame(0.016);
    let n = career_save();
    assert!(n > 10, "a fresh career must be saved");
    let p = buf_ptr(n);
    let txt = unsafe { std::slice::from_raw_parts(p, n as usize) }.to_vec();
    assert!(txt.starts_with(b"TC1;"));
    assert_eq!(career_save(), 0, "nothing changed since the last save");
    // load it back in a new session; hostile buffers are rejected
    init(9, 11);
    let p = buf_ptr(txt.len() as u32);
    unsafe { std::slice::from_raw_parts_mut(p, txt.len()) }.copy_from_slice(&txt);
    assert_eq!(career_load(txt.len() as u32), 1);
    let junk: &[u8] = b"\xff\xfeTC1;;;;=====";
    let p = buf_ptr(junk.len() as u32);
    unsafe { std::slice::from_raw_parts_mut(p, junk.len()) }.copy_from_slice(junk);
    assert_eq!(career_load(junk.len() as u32), 0);
    assert_eq!(career_load(0), 0);
    // not in career mode: saving/loading is a no-op
    init(1, 0);
    assert_eq!(career_save(), 0);
    assert_eq!(career_load(4), 0);
}

#[test]
fn browser_pause_freezes_and_restarts() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    set_pick(0);
    init(11, 0);
    set_input(4);
    for _ in 0..120 {
        frame(0.016);
    }
    assert_eq!(pause_ctl(2), 0);
    assert_eq!(pause_ctl(1), 1, "losing focus pauses");
    assert_eq!(pause_ctl(1), 1, "pausing twice stays paused");
    let (t, s) = (state_code(), score());
    for _ in 0..120 {
        frame(0.016);
    }
    assert_eq!((state_code(), score()), (t, s));
    assert_eq!(pause_ctl(0), 0, "toggle resumes");
    assert_eq!(pause_ctl(0), 1, "toggle pauses");
    assert_eq!(pause_ctl(7), 1, "unknown commands only report");
    set_input(32); // Enter while paused restarts the stage
    frame(0.016);
    set_input(0);
    assert_eq!(pause_ctl(2), 0, "a restarted stage is running");
    for v in [0u32, 1, 2, 3, u32::MAX] {
        assert!(pause_ctl(v) <= 1);
    }
}
