//! Pursuit: driving the police car, a named fugitive flees along the stage. First catch up with him
//! (60 s), then ram him until his car gives up (60 s). Three crashes - with anything - let him get
//! away. Pure logic here; the game moves the fugitive and draws him.
pub const CATCH_TIME: f32 = 60.0;
pub const RAM_TIME: f32 = 60.0;
pub const MAX_CRASHES: u32 = 3;
pub const HITS_NEEDED: f32 = 0.25;

pub const NAMES: [&str; 8] = [
    "VIPER",
    "SLICK RICK",
    "NITRO NED",
    "LADY LUCK",
    "BIG TONY",
    "GHOST",
    "RAZOR",
    "DUKE",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Catch,
    Ram,
    Caught,
    Escaped,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pursuit {
    pub z: f32,
    pub x: f32,
    pub speed: f32,
    pub phase: Phase,
    /// seconds left in the current phase
    pub left: f32,
    /// remaining health of the fugitive's car, 1..0
    pub hp: f32,
    pub name: u8,
    hit_cd: f32,
    crashes0: u32,
    t: f32,
    seed: u32,
}

impl Pursuit {
    /// A fugitive `ahead` units in front of `player_z`.
    pub fn new(seed: u64, player_z: f32, ahead: f32, crashes: u32) -> Pursuit {
        let h = crate::career::splitmix(seed ^ 0xF00D_CAFE);
        Pursuit {
            z: player_z + ahead.max(500.0),
            x: 0.0,
            speed: 0.0,
            phase: Phase::Catch,
            left: CATCH_TIME,
            hp: 1.0,
            name: (h % NAMES.len() as u64) as u8,
            hit_cd: 0.0,
            crashes0: crashes,
            t: 0.0,
            seed: (h >> 16) as u32,
        }
    }

    pub fn name(&self) -> &'static str {
        NAMES[(self.name as usize).min(NAMES.len() - 1)]
    }

    pub fn over(&self) -> bool {
        matches!(self.phase, Phase::Caught | Phase::Escaped)
    }

    /// Advances the chase. `finish_z` is the end of the stage (he escapes if he reaches it).
    pub fn update(
        &mut self,
        dt: f32,
        player_z: f32,
        player_x: f32,
        player_speed: f32,
        crashes: u32,
        finish_z: f32,
    ) {
        if self.over() || !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let dt = dt.min(0.1);
        self.t += dt;
        self.hit_cd = (self.hit_cd - dt).max(0.0);
        let ph = (self.seed % 628) as f32 / 100.0;
        self.x = (0.62 * (self.t * 0.8 + ph).sin()).clamp(-0.85, 0.85);
        // he drives hard at first and slows when his car is hurt; he never outruns a fair chase
        let top = crate::game::MAX_SPEED;
        let mut want = match self.phase {
            Phase::Catch => 0.80,
            _ => 0.62 + 0.15 * self.hp,
        } * top;
        let gap = self.z - player_z;
        if gap > 9000.0 {
            want = want.min(player_speed * 0.9);
        }
        self.speed += (want - self.speed) * (dt * 1.5).min(1.0);
        self.z += self.speed * dt;
        if self.z < player_z + 200.0 {
            // the player drove past him: put him back just in front
            self.z = player_z + 300.0;
        }
        let gap = self.z - player_z;
        let dx = (self.x - player_x).abs();
        self.left -= dt;
        if crashes.saturating_sub(self.crashes0) >= MAX_CRASHES || self.z >= finish_z {
            self.phase = Phase::Escaped;
            return;
        }
        match self.phase {
            Phase::Catch => {
                if gap < 1200.0 && dx < 0.5 {
                    self.phase = Phase::Ram;
                    self.left = RAM_TIME;
                } else if self.left <= 0.0 {
                    self.phase = Phase::Escaped;
                }
            }
            Phase::Ram => {
                if gap < 600.0 && dx < 0.32 && self.hit_cd <= 0.0 && player_speed > 0.4 * self.speed
                {
                    self.hp = (self.hp - HITS_NEEDED).max(0.0);
                    self.hit_cd = 1.5;
                    // he swerves away from the hit
                    self.t += 0.9;
                    if self.hp <= 0.0 {
                        self.phase = Phase::Caught;
                    }
                } else if self.left <= 0.0 {
                    self.phase = Phase::Escaped;
                }
            }
            _ => {}
        }
    }
}
