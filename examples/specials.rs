//! Renders each special vehicle (kinds 0..=11) 2600 units ahead of the player into /tmp/sp_<kind>.png,
//! natively at 1728x1080.
use turbocursa::game::*;
use turbocursa::share::StageCode;
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
        let (w, h) = (1728usize, 1080usize);
        let mut fb = vec![0u32; w * h];
        g.update(0.0, &inp);
        turbocursa::render::render_view(&g, &mut fb, w, h);
        let png = turbocursa::png::encode(&fb, w, h);
        std::fs::write(format!("/tmp/sp_{kind}.png"), png).unwrap();
    }
}
