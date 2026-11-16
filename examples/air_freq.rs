//! Counts how often each special vehicle kind appears in simulated 10-minute runs.
use turbocursa::game::*;
use turbocursa::share::StageCode;

fn main() {
    let mut seen = [0u32; 31];
    let runs = 6;
    for seed in 1..=runs {
        let mut g = StageCode {
            seed,
            mode: 0,
            weather: 1,
            time: 1,
            ..StageCode::default()
        }
        .game();
        let inp = Input {
            accel: true,
            ..Input::default()
        };
        let mut was = [false; 31];
        for _ in 0..(600 * 30) {
            g.update(1.0 / 30.0, &inp);
            let mut now = [false; 31];
            for e in &g.emerg {
                now[(e.kind as usize).min(30)] = true;
            }
            for k in 0..31 {
                if now[k] && !was[k] {
                    seen[k] += 1;
                }
            }
            was = now;
        }
    }
    for (k, n) in seen.iter().enumerate() {
        println!("kind {k}: {n} appearances in {runs} runs of 10 min");
    }
}
