//! Frame cost per weather and per landscape (worst combos first): `cargo run --release --example cost`
use std::time::Instant;
use turbocursa::game::*;
use turbocursa::share::StageCode;
use turbocursa::track::Layout;
use turbocursa::{render, H, NB, W};
fn cost(c: StageCode) -> f64 {
    let mut g = c.game();
    let mut fb = vec![0u32; W * H];
    let mut v = Vec::new();
    for i in 0..400 {
        let t = Instant::now();
        g.update(
            1.0 / 60.0,
            &Input {
                accel: true,
                left: (i / 90) % 2 == 0,
                ..Default::default()
            },
        );
        render::render(&g, &mut fb);
        v.push(t.elapsed().as_secs_f64() * 1e3);
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() * 95 / 100]
}
fn main() {
    let mut rows = Vec::new();
    for w in 0..=turbocursa::env::WEATHER_COUNT as u8 {
        for t in [1u8, 4] {
            rows.push((
                format!("weather {w} time {t}"),
                cost(StageCode {
                    weather: w,
                    time: t,
                    seed: 5,
                    ..StageCode::default()
                }),
            ));
        }
    }
    for b in 0..NB as u8 {
        let l = Layout {
            biomes: [b; 4],
            ..Layout::default()
        };
        rows.push((
            format!("biome {b}"),
            cost(StageCode {
                layout: l,
                seed: 5,
                ..StageCode::default()
            }),
        ));
    }
    for c in 0..turbocursa::cars::CLASS_COUNT as u8 {
        rows.push((
            format!("class {c}"),
            cost(StageCode {
                class: c,
                weather: 2,
                seed: 5,
                ..StageCode::default()
            }),
        ));
    }
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (n, c) in rows.iter().take(12) {
        println!("{n:24} p95 {c:.1} ms");
    }
}
