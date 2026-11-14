//! Portraits of the special vehicles for the presentation page, rendered natively at 1728x1080.
//!   special_shots OUTDIR  -> OUTDIR/sv_<kind>.png and .box (kinds 0..=19, 20 police car, 21 helicopter)
use turbocursa::game::*;
use turbocursa::share::StageCode;

fn scene(kind: Option<u8>) -> Vec<u32> {
    let extra = kind.filter(|&k| k >= 20);
    let kind = kind.filter(|&k| k < 20);
    let mut g = StageCode {
        seed: 7,
        mode: 0,
        weather: 1,
        time: 2,
        ..StageCode::default()
    }
    .game();
    let inp = Input::default();
    for _ in 0..240 {
        g.update(1.0 / 60.0, &inp);
    }
    if let Some(k) = kind {
        g.spawn_special(k, 1900.0);
    }
    g.update(0.0, &inp);
    let (w, h) = (1728usize, 1080usize);
    let mut fb = vec![0u32; w * h];
    turbocursa::render::render_view(&g, &mut fb, w, h);
    // 20: police car, 21: traffic helicopter (drawn straight onto the frame)
    match extra {
        Some(20) => turbocursa::render::draw_portrait(&mut fb, w, h, 0, 864.0, 715.0, 190.0),
        Some(21) => turbocursa::render::draw_portrait(&mut fb, w, h, 1, 864.0, 640.0, 250.0),
        _ => {}
    }
    fb
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or("/tmp".into());
    let (w, h) = (1728usize, 1080usize);
    let base = scene(None);
    for kind in 0..=21u8 {
        let fb = scene(Some(kind));
        let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
        for y in 300..740 {
            for x in 200..1340 {
                let (a, b) = (fb[y * w + x], base[y * w + x]);
                let d = |s: u32| ((a >> s) & 255).abs_diff((b >> s) & 255);
                if d(16) + d(8) + d(0) > 40 {
                    x0 = x0.min(x);
                    x1 = x1.max(x);
                    y0 = y0.min(y);
                    y1 = y1.max(y);
                }
            }
        }
        println!("{kind}: {x0},{y0} - {x1},{y1}");
        std::fs::write(
            format!("{out}/sv_{kind}.png"),
            turbocursa::png::encode(&fb, w, h),
        )
        .unwrap();
        std::fs::write(
            format!("{out}/sv_{kind}.box"),
            format!("{x0} {y0} {x1} {y1}"),
        )
        .unwrap();
    }
}
