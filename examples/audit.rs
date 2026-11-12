use turbocursa::game::*;
use turbocursa::track::{self, SEG_LEN};

fn bot(g: &Game, throttle_cap: f32) -> Input {
    let mut target = 0.0f32;
    let pz = g.pos + PLAYER_Z;
    let len = g.track.length;
    let mut brake = false;
    for c in &g.cars {
        let d = (c.z - pz).rem_euclid(len);
        if d < 4000.0 && (c.x - g.x).abs() < 0.4 {
            target = if c.x > 0.0 { -0.66 } else { 0.66 };
            if d < 1200.0 && g.speed > c.speed * 1.1 {
                brake = true;
            }
        }
    }
    let err = target - g.x;
    Input {
        left: err < -0.04,
        right: err > 0.04,
        accel: !brake && g.speed < MAX_SPEED * throttle_cap,
        brake,
        turbo: g.turbo > 0 && g.pos > 20000.0 && (g.elapsed as i32 % 14 == 6),
    }
}

fn run(seed: u64, cap: f32) -> (State, f32, f32) {
    let mut g = Game::new(seed);
    let mut t = 0.0;
    while g.state == State::Playing && t < 200.0 {
        let i = bot(&g, cap);
        g.update(1.0 / 60.0, &i);
        t += 1.0 / 60.0;
    }
    (g.state, g.time, g.pos / g.track.length)
}

fn main() {
    // ---- track statistics
    let (mut maxy, mut maxslope, mut maxcurve, mut maxseam, mut worst_req) =
        (0f32, 0f32, 0f32, 0f32, 0f32);
    let (mut minn, mut maxn) = (usize::MAX, 0usize);
    let mut worst_seed = 0;
    for seed in 0..300u64 {
        let t = track::build(seed);
        let n = t.segs.len();
        minn = minn.min(n);
        maxn = maxn.max(n);
        for s in &t.segs {
            maxy = maxy.max(s.y0.abs()).max(s.y1.abs());
            maxslope = maxslope.max((s.y1 - s.y0).abs());
            maxcurve = maxcurve.max(s.curve.abs());
            assert!(
                s.y0.is_finite() && s.y1.is_finite() && s.curve.is_finite() && s.tone.is_finite()
            );
        }
        let last = &t.segs[n - 1];
        maxseam = maxseam.max((last.y0 - last.y1).abs());
        // average speed (fraction of top speed) needed to survive each leg
        let mut start = 0.0;
        let mut time = START_TIME;
        let marks: Vec<f32> = t
            .checkpoints
            .iter()
            .map(|&c| c as f32 * SEG_LEN)
            .chain(std::iter::once(t.finish_seg as f32 * SEG_LEN))
            .collect();
        for (i, &m) in marks.iter().enumerate() {
            let req = (m - start) / time / MAX_SPEED;
            if req > worst_req {
                worst_req = req;
                worst_seed = seed;
            }
            start = m;
            time += if i < t.checkpoints.len() {
                CP_BONUS
            } else {
                0.0
            };
            time = if i < t.checkpoints.len() { time } else { time };
        }
    }
    println!("TRACK (300 seeds): segs {}..{}  max|y| {:.0}  max slope/seg {:.0}  max|curve| {:.2}  worst end-seam jump {:.0}", minn, maxn, maxy, maxslope, maxcurve, maxseam);
    println!(
        "  hardest leg needs {:.0}% of top speed on average (seed {})  [camera height is 1000]",
        worst_req * 100.0,
        worst_seed
    );

    // ---- completability
    for (label, cap) in [
        ("flat-out bot", 1.0f32),
        ("70% throttle bot", 0.7),
        ("55% throttle bot", 0.55),
    ] {
        let (mut fin, mut tup, mut other) = (0, 0, 0);
        let mut min_left = f32::MAX;
        let mut worst_prog = 1.0f32;
        for seed in 0..60u64 {
            let (st, left, prog) = run(seed, cap);
            match st {
                State::Finished => {
                    fin += 1;
                    min_left = min_left.min(left);
                }
                State::TimeUp => {
                    tup += 1;
                    worst_prog = worst_prog.min(prog);
                }
                _ => other += 1,
            }
        }
        println!("{label:18} finished {fin}/60  time-up {tup}  stuck {other}  min time left {:.1}s  least progress at time-up {:.0}%", if min_left == f32::MAX { 0.0 } else { min_left }, worst_prog * 100.0);
    }
}
