//! Prints a hash of rendered frames for a few weather / time / seed combinations (to compare two builds).
use turbocursa::game::*;
use turbocursa::share::StageCode;
use turbocursa::{render, H, W};
fn main() {
    let mut all = 0xcbf29ce484222325u64;
    for (seed, weather, time, mode) in [
        (5u32, 2u8, 4u8, 0u8),
        (7, 0, 1, 0),
        (9, 4, 3, 1),
        (3, 3, 0, 2),
        (11, 1, 2, 0),
    ] {
        let mut g = StageCode {
            seed,
            weather,
            time,
            mode,
            ..StageCode::default()
        }
        .game();
        let mut fb = vec![0u32; W * H];
        for i in 0..240 {
            g.update(
                1.0 / 60.0,
                &Input {
                    accel: true,
                    left: i % 90 < 20,
                    ..Input::default()
                },
            );
            if i % 60 == 59 {
                render::render(&g, &mut fb);
                for &p in &fb {
                    all = (all ^ p as u64).wrapping_mul(0x100000001b3);
                }
            }
        }
    }
    println!("{all:016x}");
}
