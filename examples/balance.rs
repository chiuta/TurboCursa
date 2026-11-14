//! Balance survey with a simple bot: finish / time-up / police / out-of-fuel / damage per class and level.
//! `cargo run --release --example balance`
use std::sync::Mutex;
use turbocursa::game::*;
use turbocursa::share::StageCode;

#[derive(Default, Clone, Copy)]
struct R {
    n: u32,
    fin: u32,
    up: u32,
    dq: u32,
    dry: u32,
    dmg: f32,
}

fn main() {
    let out = Mutex::new(Vec::new());
    let by_w = Mutex::new(std::collections::BTreeMap::<(u8, u8), (u32, u32)>::new());
    std::thread::scope(|s| {
        for class in 0..turbocursa::cars::CLASS_COUNT as u8 {
            let out = &out;
            let by_w = &by_w;
            s.spawn(move || {
                for level in [0u8, 2, 4] {
                    let mut r = R::default();
                    for (seed, weather) in [(3u32, 0u8), (4, 2), (5, 6), (6, 9)] {
                        let mut g = StageCode {
                            seed,
                            mode: 0,
                            class,
                            level,
                            weather,
                            ..StageCode::default()
                        }
                        .game();
                        let mut dry = false;
                        if std::env::var("PRACTICE").is_ok() {
                            g.practice();
                        }
                        for _ in 0..60 * 400 {
                            let i = g.autopilot();
                            g.update(1.0 / 60.0, &i);
                            dry |= g.ran_dry;
                            if g.state != State::Playing {
                                break;
                            }
                        }
                        {
                            let mut m = by_w.lock().unwrap();
                            let e = m.entry((level, weather)).or_default();
                            e.0 += 1;
                            e.1 += (g.state == State::Finished) as u32;
                        }
                        r.n += 1;
                        match g.state {
                            State::Finished => r.fin += 1,
                            State::TimeUp => r.up += 1,
                            State::Disqualified => r.dq += 1,
                            _ => {}
                        }
                        r.dry += dry as u32;
                        r.dmg += 1.0 - (g.body.min(g.engine));
                    }
                    out.lock().unwrap().push((class, level, r));
                }
            });
        }
    });
    for ((l, w), (n, f)) in by_w.into_inner().unwrap() {
        println!("level {l} weather {w}: finished {f}/{n}");
    }
    let mut v = out.into_inner().unwrap();
    v.sort_by_key(|x| (x.1, x.0));
    println!("level class | n fin timeup police dry avg-damage");
    for (c, l, r) in v {
        println!(
            "{l} {c:2} | {} {} {} {} {} {:.2}",
            r.n,
            r.fin,
            r.up,
            r.dq,
            r.dry,
            r.dmg / r.n as f32
        );
    }
}
