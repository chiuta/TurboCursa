use super::*;

// ---------- cities, jams, radar ----------

/// Segments drawn as city streets: the cities, and the detour street while it is driven.
pub(super) fn city_like(g: &Game, seg: usize) -> bool {
    g.city.in_city(seg)
        || g.detour
            .is_some_and(|ji| g.city.jams[ji].a <= seg && seg < g.city.jams[ji].b)
}

pub(super) fn hash3(a: u32, b: u32, c: u32) -> u32 {
    let mut h =
        a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77) ^ c.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 12)
}

/// Facades, kerbs, lamps, zebra crossings, traffic lights and pedestrians of one segment, plus the
/// slip road in front of a jam.
pub(super) fn draw_city_seg(
    fb: &mut [u32],
    g: &Game,
    sp: &SegProj,
    idx: usize,
    n: usize,
    sky: u32,
) {
    let sg = idx % n;
    let city = city_like(g, sg);
    let exit = g.city.exit_at(sg);
    if !city && exit.is_none() {
        return;
    }
    let vwf = vw() as f32;
    let (w1, w2) = (sp.s1 * ROAD_W * vwf / 2.0, sp.s2 * ROAD_W * vwf / 2.0);
    let fog_k = (((CAM_DEPTH / sp.s1) - 9000.0) / (DRAW_DIST as f32 * SEG_LEN - 9000.0))
        .clamp(0.0, 1.0)
        .powf(1.3)
        * 0.7;
    let fogf = |c: u32| mix(c, sky, fog_k);
    let clip = sp.clip;
    if let Some(ji) = exit {
        // slip road: asphalt flaring out to the right, white edge, green sign with an arrow
        let j = g.city.jams[ji];
        let (f1, f2) = (
            (idx - j.exit_a()) as f32 / crate::city::EXIT_LEN as f32,
            (idx + 1 - j.exit_a()) as f32 / crate::city::EXIT_LEN as f32,
        );
        let off = |f: f32| 1.02 + 1.5 * f.clamp(0.0, 1.0).sqrt();
        poly4(
            fb,
            [
                (sp.x1 + w1 * 1.02, sp.y1),
                (sp.x1 + w1 * off(f1), sp.y1),
                (sp.x2 + w2 * off(f2), sp.y2),
                (sp.x2 + w2 * 1.02, sp.y2),
            ],
            clip,
            fogf(rgb(70, 72, 80)),
        );
        poly4(
            fb,
            [
                (sp.x1 + w1 * (off(f1) - 0.06), sp.y1),
                (sp.x1 + w1 * off(f1), sp.y1),
                (sp.x2 + w2 * off(f2), sp.y2),
                (sp.x2 + w2 * (off(f2) - 0.06), sp.y2),
            ],
            clip,
            fogf(rgb(235, 235, 240)),
        );
        if (idx - j.exit_a()) % 12 == 3 {
            // painted arrow
            let cx = sp.x1 + w1 * (off(f1) + 1.02) * 0.5;
            tri(
                fb,
                cx,
                sp.y1 - w1 * 0.32,
                sp.y1,
                w1 * 0.1,
                fogf(rgb(235, 235, 240)),
                clip,
            );
        }
        if idx == j.exit_a() + 6 || idx == j.exit_a() + 30 {
            let u = sp.s1 * vwf / 2.0;
            let px = sp.x1 + w1 * 1.35;
            rect(
                fb,
                px - 25.0 * u,
                sp.y1 - 1100.0 * u,
                px + 25.0 * u,
                sp.y1,
                fogf(rgb(200, 200, 206)),
                clip,
            );
            rect(
                fb,
                px - 330.0 * u,
                sp.y1 - 1500.0 * u,
                px + 330.0 * u,
                sp.y1 - 900.0 * u,
                fogf(rgb(20, 130, 70)),
                clip,
            );
            tri(
                fb,
                px,
                sp.y1 - 1420.0 * u,
                sp.y1 - 980.0 * u,
                170.0 * u,
                fogf(rgb(245, 245, 250)),
                clip,
            );
        }
    }
    if !city {
        return;
    }
    let blk = (idx / 7) as u32;
    for side in [-1.0f32, 1.0] {
        let sd = if side < 0.0 { 0 } else { 1 };
        // pavement with a bright kerb
        poly4(
            fb,
            [
                (sp.x1 + side * w1 * 1.12, sp.y1),
                (sp.x1 + side * w1 * 1.55, sp.y1),
                (sp.x2 + side * w2 * 1.55, sp.y2),
                (sp.x2 + side * w2 * 1.12, sp.y2),
            ],
            clip,
            fogf(rgb(148, 148, 156)),
        );
        poly4(
            fb,
            [
                (sp.x1 + side * w1 * 1.12, sp.y1),
                (sp.x1 + side * w1 * 1.17, sp.y1),
                (sp.x2 + side * w2 * 1.17, sp.y2),
                (sp.x2 + side * w2 * 1.12, sp.y2),
            ],
            clip,
            fogf(rgb(205, 205, 212)),
        );
        // building block
        let h = hash3(blk, sd, 7);
        let hb = 2.4 + (h % 7) as f32 * 0.75;
        let pal = [
            rgb(150, 112, 96),
            rgb(120, 130, 150),
            rgb(170, 160, 130),
            rgb(104, 118, 112),
            rgb(158, 126, 140),
            rgb(132, 120, 108),
        ];
        let wall = pal[(h >> 8) as usize % pal.len()];
        let shade_k = if idx.is_multiple_of(2) { 1.0 } else { 0.94 };
        let (xa, xb) = (sp.x1 + side * w1 * 1.55, sp.x2 + side * w2 * 1.55);
        poly4(
            fb,
            [
                (xa, sp.y1),
                (xb, sp.y2),
                (xb, sp.y2 - hb * w2),
                (xa, sp.y1 - hb * w1),
            ],
            clip,
            fogf(shade(wall, shade_k)),
        );
        // roof edge
        poly4(
            fb,
            [
                (xa, sp.y1 - hb * w1),
                (xb, sp.y2 - hb * w2),
                (xb, sp.y2 - hb * w2 - 0.12 * w2),
                (xa, sp.y1 - hb * w1 - 0.12 * w1),
            ],
            clip,
            fogf(shade(wall, 0.7)),
        );
        // windows
        if idx.is_multiple_of(2) {
            let rows = ((hb - 0.9) / 0.8) as i32;
            for r in 0..rows {
                let (h0, h1) = (0.55 + r as f32 * 0.8, 0.55 + r as f32 * 0.8 + 0.45);
                let lit = hash3(blk, r as u32, sd + 3 * idx as u32) % 100 < 28;
                let col = if lit {
                    rgb(255, 222, 130)
                } else {
                    rgb(44, 64, 96)
                };
                let pt = |t: f32, hh: f32| {
                    (
                        lerp(xa, xb, t),
                        lerp(sp.y1, sp.y2, t) - hh * lerp(w1, w2, t),
                    )
                };
                poly4(
                    fb,
                    [pt(0.2, h0), pt(0.8, h0), pt(0.8, h1), pt(0.2, h1)],
                    clip,
                    fogf(col),
                );
            }
        }
        // front face of the block, where it starts
        if idx.is_multiple_of(7) {
            let xf = sp.x1 + side * w1 * 3.4;
            poly4(
                fb,
                [
                    (xa, sp.y1),
                    (xf, sp.y1),
                    (xf, sp.y1 - hb * w1),
                    (xa, sp.y1 - hb * w1),
                ],
                clip,
                fogf(shade(wall, 0.72)),
            );
        }
        // street lamp
        if idx.is_multiple_of(5) {
            let u = sp.s1 * vwf / 2.0;
            let px = sp.x1 + side * w1 * 1.33;
            rect(
                fb,
                px - 14.0 * u,
                sp.y1 - 760.0 * u,
                px + 14.0 * u,
                sp.y1,
                fogf(rgb(70, 74, 84)),
                clip,
            );
            rect(
                fb,
                px - side * 150.0 * u - 70.0 * u,
                sp.y1 - 780.0 * u,
                px + 20.0 * u,
                sp.y1 - 740.0 * u,
                fogf(rgb(70, 74, 84)),
                clip,
            );
            glow(
                fb,
                px - side * 150.0 * u,
                sp.y1 - 730.0 * u,
                160.0 * u,
                120.0 * u,
                rgb(255, 230, 160),
                0.35,
                clip,
            );
        }
    }
    // crosswalks, lights, pedestrians
    for l in &g.city.lights {
        if (l.z / SEG_LEN) as usize != sg {
            continue;
        }
        let u = sp.s1 * vwf / 2.0;
        for j in 0..9 {
            let (a, b) = (-0.9 + j as f32 * 0.225, -0.9 + j as f32 * 0.225 + 0.12);
            poly4(
                fb,
                [
                    (sp.x1 + w1 * a, sp.y1),
                    (sp.x1 + w1 * b, sp.y1),
                    (sp.x2 + w2 * b, sp.y2),
                    (sp.x2 + w2 * a, sp.y2),
                ],
                clip,
                fogf(rgb(232, 232, 236)),
            );
        }
        if l.lit {
            let st = l.state(g.elapsed);
            for side in [1.0f32, -1.0] {
                let px = sp.x1 + side * w1 * 1.33;
                rect(
                    fb,
                    px - 20.0 * u,
                    sp.y1 - 1000.0 * u,
                    px + 20.0 * u,
                    sp.y1,
                    fogf(rgb(60, 62, 70)),
                    clip,
                );
                // mast arm over the lane
                let tipx = px - side * 1500.0 * u;
                rect(
                    fb,
                    px.min(tipx),
                    sp.y1 - 1020.0 * u,
                    px.max(tipx),
                    sp.y1 - 980.0 * u,
                    fogf(rgb(60, 62, 70)),
                    clip,
                );
                rect(
                    fb,
                    tipx - 90.0 * u,
                    sp.y1 - 1380.0 * u,
                    tipx + 90.0 * u,
                    sp.y1 - 980.0 * u,
                    fogf(rgb(24, 24, 28)),
                    clip,
                );
                for (i, col) in [rgb(255, 50, 40), rgb(255, 200, 40), rgb(60, 230, 90)]
                    .into_iter()
                    .enumerate()
                {
                    let on = (i == 0 && st == 2) || (i == 1 && st == 1) || (i == 2 && st == 0);
                    let ly = sp.y1 - 1300.0 * u + i as f32 * 130.0 * u;
                    let c = if on { col } else { shade(col, 0.2) };
                    rect(
                        fb,
                        tipx - 50.0 * u,
                        ly - 45.0 * u,
                        tipx + 50.0 * u,
                        ly + 45.0 * u,
                        fogf(c),
                        clip,
                    );
                    if on {
                        glow(fb, tipx, ly, 260.0 * u, 260.0 * u, col, 0.55, clip);
                    }
                }
            }
        }
        if let Some(px) = l.ped_x(g.elapsed) {
            for (i, off) in [0.0f32, 0.1, -0.12].into_iter().enumerate() {
                let x = sp.x1 + w1 * (px + off * l.dir);
                let cols = [rgb(220, 70, 60), rgb(70, 130, 220), rgb(240, 200, 60)];
                let step = ((g.elapsed * 6.0 + i as f32) as i32 % 2) as f32 * 30.0;
                rect(
                    fb,
                    x - 50.0 * u,
                    sp.y1 - 190.0 * u,
                    x - 8.0 * u + step * u * 0.2,
                    sp.y1,
                    fogf(rgb(40, 44, 70)),
                    clip,
                );
                rect(
                    fb,
                    x + 8.0 * u - step * u * 0.2,
                    sp.y1 - 190.0 * u,
                    x + 50.0 * u,
                    sp.y1,
                    fogf(rgb(40, 44, 70)),
                    clip,
                );
                rect(
                    fb,
                    x - 60.0 * u,
                    sp.y1 - 360.0 * u,
                    x + 60.0 * u,
                    sp.y1 - 190.0 * u,
                    fogf(cols[i % 3]),
                    clip,
                );
                rect(
                    fb,
                    x - 42.0 * u,
                    sp.y1 - 450.0 * u,
                    x + 42.0 * u,
                    sp.y1 - 360.0 * u,
                    fogf(rgb(232, 190, 150)),
                    clip,
                );
            }
        }
    }
}

/// Heading-up map of the road ahead: cities in blue, the jam in red, the detour in green, traffic
/// lights as dots, with the time to cross the next jam on the main road and on the exit.
pub(super) fn draw_radar(fb: &mut [u32], g: &Game) {
    let k = |v: f32| v * vk();
    let (x0, y0, x1, y1) = (
        vw() as f32 - k(70.0),
        k(70.0),
        vw() as f32 - k(4.0),
        k(130.0),
    );
    let hf = vh() as f32;
    rect_a(fb, x0, y0, x1, y1, rgb(6, 12, 26), 0.62, hf);
    rect_a(fb, x0, y0, x1, y0 + 1.0, rgb(90, 200, 255), 0.7, hf);
    rect_a(fb, x0, y1 - 1.0, x1, y1, rgb(90, 200, 255), 0.7, hf);
    rect_a(fb, x0, y0, x0 + 1.0, y1, rgb(90, 200, 255), 0.7, hf);
    rect_a(fb, x1 - 1.0, y0, x1, y1, rgb(90, 200, 255), 0.7, hf);
    let n = g.track.segs.len();
    let s0 = ((g.pos + PLAYER_Z) / SEG_LEN) as usize;
    let (ox, oy) = ((x0 + x1) / 2.0, y1 - k(8.0));
    let step = k(0.30);
    let ahead = 330usize;
    let (mut px, mut py, mut th) = (0.0f32, 0.0f32, 0.0f32);
    let mut prev = (ox, oy);
    let jam_next = g.city.next_jam(s0).map(|(i, j)| (i, *j));
    for i in 0..ahead {
        let sg = s0 + i;
        th += g.track.segs[sg % n].curve * 0.0045;
        px += th.sin() * step;
        py -= th.cos() * step;
        let (sx, sy) = (ox + px, oy + py);
        if sy < y0 + 1.0 {
            break;
        }
        if sx < x0 + 1.0 || sx > x1 - 1.0 {
            prev = (f32::NAN, sy);
            continue;
        }
        let m = sg % n;
        let col = if g.city.jam_at(m).is_some() {
            rgb(255, 70, 60)
        } else if g.city.in_city(m) {
            rgb(90, 170, 255)
        } else {
            rgb(190, 196, 210)
        };
        let th_px = if g.city.jam_at(m).is_some() { 2.4 } else { 1.4 };
        if prev.0.is_finite() {
            line(fb, prev.0, prev.1, sx, sy, th_px, col, hf);
        }
        // the detour runs beside the jam, in green
        if let Some((_, j)) = jam_next {
            if m >= j.exit_a() && m < j.b {
                let (nx, ny) = (th.cos(), th.sin());
                let f = if m < j.a {
                    (m - j.exit_a()) as f32 / crate::city::EXIT_LEN as f32
                } else {
                    1.0
                };
                let f = if m >= j.b.saturating_sub(20) {
                    (j.b - m) as f32 / 20.0
                } else {
                    f
                };
                let d = k(5.0) * f.clamp(0.0, 1.0);
                if sx + nx * d < x1 - 1.0 {
                    line(
                        fb,
                        sx + nx * d,
                        sy + ny * d,
                        sx + nx * d + 0.1,
                        sy + ny * d,
                        1.4,
                        rgb(90, 240, 120),
                        hf,
                    );
                }
            }
        }
        for (ai, ar) in g.areas.iter().enumerate() {
            if ar.a == m && g.a_state[ai] < 2 {
                let c = if ar.kind == crate::bonus::PROVING {
                    rgb(255, 150, 40)
                } else {
                    rgb(80, 150, 255)
                };
                rect(fb, sx - 2.0, sy - 2.0, sx + 3.0, sy + 3.0, c, hf);
            }
        }
        if let Some(ms) = g.mission {
            let t = match g.m_state {
                crate::mission::WAITING => Some((ms.pick, rgb(255, 210, 40))),
                crate::mission::CARRYING => Some((ms.drop, rgb(70, 235, 120))),
                _ => None,
            };
            if let Some((ts, c)) = t {
                if ts == m {
                    rect(fb, sx - 2.0, sy - 2.0, sx + 3.0, sy + 3.0, c, hf);
                }
            }
        }
        if let Some(pu) = &g.pursuit {
            if (pu.z / SEG_LEN) as usize % n == m {
                let blink = (g.elapsed * 6.0) as i32 % 2 == 0;
                let c = if blink {
                    rgb(255, 60, 50)
                } else {
                    rgb(255, 190, 60)
                };
                rect(fb, sx - 3.0, sy - 3.0, sx + 4.0, sy + 4.0, c, hf);
            }
        }
        for l in &g.city.lights {
            if (l.z / SEG_LEN) as usize == m && l.lit {
                let c = [rgb(70, 230, 100), rgb(255, 200, 50), rgb(255, 60, 50)]
                    [l.state(g.elapsed) as usize];
                rect(
                    fb,
                    sx + k(1.5),
                    sy - 1.0,
                    sx + k(1.5) + 2.0,
                    sy + 1.0,
                    c,
                    hf,
                );
            }
        }
        prev = (sx, sy);
    }
    // own car
    tri(
        fb,
        ox,
        oy - k(3.0),
        oy + k(2.0),
        k(2.2),
        rgb(255, 220, 60),
        hf,
    );
    if g.detour.is_some() {
        text_shadow(
            fb,
            x0 as i32 + 3,
            y0 as i32 + 3,
            i18n::banner("DETOUR"),
            1,
            rgb(90, 240, 120),
        );
    } else if let Some((_, j)) = jam_next {
        let d = (j.exit_a() as f32 * SEG_LEN - (g.pos + PLAYER_Z)).max(0.0);
        if d < 60000.0 || s0 >= j.exit_a() {
            let a = format!("{} {:.0}S", i18n::banner("MAIN"), j.eta_main());
            let b = format!("{} {:.0}S", i18n::banner("EXIT"), j.eta_exit());
            text_shadow(fb, x0 as i32 + 3, y0 as i32 + 3, &a, 1, rgb(255, 110, 100));
            text_shadow(
                fb,
                x0 as i32 + 3,
                y0 as i32 + 3 + 7 * vk() as i32,
                &b,
                1,
                rgb(90, 240, 120),
            );
        }
    } else if g.city.zone_at(s0).is_some() {
        text_shadow(
            fb,
            x0 as i32 + 3,
            y0 as i32 + 3,
            i18n::banner("CITY"),
            1,
            rgb(90, 170, 255),
        );
    }
}

/// Mission panel: parcel or passenger symbol and the time left.
pub(super) fn draw_mission_hud(fb: &mut [u32], g: &Game) {
    let Some(ms) = g.mission else {
        return;
    };
    if g.m_state != crate::mission::CARRYING {
        return;
    }
    let k = |v: f32| v * vk();
    let hf = vh() as f32;
    let (x0, y0) = (k(3.0), k(96.0));
    rect_a(
        fb,
        x0,
        y0,
        x0 + k(46.0),
        y0 + k(13.0),
        rgb(8, 10, 24),
        0.55,
        hf,
    );
    let late = g.m_t <= 0.0;
    let col = if late || (g.m_t < 10.0 && (g.elapsed * 4.0) as i32 % 2 == 0) {
        rgb(255, 80, 70)
    } else {
        rgb(255, 255, 255)
    };
    if ms.kind == crate::mission::PARCEL {
        rect(
            fb,
            x0 + k(3.0),
            y0 + k(4.0),
            x0 + k(11.0),
            y0 + k(11.0),
            rgb(200, 150, 80),
            hf,
        );
        rect(
            fb,
            x0 + k(6.5),
            y0 + k(4.0),
            x0 + k(7.5),
            y0 + k(11.0),
            rgb(240, 220, 160),
            hf,
        );
    } else {
        ellipse(
            fb,
            x0 + k(7.0),
            y0 + k(5.0),
            k(2.0),
            k(2.0),
            rgb(240, 200, 160),
            hf,
        );
        rect(
            fb,
            x0 + k(4.5),
            y0 + k(7.0),
            x0 + k(9.5),
            y0 + k(12.0),
            rgb(90, 140, 240),
            hf,
        );
    }
    let t = g.m_t.abs().ceil() as i32;
    text_shadow(
        fb,
        (x0 + k(15.0)) as i32,
        (y0 + k(3.0)) as i32,
        &format!("{}{}", if late { "-" } else { "" }, t),
        2,
        col,
    );
}

/// Gradient of the road in a proving ground, in per cent.
pub(super) fn draw_slope(fb: &mut [u32], g: &Game) {
    let n = g.track.segs.len();
    let seg = ((g.pos + PLAYER_Z) / SEG_LEN) as usize % n;
    if !g
        .areas
        .iter()
        .any(|a| a.kind == crate::bonus::PROVING && seg >= a.a && seg < a.b)
    {
        return;
    }
    let sg = &g.track.segs[seg];
    let pct = ((sg.y1 - sg.y0) / SEG_LEN * 100.0).round() as i32;
    if pct.abs() < 4 {
        return;
    }
    let k = |v: f32| v * vk();
    let col = if pct > 0 {
        rgb(255, 170, 60)
    } else {
        rgb(90, 220, 255)
    };
    rect_a(
        fb,
        k(3.0),
        k(112.0),
        k(40.0),
        k(125.0),
        rgb(8, 10, 24),
        0.55,
        vh() as f32,
    );
    text_shadow(
        fb,
        k(7.0) as i32,
        k(115.0) as i32,
        &format!("{:+}%", pct),
        2,
        col,
    );
}

/// Speed-limit sign while in a city.
pub(super) fn draw_limit_sign(fb: &mut [u32], g: &Game) {
    let seg = ((g.pos + PLAYER_Z) / SEG_LEN) as usize % g.track.segs.len();
    if !g.city.in_city(seg) {
        if g.civic_mult() > 1 {
            let k = |v: f32| v * vk();
            text_shadow(
                fb,
                k(6.0) as i32,
                k(68.0) as i32,
                &format!("x{}", g.civic_mult()),
                2,
                rgb(90, 240, 120),
            );
        }
        return;
    }
    let k = |v: f32| v * vk();
    let (cx, cy, r) = (k(18.0), k(78.0), k(11.0));
    let hf = vh() as f32;
    let over = g.speed > MAX_SPEED * crate::city::LIMIT * 1.12 && (g.elapsed * 6.0) as i32 % 2 == 0;
    ellipse(
        fb,
        cx,
        cy,
        r,
        r,
        if over {
            rgb(255, 255, 80)
        } else {
            rgb(220, 30, 30)
        },
        hf,
    );
    ellipse(fb, cx, cy, r * 0.76, r * 0.76, rgb(250, 250, 250), hf);
    let kmh = (crate::city::LIMIT * 260.0).round() as i32;
    let s = format!("{}", kmh);
    let w = font::text_width(&s, 1) as f32;
    text(
        fb,
        (cx - w / 2.0) as i32,
        (cy - k(1.2)) as i32,
        &s,
        1,
        rgb(10, 10, 14),
    );
    if g.civic_mult() > 1 {
        text_shadow(
            fb,
            (cx - k(8.0)) as i32,
            (cy + r + k(3.0)) as i32,
            &format!("x{}", g.civic_mult()),
            2,
            rgb(90, 240, 120),
        );
    }
}

/// Flashing light bar (red and blue alternating) over a vehicle.
pub(super) fn light_bar(fb: &mut [u32], cx: f32, y: f32, w: f32, t: f32, clip: f32) {
    let ph = (t * 5.0) as i32 % 2 == 0;
    let (a, b) = if ph {
        (rgb(255, 40, 40), rgb(60, 110, 255))
    } else {
        (rgb(60, 110, 255), rgb(255, 40, 40))
    };
    rect(fb, cx - w * 0.24, y - w * 0.06, cx, y, a, clip);
    rect(fb, cx, y - w * 0.06, cx + w * 0.24, y, b, clip);
    glow(fb, cx - w * 0.2, y, w * 0.5, w * 0.3, a, 0.6, clip);
    glow(fb, cx + w * 0.2, y, w * 0.5, w * 0.3, b, 0.6, clip);
}

/// Police car from behind with an alternating light bar.
pub(super) fn draw_police(fb: &mut [u32], cx: f32, by: f32, w: f32, t: f32, clip: f32) {
    draw_car(fb, cx, by, w, rgb(238, 240, 246), clip, false, false);
    rect(
        fb,
        cx - w * 0.5,
        by - w * 0.30,
        cx + w * 0.5,
        by - w * 0.22,
        rgb(20, 24, 40),
        clip,
    );
    let fl = (t * 9.0) as i32 % 2 == 0;
    let (a, b) = if fl {
        (rgb(255, 40, 40), rgb(60, 110, 255))
    } else {
        (rgb(60, 110, 255), rgb(255, 40, 40))
    };
    let ly = by - w * 0.66;
    rect(fb, cx - w * 0.22, ly - w * 0.05, cx, ly + w * 0.02, a, clip);
    rect(fb, cx, ly - w * 0.05, cx + w * 0.22, ly + w * 0.02, b, clip);
    glow(fb, cx - w * 0.12, ly, w * 0.5, w * 0.3, a, 0.5, clip);
    glow(fb, cx + w * 0.12, ly, w * 0.5, w * 0.3, b, 0.5, clip);
}

/// A service vehicle driven by the player: police, fire engine, ambulance or hovering helicopter.
pub(super) fn draw_special(
    fb: &mut [u32],
    cx: f32,
    by: f32,
    w: f32,
    class: crate::cars::VehicleClass,
    t: f32,
    speed_k: f32,
    clip: f32,
) {
    use crate::cars::VehicleClass as V;
    match class {
        V::Police => draw_police(fb, cx, by, w, t, clip),
        V::Fire => draw_emergency(fb, cx, by, w * 0.9, 2, t, clip),
        V::Ambulance => draw_emergency(fb, cx, by, w * 0.95, 1, t, clip),
        _ => {
            // rises as it speeds up; skids stay down while it is parked
            let alt = 120.0 + 520.0 * speed_k.clamp(0.0, 1.0) + 40.0 * (t * 3.0).sin();
            let u = w / 700.0;
            draw_heli(
                fb,
                cx,
                by,
                u,
                alt * (speed_k.clamp(0.0, 1.0)).max(0.0) + 90.0,
                t,
                clip,
                true,
            );
        }
    }
}

/// Ambulance (kind 1) or fire engine (kind 2) from behind.
pub(super) fn draw_emergency(
    fb: &mut [u32],
    cx: f32,
    by: f32,
    w: f32,
    kind: u8,
    t: f32,
    clip: f32,
) {
    if w < 1.5 {
        return;
    }
    if kind == 1 {
        let w = w * 1.12;
        draw_car(fb, cx, by, w, rgb(240, 240, 244), clip, false, true);
        let h = w * 0.58;
        rect(
            fb,
            cx - w * 0.5,
            by - h * 0.62,
            cx + w * 0.5,
            by - h * 0.5,
            rgb(220, 40, 40),
            clip,
        );
        rect(
            fb,
            cx - w * 0.07,
            by - h * 0.95,
            cx + w * 0.07,
            by - h * 0.7,
            rgb(220, 40, 40),
            clip,
        );
        rect(
            fb,
            cx - w * 0.17,
            by - h * 0.86,
            cx + w * 0.17,
            by - h * 0.79,
            rgb(220, 40, 40),
            clip,
        );
        light_bar(fb, cx, by - h * 1.16, w, t, clip);
    } else {
        let w = w * 1.28;
        draw_car(fb, cx, by, w, rgb(200, 30, 30), clip, false, true);
        let h = w * 0.58;
        // ladder on the roof and a yellow reflective band
        rect(
            fb,
            cx - w * 0.36,
            by - h * 1.3,
            cx + w * 0.36,
            by - h * 1.16,
            rgb(190, 192, 200),
            clip,
        );
        for i in 0..5 {
            let x = cx - w * 0.32 + i as f32 * w * 0.16;
            rect(
                fb,
                x,
                by - h * 1.3,
                x + w * 0.02,
                by - h * 1.16,
                rgb(120, 122, 132),
                clip,
            );
        }
        rect(
            fb,
            cx - w * 0.5,
            by - h * 0.5,
            cx + w * 0.5,
            by - h * 0.42,
            rgb(250, 220, 60),
            clip,
        );
        light_bar(fb, cx, by - h * 1.34, w, t, clip);
    }
}

/// Helicopter seen from behind and below; `alt` is the height of the skids above the ground in
/// world units (0: landed with the rotor slowing down).
pub(super) fn draw_heli(
    fb: &mut [u32],
    cx: f32,
    by: f32,
    u: f32,
    alt: f32,
    t: f32,
    clip: f32,
    light: bool,
) {
    let y = by - alt * u;
    let body = rgb(215, 215, 222);
    ellipse(fb, cx, y - 260.0 * u, 330.0 * u, 190.0 * u, body, clip);
    rect(
        fb,
        cx - 330.0 * u,
        y - 300.0 * u,
        cx + 330.0 * u,
        y - 250.0 * u,
        rgb(220, 40, 40),
        clip,
    );
    ellipse(
        fb,
        cx,
        y - 300.0 * u,
        160.0 * u,
        100.0 * u,
        rgb(40, 70, 110),
        clip,
    );
    // tail boom with its rotor
    rect(
        fb,
        cx - 38.0 * u,
        y - 560.0 * u,
        cx + 38.0 * u,
        y - 330.0 * u,
        shade(body, 0.8),
        clip,
    );
    rect(
        fb,
        cx - 18.0 * u,
        y - 640.0 * u,
        cx + 18.0 * u,
        y - 520.0 * u,
        rgb(220, 40, 40),
        clip,
    );
    // skids
    rect(
        fb,
        cx - 380.0 * u,
        y - 70.0 * u,
        cx + 380.0 * u,
        y - 40.0 * u,
        rgb(40, 42, 50),
        clip,
    );
    rect(
        fb,
        cx - 250.0 * u,
        y - 140.0 * u,
        cx - 220.0 * u,
        y - 60.0 * u,
        rgb(40, 42, 50),
        clip,
    );
    rect(
        fb,
        cx + 220.0 * u,
        y - 140.0 * u,
        cx + 250.0 * u,
        y - 60.0 * u,
        rgb(40, 42, 50),
        clip,
    );
    // main rotor: a blurred disc, its two blades visible on alternate frames
    let spin = (t * 24.0) as i32 % 2 == 0;
    ellipse_a(
        fb,
        cx,
        y - 470.0 * u,
        900.0 * u,
        55.0 * u,
        rgb(30, 30, 40),
        if spin { 0.5 } else { 0.28 },
        clip,
    );
    rect(
        fb,
        cx - 26.0 * u,
        y - 490.0 * u,
        cx + 26.0 * u,
        y - 440.0 * u,
        rgb(60, 62, 70),
        clip,
    );
    if light {
        glow(
            fb,
            cx,
            y + 30.0 * u,
            700.0 * u,
            500.0 * u,
            rgb(255, 250, 200),
            0.35,
            clip,
        );
    }
    glow(
        fb,
        cx - 260.0 * u,
        y - 300.0 * u,
        160.0 * u,
        100.0 * u,
        if (t * 3.0) as i32 % 2 == 0 {
            rgb(255, 40, 40)
        } else {
            rgb(60, 60, 60)
        },
        0.6,
        clip,
    );
}

/// Accident scenes and overtaking ambulances / fire engines on one segment.
pub(super) fn draw_emergency_seg(fb: &mut [u32], g: &Game, sp: &SegProj, idx: usize, n: usize) {
    let sg = idx % n;
    let vwf = vw() as f32;
    let pz = g.pos + PLAYER_Z;
    for &z0 in &g.city.scenes {
        if z0 > g.pos + DRAW_DIST as f32 * SEG_LEN || z0 + 3000.0 < g.pos {
            continue;
        }
        // cones along the lane edge
        for c in 0..9 {
            let zc = z0 - 1600.0 + c as f32 * 450.0;
            if (zc / SEG_LEN) as usize != sg {
                continue;
            }
            let pct = (zc % SEG_LEN) / SEG_LEN;
            let s = lerp(sp.s1, sp.s2, pct);
            let cx = lerp(sp.x1, sp.x2, pct) + s * 0.28 * ROAD_W * vwf / 2.0;
            let by = lerp(sp.y1, sp.y2, pct);
            let u = s * vwf / 2.0;
            tri(
                fb,
                cx,
                by - 260.0 * u,
                by,
                80.0 * u,
                rgb(255, 120, 30),
                sp.clip,
            );
            rect(
                fb,
                cx - 60.0 * u,
                by - 150.0 * u,
                cx + 60.0 * u,
                by - 110.0 * u,
                rgb(240, 240, 240),
                sp.clip,
            );
        }
        for &(dz, x, _, kind) in &crate::city::SCENE {
            let zo = z0 + dz;
            if (zo / SEG_LEN) as usize != sg {
                continue;
            }
            let pct = (zo % SEG_LEN) / SEG_LEN;
            let s = lerp(sp.s1, sp.s2, pct);
            let cx = lerp(sp.x1, sp.x2, pct) + s * x * ROAD_W * vwf / 2.0;
            let by = lerp(sp.y1, sp.y2, pct);
            let u = s * vwf / 2.0;
            if kind == 0 {
                // the helicopter comes down while the player approaches
                let d = z0 - pz;
                let alt = 2600.0 * ((d - 9000.0) / 22000.0).clamp(0.0, 1.0);
                draw_heli(fb, cx, by, u, alt, g.elapsed, sp.clip, alt > 200.0);
            } else {
                draw_emergency(fb, cx, by, s * CAR_W * vwf / 2.0, kind, g.elapsed, sp.clip);
            }
        }
    }
    for ar in &g.areas {
        let (w1, w2) = (sp.s1 * ROAD_W * vwf / 2.0, sp.s2 * ROAD_W * vwf / 2.0);
        let u = sp.s1 * vwf / 2.0;
        if ar.kind == crate::bonus::PARKING && sg >= ar.a && sg < ar.b {
            use crate::bonus::LOT_EDGE;
            let band = (sg / 3) % 2;
            poly4(
                fb,
                [
                    (sp.x1 + w1 * LOT_EDGE, sp.y1),
                    (sp.x1 + w1 * 2.5, sp.y1),
                    (sp.x2 + w2 * 2.5, sp.y2),
                    (sp.x2 + w2 * LOT_EDGE, sp.y2),
                ],
                sp.clip,
                shade(rgb(78, 80, 90), 0.92 + 0.08 * band as f32),
            );
            // slot lines: across every third segment, one lengthwise divider
            if (sg - ar.a).is_multiple_of(3) {
                poly4(
                    fb,
                    [
                        (sp.x1 + w1 * 1.2, sp.y1),
                        (sp.x1 + w1 * 2.2, sp.y1),
                        (sp.x1 + w1 * 2.2, sp.y1 - 3.0_f32.max(w1 * 0.015)),
                        (sp.x1 + w1 * 1.2, sp.y1 - 3.0_f32.max(w1 * 0.015)),
                    ],
                    sp.clip,
                    rgb(235, 235, 240),
                );
            }
            rect(
                fb,
                sp.x1 + w1 * 1.65 - w1 * 0.01,
                sp.y2,
                sp.x1 + w1 * 1.65 + w1 * 0.01,
                sp.y1,
                rgb(235, 235, 240),
                sp.clip,
            );
        }
        if sg + 2 == ar.a {
            // gantry over the road announcing the area
            let col = if ar.kind == crate::bonus::PROVING {
                rgb(255, 140, 30)
            } else {
                rgb(40, 110, 230)
            };
            for side in [-1.0f32, 1.0] {
                let px = sp.x1 + side * w1 * 1.25;
                rect(
                    fb,
                    px - 30.0 * u,
                    sp.y1 - 1700.0 * u,
                    px + 30.0 * u,
                    sp.y1,
                    rgb(190, 192, 200),
                    sp.clip,
                );
            }
            rect(
                fb,
                sp.x1 - w1 * 1.25,
                sp.y1 - 1900.0 * u,
                sp.x1 + w1 * 1.25,
                sp.y1 - 1500.0 * u,
                col,
                sp.clip,
            );
            if ar.kind == crate::bonus::PARKING {
                // a white "P"
                let (cx, y) = (sp.x1, sp.y1 - 1700.0 * u);
                rect(
                    fb,
                    cx - 110.0 * u,
                    y - 150.0 * u,
                    cx - 50.0 * u,
                    y + 150.0 * u,
                    rgb(255, 255, 255),
                    sp.clip,
                );
                rect(
                    fb,
                    cx - 110.0 * u,
                    y - 150.0 * u,
                    cx + 110.0 * u,
                    y - 90.0 * u,
                    rgb(255, 255, 255),
                    sp.clip,
                );
                rect(
                    fb,
                    cx + 50.0 * u,
                    y - 150.0 * u,
                    cx + 110.0 * u,
                    y + 20.0 * u,
                    rgb(255, 255, 255),
                    sp.clip,
                );
                rect(
                    fb,
                    cx - 110.0 * u,
                    y - 30.0 * u,
                    cx + 110.0 * u,
                    y + 30.0 * u,
                    rgb(255, 255, 255),
                    sp.clip,
                );
            } else {
                for k in 0..6 {
                    let x = sp.x1 - w1 * 1.0 + k as f32 * w1 * 0.4;
                    rect(
                        fb,
                        x,
                        sp.y1 - 1880.0 * u,
                        x + w1 * 0.2,
                        sp.y1 - 1520.0 * u,
                        if k % 2 == 0 { rgb(30, 30, 34) } else { col },
                        sp.clip,
                    );
                }
            }
        }
    }
    if let Some(ms) = g.mission {
        let target = match g.m_state {
            crate::mission::WAITING => Some((ms.pick_z(), rgb(255, 210, 40))),
            crate::mission::CARRYING => Some((ms.drop_z(), rgb(70, 235, 120))),
            _ => None,
        };
        if let Some((z, col)) = target {
            let (wseg, c0) = (7usize, (z / SEG_LEN) as usize);
            if sg + wseg >= c0 && sg <= c0 + wseg {
                let (w1, w2) = (sp.s1 * ROAD_W * vwf / 2.0, sp.s2 * ROAD_W * vwf / 2.0);
                let bay = mix(rgb(40, 70, 130), col, 0.35 + 0.15 * (g.elapsed * 4.0).sin());
                poly4(
                    fb,
                    [
                        (sp.x1 + w1 * 0.25, sp.y1),
                        (sp.x1 + w1 * 1.0, sp.y1),
                        (sp.x2 + w2 * 1.0, sp.y2),
                        (sp.x2 + w2 * 0.25, sp.y2),
                    ],
                    sp.clip,
                    bay,
                );
            }
            if sg == c0 {
                let u = sp.s1 * vwf / 2.0;
                let cx = sp.x1 + sp.s1 * 0.7 * ROAD_W * vwf / 2.0;
                let ph = 0.5 + 0.5 * (g.elapsed * 3.0).sin();
                for k in 0..6 {
                    let y = sp.y1 - 400.0 * u * (k as f32 + ph);
                    ellipse_a(
                        fb,
                        cx,
                        y,
                        330.0 * u,
                        70.0 * u,
                        col,
                        0.55 - 0.07 * k as f32,
                        sp.clip,
                    );
                }
                rect_a(
                    fb,
                    cx - 40.0 * u,
                    sp.y1 - 3200.0 * u,
                    cx + 40.0 * u,
                    sp.y1,
                    col,
                    0.35,
                    sp.clip,
                );
                glow(
                    fb,
                    cx,
                    sp.y1 - 3200.0 * u,
                    500.0 * u,
                    400.0 * u,
                    col,
                    0.5,
                    sp.clip,
                );
            }
        }
    }
    for e in &g.emerg {
        if (e.z / SEG_LEN) as usize != sg {
            continue;
        }
        let pct = (e.z % SEG_LEN) / SEG_LEN;
        let s = lerp(sp.s1, sp.s2, pct);
        let cx = lerp(sp.x1, sp.x2, pct) + s * e.x * ROAD_W * vwf / 2.0;
        let by = lerp(sp.y1, sp.y2, pct);
        draw_emergency(
            fb,
            cx,
            by,
            s * CAR_W * vwf / 2.0,
            1 + e.kind,
            g.elapsed,
            sp.clip,
        );
    }
}

/// Traffic helicopter hovering over the road ahead of the player in cities and jams.
pub(super) fn draw_patrol(fb: &mut [u32], g: &Game, proj: &[SegProj], base: usize, n: usize) {
    let pz = g.pos + PLAYER_Z;
    let seg0 = (pz / SEG_LEN) as usize % n;
    let busy = g.city.in_city(seg0)
        || g.detour.is_some()
        || g.city
            .next_jam(seg0)
            .is_some_and(|(_, j)| seg0 + 300 >= j.exit_a() && seg0 < j.b);
    if !busy || g.city.scenes.iter().any(|&z| z > pz && z - pz < 40000.0) {
        return;
    }
    let ahead = 15000.0 + 1500.0 * (g.elapsed * 0.6).sin();
    let k = ((pz + ahead) / SEG_LEN) as usize;
    let k = k.saturating_sub(base);
    if k >= DRAW_DIST {
        return;
    }
    let sp = proj[k];
    if !sp.ok {
        return;
    }
    let u = sp.s1 * vw() as f32 / 2.0;
    let cx = sp.x1 + sp.s1 * (0.5 * (g.elapsed * 0.45).sin()) * ROAD_W * vw() as f32 / 2.0;
    draw_heli(fb, cx, sp.y1, u, 3400.0, g.elapsed, sp.clip, true);
}
