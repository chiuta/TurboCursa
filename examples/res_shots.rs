//! Screenshots for the resolution comparison on the presentation page.
//!   res_shots cand            -> /tmp/cand/NN.png (640x400 candidates) to choose scenes from
//!   res_shots shot SEED WEATHER TIME MODE SECONDS NAME  -> /tmp/shots/NAME_<res>.png for SD and every Res
use turbocursa::ace::Ace;
use turbocursa::game::*;
use turbocursa::hd::{Res, Upscaler};
use turbocursa::share::StageCode;
use turbocursa::{render, H, W};

fn drive(seed: u32, weather: u8, time: u8, mode: u8, secs: f32) -> Vec<u32> {
    let mut g = StageCode { seed, weather, time, mode, ..StageCode::default() }.game();
    g.demo = true;
    let mut ace = Ace::new();
    for _ in 0..(secs * 60.0) as usize {
        let i = ace.input(&g);
        g.update(1.0 / 60.0, &i);
    }
    let mut fb = vec![0u32; W * H];
    render::render(&g, &mut fb);
    fb
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    std::fs::create_dir_all("/tmp/cand").unwrap();
    std::fs::create_dir_all("/tmp/shots").unwrap();
    if a[1] == "cand" {
        let mut n = 0;
        for (seed, weather, time, mode) in [
            (3u32, 1u8, 2u8, 0u8), (8, 1, 3, 0), (12, 1, 4, 0), (21, 2, 2, 0), (5, 3, 4, 0), (17, 5, 2, 0),
            (4, 5, 3, 1), (30, 1, 1, 0), (41, 1, 3, 2), (6, 1, 4, 2), (9, 4, 1, 0), (14, 10, 3, 0),
            (25, 1, 2, 3), (33, 1, 3, 3), (2, 1, 4, 3), (50, 1, 1, 1), (61, 1, 2, 1), (70, 1, 3, 4),
        ] {
            let fb = drive(seed, weather, time, mode, 22.0);
            std::fs::write(format!("/tmp/cand/{n:02}.png"), turbocursa::png::encode(&fb, W, H)).unwrap();
            println!("{n:02} seed {seed} weather {weather} time {time} mode {mode}");
            n += 1;
        }
    } else {
        let p = |i: usize| a[i].parse::<f32>().unwrap();
        let fb = drive(p(2) as u32, p(3) as u8, p(4) as u8, p(5) as u8, p(6));
        let name = &a[7];
        std::fs::write(format!("/tmp/shots/{name}_sd.png"), turbocursa::png::encode(&fb, W, H)).unwrap();
        for res in Res::ALL {
            let (fw, fh) = res.frame();
            let mut dst = vec![0u32; fw * fh];
            Upscaler::new(res).run(&fb, &mut dst);
            let (pw, _) = res.pic();
            let bar = (fw - pw) / 2;
            // the picture without the black side bars
            let mut pic = Vec::with_capacity(pw * fh);
            for y in 0..fh {
                pic.extend_from_slice(&dst[y * fw + bar..y * fw + bar + pw]);
            }
            let tag = format!("{:?}", res).to_lowercase();
            std::fs::write(format!("/tmp/shots/{name}_{tag}.png"), turbocursa::png::encode(&pic, pw, fh)).unwrap();
        }
    }
}
