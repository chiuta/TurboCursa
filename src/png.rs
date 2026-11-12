//! Minimal PNG writer (stored deflate blocks, no dependencies) for the photo mode.

fn crc32(data: &[u8], mut c: u32) -> u32 {
    c = !c;
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

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let c = crc32(data, crc32(kind, 0));
    out.extend_from_slice(&c.to_be_bytes());
}

/// Encodes a 0x00RRGGBB frame as an 8-bit RGB PNG.
pub fn encode(fb: &[u32], w: usize, h: usize) -> Vec<u8> {
    assert_eq!(fb.len(), w * h);
    let mut raw = Vec::with_capacity((w * 3 + 1) * h);
    for row in fb.chunks(w) {
        raw.push(0); // filter: none
        for &c in row {
            raw.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
        }
    }
    // zlib stream of stored blocks
    let mut z = vec![0x78, 0x01];
    let mut blocks = raw.chunks(65535).peekable();
    while let Some(b) = blocks.next() {
        z.push(blocks.peek().is_none() as u8);
        z.extend_from_slice(&(b.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(b.len() as u16)).to_le_bytes());
        z.extend_from_slice(b);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &x in &raw {
        a = (a + x as u32) % 65521;
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
