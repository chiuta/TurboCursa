//! frames/hub_levels.ppm (hub with Difficulty/Vehicle rows) and frames/garage_semi.ppm
use turbocursa::career::*;
use turbocursa::career_ui::CareerApp;
use turbocursa::game::Input;
use turbocursa::{H, W};

fn ppm(name: &str, fb: &[u32]) {
    let mut v = format!("P6\n{} {}\n255\n", W, H).into_bytes();
    for &c in fb {
        v.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
    }
    std::fs::write(format!("frames/{name}.ppm"), v).unwrap();
}

fn main() {
    let none = Input::default();
    let down = Input {
        brake: true,
        ..none
    };
    let right = Input {
        right: true,
        ..none
    };
    let mut c = Career::new(0, 3);
    c.div = 1;
    let mut app = CareerApp::new(1, Some(&c.to_save()));
    let tap = |app: &mut CareerApp, i: &Input| {
        app.update(0.016, i, false, false);
        app.update(0.016, &none, false, false);
    };
    for _ in 0..10 {
        tap(&mut app, &down);
    }
    tap(&mut app, &right);
    tap(&mut app, &right);
    tap(&mut app, &down);
    for _ in 0..3 {
        tap(&mut app, &right);
    }
    let mut fb = vec![0u32; W * H];
    app.render(&mut fb);
    ppm("hub_levels", &fb);
}
