//! Renders the career screens to frames/career_*.ppm (convert with any image tool).
use turbocursa::career::*;
use turbocursa::career_ui::CareerApp;
use turbocursa::game::Input;
use turbocursa::{H, W};

fn ppm(name: &str, fb: &[u32]) {
    let mut v = format!("P6\n{} {}\n255\n", W, H).into_bytes();
    for &c in fb {
        v.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
    }
    std::fs::write(format!("frames/career_{name}.ppm"), v).unwrap();
}

fn main() {
    if let Some(l) = std::env::args().nth(1) {
        turbocursa::i18n::set_lang(turbocursa::i18n::Lang::parse(&l).unwrap());
    }
    let none = Input::default();
    let down = Input {
        brake: true,
        ..none
    };
    let right = Input {
        right: true,
        ..none
    };
    let gas = Input {
        accel: true,
        ..none
    };
    let mut fb = vec![0u32; W * H];
    let mut c = Career::new(0, 11);
    c.div = 2;
    c.season = 3;
    c.rating = 1480.0;
    c.trophies.national[0] = 2;
    for i in [1, 4, 7, 9, 18, 19, 38, 41] {
        c.trophies.national[i] = 1;
    }
    c.trophies.cont[0] = 1;
    c.q = [true, false, false];
    c.stats = Stats {
        races: 41,
        wins: 9,
        podiums: 17,
    };
    c.pts[0] = 22;
    c.day = 2;
    let mut app = CareerApp::new(1, None);
    app.render(&mut fb);
    ppm("pick", &fb);
    let mut app = CareerApp::new(1, Some(&c.to_save()));
    let step = |a: &mut CareerApp, i: Input, e: bool, b: bool, fb: &mut Vec<u32>| {
        a.update(0.016, &i, e, b);
        a.update(0.016, &none, false, false);
        a.render(fb);
    };
    app.render(&mut fb);
    ppm("hub", &fb);
    step(&mut app, down, false, false, &mut fb);
    step(&mut app, none, true, false, &mut fb);
    ppm("cup", &fb);
    // race the national cup round with the throttle held, to get a result table
    step(&mut app, none, true, false, &mut fb);
    for _ in 0..12_000 {
        app.update(0.016, &gas, false, false);
    }
    for _ in 0..90 {
        app.update(0.016, &none, false, false);
    }
    app.update(0.016, &none, true, false);
    app.render(&mut fb);
    ppm("result", &fb);
    app.update(0.016, &none, true, false);
    app.update(0.016, &down, false, false);
    app.update(0.016, &none, false, false);
    app.update(0.016, &down, false, false);
    app.update(0.016, &none, false, true);
    // from the hub: rankings tabs, trophies
    for _ in 0..5 {
        step(&mut app, down, false, false, &mut fb);
    }
    step(&mut app, none, true, false, &mut fb);
    ppm("rank_drivers", &fb);
    step(&mut app, right, false, false, &mut fb);
    ppm("rank_nations", &fb);
    step(&mut app, right, false, false, &mut fb);
    ppm("rank_league", &fb);
    step(&mut app, none, true, false, &mut fb);
    step(&mut app, down, false, false, &mut fb);
    step(&mut app, none, true, false, &mut fb);
    ppm("trophies", &fb);
}
