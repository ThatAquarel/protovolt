use heapless::Vec;

use crate::scpi::ScpiChannel;

pub const ERR_QUEUE_SIZE: usize = 8;
pub const SAVE_SLOTS: usize = 9;

#[derive(Clone, Copy, Debug, Default)]
pub struct ChannelSnapshot {
    pub voltage_set: f32,
    pub current_set: f32,
    pub ovp: f32,
    pub ocp: f32,
    pub output_on: bool,
    pub prot_latched: bool,
    pub color: [u8; 8],
    pub color_len: u8,
}

impl ChannelSnapshot {
    pub fn color_str(&self) -> &str {
        core::str::from_utf8(&self.color[..self.color_len as usize]).unwrap_or("RED")
    }

    pub fn set_color(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = bytes.len().min(self.color.len());
        self.color[..len].copy_from_slice(&bytes[..len]);
        self.color_len = len as u8;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SlotSnapshot {
    pub ch1: ChannelSnapshot,
    pub ch2: ChannelSnapshot,
    pub used: bool,
}

impl Default for SlotSnapshot {
    fn default() -> Self {
        Self {
            ch1: ChannelSnapshot::default(),
            ch2: ChannelSnapshot::default(),
            used: false,
        }
    }
}

pub struct ScpiState {
    pub remote: bool,
    pub ch1_color: [u8; 8],
    pub ch1_color_len: u8,
    pub ch2_color: [u8; 8],
    pub ch2_color_len: u8,
    pub prot_latched: [bool; 2],
    error_queue: Vec<(i32, heapless::String<64>), ERR_QUEUE_SIZE>,
    save_slots: [SlotSnapshot; SAVE_SLOTS],
}

impl Default for ScpiState {
    fn default() -> Self {
        let s = Self {
            remote: true,
            ch1_color: *b"RED\0\0\0\0\0",
            ch1_color_len: 3,
            ch2_color: *b"BLUE\0\0\0\0",
            ch2_color_len: 4,
            prot_latched: [false; 2],
            error_queue: Vec::new(),
            save_slots: [SlotSnapshot::default(); SAVE_SLOTS],
        };
        s
    }
}

impl ScpiState {
    pub fn push_error(&mut self, code: i32, message: &str) {
        let mut msg = heapless::String::new();
        let _ = msg.push_str(message);
        let _ = self.error_queue.push((code, msg));
    }

    pub fn pop_error(&mut self) -> (i32, heapless::String<64>) {
        if !self.error_queue.is_empty() {
            self.error_queue.remove(0)
        } else {
            let mut msg = heapless::String::new();
            let _ = msg.push_str("No error");
            (0, msg)
        }
    }

    pub fn color(&self, ch: ScpiChannel) -> &str {
        match ch {
            ScpiChannel::Ch1 => core::str::from_utf8(&self.ch1_color[..self.ch1_color_len as usize])
                .unwrap_or("RED"),
            ScpiChannel::Ch2 => core::str::from_utf8(&self.ch2_color[..self.ch2_color_len as usize])
                .unwrap_or("BLUE"),
        }
    }

    pub fn set_color(&mut self, ch: ScpiChannel, name: &str) -> Result<(), ()> {
        if !is_valid_color(name) {
            return Err(());
        }
        match ch {
            ScpiChannel::Ch1 => {
                let len = name.len().min(8);
                self.ch1_color[..len].copy_from_slice(&name.as_bytes()[..len]);
                self.ch1_color_len = len as u8;
            }
            ScpiChannel::Ch2 => {
                let len = name.len().min(8);
                self.ch2_color[..len].copy_from_slice(&name.as_bytes()[..len]);
                self.ch2_color_len = len as u8;
            }
        }
        Ok(())
    }

    pub fn save_slot(&mut self, slot: u8, ch1: ChannelSnapshot, ch2: ChannelSnapshot) {
        let idx = slot as usize - 1;
        if idx < SAVE_SLOTS {
            self.save_slots[idx] = SlotSnapshot {
                ch1,
                ch2,
                used: true,
            };
        }
    }

    pub fn recall_slot(&self, slot: u8) -> Option<(ChannelSnapshot, ChannelSnapshot)> {
        let idx = slot as usize - 1;
        if idx < SAVE_SLOTS && self.save_slots[idx].used {
            Some((self.save_slots[idx].ch1, self.save_slots[idx].ch2))
        } else {
            None
        }
    }

    pub fn delete_slot(&mut self, slot: u8) {
        let idx = slot as usize - 1;
        if idx < SAVE_SLOTS {
            self.save_slots[idx] = SlotSnapshot::default();
        }
    }

    pub fn prot_latched(&self, ch: ScpiChannel) -> bool {
        match ch {
            ScpiChannel::Ch1 => self.prot_latched[0],
            ScpiChannel::Ch2 => self.prot_latched[1],
        }
    }

    pub fn set_prot_latched(&mut self, ch: Option<ScpiChannel>, latched: bool) {
        match ch {
            None => {
                self.prot_latched[0] = latched;
                self.prot_latched[1] = latched;
            }
            Some(ScpiChannel::Ch1) => self.prot_latched[0] = latched,
            Some(ScpiChannel::Ch2) => self.prot_latched[1] = latched,
        }
    }
}

pub fn is_valid_color(name: &str) -> bool {
    matches!(
        name,
        "RED" | "BLUE" | "YELLOW" | "GREEN" | "ORANGE" | "TEAL" | "VIOLET" | "PINK" | "CYAN"
            | "LIME" | "GRAY"
    )
}

pub fn normalize_color(raw: &str) -> Result<&'static str, ()> {
    let colors = [
        "RED", "BLUE", "YELLOW", "GREEN", "ORANGE", "TEAL", "VIOLET", "PINK", "CYAN", "LIME",
        "GRAY",
    ];
    for c in colors {
        if raw.eq_ignore_ascii_case(c) {
            return Ok(c);
        }
    }
    Err(())
}
