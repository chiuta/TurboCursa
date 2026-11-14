//! frames/city_*.png: city, crosswalk, slip road and jam. `cargo run --release --example city_shots`
use turbocursa::game::*;
use turbocursa::{render, H, W};
fn ppm(name: &str, fb: &[u32]) {
    let mut v = format!("P6\n{} {}\n255\n", W, H).into_bytes();
    for &c in fb {
        v.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
    }
    std::fs::write(format!("frames/{name}.ppm"), v).unwrap();
}
fn shot(g: &mut Game, name: &str, seg: f32, x: f32, speed: f32) {
    g.pos = seg * 200.0 - PLAYER_Z;
    g.x = x;
    g.speed = speed;
    g.state = State::Playing;
    g.update(0.016, &Input::default());
    let mut f = vec![0u32; W * H];
    render::render(g, &mut f);
    ppm(name, &f);
}
fn main() {
    std::fs::create_dir_all("frames").unwrap();
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    let mut g = Game::new_mode(seed, Mode::Campaign);
    g.shield_t = 100.0;
    let z = g.city.zones[0];
    let l = g.city.lights[0];
    let ls = (l.z / 200.0) as f32;
    // red phase with pedestrians
    g.elapsed = (CYCLE_RED_START() - l.phase).rem_euclid(turbocursa::city::CYCLE);
    shot(&mut g, "city_a", z.a as f32 + 8.0, 0.0, 6000.0);
    g.elapsed = (turbocursa::city::GREEN + turbocursa::city::YELLOW + 1.5 - l.phase)
        .rem_euclid(turbocursa::city::CYCLE);
    shot(&mut g, "city_b", ls - 14.0, 0.0, 3000.0);
    if let Some(m) = g.mission {
        shot(&mut g, "mission_a", m.pick as f32 - 12.0, 0.0, 6000.0);
    }
    for (i, ar) in g.areas.clone().iter().enumerate() {
        let n = if ar.kind == 0 { "proving" } else { "parking" };
        shot(&mut g, &format!("{n}_a{i}"), ar.a as f32 - 6.0, 0.0, 6000.0);
        shot(
            &mut g,
            &format!("{n}_b{i}"),
            ar.a as f32 + if ar.kind == 0 { 55.0 } else { 20.0 },
            if ar.kind == 0 { 0.0 } else { 1.5 },
            800.0,
        );
    }
    let sc = g.city.scenes[0];
    shot(&mut g, "scene_a", sc / 200.0 - 60.0, -0.2, 6000.0);
    shot(&mut g, "scene_b", sc / 200.0 - 22.0, -0.2, 6000.0);
    shot(&mut g, "scene_c", sc / 200.0 - 8.0, -0.2, 3000.0);
    g.emerg.clear();
    g.pos = 40000.0;
    g.speed = 6000.0;
    g.elapsed = 30.0;
    g.update(0.016, &Input::default());
    for _ in 0..140 {
        g.update(0.016, &Input::default());
    }
    println!("emerg n={}", g.emerg.len());
    if let Some(e) = g.emerg.first().copied() {
        shot(&mut g, "emerg_a", e.z / 200.0 - 14.0, -0.5, 6000.0);
    }
    if let Some(j) = g.city.jams.first().copied() {
        shot(&mut g, "jam_a", j.exit_a() as f32 - 30.0, 0.0, 6000.0);
        shot(&mut g, "jam_b", j.exit_a() as f32 + 15.0, 0.4, 6000.0);
        shot(&mut g, "jam_c", j.a as f32 - 6.0, 0.0, 800.0);
        g.x = 1.2;
        shot(&mut g, "jam_d", j.exit_a() as f32 + 20.0, 1.3, 7000.0);
        shot(&mut g, "jam_e", j.a as f32 + 30.0, 0.5, 7000.0);
    }
}
#[allow(non_snake_case)]
fn CYCLE_RED_START() -> f32 {
    turbocursa::city::GREEN + turbocursa::city::YELLOW + 1.5
}
