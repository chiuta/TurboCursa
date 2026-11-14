fn main() {
    let g = turbocursa::game::Game::new_mode(10, turbocursa::game::Mode::Endless);
    println!(
        "zones={:?} jams={:?} scenes={:?}",
        g.city.zones.iter().map(|z| (z.a, z.b)).collect::<Vec<_>>(),
        g.city.jams.iter().map(|j| (j.a, j.b)).collect::<Vec<_>>(),
        g.city
            .scenes
            .iter()
            .map(|z| (*z / 200.0) as i32)
            .collect::<Vec<_>>()
    );
}
