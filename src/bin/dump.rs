//! Headless driver: plays the stage with a simple bot and writes PNG frames.
//! usage: dump <out_dir> \[seed\]
use std::{fs, io::Write};
use turbocursa::game::*;
use turbocursa::{render, H, W};

fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
    }
    !c
}

fn chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut body = tag.to_vec();
    body.extend_from_slice(data);
    out.extend_from_slice(&body);
    out.extend_from_slice(&crc32(&body).to_be_bytes());
}

fn png(fb: &[u32], scale: usize) -> Vec<u8> {
    let (w, h) = (W * scale, H * scale);
    let mut raw = Vec::with_capacity((w * 3 + 1) * h);
    for y in 0..h {
        raw.push(0);
        for x in 0..w {
            let c = fb[(y / scale) * W + x / scale];
            raw.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
        }
    }
    // zlib, stored blocks
    let mut z = vec![0x78, 0x01];
    for (i, blk) in raw.chunks(65535).enumerate() {
        let last = (i + 1) * 65535 >= raw.len();
        z.push(last as u8);
        z.extend_from_slice(&(blk.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(blk.len() as u16)).to_le_bytes());
        z.extend_from_slice(blk);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &v in &raw {
        a = (a + v as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

pub fn bot(g: &Game) -> Input {
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
        accel: !brake,
        brake,
        turbo: g.turbo > 0 && g.pos > 20000.0 && (g.elapsed as i32 % 14 == 6),
    }
}

fn main() {
    if let Some(l) = std::env::var("DUMP_LANG")
        .ok()
        .and_then(|s| turbocursa::i18n::Lang::parse(&s))
    {
        turbocursa::i18n::set_lang(l);
    }
    let mut args = std::env::args().skip(1);
    let dir = args.next().unwrap_or_else(|| "frames".into());
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    fs::create_dir_all(&dir).unwrap();
    let mut g = Game::new(seed);
    let mut fb = vec![0u32; W * H];
    let mut synth = turbocursa::audio::Synth::new(44100.0, seed);
    let mut wav: Vec<i16> = Vec::new();
    let mut abuf = vec![0.0f32; 735];
    if std::env::var("DUMP_VERSUS").is_ok() {
        let mut v = turbocursa::versus::Versus::new(seed);
        let mut out = vec![0u32; W * H];
        for i in 0..(60 * 14) {
            // P1 runs the bot; P2 is a slower, wobblier driver so the standings differ
            let a = bot(&v.p[0]);
            let mut b = bot(&v.p[1]);
            b.accel = i % 7 != 0;
            v.update(1.0 / 60.0, &[a, b]);
        }
        v.render(&mut out);
        fs::File::create(format!("{}/versus.png", dir))
            .unwrap()
            .write_all(&png(&out, 1))
            .unwrap();
        println!(
            "versus: p1 pos {:.0} p2 pos {:.0} winner {:?}",
            v.p[0].pos, v.p[1].pos, v.winner
        );
        return;
    }
    if let Ok(cp) = std::env::var("DUMP_GATE") {
        // teleport in front of checkpoint N and render it
        let n: usize = cp.parse().unwrap_or(0);
        g.pos = g.track.checkpoints[n] as f32 * turbocursa::track::SEG_LEN - 9000.0;
        g.speed = MAX_SPEED * 0.6;
        g.update(
            0.016,
            &Input {
                accel: true,
                ..Default::default()
            },
        );
        render::render(&g, &mut fb);
        fs::File::create(format!("{}/gate{}.png", dir, n))
            .unwrap()
            .write_all(&png(&fb, 1))
            .unwrap();
        return;
    }
    let dt = 1.0 / 60.0;
    let shots = [2.0f32, 9.0, 20.0, 34.0, 52.0, 70.0];
    let mut next = 0;
    let mut t = 0.0f32;
    while t < 140.0 && g.state == State::Playing {
        let inp = bot(&g);
        g.update(dt, &inp);
        t += dt;
        synth.render(&mut abuf, &g.audio_params());
        if t < 45.0 {
            wav.extend(abuf.iter().map(|s| (s.clamp(-1.0, 1.0) * 32000.0) as i16));
        }
        if next < shots.len() && t >= shots[next] {
            render::render(&g, &mut fb);
            let name = format!("{}/f{:02}.png", dir, next);
            fs::File::create(&name)
                .unwrap()
                .write_all(&png(&fb, 1))
                .unwrap();
            println!(
                "t={:5.1}s pos={:7.0}/{:.0} km/h={:3} time={:4.1} biome_w={:?} overtaken={}",
                t,
                g.pos,
                g.track.length,
                g.kmh(),
                g.time,
                g.biome_w.map(|v| (v * 100.0).round() / 100.0),
                g.overtaken
            );
            next += 1;
        }
    }
    render::render(&g, &mut fb);
    fs::File::create(format!("{}/final.png", dir))
        .unwrap()
        .write_all(&png(&fb, 1))
        .unwrap();
    {
        let mut out = Vec::new();
        let data_len = (wav.len() * 2) as u32;
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data_len).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&44100u32.to_le_bytes());
        out.extend_from_slice(&88200u32.to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        for s in &wav {
            out.extend_from_slice(&s.to_le_bytes());
        }
        fs::File::create(format!("{}/audio.wav", dir))
            .unwrap()
            .write_all(&out)
            .unwrap();
    }
    println!(
        "END state={:?} t={:.1}s pos={:.0} score={} overtaken={} time_left={:.1}",
        g.state, t, g.pos, g.score, g.overtaken, g.time
    );
}
