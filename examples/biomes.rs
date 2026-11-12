use std::io::Write;
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

fn main() {
    std::fs::create_dir_all("frames").unwrap();
    let names = [
        "forest",
        "desert",
        "dusk",
        "worldtrees",
        "mesas",
        "islands",
        "mirror",
        "tundra",
        "volcano",
        "abyss",
        "cosmos",
        "citadel",
        "myco",
        "inverted",
        "prism",
        "clockwork",
        "cloudsea",
        "blackhole",
        "canyon",
    ];
    let mut got = [false; turbocursa::NB];
    for seed in 0..200u64 {
        let mut g = Game::new(seed);
        for b in 0..turbocursa::NB as u8 {
            if got[b as usize] {
                continue;
            }
            if let Some(i) = g.track.segs.iter().position(|s| s.biome == b) {
                let mid = g
                    .track
                    .segs
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.biome == b)
                    .map(|(i, _)| i)
                    .nth(40)
                    .unwrap_or(i + 5);
                g.pos = mid as f32 * 200.0;
                g.x = -0.15;
                g.speed = MAX_SPEED * 0.8;
                g.biome_w = [0.0; turbocursa::NB];
                g.biome_w[b as usize] = 1.0;
                let mut fb = vec![0u32; W * H];
                for _ in 0..3 {
                    render::render(&g, &mut fb);
                }
                let mut f =
                    std::fs::File::create(format!("frames/biome_{}_{}.png", b, names[b as usize]))
                        .unwrap();
                f.write_all(&png(&fb, 1)).unwrap();
                got[b as usize] = true;
            }
        }
        if got.iter().all(|&x| x) {
            break;
        }
    }
    println!("{:?}", got);
}
