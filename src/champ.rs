//! Championships and country tours: a long event of 7, 10 or 15 stages (or the eight-stage tour of a
//! country), described by a short typo-checked code like `TCC1-...`, run by the rally engine.
use crate::career::splitmix;

pub const MAX_STAGES: usize = 15;
/// Stage counts by kind: short, medium, long championship and the country tour.
pub const KINDS: [usize; 4] = [7, 10, 15, 8];
pub const TOUR_KIND: u8 = 3;
/// Countries of the tour (a tour follows the climate of its country).
pub const COUNTRIES: [&str; 8] = [
    "ROMANIA", "FRANCE", "ITALY", "SPAIN", "PORTUGAL", "GERMANY", "HUNGARY", "FINLAND",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChampCode {
    pub seed: u32,
    /// 0..=3, index into `KINDS`
    pub kind: u8,
    pub class: u8,
    pub level: u8,
}

impl ChampCode {
    pub fn sanitized(mut self) -> ChampCode {
        self.kind = self.kind.min(KINDS.len() as u8 - 1);
        self.class = self.class.min(crate::cars::CLASS_COUNT as u8 - 1);
        self.level = self.level.min(crate::game::LEVEL_COUNT as u8 - 1);
        self
    }

    pub fn stages(&self) -> usize {
        KINDS[self.kind.min(KINDS.len() as u8 - 1) as usize]
    }

    /// Country of a tour (the seed picks it).
    pub fn country(&self) -> usize {
        (self.seed % COUNTRIES.len() as u32) as usize
    }

    fn nibbles(&self) -> Vec<u8> {
        let mut n: Vec<u8> = (0..8)
            .rev()
            .map(|i| (self.seed >> (4 * i) & 15) as u8)
            .collect();
        n.extend([self.kind, self.class >> 4, self.class & 15, self.level]);
        n
    }

    /// `TCC1-` + 12 hex digits + 3 check digits, in groups of five.
    pub fn encode(&self) -> String {
        let n = self.sanitized().nibbles();
        let c = check(&n);
        let hex: String = n
            .iter()
            .copied()
            .chain([(c >> 8) as u8, (c >> 4 & 15) as u8, (c & 15) as u8])
            .map(|d| char::from_digit(d as u32, 16).unwrap().to_ascii_uppercase())
            .collect();
        let g: Vec<&str> = (0..hex.len())
            .step_by(5)
            .map(|i| &hex[i..(i + 5).min(hex.len())])
            .collect();
        format!("TCC1-{}", g.join("-"))
    }

    pub fn decode(s: &str) -> Option<ChampCode> {
        let s = s.trim();
        if s.len() > 40 {
            return None;
        }
        let body = s
            .strip_prefix("TCC1-")
            .or_else(|| s.strip_prefix("tcc1-"))?;
        let all: Vec<u8> = body
            .chars()
            .filter(|c| *c != '-' && !c.is_whitespace())
            .map(|c| c.to_digit(16).map(|d| d as u8))
            .collect::<Option<_>>()?;
        if all.len() != 15 {
            return None;
        }
        let (n, c) = all.split_at(12);
        if (c[0] as u32) << 8 | (c[1] as u32) << 4 | c[2] as u32 != check(n) {
            return None;
        }
        Some(
            ChampCode {
                seed: n[..8].iter().fold(0u32, |a, d| a << 4 | *d as u32),
                kind: n[8],
                class: n[9] << 4 | n[10],
                level: n[11],
            }
            .sanitized(),
        )
    }

    /// The event this code describes.
    pub fn app(&self) -> crate::rally::RallyApp {
        let c = self.sanitized();
        let mut app = crate::rally::RallyApp::with_stages(
            c.seed as u64,
            c.class as usize,
            c.level as usize,
            c.stages(),
        );
        if c.kind == TOUR_KIND {
            app.country = Some(c.country());
        }
        app
    }

    /// A fresh code from a seed (what the menus hand out).
    pub fn from_seed(seed: u64, kind: u8, class: usize, level: usize) -> ChampCode {
        ChampCode {
            seed: (splitmix(seed ^ 0xC4A3) & 0xFFFF_FFFF) as u32,
            kind,
            class: class.min(255) as u8,
            level: level.min(255) as u8,
        }
        .sanitized()
    }
}

fn check(n: &[u8]) -> u32 {
    (n.iter()
        .fold(0xC1Cu64, |a, d| splitmix(a ^ (*d as u64 + 0x7A11)))
        & 0xFFF) as u32
}
