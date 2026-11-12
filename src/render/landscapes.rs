use super::*;

// ---------- generative landscapes, part two ----------

pub(super) fn hsv(h: f32, s: f32, v: f32) -> u32 {
    let h = h.rem_euclid(1.0) * 6.0;
    let (i, f) = (h as i32, h - (h as i32) as f32);
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// Per-segment ground colour: lava veins, rainbow road, neon grid, caustic sand, glowing spores.
pub(super) fn ground_color(biome: u8, pp: &Pal, band: usize, tone: f32, idx: usize, t: f32) -> u32 {
    let base = shade(pp.grass[band], 0.84 + 0.32 * tone);
    match biome {
        8 => {
            let v = fractal::fbm(idx as f32 * 0.09, band as f32 * 0.5, 41, 3, 0);
            let lava = ((v - 0.52) * 7.0).clamp(0.0, 1.0);
            let pulse = 0.8 + 0.2 * (t * 3.0 + idx as f32 * 0.3).sin();
            mix(
                base,
                rgb(255, (110.0 + 80.0 * tone) as u8, 20),
                lava * pulse,
            )
        }
        9 => shade(base, 0.9 + 0.12 * (idx as f32 * 0.7 + t * 2.2).sin()),
        10 => mix(base, hsv(idx as f32 * 0.022 + t * 0.15, 0.8, 0.55), 0.7),
        11 => match idx % 4 {
            0 => rgb(255, 60, 200),
            1 => rgb(40, 18, 80),
            _ => base,
        },
        12 => mix(
            base,
            rgb(90, 255, 200),
            (((tone * 17.0 + idx as f32).sin() - 0.92) * 9.0).clamp(0.0, 0.8),
        ),
        13 => {
            // impossible: the ground tiles flip between two hues like a chessboard in motion
            let chk = ((idx / 3) + band).is_multiple_of(2);
            mix(
                base,
                hsv(0.62 + 0.1 * (t * 0.2).sin(), 0.5, 0.6),
                if chk { 0.5 } else { 0.1 },
            )
        }
        14 => mix(base, hsv(idx as f32 * 0.017 + t * 0.05, 0.35, 0.98), 0.45),
        15 => {
            let plate = idx.is_multiple_of(6) as i32 as f32;
            mix(base, rgb(210, 160, 70), plate * 0.5)
        }
        16 => shade(
            base,
            0.96 + 0.06 * fractal::fbm(idx as f32 * 0.05, band as f32, 77, 3, 0),
        ),
        17 => mix(
            base,
            rgb(255, 120, 30),
            (((idx as f32 * 0.31 + t * 1.5).sin() - 0.96) * 14.0).clamp(0.0, 0.6),
        ),
        18 => {
            let v = fractal::fbm(idx as f32 * 0.07, band as f32 * 0.5, 91, 4, 0);
            mix(base, hsv(0.04 + v * 0.1, 0.8, 0.9), 0.35)
        }
        _ => base,
    }
}

/// Thick line made of dots (used by the recursive coral and the fractal lightning).
#[allow(clippy::too_many_arguments)]
pub(super) fn line(fb: &mut [u32], x0: f32, y0: f32, x1: f32, y1: f32, th: f32, c: u32, clip: f32) {
    let n = ((x1 - x0).abs().max((y1 - y0).abs())).ceil().max(1.0) as i32;
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
        if th <= 1.0 {
            if y < clip && x >= 0.0 && y >= 0.0 && (x as usize) < vw() && (y as usize) < vh() {
                fb[y as usize * vw() + x as usize] = c;
            }
        } else {
            rect(
                fb,
                x - th / 2.0,
                y - th / 2.0,
                x + th / 2.0,
                y + th / 2.0,
                c,
                clip,
            );
        }
    }
}

/// Recursive coral: every branch forks into two or three thinner, shorter ones.
#[allow(clippy::too_many_arguments)]
pub(super) fn coral(
    fb: &mut [u32],
    x: f32,
    y: f32,
    len: f32,
    ang: f32,
    th: f32,
    depth: u32,
    seed: u32,
    c: u32,
    clip: f32,
) {
    if depth == 0 || len < 1.0 {
        ellipse(
            fb,
            x,
            y,
            th * 1.6 + 1.0,
            th * 1.6 + 1.0,
            shade(c, 1.25),
            clip,
        );
        return;
    }
    let (nx, ny) = (x + ang.sin() * len, y - ang.cos() * len);
    line(fb, x, y, nx, ny, th, c, clip);
    let kids = 2 + (h01(seed, depth) > 0.55) as u32;
    for i in 0..kids {
        let spread =
            (i as f32 / (kids - 1) as f32 - 0.5) * 1.15 + (h01(seed + i, depth + 9) - 0.5) * 0.3;
        coral(
            fb,
            nx,
            ny,
            len * 0.72,
            ang + spread,
            (th * 0.75).max(1.0),
            depth - 1,
            seed.wrapping_mul(31).wrapping_add(i + 7),
            c,
            clip,
        );
    }
}

/// Sierpinski triangle of the given depth drawn with neon edges.
#[allow(clippy::too_many_arguments)]
pub(super) fn sierp(
    fb: &mut [u32],
    cx: f32,
    ytop: f32,
    ybot: f32,
    half: f32,
    depth: u32,
    c: u32,
    clip: f32,
) {
    if depth == 0 || half < 1.0 {
        tri(fb, cx, ytop, ybot, half, c, clip);
        return;
    }
    let (h2, hh) = (half / 2.0, (ybot - ytop) / 2.0);
    sierp(fb, cx, ytop, ytop + hh, h2, depth - 1, c, clip);
    sierp(
        fb,
        cx - h2,
        ytop + hh,
        ybot,
        h2,
        depth - 1,
        shade(c, 0.8),
        clip,
    );
    sierp(
        fb,
        cx + h2,
        ytop + hh,
        ybot,
        h2,
        depth - 1,
        shade(c, 0.8),
        clip,
    );
}

/// Sierpinski carpet: eight sub-squares around an empty centre.
#[allow(clippy::too_many_arguments)]
pub(super) fn carpet(fb: &mut [u32], x: f32, y: f32, size: f32, depth: u32, c: u32, clip: f32) {
    if depth == 0 || size < 2.0 {
        rect(fb, x, y, x + size, y + size, c, clip);
        return;
    }
    let s3 = size / 3.0;
    for j in 0..3 {
        for i in 0..3 {
            if i == 1 && j == 1 {
                continue;
            }
            let sh = if (i + j) % 2 == 0 { 1.0 } else { 0.78 };
            carpet(
                fb,
                x + i as f32 * s3,
                y + j as f32 * s3,
                s3,
                depth - 1,
                shade(c, sh),
                clip,
            );
        }
    }
}

/// Fractal lightning: midpoint displacement with random side branches.
#[allow(clippy::too_many_arguments)]
pub(super) fn bolt(
    fb: &mut [u32],
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    depth: u32,
    seed: u32,
    a: f32,
    clip: f32,
) {
    if depth == 0 {
        for d in 0..2 {
            let c = if d == 0 {
                [150.0, 170.0, 255.0]
            } else {
                [255.0, 255.0, 255.0]
            };
            let n = ((x1 - x0).abs().max((y1 - y0).abs())).ceil().max(1.0) as i32;
            for i in 0..=n {
                let t = i as f32 / n as f32;
                let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
                if y < clip {
                    add_px(
                        fb,
                        x as i32 + d,
                        y as i32,
                        c,
                        a * (if d == 0 { 0.6 } else { 1.0 }),
                    );
                }
            }
        }
        return;
    }
    let (dx, dy) = (x1 - x0, y1 - y0);
    let len = (dx * dx + dy * dy).sqrt();
    let off = (h01(seed, depth) - 0.5) * len * 0.45;
    let (mx, my) = (
        (x0 + x1) / 2.0 - dy / len.max(1e-3) * off,
        (y0 + y1) / 2.0 + dx / len.max(1e-3) * off,
    );
    bolt(
        fb,
        x0,
        y0,
        mx,
        my,
        depth - 1,
        seed.wrapping_mul(7).wrapping_add(1),
        a,
        clip,
    );
    bolt(
        fb,
        mx,
        my,
        x1,
        y1,
        depth - 1,
        seed.wrapping_mul(7).wrapping_add(2),
        a,
        clip,
    );
    if depth >= 3 && h01(seed, depth + 40) > 0.55 {
        let sgn = if h01(seed, depth + 50) > 0.5 {
            1.0
        } else {
            -1.0
        };
        bolt(
            fb,
            mx,
            my,
            mx + sgn * len * 0.6 + dx * 0.2,
            my + len * 0.5,
            depth - 2,
            seed.wrapping_add(99),
            a * 0.6,
            clip,
        );
    }
}

/// Volcano with a glowing crater, a fractal smoke plume, embers and lightning.
pub(super) fn sky_volcano(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    let cx = vw() as f32 * 0.62 - g.sky_off * 0.12 * k;
    let cx = (cx).rem_euclid(vw() as f32 + 300.0 * k) - 150.0 * k;
    let (h, w) = (74.0 * k, 120.0 * k);
    glow(
        fb,
        cx,
        hz - h,
        120.0 * k,
        90.0 * k,
        rgb(255, 110, 30),
        0.55 * a,
        hz,
    );
    for y in ((hz - h) as i32)..hz as i32 {
        let t = (hz - y as f32) / h;
        let hw = lerp(w, 9.0 * k, t.powf(0.8)) * (0.9 + 0.2 * fractal::fbm(t * 9.0, 3.0, 8, 3, 0));
        for x in ((cx - hw) as i32).max(0)..((cx + hw) as i32).min(vw() as i32) {
            let dxn = (x as f32 - cx) / hw;
            let lava = ((fractal::fbm(dxn * 6.0, t * 8.0, 5, 3, 0) - 0.62) * 6.0).clamp(0.0, 1.0)
                * (t * 1.4).min(1.0);
            blend_px(fb, x, y, mix(rgb(34, 22, 26), rgb(255, 120, 30), lava), a);
        }
    }
    // smoke plume: fractal fBm column that billows and drifts
    for y in 0..(hz - h) as i32 {
        let t = (hz - h - y as f32) / (hz - h).max(1.0);
        let spread = (14.0 + 70.0 * t) * k;
        for x in
            ((cx - spread) as i32).max(0)..((cx + spread + 40.0 * k * t) as i32).min(vw() as i32)
        {
            let n = fractal::fbm(
                x as f32 * 0.03,
                y as f32 * 0.03 - g.elapsed * 0.35,
                12,
                4,
                0,
            );
            let dens = ((n - 0.42) * 3.2 * (1.0 - ((x as f32 - cx) / spread).abs()).max(0.0))
                .clamp(0.0, 0.85);
            if dens > 0.02 {
                blend_px(
                    fb,
                    x,
                    y,
                    mix(rgb(40, 34, 38), rgb(150, 70, 50), (1.0 - t) * 0.6),
                    dens * a,
                );
            }
        }
    }
    // lightning every few seconds, with a screen flash
    let slot = (g.elapsed / 2.6) as u32;
    let ph = g.elapsed / 2.6 - slot as f32;
    if ph < 0.1 && h01(slot, 5) > 0.35 {
        let fl = 1.0 - ph / 0.1;
        let x0 = h01(slot, 6) * vw() as f32;
        bolt(
            fb,
            x0,
            0.0,
            x0 + (h01(slot, 7) - 0.5) * 80.0 * k,
            hz - 6.0 * k,
            6,
            slot.wrapping_mul(977),
            a * fl,
            hz,
        );
        for y in 0..hz as i32 {
            for x in (0..vw() as i32).step_by(2) {
                add_px(fb, x, y, [80.0, 90.0, 140.0], 0.18 * fl * a);
            }
        }
    }
    let _ = p;
}

/// Underwater: god rays, caustic shimmer and drifting manta silhouettes.
pub(super) fn sky_abyss(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    for i in 0..7u32 {
        let x0 = (h01(i, 3) * (vw() as f32 + 200.0 * k) - g.sky_off * 0.2 * k)
            .rem_euclid(vw() as f32 + 200.0 * k)
            - 100.0 * k;
        let wid = (10.0 + 20.0 * h01(i, 4)) * k;
        let pulse = 0.5 + 0.5 * (g.elapsed * 0.7 + i as f32 * 1.9).sin();
        for y in 0..hz as i32 {
            let t = y as f32 / hz;
            let shift = t * 70.0 * k;
            let hw = wid * (0.5 + t);
            for x in ((x0 + shift - hw) as i32).max(0)..((x0 + shift + hw) as i32).min(vw() as i32)
            {
                let edge = 1.0 - ((x as f32 - x0 - shift) / hw).abs();
                add_px(
                    fb,
                    x,
                    y,
                    [150.0, 235.0, 255.0],
                    0.11 * edge * (1.0 - t * 0.7) * pulse * a,
                );
            }
        }
    }
    for i in 0..3u32 {
        let x = (h01(i, 21) * (vw() as f32 + 300.0 * k) - g.elapsed * (6.0 + 5.0 * i as f32) * k)
            .rem_euclid(vw() as f32 + 300.0 * k)
            - 150.0 * k;
        let y = hz * (0.25 + 0.35 * h01(i, 22));
        let s = (14.0 + 16.0 * h01(i, 23)) * k;
        let c = mix(rgb(8, 40, 70), p.sky_hor, 0.4);
        ellipse_a(fb, x, y, s * 1.6, s * 0.35, c, 0.85 * a, hz);
        tri(
            fb,
            x - s * 1.9,
            y - 2.0 * k,
            y + 3.0 * k,
            s * 0.5,
            mix(c, p.sky_hor, 0.1),
            hz,
        );
        let flap = (g.elapsed * 2.0 + i as f32).sin() * s * 0.25;
        ellipse_a(fb, x, y + flap, s * 0.8, s * 0.22, c, 0.8 * a, hz);
    }
    for i in 0..60u32 {
        let x = (h01(i, 31) * vw() as f32 + (g.elapsed * 3.0 + i as f32).sin() * 5.0 * k)
            .rem_euclid(vw() as f32);
        let y = (hz - ((g.elapsed * (6.0 + 8.0 * h01(i, 32)) + h01(i, 33) * hz) % hz)).max(0.0);
        add_px(fb, x as i32, y as i32, [200.0, 245.0, 255.0], 0.5 * a);
    }
}

/// Cosmos: a Julia-set nebula (generated at start-up), dense stars and a milky band.
pub(super) fn sky_cosmos(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    let jt = &g.assets.julia;
    let off = (g.sky_off * 0.05 * k) as i32;
    for y in 0..hz as i32 {
        let sy = ((y as f32 / hz) * jt.h as f32) as usize;
        for x in 0..vw() as i32 {
            let sx =
                (((x + off).rem_euclid(vw() as i32) as f32 / vw() as f32) * jt.w as f32) as usize;
            let c = jt.px[sy.min(jt.h - 1) * jt.w + sx.min(jt.w - 1)];
            let al = (c >> 24) as f32 / 255.0 * 0.85 * a;
            if al > 0.01 {
                add_px(fb, x, y, unpack(c & 0xFF_FFFF), al);
            }
        }
    }
    for i in 0..240u32 {
        let sx = (h01(i, 61) * vw() as f32 - g.sky_off * 0.12 * k).rem_euclid(vw() as f32);
        let sy = h01(i, 62) * hz * 0.95;
        let tw = 0.5 + 0.5 * (g.elapsed * 2.0 + i as f32).sin();
        add_px(fb, sx as i32, sy as i32, [255.0, 255.0, 255.0], a * tw);
    }
    let _ = p;
}

/// Neon citadel: striped synthwave sun and a skyline of Sierpinski pyramids and towers.
pub(super) fn sky_citadel(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    let (sx, sy, sr) = (
        vw() as f32 * 0.5 - g.sky_off * 0.04 * k,
        hz * 0.62,
        46.0 * k,
    );
    glow(
        fb,
        sx,
        sy,
        sr * 2.0,
        sr * 1.6,
        rgb(255, 60, 160),
        0.45 * a,
        hz,
    );
    for y in ((sy - sr) as i32)..((sy + sr).min(hz) as i32) {
        let t = (y as f32 - (sy - sr)) / (2.0 * sr);
        let cut = ((t * 22.0).fract() < 0.18 + 0.5 * t) && t > 0.35;
        if cut {
            continue;
        }
        let hw = (sr * sr - (y as f32 - sy) * (y as f32 - sy))
            .max(0.0)
            .sqrt();
        let c = mix(rgb(255, 236, 90), rgb(255, 40, 170), t);
        for x in ((sx - hw) as i32).max(0)..((sx + hw) as i32).min(vw() as i32) {
            blend_px(fb, x, y, c, a);
        }
    }
    let span = vw() as f32 + 300.0 * k;
    for i in 0..9u32 {
        let near = h01(i, 81);
        let cx =
            (h01(i, 83) * span - g.sky_off * (0.2 + 0.2 * near) * k).rem_euclid(span) - 150.0 * k;
        let c = mix(rgb(30, 10, 60), p.sky_hor, 0.25 - 0.18 * near);
        let neon = if i % 2 == 0 {
            rgb(80, 240, 255)
        } else {
            rgb(255, 70, 210)
        };
        if i % 3 == 0 {
            let hh = (30.0 + 26.0 * near) * k;
            sierp(
                fb,
                cx,
                hz - hh,
                hz,
                hh * 0.62,
                3,
                mix(c, neon, 0.25 * a),
                hz,
            );
        } else {
            let (w, hh) = ((8.0 + 10.0 * near) * k, (24.0 + 40.0 * h01(i, 85)) * k);
            rect(fb, cx - w, hz - hh, cx + w, hz, c, hz);
            rect(
                fb,
                cx - w,
                hz - hh,
                cx + w,
                hz - hh + 1.5 * k,
                mix(c, neon, 0.8 * a),
                hz,
            );
            for r in 0..5 {
                if h01(i * 7 + r, 88) > 0.4 {
                    rect(
                        fb,
                        cx - w + 2.0 * k,
                        hz - hh + (6.0 + r as f32 * 8.0) * k,
                        cx - w + 4.0 * k,
                        hz - hh + (8.0 + r as f32 * 8.0) * k,
                        mix(c, rgb(255, 230, 120), 0.8 * a),
                        hz,
                    );
                }
            }
        }
    }
}

/// Mycelium jungle: far-off giant mushrooms and drifting spore haze.
pub(super) fn sky_myco(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    let span = vw() as f32 + 300.0 * k;
    for i in 0..7u32 {
        let near = h01(i, 101);
        let cx =
            (h01(i, 103) * span - g.sky_off * (0.2 + 0.25 * near) * k).rem_euclid(span) - 150.0 * k;
        let sh = (40.0 + 60.0 * near) * k;
        let haze = 0.5 - 0.35 * near;
        let hue = [rgb(80, 255, 200), rgb(180, 110, 255), rgb(255, 120, 190)][(i % 3) as usize];
        let stem = mix(rgb(70, 60, 90), p.sky_hor, haze);
        rect(fb, cx - 4.0 * k, hz - sh, cx + 4.0 * k, hz, stem, hz);
        let cap = mix(shade(hue, 0.55), p.sky_hor, haze);
        ellipse_a(
            fb,
            cx,
            hz - sh,
            (20.0 + 16.0 * near) * k,
            (11.0 + 7.0 * near) * k,
            cap,
            0.95 * a,
            hz,
        );
        glow(
            fb,
            cx,
            hz - sh + 4.0 * k,
            36.0 * k,
            18.0 * k,
            hue,
            0.35 * a,
            hz,
        );
    }
}

/// Ambient particles that make each landscape feel alive: snow, embers, bubbles, spores, stardust.
pub(super) fn weather(fb: &mut [u32], g: &Game) {
    let w = &g.biome_w;
    let kinds: [(usize, [f32; 3], f32, f32); 8] = [
        (14, [255.0, 240.0, 255.0], 0.5, 0.4),
        (15, [255.0, 190.0, 90.0], 0.55, -0.6),
        (17, [255.0, 140.0, 60.0], 0.5, 0.8),
        (7, [255.0, 255.0, 255.0], 0.9, 1.0),
        (8, [255.0, 150.0, 50.0], 0.8, -1.0),
        (9, [190.0, 240.0, 255.0], 0.55, -0.7),
        (12, [120.0, 255.0, 200.0], 0.7, -0.35),
        (10, [255.0, 200.0, 255.0], 0.6, 0.5),
    ];
    let (vwf, vhf, k) = (vw() as f32, vh() as f32, vk());
    for (b, col, str_, dir) in kinds {
        if w[b] < 0.05 {
            continue;
        }
        for i in 0..110u32 {
            let spd = (14.0 + 40.0 * h01(i, 201)) * dir * k;
            let y = (h01(i, 202) * vhf + g.elapsed * spd).rem_euclid(vhf);
            let sway = (g.elapsed * (0.6 + h01(i, 203)) + i as f32).sin() * 8.0 * k;
            let x = (h01(i, 204) * vwf + sway - g.sky_off * 0.3 * k * h01(i, 205)).rem_euclid(vwf);
            let sz = 1 + (h01(i, 206) > 0.85) as i32;
            for d in 0..sz {
                add_px(fb, x as i32 + d, y as i32, col, str_ * w[b]);
            }
        }
    }
}

/// Blackout mode: everything outside the headlight cone fades to near-darkness.
pub(super) fn blackout(fb: &mut [u32], g: &Game) {
    let (w, h) = (vw(), vh());
    let cx = w as f32 / 2.0 + g.steer * 6.0 * vk();
    let cy = h as f32 * 0.72;
    let (rx, ry) = (w as f32 * 0.42, h as f32 * 0.34);
    for y in 0..h {
        let dy = (y as f32 - cy) / ry;
        for x in 0..w {
            let dx = (x as f32 - cx) / rx;
            let d = (dx * dx + dy * dy).sqrt();
            let lit = (1.0 - d).clamp(0.0, 1.0);
            let f = 0.035 + 0.965 * lit * lit.sqrt();
            let c = fb[y * w + x];
            let (r, gg, b) = (
                ((c >> 16) & 255) as f32 * f,
                ((c >> 8) & 255) as f32 * f,
                (c & 255) as f32 * f,
            );
            fb[y * w + x] = rgb(r as u8, gg as u8, b as u8);
        }
    }
}

/// Impossible inverted world: a second mountain range hangs upside-down from the top of the sky, two moons.
pub(super) fn sky_inverted(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz, w) = (vk(), horizon() as f32, vw());
    for x in 0..w {
        let u = (x as f32 + g.sky_off * 0.3 * k) / (90.0 * k);
        let h = (0.25 + 0.75 * fractal::ridged(u, 3.3, 211, 4, 0)) * hz * 0.5;
        let c = mix(p.mount, p.sky_hor, 0.35);
        let mut y = 0.0;
        while y < h {
            let t = y / h.max(1.0);
            let col = mix(shade(c, 1.2), c, t);
            let i = y as usize * w + x;
            if i < fb.len() {
                fb[i] = mix(fb[i], col, 0.9 * a);
            }
            y += 1.0;
        }
    }
    for (i, (cx, r, c)) in [
        (0.28f32, 22.0f32, rgb(255, 236, 210)),
        (0.7, 13.0, rgb(180, 210, 255)),
    ]
    .into_iter()
    .enumerate()
    {
        let x = (w as f32 * cx - g.sky_off * (0.1 + 0.05 * i as f32) * k).rem_euclid(w as f32);
        glow(fb, x, hz * 0.42, r * 3.0 * k, r * 3.0 * k, c, 0.3 * a, hz);
        ellipse_a(fb, x, hz * 0.42, r * k, r * k, c, 0.95 * a, hz);
    }
}

/// Prism desert: spectral aurora bands that refract across the whole sky.
pub(super) fn sky_prism(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz, w) = (vk(), horizon() as f32, vw());
    for x in (0..w).step_by(2) {
        let u = (x as f32 + g.sky_off * 0.4 * k) / (140.0 * k);
        let wob = fractal::fbm(u, g.elapsed * 0.05, 311, 3, 0);
        for band in 0..6 {
            let cy = hz * (0.12 + 0.1 * band as f32) + wob * 24.0 * k;
            let col = hsv(band as f32 / 6.0 + g.elapsed * 0.02, 0.6, 1.0);
            for dy in -3i32..=3 {
                let al = 0.22 * a * (1.0 - dy.abs() as f32 / 4.0);
                add_px(
                    fb,
                    x as i32,
                    (cy + dy as f32 * k) as i32,
                    [
                        ((col >> 16) & 255) as f32,
                        ((col >> 8) & 255) as f32,
                        (col & 255) as f32,
                    ],
                    al,
                );
                add_px(
                    fb,
                    x as i32 + 1,
                    (cy + dy as f32 * k) as i32,
                    [
                        ((col >> 16) & 255) as f32,
                        ((col >> 8) & 255) as f32,
                        (col & 255) as f32,
                    ],
                    al,
                );
            }
        }
    }
    let _ = p;
}

/// Clockwork world: huge recursive gears turning in the sky.
pub(super) fn sky_clockwork(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    let span = vw() as f32 + 260.0 * k;
    for i in 0..5u32 {
        let near = h01(i, 401);
        let cx =
            (h01(i, 403) * span - g.sky_off * (0.1 + 0.2 * near) * k).rem_euclid(span) - 130.0 * k;
        let r = (34.0 + 46.0 * near) * k;
        let cy = hz * (0.25 + 0.4 * h01(i, 405));
        let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
        let rot = g.elapsed * 0.25 * dir + i as f32;
        let col = mix(rgb(52, 30, 12), p.sky_hor, 0.05 + 0.2 * (1.0 - near));
        let teeth = 10 + (i as usize % 4) * 3;
        for n in 0..teeth {
            let ang = rot + n as f32 / teeth as f32 * std::f32::consts::TAU;
            let (x1, y1) = (cx + ang.cos() * r * 1.15, cy + ang.sin() * r * 1.15);
            line(
                fb,
                cx + ang.cos() * r * 0.85,
                cy + ang.sin() * r * 0.85,
                x1,
                y1,
                3.0 * k,
                col,
                hz,
            );
            // recursive satellite gear
            let (sx, sy) = (cx + ang.cos() * r * 0.5, cy + ang.sin() * r * 0.5);
            if n % 3 == 0 {
                ellipse_a(fb, sx, sy, r * 0.12, r * 0.12, shade(col, 0.8), 0.8 * a, hz);
            }
        }
        ellipse_a(fb, cx, cy, r * 0.25, r * 0.25, shade(col, 0.7), 0.9 * a, hz);
        // rim and spokes: a recognisable gear silhouette
        let rim = shade(col, 1.25);
        let steps = 48;
        for n in 0..steps {
            let (a0, a1) = (n as f32 / steps as f32, (n + 1) as f32 / steps as f32);
            let (t0, t1) = (
                a0 * std::f32::consts::TAU + rot,
                a1 * std::f32::consts::TAU + rot,
            );
            line(
                fb,
                cx + t0.cos() * r * 0.85,
                cy + t0.sin() * r * 0.85,
                cx + t1.cos() * r * 0.85,
                cy + t1.sin() * r * 0.85,
                2.0 * k,
                rim,
                hz,
            );
        }
        for n in 0..6 {
            let ang = n as f32 / 6.0 * std::f32::consts::TAU + rot;
            line(
                fb,
                cx,
                cy,
                cx + ang.cos() * r * 0.85,
                cy + ang.sin() * r * 0.85,
                2.5 * k,
                col,
                hz,
            );
        }
    }
}

/// Sea of clouds: an fBm cloud floor under a bright sky with distant spires piercing it.
pub(super) fn sky_cloudsea(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz, w) = (vk(), horizon() as f32, vw());
    let top = (hz * 0.72) as i32;
    for y in top..hz as i32 {
        let t = (y - top) as f32 / (hz - top as f32).max(1.0);
        for x in (0..w).step_by(1) {
            let u = (x as f32 + g.sky_off * 0.3 * k) / (60.0 * k);
            let n = fractal::fbm(u, y as f32 * 0.09 / k, 511, 4, 0);
            let edge = ((n - 0.45 + t * 0.35) * 5.0).clamp(0.0, 1.0);
            let c = mix(rgb(255, 236, 226), rgb(236, 238, 255), n);
            let i = y as usize * w + x;
            fb[i] = mix(fb[i], c, edge * a);
        }
    }
    for i in 0..4u32 {
        let cx = (h01(i, 503) * (w as f32 + 200.0 * k) - g.sky_off * 0.18 * k)
            .rem_euclid(w as f32 + 200.0 * k)
            - 100.0 * k;
        let sh = (30.0 + 40.0 * h01(i, 505)) * k;
        let c = mix(p.mount, p.sky_hor, 0.5);
        for y in 0..sh as i32 {
            let half = (2.0 + y as f32 * 0.12 * (1.0 - y as f32 / sh)) * k;
            rect(
                fb,
                cx - half,
                top as f32 - y as f32,
                cx + half,
                top as f32 - y as f32 + 1.0,
                c,
                hz,
            );
        }
    }
}

/// Black hole: gravitationally lensed accretion ring and a dark disc over a red-shifted sky.
pub(super) fn sky_blackhole(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz) = (vk(), horizon() as f32);
    let cx = (vw() as f32 * 0.6 - g.sky_off * 0.06 * k).rem_euclid(vw() as f32);
    let cy = hz * 0.5;
    let r = 26.0 * k;
    glow(
        fb,
        cx,
        cy,
        r * 5.0,
        r * 3.0,
        rgb(255, 150, 60),
        0.35 * a,
        hz,
    );
    for n in 0..360 {
        let ang = n as f32 / 360.0 * std::f32::consts::TAU + g.elapsed * 0.4;
        let heat = 0.5 + 0.5 * (ang * 3.0).sin();
        for rr in [1.5f32, 1.7, 1.95, 2.3, 2.8] {
            let (x, y) = (
                cx + ang.cos() * r * rr * 1.9,
                cy + ang.sin() * r * rr * 0.35,
            );
            add_px(
                fb,
                x as i32,
                y as i32,
                [255.0, 150.0 + 90.0 * heat, 60.0 + 100.0 * heat],
                0.35 * a / rr,
            );
        }
        // lensed arc: the far side of the disc bent over the top
        let (x, y) = (cx + ang.cos() * r * 1.6, cy - (ang.sin().abs()) * r * 1.4);
        add_px(fb, x as i32, y as i32, [255.0, 200.0, 140.0], 0.3 * a);
    }
    ellipse_a(fb, cx, cy, r, r, rgb(0, 0, 0), a, hz);
    let _ = p;
}

/// Fractal canyon: layered strata cliffs carved by ridged noise, banded like a Mandelbrot escape plot.
pub(super) fn sky_canyon(fb: &mut [u32], g: &Game, p: &Pal, a: f32) {
    if a < 0.02 {
        return;
    }
    let (k, hz, w) = (vk(), horizon() as f32, vw());
    for layer in 0..3u32 {
        let f = layer as f32;
        for x in 0..w {
            let u = (x as f32 + g.sky_off * (0.15 + 0.15 * f) * k) / ((70.0 - 12.0 * f) * k);
            let h = (0.35 + 0.6 * fractal::ridged(u, 5.0 + f * 7.0, 611 + layer, 4, 0))
                * hz
                * (0.45 - 0.1 * f);
            let mut y = hz - h;
            while y < hz {
                let strata = ((y / k * 0.35 + f * 3.0).sin() * 0.5 + 0.5) * 0.25;
                let col = mix(
                    shade(p.mount, 0.8 + strata + 0.12 * f),
                    p.sky_hor,
                    0.5 - 0.17 * f,
                );
                let i = y as usize * w + x;
                if i < fb.len() {
                    fb[i] = mix(fb[i], col, a);
                }
                y += 1.0;
            }
        }
    }
}

/// National-colour livery painted over the car body (before the tail lights are drawn).
pub(super) fn livery(
    fb: &mut [u32],
    cx: f32,
    by: f32,
    w: f32,
    h: f32,
    car: &crate::cars::CarSpec,
    clip: f32,
) {
    use crate::cars::Layout;
    let (x0, x1) = (cx - w * 0.5, cx + w * 0.5);
    let (y0, y1) = (by - h * 0.64, by - h * 0.2);
    let (c2, c3) = (car.c2, car.c3);
    match car.layout {
        Layout::Horizontal => {
            let t = (y1 - y0) / 3.0;
            rect(fb, x0, y0 + t, x1, y0 + 2.0 * t, c2, clip);
            rect(fb, x0, y0 + 2.0 * t, x1, y1, c3, clip);
        }
        Layout::Vertical => {
            let t = w / 3.0;
            rect(fb, x0 + t, y0, x0 + 2.0 * t, y1, c2, clip);
            rect(fb, x0 + 2.0 * t, y0, x1, y1, c3, clip);
        }
        Layout::Cross => {
            rect(fb, cx - w * 0.2, y0, cx - w * 0.07, y1, c3, clip);
            rect(
                fb,
                x0,
                (y0 + y1) / 2.0 - h * 0.05,
                x1,
                (y0 + y1) / 2.0 + h * 0.05,
                c3,
                clip,
            );
            rect(fb, cx - w * 0.17, y0, cx - w * 0.1, y1, c2, clip);
            rect(
                fb,
                x0,
                (y0 + y1) / 2.0 - h * 0.025,
                x1,
                (y0 + y1) / 2.0 + h * 0.025,
                c2,
                clip,
            );
        }
        Layout::Diagonal => {
            quad(fb, y0, x0, x0 + w * 0.3, y1, x1 - w * 0.3, x1, clip, c2);
            quad(
                fb,
                y0,
                x0 + w * 0.3,
                x0 + w * 0.4,
                y1,
                x1 - w * 0.2,
                x1 - w * 0.3,
                clip,
                c3,
            );
        }
        Layout::Disc => {
            let (mx, my) = (cx, (y0 + y1) / 2.0);
            ellipse(fb, mx, my, w * 0.2, h * 0.2, c3, clip);
            ellipse(fb, mx, my, w * 0.15, h * 0.15, c2, clip);
        }
        Layout::Triangle => {
            let rows = 8;
            for i in 0..rows {
                let t = i as f32 / rows as f32;
                let ya = y0 + (y1 - y0) * t;
                let yb = y0 + (y1 - y0) * (t + 1.0 / rows as f32);
                let half = (0.5 - (t - 0.5).abs()) * 2.0;
                rect(fb, x0, ya, x0 + w * 0.45 * half.max(0.1), yb, c2, clip);
                rect(fb, x0, ya, x0 + w * 0.2 * half, yb, c3, clip);
            }
        }
    }
}

/// Draws a country's car large (garage screen) on a framebuffer of the given size.
pub fn draw_car_preview(
    fb: &mut [u32],
    w: usize,
    h: usize,
    cx: f32,
    by: f32,
    width: f32,
    car: &crate::cars::CarSpec,
) {
    DIM.with(|d| d.set((w, h)));
    let body = if car.is_default() {
        rgb(214, 28, 36)
    } else {
        car.body
    };
    LIVERY.with(|l| l.set((!car.is_default()).then_some(*car)));
    if car.class.is_special() {
        draw_special(fb, cx, by, width * 0.85, car.class, 0.3, 1.0, h as f32);
        LIVERY.with(|l| l.set(None));
        return;
    }
    draw_vehicle(
        fb,
        cx,
        by,
        width
            * view_scale(car.class).min(1.0)
            * if class_kind(car.class) == K_BIKE {
                0.66
            } else {
                0.85
            },
        body,
        h as f32,
        car.class != crate::cars::VehicleClass::Sedan,
        class_kind(car.class),
    );
    LIVERY.with(|l| l.set(None));
    plate(fb, cx, by, width, car);
}

/// Number plate with the country code on the rear bumper.
pub(super) fn plate(fb: &mut [u32], cx: f32, by: f32, w: f32, car: &crate::cars::CarSpec) {
    // the plate sits on a car-style bumper: skip the other body types
    if !matches!(class_kind(car.class), K_CAR | K_MICRO | K_LIMO) || car.is_default() || w < 40.0 {
        return;
    }
    let code = crate::career::COUNTRIES
        [(car.country as usize).min(crate::career::COUNTRIES.len() - 1)]
    .code;
    let h = w * 0.58;
    let (y0, y1) = (by - h * 0.42, by - h * 0.25);
    rect(
        fb,
        cx - w * 0.13,
        y0,
        cx + w * 0.13,
        y1,
        rgb(236, 236, 240),
        vh() as f32,
    );
    let sc = ((w / 84.0) as i32).max(1);
    let tw = font::text_width(code, sc);
    let th = 5 * sc;
    text(
        fb,
        cx as i32 - tw / 2,
        ((y0 + y1) / 2.0) as i32 - th / 2,
        code,
        1,
        rgb(20, 24, 60),
    );
}
