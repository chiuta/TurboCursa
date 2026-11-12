//! frames/vehicles_sheet.ppm: every vehicle class; frames/items_sheet.ppm: obstacles & bonuses; frames/scene.ppm: a race.
use turbocursa::cars::{CarSpec, VehicleClass, CLASS_COUNT};
use turbocursa::game::*;
use turbocursa::{render, H, W};

fn ppm(name: &str, fb: &[u32]) {
    let mut v = format!("P6\n{} {}\n255\n", W, H).into_bytes();
    for &c in fb {
        v.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
    }
    std::fs::write(format!("frames/{name}.ppm"), v).unwrap();
}

fn main() {
    std::fs::create_dir_all("frames").unwrap();
    // two sheets of 10 vehicles
    for sheet in 0..2 {
        let mut fb = vec![0x10163Au32; W * H];
        for k in 0..10 {
            let i = sheet * 10 + k;
            if i >= CLASS_COUNT {
                break;
            }
            let car = CarSpec::for_country(40).with_class(VehicleClass::from_index(i));
            let (cx, by) = (
                64.0 + (k % 5) as f32 * 128.0,
                170.0 + (k / 5) as f32 * 190.0,
            );
            render::draw_car_preview(&mut fb, W, H, cx, by, 110.0, &car);
        }
        ppm(&format!("vehicles_sheet{sheet}"), &fb);
    }
    // a scene with every item kind lined up ahead of the player
    let mut g = Game::new_mode(7, Mode::Campaign);
    g.items.clear();
    for k in 0..ITEM_KINDS {
        let z = 2900.0 + 520.0 * k as f32;
        g.items.push(Item {
            z,
            x: if k % 2 == 0 { -0.45 } else { 0.45 },
            kind: k,
            taken: false,
        });
    }
    g.cars.clear();
    g.pos = 2300.0;
    g.speed = 5000.0;
    g.shield_t = 4.0;
    g.boost_t = 2.0;
    g.x2_t = 6.0;
    let mut f = vec![0u32; W * H];
    render::render(&g, &mut f);
    ppm("items_scene", &f);
}
