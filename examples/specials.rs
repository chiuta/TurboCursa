//! Renders each special vehicle (kinds 2..=11) 3000 units ahead of the player into /tmp/sp_<kind>.png.
use turbocursa::game::*;
use turbocursa::share::StageCode;
use turbocursa::{H, W};
fn main() {
    for kind in 0..=11u8 {
        let mut g = StageCode {
            seed: 7,
            mode: 0,
            ..StageCode::default()
        }
        .game();
        let inp = Input::default();
        for _ in 0..240 {
            g.update(1.0 / 60.0, &inp);
        }
        g.spawn_special(kind, 2600.0);
        let mut fb = vec![0u32; W * H];
        g.update(0.0, &inp);
        turbocursa::render::render(&g, &mut fb);
        let png = turbocursa::png::encode(&fb, W, H);
        std::fs::write(format!("/tmp/sp_{kind}.png"), png).unwrap();
    }
}
