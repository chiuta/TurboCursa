//! Start-of-game selection screen: choose a vehicle class and the difficulty level.
use crate::career_ui::{class_name, fill, level_name, tx, txc};
use crate::cars::{CarSpec, VehicleClass, CLASS_COUNT, CLASS_TOTAL};
use crate::env::{SURFACE_COUNT, WEATHER_COUNT};
use crate::game::{Input, LEVEL_COUNT};
use crate::i18n_career::{t, C};
use crate::{render, H, W};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub class: usize,
    pub level: usize,
    /// 0 = automatic, k = weather k-1
    pub weather: usize,
    /// 0 = automatic, k = surface k-1
    pub surface: usize,
    /// tyre set: 0 = automatic (the best for the conditions), k = set k-1
    pub tyre: usize,
    /// set-up index 0..=3
    pub setup: usize,
}

/// What the start screen shows about the player's profile.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProfileInfo {
    pub rank: u16,
    pub xp: u64,
    pub next: u64,
    pub streak: u16,
    /// collection: badges earned, vehicle classes, weathers, surfaces and countries finished
    pub badges: u16,
    pub classes: u8,
    pub weathers: u8,
    pub surfaces: u8,
    pub countries: u8,
    /// unlocked service vehicles (bit 0 police, 1 fire, 2 ambulance, 3 helicopter)
    pub specials: u8,
    /// 0 none, 1 take a break, 2 done for today
    pub advice: u8,
}

/// The options page of the start screen: everything that used to need a command-line flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Extras {
    /// 0 normal, 1 calm, 2 drive, 3 off
    pub music: u8,
    pub mph: bool,
    /// key layout of the native build, 0..=3
    pub keys: u8,
    /// visible pack size: 0 = no pack, else 8..=16
    pub pack: u8,
    pub inverted: bool,
    /// 0 = one or two players, 3 or 4 = split screen
    pub players: u8,
    pub garage: crate::garage::Garage,
    /// the browser build: the key layouts are not selectable there
    pub web: bool,
    /// the game runs with the stock car (daily, weekly, shared stage): upgrades are not for sale
    pub stock: bool,
}

const PACK_STEPS: [u8; 4] = [0, 8, 12, 16];
/// rows of the options page: 0-5 settings, 6 the code field, 7.. the garage upgrades
const ROW_CODE: usize = 6;
const ROW_GARAGE: usize = 7;
const EXTRA_ROWS: usize = ROW_GARAGE + crate::garage::UPGRADES;

impl Extras {
    pub fn sanitized(mut self) -> Extras {
        self.music = self.music.min(crate::audio::STYLE_COUNT - 1);
        self.keys = self.keys.min(crate::settings::KEY_SCHEMES as u8 - 1);
        self.pack = if self.pack == 0 {
            0
        } else {
            self.pack.clamp(8, 16)
        };
        self.players = if self.players < 3 {
            0
        } else {
            self.players.min(4)
        };
        self.garage = self.garage.sanitized();
        self
    }

    /// The part that is stored as player settings (the paint job is not chosen here).
    pub fn settings(&self) -> crate::settings::Settings {
        crate::settings::Settings {
            music: self.music,
            mph: self.mph,
            keys: self.keys,
            livery: crate::settings::livery(),
        }
        .sanitized()
    }
}

fn step(v: usize, n: usize, right: bool) -> usize {
    if right {
        (v + 1) % n
    } else {
        (v + n - 1) % n
    }
}

pub struct PickApp {
    /// the options page values; read them back with `take_extras` after every update
    pub extras: Extras,
    extras_dirty: bool,
    row: usize,
    /// the code being typed on the options page
    typing: Option<String>,
    submitted: Option<String>,
    /// feedback line: key of the message and its remaining seconds
    note: Option<(&'static str, f32)>,
    pub class: usize,
    pub level: usize,
    pub weather: usize,
    pub surface: usize,
    pub tyre: usize,
    pub setup: usize,
    /// 0 = vehicle page, 1 = service page, 2 = options page
    pub page: u8,
    pub info: ProfileInfo,
    forecast: Option<(crate::env::Weather, crate::env::Surface)>,
    t: f32,
    prev: [bool; 4],
}

impl PickApp {
    /// A service vehicle that the profile has not unlocked yet.
    pub fn locked(&self, class: usize) -> bool {
        class >= CLASS_COUNT && (self.info.specials >> (class - CLASS_COUNT).min(7)) & 1 == 0
    }

    pub fn new(class: usize, level: usize) -> PickApp {
        PickApp {
            extras: Extras::default(),
            extras_dirty: false,
            row: 0,
            typing: None,
            submitted: None,
            note: None,
            class: class.min(CLASS_TOTAL - 1),
            level: level.min(LEVEL_COUNT - 1),
            weather: 0,
            surface: 0,
            tyre: 0,
            setup: 0,
            page: 0,
            info: ProfileInfo::default(),
            forecast: None,
            t: 0.0,
            prev: [true; 4], // a key already held at the start does not move the cursor
        }
    }

    /// Show the forecast of the stage with this seed (weather and surface are known before the start).
    pub fn with_forecast(mut self, seed: u64) -> PickApp {
        let s = crate::env::auto_surface(seed);
        self.forecast = Some((crate::env::auto_weather(seed, s), s));
        self
    }

    pub fn with_profile(mut self, info: ProfileInfo) -> PickApp {
        self.info = info;
        if self.locked(self.class) {
            self.class = 0;
        }
        self
    }

    /// The options page starts from the values the host holds.
    pub fn with_extras(mut self, e: Extras) -> PickApp {
        self.extras = e.sanitized();
        self
    }

    /// The changed options, once, so the host can store them (`None` when nothing changed).
    pub fn take_extras(&mut self) -> Option<Extras> {
        std::mem::take(&mut self.extras_dirty).then_some(self.extras)
    }

    /// Backspace: erases a character while a code is typed (an empty line cancels), otherwise the
    /// vehicle page opens the options page and any other page goes back to it.
    pub fn back(&mut self) {
        if let Some(t) = &mut self.typing {
            if t.pop().is_none() {
                self.typing = None;
            }
            return;
        }
        self.page = if self.page == 0 { 2 } else { 0 };
    }

    pub fn is_typing(&self) -> bool {
        self.typing.is_some()
    }

    /// A typed character (printable ASCII only, bounded).
    pub fn type_char(&mut self, c: char) {
        if let Some(t) = &mut self.typing {
            if c.is_ascii_graphic() && t.len() < crate::codes::MAX_LEN {
                t.push(c);
            }
        }
    }

    /// The code the player confirmed with Enter, once.
    pub fn take_code(&mut self) -> Option<String> {
        self.submitted.take()
    }

    /// Feedback for the last code: a banner key such as "CODE OK" or "CODE INVALID".
    pub fn set_note(&mut self, key: &'static str) {
        self.note = Some((key, 4.0));
    }

    fn change(&mut self, right: bool) {
        let e = &mut self.extras;
        match self.row {
            0 => e.music = step(e.music as usize, crate::audio::STYLE_COUNT as usize, right) as u8,
            1 => e.mph = !e.mph,
            2 if !e.web => {
                e.keys = step(e.keys as usize, crate::settings::KEY_SCHEMES, right) as u8
            }
            3 => {
                let i = PACK_STEPS.iter().position(|&n| n == e.pack).unwrap_or(0);
                e.pack = PACK_STEPS[step(i, PACK_STEPS.len(), right)];
            }
            4 => e.inverted = !e.inverted,
            5 => {
                e.players = match (e.players, right) {
                    (0, true) | (4, false) => 3,
                    (3, true) => 4,
                    (3, false) | (4, true) => 0,
                    (_, true) => 3,
                    (_, false) => 4,
                }
            }
            _ => return,
        }
        self.extras_dirty = true;
        self.extras.settings().apply();
    }

    fn buy(&mut self) {
        if self.row >= ROW_GARAGE
            && !self.extras.stock
            && self.extras.garage.buy(self.row - ROW_GARAGE)
        {
            self.extras_dirty = true;
        }
    }

    fn conditions(&self) -> Option<(crate::env::Weather, crate::env::Surface)> {
        use crate::env::{Surface, Weather};
        let (fw, fs) = self.forecast?;
        let s = if self.surface >= 1 && self.surface <= SURFACE_COUNT {
            Surface::from_index(self.surface - 1)
        } else {
            fs
        };
        let w = if self.weather >= 1 && self.weather <= WEATHER_COUNT {
            Weather::from_index(self.weather - 1)
        } else if self.surface != 0 {
            crate::env::auto_weather(0, s)
        } else {
            fw
        };
        Some((w, s))
    }

    /// Left/right: vehicle, up/down: difficulty, Enter: start. Returns the choice once confirmed.
    pub fn update(&mut self, dt: f32, inp: &Input, enter: bool) -> Option<Choice> {
        if dt.is_finite() {
            self.t += dt.clamp(0.0, 0.1);
        }
        let now = [inp.accel, inp.brake, inp.left, inp.right];
        let edge = |i: usize| now[i] && !self.prev[i];
        let (up, down, left, right) = (edge(0), edge(1), edge(2), edge(3));
        self.prev = now;
        let (nw, ns) = (WEATHER_COUNT + 1, SURFACE_COUNT + 1);
        if let Some((_, t)) = &mut self.note {
            *t -= dt.clamp(0.0, 0.1);
            if *t <= 0.0 {
                self.note = None;
            }
        }
        if self.page == 2 && self.typing.is_some() {
            // the keys type: the arrows and letters must not move the cursor
            if enter {
                self.submitted = self.typing.take().filter(|t| !t.is_empty());
            }
            return None;
        }
        if self.page == 2 {
            if down {
                self.row = (self.row + 1) % EXTRA_ROWS;
            }
            if up {
                self.row = (self.row + EXTRA_ROWS - 1) % EXTRA_ROWS;
            }
            if left || right {
                self.change(right);
            }
            if enter {
                if self.row >= ROW_GARAGE {
                    self.buy();
                } else if self.row == ROW_CODE {
                    self.typing = Some(String::new());
                } else {
                    self.page = 0;
                }
            }
            return None;
        }
        if self.page == 1 {
            // service: left/right tyres (auto + 4), up/down set-up
            let nt = crate::env::TYRE_COUNT + 1;
            if right {
                self.tyre = (self.tyre + 1) % nt;
            }
            if left {
                self.tyre = (self.tyre + nt - 1) % nt;
            }
            let nsu = crate::env::SETUP_COUNT;
            if down {
                self.setup = (self.setup + 1) % nsu;
            }
            if up {
                self.setup = (self.setup + nsu - 1) % nsu;
            }
        } else if inp.turbo {
            // turbo held: left/right weather, up/down surface
            if right {
                self.weather = (self.weather + 1) % nw;
            }
            if left {
                self.weather = (self.weather + nw - 1) % nw;
            }
            if down {
                self.surface = (self.surface + 1) % ns;
            }
            if up {
                self.surface = (self.surface + ns - 1) % ns;
            }
        } else {
            if right {
                self.class = (self.class + 1) % CLASS_TOTAL;
            }
            if left {
                self.class = (self.class + CLASS_TOTAL - 1) % CLASS_TOTAL;
            }
            if down {
                self.level = (self.level + 1) % LEVEL_COUNT;
            }
            if up {
                self.level = (self.level + LEVEL_COUNT - 1) % LEVEL_COUNT;
            }
        }
        if !enter {
            return None;
        }
        if self.locked(self.class) {
            return None;
        }
        if self.page == 0 {
            self.page = 1;
            return None;
        }
        Some(Choice {
            class: self.class,
            level: self.level,
            weather: self.weather,
            surface: self.surface,
            tyre: self.tyre,
            setup: self.setup,
        })
    }

    pub fn render(&self, fb: &mut [u32]) {
        if fb.len() != W * H {
            return;
        }
        if self.page == 2 {
            self.render_extras(fb);
            return;
        }
        if self.page == 1 {
            self.render_service(fb);
            return;
        }
        for y in 0..H {
            let k = y as f32 / H as f32;
            let c = ((10.0 + 24.0 * k) as u32) << 16
                | ((14.0 + 30.0 * k) as u32) << 8
                | (50.0 + 50.0 * k) as u32;
            fb[y * W..(y + 1) * W].fill(c);
        }
        txc(fb, 14, t(C::PickTitle), 2, 0xFFD23C);
        let car = CarSpec::default().with_class(VehicleClass::from_index(self.class));
        fill(fb, 150, 56, 490, 232, 0x10163A);
        fill(fb, 150, 218, 490, 232, 0x1A1F44);
        render::draw_car_preview(fb, W, H, 320.0, 216.0, 210.0, &car);
        txc(
            fb,
            240,
            &format!(
                "{}  {}/{}",
                class_name(self.class),
                self.class + 1,
                CLASS_TOTAL
            ),
            2,
            0xFFFFFF,
        );
        let bars = [
            (C::TopSpeed, (car.top - 0.5) / 0.8),
            (C::Accel, (car.accel - 0.5) / 0.8),
            (C::Grip, (car.grip - 0.5) / 0.8),
            (C::Size, (car.width - 0.4) / 1.1),
            (C::Tough, (car.crash_keep - 0.5) / 1.1),
        ];
        for (i, (k, v)) in bars.iter().enumerate() {
            let y = 268 + i as i32 * 14;
            tx(fb, 150, y, t(*k), 1, 0x9AA3C7);
            fill(fb, 300, y, 490, y + 8, 0x1C2444);
            fill(
                fb,
                300,
                y,
                300 + (190.0 * v.clamp(0.04, 1.0)) as i32,
                y + 8,
                0x3CDC78,
            );
        }
        if self.locked(self.class) {
            fill(fb, 150, 56, 490, 232, 0x0A0C1C);
            txc(fb, 120, t(C::Locked), 3, 0xFF6A3C);
            let req = [C::Req0, C::Req1, C::Req2, C::Req3][(self.class - CLASS_COUNT).min(3)];
            txc(fb, 170, t(req), 1, 0xFFD23C);
        }
        let lv = format!("{}: {}", t(C::Level), level_name(self.level as u8));
        txc(fb, 344, &lv, 1, 0xFFD23C);
        for i in 0..LEVEL_COUNT as i32 {
            let on = i <= self.level as i32;
            fill(
                fb,
                270 + i * 20,
                358,
                286 + i * 20,
                364,
                if on { 0xFF6A3C } else { 0x2A3156 },
            );
        }
        let wn = if self.weather == 0 {
            t(C::Auto).to_string()
        } else {
            t(WX[self.weather - 1]).to_string()
        };
        let sn = if self.surface == 0 {
            t(C::Auto).to_string()
        } else {
            t(SF[self.surface - 1]).to_string()
        };
        txc(
            fb,
            369,
            &format!("{}: {}   {}: {}", t(C::Weather), wn, t(C::Surface), sn),
            1,
            0x8FD8FF,
        );
        txc(fb, 384, t(C::PickHint), 1, 0x9AA3C7);
        txc(
            fb,
            392,
            &format!(
                "{}: {}",
                crate::i18n::banner("BACK KEY"),
                crate::i18n::banner("EXTRAS")
            ),
            1,
            0x6F79A8,
        );
        self.render_profile(fb);
    }

    fn render_profile(&self, fb: &mut [u32]) {
        let i = &self.info;
        if i.rank == 0 && i.xp == 0 && i.streak == 0 {
            return;
        }
        tx(
            fb,
            10,
            10,
            &format!("{} {}/999", t(C::Rank), i.rank),
            1,
            0xFFD23C,
        );
        fill(fb, 10, 20, 110, 25, 0x1C2444);
        if let Some(w) = (100 * i.xp.min(i.next)).checked_div(i.next) {
            fill(fb, 10, 20, 10 + w as i32, 25, 0x3CDC78);
        }
        if i.streak > 1 {
            tx(
                fb,
                10,
                30,
                &format!("{} {}", i.streak, t(C::Streak)),
                1,
                0x9AA3C7,
            );
        }
        if i.badges > 0 || i.classes > 0 {
            let s = format!(
                "{} {}/{}  {} {}/{}  {} {}/{}  {} {}/{}  {} {}",
                t(C::Badges),
                i.badges,
                crate::badges::BADGE_COUNT,
                t(C::Cars),
                i.classes,
                crate::cars::CLASS_COUNT,
                t(C::Weather),
                i.weathers,
                crate::env::WEATHER_COUNT,
                t(C::Terrain),
                i.surfaces,
                crate::env::SURFACE_COUNT,
                t(C::Flags),
                i.countries,
            );
            tx(fb, 10, 38, &s, 1, 0x9AA3C7);
        }
        match i.advice {
            1 => txc(fb, 400, t(C::TakeBreak), 1, 0xFF9A3C),
            2 => txc(fb, 400, t(C::Tomorrow), 1, 0xFF6A3C),
            _ => {}
        }
    }

    /// Options page: music, units, keys, pack, split screen and the garage.
    fn render_extras(&self, fb: &mut [u32]) {
        use crate::i18n::banner as b;
        for y in 0..H {
            let k = y as f32 / H as f32;
            let c = ((16.0 + 14.0 * k) as u32) << 16
                | ((18.0 + 22.0 * k) as u32) << 8
                | (40.0 + 30.0 * k) as u32;
            fb[y * W..(y + 1) * W].fill(c);
        }
        txc(fb, 12, b("EXTRAS"), 3, 0xFFD23C);
        let e = &self.extras;
        let onoff = |v: bool| if v { b("ON") } else { b("OFF") }.to_string();
        let music = [
            b("MUSIC NORMAL"),
            b("MUSIC CALM"),
            b("MUSIC DRIVE"),
            b("OFF"),
        ];
        let rows: [(String, String); 6] = [
            (
                b("MUSIC").to_string(),
                music[e.music.min(3) as usize].to_string(),
            ),
            (
                b("SPEED UNIT").to_string(),
                if e.mph { "MPH" } else { "KM/H" }.to_string(),
            ),
            (
                b("KEYS").to_string(),
                if e.web {
                    b("NATIVE ONLY").to_string()
                } else {
                    crate::settings::scheme_name(e.keys).to_string()
                },
            ),
            (
                b("PACK").to_string(),
                if e.pack == 0 {
                    b("OFF").to_string()
                } else {
                    e.pack.to_string()
                },
            ),
            (b("INVERTED GRID").to_string(), onoff(e.inverted)),
            (
                b("PLAYERS").to_string(),
                if e.players == 0 {
                    "1-2".to_string()
                } else {
                    e.players.to_string()
                },
            ),
        ];
        let mut y = 46;
        for (i, (l, v)) in rows.iter().enumerate() {
            let on = i == self.row;
            fill(
                fb,
                120,
                y,
                520,
                y + 21,
                if on { 0xFF6A3C } else { 0x2A3156 },
            );
            tx(fb, 132, y + 7, l, 1, 0xFFFFFF);
            tx(fb, 330, y + 7, v, 1, 0xFFFFFF);
            let note = match i {
                3 | 4 => b("SOLO RACE"),
                5 => b("VERSUS MODES"),
                _ => "",
            };
            tx(fb, 430, y + 7, note, 1, 0x9AA3C7);
            y += 25;
        }
        // the code field
        {
            let on = self.row == ROW_CODE;
            fill(
                fb,
                120,
                y,
                520,
                y + 21,
                if on { 0xFF6A3C } else { 0x2A3156 },
            );
            tx(fb, 132, y + 7, b("ENTER CODE"), 1, 0xFFFFFF);
            let shown = match &self.typing {
                Some(t) => {
                    let tail: String = t
                        .chars()
                        .rev()
                        .take(30)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                    format!("{tail}_")
                }
                None => b("CODE HINT").to_string(),
            };
            tx(fb, 250, y + 7, &shown, 1, 0xFFFFFF);
            y += 25;
        }
        tx(
            fb,
            120,
            y + 4,
            &format!("{}  {} {}", b("GARAGE"), e.garage.coins, b("COINS")),
            2,
            0xFFD23C,
        );
        y += 24;
        for i in 0..crate::garage::UPGRADES {
            let on = self.row == ROW_GARAGE + i;
            fill(
                fb,
                120,
                y,
                520,
                y + 21,
                if on { 0xFF6A3C } else { 0x2A3156 },
            );
            tx(fb, 132, y + 7, b(crate::garage::NAMES[i]), 1, 0xFFFFFF);
            let lv = e.garage.lv[i] as i32;
            for k in 0..crate::garage::MAX_LEVEL as i32 {
                let c = if k < lv { 0x3CDC78 } else { 0x1C2444 };
                fill(fb, 250 + k * 22, y + 5, 266 + k * 22, y + 15, c);
            }
            let price = match e.garage.price(i) {
                _ if e.stock => b("STOCK MODE").to_string(),
                Some(p) => format!("{p} {}", b("COINS")),
                None => b("MAX").to_string(),
            };
            tx(fb, 340, y + 7, &price, 1, 0xFFFFFF);
            y += 25;
        }
        if let Some((k, _)) = self.note {
            txc(fb, y + 3, b(k), 1, 0x3CDC78);
        } else {
            txc(fb, y + 3, b("EXTRAS HINT"), 1, 0x9AA3C7);
        }
    }

    /// Service area: forecast of the stage, tyres and set-up.
    fn render_service(&self, fb: &mut [u32]) {
        use crate::env::{Setup, Tyre};
        for y in 0..H {
            let k = y as f32 / H as f32;
            let c = ((20.0 + 18.0 * k) as u32) << 16
                | ((20.0 + 22.0 * k) as u32) << 8
                | (26.0 + 30.0 * k) as u32;
            fb[y * W..(y + 1) * W].fill(c);
        }
        txc(fb, 14, t(C::Service), 3, 0xFFD23C);
        let cond = self.conditions();
        if let Some((w, s)) = cond {
            let wn = t(WX[w.index()]);
            let sn = t(SF[s.index()]);
            txc(
                fb,
                58,
                &format!("{}: {} / {}", t(C::Forecast), wn, sn),
                2,
                0x8FD8FF,
            );
        }
        let best = cond.map(|(w, s)| Tyre::best_for(w, s));
        let names = [C::TyRoad, C::TyRain, C::TySnow, C::TyOff];
        tx(fb, 120, 100, t(C::Tyres), 2, 0xFFFFFF);
        for i in 0..=crate::env::TYRE_COUNT {
            let on = i == self.tyre;
            let label = if i == 0 { t(C::Auto) } else { t(names[i - 1]) };
            let x = 120 + (i as i32) * 78;
            fill(
                fb,
                x,
                124,
                x + 72,
                150,
                if on { 0xFF6A3C } else { 0x2A3156 },
            );
            txc_at(fb, x, 72, 133, label, 1, 0xFFFFFF);
            if i >= 1 && best == Some(Tyre::from_index(i - 1)) {
                fill(fb, x, 152, x + 72, 155, 0x3CDC78);
            }
        }
        tx(fb, 120, 180, t(C::Setup), 2, 0xFFFFFF);
        let sn = [C::SuBalanced, C::SuSprint, C::SuGrip, C::SuEco];
        for (i, c) in sn.iter().enumerate() {
            let on = i == self.setup;
            let y = 204 + i as i32 * 26;
            fill(
                fb,
                120,
                y,
                520,
                y + 20,
                if on { 0xFF6A3C } else { 0x2A3156 },
            );
            tx(fb, 130, y + 7, t(*c), 1, 0xFFFFFF);
            let (a, tp, gr, fu) = Setup::from_index(i).mods();
            let s = format!(
                "A{:+} T{:+} G{:+} F{:+}",
                ((a - 1.0) * 100.0).round() as i32,
                ((tp - 1.0) * 100.0).round() as i32,
                ((gr - 1.0) * 100.0).round() as i32,
                ((fu - 1.0) * 100.0).round() as i32
            );
            tx(fb, 330, y + 7, &s, 1, 0xC8D0F0);
        }
        txc(fb, 340, t(C::ServiceHint), 1, 0x9AA3C7);
    }
}

fn txc_at(fb: &mut [u32], x: i32, w: i32, y: i32, s: &str, scale: i32, col: u32) {
    let tw = crate::font::text_width(s, scale);
    tx(fb, x + (w - tw) / 2, y, s, scale, col);
}

pub const WX: [C; 11] = [
    C::Wx0,
    C::Wx1,
    C::Wx2,
    C::Wx3,
    C::Wx4,
    C::Wx5,
    C::Wx6,
    C::Wx7,
    C::Wx8,
    C::Wx9,
    C::Wx10,
];
pub const SF: [C; 6] = [C::Sf0, C::Sf1, C::Sf2, C::Sf3, C::Sf4, C::Sf5];
