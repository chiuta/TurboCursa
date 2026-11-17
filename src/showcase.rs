//! Showcase stages: twelve hand-picked stages with a fixed layout (landscapes in a chosen order,
//! twist, hills, length), vehicle, level, weather, surface and time of day. Unlike seeded stages
//! they are curated: each one is built to have a character, and its `TCS1-...` code never changes.
use crate::share::StageCode;
use crate::track::Layout;

/// Number of showcase stages.
pub const COUNT: usize = 12;

struct Def {
    seed: u32,
    class: u8,
    level: u8,
    weather: u8,
    surface: u8,
    time: u8,
    biomes: [u8; 4],
    twist: u8,
    hills: u8,
    length: u8,
}

const fn d(
    seed: u32,
    class: u8,
    level: u8,
    (weather, surface, time): (u8, u8, u8),
    biomes: [u8; 4],
    (twist, hills, length): (u8, u8, u8),
) -> Def {
    Def {
        seed,
        class,
        level,
        weather,
        surface,
        time,
        biomes,
        twist,
        hills,
        length,
    }
}

/// weather/surface/time are menu indices (0 = automatic, k = item k - 1).
const DEFS: [Def; COUNT] = [
    d(1001, 0, 1, (1, 1, 2), [0, 1, 2, 3], (1, 1, 1)), // classic sampler, clear day
    d(1002, 0, 2, (1, 1, 3), [4, 5, 6, 7], (3, 2, 1)), // twisty dusk
    d(1003, 0, 2, (2, 1, 4), [8, 9, 10, 11], (2, 3, 1)), // hilly night rain
    d(1004, 0, 1, (1, 1, 1), [12, 13, 14, 15], (0, 1, 2)), // long, fast dawn
    d(1005, 0, 3, (5, 2, 2), [2, 2, 2, 2], (1, 1, 1)), // snow on ice lake
    d(1006, 0, 3, (7, 5, 2), [1, 1, 1, 1], (1, 2, 2)), // sandstorm over dunes
    d(1007, 0, 3, (1, 4, 3), [3, 3, 3, 3], (2, 2, 1)), // lava at dusk
    d(1008, 0, 2, (4, 1, 4), [16, 17, 18, 0], (3, 1, 0)), // short, tight night fog
    d(1009, 0, 4, (3, 1, 4), [5, 7, 9, 11], (3, 3, 1)), // expert storm night
    d(1010, 0, 2, (1, 3, 2), [4, 4, 4, 4], (0, 0, 2)), // salt flat, flat and long
    d(1011, 0, 3, (6, 6, 3), [13, 14, 15, 16], (2, 2, 1)), // blizzard-free bog dusk
    d(1012, 0, 4, (10, 1, 4), [17, 18, 0, 1], (3, 3, 2)), // finale: ash fall, night
];

/// The showcase stage `n` (1-based) as a stage code, or `None` when out of range.
pub fn stage(n: usize) -> Option<StageCode> {
    let x = DEFS.get(n.checked_sub(1)?)?;
    Some(
        StageCode {
            seed: x.seed,
            mode: 0,
            class: x.class,
            level: x.level,
            weather: x.weather,
            surface: x.surface,
            time: x.time,
            layout: Layout {
                biomes: x.biomes,
                twist: x.twist,
                hills: x.hills,
                length: x.length,
            },
        }
        .sanitized(),
    )
}

/// All twelve codes, for listing (`turbo --showcase`).
pub fn codes() -> Vec<String> {
    (1..=COUNT).filter_map(stage).map(|c| c.encode()).collect()
}
