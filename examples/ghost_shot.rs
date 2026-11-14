//! Ghost screenshot: `cargo run --release --example ghost_shot -- out.png`
use turbocursa::game::*;
fn main() {
    let out = std::env::args().nth(1).unwrap_or("ghost.png".into());
    let mut a = Game::new_mode(33, Mode::TimeTrial);
    for _ in 0..1200 {
        let i = a.autopilot();
        a.update(1.0 / 60.0, &Input { accel: true, ..i });
    }
    a.state = State::Finished;
    let gh = a.make_ghost().unwrap();
    let mut b = Game::new_mode(33, Mode::TimeTrial);
    b.set_ghost(Some(gh));
    for _ in 0..420 {
        let i = b.autopilot();
        // a slightly timid driver: the ghost pulls ahead
        b.update(
            1.0 / 60.0,
            &Input {
                accel: (b.elapsed * 3.0) as i32 % 5 != 0,
                ..i
            },
        );
    }
    let mut fb = vec![0u32; turbocursa::W * turbocursa::H];
    turbocursa::render::render(&b, &mut fb);
    std::fs::write(
        &out,
        turbocursa::png::encode(&fb, turbocursa::W, turbocursa::H),
    )
    .unwrap();
}
