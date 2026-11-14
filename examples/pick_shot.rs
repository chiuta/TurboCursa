//! frames/pick_*.ppm: the start-of-game selection screen for a few classes; frames/traffic_mix.ppm: busy road.
use turbocursa::game::*;
use turbocursa::pick::PickApp;
use turbocursa::{render, H, W};

fn ppm(name: &str, fb: &[u32]) {
    let mut v = format!("P6\n{} {}\n255\n", W, H).into_bytes();
    for &c in fb {
        v.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
    }
    std::fs::write(format!("frames/{name}.ppm"), v).unwrap();
}

fn main() {
    let mut fb = vec![0u32; W * H];
    for (n, c) in [6usize, 15].into_iter().enumerate() {
        PickApp::new(c, 3).render(&mut fb);
        ppm(&format!("pick_{n}"), &fb);
    }
    let mut g = Game::new_mode(21, Mode::Campaign);
    g.set_car(
        turbocursa::cars::CarSpec::default().with_class(turbocursa::cars::VehicleClass::Motorcycle),
    );
    g.items.clear();
    g.pos = 3000.0;
    g.speed = 6000.0;
    let base = g.pos + PLAYER_Z;
    let lanes = [-0.6f32, 0.0, 0.6];
    for (i, kind) in [
        K_SEMI, K_BUS, K_BIKE, K_TRACTOR, K_TRIKE, K_TRUCK, K_MICRO, K_CAR,
    ]
    .into_iter()
    .enumerate()
    {
        g.cars[i].kind = kind;
        g.cars[i].z = base + 1100.0 + 520.0 * i as f32;
        g.cars[i].x = lanes[i % 3];
    }
    let mut f = vec![0u32; W * H];
    render::render(&g, &mut f);
    ppm("traffic_mix", &f);
}
