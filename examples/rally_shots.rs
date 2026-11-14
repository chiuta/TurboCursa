//! Rally screens montage: `cargo run --release --example rally_shots -- out.png`
use turbocursa::game::*;
use turbocursa::rally::*;
fn main() {
    let out = std::env::args().nth(1).unwrap_or("rally.png".into());
    let (w, h) = (turbocursa::W, turbocursa::H);
    let mut app = RallyApp::new(4242, 12, 1);
    let none = Input::default();
    let mut shots: Vec<Vec<u32>> = Vec::new();
    let snap = |app: &mut RallyApp, shots: &mut Vec<Vec<u32>>| {
        let mut fb = vec![0u32; w * h];
        app.render(&mut fb);
        shots.push(fb);
    };
    snap(&mut app, &mut shots); // briefing
    app.update(0.016, &none, true, false);
    for _ in 0..900 {
        let g = app.game.as_ref().unwrap();
        let i = g.autopilot();
        app.update(0.016, &Input { accel: true, ..i }, false, false);
    }
    snap(&mut app, &mut shots); // racing
    for _ in 0..40000 {
        let g = app.game.as_ref().unwrap();
        if g.state != State::Playing {
            break;
        }
        let i = g.autopilot();
        app.update(0.016, &Input { accel: true, ..i }, false, false);
    }
    app.update(0.016, &none, true, false); // end stage
    snap(&mut app, &mut shots); // stage end
    app.update(0.016, &none, true, false);
    app.svc = Service {
        repair: 1,
        tyre: 0,
        refuel: true,
    };
    snap(&mut app, &mut shots); // service
    let mut out_fb = vec![0u32; w * 2 * h * 2];
    for (k, s) in shots.iter().enumerate() {
        let (ox, oy) = ((k % 2) * w, (k / 2) * h);
        for y in 0..h {
            for x in 0..w {
                out_fb[(oy + y) * w * 2 + ox + x] = s[y * w + x];
            }
        }
    }
    std::fs::write(&out, turbocursa::png::encode(&out_fb, w * 2, h * 2)).unwrap();
}
