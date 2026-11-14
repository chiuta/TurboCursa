//! frames/world_*.ppm: tunnel, bridge, viaduct, mountain pass, animals, police, ramps, station, day and night.
use turbocursa::game::*;
use turbocursa::track::*;
use turbocursa::{render, H, W};

fn ppm(name: &str, fb: &[u32]) {
    let mut v = format!("P6\n{} {}\n255\n", W, H).into_bytes();
    for &c in fb {
        v.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
    }
    std::fs::write(format!("frames/{name}.ppm"), v).unwrap();
}

fn main() {
    std::fs::create_dir_all("frames").unwrap();
    let mut f = vec![0u32; W * H];
    let seed = 21;
    let mut g = Game::new_mode(seed, Mode::Campaign);
    g.set_env(1, 1);
    g.practice();
    g.set_time(2);
    println!("features: {:?}", g.track.features);
    for kind in [F_TUNNEL, F_BRIDGE, F_VIADUCT, F_PASS] {
        if let Some(&(a, b, _)) = g.track.features.iter().find(|x| x.2 == kind) {
            for (tag, off) in [("in", 8), ("approach", -28)] {
                let seg = (a as i32 + off + (b - a) as i32 / 4).max(0) as f32;
                g.pos = seg * SEG_LEN - PLAYER_Z;
                g.speed = 7000.0;
                g.x = 0.2;
                render::render(&g, &mut f);
                ppm(&format!("world_{}_{}", kind, tag), &f);
            }
        }
    }
    // animals, cops, ramps, pad, night
    let mut g = Game::new_mode(seed, Mode::Campaign);
    g.set_env(1, 1);
    g.set_time(2);
    g.items.clear();
    g.pos = 30000.0;
    g.speed = 6000.0;
    let pz = g.pos + PLAYER_Z;
    g.animals.clear();
    for (i, sp) in [0u8, 3, 5, 7].iter().enumerate() {
        g.animals.push(Animal {
            z: pz + 1300.0 + 500.0 * i as f32,
            x: -0.7 + 0.45 * i as f32,
            dir: 1.0,
            species: *sp,
            lead: 2.0,
            active: true,
            done: false,
            hit: false,
            passed: false,
            hit_t: 0.0,
        });
    }
    g.cops.push(Cop {
        z: pz + 2600.0,
        x: 0.3,
        speed: 5000.0,
        age: 3.0,
        mode: 1,
    });
    for (i, k) in [I_RAMP_S, I_RAMP_M, I_RAMP_L, I_PAD].iter().enumerate() {
        g.items.push(Item {
            z: pz + 3200.0 + 450.0 * i as f32,
            x: -0.6 + 0.4 * i as f32,
            kind: *k,
            taken: false,
        });
    }
    render::render(&g, &mut f);
    ppm("world_fauna", &f);
    g.set_time(4);
    render::render(&g, &mut f);
    ppm("world_night", &f);
    g.set_time(1);
    render::render(&g, &mut f);
    ppm("world_dawn", &f);
    // station and a jump
    let mut g = Game::new_mode(seed, Mode::Campaign);
    g.practice();
    g.fuel_rule = true;
    g.fuel = 0.3;
    g.set_time(2);
    let (a, _) = g.stations[0];
    g.pos = a - 5200.0;
    g.speed = 5000.0;
    render::render(&g, &mut f);
    ppm("world_station_far", &f);
    g.pos = a - 600.0;
    g.x = 0.5;
    render::render(&g, &mut f);
    ppm("world_station", &f);
    g.air = 3.0;
    render::render(&g, &mut f);
    ppm("world_air", &f);
}
