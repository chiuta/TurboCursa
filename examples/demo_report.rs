//! Headless run of the whole demo playlist: `cargo run --release --example demo_report [FROM [TO]]`.
use turbocursa::ace::{Ace, Demo, ENDLESS_CAP};
use turbocursa::game::State;
fn main() {
    let a: Vec<usize> = std::env::args()
        .skip(1)
        .filter_map(|x| x.parse().ok())
        .collect();
    let list = Demo::list();
    let (from, to) = (
        a.first().copied().unwrap_or(0),
        a.get(1).copied().unwrap_or(list.len()),
    );
    let (mut total, mut ok) = (0.0, 0);
    for (i, c) in list.iter().enumerate().skip(from).take(to - from) {
        let mut g = c.game();
        g.demo = std::env::var("NODEMO").is_err();
        let mut ace = Ace::new();
        let limit = 300 * 60;
        let lap0 = g.lap;
        for _ in 0..limit {
            let inp = ace.input(&g);
            g.update(1.0 / 60.0, &inp);
            let lap = g.mode == turbocursa::game::Mode::Endless
                && (g.lap > lap0 || g.elapsed > ENDLESS_CAP);
            if g.state != State::Playing || lap {
                break;
            }
        }
        let fin = g.state == State::Finished || (g.mode == turbocursa::game::Mode::Endless);
        if fin {
            ok += 1;
            total += g.elapsed;
        }
        println!(
            "{i:>2} mode {} level {} class {} -> {:?} {:.1}s crashes {}",
            c.mode, c.level, c.class, g.state, g.elapsed, g.crashes
        );
    }
    println!("finished {ok}/{} stages, total {:.0}s", to - from, total);
}
