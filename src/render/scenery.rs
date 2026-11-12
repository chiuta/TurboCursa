use super::*;

// ---------- animals, ramps, pads, stations, day and night ----------

/// Fuel, gearbox, police pressure, clock and the distance to the next station.
pub(super) fn draw_hud_extra(fb: &mut [u32], g: &Game) {
    use crate::i18n_career::{t, C};
    let k = |v: f32| (v * vk()).round() as i32;
    let kf = |v: f32| v * vk();
    let hf = vh() as f32;
    let panel = rgb(8, 10, 24);
    let white = rgb(255, 255, 255);
    // fuel gauge, bottom left
    if g.fuel_rule && g.mode != Mode::Zen {
        let low = g.fuel < 0.2;
        let col = if low && (g.elapsed * 5.0) as i32 % 2 == 0 {
            rgb(255, 60, 60)
        } else if g.fuel < 0.4 {
            rgb(255, 190, 40)
        } else {
            rgb(70, 220, 110)
        };
        rect_a(
            fb,
            kf(3.0),
            hf - kf(66.0),
            kf(62.0),
            hf - kf(45.0),
            panel,
            0.55,
            hf,
        );
        text_shadow(
            fb,
            k(6.0),
            vh() as i32 - k(64.0),
            t(C::Fuel),
            1,
            rgb(190, 200, 230),
        );
        rect(
            fb,
            kf(6.0),
            hf - kf(56.0),
            kf(59.0),
            hf - kf(48.0),
            rgb(30, 34, 54),
            hf,
        );
        rect(
            fb,
            kf(6.0),
            hf - kf(56.0),
            kf(6.0 + 53.0 * g.fuel),
            hf - kf(48.0),
            col,
            hf,
        );
        if g.fuel < 0.5 || g.refueling {
            let pz = g.pos + PLAYER_Z;
            if let Some(&(a, b)) = g.stations.iter().find(|&&(_, b)| b > pz) {
                let d = (a - pz).max(0.0);
                let s = if pz >= a && pz < b {
                    format!("{}!", t(C::Station))
                } else {
                    format!("{} {:.1}K", t(C::Station), d / 10000.0)
                };
                text_shadow(fb, k(6.0), vh() as i32 - k(76.0), &s, 1, rgb(255, 230, 120));
            }
        }
    }
    // ghost gap
    if let Some(gap) = g.ghost_gap() {
        let s = format!("{} {:+.1}S", t(C::Ghost), gap);
        let col = if gap >= 0.0 {
            rgb(120, 255, 160)
        } else {
            rgb(255, 150, 120)
        };
        text_shadow(
            fb,
            (vw() as f32 * 0.5) as i32 - 2 * s.len() as i32 * vk().round() as i32,
            k(38.0),
            &s,
            1,
            col,
        );
    }
    // damage gauge next to the fuel gauge: the worse of body and engine
    if g.mode != Mode::Zen && !g.photo && (g.body < 0.999 || g.engine < 0.999) {
        let h = g.body.min(g.engine);
        let col = if h < 0.25 {
            rgb(255, 60, 60)
        } else if h < 0.55 {
            rgb(255, 190, 40)
        } else {
            rgb(150, 200, 255)
        };
        rect_a(
            fb,
            kf(66.0),
            hf - kf(66.0),
            kf(125.0),
            hf - kf(45.0),
            panel,
            0.55,
            hf,
        );
        text_shadow(
            fb,
            k(69.0),
            vh() as i32 - k(64.0),
            t(C::Damage),
            1,
            rgb(190, 200, 230),
        );
        rect(
            fb,
            kf(69.0),
            hf - kf(56.0),
            kf(122.0),
            hf - kf(48.0),
            rgb(30, 34, 54),
            hf,
        );
        rect(
            fb,
            kf(69.0),
            hf - kf(56.0),
            kf(69.0 + 53.0 * h),
            hf - kf(48.0),
            col,
            hf,
        );
    }
    // gearbox, bottom right
    {
        let x1 = vw() as f32 - kf(3.0);
        rect_a(
            fb,
            x1 - kf(46.0),
            hf - kf(30.0),
            x1,
            hf - kf(3.0),
            panel,
            0.55,
            hf,
        );
        let lab = if g.manual { t(C::Manual) } else { t(C::Auto) };
        text_shadow(
            fb,
            (x1 - kf(43.0)) as i32,
            vh() as i32 - k(28.0),
            lab,
            1,
            rgb(190, 200, 230),
        );
        text_shadow(
            fb,
            (x1 - kf(43.0)) as i32,
            vh() as i32 - k(20.0),
            &format!("{}", g.gear),
            2,
            white,
        );
        let r = (g.rpm / 1.2).clamp(0.0, 1.0);
        rect(
            fb,
            x1 - kf(30.0),
            hf - kf(12.0),
            x1 - kf(5.0),
            hf - kf(6.0),
            rgb(30, 34, 54),
            hf,
        );
        rect(
            fb,
            x1 - kf(30.0),
            hf - kf(12.0),
            x1 - kf(30.0 - 25.0 * r),
            hf - kf(6.0),
            if g.rpm > 1.0 {
                rgb(255, 70, 70)
            } else {
                rgb(90, 200, 255)
            },
            hf,
        );
    }
    // fork signs: everybody sees the two routes coming; the helicopter gets them earlier
    if g.route == 0 {
        if let Some((fs, _)) = g.fork {
            let d = fs as f32 * SEG_LEN - (g.pos + PLAYER_Z);
            let reach = if g.car.class == crate::cars::VehicleClass::Helicopter {
                25000.0
            } else {
                16000.0
            };
            if d > 0.0 && d < reach {
                let blink = (g.elapsed * 3.0) as i32 % 2 == 0;
                let c = if blink {
                    rgb(255, 230, 90)
                } else {
                    rgb(255, 150, 40)
                };
                let y = k(44.0);
                text_shadow(
                    fb,
                    k(6.0),
                    y,
                    &format!("< {}", i18n::banner("QUICK ROUTE")),
                    1,
                    c,
                );
                let s = format!("{} >", i18n::banner("SCENIC ROUTE"));
                let wpx = s.chars().count() as i32 * 6 * vk() as i32;
                text_shadow(fb, vw() as i32 - wpx - k(6.0), y, &s, 1, c);
            }
        }
    }
    // race position, top left under the score
    if !g.pack.is_empty() {
        text_shadow(
            fb,
            k(6.0),
            k(34.0),
            &format!("{} {}/{}", i18n::banner("POS"), g.place, g.pack.len() + 1),
            2,
            rgb(255, 210, 60),
        );
    }
    // pursuit: the fugitive's name, the phase and his car's health, top centre
    if let Some(p) = &g.pursuit {
        let x0 = vw() as f32 / 2.0 - kf(60.0);
        rect_a(fb, x0, kf(14.0), x0 + kf(120.0), kf(38.0), panel, 0.55, hf);
        let label = match p.phase {
            crate::pursuit::Phase::Catch => i18n::banner("CATCH UP"),
            _ => i18n::banner("RAM HIM!"),
        };
        text_shadow(
            fb,
            (x0 + kf(3.0)) as i32,
            k(16.0),
            p.name(),
            1,
            rgb(255, 210, 60),
        );
        text_shadow(
            fb,
            (x0 + kf(3.0)) as i32,
            k(24.0),
            &format!("{} {:.0}", label, p.left.max(0.0)),
            1,
            white,
        );
        rect(
            fb,
            x0 + kf(3.0),
            kf(32.0),
            x0 + kf(117.0),
            kf(36.0),
            rgb(30, 34, 54),
            hf,
        );
        rect(
            fb,
            x0 + kf(3.0),
            kf(32.0),
            x0 + kf(3.0 + 114.0 * p.hp),
            kf(36.0),
            rgb(255, 80, 60),
            hf,
        );
    }
    // police pressure, top right
    if g.bust > 0.01 || !g.cops.is_empty() {
        let x1 = vw() as f32 - kf(3.0);
        let fl = (g.elapsed * 8.0) as i32 % 2 == 0;
        let col = if fl {
            rgb(255, 50, 50)
        } else {
            rgb(70, 120, 255)
        };
        rect_a(fb, x1 - kf(60.0), kf(36.0), x1, kf(55.0), panel, 0.55, hf);
        text_shadow(fb, (x1 - kf(57.0)) as i32, k(38.0), t(C::Police), 1, col);
        rect(
            fb,
            x1 - kf(57.0),
            kf(47.0),
            x1 - kf(5.0),
            kf(52.0),
            rgb(30, 34, 54),
            hf,
        );
        rect(
            fb,
            x1 - kf(57.0),
            kf(47.0),
            x1 - kf(57.0 - 52.0 * g.bust),
            kf(52.0),
            col,
            hf,
        );
    }
    // in-game clock
    {
        let m = g.clock_minutes();
        let s = format!("{:02}:{:02}", m / 60, m % 60);
        let x1 = vw() as f32 - kf(3.0);
        text_shadow(
            fb,
            (x1 - kf(26.0)) as i32,
            k(26.0),
            &s,
            1,
            rgb(200, 210, 240),
        );
    }
}

pub(super) fn draw_animal(
    fb: &mut [u32],
    a: &Animal,
    cx: f32,
    by: f32,
    u: f32,
    clip: f32,
    t: f32,
    fog: (u32, f32),
) {
    if u < 0.004 {
        return;
    }
    let f = |c: u32| mix(c, fog.0, fog.1);
    let (len, hgt, col, col2) = match a.species % 8 {
        0 => (420.0, 330.0, rgb(168, 112, 60), rgb(236, 214, 170)),
        1 => (560.0, 470.0, rgb(84, 58, 40), rgb(60, 40, 30)),
        2 => (260.0, 150.0, rgb(222, 108, 36), rgb(250, 246, 240)),
        3 => (470.0, 340.0, rgb(70, 46, 34), rgb(50, 32, 24)),
        4 => (520.0, 520.0, rgb(206, 164, 104), rgb(180, 140, 84)),
        5 => (130.0, 230.0, rgb(24, 26, 36), rgb(244, 244, 248)),
        6 => (330.0, 200.0, rgb(92, 76, 66), rgb(60, 50, 44)),
        _ => (240.0, 400.0, rgb(176, 124, 80), rgb(220, 184, 140)),
    };
    let d = a.dir;
    let lift = if a.hit {
        (a.hit_t * 3.2).sin().max(0.0) * 280.0 * u
    } else {
        0.0
    };
    let by0 = by;
    let by = by - lift;
    ellipse_a(
        fb,
        cx,
        by0 - 10.0 * u,
        len * 0.55 * u,
        26.0 * u,
        rgb(0, 0, 0),
        0.35,
        clip,
    );
    let sw = (t * 9.0 + a.z * 0.01).sin() * if a.hit { 0.0 } else { 1.0 };
    let upright = matches!(a.species % 8, 5 | 7);
    let ll = hgt * if upright { 0.25 } else { 0.42 };
    if !upright {
        for (i, off) in [-0.34f32, -0.16, 0.16, 0.34].iter().enumerate() {
            let ph = if i % 2 == 0 { sw } else { -sw };
            let lx = cx + d * len * off * u + ph * len * 0.07 * u;
            rect(
                fb,
                lx - 15.0 * u,
                by - ll * u,
                lx + 15.0 * u,
                by,
                f(shade(col, 0.7)),
                clip,
            );
        }
        // body, neck and head
        ellipse(
            fb,
            cx,
            by - (ll + hgt * 0.18) * u,
            len * 0.5 * u,
            hgt * 0.22 * u,
            f(col),
            clip,
        );
        let hx = cx + d * len * 0.52 * u;
        let hy = by - (ll + hgt * 0.45) * u;
        quad(
            fb,
            hy,
            hx - 30.0 * u,
            hx + 30.0 * u,
            by - (ll + hgt * 0.2) * u,
            cx + d * len * 0.34 * u - 70.0 * u,
            cx + d * len * 0.34 * u + 70.0 * u,
            clip,
            f(col),
        );
        ellipse(
            fb,
            hx + d * 30.0 * u,
            hy,
            hgt * 0.1 * u,
            hgt * 0.08 * u,
            f(col),
            clip,
        );
        ellipse(
            fb,
            hx + d * 50.0 * u,
            hy - 4.0 * u,
            9.0 * u,
            9.0 * u,
            f(rgb(10, 10, 10)),
            clip,
        );
        // tail and species marks
        let tx = cx - d * len * 0.5 * u;
        match a.species % 8 {
            0 | 1 => {
                // antlers
                let w = if a.species % 8 == 1 { 150.0 } else { 90.0 };
                for s in [-1.0f32, 1.0] {
                    seg_line(
                        fb,
                        (hx + s * 14.0 * u, hy - 20.0 * u),
                        (hx + s * w * u, hy - 150.0 * u),
                        6.0 * u,
                        clip,
                        f(col2),
                    );
                }
                ellipse(
                    fb,
                    tx,
                    by - (ll + hgt * 0.22) * u,
                    25.0 * u,
                    20.0 * u,
                    f(col2),
                    clip,
                );
            }
            2 => ellipse(
                fb,
                tx - d * 90.0 * u,
                by - (ll + hgt * 0.2) * u,
                110.0 * u,
                36.0 * u,
                f(col2),
                clip,
            ),
            4 => ellipse(
                fb,
                cx,
                by - (ll + hgt * 0.42) * u,
                90.0 * u,
                80.0 * u,
                f(col),
                clip,
            ),
            6 => seg_line(
                fb,
                (hx + d * 56.0 * u, hy + 6.0 * u),
                (hx + d * 66.0 * u, hy - 24.0 * u),
                5.0 * u,
                clip,
                f(rgb(240, 240, 230)),
            ),
            _ => {}
        }
    } else {
        // upright: penguin or kangaroo
        let hop = if a.species % 8 == 7 && !a.hit {
            (t * 7.0 + a.z * 0.01).sin().abs() * 120.0 * u
        } else {
            0.0
        };
        let by = by - hop;
        ellipse(
            fb,
            cx,
            by - hgt * 0.4 * u,
            len * 0.5 * u * if a.species % 8 == 5 { 1.0 } else { 1.3 },
            hgt * 0.4 * u,
            f(col),
            clip,
        );
        if a.species % 8 == 5 {
            ellipse(
                fb,
                cx + d * 14.0 * u,
                by - hgt * 0.38 * u,
                len * 0.32 * u,
                hgt * 0.3 * u,
                f(col2),
                clip,
            );
            ellipse(
                fb,
                cx,
                by - hgt * 0.86 * u,
                len * 0.3 * u,
                hgt * 0.14 * u,
                f(col),
                clip,
            );
            tri(
                fb,
                cx + d * 48.0 * u,
                by - hgt * 0.9 * u,
                by - hgt * 0.8 * u,
                18.0 * u,
                f(rgb(250, 160, 30)),
                clip,
            );
        } else {
            ellipse(
                fb,
                cx + d * 30.0 * u,
                by - hgt * 0.92 * u,
                56.0 * u,
                70.0 * u,
                f(col),
                clip,
            );
            seg_line(
                fb,
                (cx + d * 40.0 * u, by - hgt * 1.05 * u),
                (cx + d * 50.0 * u, by - hgt * 1.25 * u),
                14.0 * u,
                clip,
                f(col),
            );
            seg_line(
                fb,
                (cx - d * 60.0 * u, by - 30.0 * u),
                (cx - d * 300.0 * u, by - 10.0 * u),
                24.0 * u,
                clip,
                f(col2),
            );
        }
    }
}

/// Jump ramps (3 sizes) and turbo pads.
pub(super) fn draw_ramp_or_pad(
    fb: &mut [u32],
    kind: u8,
    x: f32,
    y: f32,
    u: f32,
    clip: f32,
    t: f32,
    f: &dyn Fn(u32) -> u32,
) {
    match kind {
        I_PAD => {
            let pulse = 0.55 + 0.45 * (t * 8.0).sin();
            glow(
                fb,
                x,
                y - 20.0 * u,
                420.0 * u,
                110.0 * u,
                f(rgb(40, 200, 255)),
                0.35 * pulse,
                clip,
            );
            quad(
                fb,
                y - 70.0 * u,
                x - 260.0 * u,
                x + 260.0 * u,
                y,
                x - 330.0 * u,
                x + 330.0 * u,
                clip,
                f(rgb(18, 40, 92)),
            );
            for i in 0..3 {
                let yy = y - (12.0 + i as f32 * 22.0) * u;
                let c = mix(
                    rgb(40, 190, 255),
                    rgb(255, 255, 255),
                    ((t * 6.0 - i as f32).sin() * 0.5 + 0.5) * 0.7,
                );
                poly4(
                    fb,
                    [
                        (x, yy - 20.0 * u),
                        (x + 230.0 * u, yy),
                        (x + 230.0 * u, yy + 12.0 * u),
                        (x, yy - 8.0 * u),
                    ],
                    clip,
                    f(c),
                );
                poly4(
                    fb,
                    [
                        (x, yy - 20.0 * u),
                        (x - 230.0 * u, yy),
                        (x - 230.0 * u, yy + 12.0 * u),
                        (x, yy - 8.0 * u),
                    ],
                    clip,
                    f(c),
                );
            }
        }
        _ => {
            let (w, h, col) = match kind {
                I_RAMP_S => (300.0, 130.0, rgb(70, 190, 90)),
                I_RAMP_M => (360.0, 280.0, rgb(255, 150, 30)),
                _ => (430.0, 500.0, rgb(230, 50, 50)),
            };
            // side faces, sloped deck and a striped lip
            poly4(
                fb,
                [
                    (x - w * u, y),
                    (x - w * 0.8 * u, y - h * u),
                    (x - w * 0.8 * u, y - h * u * 0.9),
                    (x - w * u, y),
                ],
                clip,
                f(shade(col, 0.5)),
            );
            quad(
                fb,
                y - h * u,
                x - w * 0.8 * u,
                x + w * 0.8 * u,
                y,
                x - w * u,
                x + w * u,
                clip,
                f(col),
            );
            quad(
                fb,
                y - h * u,
                x - w * 0.8 * u,
                x + w * 0.8 * u,
                y - h * 0.86 * u,
                x - w * 0.82 * u,
                x + w * 0.82 * u,
                clip,
                f(rgb(250, 250, 250)),
            );
            for i in 0..5 {
                let a = x - w * 0.8 * u + (i as f32 * 2.0 + 1.0) * w * 0.16 * u;
                rect(
                    fb,
                    a,
                    y - h * u,
                    a + w * 0.16 * u,
                    y - h * 0.86 * u,
                    f(rgb(30, 30, 34)),
                    clip,
                );
            }
            quad(
                fb,
                y - h * 0.5 * u,
                x - w * 0.9 * u,
                x + w * 0.9 * u,
                y - h * 0.42 * u,
                x - w * 0.92 * u,
                x + w * 0.92 * u,
                clip,
                f(shade(col, 0.8)),
            );
        }
    }
}

/// A roadside fuel station: canopy, two pumps and a sign.
pub(super) fn draw_station(
    fb: &mut [u32],
    cx: f32,
    by: f32,
    u: f32,
    clip: f32,
    t: f32,
    f: &dyn Fn(u32) -> u32,
) {
    if u < 0.004 {
        return;
    }
    rect(
        fb,
        cx - 560.0 * u,
        by - 940.0 * u,
        cx + 560.0 * u,
        by - 820.0 * u,
        f(rgb(230, 70, 40)),
        clip,
    );
    rect(
        fb,
        cx - 560.0 * u,
        by - 840.0 * u,
        cx + 560.0 * u,
        by - 820.0 * u,
        f(rgb(255, 230, 80)),
        clip,
    );
    for s in [-1.0f32, 1.0] {
        rect(
            fb,
            cx + s * 480.0 * u - 22.0 * u,
            by - 820.0 * u,
            cx + s * 480.0 * u + 22.0 * u,
            by,
            f(rgb(200, 200, 210)),
            clip,
        );
    }
    for s in [-1.0f32, 1.0] {
        let px = cx + s * 160.0 * u;
        rect(
            fb,
            px - 55.0 * u,
            by - 330.0 * u,
            px + 55.0 * u,
            by,
            f(rgb(240, 240, 244)),
            clip,
        );
        rect(
            fb,
            px - 55.0 * u,
            by - 330.0 * u,
            px + 55.0 * u,
            by - 230.0 * u,
            f(rgb(40, 140, 220)),
            clip,
        );
        rect(
            fb,
            px - 30.0 * u,
            by - 200.0 * u,
            px + 30.0 * u,
            by - 120.0 * u,
            f(rgb(30, 34, 50)),
            clip,
        );
    }
    glow(
        fb,
        cx,
        by - 760.0 * u,
        520.0 * u,
        120.0 * u,
        f(rgb(255, 240, 180)),
        0.35 + 0.1 * (t * 3.0).sin(),
        clip,
    );
}

/// A tall pole sign announcing a station ahead.
pub(super) fn draw_station_sign(
    fb: &mut [u32],
    cx: f32,
    by: f32,
    u: f32,
    clip: f32,
    f: &dyn Fn(u32) -> u32,
) {
    if u < 0.004 {
        return;
    }
    rect(
        fb,
        cx - 24.0 * u,
        by - 1500.0 * u,
        cx + 24.0 * u,
        by,
        f(rgb(120, 124, 140)),
        clip,
    );
    rect(
        fb,
        cx - 300.0 * u,
        by - 1700.0 * u,
        cx + 300.0 * u,
        by - 1250.0 * u,
        f(rgb(20, 130, 70)),
        clip,
    );
    rect(
        fb,
        cx - 270.0 * u,
        by - 1670.0 * u,
        cx + 270.0 * u,
        by - 1280.0 * u,
        f(rgb(255, 255, 255)),
        clip,
    );
    ellipse(
        fb,
        cx,
        by - 1450.0 * u,
        120.0 * u,
        140.0 * u,
        f(rgb(230, 70, 40)),
        clip,
    );
    tri(
        fb,
        cx,
        by - 1700.0 * u + 30.0 * u,
        by - 1560.0 * u,
        80.0 * u,
        f(rgb(230, 70, 40)),
        clip,
    );
}

/// Daylight: night tint on everything but tunnels, stars, sun and moon, headlight beams.
pub(super) fn day_night(fb: &mut [u32], g: &Game) {
    let l = g.daylight();
    let ss = g.sunset();
    let d = 1.0 - l;
    let (vwf, k) = (vw() as f32, vk());
    let hz = horizon() as f32;
    if d > 0.02 || ss > 0.02 {
        let night = [0.30f32, 0.38, 0.64];
        let warm = [1.0f32, 0.86, 0.72];
        let m: [f32; 3] = std::array::from_fn(|i| {
            let n = 1.0 + (night[i] - 1.0) * d;
            n * (1.0 + (warm[i] - 1.0) * ss * 0.55)
        });
        for px in fb.iter_mut() {
            let o = unpack(*px);
            *px = rgb(
                (o[0] * m[0]).min(255.0) as u8,
                (o[1] * m[1]).min(255.0) as u8,
                (o[2] * m[2]).min(255.0) as u8,
            );
        }
    }
    // stars
    if d > 0.25 {
        for i in 0..110u32 {
            let x = h01(i, 801) * vwf;
            let y = h01(i, 802) * hz * 0.85;
            let tw = 0.55 + 0.45 * (g.elapsed * (1.0 + h01(i, 803) * 2.0) + i as f32).sin();
            blend_px(fb, x as i32, y as i32, rgb(255, 255, 240), (d - 0.2) * tw);
        }
    }
    // sun and moon cross the sky
    let tod = g.tod;
    if (0.2..0.8).contains(&tod) {
        let u = (tod - 0.2) / 0.6;
        let sx = vwf * (0.08 + 0.84 * u);
        let sy = hz + 14.0 * k - (u * std::f32::consts::PI).sin() * hz * 0.8;
        let col = mix(
            rgb(255, 120, 50),
            rgb(255, 244, 200),
            (u * std::f32::consts::PI).sin().powf(0.5),
        );
        glow(fb, sx, sy, 60.0 * k, 60.0 * k, col, 0.5, hz);
        ellipse(fb, sx, sy, 11.0 * k, 11.0 * k, col, hz);
    } else {
        let m = if tod >= 0.8 {
            (tod - 0.8) / 0.4
        } else {
            (tod + 0.2) / 0.4
        };
        let sx = vwf * (0.08 + 0.84 * m.clamp(0.0, 1.0));
        let sy = hz + 14.0 * k - (m.clamp(0.0, 1.0) * std::f32::consts::PI).sin() * hz * 0.75;
        glow(fb, sx, sy, 40.0 * k, 40.0 * k, rgb(170, 190, 255), 0.35, hz);
        ellipse(fb, sx, sy, 8.0 * k, 8.0 * k, rgb(236, 240, 255), hz);
        ellipse(
            fb,
            sx + 3.0 * k,
            sy - 2.0 * k,
            6.5 * k,
            6.5 * k,
            rgb(60, 70, 120),
            hz,
        );
    }
    // headlights
    if d > 0.15 {
        let cx = vwf / 2.0 + g.steer * 6.0 * k;
        let vh_ = vh() as f32;
        glow(
            fb,
            cx,
            vh_ * 0.78,
            250.0 * k,
            90.0 * k,
            rgb(255, 238, 200),
            0.5 * d,
            vh_,
        );
        glow(
            fb,
            cx,
            vh_ * 0.60,
            170.0 * k,
            40.0 * k,
            rgb(255, 238, 200),
            0.30 * d,
            vh_,
        );
        glow(
            fb,
            cx,
            vh_ * 0.54,
            100.0 * k,
            24.0 * k,
            rgb(255, 238, 200),
            0.20 * d,
            vh_,
        );
    }
}
