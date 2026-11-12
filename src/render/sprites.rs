use super::*;

// ---------- sprites ----------

pub(super) const CAR_COLORS: [u32; 7] = [
    0xD92222, 0xF0C814, 0x1E6EE6, 0xEEEEEE, 0x28C85A, 0xE67814, 0x8C3CC8,
];

/// Obstacles lie on the road; bonuses float above it as glowing tokens.
pub(super) fn draw_item(
    fb: &mut [u32],
    it: &Item,
    x: f32,
    y: f32,
    u: f32,
    clip: f32,
    t: f32,
    fog: (u32, f32),
) {
    if u < 0.004 {
        return;
    }
    let f = |c: u32| mix(c, fog.0, fog.1);
    match it.kind {
        I_CONE => {
            let (w, h) = (110.0 * u, 300.0 * u);
            quad(
                fb,
                y - h,
                x - w * 0.18,
                x + w * 0.18,
                y,
                x - w,
                x + w,
                clip,
                f(rgb(255, 120, 24)),
            );
            quad(
                fb,
                y - h * 0.6,
                x - w * 0.5,
                x + w * 0.5,
                y - h * 0.4,
                x - w * 0.62,
                x + w * 0.62,
                clip,
                f(rgb(250, 250, 250)),
            );
            rect(
                fb,
                x - w * 1.15,
                y - h * 0.08,
                x + w * 1.15,
                y,
                f(rgb(60, 60, 66)),
                clip,
            );
        }
        I_OIL => {
            ellipse_a(
                fb,
                x,
                y - 25.0 * u,
                270.0 * u,
                62.0 * u,
                f(rgb(8, 8, 14)),
                0.92,
                clip,
            );
            ellipse_a(
                fb,
                x - 70.0 * u,
                y - 38.0 * u,
                110.0 * u,
                20.0 * u,
                f(hsv(0.55 + (t * 0.2).fract() * 0.4, 0.6, 0.9)),
                0.35,
                clip,
            );
        }
        I_BARRIER => {
            let (w, h) = (380.0 * u, 250.0 * u);
            rect(
                fb,
                x - w * 0.9,
                y - h,
                x - w * 0.82,
                y,
                f(rgb(70, 70, 78)),
                clip,
            );
            rect(
                fb,
                x + w * 0.82,
                y - h,
                x + w * 0.9,
                y,
                f(rgb(70, 70, 78)),
                clip,
            );
            for i in 0..8 {
                let (a, b) = (
                    x - w + w * 2.0 * i as f32 / 8.0,
                    x - w + w * 2.0 * (i as f32 + 1.0) / 8.0,
                );
                let c = if i % 2 == 0 {
                    rgb(236, 40, 44)
                } else {
                    rgb(245, 245, 245)
                };
                rect(fb, a, y - h, b, y - h * 0.55, f(c), clip);
            }
        }
        I_POTHOLE => {
            ellipse(
                fb,
                x,
                y - 20.0 * u,
                190.0 * u,
                46.0 * u,
                f(rgb(70, 70, 76)),
                clip,
            );
            ellipse(
                fb,
                x,
                y - 22.0 * u,
                150.0 * u,
                34.0 * u,
                f(rgb(12, 12, 16)),
                clip,
            );
        }
        I_PUDDLE => {
            ellipse_a(
                fb,
                x,
                y - 22.0 * u,
                330.0 * u,
                70.0 * u,
                f(rgb(60, 110, 190)),
                0.7,
                clip,
            );
            ellipse_a(
                fb,
                x - 90.0 * u,
                y - 36.0 * u,
                120.0 * u,
                18.0 * u,
                f(rgb(220, 240, 255)),
                0.5,
                clip,
            );
        }
        I_MUD => {
            ellipse(
                fb,
                x,
                y - 24.0 * u,
                360.0 * u,
                80.0 * u,
                f(rgb(92, 62, 36)),
                clip,
            );
            ellipse(
                fb,
                x + 60.0 * u,
                y - 30.0 * u,
                200.0 * u,
                40.0 * u,
                f(rgb(70, 46, 26)),
                clip,
            );
        }
        I_RAMP_S | I_RAMP_M | I_RAMP_L | I_PAD => {
            draw_ramp_or_pad(fb, it.kind, x, y, u, clip, t, &f)
        }
        I_CRACK => {
            // a fissure that opens as the car approaches
            let open = (1.0 - (u * 6.0).min(1.0)).max(0.15) + 0.2;
            let w = 330.0 * u * open.min(1.0);
            ellipse_a(
                fb,
                x,
                y - 22.0 * u,
                w * 1.1,
                46.0 * u,
                f(rgb(20, 60, 100)),
                0.55,
                clip,
            );
            for i in 0..5 {
                let a = (i as f32 - 2.0) * 0.5;
                quad(
                    fb,
                    y - 40.0 * u,
                    x + w * a * 0.5 - 6.0 * u,
                    x + w * a * 0.5 + 6.0 * u,
                    y,
                    x + w * a - 14.0 * u,
                    x + w * a + 14.0 * u,
                    clip,
                    f(rgb(8, 22, 44)),
                );
            }
            ellipse_a(
                fb,
                x,
                y - 22.0 * u,
                w * 0.6,
                14.0 * u,
                f(rgb(4, 12, 30)),
                0.9,
                clip,
            );
        }
        I_MAGMA => {
            let pulse = 0.75 + 0.25 * (t * 4.0 + it.z * 0.002).sin();
            glow(
                fb,
                x,
                y - 30.0 * u,
                380.0 * u,
                120.0 * u,
                f(rgb(255, 90, 10)),
                0.5 * pulse,
                clip,
            );
            ellipse(
                fb,
                x,
                y - 24.0 * u,
                300.0 * u,
                70.0 * u,
                f(rgb(150, 30, 6)),
                clip,
            );
            ellipse(
                fb,
                x,
                y - 28.0 * u,
                230.0 * u,
                48.0 * u,
                f(rgb(255, 120, 20)),
                clip,
            );
            ellipse(
                fb,
                x + 30.0 * u,
                y - 30.0 * u,
                110.0 * u,
                24.0 * u,
                f(rgb(255, 220, 90)),
                clip,
            );
        }
        k => {
            let bob = (t * 3.0 + it.z * 0.001).sin() * 28.0 * u;
            let cy = y - 360.0 * u + bob;
            let (col, glyph) = match k {
                I_TIME => (rgb(70, 220, 110), 0),
                I_TURBO => (rgb(255, 150, 30), 1),
                I_BOOST => (rgb(40, 210, 255), 2),
                I_SHIELD => (rgb(80, 120, 255), 3),
                I_COIN => (rgb(255, 210, 40), 4),
                I_FREEZE => (rgb(170, 230, 255), 5),
                _ => (rgb(240, 70, 220), 6),
            };
            glow(fb, x, cy, 430.0 * u, 430.0 * u, f(col), 0.55, clip);
            ellipse(fb, x, cy, 170.0 * u, 170.0 * u, f(col), clip);
            ellipse(fb, x, cy, 128.0 * u, 128.0 * u, f(shade(col, 0.72)), clip);
            let wh = f(rgb(255, 255, 255));
            match glyph {
                0 => {
                    // clock
                    rect(
                        fb,
                        x - 14.0 * u,
                        cy - 90.0 * u,
                        x + 14.0 * u,
                        cy + 6.0 * u,
                        wh,
                        clip,
                    );
                    rect(
                        fb,
                        x - 14.0 * u,
                        cy - 14.0 * u,
                        x + 70.0 * u,
                        cy + 14.0 * u,
                        wh,
                        clip,
                    );
                }
                1 => tri(fb, x, cy - 100.0 * u, cy + 90.0 * u, 70.0 * u, wh, clip),
                2 => {
                    tri(fb, x, cy - 100.0 * u, cy, 70.0 * u, wh, clip);
                    tri(fb, x, cy - 10.0 * u, cy + 90.0 * u, 70.0 * u, wh, clip);
                }
                3 => {
                    ellipse(fb, x, cy, 80.0 * u, 80.0 * u, wh, clip);
                    ellipse(fb, x, cy, 52.0 * u, 52.0 * u, f(shade(col, 0.72)), clip);
                }
                4 => ellipse(fb, x, cy, 74.0 * u, 74.0 * u, f(rgb(255, 245, 160)), clip),
                5 => {
                    rect(
                        fb,
                        x - 12.0 * u,
                        cy - 95.0 * u,
                        x + 12.0 * u,
                        cy + 95.0 * u,
                        wh,
                        clip,
                    );
                    rect(
                        fb,
                        x - 95.0 * u,
                        cy - 12.0 * u,
                        x + 95.0 * u,
                        cy + 12.0 * u,
                        wh,
                        clip,
                    );
                }
                _ => {
                    rect(
                        fb,
                        x - 90.0 * u,
                        cy - 14.0 * u,
                        x - 10.0 * u,
                        cy + 14.0 * u,
                        wh,
                        clip,
                    );
                    rect(
                        fb,
                        x + 10.0 * u,
                        cy - 14.0 * u,
                        x + 90.0 * u,
                        cy + 14.0 * u,
                        wh,
                        clip,
                    );
                    rect(
                        fb,
                        x - 14.0 * u,
                        cy - 90.0 * u,
                        x + 14.0 * u,
                        cy + 90.0 * u,
                        wh,
                        clip,
                    );
                }
            }
        }
    }
}

/// Tall vehicles are drawn smaller (and bikes larger) so the road stays visible.
pub(super) fn view_scale(c: crate::cars::VehicleClass) -> f32 {
    use crate::cars::VehicleClass as V;
    match c {
        V::Semi => 0.5,
        V::Truck | V::Pickup | V::Van => 0.64,
        V::Bus => 0.5,
        V::Tractor | V::Monster => 0.6,
        V::Moped | V::Motorcycle => 1.0,
        V::Trike | V::TukTuk => 1.0,
        _ => 1.0,
    }
}

/// Sprite family of a vehicle class.
pub(super) fn class_kind(c: crate::cars::VehicleClass) -> u8 {
    use crate::cars::VehicleClass as V;
    match c {
        V::Truck | V::Pickup | V::Van => K_TRUCK,
        V::Semi => K_SEMI,
        V::City => K_MICRO,
        V::Moped | V::Motorcycle => K_BIKE,
        V::Trike => K_TRIKE,
        V::TukTuk => K_TUKTUK,
        V::Quad => K_QUAD,
        V::Buggy => K_BUGGY,
        V::Formula => K_FORMULA,
        V::Bus => K_BUS,
        V::Tractor => K_TRACTOR,
        V::Limousine => K_LIMO,
        V::Monster => K_MONSTER,
        _ => K_CAR,
    }
}

/// Any vehicle seen from behind: kind 0 car, 1 truck, 2 semi-trailer, 3 microcar.
pub(super) fn draw_vehicle(
    fb: &mut [u32],
    cx: f32,
    by: f32,
    w: f32,
    color: u32,
    clip: f32,
    wing: bool,
    kind: u8,
) {
    match kind {
        crate::game::K_TRUCK => {
            let w = w * 1.1;
            draw_car(fb, cx, by, w, color, clip, false, true);
            flag_stripe(fb, cx, by - w * 0.58 * 1.15, by - w * 0.58 * 0.92, w, clip);
        }
        crate::game::K_SEMI => {
            let w = w * 1.25;
            draw_semi(fb, cx, by, w, color, clip);
            flag_stripe(fb, cx, by - w * 0.58 * 1.55, by - w * 0.58 * 1.3, w, clip);
        }
        crate::game::K_MICRO => {
            let w = w * 0.78;
            draw_car(fb, cx, by, w, color, clip, false, false);
            let h = w * 0.58;
            // boxy tall roof
            quad(
                fb,
                by - h * 1.16,
                cx - w * 0.22,
                cx + w * 0.22,
                by - h * 0.9,
                cx - w * 0.3,
                cx + w * 0.3,
                clip,
                shade(color, 0.9),
            );
            quad(
                fb,
                by - h * 1.1,
                cx - w * 0.17,
                cx + w * 0.17,
                by - h * 0.92,
                cx - w * 0.25,
                cx + w * 0.25,
                clip,
                rgb(18, 26, 46),
            );
        }
        K_BIKE => draw_bike(fb, cx, by, w, color, clip),
        K_TRIKE => draw_trike(fb, cx, by, w, color, clip, false),
        K_TUKTUK => draw_trike(fb, cx, by, w, color, clip, true),
        K_BUS => {
            let w = w * 1.2;
            draw_bus(fb, cx, by, w, color, clip);
            flag_stripe(fb, cx, by - w * 0.58 * 1.12, by - w * 0.58 * 0.95, w, clip);
        }
        K_TRACTOR => draw_tractor(fb, cx, by, w, color, clip),
        K_BUGGY => draw_buggy(fb, cx, by, w, color, clip, true),
        K_QUAD => draw_buggy(fb, cx, by, w * 0.8, color, clip, false),
        K_FORMULA => draw_formula(fb, cx, by, w, color, clip),
        K_MONSTER => draw_monster(fb, cx, by, w, color, clip),
        K_LIMO => draw_car(fb, cx, by, w * 1.12, color, clip, wing, false),
        _ => draw_car(fb, cx, by, w, color, clip, wing, false),
    }
}

pub(super) fn rider_color(color: u32) -> u32 {
    match LIVERY.with(|l| l.get()) {
        Some(c) => c.c2,
        None => shade(color, 0.55),
    }
}

/// Two-wheeler from behind: tyre, tail, rider and handlebars.
pub(super) fn draw_bike(fb: &mut [u32], cx: f32, by: f32, w: f32, color: u32, clip: f32) {
    if w < 1.5 {
        return;
    }
    ellipse_a(
        fb,
        cx,
        by - 0.5,
        w * 0.34,
        (w * 0.05).max(0.6),
        rgb(0, 0, 0),
        0.45,
        clip,
    );
    rect(
        fb,
        cx - w * 0.08,
        by - w * 0.36,
        cx + w * 0.08,
        by,
        rgb(14, 14, 18),
        clip,
    );
    vrect(
        fb,
        cx - w * 0.16,
        by - w * 0.52,
        cx + w * 0.16,
        by - w * 0.3,
        shade(color, 1.2),
        shade(color, 0.7),
        clip,
    );
    rect(
        fb,
        cx - w * 0.1,
        by - w * 0.47,
        cx + w * 0.1,
        by - w * 0.4,
        rgb(255, 40, 44),
        clip,
    );
    let jacket = rider_color(color);
    let jacket = if jacket == rgb(255, 255, 255) {
        shade(color, 0.55)
    } else {
        jacket
    };
    vrect(
        fb,
        cx - w * 0.18,
        by - w * 0.86,
        cx + w * 0.18,
        by - w * 0.5,
        shade(jacket, 1.1),
        shade(jacket, 0.7),
        clip,
    );
    rect(
        fb,
        cx - w * 0.3,
        by - w * 0.74,
        cx + w * 0.3,
        by - w * 0.68,
        rgb(30, 30, 36),
        clip,
    );
    ellipse(
        fb,
        cx,
        by - w * 0.95,
        w * 0.11,
        w * 0.11,
        shade(color, 1.1),
        clip,
    );
    rect(
        fb,
        cx - w * 0.07,
        by - w * 0.97,
        cx + w * 0.07,
        by - w * 0.92,
        rgb(18, 26, 46),
        clip,
    );
}

/// Three-wheeler: motorcycle front with a rear axle (tuk-tuk adds a canopy).
pub(super) fn draw_trike(
    fb: &mut [u32],
    cx: f32,
    by: f32,
    w: f32,
    color: u32,
    clip: f32,
    canopy: bool,
) {
    if w < 1.5 {
        return;
    }
    ellipse_a(
        fb,
        cx,
        by - 0.5,
        w * 0.5,
        (w * 0.06).max(0.6),
        rgb(0, 0, 0),
        0.45,
        clip,
    );
    let tyre = rgb(14, 14, 18);
    rect(
        fb,
        cx - w * 0.36,
        by - w * 0.3,
        cx - w * 0.24,
        by,
        tyre,
        clip,
    );
    rect(
        fb,
        cx + w * 0.24,
        by - w * 0.3,
        cx + w * 0.36,
        by,
        tyre,
        clip,
    );
    vrect(
        fb,
        cx - w * 0.32,
        by - w * 0.55,
        cx + w * 0.32,
        by - w * 0.2,
        shade(color, 1.15),
        shade(color, 0.7),
        clip,
    );
    rect(
        fb,
        cx - w * 0.28,
        by - w * 0.46,
        cx - w * 0.2,
        by - w * 0.36,
        rgb(255, 40, 44),
        clip,
    );
    rect(
        fb,
        cx + w * 0.2,
        by - w * 0.46,
        cx + w * 0.28,
        by - w * 0.36,
        rgb(255, 40, 44),
        clip,
    );
    if canopy {
        let r = rider_color(color);
        rect(
            fb,
            cx - w * 0.3,
            by - w * 0.6,
            cx - w * 0.27,
            by - w * 1.0,
            shade(color, 0.6),
            clip,
        );
        rect(
            fb,
            cx + w * 0.27,
            by - w * 0.6,
            cx + w * 0.3,
            by - w * 1.0,
            shade(color, 0.6),
            clip,
        );
        rect(
            fb,
            cx - w * 0.36,
            by - w * 1.08,
            cx + w * 0.36,
            by - w * 0.98,
            shade(r, 1.2),
            clip,
        );
        rect(
            fb,
            cx - w * 0.2,
            by - w * 0.9,
            cx + w * 0.2,
            by - w * 0.6,
            rgb(24, 30, 50),
            clip,
        );
    } else {
        let j = rider_color(color);
        vrect(
            fb,
            cx - w * 0.13,
            by - w * 1.0,
            cx + w * 0.13,
            by - w * 0.55,
            shade(j, 1.1),
            shade(j, 0.7),
            clip,
        );
        ellipse(
            fb,
            cx,
            by - w * 1.08,
            w * 0.09,
            w * 0.095,
            shade(color, 1.1),
            clip,
        );
    }
}

/// Coach / city bus: tall box with a glass band.
pub(super) fn draw_bus(fb: &mut [u32], cx: f32, by: f32, w: f32, color: u32, clip: f32) {
    if w < 1.5 {
        return;
    }
    let h = w * 0.58;
    ellipse_a(
        fb,
        cx,
        by - 0.5,
        w * 0.62,
        (w * 0.08).max(0.6),
        rgb(0, 0, 0),
        0.45,
        clip,
    );
    rect(
        fb,
        cx - w * 0.5,
        by - h * 0.42,
        cx - w * 0.33,
        by,
        rgb(14, 14, 18),
        clip,
    );
    rect(
        fb,
        cx + w * 0.33,
        by - h * 0.42,
        cx + w * 0.5,
        by,
        rgb(14, 14, 18),
        clip,
    );
    vrect(
        fb,
        cx - w * 0.5,
        by - h * 2.0,
        cx + w * 0.5,
        by - h * 0.3,
        shade(color, 1.1),
        shade(color, 0.72),
        clip,
    );
    rect(
        fb,
        cx - w * 0.44,
        by - h * 1.85,
        cx + w * 0.44,
        by - h * 1.3,
        rgb(26, 40, 66),
        clip,
    );
    for i in 1..4 {
        let x = cx - w * 0.44 + w * 0.88 * i as f32 / 4.0;
        rect(
            fb,
            x - w * 0.012,
            by - h * 1.85,
            x + w * 0.012,
            by - h * 1.3,
            shade(color, 0.7),
            clip,
        );
    }
    rect_a(
        fb,
        cx - w * 0.4,
        by - h * 1.8,
        cx - w * 0.1,
        by - h * 1.7,
        rgb(255, 255, 255),
        0.2,
        clip,
    );
    rect(
        fb,
        cx - w * 0.47,
        by - h * 0.7,
        cx - w * 0.36,
        by - h * 0.5,
        rgb(255, 40, 44),
        clip,
    );
    rect(
        fb,
        cx + w * 0.36,
        by - h * 0.7,
        cx + w * 0.47,
        by - h * 0.5,
        rgb(255, 40, 44),
        clip,
    );
}

/// Farm tractor from behind: huge rear tyres and a cab.
pub(super) fn draw_tractor(fb: &mut [u32], cx: f32, by: f32, w: f32, color: u32, clip: f32) {
    if w < 1.5 {
        return;
    }
    let h = w * 0.58;
    ellipse_a(
        fb,
        cx,
        by - 0.5,
        w * 0.6,
        (w * 0.08).max(0.6),
        rgb(0, 0, 0),
        0.45,
        clip,
    );
    rect(
        fb,
        cx - w * 0.5,
        by - h * 1.35,
        cx - w * 0.3,
        by,
        rgb(16, 16, 20),
        clip,
    );
    rect(
        fb,
        cx + w * 0.3,
        by - h * 1.35,
        cx + w * 0.5,
        by,
        rgb(16, 16, 20),
        clip,
    );
    for i in 0..4 {
        let y = by - h * (0.2 + 0.3 * i as f32);
        rect(
            fb,
            cx - w * 0.5,
            y,
            cx - w * 0.3,
            y + h * 0.05,
            rgb(40, 40, 46),
            clip,
        );
        rect(
            fb,
            cx + w * 0.3,
            y,
            cx + w * 0.5,
            y + h * 0.05,
            rgb(40, 40, 46),
            clip,
        );
    }
    vrect(
        fb,
        cx - w * 0.26,
        by - h * 1.9,
        cx + w * 0.26,
        by - h * 0.5,
        shade(color, 1.15),
        shade(color, 0.7),
        clip,
    );
    rect(
        fb,
        cx - w * 0.2,
        by - h * 1.75,
        cx + w * 0.2,
        by - h * 1.2,
        rgb(24, 32, 52),
        clip,
    );
    rect(
        fb,
        cx - w * 0.3,
        by - h * 1.98,
        cx + w * 0.3,
        by - h * 1.9,
        shade(color, 0.6),
        clip,
    );
    rect(
        fb,
        cx + w * 0.14,
        by - h * 2.5,
        cx + w * 0.18,
        by - h * 1.98,
        rgb(60, 60, 66),
        clip,
    );
}

/// Open buggy with a roll cage (cage=false: quad bike with handlebars).
pub(super) fn draw_buggy(
    fb: &mut [u32],
    cx: f32,
    by: f32,
    w: f32,
    color: u32,
    clip: f32,
    cage: bool,
) {
    if w < 1.5 {
        return;
    }
    let h = w * 0.58;
    ellipse_a(
        fb,
        cx,
        by - 0.5,
        w * 0.6,
        (w * 0.08).max(0.6),
        rgb(0, 0, 0),
        0.45,
        clip,
    );
    let tyre = rgb(14, 14, 18);
    rect(fb, cx - w * 0.5, by - h * 0.7, cx - w * 0.3, by, tyre, clip);
    rect(fb, cx + w * 0.3, by - h * 0.7, cx + w * 0.5, by, tyre, clip);
    vrect(
        fb,
        cx - w * 0.32,
        by - h * 0.62,
        cx + w * 0.32,
        by - h * 0.22,
        shade(color, 1.15),
        shade(color, 0.7),
        clip,
    );
    rect(
        fb,
        cx - w * 0.28,
        by - h * 0.55,
        cx - w * 0.2,
        by - h * 0.4,
        rgb(255, 40, 44),
        clip,
    );
    rect(
        fb,
        cx + w * 0.2,
        by - h * 0.55,
        cx + w * 0.28,
        by - h * 0.4,
        rgb(255, 40, 44),
        clip,
    );
    let j = rider_color(color);
    vrect(
        fb,
        cx - w * 0.12,
        by - h * 1.25,
        cx + w * 0.12,
        by - h * 0.6,
        shade(j, 1.1),
        shade(j, 0.7),
        clip,
    );
    ellipse(
        fb,
        cx,
        by - h * 1.4,
        w * 0.085,
        w * 0.09,
        shade(color, 1.1),
        clip,
    );
    if cage {
        let bar = rgb(210, 210, 216);
        rect(
            fb,
            cx - w * 0.3,
            by - h * 1.6,
            cx - w * 0.27,
            by - h * 0.6,
            bar,
            clip,
        );
        rect(
            fb,
            cx + w * 0.27,
            by - h * 1.6,
            cx + w * 0.3,
            by - h * 0.6,
            bar,
            clip,
        );
        rect(
            fb,
            cx - w * 0.3,
            by - h * 1.63,
            cx + w * 0.3,
            by - h * 1.57,
            bar,
            clip,
        );
    } else {
        rect(
            fb,
            cx - w * 0.28,
            by - h * 1.0,
            cx + w * 0.28,
            by - h * 0.94,
            rgb(30, 30, 36),
            clip,
        );
    }
}

/// Open-wheel single-seater: slim body, fat rear tyres, big wing.
pub(super) fn draw_formula(fb: &mut [u32], cx: f32, by: f32, w: f32, color: u32, clip: f32) {
    if w < 1.5 {
        return;
    }
    let h = w * 0.58;
    ellipse_a(
        fb,
        cx,
        by - 0.5,
        w * 0.6,
        (w * 0.08).max(0.6),
        rgb(0, 0, 0),
        0.45,
        clip,
    );
    let tyre = rgb(14, 14, 18);
    rect(
        fb,
        cx - w * 0.5,
        by - h * 0.78,
        cx - w * 0.28,
        by,
        tyre,
        clip,
    );
    rect(
        fb,
        cx + w * 0.28,
        by - h * 0.78,
        cx + w * 0.5,
        by,
        tyre,
        clip,
    );
    vrect(
        fb,
        cx - w * 0.2,
        by - h * 0.6,
        cx + w * 0.2,
        by - h * 0.2,
        shade(color, 1.2),
        shade(color, 0.7),
        clip,
    );
    quad(
        fb,
        by - h * 0.95,
        cx - w * 0.08,
        cx + w * 0.08,
        by - h * 0.6,
        cx - w * 0.18,
        cx + w * 0.18,
        clip,
        shade(color, 0.9),
    );
    ellipse(
        fb,
        cx,
        by - h * 1.05,
        w * 0.07,
        w * 0.07,
        rgb(240, 240, 240),
        clip,
    );
    rect(
        fb,
        cx - w * 0.5,
        by - h * 1.2,
        cx + w * 0.5,
        by - h * 1.05,
        shade(color, 0.8),
        clip,
    );
    rect(
        fb,
        cx - w * 0.5,
        by - h * 1.3,
        cx - w * 0.46,
        by - h * 0.85,
        rgb(30, 30, 36),
        clip,
    );
    rect(
        fb,
        cx + w * 0.46,
        by - h * 1.3,
        cx + w * 0.5,
        by - h * 0.85,
        rgb(30, 30, 36),
        clip,
    );
    rect(
        fb,
        cx - w * 0.05,
        by - h * 0.5,
        cx + w * 0.05,
        by - h * 0.4,
        rgb(255, 40, 44),
        clip,
    );
}

/// Monster truck: a truck body perched on gigantic tyres.
pub(super) fn draw_monster(fb: &mut [u32], cx: f32, by: f32, w: f32, color: u32, clip: f32) {
    if w < 1.5 {
        return;
    }
    let h = w * 0.58;
    draw_car(fb, cx, by - h * 0.9, w * 0.82, color, clip, false, true);
    let tyre = rgb(14, 14, 18);
    rect(
        fb,
        cx - w * 0.55,
        by - h * 1.45,
        cx - w * 0.3,
        by,
        tyre,
        clip,
    );
    rect(
        fb,
        cx + w * 0.3,
        by - h * 1.45,
        cx + w * 0.55,
        by,
        tyre,
        clip,
    );
    for i in 0..5 {
        let y = by - h * (0.15 + 0.28 * i as f32);
        rect(
            fb,
            cx - w * 0.55,
            y,
            cx - w * 0.3,
            y + h * 0.06,
            rgb(44, 44, 50),
            clip,
        );
        rect(
            fb,
            cx + w * 0.3,
            y,
            cx + w * 0.55,
            y + h * 0.06,
            rgb(44, 44, 50),
            clip,
        );
    }
    rect(
        fb,
        cx - w * 0.3,
        by - h * 0.75,
        cx + w * 0.3,
        by - h * 0.65,
        rgb(70, 70, 78),
        clip,
    );
}

/// The country's colours as three bands (heavy vehicles carry them across the body).
pub(super) fn flag_stripe(fb: &mut [u32], cx: f32, y0: f32, y1: f32, w: f32, clip: f32) {
    if let Some(car) = LIVERY.with(|l| l.get()) {
        let x0 = cx - w * 0.5;
        let t = w / 3.0;
        for (i, c) in [car.body, car.c2, car.c3].into_iter().enumerate() {
            rect(
                fb,
                x0 + t * i as f32,
                y0,
                x0 + t * (i as f32 + 1.0),
                y1,
                c,
                clip,
            );
        }
    }
}

/// Tractor-trailer from behind: tall box trailer, rear doors, underride bar, twin wheels.
pub(super) fn draw_semi(fb: &mut [u32], cx: f32, by: f32, w: f32, color: u32, clip: f32) {
    if w < 1.5 {
        return;
    }
    let h = w * 0.58;
    ellipse_a(
        fb,
        cx,
        by - 0.5,
        w * 0.64,
        (w * 0.08).max(0.6),
        rgb(0, 0, 0),
        0.45,
        clip,
    );
    let wheel = rgb(14, 14, 18);
    for (a, b) in [(0.5, 0.36), (0.33, 0.2)] {
        rect(fb, cx - w * a, by - h * 0.42, cx - w * b, by, wheel, clip);
        rect(fb, cx + w * b, by - h * 0.42, cx + w * a, by, wheel, clip);
    }
    // cab roof peeking over the trailer
    rect(
        fb,
        cx - w * 0.3,
        by - h * 2.45,
        cx + w * 0.3,
        by - h * 2.1,
        shade(color, 0.8),
        clip,
    );
    // trailer
    vrect(
        fb,
        cx - w * 0.5,
        by - h * 2.15,
        cx + w * 0.5,
        by - h * 0.45,
        rgb(214, 216, 222),
        rgb(150, 154, 166),
        clip,
    );
    rect(
        fb,
        cx - w * 0.5,
        by - h * 1.55,
        cx + w * 0.5,
        by - h * 1.3,
        shade(color, 1.0),
        clip,
    );
    rect(
        fb,
        cx - w * 0.015,
        by - h * 2.1,
        cx + w * 0.015,
        by - h * 0.5,
        rgb(96, 100, 112),
        clip,
    );
    // reflective strip + underride bar
    rect(
        fb,
        cx - w * 0.5,
        by - h * 0.62,
        cx + w * 0.5,
        by - h * 0.5,
        rgb(235, 40, 44),
        clip,
    );
    rect(
        fb,
        cx - w * 0.5,
        by - h * 0.45,
        cx + w * 0.5,
        by - h * 0.3,
        rgb(40, 42, 50),
        clip,
    );
    let red = rgb(255, 40, 44);
    rect(
        fb,
        cx - w * 0.48,
        by - h * 0.95,
        cx - w * 0.38,
        by - h * 0.7,
        red,
        clip,
    );
    rect(
        fb,
        cx + w * 0.38,
        by - h * 0.95,
        cx + w * 0.48,
        by - h * 0.7,
        red,
        clip,
    );
}

pub(super) fn draw_car(
    fb: &mut [u32],
    cx: f32,
    by: f32,
    w: f32,
    color: u32,
    clip: f32,
    big_wing: bool,
    truck: bool,
) {
    if w < 1.5 {
        return;
    }
    let h = w * 0.58;
    ellipse_a(
        fb,
        cx,
        by - 0.5,
        w * 0.62,
        (w * 0.08).max(0.6),
        rgb(0, 0, 0),
        0.45,
        clip,
    );
    let wheel = rgb(14, 14, 18);
    rect(
        fb,
        cx - w * 0.5,
        by - h * 0.42,
        cx - w * 0.33,
        by,
        wheel,
        clip,
    );
    rect(
        fb,
        cx + w * 0.33,
        by - h * 0.42,
        cx + w * 0.5,
        by,
        wheel,
        clip,
    );
    let red = rgb(255, 40, 44);
    if truck {
        vrect(
            fb,
            cx - w * 0.5,
            by - h * 1.55,
            cx + w * 0.5,
            by - h * 0.3,
            shade(color, 1.05),
            shade(color, 0.75),
            clip,
        );
        rect(
            fb,
            cx - w * 0.5,
            by - h * 1.55,
            cx + w * 0.5,
            by - h * 1.48,
            shade(color, 1.25),
            clip,
        );
        rect(
            fb,
            cx - w * 0.04,
            by - h * 1.5,
            cx + w * 0.04,
            by - h * 0.35,
            shade(color, 0.6),
            clip,
        );
        rect(
            fb,
            cx - w * 0.47,
            by - h * 0.62,
            cx - w * 0.33,
            by - h * 0.42,
            red,
            clip,
        );
        rect(
            fb,
            cx + w * 0.33,
            by - h * 0.62,
            cx + w * 0.47,
            by - h * 0.42,
            red,
            clip,
        );
    } else {
        vrect(
            fb,
            cx - w * 0.5,
            by - h * 0.64,
            cx + w * 0.5,
            by - h * 0.2,
            shade(color, 1.2),
            shade(color, 0.7),
            clip,
        );
        quad(
            fb,
            by - h,
            cx - w * 0.26,
            cx + w * 0.26,
            by - h * 0.64,
            cx - w * 0.4,
            cx + w * 0.4,
            clip,
            shade(color, 0.85),
        );
        quad(
            fb,
            by - h * 0.92,
            cx - w * 0.2,
            cx + w * 0.2,
            by - h * 0.68,
            cx - w * 0.33,
            cx + w * 0.33,
            clip,
            rgb(18, 26, 46),
        );
        rect_a(
            fb,
            cx - w * 0.19,
            by - h * 0.9,
            cx + w * 0.05,
            by - h * 0.84,
            rgb(255, 255, 255),
            0.22,
            clip,
        );
        if let Some(car) = LIVERY.with(|l| l.get()) {
            livery(fb, cx, by, w, h, &car, clip);
        }
        // full-width LED bar + clusters
        rect(
            fb,
            cx - w * 0.46,
            by - h * 0.5,
            cx + w * 0.46,
            by - h * 0.46,
            red,
            clip,
        );
        rect(
            fb,
            cx - w * 0.47,
            by - h * 0.57,
            cx - w * 0.27,
            by - h * 0.38,
            red,
            clip,
        );
        rect(
            fb,
            cx + w * 0.27,
            by - h * 0.57,
            cx + w * 0.47,
            by - h * 0.38,
            red,
            clip,
        );
        rect(
            fb,
            cx - w * 0.47,
            by - h * 0.57,
            cx - w * 0.27,
            by - h * 0.53,
            rgb(255, 150, 140),
            clip,
        );
        rect(
            fb,
            cx + w * 0.27,
            by - h * 0.57,
            cx + w * 0.47,
            by - h * 0.53,
            rgb(255, 150, 140),
            clip,
        );
        rect(
            fb,
            cx - w * 0.5,
            by - h * 0.26,
            cx + w * 0.5,
            by - h * 0.12,
            shade(color, 0.4),
            clip,
        );
        rect(
            fb,
            cx - w * 0.1,
            by - h * 0.36,
            cx + w * 0.1,
            by - h * 0.28,
            rgb(220, 220, 225),
            clip,
        );
        if big_wing {
            vrect(
                fb,
                cx - w * 0.5,
                by - h * 1.06,
                cx + w * 0.5,
                by - h * 0.94,
                shade(color, 0.75),
                shade(color, 0.45),
                clip,
            );
            rect(
                fb,
                cx - w * 0.4,
                by - h * 0.94,
                cx - w * 0.34,
                by - h * 0.7,
                shade(color, 0.45),
                clip,
            );
            rect(
                fb,
                cx + w * 0.34,
                by - h * 0.94,
                cx + w * 0.4,
                by - h * 0.7,
                shade(color, 0.45),
                clip,
            );
        }
    }
    if w > 5.0 {
        for s in [-1.0f32, 1.0] {
            glow(
                fb,
                cx + s * w * 0.37,
                by - h * 0.47,
                w * 0.42,
                w * 0.24,
                red,
                0.85,
                clip,
            );
        }
    }
}

/// Which of the three base asset sets (forest / desert / dusk) a biome borrows its trees and rocks from.
pub(super) fn asset_set(biome: u8) -> usize {
    match biome {
        0 | 3 | 5 => 0,
        1 | 4 => 1,
        _ => 2,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_prop(
    fb: &mut [u32],
    kind: Prop,
    cx: f32,
    by: f32,
    ux: f32,
    clip: f32,
    biome: u8,
    off: f32,
    assets: &Assets,
    variant: u8,
    fog: (u32, f32),
) {
    match kind {
        Prop::Lamp => {
            let dir = if off < 0.0 { 1.0 } else { -1.0 };
            let top = by - 1250.0 * ux;
            rect(
                fb,
                cx - 28.0 * ux,
                top,
                cx + 28.0 * ux,
                by,
                rgb(52, 54, 66),
                clip,
            );
            let (x0, x1) = if dir > 0.0 {
                (cx, cx + 420.0 * ux)
            } else {
                (cx - 420.0 * ux, cx)
            };
            rect(
                fb,
                x0,
                top - 30.0 * ux,
                x1,
                top + 20.0 * ux,
                rgb(52, 54, 66),
                clip,
            );
            let hx = cx + dir * 420.0 * ux;
            ellipse(
                fb,
                hx,
                top + 30.0 * ux,
                140.0 * ux,
                55.0 * ux,
                rgb(255, 238, 180),
                clip,
            );
            glow(
                fb,
                hx,
                top + 40.0 * ux,
                520.0 * ux,
                380.0 * ux,
                rgb(255, 190, 100),
                0.6,
                clip,
            );
        }
        Prop::Pine => {
            let v = &assets.conifer[asset_set(biome)];
            sprite_tree(
                fb,
                &v[variant as usize % v.len()],
                cx,
                by,
                ux,
                1750.0,
                clip,
                fog,
            );
        }
        Prop::Round => {
            let v = &assets.broadleaf;
            sprite_tree(
                fb,
                &v[variant as usize % v.len()],
                cx,
                by,
                ux,
                1550.0,
                clip,
                fog,
            );
        }
        Prop::Dead => {
            let v = &assets.dead[asset_set(biome)];
            sprite_tree(
                fb,
                &v[variant as usize % v.len()],
                cx,
                by,
                ux,
                1450.0,
                clip,
                fog,
            );
        }
        Prop::Rock => {
            let v = &assets.rocks[asset_set(biome)];
            let img = &v[variant as usize % v.len()];
            let dw = 780.0 * ux;
            let dh = dw * 0.75;
            blit(
                fb,
                img,
                (0.0, 0.0, 1.0, 1.0),
                cx - dw / 2.0,
                by - dh * 0.9,
                dw,
                dh,
                clip,
                fog,
            );
        }
        Prop::Cactus => {
            let g = rgb(40, 130, 60);
            rect(
                fb,
                cx - 60.0 * ux,
                by - 900.0 * ux,
                cx + 60.0 * ux,
                by,
                g,
                clip,
            );
            rect(
                fb,
                cx - 260.0 * ux,
                by - 640.0 * ux,
                cx - 60.0 * ux,
                by - 540.0 * ux,
                g,
                clip,
            );
            rect(
                fb,
                cx - 260.0 * ux,
                by - 760.0 * ux,
                cx - 190.0 * ux,
                by - 540.0 * ux,
                g,
                clip,
            );
            rect(
                fb,
                cx + 60.0 * ux,
                by - 520.0 * ux,
                cx + 250.0 * ux,
                by - 430.0 * ux,
                g,
                clip,
            );
            rect(
                fb,
                cx + 190.0 * ux,
                by - 680.0 * ux,
                cx + 250.0 * ux,
                by - 430.0 * ux,
                g,
                clip,
            );
        }
        Prop::Giant => {
            // colossal trunk, taller than the screen, with a flared root and cloud collars
            let h = 9000.0 * ux;
            let top = (by - h).max(0.0);
            let nv = variant as f32 * 7.3;
            for y in (top as i32)..(by.min(clip) as i32) {
                let t = ((by - y as f32) / h).clamp(0.0, 1.0);
                let hw = 1500.0 * ux * (1.0 - 0.45 * t) + 1900.0 * ux * (1.0 - t).powi(9);
                let wy = (by - y as f32) / ux.max(1e-3);
                for x in ((cx - hw) as i32).max(0)..((cx + hw) as i32).min(vw() as i32) {
                    let dxn = (x as f32 - cx) / hw;
                    let gr = fractal::fbm(dxn * 5.0 + nv, wy * 0.0016, 9, 3, 0);
                    let lit = 1.1 - 0.42 * (dxn * 0.5 + 0.5);
                    let k = (0.55 + 0.9 * gr) * lit;
                    let moss = ((1.0 - t) * 0.5 * (1.0 - gr)).clamp(0.0, 0.5);
                    let bark = mix(rgb(112, 86, 62), rgb(60, 110, 50), moss);
                    fb[y as usize * vw() + x as usize] = mix(shade(bark, k), fog.0, fog.1);
                }
            }
            for (wh, rx, ry, a) in [
                (3300.0f32, 2600.0f32, 520.0f32, 0.92f32),
                (5200.0, 2300.0, 380.0, 0.8),
                (7400.0, 2000.0, 300.0, 0.7),
            ] {
                for i in 0..7 {
                    let f = i as f32 / 6.0 - 0.5;
                    let jitter = fractal::fbm(f * 3.0 + wh, nv, 4, 2, 0) - 0.5;
                    ellipse_a(
                        fb,
                        cx + f * rx * 1.5 * ux,
                        by - (wh + jitter * 500.0) * ux,
                        (rx * 0.42 + 200.0 * jitter) * ux,
                        ry * ux,
                        mix(rgb(250, 252, 255), fog.0, fog.1),
                        a,
                        clip,
                    );
                }
            }
        }
        Prop::Bloom => {
            let pal_h = [
                rgb(255, 120, 190),
                rgb(255, 210, 80),
                rgb(120, 220, 255),
                rgb(200, 150, 255),
            ];
            let head = mix(pal_h[variant as usize % 4], fog.0, fog.1 * 0.6);
            let stem = mix(rgb(40, 130, 56), fog.0, fog.1);
            for i in 0..5 {
                let dx = (i as f32 - 2.0) * 150.0 * ux;
                let hh = (260.0 + 70.0 * ((variant as usize + i * 3) % 5) as f32) * ux;
                rect(
                    fb,
                    cx + dx - 12.0 * ux,
                    by - hh,
                    cx + dx + 12.0 * ux,
                    by,
                    stem,
                    clip,
                );
                ellipse(fb, cx + dx, by - hh, 64.0 * ux, 52.0 * ux, head, clip);
                glow(
                    fb,
                    cx + dx,
                    by - hh,
                    190.0 * ux,
                    160.0 * ux,
                    head,
                    0.35,
                    clip,
                );
            }
        }
        Prop::Spire => {
            // stratified red butte: talus skirt, three cliff tiers and a flat cap
            let w = (760.0 + 90.0 * (variant % 4) as f32) * ux;
            let h = (3300.0 + 380.0 * (variant % 5) as f32) * ux;
            let strata = [
                rgb(168, 72, 48),
                rgb(148, 58, 40),
                rgb(184, 92, 58),
                rgb(132, 52, 38),
            ];
            tri(
                fb,
                cx,
                by - h * 0.42,
                by,
                w * 1.9,
                mix(rgb(176, 84, 56), fog.0, fog.1),
                clip,
            );
            for i in 0..4 {
                let y1 = by - h * (0.30 + 0.175 * i as f32);
                let y0 = by - h * (0.30 + 0.175 * (i + 1) as f32);
                let ww = w * (1.0 - 0.03 * i as f32);
                rect(
                    fb,
                    cx - ww,
                    y0,
                    cx + ww,
                    y1,
                    mix(strata[(i + variant as usize) % 4], fog.0, fog.1),
                    clip,
                );
                rect(
                    fb,
                    cx + ww * 0.35,
                    y0,
                    cx + ww,
                    y1,
                    mix(shade(strata[(i + variant as usize) % 4], 0.7), fog.0, fog.1),
                    clip,
                );
            }
            rect(
                fb,
                cx - w * 1.04,
                by - h * 1.02,
                cx + w * 1.04,
                by - h * 0.97,
                mix(rgb(196, 106, 70), fog.0, fog.1),
                clip,
            );
        }
        Prop::Scrub => {
            let c1 = mix(rgb(112, 136, 92), fog.0, fog.1);
            let c2 = mix(rgb(140, 156, 108), fog.0, fog.1);
            ellipse(
                fb,
                cx - 120.0 * ux,
                by - 90.0 * ux,
                170.0 * ux,
                110.0 * ux,
                c1,
                clip,
            );
            ellipse(
                fb,
                cx + 110.0 * ux,
                by - 110.0 * ux,
                190.0 * ux,
                130.0 * ux,
                c1,
                clip,
            );
            ellipse(fb, cx, by - 150.0 * ux, 150.0 * ux, 100.0 * ux, c2, clip);
        }
        Prop::Hover => {
            // hovering rock chunk: grass cap, jagged underside, glow and a soft shadow on the ground
            let hover = 1100.0 * ux + 130.0 * ux * (g_t_hover(variant)).sin();
            let w = (520.0 + 60.0 * (variant % 4) as f32) * ux;
            let cy = by - hover;
            ellipse_a(
                fb,
                cx,
                by - 8.0 * ux,
                w * 0.9,
                40.0 * ux,
                rgb(0, 20, 20),
                0.35,
                clip,
            );
            let rows = 14;
            for r in 0..rows {
                let f = r as f32 / rows as f32;
                let jag = 1.0 + 0.25 * fractal::fbm(f * 6.0, variant as f32, 3, 2, 0) - 0.12;
                let hw = w * (1.0 - f).powf(0.85) * jag;
                let y0 = cy + f * 640.0 * ux;
                rect(
                    fb,
                    cx - hw,
                    y0,
                    cx + hw,
                    y0 + 640.0 * ux / rows as f32 + 1.0,
                    mix(shade(rgb(118, 102, 98), 1.1 - 0.5 * f), fog.0, fog.1),
                    clip,
                );
            }
            ellipse(
                fb,
                cx,
                cy,
                w * 1.02,
                90.0 * ux,
                mix(rgb(86, 196, 120), fog.0, fog.1),
                clip,
            );
            glow(
                fb,
                cx,
                cy + 620.0 * ux,
                w * 1.2,
                220.0 * ux,
                rgb(120, 230, 255),
                0.45,
                clip,
            );
        }
        Prop::Crystal => {
            let hue = [
                (rgb(80, 230, 240), rgb(210, 255, 255)),
                (rgb(220, 90, 240), rgb(255, 220, 255)),
                (rgb(110, 150, 255), rgb(230, 240, 255)),
            ][variant as usize % 3];
            for (i, (dx, hh, ww)) in [
                (-220.0f32, 900.0f32, 130.0f32),
                (0.0, 1500.0, 170.0),
                (230.0, 1000.0, 120.0),
                (90.0, 620.0, 90.0),
            ]
            .iter()
            .enumerate()
            {
                let c = mix(hue.0, fog.0, fog.1);
                let cxx = cx + dx * ux;
                tri(fb, cxx, by - hh * ux, by, ww * ux, shade(c, 0.85), clip);
                tri(
                    fb,
                    cxx - ww * 0.2 * ux,
                    by - hh * ux * 0.97,
                    by,
                    ww * 0.45 * ux,
                    mix(hue.1, fog.0, fog.1),
                    clip,
                );
                glow(
                    fb,
                    cxx,
                    by - hh * ux * 0.5,
                    ww * 3.2 * ux,
                    hh * 0.6 * ux,
                    hue.0,
                    0.32,
                    clip,
                );
                let _ = i;
            }
        }
        Prop::Obsidian => {
            let w = (620.0 + 80.0 * (variant % 4) as f32) * ux;
            let h = (3000.0 + 450.0 * (variant % 5) as f32) * ux;
            let dark = mix(rgb(30, 24, 32), fog.0, fog.1);
            tri(fb, cx - w * 0.7, by - h * 0.62, by, w * 0.7, dark, clip);
            tri(fb, cx, by - h, by, w, shade(dark, 1.15), clip);
            tri(
                fb,
                cx + w * 0.8,
                by - h * 0.5,
                by,
                w * 0.6,
                shade(dark, 0.8),
                clip,
            );
            for i in 0..4 {
                let yy = by - h * (0.2 + 0.18 * i as f32);
                rect(
                    fb,
                    cx - w * 0.04,
                    yy,
                    cx + w * 0.05,
                    yy + h * 0.1,
                    mix(rgb(255, 120, 30), fog.0, fog.1 * 0.5),
                    clip,
                );
            }
            glow(
                fb,
                cx,
                by - h * 0.1,
                w * 1.6,
                h * 0.2,
                rgb(255, 100, 20),
                0.5,
                clip,
            );
        }
        Prop::Kelp => {
            for strand in 0..3 {
                let sx = cx + (strand as f32 - 1.0) * 160.0 * ux;
                let mut x = sx;
                for i in 0..16 {
                    let y = by - i as f32 * 130.0 * ux;
                    let nx =
                        sx + (i as f32 * 0.55 + variant as f32 + strand as f32).sin() * 90.0 * ux;
                    let c = mix(
                        mix(rgb(20, 120, 60), rgb(120, 220, 120), i as f32 / 16.0),
                        fog.0,
                        fog.1,
                    );
                    line(fb, x, y, nx, y - 130.0 * ux, (46.0 * ux).max(1.0), c, clip);
                    x = nx;
                }
            }
        }
        Prop::Coral => {
            let hue = [
                rgb(255, 110, 150),
                rgb(255, 160, 60),
                rgb(190, 110, 255),
                rgb(90, 230, 210),
            ][variant as usize % 4];
            let c = mix(hue, fog.0, fog.1);
            coral(
                fb,
                cx,
                by,
                560.0 * ux,
                0.0,
                (70.0 * ux).max(1.0),
                5,
                variant as u32 + 13,
                c,
                clip,
            );
        }
        Prop::Sierpinski => {
            let h = (2400.0 + 300.0 * (variant % 4) as f32) * ux;
            let neon = if variant.is_multiple_of(2) {
                rgb(70, 230, 255)
            } else {
                rgb(255, 70, 210)
            };
            sierp(
                fb,
                cx,
                by - h,
                by,
                h * 0.58,
                4,
                mix(shade(neon, 0.55), fog.0, fog.1),
                clip,
            );
            glow(fb, cx, by - h * 0.4, h * 0.7, h * 0.5, neon, 0.28, clip);
        }
        Prop::Menger => {
            let sz = (1500.0 + 200.0 * (variant % 4) as f32) * ux;
            let neon = if variant.is_multiple_of(2) {
                rgb(255, 70, 210)
            } else {
                rgb(70, 230, 255)
            };
            rect(
                fb,
                cx - sz / 2.0,
                by - sz,
                cx + sz / 2.0,
                by,
                rgb(14, 8, 30),
                clip,
            );
            carpet(
                fb,
                cx - sz / 2.0,
                by - sz,
                sz,
                3,
                mix(shade(neon, 0.6), fog.0, fog.1),
                clip,
            );
            glow(fb, cx, by - sz * 0.5, sz * 0.9, sz * 0.6, neon, 0.25, clip);
        }
        Prop::Mushroom => {
            let hue =
                [rgb(80, 255, 200), rgb(180, 110, 255), rgb(255, 120, 190)][variant as usize % 3];
            let hh = (1500.0 + 420.0 * (variant % 4) as f32) * ux;
            let r = (780.0 + 90.0 * (variant % 3) as f32) * ux;
            rect(
                fb,
                cx - r * 0.18,
                by - hh,
                cx + r * 0.18,
                by,
                mix(rgb(205, 200, 225), fog.0, fog.1),
                clip,
            );
            ellipse(
                fb,
                cx,
                by - hh,
                r,
                r * 0.55,
                mix(shade(hue, 0.7), fog.0, fog.1),
                clip,
            );
            for i in 0..6 {
                let (dx, dy) = (
                    (h01(i + variant as u32 * 9, 7) - 0.5) * 1.5,
                    h01(i + variant as u32 * 9, 8) * 0.4,
                );
                ellipse(
                    fb,
                    cx + dx * r * 0.8,
                    by - hh - r * 0.18 + dy * r * 0.3,
                    r * 0.1,
                    r * 0.07,
                    mix(rgb(250, 250, 255), fog.0, fog.1),
                    clip,
                );
            }
            glow(fb, cx, by - hh + r * 0.2, r * 1.5, r * 0.8, hue, 0.4, clip);
        }
        Prop::Sign => {
            rect(
                fb,
                cx - 25.0 * ux,
                by - 520.0 * ux,
                cx + 25.0 * ux,
                by,
                rgb(80, 80, 80),
                clip,
            );
            quad(
                fb,
                by - 900.0 * ux,
                cx,
                cx,
                by - 640.0 * ux,
                cx - 280.0 * ux,
                cx + 280.0 * ux,
                clip,
                rgb(20, 20, 20),
            );
            quad(
                fb,
                by - 860.0 * ux,
                cx,
                cx,
                by - 670.0 * ux,
                cx - 220.0 * ux,
                cx + 220.0 * ux,
                clip,
                rgb(250, 200, 20),
            );
            rect(
                fb,
                cx - 18.0 * ux,
                by - 800.0 * ux,
                cx + 18.0 * ux,
                by - 730.0 * ux,
                rgb(20, 20, 20),
                clip,
            );
        }
        Prop::Gate | Prop::Finish => {
            let post = rgb(230, 230, 230);
            let half = ROAD_W * 1.12 * ux;
            rect(
                fb,
                cx - half - 60.0 * ux,
                by - 1500.0 * ux,
                cx - half,
                by,
                post,
                clip,
            );
            rect(
                fb,
                cx + half,
                by - 1500.0 * ux,
                cx + half + 60.0 * ux,
                by,
                post,
                clip,
            );
            let (y0, y1) = (by - 1700.0 * ux, by - 1350.0 * ux);
            if kind == Prop::Finish {
                let cells = 16;
                let cw = (2.0 * half + 120.0 * ux) / cells as f32;
                for i in 0..cells {
                    for r in 0..2 {
                        let c = if (i + r) % 2 == 0 {
                            rgb(240, 240, 240)
                        } else {
                            rgb(20, 20, 20)
                        };
                        let x = cx - half - 60.0 * ux + cw * i as f32;
                        rect(
                            fb,
                            x,
                            lerp(y0, y1, r as f32 / 2.0),
                            x + cw,
                            lerp(y0, y1, (r + 1) as f32 / 2.0),
                            c,
                            clip,
                        );
                    }
                }
            } else {
                let g = &assets.gate[variant as usize % assets.gate.len()];
                let (l, wdt) = (cx - half - 60.0 * ux, 2.0 * half + 120.0 * ux);
                blit(fb, g, (0.0, 0.0, 1.0, 1.0), l, y0, wdt, y1 - y0, clip, fog);
                glow(
                    fb,
                    cx,
                    (y0 + y1) * 0.5,
                    wdt * 0.55,
                    (y1 - y0) * 1.4,
                    rgb(140, 120, 255),
                    0.35,
                    clip,
                );
                rect(
                    fb,
                    l,
                    y0,
                    l + wdt,
                    y0 + 3.0 * ux * 6.0,
                    rgb(235, 235, 245),
                    clip,
                );
                rect(
                    fb,
                    l,
                    y1 - 3.0 * ux * 6.0,
                    l + wdt,
                    y1,
                    rgb(235, 235, 245),
                    clip,
                );
            }
        }
    }
}
