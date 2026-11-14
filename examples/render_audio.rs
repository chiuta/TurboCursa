//! Renders the game's own synthesizer to a raw mono f32 file (little endian, 44.1 kHz) following a
//! timeline "end_seconds:mode,..." with modes menu, drive, pause, demo. Used for the page videos.
use turbocursa::ace::Demo;
use turbocursa::audio::{AudioParams, Synth};
use turbocursa::game::{Game, Input, Mode};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let plan: Vec<(f32, String)> = a[1]
        .split(',')
        .filter_map(|s| {
            let (t, m) = s.split_once(':')?;
            Some((t.parse().ok()?, m.to_string()))
        })
        .collect();
    let sr = 44100.0f32;
    let mut syn = Synth::new(sr, 7);
    let mut out: Vec<f32> = Vec::new();
    let mut game: Option<Game> = None;
    let mut demo: Option<Demo> = None;
    let mut prev = String::new();
    let dt = 1.0 / 60.0;
    let spf = (sr / 60.0) as usize;
    let mut frame = 0u32;
    for (end, mode) in &plan {
        while (out.len() as f32) / sr < *end {
            frame += 1;
            let mut buf = vec![0.0f32; spf];
            let p = match mode.as_str() {
                "drive" => {
                    if prev != "drive" {
                        game = Some(Game::new_mode(7, Mode::Campaign));
                    }
                    let g = game.as_mut().unwrap();
                    let wob = (frame / 90) % 4;
                    let inp = Input {
                        accel: true,
                        left: wob == 1 && frame % 90 < 30,
                        right: wob == 3 && frame % 90 < 30,
                        ..Input::default()
                    };
                    g.update(dt, &inp);
                    g.audio_params()
                }
                "demo" => {
                    if prev != "demo" {
                        let d = Demo::new();
                        game = Some(d.current());
                        demo = Some(d);
                    }
                    let d = demo.as_mut().unwrap();
                    let g = game.as_mut().unwrap();
                    let (inp, ng) = d.step(g, dt);
                    g.update(dt, &inp);
                    if let Some(n) = ng {
                        *g = n;
                    }
                    g.audio_params()
                }
                _ => AudioParams::default(),
            };
            prev = mode.clone();
            syn.render(&mut buf, &p);
            out.extend_from_slice(&buf);
        }
    }
    let bytes: Vec<u8> = out.iter().flat_map(|s| s.to_le_bytes()).collect();
    std::fs::write(&a[2], bytes).unwrap();
}
