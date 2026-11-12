//! One race car per country: a livery painted from the national colours (approximate, original art)
//! and a handling profile. The profile is a zero-sum budget: whatever a car gains in one trait it
//! pays for in another, so no country has a strictly better car.

use crate::career::{splitmix, COUNTRIES};

/// Flag colours and layout per country, as `CODE=<1-3 colours><layout>`.
/// Colours: R red, W white, B blue, G green, Y yellow, K black, O orange, L light blue, M maroon.
/// Layouts: H horizontal bands, V vertical bands, C cross, X diagonal band, S central disc, T hoist triangle.
const FLAGS: &str = "RO=BYRV;FR=BWRV;DE=KRYH;IT=GWRV;ES=RYRH;PT=GRYV;GB=BRWC;IE=GWOV;NL=RWBH;BE=KYRV;LU=RWLH;CH=RWC;AT=RWRH;SE=BYC;NO=RWBC;FI=WBC;DK=RWC;IS=BRWC;PL=WRH;HU=RWGH;CZ=WRBT;SK=WBRH;SI=WBRH;HR=RWBH;RS=RBWH;BA=BYWT;MK=RYS;AL=RKS;GR=BWH;BG=WGRH;MD=BYRV;UA=BYH;BY=RGWH;LT=YGRH;LV=MWH;EE=BKWH;MT=WRV;CY=WOGS;RU=WBRH;AD=BYRV;ME=RYS;MC=RWH;LI=BRH;SM=WLH;VA=YWV;\
JP=WRS;CN=RYS;IN=OWGH;KR=WRBS;TW=RBWS;TH=RWBH;VN=RYS;ID=RWH;PH=BRWT;MY=RWBH;SG=RWH;BD=GRS;LK=MYGV;NP=RBT;MN=RBRV;KZ=LYS;UZ=LWGH;PK=GWV;IR=GWRH;IQ=RWKH;SA=GWS;AE=RGWV;QA=MWV;JO=KWGT;LB=RWGH;IL=WBH;TR=RWS;GE=WRC;AM=RBOH;AZ=LRGH;AF=KRGV;BH=RWV;BT=YOT;BN=YWKX;KH=BRWH;KP=BRWH;KW=GWRH;KG=RYS;LA=RBWH;MV=RGWS;MM=YGRH;OM=RWGH;PS=KWGT;SY=RWKH;TJ=RWGH;TL=RKYT;TM=GRWV;YE=RWKH;\
EG=RWKH;MA=RGS;DZ=GWRS;TN=RWS;LY=RKGH;SN=GYRV;GH=RYGH;CI=OWGV;NG=GWGV;CM=GRYV;ET=GYRH;KE=KRGH;TZ=GLKX;UG=KYRH;AO=RKS;MZ=GKYT;ZW=GYRH;ZA=GRBH;BJ=GYRV;BW=LWKH;BF=RGYH;BI=RGWX;CV=BWRH;CF=BWGH;TD=BYRV;KM=GYWH;CG=GYRX;CD=LRYX;DJ=LGWT;GQ=GWRT;ER=GLRT;SZ=BYRH;GA=GYBH;GM=RBGH;GN=RYGV;GW=RYGH;LS=BWGH;LR=RWBH;MG=WRGV;MW=KRGH;ML=GYRV;MR=GYRH;MU=RBYH;NA=BRGX;NE=OWGH;RW=LYGH;ST=GYRT;SC=BYRX;SL=GWBH;SO=LWS;SS=KRGT;SD=RWKT;TG=GYRH;ZM=GROH;\
US=BRWH;CA=RWRV;MX=GWRV;GT=LWV;HN=BWBH;NI=BWBH;SV=BWBH;CR=BWRH;PA=BRWH;CU=BWRT;JM=GKYX;HT=BRH;DO=BRWC;BS=LYKT;TT=RWKX;AG=RKBH;BB=BYBV;BZ=BRWH;DM=GYKH;GD=RYGH;KN=GYKX;LC=LYKT;VC=BYGV;\
BR=GYBS;AR=LWYH;CL=WBRH;CO=YBRH;PE=RWRV;UY=WBYH;VE=YBRH;EC=YBRH;BO=RYGH;PY=RWBH;GY=GYRT;SR=GWRH;\
AU=BRWH;NZ=BRWS;FJ=LRWH;PG=RKYX;WS=RBWH;KI=RBYH;MH=BOWX;FM=LWS;NR=BYWH;PW=LYS;SB=BYGX;TO=RWRC;TV=LYBX;VU=RGKX";

fn colour(c: u8) -> u32 {
    match c {
        b'R' => 0xCE2028,
        b'W' => 0xF0F0F0,
        b'B' => 0x1E46AA,
        b'G' => 0x008C46,
        b'Y' => 0xFAC814,
        b'K' => 0x18181C,
        b'O' => 0xF58214,
        b'L' => 0x50AAE6,
        b'M' => 0x801828,
        _ => 0x808080,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout {
    Horizontal,
    Vertical,
    Cross,
    Diagonal,
    Disc,
    Triangle,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Archetype {
    Balanced,
    TopSpeed,
    Dragster,
    Grip,
    Turbo,
}

/// Vehicle class: a trade-off between speed, agility, size, durability and the clock.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VehicleClass {
    Sports,
    Sedan,
    /// tiny city car: narrow and nimble, fragile
    City,
    Truck,
    /// articulated lorry (tractor-trailer): huge, slow, nearly unstoppable
    Semi,
    Moped,
    Motorcycle,
    Trike,
    TukTuk,
    Quad,
    Buggy,
    Formula,
    Rally,
    Pickup,
    Van,
    Bus,
    Tractor,
    Limousine,
    Monster,
    Police,
    Fire,
    Ambulance,
    Helicopter,
}

/// Classes selectable from the start (codes, daily, career, design use this range).
pub const CLASS_COUNT: usize = 19;
/// Including the unlockable service vehicles.
pub const CLASS_TOTAL: usize = 23;

const ALL: [VehicleClass; CLASS_TOTAL] = [
    VehicleClass::Sports,
    VehicleClass::Sedan,
    VehicleClass::City,
    VehicleClass::Truck,
    VehicleClass::Semi,
    VehicleClass::Moped,
    VehicleClass::Motorcycle,
    VehicleClass::Trike,
    VehicleClass::TukTuk,
    VehicleClass::Quad,
    VehicleClass::Buggy,
    VehicleClass::Formula,
    VehicleClass::Rally,
    VehicleClass::Pickup,
    VehicleClass::Van,
    VehicleClass::Bus,
    VehicleClass::Tractor,
    VehicleClass::Limousine,
    VehicleClass::Monster,
    VehicleClass::Police,
    VehicleClass::Fire,
    VehicleClass::Ambulance,
    VehicleClass::Helicopter,
];

const NAMES: [&str; CLASS_TOTAL] = [
    "sports",
    "sedan",
    "city",
    "truck",
    "semi",
    "moped",
    "motorcycle",
    "trike",
    "tuktuk",
    "quad",
    "buggy",
    "formula",
    "rally",
    "pickup",
    "van",
    "bus",
    "tractor",
    "limousine",
    "monster",
    "police",
    "fire",
    "ambulance",
    "helicopter",
];

impl VehicleClass {
    pub fn from_index(i: usize) -> VehicleClass {
        ALL[i.min(CLASS_TOTAL - 1)]
    }
    /// Police, fire engine, ambulance, helicopter: unlocked by performance.
    pub fn is_special(self) -> bool {
        self.index() >= CLASS_COUNT
    }
    /// Emergency vehicles may break traffic rules and traffic gives way.
    pub fn is_heli(self) -> bool {
        self == VehicleClass::Helicopter
    }
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        NAMES[self.index()]
    }
    /// "sports", "city", "3", ...
    pub fn parse(s: &str) -> Option<VehicleClass> {
        let t = s.trim().to_ascii_lowercase();
        if let Ok(i) = t.parse::<usize>() {
            return (i < CLASS_COUNT).then(|| Self::from_index(i));
        }
        NAMES.iter().position(|n| *n == t).map(Self::from_index)
    }
    /// Off-road ability: how much of a low-grip surface's penalty the vehicle shrugs off.
    pub fn traction(self) -> f32 {
        use VehicleClass::*;
        match self {
            Rally | Buggy => 0.5,
            Quad => 0.45,
            Monster => 0.4,
            Tractor => 0.35,
            Pickup | Truck => 0.15,
            Bus | Van | Semi => 0.05,
            Formula => -0.25,
            Helicopter => 1.0,
            Fire => 0.2,
            Moped | Motorcycle => -0.1,
            _ => 0.0,
        }
    }
    /// (top, accel, grip, width, crash_keep, prop_keep, time_mult, extra turbo charges)
    fn mods(self) -> (f32, f32, f32, f32, f32, f32, f32, i8) {
        use VehicleClass::*;
        match self {
            Sports => (1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0),
            Sedan => (0.97, 0.98, 1.02, 1.05, 1.1, 1.1, 1.0, 0),
            City => (0.90, 1.06, 1.10, 0.78, 0.85, 0.9, 0.95, 0),
            Truck => (0.90, 0.86, 0.88, 1.18, 1.25, 1.4, 1.10, 0),
            Semi => (0.88, 0.72, 0.80, 1.32, 1.4, 1.8, 1.20, 1),
            Moped => (0.62, 1.00, 1.20, 0.45, 0.60, 0.70, 1.25, 0),
            Motorcycle => (1.08, 1.18, 1.20, 0.50, 0.60, 0.70, 1.00, 0),
            Trike => (0.85, 0.95, 0.95, 0.70, 0.80, 0.90, 1.10, 0),
            TukTuk => (0.66, 0.90, 1.05, 0.72, 0.75, 0.85, 1.25, 0),
            Quad => (0.90, 1.10, 1.10, 0.75, 0.90, 1.00, 1.00, 0),
            Buggy => (0.95, 1.10, 1.15, 0.90, 1.00, 1.10, 1.00, 0),
            Formula => (1.20, 1.20, 1.20, 0.95, 0.55, 0.50, 0.90, 0),
            Rally => (1.00, 1.05, 1.12, 1.00, 1.10, 1.10, 1.00, 0),
            Pickup => (0.95, 0.95, 0.95, 1.10, 1.15, 1.25, 1.00, 0),
            Van => (0.92, 0.90, 0.90, 1.12, 1.20, 1.30, 1.05, 0),
            Bus => (0.82, 0.75, 0.82, 1.30, 1.35, 1.70, 1.20, 0),
            Tractor => (0.62, 0.78, 0.85, 1.20, 1.40, 2.00, 1.35, 0),
            Limousine => (0.95, 0.80, 0.80, 1.28, 1.10, 1.20, 1.10, 0),
            Monster => (0.85, 0.90, 0.80, 1.35, 1.50, 2.00, 1.10, 1),
            Police => (1.12, 1.15, 1.12, 1.0, 1.1, 1.1, 1.0, 1),
            Fire => (0.84, 0.80, 0.82, 1.30, 1.45, 1.8, 1.15, 0),
            Ambulance => (0.98, 1.0, 0.98, 1.12, 1.15, 1.2, 1.0, 0),
            Helicopter => (1.12, 1.20, 1.30, 0.8, 2.0, 2.0, 1.0, 1),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CarSpec {
    pub class: VehicleClass,
    /// body width factor (collision box and sprite)
    pub width: f32,
    /// how much speed survives a crash with traffic / with the roadside (1 = sports car)
    pub crash_keep: f32,
    pub prop_keep: f32,
    /// multiplier on the clock (start time and checkpoint bonuses)
    pub time_mult: f32,
    /// how well the vehicle copes with low-grip surfaces (0 = road tyres, 0.5 = off-road; negative = slicks)
    pub traction: f32,
    pub country: u8,
    pub body: u32,
    pub c2: u32,
    pub c3: u32,
    pub layout: Layout,
    /// multipliers on top speed, acceleration and steering/grip
    pub top: f32,
    pub accel: f32,
    pub grip: f32,
    /// turbo charges per stage (2..=4)
    pub turbos: u8,
    /// garage upgrades are already applied (so a spec handed to `set_car` twice is not upgraded twice)
    pub tuned: bool,
    pub archetype: Archetype,
}

impl Default for CarSpec {
    /// The neutral red car every other mode used before countries existed.
    fn default() -> Self {
        CarSpec {
            class: VehicleClass::Sports,
            width: 1.0,
            crash_keep: 1.0,
            prop_keep: 1.0,
            time_mult: 1.0,
            traction: 0.0,
            country: u8::MAX,
            body: 0xD61C24,
            c2: 0xD61C24,
            c3: 0xD61C24,
            layout: Layout::Disc,
            top: 1.0,
            accel: 1.0,
            grip: 1.0,
            turbos: 3,
            tuned: false,
            archetype: Archetype::Balanced,
        }
    }
}

fn flag_of(code: &str) -> Option<&'static str> {
    FLAGS.split(';').find_map(|e| {
        let (c, v) = e.split_once('=')?;
        (c == code).then_some(v)
    })
}

impl CarSpec {
    /// Whether the neutral default (no livery overlay) is in use.
    pub fn is_default(&self) -> bool {
        self.country == u8::MAX
    }

    pub fn for_country(idx: usize) -> CarSpec {
        let idx = idx.min(COUNTRIES.len() - 1);
        let code = COUNTRIES[idx].code;
        let (cols, layout) = match flag_of(code) {
            Some(v) if v.len() >= 3 => {
                let b = v.as_bytes();
                let l = match b[b.len() - 1] {
                    b'H' => Layout::Horizontal,
                    b'V' => Layout::Vertical,
                    b'C' => Layout::Cross,
                    b'X' => Layout::Diagonal,
                    b'S' => Layout::Disc,
                    _ => Layout::Triangle,
                };
                (&b[..b.len() - 1], l)
            }
            _ => (&b"RWB"[..], Layout::Horizontal),
        };
        let c = |i: usize| colour(cols[i.min(cols.len() - 1)]);
        let h = splitmix(code.bytes().fold(0x5EED_u64, |a, b| a * 131 + b as u64));
        let u = |k: u32| ((h >> (k * 12)) & 0xFFF) as f32 / 2047.5 - 1.0; // -1..1
                                                                          // zero-sum budget over the three handling traits, tilted a little by continent character
        let tilt = match COUNTRIES[idx].cont {
            0 => [0.0, 0.0, 0.5],
            1 => [0.0, 0.6, 0.0],
            2 => [0.6, 0.0, 0.0],
            3 => [0.4, 0.3, -0.2],
            4 => [-0.2, 0.4, 0.4],
            _ => [0.2, -0.2, 0.2],
        };
        let mut d = [u(0) + tilt[0], u(1) + tilt[1], u(2) + tilt[2]];
        let mean = (d[0] + d[1] + d[2]) / 3.0;
        for x in &mut d {
            *x -= mean;
        }
        let big = d.iter().fold(1.0f32, |m, x| m.max(x.abs()));
        for x in &mut d {
            *x /= big; // uniform scale keeps the sum at zero
        }
        let tcode = (h >> 40) % 3; // 0: -1 turbo, 1: 0, 2: +1
        let turbos = 2 + tcode as u8;
        let tpay = (tcode as f32 - 1.0) * 0.03; // an extra turbo costs top speed
        let top = 1.0 + 0.05 * d[0] - tpay;
        let accel = 1.0 + 0.10 * d[1];
        let grip = 1.0 + 0.08 * d[2];
        let archetype = if tcode == 2 {
            Archetype::Turbo
        } else if d[0] > 0.45 {
            Archetype::TopSpeed
        } else if d[1] > 0.45 {
            Archetype::Dragster
        } else if d[2] > 0.45 {
            Archetype::Grip
        } else {
            Archetype::Balanced
        };
        CarSpec {
            class: VehicleClass::Sports,
            width: 1.0,
            crash_keep: 1.0,
            prop_keep: 1.0,
            time_mult: 1.0,
            traction: 0.0,
            country: idx as u8,
            body: c(0),
            c2: c(1),
            c3: c(2),
            layout,
            top,
            accel,
            grip,
            turbos,
            tuned: false,
            archetype,
        }
    }

    /// Same country profile, different vehicle class.
    pub fn with_class(mut self, class: VehicleClass) -> CarSpec {
        let (t, a, g, w, ck, pk, tm, tb) = class.mods();
        self.class = class;
        self.tuned = false;
        self.top *= t;
        self.accel *= a;
        self.grip *= g;
        self.width = w;
        self.crash_keep = ck;
        self.prop_keep = pk;
        self.time_mult = tm;
        self.traction = class.traction();
        self.turbos = (self.turbos as i8 + tb).clamp(1, 6) as u8;
        self
    }

    /// Model designation, e.g. "RO-47".
    pub fn model(&self) -> String {
        if self.is_default() {
            return "TC-00".into();
        }
        let code = COUNTRIES[self.country as usize].code;
        format!(
            "{}-{:02}",
            code,
            10 + splitmix(self.country as u64 + 99) % 90
        )
    }
}

pub fn archetype_index(a: Archetype) -> usize {
    a as usize
}

/// True when the country has a flag entry (otherwise a neutral fallback livery is used).
pub fn has_flag(code: &str) -> bool {
    flag_of(code).is_some()
}
