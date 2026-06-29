use heapless::Vec;

use crate::config::FACTORY;
use crate::scpi::{
    ScpiChannel,
    colors::{self, Rgb},
};

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
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
}

pub struct ScpiState {
    pub remote: bool,
    ch1_color: Rgb,
    ch2_color: Rgb,
    pub lcd_brightness: u8,
    pub led_brightness: u8,
    pub prot_latched: [bool; 2],
    error_queue: Vec<(i32, heapless::String<64>), ERR_QUEUE_SIZE>,
    save_slots: [SlotSnapshot; SAVE_SLOTS],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SlotSnapshot {
    pub ch1: ChannelSnapshot,
    pub ch2: ChannelSnapshot,
    pub used: bool,
}

impl Default for ScpiState {
    fn default() -> Self {
        let appearance = FACTORY.appearance;
        Self {
            remote: true,
            ch1_color: appearance.ch1,
            ch2_color: appearance.ch2,
            lcd_brightness: appearance.lcd_brightness,
            led_brightness: appearance.led_brightness,
            prot_latched: [false; 2],
            error_queue: Vec::new(),
            save_slots: [SlotSnapshot::default(); SAVE_SLOTS],
        }
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

    pub fn color(&self, ch: ScpiChannel) -> Rgb {
        match ch {
            ScpiChannel::Ch1 => self.ch1_color,
            ScpiChannel::Ch2 => self.ch2_color,
        }
    }

    pub fn set_color(&mut self, ch: ScpiChannel, rgb: Rgb) {
        match ch {
            ScpiChannel::Ch1 => self.ch1_color = rgb,
            ScpiChannel::Ch2 => self.ch2_color = rgb,
        }
    }

    pub fn reset_appearance(&mut self) {
        let appearance = FACTORY.appearance;
        self.ch1_color = appearance.ch1;
        self.ch2_color = appearance.ch2;
        self.lcd_brightness = appearance.lcd_brightness;
        self.led_brightness = appearance.led_brightness;
    }

    pub fn color_for_hal(&self, ch: crate::model::Channel) -> Rgb {
        match ch {
            crate::model::Channel::A => self.ch1_color,
            crate::model::Channel::B => self.ch2_color,
        }
    }

    pub fn selected_rgb565(
        &self,
        ch: crate::model::Channel,
    ) -> embedded_graphics::pixelcolor::Rgb565 {
        colors::to_rgb565(self.color_for_hal(ch))
    }

    pub fn unselected_rgb565(
        &self,
        ch: crate::model::Channel,
    ) -> embedded_graphics::pixelcolor::Rgb565 {
        colors::to_rgb565(colors::darken(self.color_for_hal(ch)))
    }

    pub fn channel_led(&self, ch: crate::model::Channel) -> smart_leds::RGB8 {
        colors::scale_led(self.color_for_hal(ch), self.led_brightness)
    }

    pub fn white_led(&self) -> smart_leds::RGB8 {
        colors::scale_white(self.led_brightness)
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

    #[cfg(any(test, feature = "test-harness"))]
    pub fn clear_error_queue(&mut self) {
        self.error_queue.clear();
    }

    #[cfg(any(test, feature = "test-harness"))]
    pub fn seed_error(&mut self, code: i32, message: &str) {
        self.push_error(code, message);
    }
}

#[cfg(test)]
mod tests;
