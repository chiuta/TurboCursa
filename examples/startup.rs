use std::time::Instant;
use turbocursa::{render, H, W};
fn main() {
    let t = Instant::now();
    let g = turbocursa::game::Game::new(1);
    let a = t.elapsed().as_secs_f64() * 1000.0;
    let mut fb = vec![0u32; W * H];
    render::render(&g, &mut fb);
    let b = t.elapsed().as_secs_f64() * 1000.0;
    render::render(&g, &mut fb);
    println!(
        "Game::new {a:.0} ms, first frame done at {b:.0} ms, second frame {:.1} ms",
        t.elapsed().as_secs_f64() * 1000.0 - b
    );
}
