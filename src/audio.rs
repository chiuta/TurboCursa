//! Procedural audio: engine model, effects and generative music. Pure Rust, no samples and no
//! dependencies; the synth fills mono f32 buffers and is driven by `AudioParams` taken from the game.
use crate::fx::hash;
use crate::rng::Rng;
use std::f32::consts::TAU;
use std::sync::atomic::{AtomicU8, Ordering};

/// Music style chosen by the player: 0 normal, 1 calm, 2 drive, 3 no music.
static STYLE: AtomicU8 = AtomicU8::new(0);
pub const STYLE_COUNT: u8 = 4;

pub fn set_music_style(s: u8) {
    STYLE.store(s.min(STYLE_COUNT - 1), Ordering::Relaxed);
}

pub fn music_style() -> u8 {
    STYLE.load(Ordering::Relaxed)
}

#[derive(Clone, Copy, Default, Debug)]
pub struct AudioParams {
    /// 0..1.3 of top speed
    pub speed: f32,
    pub offroad: bool,
    pub turbo: bool,
    /// 0 forest, 1 desert, 2 dusk
    pub biome: u8,
    pub playing: bool,
    pub finished: bool,
    pub timeup: bool,
    pub time_low: bool,
    pub crashes: u32,
    pub cps: u32,
    pub turbos: u32,
    /// weather index (see `env::Weather`), daylight 0 (night) .. 1 (day)
    pub weather: u8,
    pub daylight: f32,
    /// 0..1 how close a police car is (or how full the bust meter is)
    pub cop: f32,
    /// 0..1 how close a level crossing is, and whether the train is on it
    pub train: f32,
    pub train_on: bool,
    /// 0..1 how long the ice has been carrying the car (it starts to crack)
    pub ice: f32,
    pub fuel_low: bool,
    /// engine condition 1.0 = perfect (0 = unset in `default()`, treated as perfect)
    pub engine: f32,
    pub air: bool,
    pub gear: u8,
    pub pads: u32,
    pub animals: u32,
    /// own siren of a service vehicle: 0 off, 1 police, 2 ambulance, 3 fire engine
    pub own_siren: u8,
}

/// Rain-like hiss per weather: clear, rain, storm, fog, snow, blizzard, sandstorm, hail, glaze, heat, ash.
const RAIN: [f32; 11] = [0.0, 0.5, 1.0, 0.0, 0.12, 0.25, 0.3, 0.8, 0.05, 0.0, 0.15];
/// Extra wind per weather.
const WINDY: [f32; 11] = [0.0, 0.0, 0.7, 0.0, 0.1, 1.0, 0.9, 0.2, 0.0, 0.0, 0.1];

#[derive(Clone, Copy)]
struct Voice {
    on: bool,
    kind: u8, // 0 saw, 1 pulse, 2 sine, 3 noise, 4 kick
    bus: u8,  // 0 music, 1 fx
    hp: bool,
    freq: f32,
    slide: f32,
    phase: f32,
    amp: f32,
    decay: f32,
    cutoff: f32,
    lp: f32,
}

const VOICE_OFF: Voice = Voice {
    on: false,
    kind: 0,
    bus: 0,
    hp: false,
    freq: 0.0,
    slide: 1.0,
    phase: 0.0,
    amp: 0.0,
    decay: 1.0,
    cutoff: 1000.0,
    lp: 0.0,
};

#[derive(Clone, Copy)]
struct Sched {
    at: u64,
    v: Voice,
}

pub struct Synth {
    sr: f32,
    rng: Rng,
    seed: u32,
    t: u64,
    voices: [Voice; 20],
    sched: Vec<Sched>,
    // engine / ambience
    e_f: f32,
    e_ph1: f32,
    e_ph2: f32,
    e_lp: f32,
    wind_a: f32,
    wind_b: f32,
    rumble: f32,
    turbo_lp: f32,
    off_amp: f32,
    turbo_amp: f32,
    eng_amp: f32,
    // music
    acc: f32,
    step: usize,
    bar: usize,
    prog: [usize; 4],
    music_gain: f32,
    delay: Vec<f32>,
    di: usize,
    // edge detection
    last: (u32, u32, u32),
    was_finished: bool,
    was_timeup: bool,
    // weather, police, trains, ice, warnings
    rain_lp: f32,
    siren_ph: f32,
    siren_lp: f32,
    own_ph: f32,
    own_lp: f32,
    horn1: f32,
    horn2: f32,
    thunder_t: i64,
    bell_t: i64,
    beep_t: i64,
    cricket_ph: f32,
    last2: (u8, bool, u32, u32),
}

fn midi_hz(n: f32) -> f32 {
    440.0 * 2f32.powf((n - 69.0) / 12.0)
}

const SCALES: [&[i32]; crate::NB] = [
    &[0, 2, 4, 7, 9],
    &[0, 1, 4, 5, 7, 8, 10],
    &[0, 2, 3, 5, 7, 8, 10],
    &[0, 2, 4, 6, 7, 9, 11], // world trees: lydian
    &[0, 3, 5, 7, 10],       // mesas: minor pentatonic
    &[0, 2, 4, 7, 9],        // floating isles: bright pentatonic, high root
    &[0, 2, 3, 5, 7, 9, 10], // mirror flats: dorian
    &[0, 3, 5, 7, 10],       // tundra: icy minor pentatonic
    &[0, 1, 3, 5, 6, 8, 10], // volcano: locrian, low
    &[0, 2, 5, 7, 9],        // abyss: suspended, slow
    &[0, 2, 4, 6, 8, 10],    // cosmos: whole tone
    &[0, 2, 3, 5, 7, 8, 10], // neon citadel: natural minor
    &[0, 2, 3, 7, 8],        // mycelium: hirajoshi
    &[0, 1, 4, 6, 7, 10],    // inverted world: unsettled
    &[0, 2, 4, 7, 11],       // prism desert: bright major seventh
    &[0, 2, 3, 5, 7, 9, 10], // clockwork: dorian
    &[0, 4, 7, 9, 11],       // cloud sea: airy
    &[0, 1, 5, 6, 10],       // black hole: dark, dissonant
    &[0, 2, 3, 6, 7, 8, 11], // fractal canyon: harmonic minor
];
const ROOTS: [i32; crate::NB] = [
    45, 43, 40, 47, 38, 52, 41, 50, 33, 43, 48, 45, 42, 36, 55, 40, 57, 31, 44,
];

impl Synth {
    pub fn new(sr: f32, seed: u64) -> Synth {
        let sr = if sr.is_finite() {
            sr.clamp(8000.0, 384000.0)
        } else {
            48000.0
        };
        let mut rng = Rng::new(seed ^ 0xA0D10);
        let prog = [
            0,
            1 + rng.below(4) as usize,
            2 + rng.below(3) as usize,
            rng.below(5) as usize,
        ];
        Synth {
            sr,
            rng,
            seed: seed as u32,
            t: 0,
            voices: [VOICE_OFF; 20],
            sched: Vec::new(),
            e_f: 32.0,
            e_ph1: 0.0,
            e_ph2: 0.0,
            e_lp: 0.0,
            wind_a: 0.0,
            wind_b: 0.0,
            rumble: 0.0,
            turbo_lp: 0.0,
            off_amp: 0.0,
            turbo_amp: 0.0,
            eng_amp: 0.0,
            acc: 0.0,
            step: 0,
            bar: 0,
            prog,
            music_gain: 0.0,
            delay: vec![0.0; (sr * 0.27) as usize],
            di: 0,
            last: (0, 0, 0),
            was_finished: false,
            was_timeup: false,
            rain_lp: 0.0,
            siren_ph: 0.0,
            siren_lp: 0.0,
            own_ph: 0.0,
            own_lp: 0.0,
            horn1: 0.0,
            horn2: 0.0,
            thunder_t: (sr * 5.0) as i64,
            bell_t: 0,
            beep_t: 0,
            cricket_ph: 0.0,
            last2: (1, false, 0, 0),
        }
    }

    fn trigger(&mut self, v: Voice) {
        let i = self.voices.iter().position(|x| !x.on).unwrap_or(0);
        self.voices[i] = Voice { on: true, ..v };
    }

    fn at(&mut self, delay_s: f32, v: Voice) {
        self.sched.push(Sched {
            at: self.t + (delay_s * self.sr) as u64,
            v,
        });
    }

    fn decay_mul(&self, secs: f32) -> f32 {
        (-6.9 / (secs.max(0.005) * self.sr)).exp()
    }

    fn tone(&self, kind: u8, bus: u8, freq: f32, amp: f32, secs: f32, cutoff: f32) -> Voice {
        Voice {
            on: true,
            kind,
            bus,
            hp: false,
            freq,
            slide: 1.0,
            phase: 0.0,
            amp,
            decay: self.decay_mul(secs),
            cutoff,
            lp: 0.0,
        }
    }

    fn edges(&mut self, p: &AudioParams) {
        if p.crashes > self.last.0 {
            let n = self.tone(3, 1, 0.0, 0.55, 0.38, 2200.0);
            self.trigger(n);
            let mut k = self.tone(4, 1, 95.0, 0.6, 0.3, 0.0);
            k.slide = (0.5f32).powf(1.0 / (0.25 * self.sr));
            self.trigger(k);
        }
        if p.cps > self.last.1 {
            for (i, f) in [880.0, 1108.7, 1318.5, 1760.0].iter().enumerate() {
                let v = self.tone(2, 1, *f, 0.13, 0.55, 0.0);
                self.at(i as f32 * 0.08, v);
            }
        }
        if p.turbos > self.last.2 {
            let mut v = self.tone(2, 1, 180.0, 0.12, 0.7, 0.0);
            v.slide = (5.0f32).powf(1.0 / (0.6 * self.sr));
            self.trigger(v);
        }
        self.last = (p.crashes, p.cps, p.turbos);
        if p.finished && !self.was_finished {
            for (i, f) in [523.25, 659.25, 783.99, 1046.5, 1318.5].iter().enumerate() {
                let v = self.tone(1, 1, *f, 0.12, 0.6, 3500.0);
                self.at(i as f32 * 0.13, v);
            }
        }
        if p.timeup && !self.was_timeup {
            for (i, f) in [440.0, 369.99, 311.13, 261.63].iter().enumerate() {
                let v = self.tone(0, 1, *f, 0.16, 0.45, 1200.0);
                self.at(i as f32 * 0.24, v);
            }
        }
        self.was_finished = p.finished;
        self.was_timeup = p.timeup;
        // gear change: a short mechanical chirp; take-off whoosh and landing thud; pad whoosh; animal thump
        let (g0, a0, pd0, an0) = self.last2;
        if p.gear != g0 && p.gear > 0 && g0 > 0 && p.playing {
            let mut n = self.tone(3, 1, 0.0, 0.10, 0.06, 1800.0);
            n.hp = true;
            self.trigger(n);
        }
        if p.air && !a0 {
            let mut n = self.tone(3, 1, 0.0, 0.14, 0.45, 900.0);
            n.hp = true;
            self.trigger(n);
        }
        if !p.air && a0 && p.playing {
            let n = self.tone(3, 1, 0.0, 0.3, 0.2, 700.0);
            self.trigger(n);
            let mut k = self.tone(4, 1, 80.0, 0.45, 0.25, 0.0);
            k.slide = (0.5f32).powf(1.0 / (0.2 * self.sr));
            self.trigger(k);
        }
        if p.pads > pd0 {
            let mut v = self.tone(2, 1, 300.0, 0.13, 0.5, 0.0);
            v.slide = (4.0f32).powf(1.0 / (0.45 * self.sr));
            self.trigger(v);
            let mut n = self.tone(3, 1, 0.0, 0.12, 0.5, 2500.0);
            n.hp = true;
            self.trigger(n);
        }
        if p.animals > an0 {
            let mut k = self.tone(4, 1, 140.0, 0.5, 0.2, 0.0);
            k.slide = (0.4f32).powf(1.0 / (0.15 * self.sr));
            self.trigger(k);
            let v = self.tone(2, 1, 520.0, 0.1, 0.18, 0.0);
            self.trigger(v);
        }
        self.last2 = (p.gear, p.air, p.pads, p.animals);
    }

    fn music_step(&mut self, p: &AudioParams) {
        let b = (p.biome as usize).min(crate::NB - 1);
        let scale = SCALES[b];
        let sp = p.speed.clamp(0.0, 1.3);
        let st = self.step % 16;
        let degree = scale[self.prog[self.bar % 4] % scale.len()];
        let root = ROOTS[b] + degree;
        let style = music_style();
        let g = self.music_gain;
        if g < 0.02 || style >= 3 {
            self.step += 1;
            if st == 15 {
                self.bar += 1;
            }
            return;
        }
        let calm = style == 1;
        let drive = style == 2;
        // bass: syncopated 8ths with an octave pop
        if (!calm && [0, 3, 6, 8, 10, 12, 14].contains(&st))
            || (calm && st.is_multiple_of(8))
            || (drive && st.is_multiple_of(2))
        {
            let oct = if st == 6 || st == 14 { 12 } else { 0 };
            let v = self.tone(
                0,
                0,
                midi_hz((root + oct) as f32),
                0.20 * g,
                0.22,
                380.0 + sp * 500.0,
            );
            self.trigger(v);
        }
        // drums
        if (!calm && (st.is_multiple_of(4) || (st == 14 && sp > 0.8)))
            || (calm && st == 0)
            || (drive && st % 4 == 2)
        {
            let mut k = self.tone(4, 0, 120.0, 0.55 * g, 0.16, 0.0);
            k.slide = (0.35f32).powf(1.0 / (0.1 * self.sr));
            self.trigger(k);
        }
        if !calm && (st == 4 || st == 12) {
            let mut n = self.tone(3, 0, 0.0, 0.20 * g, 0.12, 3800.0);
            n.hp = true;
            self.trigger(n);
            let s = self.tone(2, 0, 190.0, 0.14 * g, 0.08, 0.0);
            self.trigger(s);
        }
        if !calm
            && (st.is_multiple_of(2)
                || (sp > 0.6
                    && st % 2 == 1
                    && hash(self.seed ^ (self.bar * 31 + st) as u32).is_multiple_of(3)))
        {
            let mut h = self.tone(3, 0, 0.0, 0.07 * g, 0.03, 7000.0);
            h.hp = true;
            self.trigger(h);
        }
        // lead: hash-driven arpeggio, denser when going fast
        let hh = hash(
            self.seed
                ^ (self.bar as u32).wrapping_mul(2654435761)
                ^ (st as u32).wrapping_mul(40503),
        );
        let prob = (0.30 + 0.45 * (sp / 1.3))
            * if calm {
                0.55
            } else if drive {
                1.15
            } else {
                1.0
            };
        if (hh & 255) as f32 / 255.0 < prob && (st.is_multiple_of(2) || hh & 0x300 == 0) {
            let idx = ((hh >> 8) as usize) % scale.len();
            let oct = [12, 24, 24, 36][((hh >> 16) & 3) as usize];
            let note = ROOTS[b] + scale[idx] + oct;
            let v = self.tone(1, 0, midi_hz(note as f32), 0.07 * g, 0.16, 3200.0);
            self.trigger(v);
        }
        // low-time warning
        if p.time_low && p.playing && st.is_multiple_of(4) {
            let v = self.tone(2, 1, 1568.0, 0.08, 0.05, 0.0);
            self.trigger(v);
        }
        if st == 15 {
            self.bar += 1;
        }
        self.step += 1;
    }

    fn noise(&mut self) -> f32 {
        self.rng.f() * 2.0 - 1.0
    }

    pub fn render(&mut self, out: &mut [f32], p: &AudioParams) {
        let mut clean = *p;
        clean.speed = if p.speed.is_finite() {
            p.speed.clamp(0.0, 1.5)
        } else {
            0.0
        };
        clean.daylight = if p.daylight.is_finite() {
            p.daylight.clamp(0.0, 1.0)
        } else {
            1.0
        };
        for v in [
            &mut clean.cop,
            &mut clean.train,
            &mut clean.ice,
            &mut clean.engine,
        ] {
            *v = if v.is_finite() {
                v.clamp(0.0, 1.0)
            } else {
                0.0
            };
        }
        let p = &clean;
        self.edges(p);
        let sr = self.sr;
        let sp = p.speed.clamp(0.0, 1.3);
        let bpm = 108.0 + sp * 34.0;
        let step_len = sr * 60.0 / (bpm * 4.0);
        let mg_target = if p.playing {
            1.0
        } else if p.finished {
            0.8
        } else {
            0.35
        };
        let gear = if sp < 0.2 {
            0
        } else if sp < 0.4 {
            1
        } else if sp < 0.6 {
            2
        } else if sp < 0.8 {
            3
        } else {
            4
        };
        let lo = [0.0, 0.2, 0.4, 0.6, 0.8][gear];
        let hi = [0.2, 0.4, 0.6, 0.8, 1.3][gear];
        let frac = ((sp - lo) / (hi - lo)).clamp(0.0, 1.0);
        let target_f = if p.speed < 0.01 {
            30.0
        } else {
            40.0 + frac * 72.0 + gear as f32 * 6.0
        };
        let cut = 260.0 + frac * 900.0 + sp * 600.0;
        let ec = (TAU * cut / sr).min(0.9);
        let eng_target = if p.playing || p.finished {
            0.10 + 0.10 * sp
        } else {
            0.03
        };
        for o in out.iter_mut() {
            self.music_gain += (mg_target - self.music_gain) * 0.00008;
            self.acc += 1.0;
            if self.acc >= step_len {
                self.acc -= step_len;
                self.music_step(p);
            }
            if !self.sched.is_empty() {
                while let Some(i) = self.sched.iter().position(|s| s.at <= self.t) {
                    let s = self.sched.swap_remove(i);
                    self.trigger(s.v);
                }
            }
            let (mut music, mut fx) = (0.0f32, 0.0f32);
            for vi in 0..self.voices.len() {
                if !self.voices[vi].on {
                    continue;
                }
                let n = if self.voices[vi].kind == 3 {
                    self.noise()
                } else {
                    0.0
                };
                let v = &mut self.voices[vi];
                v.freq *= v.slide;
                v.phase += v.freq / sr;
                if v.phase >= 1.0 {
                    v.phase -= 1.0;
                }
                let a = (TAU * v.cutoff / sr).min(0.95);
                let s = match v.kind {
                    0 => {
                        let raw = 2.0 * v.phase - 1.0;
                        v.lp += a * (raw - v.lp);
                        v.lp
                    }
                    1 => {
                        let raw = if v.phase < 0.28 { 1.0 } else { -1.0 };
                        v.lp += a * (raw - v.lp);
                        v.lp
                    }
                    3 => {
                        v.lp += a * (n - v.lp);
                        if v.hp {
                            n - v.lp
                        } else {
                            v.lp
                        }
                    }
                    _ => (TAU * v.phase).sin(),
                } * v.amp;
                v.amp *= v.decay;
                if v.amp < 0.0006 {
                    v.on = false;
                }
                if v.bus == 0 {
                    music += s;
                } else {
                    fx += s;
                }
            }
            // engine: two detuned saws through a speed-dependent low-pass, with gear steps
            self.e_f += (target_f - self.e_f) * 0.0015;
            self.eng_amp += (eng_target - self.eng_amp) * 0.0008;
            self.e_ph1 = (self.e_ph1 + self.e_f / sr).fract();
            self.e_ph2 = (self.e_ph2 + self.e_f * 1.504 / sr).fract();
            let health = if p.engine <= 0.0 {
                1.0
            } else {
                p.engine.min(1.0)
            };
            let rattle = if health < 0.6 {
                self.noise() * (0.6 - health) * 1.2
            } else {
                0.0
            };
            let raw = ((2.0 * self.e_ph1 - 1.0) + 0.6 * (2.0 * self.e_ph2 - 1.0) + rattle) * 0.6;
            self.e_lp += ec * (raw - self.e_lp);
            fx += self.e_lp * self.eng_amp;
            // wind and tyre noise
            let n = self.noise();
            self.wind_a += 0.12 * (n - self.wind_a);
            self.wind_b += 0.35 * (self.wind_a - self.wind_b);
            fx += self.wind_b * 0.55 * sp * sp;
            // off-road rumble
            self.off_amp += ((p.offroad && sp > 0.05) as i32 as f32 * 0.2 - self.off_amp) * 0.0008;
            self.rumble += 0.03 * (n - self.rumble);
            fx += self.rumble * self.off_amp * 3.0;
            // turbo hiss
            self.turbo_amp += ((p.turbo as i32 as f32) * 0.11 - self.turbo_amp) * 0.0012;
            self.turbo_lp += 0.25 * (n - self.turbo_lp);
            fx += (n - self.turbo_lp) * self.turbo_amp;

            // weather: rain / hail hiss, extra wind, thunder, hail ticks
            let w = (p.weather as usize).min(10);
            if RAIN[w] > 0.0 {
                self.rain_lp += 0.5 * (n - self.rain_lp);
                fx += (n - self.rain_lp) * RAIN[w] * 0.11;
            }
            if WINDY[w] > 0.0 {
                fx += self.wind_b * WINDY[w] * (0.25 + 0.5 * sp);
            }
            if w == 2 {
                self.thunder_t -= 1;
                if self.thunder_t <= 0 {
                    let mut th = self.tone(3, 1, 0.0, 0.5, 1.9, 160.0);
                    th.lp = 0.0;
                    self.trigger(th);
                    self.thunder_t = (sr * (6.0 + self.rng.f() * 9.0)) as i64;
                }
            }
            if w == 7 && self.rng.f() < 0.002 {
                let mut tick = self.tone(3, 1, 0.0, 0.18, 0.03, 4500.0);
                tick.hp = true;
                self.trigger(tick);
            }
            // ice carrying the car: creaks and cracks
            if p.ice > 0.05 && self.rng.f() < p.ice * 0.0025 {
                let f = 1800.0 + self.rng.f() * 3200.0;
                let mut c = self.tone(2, 1, f, 0.10 * p.ice, 0.04, 0.0);
                c.slide = 0.99985;
                self.trigger(c);
            }
            // police siren (two alternating tones) louder the closer the car
            if p.cop > 0.02 {
                let hi = (self.t as f32 / sr * 2.4).fract() < 0.5;
                self.siren_ph = (self.siren_ph + if hi { 960.0 } else { 720.0 } / sr).fract();
                let raw = if self.siren_ph < 0.35 { 1.0 } else { -1.0 };
                self.siren_lp += 0.25 * (raw - self.siren_lp);
                fx += self.siren_lp * 0.06 * p.cop;
            }
            // own siren of a police car, ambulance or fire engine
            if p.own_siren > 0 {
                let ts = self.t as f32 / sr;
                let f = match p.own_siren {
                    1 => {
                        if (ts * 2.4).fract() < 0.5 {
                            960.0
                        } else {
                            720.0
                        }
                    }
                    2 => {
                        if (ts * 1.6).fract() < 0.5 {
                            435.0
                        } else {
                            580.0
                        }
                    }
                    _ => 500.0 + 700.0 * (1.0 - ((ts * 0.9).fract() * 2.0 - 1.0).abs()),
                };
                self.own_ph = (self.own_ph + f / sr).fract();
                let raw = if self.own_ph < 0.4 { 1.0 } else { -1.0 };
                self.own_lp += 0.25 * (raw - self.own_lp);
                fx += self.own_lp * 0.05;
            }
            // level crossing: bell while it warns, horn while the train is on it
            if p.train > 0.05 {
                self.bell_t -= 1;
                if self.bell_t <= 0 {
                    let v = self.tone(2, 1, 1040.0, 0.11 * p.train, 0.4, 0.0);
                    self.trigger(v);
                    self.bell_t = (sr * 0.55) as i64;
                }
                if p.train_on {
                    self.horn1 = (self.horn1 + 196.0 / sr).fract();
                    self.horn2 = (self.horn2 + 247.0 / sr).fract();
                    let h = (2.0 * self.horn1 - 1.0) + 0.8 * (2.0 * self.horn2 - 1.0);
                    fx += h * 0.05 * p.train;
                }
            }
            // low fuel beeps
            if p.fuel_low && p.playing {
                self.beep_t -= 1;
                if self.beep_t <= 0 {
                    let v = self.tone(2, 1, 1320.0, 0.16, 0.12, 0.0);
                    self.trigger(v);
                    self.beep_t = sr as i64;
                }
            }
            // night, standing still: crickets
            if p.daylight < 0.3 && sp < 0.2 && p.playing {
                self.cricket_ph = (self.cricket_ph + 4300.0 / sr).fract();
                let gate = ((self.t as f32 / sr) * 7.0).fract() < 0.3;
                if gate {
                    fx += (TAU * self.cricket_ph).sin() * 0.012 * (1.0 - p.daylight / 0.3);
                }
            }
            // delay on the music bus
            let d = self.delay[self.di];
            self.delay[self.di] = music + d * 0.3;
            self.di = (self.di + 1) % self.delay.len();
            let mix = music + d * 0.22 + fx;
            *o = (mix * 0.95).tanh();
            self.t += 1;
        }
    }
}
