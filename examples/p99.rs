use std::time::Instant;
use turbocursa::{game::*, render, versus::Versus, H, W};
fn pct(v: &mut Vec<f64>, p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() as f64 - 1.0) * p) as usize]
}
fn main() {
    let t0 = Instant::now();
    let mut g = Game::new(11);
    println!(
        "Game::new (incl. assets): {:.0} ms",
        t0.elapsed().as_secs_f64() * 1e3
    );
    let mut fb = vec![0u32; W * H];
    let mut v = Vec::new();
    for i in 0..1800 {
        let t = Instant::now();
        g.update(
            1.0 / 60.0,
            &Input {
                accel: true,
                turbo: i % 300 < 60,
                left: (i / 90) % 2 == 0,
                ..Default::default()
            },
        );
        render::render(&g, &mut fb);
        v.push(t.elapsed().as_secs_f64() * 1e3);
    }
    println!(
        "single  p50 {:.1} p99 {:.1} max {:.1} ms",
        pct(&mut v.clone(), 0.5),
        pct(&mut v.clone(), 0.99),
        v.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    );
    let t0 = Instant::now();
    let mut vs = Versus::new(12);
    println!("Versus::new: {:.0} ms", t0.elapsed().as_secs_f64() * 1e3);
    let mut out = vec![0u32; W * H];
    let mut v = Vec::new();
    for _ in 0..1200 {
        let t = Instant::now();
        let a = Input {
            accel: true,
            ..Default::default()
        };
        vs.update(1.0 / 60.0, &[a, a]);
        vs.render(&mut out);
        v.push(t.elapsed().as_secs_f64() * 1e3);
    }
    println!(
        "versus  p50 {:.1} p99 {:.1} max {:.1} ms",
        pct(&mut v.clone(), 0.5),
        pct(&mut v.clone(), 0.99),
        v.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    );
}
