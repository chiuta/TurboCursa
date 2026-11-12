//! Weather and road surface: how they change grip, speed, wind and visibility.
use crate::rng::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weather {
    Clear,
    Rain,
    Storm,
    Fog,
    Snow,
    Blizzard,
    Sandstorm,
    Hail,
    /// freezing rain: black ice on the road
    Glaze,
    Heatwave,
    /// volcanic ash fall
    Ash,
}

pub const WEATHER_COUNT: usize = 11;

const WEATHERS: [Weather; WEATHER_COUNT] = [
    Weather::Clear,
    Weather::Rain,
    Weather::Storm,
    Weather::Fog,
    Weather::Snow,
    Weather::Blizzard,
    Weather::Sandstorm,
    Weather::Hail,
    Weather::Glaze,
    Weather::Heatwave,
    Weather::Ash,
];

const WNAMES: [&str; WEATHER_COUNT] = [
    "clear",
    "rain",
    "storm",
    "fog",
    "snow",
    "blizzard",
    "sandstorm",
    "hail",
    "glaze",
    "heatwave",
    "ash",
];

/// Effects of a weather: tyre grip, top-speed and acceleration multipliers, wind strength, clarity of the air (1 = clear).
#[derive(Clone, Copy, Debug)]
pub struct WeatherParams {
    pub grip: f32,
    pub top: f32,
    pub accel: f32,
    pub wind: f32,
    pub clarity: f32,
}

impl Weather {
    pub fn from_index(i: usize) -> Weather {
        WEATHERS[i.min(WEATHER_COUNT - 1)]
    }
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        WNAMES[self.index()]
    }
    pub fn parse(s: &str) -> Option<Weather> {
        let t = s.trim().to_ascii_lowercase();
        if let Ok(i) = t.parse::<usize>() {
            return (i < WEATHER_COUNT).then(|| Self::from_index(i));
        }
        WNAMES.iter().position(|n| *n == t).map(Self::from_index)
    }
    pub fn params(self) -> WeatherParams {
        let p = |grip, top, accel, wind, clarity| WeatherParams {
            grip,
            top,
            accel,
            wind,
            clarity,
        };
        match self {
            Weather::Clear => p(1.0, 1.0, 1.0, 0.0, 1.0),
            Weather::Rain => p(0.80, 0.98, 0.95, 0.15, 0.75),
            Weather::Storm => p(0.68, 0.94, 0.90, 0.80, 0.55),
            Weather::Fog => p(0.95, 1.0, 1.0, 0.0, 0.30),
            Weather::Snow => p(0.60, 0.95, 0.85, 0.20, 0.60),
            Weather::Blizzard => p(0.50, 0.88, 0.80, 1.00, 0.22),
            Weather::Sandstorm => p(0.85, 0.90, 0.90, 0.90, 0.28),
            Weather::Hail => p(0.75, 0.95, 0.92, 0.30, 0.65),
            Weather::Glaze => p(0.38, 0.97, 0.70, 0.10, 0.85),
            Weather::Heatwave => p(1.0, 0.97, 0.95, 0.0, 0.80),
            Weather::Ash => p(0.80, 0.95, 0.95, 0.20, 0.38),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Surface {
    Asphalt,
    /// frozen lake: slippery, with cracks and thin-ice zones that break if you stop
    IceLake,
    SaltFlat,
    Lava,
    Dunes,
    Bog,
}

pub const SURFACE_COUNT: usize = 6;

const SURFACES: [Surface; SURFACE_COUNT] = [
    Surface::Asphalt,
    Surface::IceLake,
    Surface::SaltFlat,
    Surface::Lava,
    Surface::Dunes,
    Surface::Bog,
];

const SNAMES: [&str; SURFACE_COUNT] = ["asphalt", "icelake", "saltflat", "lava", "dunes", "bog"];

impl Surface {
    pub fn from_index(i: usize) -> Surface {
        SURFACES[i.min(SURFACE_COUNT - 1)]
    }
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        SNAMES[self.index()]
    }
    pub fn parse(s: &str) -> Option<Surface> {
        let t = s.trim().to_ascii_lowercase();
        if let Ok(i) = t.parse::<usize>() {
            return (i < SURFACE_COUNT).then(|| Self::from_index(i));
        }
        SNAMES.iter().position(|n| *n == t).map(Self::from_index)
    }
    /// (grip, top speed, acceleration) multipliers
    pub fn params(self) -> (f32, f32, f32) {
        match self {
            Surface::Asphalt => (1.0, 1.0, 1.0),
            Surface::IceLake => (0.42, 0.98, 0.75),
            Surface::SaltFlat => (0.80, 1.0, 0.95),
            Surface::Lava => (0.90, 0.95, 0.95),
            Surface::Dunes => (0.75, 0.88, 0.80),
            Surface::Bog => (0.70, 0.80, 0.75),
        }
    }
}

fn pick<T: Copy>(rng: &mut Rng, w: &[(T, f32)]) -> T {
    let tot: f32 = w.iter().map(|x| x.1).sum();
    let mut r = rng.f() * tot;
    for (k, wt) in w {
        if r < *wt {
            return *k;
        }
        r -= wt;
    }
    w[0].0
}

/// Surface of a stage, derived from its seed (60 % plain asphalt).
pub fn auto_surface(seed: u64) -> Surface {
    let mut r = Rng::new(seed ^ 0x5F5FA11);
    pick(
        &mut r,
        &[
            (Surface::Asphalt, 0.60),
            (Surface::IceLake, 0.12),
            (Surface::SaltFlat, 0.07),
            (Surface::Lava, 0.07),
            (Surface::Dunes, 0.08),
            (Surface::Bog, 0.06),
        ],
    )
}

/// Weather of a stage, derived from its seed and biased by the surface it lies on.
pub fn auto_weather(seed: u64, surface: Surface) -> Weather {
    use Weather::*;
    let mut r = Rng::new(seed ^ 0xC10_0D5);
    let themed: &[(Weather, f32)] = match surface {
        Surface::IceLake => &[
            (Snow, 3.0),
            (Blizzard, 2.0),
            (Glaze, 2.0),
            (Clear, 2.0),
            (Fog, 1.0),
        ],
        Surface::Dunes => &[(Sandstorm, 3.0), (Heatwave, 3.0), (Clear, 2.0)],
        Surface::Lava => &[(Ash, 3.0), (Heatwave, 3.0), (Clear, 1.0), (Storm, 1.0)],
        Surface::SaltFlat => &[(Heatwave, 2.0), (Clear, 3.0), (Fog, 2.0)],
        Surface::Bog => &[(Rain, 3.0), (Fog, 2.0), (Storm, 1.0), (Clear, 1.0)],
        Surface::Asphalt => &[],
    };
    if !themed.is_empty() && r.f() < 0.75 {
        return pick(&mut r, themed);
    }
    pick(
        &mut r,
        &[
            (Clear, 0.28),
            (Rain, 0.13),
            (Storm, 0.06),
            (Fog, 0.08),
            (Snow, 0.10),
            (Blizzard, 0.05),
            (Sandstorm, 0.05),
            (Hail, 0.04),
            (Glaze, 0.05),
            (Heatwave, 0.06),
            (Ash, 0.04),
        ],
    )
}

/// Tyre sets: the right one depends on weather and surface.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tyre {
    Road,
    Rain,
    Snow,
    Offroad,
}
pub const TYRE_COUNT: usize = 4;

impl Tyre {
    pub const ALL: [Tyre; 4] = [Tyre::Road, Tyre::Rain, Tyre::Snow, Tyre::Offroad];
    pub fn from_index(i: usize) -> Tyre {
        Tyre::ALL[i.min(3)]
    }
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        ["road", "rain", "snow", "offroad"][self.index()]
    }
    pub fn parse(s: &str) -> Option<Tyre> {
        let s = s.trim().to_ascii_lowercase();
        Tyre::ALL.iter().copied().find(|t| t.name() == s)
    }

    /// What the conditions ask for: ice and snow, wet roads, loose ground or plain dry asphalt.
    pub fn best_for(w: Weather, s: Surface) -> Tyre {
        use Surface as S;
        use Weather as W;
        if matches!(s, S::IceLake) || matches!(w, W::Snow | W::Blizzard | W::Glaze) {
            Tyre::Snow
        } else if matches!(s, S::Dunes | S::Bog) || matches!(w, W::Sandstorm | W::Ash) {
            Tyre::Offroad
        } else if matches!(w, W::Rain | W::Storm | W::Hail | W::Fog) {
            Tyre::Rain
        } else {
            Tyre::Road
        }
    }

    /// Grip multiplier of this set under the given conditions.
    pub fn grip(self, w: Weather, s: Surface) -> f32 {
        let best = Tyre::best_for(w, s);
        if self == best {
            return 1.12;
        }
        // second-best neighbours are forgiving, the wrong kind costs a lot
        match (best, self) {
            (Tyre::Road, Tyre::Rain) => 0.98,
            (Tyre::Road, Tyre::Snow) => 0.92,
            (Tyre::Road, Tyre::Offroad) => 0.94,
            (Tyre::Rain, Tyre::Road) => 0.88,
            (Tyre::Rain, Tyre::Snow) => 0.97,
            (Tyre::Rain, Tyre::Offroad) => 0.95,
            (Tyre::Snow, Tyre::Road) => 0.72,
            (Tyre::Snow, Tyre::Rain) => 0.82,
            (Tyre::Snow, Tyre::Offroad) => 0.9,
            (Tyre::Offroad, Tyre::Road) => 0.8,
            (Tyre::Offroad, Tyre::Rain) => 0.86,
            (Tyre::Offroad, Tyre::Snow) => 0.92,
            _ => 1.0,
        }
    }
}

/// Car set-up chosen in the service area before the start.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Setup {
    Balanced,
    Sprint,
    Downforce,
    Economy,
}
pub const SETUP_COUNT: usize = 4;

impl Setup {
    pub const ALL: [Setup; 4] = [
        Setup::Balanced,
        Setup::Sprint,
        Setup::Downforce,
        Setup::Economy,
    ];
    pub fn from_index(i: usize) -> Setup {
        Setup::ALL[i.min(3)]
    }
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        ["balanced", "sprint", "downforce", "economy"][self.index()]
    }
    pub fn parse(s: &str) -> Option<Setup> {
        let s = s.trim().to_ascii_lowercase();
        Setup::ALL.iter().copied().find(|t| t.name() == s)
    }
    /// (acceleration, top speed, grip, fuel range)
    pub fn mods(self) -> (f32, f32, f32, f32) {
        match self {
            Setup::Balanced => (1.0, 1.0, 1.0, 1.0),
            Setup::Sprint => (1.12, 0.96, 0.97, 0.95),
            Setup::Downforce => (0.98, 0.95, 1.10, 1.0),
            Setup::Economy => (0.94, 0.98, 1.0, 1.30),
        }
    }
}
