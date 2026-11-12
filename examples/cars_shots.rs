//! Renders a sheet of country cars (frames/cars_sheet.ppm) and the garage screen (frames/cars_garage.ppm).
use turbocursa::career::*;
use turbocursa::career_ui::CareerApp;
use turbocursa::cars::CarSpec;
use turbocursa::game::Input;
use turbocursa::{render, H, W};

fn ppm(name: &str, fb: &[u32]) {
    let mut v = format!("P6\n{} {}\n255\n", W, H).into_bytes();
    for &c in fb {
        v.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
    }
    std::fs::write(format!("frames/{name}.ppm"), v).unwrap();
}

fn main() {
    let mut fb = vec![0x10163Au32; W * H];
    let codes = [
        "RO", "FR", "DE", "JP", "BR", "US", "EG", "AU", "IN", "CH", "AR", "ZA", "TR", "KR", "MX",
        "NG",
    ];
    for (n, code) in codes.iter().enumerate() {
        let i = COUNTRIES.iter().position(|c| c.code == *code).unwrap();
        let (cx, by) = (80.0 + (n % 4) as f32 * 160.0, 90.0 + (n / 4) as f32 * 100.0);
        render::draw_car_preview(&mut fb, W, H, cx, by, 100.0, &CarSpec::for_country(i));
    }
    ppm("cars_sheet", &fb);
    let none = Input::default();
    let down = Input {
        brake: true,
        ..none
    };
    let mut c = Career::new(0, 3);
    c.div = 1;
    let mut app = CareerApp::new(1, Some(&c.to_save()));
    for _ in 0..7 {
        app.update(0.016, &down, false, false);
        app.update(0.016, &none, false, false);
    }
    app.update(0.016, &none, true, false);
    app.render(&mut fb);
    ppm("cars_garage", &fb);
}
