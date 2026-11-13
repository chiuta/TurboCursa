//! Gamepad for the native build without any extra dependency: on Linux it reads the kernel joystick
//! interface (`/dev/input/js0`), elsewhere it is simply absent. The mapping is the one the browser
//! build uses (sticks / d-pad steer, RT or A gas, LT or B brake, X turbo, Start enter, Select back,
//! LB / RB gears, Y manual).
use crate::game::Input;

/// What the pad is asking for right now; the `*_pressed` fields are edges, set for one `poll`.
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct PadState {
    pub input: Input,
    pub enter: bool,
    pub back: bool,
    pub gear_up: bool,
    pub gear_down: bool,
    pub manual: bool,
}

/// Decoder of the 8-byte kernel events; separated from the file so it can be tested.
#[derive(Default)]
pub struct Decoder {
    axes: [i16; 8],
    btn: [bool; 16],
    edges: [bool; 16],
}

const DEAD: i16 = 9000;

impl Decoder {
    /// Feed one `struct js_event` (time u32, value i16, type u8, number u8).
    pub fn feed(&mut self, ev: &[u8; 8]) {
        let value = i16::from_le_bytes([ev[4], ev[5]]);
        let kind = ev[6] & 0x7F; // 0x80 marks the synthetic initial-state events
        let num = ev[7] as usize;
        match kind {
            1 if num < 16 => {
                let down = value != 0;
                if down && !self.btn[num] && ev[6] & 0x80 == 0 {
                    self.edges[num] = true;
                }
                self.btn[num] = down;
            }
            2 if num < 8 => self.axes[num] = value,
            _ => {}
        }
    }

    /// Current state; clears the one-shot edges.
    pub fn state(&mut self) -> PadState {
        let a = |i: usize| self.axes[i];
        // triggers rest at -32767 on most drivers and read 0 until first used, so only a clearly
        // positive value counts as pressed
        let trig = |v: i16| v > 8000;
        let steer = a(0).max(a(3)).max(0) > DEAD || a(6) > 0;
        let steer_l = a(0).min(a(3)).min(0) < -DEAD || a(6) < 0;
        let s = PadState {
            input: Input {
                left: steer_l,
                right: steer && !steer_l,
                accel: self.btn[0] || trig(a(5)) || a(1) < -DEAD || a(7) < 0,
                brake: self.btn[1] || trig(a(2)) || a(1) > DEAD || a(7) > 0,
                turbo: self.btn[2],
            },
            enter: self.edges[7],
            back: self.edges[6],
            gear_up: self.edges[5],
            gear_down: self.edges[4],
            manual: self.edges[3],
        };
        self.edges = [false; 16];
        s
    }
}

#[cfg(target_os = "linux")]
pub struct Gamepad {
    file: Option<std::fs::File>,
    dec: Decoder,
    retry: f32,
}

#[cfg(target_os = "linux")]
impl Default for Gamepad {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "linux")]
impl Gamepad {
    pub fn new() -> Gamepad {
        Gamepad {
            file: None,
            dec: Decoder::default(),
            retry: 0.0,
        }
    }

    pub fn connected(&self) -> bool {
        self.file.is_some()
    }

    /// Drain pending events and return the state; looks for a pad every two seconds while none is open.
    pub fn poll(&mut self, dt: f32) -> PadState {
        use std::io::Read;
        use std::os::unix::fs::OpenOptionsExt;
        if self.file.is_none() {
            self.retry -= dt;
            if self.retry <= 0.0 {
                self.retry = 2.0;
                self.file = (0..4).find_map(|i| {
                    std::fs::OpenOptions::new()
                        .read(true)
                        .custom_flags(0o4000) // O_NONBLOCK
                        .open(format!("/dev/input/js{i}"))
                        .ok()
                });
                self.dec = Decoder::default();
            }
        }
        let mut lost = false;
        if let Some(f) = self.file.as_mut() {
            let mut buf = [0u8; 8 * 32];
            loop {
                match f.read(&mut buf) {
                    Ok(0) => {
                        lost = true;
                        break;
                    }
                    Ok(n) => {
                        for ev in buf[..n - n % 8].as_chunks::<8>().0 {
                            self.dec.feed(ev);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => {
                        lost = true; // unplugged: the next poll looks for a pad again
                        break;
                    }
                }
            }
        }
        if lost {
            self.file = None;
        }
        self.dec.state()
    }
}

#[cfg(not(target_os = "linux"))]
pub struct Gamepad;

#[cfg(not(target_os = "linux"))]
impl Gamepad {
    pub fn new() -> Gamepad {
        Gamepad
    }
    pub fn connected(&self) -> bool {
        false
    }
    pub fn poll(&mut self, _dt: f32) -> PadState {
        PadState::default()
    }
}
