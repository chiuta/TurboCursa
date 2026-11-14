//! Designer screenshot: `cargo run --release --example design_shot -- out.png`
use turbocursa::design::DesignApp;
use turbocursa::game::Input;
fn main() {
    let out = std::env::args().nth(1).unwrap_or("design.png".into());
    let mut d = DesignApp::new(None);
    let (idle, right, down) = (
        Input::default(),
        Input {
            right: true,
            ..Input::default()
        },
        Input {
            brake: true,
            ..Input::default()
        },
    );
    let step = |d: &mut DesignApp, i: &Input| {
        d.update(0.016, i, false, false);
        d.update(0.016, &idle, false, false);
    };
    for _ in 0..40 {
        step(&mut d, &right);
    }
    for row in 1..13 {
        for _ in 0..row {
            step(&mut d, &down);
        }
        step(&mut d, &right);
        step(&mut d, &right);
        for _ in 0..(14 - row) {
            step(&mut d, &down);
        }
    }
    for _ in 0..7 {
        step(&mut d, &down);
    }
    for _ in 0..3 {
        step(&mut d, &right);
    }
    let mut fb = vec![0u32; turbocursa::W * turbocursa::H];
    d.render(&mut fb);
    std::fs::write(
        &out,
        turbocursa::png::encode(&fb, turbocursa::W, turbocursa::H),
    )
    .unwrap();
}
