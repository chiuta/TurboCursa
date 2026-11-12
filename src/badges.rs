//! Collections and badges: a long-term, honest sense of progress (no loot boxes, no timers).
//!
//! A badge is earned once, by playing; nothing here is lost by missing a day.
use crate::i18n::{lang, Lang};

pub const BADGE_COUNT: usize = 25;
pub const B_FIRST: u8 = 0;
pub const B_CLEAN: u8 = 1;
pub const B_NIGHT: u8 = 2;
pub const B_DAWN: u8 = 3;
pub const B_ICE: u8 = 4;
pub const B_STORM: u8 = 5;
pub const B_DARE: u8 = 6;
pub const B_ANIMAL: u8 = 7;
pub const B_AIR: u8 = 8;
pub const B_BOOST: u8 = 9;
/// rank badges 10..=15 at these ranks
pub const RANK_BADGES: [u16; 6] = [10, 50, 100, 250, 500, 999];
/// streak badges 16..=18 at these day counts
pub const STREAK_BADGES: [u16; 3] = [3, 7, 30];
pub const B_VEHICLES: u8 = 19;
pub const B_WEATHERS: u8 = 20;
pub const B_SURFACES: u8 = 21;
pub const B_COUNTRIES: u8 = 22;
pub const B_ALLDAY: u8 = 23;
pub const B_FUEL: u8 = 24;

const NAMES: [[&str; 8]; BADGE_COUNT] = [
    [
        "FIRST FINISH",
        "PRIMA SOSIRE",
        "PREMIÈRE ARRIVÉE",
        "PRIMO ARRIVO",
        "PRIMERA META",
        "PRIMEIRA CHEGADA",
        "ERSTES ZIEL",
        "ELSŐ CÉLBAÉRÉS",
    ],
    [
        "CLEAN DRIVER",
        "ȘOFER CURAT",
        "PILOTE PROPRE",
        "PILOTA PULITO",
        "CONDUCTOR LIMPIO",
        "PILOTO LIMPO",
        "SAUBERER FAHRER",
        "TISZTA VEZETŐ",
    ],
    [
        "NIGHT OWL",
        "BUHĂ DE NOAPTE",
        "OISEAU DE NUIT",
        "GUFO NOTTURNO",
        "BÚHO NOCTURNO",
        "CORUJA NOTURNA",
        "NACHTEULE",
        "ÉJSZAKAI BAGOLY",
    ],
    [
        "EARLY BIRD",
        "DIMINEAȚĂ DEVREME",
        "LÈVE-TÔT",
        "MATTINIERO",
        "MADRUGADOR",
        "MADRUGADOR",
        "FRÜHAUFSTEHER",
        "KORAI MADÁR",
    ],
    [
        "ICE DRIVER",
        "PE GHEAȚĂ",
        "PILOTE DES GLACES",
        "SUL GHIACCIO",
        "SOBRE HIELO",
        "NO GELO",
        "EISFAHRER",
        "JÉGLOVAG",
    ],
    [
        "STORM CHASER",
        "VÂNĂTOR DE FURTUNI",
        "CHASSEUR D'ORAGES",
        "CACCIATORE DI TEMPESTE",
        "CAZATORMENTAS",
        "CAÇADOR DE TEMPESTADES",
        "STURMJÄGER",
        "VIHARVADÁSZ",
    ],
    [
        "DAREDEVIL",
        "TEMERAR",
        "TÉMÉRAIRE",
        "TEMERARIO",
        "TEMERARIO",
        "TEMERÁRIO",
        "WAGHALS",
        "VAKMERŐ",
    ],
    [
        "ANIMAL FRIEND",
        "PRIETEN AL ANIMALELOR",
        "AMI DES BÊTES",
        "AMICO DEGLI ANIMALI",
        "AMIGO DE ANIMALES",
        "AMIGO DOS ANIMAIS",
        "TIERFREUND",
        "ÁLLATBARÁT",
    ],
    [
        "AIR TIME",
        "ZBOR LIBER",
        "TEMPS EN L'AIR",
        "VOLO LIBERO",
        "TIEMPO EN EL AIRE",
        "TEMPO NO AR",
        "LUFTZEIT",
        "LEVEGŐBEN",
    ],
    [
        "BOOSTED",
        "ÎN TURBO",
        "BOOSTÉ",
        "IN TURBO",
        "IMPULSADO",
        "IMPULSIONADO",
        "AUFGELADEN",
        "FELTURBÓZVA",
    ],
    [
        "RANK 10",
        "RANG 10",
        "RANG 10",
        "RANGO 10",
        "RANGO 10",
        "NÍVEL 10",
        "RANG 10",
        "10. RANG",
    ],
    [
        "RANK 50",
        "RANG 50",
        "RANG 50",
        "RANGO 50",
        "RANGO 50",
        "NÍVEL 50",
        "RANG 50",
        "50. RANG",
    ],
    [
        "RANK 100",
        "RANG 100",
        "RANG 100",
        "RANGO 100",
        "RANGO 100",
        "NÍVEL 100",
        "RANG 100",
        "100. RANG",
    ],
    [
        "RANK 250",
        "RANG 250",
        "RANG 250",
        "RANGO 250",
        "RANGO 250",
        "NÍVEL 250",
        "RANG 250",
        "250. RANG",
    ],
    [
        "RANK 500",
        "RANG 500",
        "RANG 500",
        "RANGO 500",
        "RANGO 500",
        "NÍVEL 500",
        "RANG 500",
        "500. RANG",
    ],
    [
        "RANK 999",
        "RANG 999",
        "RANG 999",
        "RANGO 999",
        "RANGO 999",
        "NÍVEL 999",
        "RANG 999",
        "999. RANG",
    ],
    [
        "3 DAYS", "3 ZILE", "3 JOURS", "3 GIORNI", "3 DÍAS", "3 DIAS", "3 TAGE", "3 NAP",
    ],
    [
        "7 DAYS", "7 ZILE", "7 JOURS", "7 GIORNI", "7 DÍAS", "7 DIAS", "7 TAGE", "7 NAP",
    ],
    [
        "30 DAYS",
        "30 DE ZILE",
        "30 JOURS",
        "30 GIORNI",
        "30 DÍAS",
        "30 DIAS",
        "30 TAGE",
        "30 NAP",
    ],
    [
        "ALL VEHICLES",
        "TOATE VEHICULELE",
        "TOUS LES VÉHICULES",
        "TUTTI I VEICOLI",
        "TODOS LOS VEHÍCULOS",
        "TODOS OS VEÍCULOS",
        "ALLE FAHRZEUGE",
        "MINDEN JÁRMŰ",
    ],
    [
        "ALL WEATHER",
        "TOATE VREMURILE",
        "TOUTES LES MÉTÉOS",
        "TUTTI I METEO",
        "TODO EL CLIMA",
        "TODOS OS CLIMAS",
        "ALLE WETTER",
        "MINDEN IDŐJÁRÁS",
    ],
    [
        "ALL SURFACES",
        "TOATE TERENURILE",
        "TOUS LES SOLS",
        "TUTTI I FONDI",
        "TODAS LAS SUPERFICIES",
        "TODAS AS SUPERFÍCIES",
        "ALLE UNTERGRÜNDE",
        "MINDEN ÚTFELÜLET",
    ],
    [
        "50 COUNTRIES",
        "50 DE ȚĂRI",
        "50 PAYS",
        "50 PAESI",
        "50 PAÍSES",
        "50 PAÍSES",
        "50 LÄNDER",
        "50 ORSZÁG",
    ],
    [
        "DAWN TO NIGHT",
        "DE LA ZORI LA NOAPTE",
        "DE L'AUBE À LA NUIT",
        "DALL'ALBA ALLA NOTTE",
        "DEL ALBA A LA NOCHE",
        "DA AURORA À NOITE",
        "VOM MORGEN BIS NACHT",
        "HAJNALTÓL ÉJFÉLIG",
    ],
    [
        "FUEL SAVER",
        "ECONOM",
        "ÉCONOME",
        "RISPARMIATORE",
        "AHORRADOR",
        "POUPADOR",
        "SPARFUCHS",
        "TAKARÉKOS",
    ],
];

pub fn badge_name(id: u8) -> &'static str {
    let l = match lang() {
        Lang::En => 0,
        Lang::Ro => 1,
        Lang::Fr => 2,
        Lang::It => 3,
        Lang::Es => 4,
        Lang::Pt => 5,
        Lang::De => 6,
        Lang::Hu => 7,
    };
    NAMES[(id as usize).min(BADGE_COUNT - 1)][l]
}

/// Names (current language) of the badges in a bit mask, joined with " / ".
pub fn names_of(mask: u32) -> String {
    (0..BADGE_COUNT as u8)
        .filter(|i| mask >> i & 1 != 0)
        .map(badge_name)
        .collect::<Vec<_>>()
        .join(" / ")
}

/// What a finished game shows about its conditions (filled by `GameResult::from_game`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Conditions {
    pub class: u8,
    pub weather: u8,
    pub surface: u8,
    /// 0 dawn, 1 day, 2 dusk, 3 night
    pub time: u8,
    pub country: u8,
    /// fuel was in play and the tank never ran dry
    pub fuel_careful: bool,
}

/// Time-of-day bucket from the in-game clock (minutes after midnight).
pub fn time_bucket(minutes: u32) -> u8 {
    match minutes % 1440 {
        300..=479 => 0,
        480..=1019 => 1,
        1020..=1199 => 2,
        _ => 3,
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Collection {
    pub classes: u32,
    pub weathers: u16,
    pub surfaces: u8,
    pub times: u8,
    /// bit per country index (up to 256 countries)
    pub countries: [u64; 4],
    pub badges: u32,
}

impl Collection {
    pub fn has(&self, id: u8) -> bool {
        id < 32 && self.badges >> id & 1 != 0
    }

    pub fn count(&self) -> u32 {
        (self.badges & ((1u32 << BADGE_COUNT) - 1)).count_ones()
    }

    pub fn countries_seen(&self) -> u32 {
        self.countries.iter().map(|w| w.count_ones()).sum()
    }

    /// Books a finished game (only a finish adds to the collections); returns the badges earned now.
    pub fn record(&mut self, r: &crate::profile::GameResult, rank: u16, streak: u16) -> u32 {
        let before = self.badges;
        let c = &r.cond;
        if r.finished {
            if (c.class as usize) < crate::cars::CLASS_COUNT {
                self.classes |= 1 << c.class as u32;
            }
            self.weathers |= 1 << (c.weather as u32).min(15);
            self.surfaces |= 1 << (c.surface as u32).min(7);
            self.times |= 1 << (c.time as u32).min(7);
            let i = (c.country as usize).min(255);
            self.countries[i / 64] |= 1 << (i % 64);
            self.badges |= 1 << B_FIRST;
            if r.crashes == 0 {
                self.badges |= 1 << B_CLEAN;
            }
            if c.time == 3 {
                self.badges |= 1 << B_NIGHT;
            }
            if c.time == 0 {
                self.badges |= 1 << B_DAWN;
            }
            if c.surface == 1 {
                self.badges |= 1 << B_ICE; // frozen lake
            }
            if matches!(c.weather, 2 | 5 | 6) {
                self.badges |= 1 << B_STORM; // storm, blizzard, sandstorm
            }
            if c.fuel_careful {
                self.badges |= 1 << B_FUEL;
            }
        }
        if r.close_calls >= 3 {
            self.badges |= 1 << B_DARE;
        }
        if r.dodged >= 10 {
            self.badges |= 1 << B_ANIMAL;
        }
        if r.jumps >= 5 {
            self.badges |= 1 << B_AIR;
        }
        if r.pads >= 5 {
            self.badges |= 1 << B_BOOST;
        }
        for (i, &n) in RANK_BADGES.iter().enumerate() {
            if rank >= n {
                self.badges |= 1 << (10 + i);
            }
        }
        for (i, &n) in STREAK_BADGES.iter().enumerate() {
            if streak >= n {
                self.badges |= 1 << (16 + i);
            }
        }
        if self.classes.count_ones() as usize >= crate::cars::CLASS_COUNT {
            self.badges |= 1 << B_VEHICLES;
        }
        if self.weathers.count_ones() as usize >= crate::env::WEATHER_COUNT {
            self.badges |= 1 << B_WEATHERS;
        }
        if self.surfaces.count_ones() as usize >= crate::env::SURFACE_COUNT {
            self.badges |= 1 << B_SURFACES;
        }
        if self.countries_seen() >= 50 {
            self.badges |= 1 << B_COUNTRIES;
        }
        if self.times & 0xF == 0xF {
            self.badges |= 1 << B_ALLDAY;
        }
        self.badges & !before
    }

    pub fn encode(&self) -> String {
        format!(
            "cl={:x};we={:x};su={:x};tm={:x};bd={:x};cy={:x}.{:x}.{:x}.{:x}",
            self.classes,
            self.weathers,
            self.surfaces,
            self.times,
            self.badges,
            self.countries[0],
            self.countries[1],
            self.countries[2],
            self.countries[3]
        )
    }

    /// One `key=value` pair of the profile save; unknown keys and bad values are ignored.
    pub fn decode_kv(&mut self, k: &str, v: &str) {
        let h = |s: &str| u64::from_str_radix(s, 16).unwrap_or(0);
        let all = (1u64 << BADGE_COUNT) - 1;
        match k {
            "cl" => self.classes = (h(v) & ((1u64 << crate::cars::CLASS_COUNT) - 1)) as u32,
            "we" => self.weathers = (h(v) & ((1u64 << crate::env::WEATHER_COUNT) - 1)) as u16,
            "su" => self.surfaces = (h(v) & ((1u64 << crate::env::SURFACE_COUNT) - 1)) as u8,
            "tm" => self.times = (h(v) & 0xF) as u8,
            "bd" => self.badges = (h(v) & all) as u32,
            "cy" => {
                for (i, part) in v.split('.').take(4).enumerate() {
                    self.countries[i] = h(part);
                }
            }
            _ => {}
        }
    }
}
