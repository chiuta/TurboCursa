use std::time::Instant;
use turbocursa::{game::*, render, H, W};
fn main() {
    let mut g = Game::new_mode(4, Mode::Campaign);
    g.shield_t = 1000.0;
    let z = g.city.zones[0];
    let mut fb = vec![0u32; W * H];
    let mut v = Vec::new();
    for i in 0..300 {
        g.pos = (z.a as f32 + 10.0 + (i % 200) as f32) * 200.0 - PLAYER_Z;
        g.speed = 5000.0;
        g.state = State::Playing;
        g.update(0.016, &Input::default());
        let t = Instant::now();
        render::render(&g, &mut fb);
        v.push(t.elapsed().as_secs_f64() * 1e3);
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "city render p50 {:.1} p99 {:.1} max {:.1} ms",
        v[150], v[296], v[299]
    );
}
