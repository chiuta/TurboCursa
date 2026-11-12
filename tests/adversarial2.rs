use turbocursa::game::Input;
use turbocursa::garage::Garage;
use turbocursa::ghost::*;
use turbocursa::pick::*;

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
}

#[test]
fn the_start_screen_survives_random_keys_on_every_page() {
    let mut r = Lcg(7);
    let mut p = PickApp::new(3, 2).with_extras(Extras {
        garage: Garage {
            coins: 5000,
            ..Default::default()
        },
        ..Default::default()
    });
    let mut fb = vec![0u32; turbocursa::W * turbocursa::H];
    let mut started = 0;
    for f in 0..20000 {
        let b = r.next();
        let inp = Input {
            left: b & 1 != 0,
            right: b & 2 != 0,
            accel: b & 4 != 0,
            brake: b & 8 != 0,
            turbo: b & 16 != 0,
        };
        if b & 0x700 == 0x700 {
            p.back();
        }
        let dt = [0.0, 0.016, 1e9, f32::NAN, -1.0][(b >> 12) as usize % 5];
        if p.update(dt, &inp, b & 0x20 != 0 && b & 0xC0 == 0).is_some() {
            started += 1;
        }
        if let Some(e) = p.take_extras() {
            assert_eq!(e, e.sanitized(), "frame {f}");
        }
        assert!(p.page <= 2);
        if f % 50 == 0 {
            p.render(&mut fb);
        }
    }
    assert!(started > 0);
}

#[test]
fn compact_ghost_codes_never_panic_and_stay_stable() {
    let mut r = Lcg(99);
    let alpha = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_;!";
    for _ in 0..20000 {
        let n = r.next() as usize % 80;
        let body: String = (0..n)
            .map(|_| alpha[r.next() as usize % alpha.len()] as char)
            .collect();
        for pre in [
            "TG2;1;0;0;10;5;",
            "TG2;",
            "TG2;18446744073709551615;255;255;4294967295;4294967295;",
        ] {
            if let Some(g) = Ghost::decode(&format!("{pre}{body}")) {
                assert!(g.pts.iter().all(|p| p.0.is_finite() && p.1.abs() <= 2.2));
                assert!(g.at(5.0).is_some() || g.pts.is_empty());
                let g2 = Ghost::decode(&g.encode()).expect("re-encodes");
                assert_eq!(g2.pts.len(), g.pts.len());
            }
        }
    }
}

#[test]
fn coins_cannot_overflow_or_go_negative() {
    let mut g = Garage {
        coins: u32::MAX,
        lv: [255; 4],
    }
    .sanitized();
    assert!(g.coins <= 9_999_999 && g.lv.iter().all(|&l| l <= 3));
    for i in 0..9 {
        assert!(!g.buy(i));
    }
    let huge = turbocursa::profile::GameResult {
        distance_frac: f32::NAN,
        overtaken: u32::MAX,
        bonuses: u32::MAX,
        level: usize::MAX,
        finished: true,
        ..Default::default()
    };
    let c = Garage::coins_for(&huge);
    assert!(c < 1000, "{c}");
}
