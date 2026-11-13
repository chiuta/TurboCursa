use super::*;

// ---------- weather and surface ----------

/// Look of the road and ground derived from weather and surface.
#[derive(Clone, Copy)]
pub(super) struct Env {
    surface: crate::env::Surface,
    snow: f32,
    wet: f32,
    sand: f32,
    ash: f32,
    /// sky veil colour, how strongly the sky is tinted, and the depth-fog density
    veil: u32,
    sky_tint: f32,
    fog: f32,
}

pub(super) fn env_of(g: &Game) -> Env {
    use crate::env::Weather as W;
    let wt = g.weather;
    let (snow, wet, sand, ash) = match wt {
        W::Snow => (0.7, 0.0, 0.0, 0.0),
        W::Blizzard => (1.0, 0.0, 0.0, 0.0),
        W::Hail => (0.25, 0.4, 0.0, 0.0),
        W::Rain => (0.0, 0.6, 0.0, 0.0),
        W::Storm => (0.0, 0.85, 0.0, 0.0),
        W::Glaze => (0.0, 0.6, 0.0, 0.0),
        W::Sandstorm => (0.0, 0.0, 0.5, 0.0),
        W::Ash => (0.0, 0.0, 0.0, 0.6),
        _ => (0.0, 0.0, 0.0, 0.0),
    };
    let (veil, sky_tint) = match wt {
        W::Clear => (rgb(255, 255, 255), 0.0),
        W::Rain => (rgb(96, 104, 120), 0.55),
        W::Storm => (rgb(52, 58, 74), 0.75),
        W::Fog => (rgb(206, 210, 214), 0.85),
        W::Snow => (rgb(196, 206, 220), 0.6),
        W::Blizzard => (rgb(232, 238, 246), 0.92),
        W::Sandstorm => (rgb(206, 160, 98), 0.85),
        W::Hail => (rgb(120, 130, 150), 0.5),
        W::Glaze => (rgb(150, 172, 194), 0.4),
        W::Heatwave => (rgb(255, 214, 150), 0.3),
        W::Ash => (rgb(78, 72, 72), 0.85),
    };
    Env {
        surface: g.surface,
        snow,
        wet,
        sand,
        ash,
        veil,
        sky_tint,
        fog: (1.0 - wt.params().clarity) * 0.95,
    }
}

impl Env {
    /// Re-colours a biome palette for the current weather and surface.
    pub(super) fn apply(&self, mut p: Pal) -> Pal {
        use crate::env::Surface as S;
        match self.surface {
            S::Asphalt => {}
            S::IceLake => {
                p.grass = [rgb(206, 226, 242), rgb(190, 214, 236)];
                p.road = [rgb(150, 200, 232), rgb(138, 190, 226)];
                p.rumble = [rgb(238, 246, 255), rgb(118, 168, 210)];
                p.lane = rgb(255, 255, 255);
            }
            S::SaltFlat => {
                p.grass = [rgb(238, 238, 232), rgb(226, 226, 220)];
                p.road = [rgb(214, 220, 226), rgb(204, 210, 218)];
                p.rumble = [rgb(250, 250, 250), rgb(200, 90, 70)];
                p.lane = rgb(120, 130, 150);
            }
            S::Lava => {
                p.grass = [rgb(44, 18, 12), rgb(58, 22, 10)];
                p.road = [rgb(62, 54, 54), rgb(54, 48, 50)];
                p.rumble = [rgb(255, 112, 24), rgb(130, 34, 10)];
                p.lane = rgb(255, 170, 60);
            }
            S::Dunes => {
                p.grass = [rgb(226, 180, 106), rgb(212, 164, 92)];
                p.road = [rgb(192, 162, 122), rgb(182, 152, 114)];
                p.rumble = [rgb(236, 206, 150), rgb(190, 150, 96)];
                p.lane = rgb(240, 226, 190);
            }
            S::Bog => {
                p.grass = [rgb(42, 72, 42), rgb(34, 60, 36)];
                p.road = [rgb(96, 76, 54), rgb(86, 68, 48)];
                p.rumble = [rgb(124, 104, 72), rgb(88, 72, 50)];
                p.lane = rgb(176, 160, 120);
            }
        }
        let white = rgb(240, 244, 250);
        if self.snow > 0.0 {
            let s = self.snow;
            p.grass = p.grass.map(|c| mix(c, white, 0.85 * s));
            p.road = p.road.map(|c| mix(c, rgb(222, 228, 236), 0.62 * s));
            p.rumble = p.rumble.map(|c| mix(c, white, 0.5 * s));
            p.lane = mix(p.lane, white, 0.3 * s);
        }
        if self.wet > 0.0 {
            let w = self.wet;
            p.road = p
                .road
                .map(|c| mix(shade(c, 0.78), rgb(40, 52, 78), 0.25 * w));
            p.rumble = p.rumble.map(|c| shade(c, 1.0 - 0.18 * w));
            p.grass = p.grass.map(|c| shade(c, 1.0 - 0.15 * w));
        }
        if self.sand > 0.0 {
            let tan = rgb(210, 170, 110);
            p.grass = p.grass.map(|c| mix(c, tan, 0.5 * self.sand));
            p.road = p.road.map(|c| mix(c, tan, 0.55 * self.sand));
        }
        if self.ash > 0.0 {
            let grey = rgb(92, 90, 94);
            p.grass = p.grass.map(|c| mix(c, grey, 0.7 * self.ash));
            p.road = p.road.map(|c| mix(c, grey, 0.55 * self.ash));
            p.rumble = p.rumble.map(|c| mix(c, grey, 0.4 * self.ash));
        }
        p
    }

    pub(super) fn sky(&self, p: &mut Pal) {
        let t = self.sky_tint;
        if t > 0.0 {
            p.sky_top = mix(p.sky_top, self.veil, t * 0.8);
            p.sky_hor = mix(p.sky_hor, self.veil, t);
            p.mount = mix(p.mount, self.veil, t * 0.6);
        }
    }
}

/// Fills a convex quadrilateral (scanline), limited to rows above `clip`.
pub(super) fn poly4(fb: &mut [u32], pts: [(f32, f32); 4], clip: f32, c: u32) {
    let (w, h) = (vw() as i32, vh() as i32);
    let ymin = pts.iter().map(|p| p.1).fold(f32::MAX, f32::min);
    let ymax = pts.iter().map(|p| p.1).fold(f32::MIN, f32::max);
    if !ymin.is_finite() || !ymax.is_finite() {
        return;
    }
    let y0 = (ymin.floor() as i32).max(0);
    let y1 = (ymax.ceil() as i32).min(h - 1).min(clip.round() as i32 - 1);
    for y in y0..=y1 {
        let yf = y as f32 + 0.5;
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for i in 0..4 {
            let (a, b) = (pts[i], pts[(i + 1) % 4]);
            if (a.1 <= yf && b.1 > yf) || (b.1 <= yf && a.1 > yf) {
                let x = a.0 + (b.0 - a.0) * (yf - a.1) / (b.1 - a.1);
                lo = lo.min(x);
                hi = hi.max(x);
            }
        }
        if lo > hi {
            continue;
        }
        let xa = (lo.round() as i32).clamp(0, w);
        let xb = (hi.round() as i32).clamp(0, w);
        if xb > xa {
            let r = y as usize * w as usize;
            fb[r + xa as usize..r + xb as usize].fill(c);
        }
    }
}

/// A thin line between two screen points.
pub(super) fn seg_line(fb: &mut [u32], a: (f32, f32), b: (f32, f32), th: f32, clip: f32, c: u32) {
    let (a, b) = if a.1 <= b.1 { (a, b) } else { (b, a) };
    if (b.1 - a.1).abs() < 0.5 {
        return;
    }
    poly4(
        fb,
        [
            (a.0 - th, a.1),
            (a.0 + th, a.1),
            (b.0 + th, b.1),
            (b.0 - th, b.1),
        ],
        clip,
        c,
    );
}

/// Depth fog: the farther (nearer the horizon) the thicker, the sky is hazy too.
pub(super) fn fog_veil(fb: &mut [u32], col: u32, density: f32) {
    if density < 0.02 {
        return;
    }
    let (w, h) = (vw(), vh());
    let hz = horizon() as f32;
    let c = unpack(col);
    for y in 0..h {
        let yf = y as f32;
        let a = if yf < hz {
            density * 0.55
        } else {
            let t = ((yf - hz) / (h as f32 - hz)).clamp(0.0, 1.0);
            density * (1.0 - t).powf(1.25)
        };
        if a < 0.01 {
            continue;
        }
        let row = &mut fb[y * w..(y + 1) * w];
        for px in row.iter_mut() {
            let o = unpack(*px);
            *px = rgb(
                (o[0] + (c[0] - o[0]) * a) as u8,
                (o[1] + (c[1] - o[1]) * a) as u8,
                (o[2] + (c[2] - o[2]) * a) as u8,
            );
        }
    }
}

/// Precipitation, blowing particles, lightning, heat shimmer and the depth veil.
pub(super) fn weather_fx(fb: &mut [u32], g: &Game, env: &Env) {
    use crate::env::Weather as W;
    let (vwf, vhf, k) = (vw() as f32, vh() as f32, vk());
    let t = g.elapsed;
    let wind = g.wind;
    fog_veil(fb, env.veil, env.fog);
    let streak =
        |fb: &mut [u32], n: u32, len: f32, fall: f32, slant: f32, col: u32, a: f32, salt: u32| {
            for i in 0..n {
                let sp = (0.7 + 0.6 * h01(i, salt + 1)) * fall * k;
                let y = (h01(i, salt + 2) * vhf + t * sp).rem_euclid(vhf);
                let x = (h01(i, salt + 3) * vwf + slant * y).rem_euclid(vwf);
                let steps = (len * k).max(1.0) as i32;
                for d in 0..steps {
                    blend_px(
                        fb,
                        (x + slant * d as f32) as i32,
                        (y - d as f32) as i32,
                        col,
                        a * (1.0 - d as f32 / steps as f32 * 0.6),
                    );
                }
            }
        };
    match g.weather {
        W::Rain => streak(
            fb,
            230,
            9.0,
            620.0,
            0.12 + wind * 0.1,
            rgb(190, 210, 235),
            0.55,
            600,
        ),
        W::Storm => {
            streak(
                fb,
                460,
                12.0,
                760.0,
                0.35 + wind * 0.25,
                rgb(200, 216, 240),
                0.6,
                610,
            );
            // lightning: a flash and a jagged bolt every few seconds
            let b = (t / 3.2) as i32;
            let ph = (t / 3.2).fract();
            if h01(b as u32, 300) > 0.45 && ph < 0.12 {
                let f = (1.0 - ph / 0.12) * if crate::access::calm() { 0.1 } else { 0.5 };
                for px in fb.iter_mut() {
                    let o = unpack(*px);
                    *px = rgb(
                        (o[0] + 200.0 * f).min(255.0) as u8,
                        (o[1] + 200.0 * f).min(255.0) as u8,
                        (o[2] + 210.0 * f).min(255.0) as u8,
                    );
                }
                let mut x = (0.15 + 0.7 * h01(b as u32, 301)) * vwf;
                let mut y = 0.0;
                let mut i = 0;
                while y < horizon() as f32 {
                    let nx = x + (h01(b as u32 * 31 + i, 302) - 0.5) * 26.0 * k;
                    for d in 0..(6.0 * k) as i32 {
                        blend_px(
                            fb,
                            (x + (nx - x) * d as f32 / (6.0 * k)) as i32,
                            (y + d as f32) as i32,
                            rgb(255, 255, 255),
                            0.95,
                        );
                    }
                    x = nx;
                    y += 6.0 * k;
                    i += 1;
                }
            }
        }
        W::Hail => {
            for i in 0..110u32 {
                let sp = (0.8 + 0.5 * h01(i, 621)) * 520.0 * k;
                let y = (h01(i, 622) * vhf + t * sp).rem_euclid(vhf);
                let x = (h01(i, 623) * vwf + 0.1 * y).rem_euclid(vwf);
                let s = 1 + (h01(i, 624) > 0.6) as i32;
                for dy in 0..s {
                    for dx in 0..s {
                        blend_px(fb, x as i32 + dx, y as i32 + dy, rgb(240, 248, 255), 0.85);
                    }
                }
            }
        }
        W::Glaze => streak(fb, 150, 7.0, 520.0, 0.05, rgb(170, 210, 250), 0.5, 630),
        W::Snow | W::Blizzard => {
            let bl = g.weather == W::Blizzard;
            let n = if bl { 380 } else { 170 };
            for i in 0..n as u32 {
                let fall = (if bl { 150.0 } else { 70.0 }) * (0.5 + h01(i, 641)) * k;
                let drift =
                    (if bl { 420.0 } else { 40.0 }) * k * (1.0 + 0.4 * wind) * (0.5 + h01(i, 642));
                let y = (h01(i, 643) * vhf + t * fall).rem_euclid(vhf);
                let sway = (t * (0.6 + h01(i, 644)) + i as f32).sin() * 9.0 * k;
                let x = (h01(i, 645) * vwf - t * drift + sway).rem_euclid(vwf);
                let s = 1 + (h01(i, 646) > 0.7) as i32 + (h01(i, 647) > 0.93) as i32;
                for dy in 0..s {
                    for dx in 0..(s + bl as i32 * 3) {
                        blend_px(fb, x as i32 + dx, y as i32 + dy, rgb(255, 255, 255), 0.8);
                    }
                }
            }
        }
        W::Sandstorm => {
            for i in 0..420u32 {
                let drift = (900.0 + 500.0 * h01(i, 651)) * k * (1.0 + 0.3 * wind);
                let y = h01(i, 652) * vhf;
                let x = (h01(i, 653) * vwf - t * drift).rem_euclid(vwf);
                for d in 0..(8.0 * k) as i32 {
                    blend_px(
                        fb,
                        (x + d as f32) as i32,
                        y as i32,
                        rgb(224, 184, 120),
                        0.55,
                    );
                }
            }
        }
        W::Ash => {
            for i in 0..170u32 {
                let fall = (40.0 + 60.0 * h01(i, 661)) * k;
                let y = (h01(i, 662) * vhf + t * fall).rem_euclid(vhf);
                let sway = (t * (0.4 + h01(i, 663)) + i as f32).sin() * 12.0 * k;
                let x = (h01(i, 664) * vwf + sway).rem_euclid(vwf);
                let s = 1 + (h01(i, 665) > 0.8) as i32;
                let c = if h01(i, 666) > 0.85 {
                    rgb(255, 120, 40)
                } else {
                    rgb(50, 48, 52)
                };
                for dy in 0..s {
                    for dx in 0..s {
                        blend_px(fb, x as i32 + dx, y as i32 + dy, c, 0.85);
                    }
                }
            }
        }
        W::Heatwave => {
            // shimmering air above the horizon band
            let (w, hz) = (vw(), horizon());
            let band = (50.0 * k) as i32;
            let mut row = vec![0u32; w];
            for y in (hz - band / 3).max(0)..(hz + band).min(vh() as i32) {
                let amp = (1.0 - ((y - hz) as f32 / band as f32).abs().min(1.0)) * 2.4 * k;
                let sh = ((y as f32 * 0.35 + t * 6.0).sin() * amp) as i32;
                if sh == 0 {
                    continue;
                }
                let r = y as usize * w;
                row.copy_from_slice(&fb[r..r + w]);
                for x in 0..w as i32 {
                    fb[r + x as usize] = row[(x + sh).rem_euclid(w as i32) as usize];
                }
            }
            // warm veil
            for px in fb.iter_mut().step_by(1) {
                *px = mix(*px, rgb(255, 196, 120), 0.07);
            }
        }
        W::Fog | W::Clear => {}
    }
}

pub(super) const RUMBLE_LEN: usize = 3;

/// The ghost of the best run, drawn as a pale cyan car with a halo, and the time gap.
pub(super) fn draw_ghost(fb: &mut [u32], g: &Game, proj: &[SegProj]) {
    let Some((gp, gx)) = g.ghost_now() else {
        return;
    };
    let n = g.track.segs.len();
    let base = (g.pos / SEG_LEN) as usize;
    let idx = (gp / SEG_LEN) as usize;
    if idx < base || idx - base >= proj.len() || idx - base >= DRAW_DIST {
        return;
    }
    let sp = proj[idx - base];
    if !sp.ok {
        return;
    }
    let _ = n;
    let pct = (gp % SEG_LEN) / SEG_LEN;
    let s = lerp(sp.s1, sp.s2, pct);
    let cx = lerp(sp.x1, sp.x2, pct) + s * gx * ROAD_W * vw() as f32 / 2.0;
    let by = lerp(sp.y1, sp.y2, pct);
    let w = s * CAR_W * vw() as f32 / 2.0;
    glow(
        fb,
        cx,
        by - w * 0.3,
        w * 0.9,
        w * 0.5,
        rgb(120, 220, 255),
        0.35,
        sp.clip,
    );
    draw_vehicle(fb, cx, by, w, rgb(150, 230, 255), sp.clip, false, 0);
}

pub(super) fn draw_player(fb: &mut [u32], g: &Game) {
    let sp = g.speed / MAX_SPEED;
    let bounce = ((g.elapsed * 38.0).sin() * sp * 0.8).abs() * vk();
    let shake = if g.shake > 0.0 && !crate::access::calm() {
        (g.elapsed * 90.0).sin() * 2.0 * vk()
    } else {
        0.0
    };
    let cx = vw() as f32 / 2.0 + g.steer * 4.0 * vk() + shake;
    let by0 = vh() as f32 - 10.0 * vk() - bounce;
    let lift = (g.air * 26.0 * vk()).min(vh() as f32 * 0.5);
    let by = by0 - lift;
    if lift > 0.5 {
        ellipse_a(
            fb,
            cx,
            by0 - 2.0 * vk(),
            44.0 * vk(),
            7.0 * vk(),
            rgb(0, 0, 0),
            0.4 / (1.0 + g.air * 0.4),
            vh() as f32,
        );
    }
    if g.turbo_t > 0.0 {
        let fl = (8.0 + (g.elapsed * 60.0).sin().abs() * 6.0) * vk();
        for s in [-1.0f32, 1.0] {
            let fx0 = cx + s * 22.0 * vk();
            glow(
                fb,
                fx0,
                by - 10.0 * vk(),
                36.0 * vk(),
                30.0 * vk(),
                rgb(255, 140, 40),
                0.9,
                vh() as f32,
            );
            tri(
                fb,
                fx0,
                by - 14.0 * vk() + fl,
                by - 14.0 * vk(),
                6.0 * vk(),
                rgb(255, 170, 40),
                vh() as f32,
            );
            tri(
                fb,
                fx0,
                by - 14.0 * vk() + fl * 0.55,
                by - 14.0 * vk(),
                3.0 * vk(),
                rgb(255, 250, 200),
                vh() as f32,
            );
        }
    }
    let body = if g.car.is_default() {
        rgb(214, 28, 36)
    } else {
        g.car.body
    };
    LIVERY.with(|l| l.set((!g.car.is_default()).then_some(g.car)));
    if g.car.class.is_special() {
        let k = (g.speed / 2200.0).min(1.0);
        draw_special(
            fb,
            cx,
            by,
            84.0 * vk(),
            g.car.class,
            g.elapsed,
            k,
            vh() as f32,
        );
    } else {
        draw_vehicle(
            fb,
            cx,
            by,
            84.0 * vk() * view_scale(g.car.class),
            body,
            vh() as f32,
            g.car.class != crate::cars::VehicleClass::Sedan,
            class_kind(g.car.class),
        );
    }
    LIVERY.with(|l| l.set(None));
    plate(fb, cx, by, 84.0 * vk() * view_scale(g.car.class), &g.car);
    if g.shield_t > 0.0 && (g.shield_t > 1.5 || (g.elapsed * 10.0) as i32 % 2 == 0) {
        glow(
            fb,
            cx,
            by - 34.0 * vk(),
            78.0 * vk(),
            56.0 * vk(),
            rgb(90, 150, 255),
            0.55,
            vh() as f32,
        );
    }
    rect(
        fb,
        cx - 3.0 * vk(),
        by - 33.0 * vk(),
        cx + 3.0 * vk(),
        by - 18.0 * vk(),
        rgb(240, 240, 240),
        vh() as f32,
    );
}

pub(super) fn draw_particles(fb: &mut [u32], g: &Game) {
    for p in &g.particles {
        let t = (p.life / p.max).clamp(0.0, 1.0);
        match p.kind {
            0 => {
                let r = p.size * (1.0 + (1.0 - t) * 2.2);
                ellipse_a(
                    fb,
                    p.x,
                    p.y,
                    r,
                    r * 0.8,
                    rgb(198, 178, 142),
                    0.38 * t,
                    vh() as f32,
                );
            }
            1 => {
                glow(
                    fb,
                    p.x,
                    p.y,
                    p.size * 4.0,
                    p.size * 4.0,
                    rgb(255, 180, 70),
                    t,
                    vh() as f32,
                );
                ellipse(
                    fb,
                    p.x,
                    p.y,
                    p.size * 0.7,
                    p.size * 0.7,
                    rgb(255, 245, 200),
                    vh() as f32,
                );
            }
            _ => {
                glow(
                    fb,
                    p.x,
                    p.y,
                    p.size * (2.5 + (1.0 - t) * 2.0),
                    p.size * 2.5,
                    rgb(255, 140, 40),
                    0.8 * t,
                    vh() as f32,
                );
                glow(
                    fb,
                    p.x,
                    p.y,
                    p.size,
                    p.size,
                    rgb(255, 240, 200),
                    0.9 * t,
                    vh() as f32,
                );
            }
        }
    }
}

pub(super) fn draw_hud(fb: &mut [u32], g: &Game) {
    let k = |v: f32| (v * vk()).round() as i32;
    let kf = |v: f32| v * vk();
    let white = rgb(255, 255, 255);
    let yellow = rgb(255, 220, 60);
    let panel = rgb(8, 10, 24);
    let hf = vh() as f32;

    // active bonus / hazard timers: coloured bars under the clock
    let mut by = kf(38.0);
    for (t, max, col) in [
        (g.shield_t, 6.0, rgb(80, 120, 255)),
        (g.boost_t, 3.0, rgb(40, 210, 255)),
        (g.freeze_t, 5.0, rgb(170, 230, 255)),
        (g.x2_t, 12.0, rgb(240, 70, 220)),
        (g.slip_t, 1.4, rgb(120, 120, 130)),
        (g.wet_t, 2.0, rgb(60, 110, 190)),
        (g.mud_t, 1.5, rgb(120, 84, 48)),
    ] {
        if t > 0.0 {
            rect_a(fb, kf(3.0), by, kf(50.0), by + kf(5.0), panel, 0.5, hf);
            rect(
                fb,
                kf(3.0),
                by,
                kf(3.0 + 47.0 * (t / max).min(1.0)),
                by + kf(5.0),
                col,
                hf,
            );
            by += kf(7.0);
        }
    }

    // top-left: time
    rect_a(fb, kf(3.0), kf(3.0), kf(50.0), kf(34.0), panel, 0.5, hf);
    rect(fb, kf(3.0), kf(3.0), kf(5.0), kf(34.0), yellow, hf);
    let tcol = if g.time < 10.0 && (g.elapsed * 4.0) as i32 % 2 == 0 {
        rgb(255, 80, 80)
    } else {
        yellow
    };
    text_shadow(fb, k(9.0), k(6.0), i18n::tr(S::Time), 1, rgb(190, 200, 230));
    text_shadow(
        fb,
        k(9.0),
        k(14.0),
        &format!("{:02}", g.time.ceil() as i32),
        3,
        tcol,
    );

    // mode label and its live counters, under the progress bar
    if g.mode != Mode::Campaign {
        let extra = match g.mode {
            Mode::Endless | Mode::Zen => format!("  {} {}", i18n::tr(S::Lap), g.lap),
            Mode::Collector => format!("  {} {}", i18n::tr(S::Orbs), g.collected),
            Mode::Slalom => format!("  {} X{}", i18n::tr(S::Combo), g.combo.max(1)),
            _ => String::new(),
        };
        let label = format!("{}{}", i18n::mode_name(g.mode.index() as usize), extra);
        let lw = font::text_width(&label, vk() as i32);
        let hot = g.inverted && (g.elapsed * 6.0) as i32 % 2 == 0;
        text_shadow(
            fb,
            (vw() as i32 - lw) / 2,
            k(24.0),
            &label,
            1,
            if hot {
                rgb(255, 80, 80)
            } else {
                rgb(190, 200, 230)
            },
        );
    }

    // top-right: score
    let sc = format!("{:07}", g.score);
    let sw = font::text_width(&sc, 2 * vk() as i32);
    rect_a(
        fb,
        vw() as f32 - kf(3.0) - sw as f32 - kf(10.0),
        kf(3.0),
        vw() as f32 - kf(3.0),
        kf(22.0),
        panel,
        0.5,
        hf,
    );
    text_shadow(fb, vw() as i32 - k(8.0) - sw, k(8.0), &sc, 2, white);

    // bottom-left: speed
    rect_a(
        fb,
        kf(3.0),
        hf - kf(36.0),
        kf(84.0),
        hf - kf(4.0),
        panel,
        0.5,
        hf,
    );
    text_shadow(
        fb,
        k(8.0),
        vh() as i32 - k(31.0),
        &format!("{}", crate::settings::display_speed(g.kmh())),
        3,
        white,
    );
    text_shadow(
        fb,
        k(46.0),
        vh() as i32 - k(24.0),
        if crate::settings::mph() {
            "MPH"
        } else {
            i18n::tr(S::Kmh)
        },
        1,
        yellow,
    );
    let sp = (g.speed / (MAX_SPEED * 1.3)).clamp(0.0, 1.0);
    rect(
        fb,
        kf(8.0),
        hf - kf(10.0),
        kf(80.0),
        hf - kf(7.0),
        rgb(40, 44, 64),
        hf,
    );
    let bar = mix(rgb(60, 200, 255), rgb(255, 90, 60), sp);
    rect(
        fb,
        kf(8.0),
        hf - kf(10.0),
        kf(8.0) + kf(72.0) * sp,
        hf - kf(7.0),
        bar,
        hf,
    );

    // bottom-right: turbo charges
    rect_a(
        fb,
        vw() as f32 - kf(52.0),
        hf - kf(30.0),
        vw() as f32 - kf(3.0),
        hf - kf(4.0),
        panel,
        0.5,
        hf,
    );
    text_shadow(
        fb,
        vw() as i32 - k(48.0),
        vh() as i32 - k(27.0),
        i18n::tr(S::Turbo),
        1,
        rgb(190, 200, 230),
    );
    let npips = g.car.turbos.max(g.turbo).clamp(1, 6) as usize;
    let pitch = 42.0 / npips as f32;
    for i in 0..npips {
        let on = i < g.turbo as usize;
        let c = if on {
            rgb(255, 150, 30)
        } else {
            rgb(60, 62, 80)
        };
        let x0 = vw() as f32 - kf(48.0) + i as f32 * kf(pitch);
        rect(
            fb,
            x0,
            hf - kf(18.0),
            x0 + kf(pitch - 4.0),
            hf - kf(9.0),
            c,
            hf,
        );
        if on {
            glow(
                fb,
                x0 + kf((pitch - 4.0) / 2.0),
                hf - kf(13.5),
                kf(14.0),
                kf(10.0),
                rgb(255, 140, 30),
                0.55,
                hf,
            );
        }
    }

    // progress
    let prog = (g.pos / (g.track.finish_seg as f32 * SEG_LEN)).clamp(0.0, 1.0);
    let (bx0, bx1) = (kf(110.0), kf(210.0));
    rect_a(
        fb,
        bx0 - kf(4.0),
        kf(3.0),
        bx1 + kf(4.0),
        kf(13.0),
        panel,
        0.5,
        hf,
    );
    rect(fb, bx0, kf(7.0), bx1, kf(9.0), rgb(50, 54, 78), hf);
    rect(
        fb,
        bx0,
        kf(7.0),
        bx0 + (bx1 - bx0) * prog,
        kf(9.0),
        yellow,
        hf,
    );
    for &c in &g.track.checkpoints {
        let t = c as f32 / g.track.finish_seg as f32;
        rect(
            fb,
            bx0 + (bx1 - bx0) * t - 0.5 * vk(),
            kf(5.0),
            bx0 + (bx1 - bx0) * t + 0.5 * vk(),
            kf(11.0),
            white,
            hf,
        );
    }
    tri(
        fb,
        bx0 + (bx1 - bx0) * prog,
        kf(4.0),
        kf(8.0),
        kf(2.5),
        rgb(255, 60, 60),
        hf,
    );

    if g.banner_t > 0.0 {
        let a = (g.banner_t * 3.0).min(1.0);
        let btxt = g.banner_text();
        let w = font::text_width(&btxt, 2 * vk() as i32) as f32;
        rect_a(
            fb,
            (vw() as f32 - w) / 2.0 - kf(10.0),
            kf(48.0),
            (vw() as f32 + w) / 2.0 + kf(10.0),
            kf(68.0),
            panel,
            0.55 * a,
            hf,
        );
        text_center(fb, k(53.0), &btxt, 2, yellow);
    }
    match g.state {
        State::TimeUp => {
            rect_a(fb, 0.0, 0.0, vw() as f32, hf, rgb(0, 0, 10), 0.35, hf);
            text_center(fb, k(66.0), i18n::tr(S::TimeUp), 4, rgb(255, 80, 80));
            text_center(
                fb,
                k(100.0),
                &format!("{} {}", i18n::tr(S::Score), g.score),
                2,
                white,
            );
            text_center(fb, k(120.0), i18n::tr(S::PressEnter), 1, white);
        }
        State::Finished => {
            rect_a(fb, 0.0, 0.0, vw() as f32, hf, rgb(0, 0, 10), 0.3, hf);
            let pb = crate::access::prev_best();
            if pb > 0 && g.score > pb {
                let gold = if (g.elapsed * 4.0) as i32 % 2 == 0 || crate::access::calm() {
                    rgb(255, 235, 120)
                } else {
                    rgb(255, 170, 40)
                };
                text_center(fb, k(50.0), i18n::tr(S::NewBest), 2, gold);
            }
            text_center(fb, k(66.0), i18n::tr(S::StageClear), 3, yellow);
            text_center(
                fb,
                k(92.0),
                &format!("{} {}", i18n::tr(S::Score), g.score),
                3,
                white,
            );
            text_center(fb, k(120.0), &i18n::passed(g.overtaken), 1, white);
            if g.coins_earned > 0 {
                let t = format!("+{} {}", g.coins_earned, i18n::banner("COINS"));
                text_center(fb, k(132.0), &t, 1, yellow);
            }
            text_center(fb, k(146.0), i18n::tr(S::PressEnter), 1, white);
        }
        State::Disqualified => {
            rect_a(fb, 0.0, 0.0, vw() as f32, hf, rgb(20, 0, 10), 0.4, hf);
            let flash = if crate::access::calm() || (g.elapsed * 6.0) as i32 % 2 == 0 {
                rgb(255, 60, 60)
            } else {
                rgb(80, 120, 255)
            };
            text_center(fb, k(60.0), i18n::banner("BUSTED"), 4, flash);
            text_center(
                fb,
                k(104.0),
                &format!("{} {}", i18n::tr(S::Score), g.score),
                2,
                white,
            );
            text_center(fb, k(124.0), i18n::tr(S::PressEnter), 1, white);
        }
        State::Playing => {}
    }
}
