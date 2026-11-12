use super::*;

impl Game {
    pub fn audio_params(&self) -> crate::audio::AudioParams {
        let w = self.biome_w;
        let mut biome = 0u8;
        for (i, &v) in w.iter().enumerate() {
            if v > w[biome as usize] {
                biome = i as u8;
            }
        }
        crate::audio::AudioParams {
            speed: self.speed / MAX_SPEED,
            offroad: self.x < -1.0 || self.x > 1.0,
            turbo: self.turbo_t > 0.0,
            biome,
            playing: self.state == State::Playing,
            finished: self.state == State::Finished,
            timeup: self.state == State::TimeUp,
            time_low: self.state == State::Playing && self.time < 10.0,
            crashes: self.crashes,
            cps: self.cps,
            turbos: self.turbos,
            weather: self.weather.index() as u8,
            daylight: self.daylight(),
            cop: {
                let pz = self.pos + PLAYER_Z;
                let near = self
                    .cops
                    .iter()
                    .map(|c| (1.0 - (c.z - pz).abs() / 9000.0).clamp(0.0, 1.0))
                    .fold(0.0f32, f32::max);
                near.max(self.bust)
            },
            train: {
                let pz = self.pos + PLAYER_Z;
                self.crossings
                    .iter()
                    .filter(|c| c.z - pz > -300.0)
                    .map(|c| (1.0 - (c.z - pz) / 12000.0).clamp(0.0, 1.0))
                    .fold(0.0f32, f32::max)
            },
            train_on: {
                let pz = self.pos + PLAYER_Z;
                self.crossings
                    .iter()
                    .any(|c| (c.z - pz).abs() < 6000.0 && c.train_on(self.elapsed))
            },
            ice: (self.ice_t / 1.5).clamp(0.0, 1.0),
            fuel_low: self.fuel_rule && self.mode != Mode::Zen && self.fuel < self.low_fuel_level(),
            engine: self.engine.max(0.001),
            air: self.air > 0.0,
            gear: self.gear,
            pads: self.pads,
            animals: self.animals_hit,
            own_siren: if self.privileged() && self.speed > 800.0 && self.state == State::Playing {
                match self.car.class {
                    crate::cars::VehicleClass::Police => 1,
                    crate::cars::VehicleClass::Ambulance => 2,
                    crate::cars::VehicleClass::Fire => 3,
                    _ => 0,
                }
            } else {
                0
            },
        }
    }

    pub fn set_view(&mut self, w: usize, h: usize) {
        self.vw = w;
        self.vh = h;
    }

    pub(super) fn k(&self) -> f32 {
        (self.vw as f32 / 320.0).min(self.vh as f32 / 200.0)
    }

    pub(super) fn player_screen(&self) -> (f32, f32) {
        let k = self.k();
        (
            self.vw as f32 / 2.0 + self.steer * 4.0 * k,
            self.vh as f32 - 10.0 * k,
        )
    }

    pub(super) fn burst_sparks(&mut self) {
        let k = self.k();
        let (cx, by) = self.player_screen();
        for _ in 0..28 {
            let a = self.fx_rng.range(0.1, 3.04);
            let s = self.fx_rng.range(60.0, 300.0) * k;
            let life = self.fx_rng.range(0.3, 0.7);
            self.particles.push(Particle {
                x: cx + self.fx_rng.range(-30.0, 30.0) * k,
                y: by - 14.0 * k,
                vx: a.cos() * s,
                vy: -a.sin() * s,
                life,
                max: life,
                kind: 1,
                size: self.fx_rng.range(1.0, 2.2) * k,
            });
        }
    }

    pub(super) fn spawn_fx(&mut self) {
        let k = self.k();
        let (cx, by) = self.player_screen();
        let sp = self.speed / MAX_SPEED;
        if (self.x < -1.0 || self.x > 1.0) && sp > 0.15 {
            for side in [-1.0f32, 1.0] {
                let life = self.fx_rng.range(0.6, 1.0);
                self.particles.push(Particle {
                    x: cx + side * 30.0 * k + self.fx_rng.range(-6.0, 6.0) * k,
                    y: by - 2.0 * k,
                    vx: self.fx_rng.range(-40.0, 40.0) * k + side * 25.0 * k,
                    vy: self.fx_rng.range(-80.0, -15.0) * k,
                    life,
                    max: life,
                    kind: 0,
                    size: self.fx_rng.range(4.0, 9.0) * k,
                });
            }
        }
        if self.turbo_t > 0.0 {
            for side in [-1.0f32, 1.0] {
                let life = self.fx_rng.range(0.2, 0.4);
                self.particles.push(Particle {
                    x: cx + side * 22.0 * k,
                    y: by - 14.0 * k,
                    vx: self.fx_rng.range(-25.0, 25.0) * k + side * 10.0 * k,
                    vy: self.fx_rng.range(40.0, 120.0) * k,
                    life,
                    max: life,
                    kind: 2,
                    size: self.fx_rng.range(3.0, 5.5) * k,
                });
            }
        }
        if self.particles.len() > 600 {
            self.particles.drain(0..100);
        }
    }

    pub(super) fn step_particles(&mut self, dt: f32) {
        let k = self.k();
        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            if p.kind == 1 {
                p.vy += 520.0 * k * dt;
            }
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }
}
