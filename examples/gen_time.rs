use std::time::Instant;
fn main() {
    let t = Instant::now();
    let g = turbocursa::game::Game::new(1);
    println!(
        "first Game::new (track + assets, threaded): {:.0} ms, {} segments",
        t.elapsed().as_secs_f64() * 1000.0,
        g.track.segs.len()
    );
    let t = Instant::now();
    let g2 = turbocursa::game::Game::new(2);
    println!(
        "retry Game::new (cached assets): {:.0} ms, {} segments",
        t.elapsed().as_secs_f64() * 1000.0,
        g2.track.segs.len()
    );
}
