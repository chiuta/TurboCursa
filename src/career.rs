//! Career layer: divisions, a six-race league season, and elimination cups for every country,
//! continent, the intercontinental stage and the world. Pure logic (no drawing, no I/O): the
//! player's real lap time is the only input; rivals are simulated from their rating, so every
//! result is deterministic for a given seed and the whole state fits in one save string.

pub const DIVS: usize = 5;
pub const NCONT: usize = 6;
pub const LEAGUE_DAYS: u8 = 6;
pub const LEAGUE_SIZE: usize = 12;
const LEAGUE_POINTS: [u16; LEAGUE_SIZE] = [15, 12, 10, 8, 6, 5, 4, 3, 2, 1, 0, 0];

pub struct Country {
    pub code: &'static str,
    pub name: &'static str,
    pub cont: u8,
}

macro_rules! c {
    ($code:expr, $name:expr, $cont:expr) => {
        Country {
            code: $code,
            name: $name,
            cont: $cont,
        }
    };
}

/// Continents: 0 Europe, 1 Asia, 2 Africa, 3 North America, 4 South America, 5 Oceania.
pub const COUNTRIES: &[Country] = &[
    c!("RO", "ROMANIA", 0),
    c!("FR", "FRANCE", 0),
    c!("DE", "GERMANY", 0),
    c!("IT", "ITALY", 0),
    c!("ES", "SPAIN", 0),
    c!("PT", "PORTUGAL", 0),
    c!("GB", "UNITED KINGDOM", 0),
    c!("IE", "IRELAND", 0),
    c!("NL", "NETHERLANDS", 0),
    c!("BE", "BELGIUM", 0),
    c!("LU", "LUXEMBOURG", 0),
    c!("CH", "SWITZERLAND", 0),
    c!("AT", "AUSTRIA", 0),
    c!("SE", "SWEDEN", 0),
    c!("NO", "NORWAY", 0),
    c!("FI", "FINLAND", 0),
    c!("DK", "DENMARK", 0),
    c!("IS", "ICELAND", 0),
    c!("PL", "POLAND", 0),
    c!("HU", "HUNGARY", 0),
    c!("CZ", "CZECHIA", 0),
    c!("SK", "SLOVAKIA", 0),
    c!("SI", "SLOVENIA", 0),
    c!("HR", "CROATIA", 0),
    c!("RS", "SERBIA", 0),
    c!("BA", "BOSNIA", 0),
    c!("MK", "N. MACEDONIA", 0),
    c!("AL", "ALBANIA", 0),
    c!("GR", "GREECE", 0),
    c!("BG", "BULGARIA", 0),
    c!("MD", "MOLDOVA", 0),
    c!("UA", "UKRAINE", 0),
    c!("BY", "BELARUS", 0),
    c!("LT", "LITHUANIA", 0),
    c!("LV", "LATVIA", 0),
    c!("EE", "ESTONIA", 0),
    c!("MT", "MALTA", 0),
    c!("CY", "CYPRUS", 0),
    c!("JP", "JAPAN", 1),
    c!("CN", "CHINA", 1),
    c!("IN", "INDIA", 1),
    c!("KR", "S. KOREA", 1),
    c!("TW", "TAIWAN", 1),
    c!("TH", "THAILAND", 1),
    c!("VN", "VIETNAM", 1),
    c!("ID", "INDONESIA", 1),
    c!("PH", "PHILIPPINES", 1),
    c!("MY", "MALAYSIA", 1),
    c!("SG", "SINGAPORE", 1),
    c!("BD", "BANGLADESH", 1),
    c!("LK", "SRI LANKA", 1),
    c!("NP", "NEPAL", 1),
    c!("MN", "MONGOLIA", 1),
    c!("KZ", "KAZAKHSTAN", 1),
    c!("UZ", "UZBEKISTAN", 1),
    c!("PK", "PAKISTAN", 1),
    c!("IR", "IRAN", 1),
    c!("IQ", "IRAQ", 1),
    c!("SA", "SAUDI ARABIA", 1),
    c!("AE", "UAE", 1),
    c!("QA", "QATAR", 1),
    c!("JO", "JORDAN", 1),
    c!("LB", "LEBANON", 1),
    c!("IL", "ISRAEL", 1),
    c!("TR", "TURKEY", 1),
    c!("GE", "GEORGIA", 1),
    c!("AM", "ARMENIA", 1),
    c!("AZ", "AZERBAIJAN", 1),
    c!("EG", "EGYPT", 2),
    c!("MA", "MOROCCO", 2),
    c!("DZ", "ALGERIA", 2),
    c!("TN", "TUNISIA", 2),
    c!("LY", "LIBYA", 2),
    c!("SN", "SENEGAL", 2),
    c!("GH", "GHANA", 2),
    c!("CI", "IVORY COAST", 2),
    c!("NG", "NIGERIA", 2),
    c!("CM", "CAMEROON", 2),
    c!("ET", "ETHIOPIA", 2),
    c!("KE", "KENYA", 2),
    c!("TZ", "TANZANIA", 2),
    c!("UG", "UGANDA", 2),
    c!("AO", "ANGOLA", 2),
    c!("MZ", "MOZAMBIQUE", 2),
    c!("ZW", "ZIMBABWE", 2),
    c!("ZA", "SOUTH AFRICA", 2),
    c!("US", "USA", 3),
    c!("CA", "CANADA", 3),
    c!("MX", "MEXICO", 3),
    c!("GT", "GUATEMALA", 3),
    c!("HN", "HONDURAS", 3),
    c!("NI", "NICARAGUA", 3),
    c!("SV", "EL SALVADOR", 3),
    c!("CR", "COSTA RICA", 3),
    c!("PA", "PANAMA", 3),
    c!("CU", "CUBA", 3),
    c!("JM", "JAMAICA", 3),
    c!("HT", "HAITI", 3),
    c!("DO", "DOMINICAN REP.", 3),
    c!("BS", "BAHAMAS", 3),
    c!("TT", "TRINIDAD", 3),
    c!("BR", "BRAZIL", 4),
    c!("AR", "ARGENTINA", 4),
    c!("CL", "CHILE", 4),
    c!("CO", "COLOMBIA", 4),
    c!("PE", "PERU", 4),
    c!("UY", "URUGUAY", 4),
    c!("VE", "VENEZUELA", 4),
    c!("EC", "ECUADOR", 4),
    c!("BO", "BOLIVIA", 4),
    c!("PY", "PARAGUAY", 4),
    c!("GY", "GUYANA", 4),
    c!("SR", "SURINAME", 4),
    c!("AU", "AUSTRALIA", 5),
    c!("NZ", "NEW ZEALAND", 5),
    c!("FJ", "FIJI", 5),
    c!("PG", "PAPUA N. GUINEA", 5),
    c!("WS", "SAMOA", 5),
    c!("RU", "RUSSIA", 0),
    c!("AD", "ANDORRA", 0),
    c!("ME", "MONTENEGRO", 0),
    c!("MC", "MONACO", 0),
    c!("LI", "LIECHTENSTEIN", 0),
    c!("SM", "SAN MARINO", 0),
    c!("VA", "VATICAN", 0),
    c!("AF", "AFGHANISTAN", 1),
    c!("BH", "BAHRAIN", 1),
    c!("BT", "BHUTAN", 1),
    c!("BN", "BRUNEI", 1),
    c!("KH", "CAMBODIA", 1),
    c!("KP", "N. KOREA", 1),
    c!("KW", "KUWAIT", 1),
    c!("KG", "KYRGYZSTAN", 1),
    c!("LA", "LAOS", 1),
    c!("MV", "MALDIVES", 1),
    c!("MM", "MYANMAR", 1),
    c!("OM", "OMAN", 1),
    c!("PS", "PALESTINE", 1),
    c!("SY", "SYRIA", 1),
    c!("TJ", "TAJIKISTAN", 1),
    c!("TL", "TIMOR-LESTE", 1),
    c!("TM", "TURKMENISTAN", 1),
    c!("YE", "YEMEN", 1),
    c!("BJ", "BENIN", 2),
    c!("BW", "BOTSWANA", 2),
    c!("BF", "BURKINA FASO", 2),
    c!("BI", "BURUNDI", 2),
    c!("CV", "CABO VERDE", 2),
    c!("CF", "C. AFRICAN REP.", 2),
    c!("TD", "CHAD", 2),
    c!("KM", "COMOROS", 2),
    c!("CG", "CONGO", 2),
    c!("CD", "DR CONGO", 2),
    c!("DJ", "DJIBOUTI", 2),
    c!("GQ", "EQ. GUINEA", 2),
    c!("ER", "ERITREA", 2),
    c!("SZ", "ESWATINI", 2),
    c!("GA", "GABON", 2),
    c!("GM", "GAMBIA", 2),
    c!("GN", "GUINEA", 2),
    c!("GW", "GUINEA-BISSAU", 2),
    c!("LS", "LESOTHO", 2),
    c!("LR", "LIBERIA", 2),
    c!("MG", "MADAGASCAR", 2),
    c!("MW", "MALAWI", 2),
    c!("ML", "MALI", 2),
    c!("MR", "MAURITANIA", 2),
    c!("MU", "MAURITIUS", 2),
    c!("NA", "NAMIBIA", 2),
    c!("NE", "NIGER", 2),
    c!("RW", "RWANDA", 2),
    c!("ST", "SAO TOME", 2),
    c!("SC", "SEYCHELLES", 2),
    c!("SL", "SIERRA LEONE", 2),
    c!("SO", "SOMALIA", 2),
    c!("SS", "SOUTH SUDAN", 2),
    c!("SD", "SUDAN", 2),
    c!("TG", "TOGO", 2),
    c!("ZM", "ZAMBIA", 2),
    c!("AG", "ANTIGUA", 3),
    c!("BB", "BARBADOS", 3),
    c!("BZ", "BELIZE", 3),
    c!("DM", "DOMINICA", 3),
    c!("GD", "GRENADA", 3),
    c!("KN", "ST KITTS-NEVIS", 3),
    c!("LC", "SAINT LUCIA", 3),
    c!("VC", "ST VINCENT", 3),
    c!("KI", "KIRIBATI", 5),
    c!("MH", "MARSHALL IS.", 5),
    c!("FM", "MICRONESIA", 5),
    c!("NR", "NAURU", 5),
    c!("PW", "PALAU", 5),
    c!("SB", "SOLOMON IS.", 5),
    c!("TO", "TONGA", 5),
    c!("TV", "TUVALU", 5),
    c!("VU", "VANUATU", 5),
];

pub fn ncountries() -> usize {
    COUNTRIES.len()
}

/// Countries of a continent, in table order.
pub fn countries_of(cont: u8) -> Vec<u8> {
    (0..COUNTRIES.len() as u8)
        .filter(|&i| COUNTRIES[i as usize].cont == cont)
        .collect()
}

pub fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn mixh(a: u64, b: u64) -> u64 {
    splitmix(a ^ splitmix(b.wrapping_add(0x1234_5678_9ABC)))
}

fn unit(h: u64) -> f32 {
    (h >> 40) as f32 / (1u64 << 24) as f32
}

#[derive(Clone, Debug)]
pub struct Rival {
    pub name: String,
    pub country: u8,
    /// 0..1, how close to the reference (autopilot) time this driver gets
    pub rating: f32,
}

const SYL_A: [&str; 16] = [
    "KOR", "VEL", "MAR", "TAN", "ORS", "BRA", "LEN", "DRA", "SOL", "NIK", "HAR", "QUI", "FER",
    "GAL", "RUS", "TOR",
];
const SYL_B: [&str; 16] = [
    "VAN", "DEN", "SKI", "ETTI", "OV", "SON", "ARD", "ELLI", "IC", "BERG", "IDES", "UCCI", "AN",
    "ENKO", "LUND", "ASH",
];

fn driver_name(h: u64) -> String {
    let ini = (b'A' + (h % 26) as u8) as char;
    format!(
        "{}. {}{}",
        ini,
        SYL_A[((h >> 8) % 16) as usize],
        SYL_B[((h >> 16) % 16) as usize]
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CupKind {
    National(u8),
    Continental(u8),
    Inter,
    World,
}

type Plan = &'static [(usize, usize)];
const PLAN_NAT: Plan = &[(16, 8), (8, 4), (4, 1)];
const PLAN_CONT: Plan = &[(24, 16), (16, 8), (8, 4), (4, 1)];
const PLAN_INTER: Plan = &[(32, 16), (16, 8), (8, 4), (4, 1)];
const PLAN_WORLD: Plan = &[(32, 24), (24, 16), (16, 8), (8, 4), (4, 1)];

impl CupKind {
    /// (entrants, advancing) per round; the last round has one winner.
    pub fn plan(self) -> Plan {
        match self {
            CupKind::National(_) => PLAN_NAT,
            CupKind::Continental(_) => PLAN_CONT,
            CupKind::Inter => PLAN_INTER,
            CupKind::World => PLAN_WORLD,
        }
    }
    fn weight(self) -> f32 {
        match self {
            CupKind::National(_) => 2.0,
            CupKind::Continental(_) => 3.0,
            CupKind::Inter => 4.0,
            CupKind::World => 6.0,
        }
    }
    fn base(self) -> f32 {
        match self {
            CupKind::National(_) => 0.30,
            CupKind::Continental(_) => 0.42,
            CupKind::Inter => 0.52,
            CupKind::World => 0.62,
        }
    }
    fn code(self) -> (u8, u8) {
        match self {
            CupKind::National(c) => (0, c),
            CupKind::Continental(c) => (1, c),
            CupKind::Inter => (2, 0),
            CupKind::World => (3, 0),
        }
    }
    fn from_code(k: u8, a: u8) -> Option<CupKind> {
        match k {
            0 if (a as usize) < COUNTRIES.len() => Some(CupKind::National(a)),
            1 if (a as usize) < NCONT => Some(CupKind::Continental(a)),
            2 => Some(CupKind::Inter),
            3 => Some(CupKind::World),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Round,
    Repechage,
    Won,
    Lost,
}

/// Rival strength for a given tier/division.
fn rival_rating(base: f32, div: u8, h: u64) -> f32 {
    (base + 0.07 * div as f32 + 0.44 * (unit(splitmix(h)) - 0.5)).clamp(0.05, 0.99)
}

fn rival_time(par: f32, rating: f32, h: u64) -> Option<f32> {
    if unit(splitmix(h ^ 0xD1F)) < 0.03 {
        return None; // rival crashed out / ran out of time
    }
    Some(par * (1.28 - 0.30 * rating) * (1.0 + 0.12 * (unit(h) - 0.5)))
}

fn sane_par(par: f32) -> f32 {
    if par.is_finite() && par > 5.0 {
        par.min(1000.0)
    } else {
        100.0
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub id: u16,
    pub name: String,
    pub country: u8,
    pub time: Option<f32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Advanced,
    /// eliminated; `chance` = a repechage race is available
    Eliminated {
        chance: bool,
    },
    Revived,
    Champion,
    /// lost the final (finished 2nd-4th)
    FinalPlace,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub table: Vec<Row>,
    pub player_pos: usize,
    pub outcome: Outcome,
    /// league only: (old division, new division, final position) when the season ended
    pub season_end: Option<(u8, u8, usize)>,
}

#[derive(Clone, Debug)]
pub struct Cup {
    pub kind: CupKind,
    pub seed: u64,
    pub div: u8,
    pub round: usize,
    pub lives: u8,
    pub phase: Phase,
    pub final_reached: bool,
    /// ids (0 = the player) still in the cup
    pub alive: Vec<u16>,
    /// ids eliminated in the last round (repechage pool, best first)
    pub out: Vec<u16>,
    /// rival strength shift from the difficulty level (not saved; set by Career)
    pub shift: f32,
}

impl Cup {
    pub fn new(kind: CupKind, seed: u64, div: u8) -> Cup {
        let n = kind.plan()[0].0 as u16;
        Cup {
            shift: 0.0,
            kind,
            seed,
            div: div.min(DIVS as u8 - 1),
            round: 0,
            lives: 2,
            phase: Phase::Round,
            final_reached: false,
            alive: (0..n).collect(),
            out: Vec::new(),
        }
    }

    pub fn rounds(&self) -> usize {
        self.kind.plan().len()
    }

    pub fn is_over(&self) -> bool {
        matches!(self.phase, Phase::Won | Phase::Lost)
    }

    pub fn is_final(&self) -> bool {
        self.phase == Phase::Round && self.round + 1 == self.rounds()
    }

    pub fn rival(&self, id: u16) -> Rival {
        let h = mixh(self.seed, id as u64 + 7);
        let country = match self.kind {
            CupKind::National(c) => c,
            CupKind::Continental(k) => {
                let cs = countries_of(k);
                cs[(h >> 20) as usize % cs.len()]
            }
            _ => ((h >> 20) % COUNTRIES.len() as u64) as u8,
        };
        Rival {
            name: driver_name(h),
            country,
            rating: rival_rating(self.kind.base(), self.div, h >> 4),
        }
    }

    pub fn entrants(&self) -> Vec<u16> {
        match self.phase {
            Phase::Round => self.alive.clone(),
            Phase::Repechage => self.out.clone(),
            _ => Vec::new(),
        }
    }

    pub fn race_seed(&self) -> u64 {
        mixh(
            self.seed,
            1000 + self.round as u64 * 4 + (self.phase == Phase::Repechage) as u64,
        ) % 100_000
    }

    fn rows(&self, ids: &[u16], player: Option<f32>, par: f32) -> Vec<Row> {
        let salt = self.round as u64 * 4 + (self.phase == Phase::Repechage) as u64;
        let mut rows: Vec<Row> = ids
            .iter()
            .map(|&id| {
                if id == 0 {
                    Row {
                        id,
                        name: "YOU".into(),
                        country: 0,
                        time: player.filter(|t| t.is_finite() && *t > 0.0),
                    }
                } else {
                    let r = self.rival(id);
                    let h = mixh(self.seed ^ 0x77, id as u64 * 64 + salt);
                    Row {
                        id,
                        name: r.name,
                        country: r.country,
                        time: rival_time(par, r.rating + self.shift, h),
                    }
                }
            })
            .collect();
        rows.sort_by(|a, b| match (a.time, b.time) {
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id.cmp(&b.id),
        });
        rows
    }

    /// Resolve the current race with the player's time (None = did not finish).
    pub fn advance(&mut self, player: Option<f32>, par: f32) -> Option<Report> {
        let par = sane_par(par);
        match self.phase {
            Phase::Round => {
                let ids = self.alive.clone();
                let rows = self.rows(&ids, player, par);
                let pos = rows
                    .iter()
                    .position(|r| r.id == 0)
                    .map_or(rows.len(), |i| i + 1);
                let adv = self.kind.plan()[self.round].1;
                let out;
                if adv == 1 {
                    let won = pos == 1 && rows[0].time.is_some();
                    self.phase = if won { Phase::Won } else { Phase::Lost };
                    out = if won {
                        Outcome::Champion
                    } else {
                        Outcome::FinalPlace
                    };
                } else {
                    let keep: Vec<u16> = rows.iter().take(adv).map(|r| r.id).collect();
                    let rest: Vec<u16> = rows.iter().skip(adv).map(|r| r.id).collect();
                    if keep.contains(&0) && rows[pos - 1].time.is_some() {
                        self.alive = keep;
                        self.out = rest;
                        self.round += 1;
                        if self.round + 1 == self.rounds() {
                            self.final_reached = true;
                        }
                        out = Outcome::Advanced;
                    } else if self.lives > 0 {
                        // best of the eliminated plus the player go to the repechage
                        let mut pool: Vec<u16> =
                            rest.iter().copied().filter(|&i| i != 0).take(7).collect();
                        pool.push(0);
                        self.alive = rows
                            .iter()
                            .take(adv)
                            .map(|r| r.id)
                            .filter(|&i| i != 0)
                            .collect();
                        self.out = pool;
                        self.phase = Phase::Repechage;
                        out = Outcome::Eliminated { chance: true };
                    } else {
                        self.phase = Phase::Lost;
                        out = Outcome::Eliminated { chance: false };
                    }
                }
                Some(Report {
                    table: rows,
                    player_pos: pos,
                    outcome: out,
                    season_end: None,
                })
            }
            Phase::Repechage => {
                let ids = self.out.clone();
                let rows = self.rows(&ids, player, par);
                let pos = rows
                    .iter()
                    .position(|r| r.id == 0)
                    .map_or(rows.len(), |i| i + 1);
                let back = pos <= 2 && rows[pos - 1].time.is_some();
                self.lives = self.lives.saturating_sub(1);
                let out;
                if back {
                    for r in rows.iter().take(2) {
                        self.alive.push(r.id);
                    }
                    self.out.clear();
                    self.round += 1;
                    self.phase = Phase::Round;
                    if self.round + 1 == self.rounds() {
                        self.final_reached = true;
                    }
                    out = Outcome::Revived;
                } else {
                    self.phase = Phase::Lost;
                    out = Outcome::Eliminated { chance: false };
                }
                Some(Report {
                    table: rows,
                    player_pos: pos,
                    outcome: out,
                    season_end: None,
                })
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Trophies {
    pub national: Vec<u16>,
    pub cont: [u16; NCONT],
    pub inter: u16,
    pub world: u16,
    pub league: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stats {
    pub races: u32,
    pub wins: u32,
    pub podiums: u32,
}

#[derive(Clone, Debug)]
pub struct Career {
    pub country: u8,
    pub season: u32,
    pub div: u8,
    pub rating: f32,
    pub seed: u64,
    pub day: u8,
    pub pts: [u16; LEAGUE_SIZE],
    /// qualified this season for: continental, intercontinental, world
    pub q: [bool; 3],
    pub trophies: Trophies,
    pub stats: Stats,
    pub cup: Option<Cup>,
    /// difficulty level (0 easy .. 4 insane) and vehicle class (0..5) of the player
    pub level: u8,
    pub class: u8,
    counter: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Locked {
    NotQualified,
    CupInProgress,
}

impl Career {
    pub fn new(country: u8, seed: u64) -> Career {
        Career {
            country: country.min(COUNTRIES.len() as u8 - 1),
            season: 1,
            div: 0,
            rating: 1000.0,
            seed,
            day: 0,
            pts: [0; LEAGUE_SIZE],
            q: [false; 3],
            trophies: Trophies {
                national: vec![0; COUNTRIES.len()],
                ..Default::default()
            },
            stats: Stats::default(),
            cup: None,
            level: 1,
            class: 0,
            counter: 0,
        }
    }

    pub fn level_shift(&self) -> f32 {
        crate::game::Difficulty::from_index(self.level as usize).rival_shift()
    }

    pub fn continent(&self) -> u8 {
        COUNTRIES[self.country as usize].cont
    }

    pub fn league_rival(&self, i: usize) -> Rival {
        let h = mixh(
            self.seed ^ 0x1EA6,
            self.season as u64 * 64 + self.div as u64 * 16 + i as u64,
        );
        Rival {
            name: driver_name(h),
            country: ((h >> 24) % COUNTRIES.len() as u64) as u8,
            rating: rival_rating(0.22, self.div, h >> 4),
        }
    }

    pub fn league_seed(&self) -> u64 {
        mixh(self.seed, 5000 + self.season as u64 * 8 + self.day as u64) % 100_000
    }

    fn bump_rating(&mut self, weight: f32, n: usize, pos: usize, finished: bool) {
        let pos = if finished { pos } else { n };
        let d = ((n as f32 + 1.0) / 2.0 - pos as f32) * weight * 3.0;
        self.rating = (self.rating + d).clamp(0.0, 9999.0);
        self.stats.races += 1;
        if finished && pos == 1 {
            self.stats.wins += 1;
        }
        if finished && pos <= 3 {
            self.stats.podiums += 1;
        }
    }

    /// Play one league matchday. Ends the season (promotion / relegation) after the last day.
    pub fn run_league(&mut self, player: Option<f32>, par: f32) -> Report {
        let par = sane_par(par);
        let mut rows = Vec::with_capacity(LEAGUE_SIZE);
        rows.push(Row {
            id: 0,
            name: "YOU".into(),
            country: self.country,
            time: player.filter(|t| t.is_finite() && *t > 0.0),
        });
        for i in 1..LEAGUE_SIZE {
            let r = self.league_rival(i);
            let h = mixh(
                self.seed ^ 0x55,
                self.season as u64 * 1000 + self.day as u64 * 20 + i as u64,
            );
            rows.push(Row {
                id: i as u16,
                name: r.name,
                country: r.country,
                time: rival_time(par, r.rating + self.level_shift(), h),
            });
        }
        rows.sort_by(|a, b| match (a.time, b.time) {
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id.cmp(&b.id),
        });
        for (rank, r) in rows.iter().enumerate() {
            if r.time.is_some() {
                self.pts[r.id as usize] += LEAGUE_POINTS[rank];
            }
        }
        let pos = rows
            .iter()
            .position(|r| r.id == 0)
            .unwrap_or(LEAGUE_SIZE - 1)
            + 1;
        let fin = rows[pos - 1].time.is_some();
        self.bump_rating(1.0, LEAGUE_SIZE, pos, fin);
        self.day += 1;
        let mut season_end = None;
        if self.day >= LEAGUE_DAYS {
            let mut order: Vec<usize> = (0..LEAGUE_SIZE).collect();
            order.sort_by(|&a, &b| self.pts[b].cmp(&self.pts[a]).then(a.cmp(&b)));
            let place = order
                .iter()
                .position(|&i| i == 0)
                .unwrap_or(LEAGUE_SIZE - 1)
                + 1;
            let old = self.div;
            if place == 1 && old as usize == DIVS - 1 {
                self.trophies.league = self.trophies.league.saturating_add(1);
            }
            if place <= 3 && (old as usize) < DIVS - 1 {
                self.div += 1;
            } else if place > LEAGUE_SIZE - 3 && old > 0 {
                self.div -= 1;
            }
            season_end = Some((old, self.div, place));
            self.season += 1;
            self.day = 0;
            self.pts = [0; LEAGUE_SIZE];
            self.q = [false; 3];
            // a league champion of the top division is invited straight to the World Cup
            if place == 1 && old as usize == DIVS - 1 {
                self.q = [true, true, true];
            }
        }
        Report {
            table: rows,
            player_pos: pos,
            outcome: if pos == 1 {
                Outcome::Champion
            } else {
                Outcome::Advanced
            },
            season_end,
        }
    }

    pub fn can_start(&self, kind: CupKind) -> Result<(), Locked> {
        if self.cup.is_some() {
            return Err(Locked::CupInProgress);
        }
        let ok = match kind {
            CupKind::National(_) => true,
            CupKind::Continental(k) => self.q[0] && k == self.continent(),
            CupKind::Inter => self.q[1],
            CupKind::World => self.q[2],
        };
        if ok {
            Ok(())
        } else {
            Err(Locked::NotQualified)
        }
    }

    pub fn start_cup(&mut self, kind: CupKind) -> Result<(), Locked> {
        self.can_start(kind)?;
        self.counter += 1;
        let (a, b) = kind.code();
        let seed = mixh(
            self.seed,
            9000 + self.season as u64 * 10_000 + self.counter as u64 * 64 + a as u64 * 8 + b as u64,
        );
        self.cup = Some(Cup::new(kind, seed, self.div));
        Ok(())
    }

    /// Resolve the active cup's race. Books trophies, qualification and rating when the cup ends.
    pub fn run_cup(&mut self, player: Option<f32>, par: f32) -> Option<Report> {
        let mut cup = self.cup.take()?;
        let n = cup.entrants().len();
        cup.shift = self.level_shift();
        let rep = cup.advance(player, par);
        if let Some(r) = &rep {
            let fin = r
                .table
                .iter()
                .find(|x| x.id == 0)
                .is_some_and(|x| x.time.is_some());
            self.bump_rating(cup.kind.weight(), n.max(2), r.player_pos, fin);
        }
        if cup.is_over() {
            let won = cup.phase == Phase::Won;
            match cup.kind {
                CupKind::National(c) => {
                    if won {
                        let t = &mut self.trophies.national[c as usize];
                        *t = t.saturating_add(1);
                    }
                    if c == self.country && cup.final_reached {
                        self.q[0] = true;
                    }
                }
                CupKind::Continental(k) => {
                    if won {
                        let t = &mut self.trophies.cont[k as usize % NCONT];
                        *t = t.saturating_add(1);
                    }
                    if cup.final_reached {
                        self.q[1] = true;
                    }
                }
                CupKind::Inter => {
                    if won {
                        self.trophies.inter = self.trophies.inter.saturating_add(1);
                    }
                    if cup.final_reached {
                        self.q[2] = true;
                    }
                }
                CupKind::World => {
                    if won {
                        self.trophies.world = self.trophies.world.saturating_add(1);
                    }
                }
            }
            self.rating =
                (self.rating + if won { 20.0 * cup.kind.weight() } else { 0.0 }).min(9999.0);
        } else {
            self.cup = Some(cup);
        }
        rep
    }

    /// Abandon the cup in progress (counts as lost; nothing is booked).
    pub fn abandon_cup(&mut self) {
        self.cup = None;
    }

    /// World driver ranking: a fixed pool of rivals whose ratings drift each season, plus the player.
    /// Returns (name, country, rating, is_player) sorted best-first.
    pub fn drivers_ranking(&self) -> Vec<(String, u8, i32, bool)> {
        let mut v: Vec<(String, u8, i32, bool)> = (0..300u64)
            .map(|i| {
                let h = mixh(self.seed ^ 0xFACE, i);
                let base = unit(h);
                let drift = 160.0 * (unit(mixh(h, self.season as u64)) - 0.5);
                let r = 700.0 + 1700.0 * base.powf(1.5) + drift;
                (
                    driver_name(h),
                    ((h >> 24) % COUNTRIES.len() as u64) as u8,
                    r as i32,
                    false,
                )
            })
            .collect();
        v.push(("YOU".into(), self.country, self.rating as i32, true));
        v.sort_by(|a, b| b.2.cmp(&a.2).then(a.3.cmp(&b.3)));
        v
    }

    /// Nations ranking: sum of the three best drivers of each country (the player counts for the home country).
    pub fn nations_ranking(&self) -> Vec<(u8, i32)> {
        let mut best: Vec<Vec<i32>> = vec![Vec::new(); COUNTRIES.len()];
        for (_, c, r, _) in self.drivers_ranking() {
            best[c as usize].push(r);
        }
        let mut v: Vec<(u8, i32)> = best
            .into_iter()
            .enumerate()
            .map(|(i, mut rs)| {
                rs.sort_unstable_by(|a, b| b.cmp(a));
                (i as u8, rs.iter().take(3).sum::<i32>())
            })
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }

    pub fn league_table(&self) -> Vec<(String, u8, u16, bool)> {
        let mut v: Vec<(String, u8, u16, bool)> = (0..LEAGUE_SIZE)
            .map(|i| {
                if i == 0 {
                    ("YOU".into(), self.country, self.pts[0], true)
                } else {
                    let r = self.league_rival(i);
                    (r.name, r.country, self.pts[i], false)
                }
            })
            .collect();
        v.sort_by(|a, b| b.2.cmp(&a.2).then(a.3.cmp(&b.3).reverse()));
        v
    }

    // ---------------------------------------------------------------- persistence

    pub fn to_save(&self) -> String {
        let join = |it: Vec<String>| it.join(",");
        let mut s = format!(
            "TC1;c={};s={};d={};r={};seed={};ld={};lp={};q={};cn={};tc={};ti={};tw={};tl={};st={},{},{};lv={};cl={};n={}",
            self.country,
            self.season,
            self.div,
            self.rating as i64,
            self.seed,
            self.day,
            join(self.pts.iter().map(|x| x.to_string()).collect()),
            join(self.q.iter().map(|&b| (b as u8).to_string()).collect()),
            join(self.trophies.national.iter().map(|x| x.to_string()).collect()),
            join(self.trophies.cont.iter().map(|x| x.to_string()).collect()),
            self.trophies.inter,
            self.trophies.world,
            self.trophies.league,
            self.stats.races,
            self.stats.wins,
            self.stats.podiums,
            self.level,
            self.class,
            self.counter,
        );
        if let Some(c) = &self.cup {
            let (k, a) = c.kind.code();
            let ph = match c.phase {
                Phase::Round => 0,
                Phase::Repechage => 1,
                Phase::Won => 2,
                Phase::Lost => 3,
            };
            s += &format!(
                ";cup={},{},{},{},{},{},{},{}|{}|{}",
                k,
                a,
                c.seed,
                c.div,
                c.round,
                c.lives,
                ph,
                c.final_reached as u8,
                join(c.alive.iter().map(|x| x.to_string()).collect()),
                join(c.out.iter().map(|x| x.to_string()).collect()),
            );
        }
        s
    }

    /// Parses a save string; anything malformed yields None (never panics).
    pub fn from_save(s: &str) -> Option<Career> {
        if s.len() > 20_000 {
            return None;
        }
        let mut parts = s.trim().split(';');
        if parts.next()? != "TC1" {
            return None;
        }
        let mut c = Career::new(0, 1);
        let nums = |v: &str, max: usize| -> Option<Vec<u64>> {
            if v.is_empty() {
                return Some(Vec::new());
            }
            let r: Option<Vec<u64>> = v.split(',').map(|x| x.parse().ok()).collect();
            r.filter(|r| r.len() <= max)
        };
        for p in parts {
            let (k, v) = p.split_once('=')?;
            match k {
                "c" => c.country = (v.parse::<u64>().ok()?).min(COUNTRIES.len() as u64 - 1) as u8,
                "s" => c.season = (v.parse::<u32>().ok()?).clamp(1, 100_000),
                "d" => c.div = (v.parse::<u8>().ok()?).min(DIVS as u8 - 1),
                "r" => c.rating = (v.parse::<f32>().ok()?).clamp(0.0, 9999.0),
                "seed" => c.seed = v.parse().ok()?,
                "ld" => c.day = (v.parse::<u8>().ok()?).min(LEAGUE_DAYS - 1),
                "lp" => {
                    let n = nums(v, LEAGUE_SIZE)?;
                    for (i, x) in n.into_iter().enumerate() {
                        c.pts[i] = x.min(999) as u16;
                    }
                }
                "q" => {
                    for (i, x) in nums(v, 3)?.into_iter().enumerate() {
                        c.q[i] = x != 0;
                    }
                }
                "cn" => {
                    for (i, x) in nums(v, COUNTRIES.len())?.into_iter().enumerate() {
                        c.trophies.national[i] = x.min(9999) as u16;
                    }
                }
                "tc" => {
                    for (i, x) in nums(v, NCONT)?.into_iter().enumerate() {
                        c.trophies.cont[i] = x.min(9999) as u16;
                    }
                }
                "ti" => c.trophies.inter = v.parse::<u16>().ok()?,
                "tw" => c.trophies.world = v.parse::<u16>().ok()?,
                "tl" => c.trophies.league = v.parse::<u16>().ok()?,
                "st" => {
                    let n = nums(v, 3)?;
                    if n.len() == 3 {
                        c.stats = Stats {
                            races: n[0].min(u32::MAX as u64) as u32,
                            wins: n[1].min(u32::MAX as u64) as u32,
                            podiums: n[2].min(u32::MAX as u64) as u32,
                        };
                    }
                }
                "lv" => c.level = (v.parse::<u8>().ok()?).min(crate::game::LEVEL_COUNT as u8 - 1),
                "cl" => c.class = (v.parse::<u8>().ok()?).min(crate::cars::CLASS_COUNT as u8 - 1),
                "n" => c.counter = v.parse().ok()?,
                "cup" => {
                    let f: Vec<&str> = v.split('|').collect();
                    if f.len() != 3 {
                        return None;
                    }
                    let h = nums(f[0], 8)?;
                    if h.len() != 8 {
                        return None;
                    }
                    let kind = CupKind::from_code(h[0] as u8, h[1] as u8)?;
                    let round = h[4] as usize;
                    if round >= kind.plan().len() {
                        return None;
                    }
                    let alive: Vec<u16> = nums(f[1], 64)?
                        .into_iter()
                        .map(|x| x.min(4096) as u16)
                        .collect();
                    let out: Vec<u16> = nums(f[2], 64)?
                        .into_iter()
                        .map(|x| x.min(4096) as u16)
                        .collect();
                    let phase = match h[6] {
                        0 => Phase::Round,
                        1 => Phase::Repechage,
                        _ => return None, // finished cups are never saved
                    };
                    if alive.is_empty() && phase == Phase::Round {
                        return None;
                    }
                    if phase == Phase::Repechage && !out.contains(&0) {
                        return None;
                    }
                    if phase == Phase::Round && !alive.contains(&0) {
                        return None;
                    }
                    c.cup = Some(Cup {
                        shift: 0.0,
                        kind,
                        seed: h[2],
                        div: (h[3] as u8).min(DIVS as u8 - 1),
                        round,
                        lives: (h[5] as u8).min(9),
                        phase,
                        final_reached: h[7] != 0,
                        alive,
                        out,
                    });
                }
                _ => {}
            }
        }
        Some(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut c = Career::new(5, 42);
        c.start_cup(CupKind::National(5)).unwrap();
        c.run_cup(Some(90.0), 100.0);
        let s = c.to_save();
        let d = Career::from_save(&s).unwrap();
        assert_eq!(d.to_save(), s);
    }
}
