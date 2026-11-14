use turbocursa::hd::{Res, Upscaler, HD_H, HD_W, PIC_W};
use turbocursa::{H, W};

fn frame(f: impl Fn(usize, usize) -> u32) -> Vec<u32> {
    (0..W * H).map(|i| f(i % W, i / W)).collect()
}

#[test]
fn a_flat_picture_stays_flat_with_black_side_bars() {
    let src = frame(|_, _| 0x00_33_66_99);
    let mut dst = vec![1u32; HD_W * HD_H];
    Upscaler::default().run(&src, &mut dst);
    let bar = (HD_W - PIC_W) / 2;
    for y in [0, 500, HD_H - 1] {
        assert_eq!(dst[y * HD_W], 0, "left bar");
        assert_eq!(dst[y * HD_W + HD_W - 1], 0, "right bar");
        assert_eq!(dst[y * HD_W + bar], 0x00_33_66_99);
        assert_eq!(dst[y * HD_W + bar + PIC_W - 1], 0x00_33_66_99);
    }
}

#[test]
fn edges_stay_crisp_and_nothing_is_invented() {
    // black and white stripes, one source pixel wide
    let src = frame(|x, _| if x % 2 == 0 { 0 } else { 0x00FF_FFFF });
    let mut dst = vec![0u32; HD_W * HD_H];
    Upscaler::default().run(&src, &mut dst);
    let bar = (HD_W - PIC_W) / 2;
    let row = &dst[540 * HD_W + bar..540 * HD_W + bar + PIC_W];
    let pure = row.iter().filter(|&&p| p == 0 || p == 0x00FF_FFFF).count();
    assert!(
        pure * 100 / PIC_W >= 55,
        "most pixels are pure, only edges blend"
    );
    assert!(row.iter().all(|&p| (p >> 24) == 0));
    // grey stays grey: the blend of black and white has equal channels
    assert!(row
        .iter()
        .all(|&p| (p & 255) == ((p >> 8) & 255) && (p & 255) == ((p >> 16) & 255)));
}

#[test]
fn it_is_fast_enough_for_sixty_frames_a_second() {
    let src = frame(|x, y| (x * 7 + y * 13) as u32 & 0x00FF_FFFF);
    let mut dst = vec![0u32; HD_W * HD_H];
    let mut up = Upscaler::default();
    up.run(&src, &mut dst); // warm up
    let t = std::time::Instant::now();
    for _ in 0..30 {
        up.run(&src, &mut dst);
    }
    let ms = t.elapsed().as_secs_f64() * 1000.0 / 30.0;
    eprintln!("upscale: {ms:.2} ms per frame");
    // generous: debug builds on a busy CI box are far slower than release
    let limit = if cfg!(debug_assertions) { 120.0 } else { 8.0 };
    assert!(ms < limit, "{ms} ms");
}

#[test]
fn short_buffers_are_ignored_not_panicked_on() {
    let mut up = Upscaler::default();
    let mut dst = vec![7u32; 100];
    up.run(&[0; 10], &mut dst);
    assert!(dst.iter().all(|&p| p == 7));
}

#[test]
fn every_resolution_scales_crisply_and_fills_the_frame_height() {
    let src = frame(|x, y| {
        if (x / 8 + y / 8) % 2 == 0 {
            0x00FF_8000
        } else {
            0x0000_80FF
        }
    });
    for res in Res::ALL {
        let (fw, fh) = res.frame();
        let (pw, ph) = res.pic();
        assert_eq!(ph, fh, "{res:?}");
        assert_eq!(pw * 5, fh * 8, "{res:?} keeps the 1.6:1 shape");
        assert!(pw <= fw);
        let mut dst = vec![1u32; fw * fh];
        let mut up = Upscaler::new(res);
        up.run(&src, &mut dst);
        let bar = (fw - pw) / 2;
        assert_eq!(dst[0], if bar > 0 { 0 } else { dst[0] }, "{res:?} bar");
        for y in [0, fh / 2, fh - 1] {
            let row = &dst[y * fw + bar..y * fw + bar + pw];
            let pure = row
                .iter()
                .filter(|&&p| p == 0x00FF_8000 || p == 0x0000_80FF)
                .count();
            assert!(pure * 100 / pw >= 70, "{res:?} line {y}: edges only blend");
        }
    }
}

#[test]
fn resolution_names_parse() {
    assert_eq!(Res::parse("4K"), Some(Res::K4));
    assert_eq!(Res::parse("1440"), Some(Res::K2));
    assert_eq!(Res::parse("fullhd"), Some(Res::Hd));
    assert_eq!(Res::parse("8k"), Some(Res::K8));
    assert_eq!(Res::parse("5k"), None);
}

#[test]
fn eight_k_costs_are_reported() {
    let src = frame(|x, y| (x * 7 + y * 13) as u32 & 0x00FF_FFFF);
    for res in [Res::K2, Res::K4, Res::K8] {
        let (fw, fh) = res.frame();
        let mut dst = vec![0u32; fw * fh];
        let mut up = Upscaler::new(res);
        up.run(&src, &mut dst);
        let t = std::time::Instant::now();
        for _ in 0..5 {
            up.run(&src, &mut dst);
        }
        eprintln!(
            "{}: {:.1} ms per frame",
            res.label(),
            t.elapsed().as_secs_f64() * 200.0
        );
    }
}
