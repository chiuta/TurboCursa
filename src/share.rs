//! Stage codes: everything needed to recreate a stage (seed, mode, vehicle, level, conditions and
//! the designer layout) in one short, typo-checked text, e.g. `TCS1-0001F-4A302-...`.
use crate::career::splitmix;
use crate::track::Layout;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageCode {
    pub seed: u32,
    /// game mode 0..=8
    pub mode: u8,
    pub class: u8,
    /// difficulty level 0..=4
    pub level: u8,
    /// menu indices: 0 = automatic, k = item k - 1
    pub weather: u8,
    pub surface: u8,
    /// 0 = automatic, 1 dawn, 2 day, 3 dusk, 4 night
    pub time: u8,
    pub layout: Layout,
}

impl Default for StageCode {
    fn default() -> Self {
        StageCode {
            seed: 1,
            mode: 0,
            class: 0,
            level: 1,
            weather: 0,
            surface: 0,
            time: 0,
            layout: Layout::default(),
        }
    }
}

impl StageCode {
    /// Clamps every field to its valid range.
    pub fn sanitized(mut self) -> StageCode {
        self.mode = self.mode.min(crate::game::MODE_COUNT as u8 - 1);
        self.class = self.class.min(crate::cars::CLASS_COUNT as u8 - 1);
        self.level = self.level.min(crate::game::LEVEL_COUNT as u8 - 1);
        self.weather = self.weather.min(crate::env::WEATHER_COUNT as u8);
        self.surface = self.surface.min(crate::env::SURFACE_COUNT as u8);
        self.time = self.time.min(4);
        self.layout = self.layout.sanitized();
        self
    }

    fn nibbles(&self) -> Vec<u8> {
        let mut n = Vec::with_capacity(27);
        for i in (0..8).rev() {
            n.push((self.seed >> (4 * i) & 15) as u8);
        }
        n.push(self.mode);
        n.extend([self.class >> 4, self.class & 15]);
        n.push(self.level);
        n.extend([self.weather >> 4, self.weather & 15]);
        n.push(self.surface);
        n.push(self.time);
        for b in self.layout.biomes {
            n.extend([b >> 4, b & 15]);
        }
        n.extend([self.layout.twist, self.layout.hills, self.layout.length]);
        n
    }

    /// `TCS1-` + 27 hex digits + 3 check digits, in groups of five.
    pub fn encode(&self) -> String {
        let n = self.sanitized().nibbles();
        let chk = check(&n);
        let all: Vec<u8> = n
            .into_iter()
            .chain([(chk >> 8) as u8, (chk >> 4 & 15) as u8, (chk & 15) as u8])
            .collect();
        let hex: String = all
            .iter()
            .map(|d| {
                char::from_digit(*d as u32, 16)
                    .unwrap()
                    .to_ascii_uppercase()
            })
            .collect();
        let groups: Vec<&str> = (0..hex.len())
            .step_by(5)
            .map(|i| &hex[i..(i + 5).min(hex.len())])
            .collect();
        format!("TCS1-{}", groups.join("-"))
    }

    /// Parses a code; `None` when it is malformed or the check digits do not match.
    pub fn decode(s: &str) -> Option<StageCode> {
        let s = s.trim();
        if s.len() > 64 {
            return None;
        }
        let body = s
            .strip_prefix("TCS1-")
            .or_else(|| s.strip_prefix("tcs1-"))?;
        let all: Vec<u8> = body
            .chars()
            .filter(|c| *c != '-' && !c.is_whitespace())
            .map(|c| c.to_digit(16).map(|d| d as u8))
            .collect::<Option<_>>()?;
        if all.len() != 30 {
            return None;
        }
        let (n, c) = all.split_at(27);
        let got = (c[0] as u32) << 8 | (c[1] as u32) << 4 | c[2] as u32;
        if got != check(n) {
            return None;
        }
        let seed = n[..8].iter().fold(0u32, |a, d| a << 4 | *d as u32);
        let code = StageCode {
            seed,
            mode: n[8],
            class: n[9] << 4 | n[10],
            level: n[11],
            weather: n[12] << 4 | n[13],
            surface: n[14],
            time: n[15],
            layout: Layout {
                biomes: [
                    n[16] << 4 | n[17],
                    n[18] << 4 | n[19],
                    n[20] << 4 | n[21],
                    n[22] << 4 | n[23],
                ],
                twist: n[24],
                hills: n[25],
                length: n[26],
            },
        };
        Some(code.sanitized())
    }

    /// Builds the game this code describes (single player).
    pub fn game(&self) -> crate::game::Game {
        let c = self.sanitized();
        let mut g = crate::track::with_layout(c.layout, || {
            crate::game::Game::new_mode(c.seed as u64, crate::game::Mode::from_index(c.mode as u32))
        });
        g.layout = c.layout;
        g.set_car(
            crate::cars::CarSpec::default()
                .with_class(crate::cars::VehicleClass::from_index(c.class as usize)),
        );
        g.set_level(crate::game::Difficulty::from_index(c.level as usize));
        g.set_env(c.weather as usize, c.surface as usize);
        g.set_time(c.time as usize);
        g
    }
}

fn check(n: &[u8]) -> u32 {
    let h = n
        .iter()
        .fold(0x5C5u64, |a, d| splitmix(a ^ (*d as u64 + 0x9E37)));
    (h & 0xFFF) as u32
}
