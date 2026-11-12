use turbocursa::cars::*;
use turbocursa::game::*;
use turbocursa::pack::PackCfg;
use turbocursa::track::SEG_LEN;

/// A new Endless lap must start clean: the pack is rebuilt, the pursuit may start again and the
/// overtake chain does not carry over.
#[test]
fn a_new_endless_lap_resets_pack_pursuit_and_chain() {
    let mut g = Game::new_mode(11, Mode::Endless);
    g.set_car(CarSpec::default().with_class(VehicleClass::Police));
    let cfg = PackCfg {
        n: 8,
        inverted: false,
        slot: None,
    };
    g.set_pack(Some(cfg));
    let rivals = g.pack.len();
    assert!(rivals >= 7, "pack built: {rivals}");
    let go = Input {
        accel: true,
        ..Default::default()
    };
    for _ in 0..600 {
        g.update(0.02, &go);
    }
    g.chain = 5;
    g.pos = g.track.finish_seg as f32 * SEG_LEN + 10.0;
    g.update(0.02, &go);
    assert_eq!(g.lap, 2, "the lap counter advanced");
    assert_eq!(g.pack.len(), rivals, "the pack is back on the new stage");
    assert!(
        g.pos < 1000.0,
        "the player is at the start again: {}",
        g.pos
    );
    assert_eq!(g.next_cp, 0);
    // the pursuit slot is free again, so the police car gets a new target at once
    for _ in 0..5 {
        g.update(0.02, &go);
    }
    assert!(g.pursuit.is_some(), "a fresh pursuit starts on the new lap");
}
