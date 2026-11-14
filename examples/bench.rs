use std::time::Instant;
use turbocursa::fx::{post, FxParams};
use turbocursa::game::*;
use turbocursa::{render, H, W};
fn main() {
    let mut g = Game::new(1);
    let mut fb = vec![0u32; W * H];
    let inp = Input {
        accel: true,
        ..Default::default()
    };
    for _ in 0..300 {
        g.update(1.0 / 60.0, &inp);
    }
    let n = 120;
    let t = Instant::now();
    for _ in 0..n {
        g.update(1.0 / 60.0, &inp);
        render::render(&g, &mut fb);
    }
    let total = t.elapsed().as_secs_f64() * 1000.0 / n as f64;
    let p = FxParams {
        w: W,
        h: H,
        speed: g.speed / MAX_SPEED,
        turbo: false,
        shake: 0.0,
        biome_w: g.biome_w,
        t: 1.0,
        haze: 0xbbddff,
    };
    let t2 = Instant::now();
    for _ in 0..n {
        post(&mut fb, &p);
    }
    let postms = t2.elapsed().as_secs_f64() * 1000.0 / n as f64;
    println!(
        "total {:.2} ms/frame, post {:.2} ms, scene+hud {:.2} ms (speed {:.2})",
        total,
        postms,
        total - postms,
        p.speed
    );
}
