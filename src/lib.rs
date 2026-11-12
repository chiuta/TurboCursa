//! TURBOCURSA — clean-room pseudo-3D arcade racer.
//! Everything here (engine, art, stage layout, name) is original work.
// Drawing helpers take many plain parameters on purpose; index loops over parallel arrays read best as they are.
#![allow(clippy::too_many_arguments, clippy::needless_range_loop)]
pub mod access;
pub mod audio;
pub mod badges;
pub mod bonus;
pub mod career;
pub mod career_ui;
pub mod cars;
pub mod champ;
pub mod city;
pub mod codes;
pub mod daily;
pub mod design;
pub mod env;
pub mod font;
pub mod fractal;
pub mod fx;
pub mod game;
pub mod gamepad;
pub mod garage;
pub mod ghost;
pub mod i18n;
pub mod i18n_career;
pub mod launch;
pub mod mission;
pub mod multi;
pub mod pack;
pub mod pick;
pub mod png;
pub mod profile;
pub mod pursuit;
pub mod rally;
pub mod render;
pub mod rng;
pub mod settings;
pub mod share;
pub mod track;
pub mod versus;
#[cfg(any(target_arch = "wasm32", feature = "web"))]
pub mod web;

/// Number of landscape biomes: forest, desert, dusk, world-tree grove, red mesas, floating isles, mirror flats,
/// aurora tundra, volcano, abyss, cosmos (Julia sky), neon citadel (Sierpinski / Menger), mycelium jungle,
/// plus six impossible/fractal worlds: inverted world, prism desert, clockwork, cloud sea, black hole, fractal canyon.
pub const NB: usize = 19;
pub const W: usize = 640;
pub const H: usize = 400;
/// Scale factor relative to the 320x200 design grid.
pub const K: f32 = W as f32 / 320.0;

/// Same stage for everyone on a given UTC day.
pub fn daily_seed(unix_secs: u64) -> u64 {
    100_000 + unix_secs / 86_400
}
