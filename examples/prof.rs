use turbocursa::game::*;
use turbocursa::share::StageCode;
use turbocursa::{render, H, W};
fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let c = StageCode {
        weather: 2,
        time: 4,
        seed: 5,
        ..StageCode::default()
    };
    let mut g = c.game();
    let mut fb = vec![0u32; W * H];
    for i in 0..n {
        g.update(
            1.0 / 60.0,
            &Input {
                accel: true,
                left: (i / 90) % 2 == 0,
                ..Default::default()
            },
        );
        render::render(&g, &mut fb);
    }
    println!("{}", fb[1000]);
}
