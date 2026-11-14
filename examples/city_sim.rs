//! City / jam survey with the bot: `cargo run --release --example city_sim`
use turbocursa::game::*;
use turbocursa::share::StageCode;

fn main() {
    for level in [0u8, 2, 4] {
        for seed in [3u32, 4, 5, 6, 7, 8] {
            let mut g = StageCode {
                seed,
                mode: 0,
                class: 0,
                level,
                weather: 0,
                ..StageCode::default()
            }
            .game();
            let mut maxspd_city = 0.0f32;
            if std::env::var("NOCITY").is_ok() {
                g.city = Default::default();
                g.jam_cars.clear();
            }
            if std::env::var("PRACTICE").is_ok() {
                g.practice();
            }
            let mut t_end = 0.0;
            for _ in 0..60 * 400 {
                let i = g.autopilot();
                g.update(1.0 / 60.0, &i);
                let seg = ((g.pos + PLAYER_Z) / 200.0) as usize;
                if g.city.zones.iter().any(|z| seg >= z.a + 40 && seg < z.b) {
                    maxspd_city = maxspd_city.max(g.speed);
                }
                t_end = g.elapsed;
                if g.state != State::Playing {
                    break;
                }
            }
            println!(
                "L{} s{} {:?} t={:.0} zones={} jams={} lights={} viol={} hit={} yield={} detours={} cityspd={:.0}% crashes={}",
                level, seed, g.state, t_end, g.city.zones.len(), g.city.jams.len(), g.city.lights.len(),
                g.violations, g.peds_hit, g.peds_yielded, g.detours, maxspd_city / MAX_SPEED * 100.0, g.crashes
            );
        }
    }
}
