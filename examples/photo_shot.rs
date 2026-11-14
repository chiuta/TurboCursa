//! Writes a photo-mode PNG: `cargo run --release --example photo_shot -- out.png`
use turbocursa::game::*;
fn main() {
    let out = std::env::args().nth(1).unwrap_or("photo.png".into());
    let mut g = Game::new_mode(11, Mode::Campaign);
    g.set_time(3);
    let go = Input {
        accel: true,
        ..Default::default()
    };
    for _ in 0..600 {
        let i = g.autopilot();
        g.update(1.0 / 60.0, &Input { accel: true, ..i });
        let _ = go;
    }
    g.photo_toggle();
    let up = Input {
        right: true,
        turbo: false,
        ..Default::default()
    };
    for _ in 0..40 {
        g.update(1.0 / 60.0, &up);
    }
    let hi = Input {
        accel: true,
        turbo: true,
        ..Default::default()
    };
    for _ in 0..25 {
        g.update(1.0 / 60.0, &hi);
    }
    let mut fb = vec![0u32; turbocursa::W * turbocursa::H];
    turbocursa::render::render(&g, &mut fb);
    std::fs::write(
        &out,
        turbocursa::png::encode(&fb, turbocursa::W, turbocursa::H),
    )
    .unwrap();
}
