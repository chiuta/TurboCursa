use turbocursa::cars::*;
use turbocursa::garage::*;
use turbocursa::profile::GameResult;

// the upgrade levels are process-wide, so everything runs in one test
#[test]
fn garage_economy_and_effect() {
    let mut g = Garage::default();
    let r = GameResult {
        distance_frac: 1.0,
        finished: true,
        overtaken: 40,
        bonuses: 20,
        level: 4,
        ..Default::default()
    };
    let c = Garage::coins_for(&r);
    assert!(
        c > 40 && c < 200,
        "a perfect run earns a sensible amount: {c}"
    );
    assert_eq!(
        Garage::coins_for(&GameResult {
            disqualified: true,
            ..r
        }),
        0
    );
    assert_eq!(Garage::coins_for(&GameResult::default()), 0);

    assert!(!g.buy(0), "no coins, no upgrade");
    g.coins = 1000;
    for _ in 0..3 {
        assert!(g.buy(0));
    }
    assert!(!g.buy(0), "maxed out");
    assert_eq!((g.lv[0], g.price(0), g.price(9)), (3, None, None));
    assert_eq!(Garage::decode(&g.encode()), g);
    assert_eq!(Garage::decode("junk"), Garage::default());
    assert_eq!(
        Garage::decode("TG2;c=99999999999;u=99999").sanitized().lv,
        [3; 4]
    );

    // effect on the car, and none in stock mode
    let base = CarSpec::default();
    set_stock(false);
    Garage::default().apply();
    assert_eq!(upgrade(base).top, base.top);
    Garage {
        coins: 0,
        lv: [3, 3, 3, 3],
    }
    .apply();
    let up = upgrade(base);
    assert!(
        up.top > base.top
            && up.accel > base.accel
            && up.grip > base.grip
            && up.turbos > base.turbos
    );
    set_stock(true);
    assert_eq!(upgrade(base).top, base.top);
    // even the best garage cannot push a game past its sanity limits
    set_stock(false);
    let mut game = turbocursa::game::Game::new_mode(2, turbocursa::game::Mode::Campaign);
    game.set_car(base.with_class(VehicleClass::Formula));
    assert!(game.car.top <= 1.3 && game.car.turbos <= 6);
    // a spec handed to set_car again (a retry) is not upgraded a second time
    set_stock(false);
    Garage {
        coins: 0,
        lv: [2, 0, 0, 0],
    }
    .apply();
    let mut g2 = turbocursa::game::Game::new_mode(2, turbocursa::game::Mode::Campaign);
    g2.set_car(base);
    let once = g2.car.top;
    for _ in 0..5 {
        let c = g2.car;
        g2.set_car(c);
    }
    assert_eq!(g2.car.top, once, "retries must not compound the upgrades");
    set_stock(true);
    Garage::default().apply();
}
