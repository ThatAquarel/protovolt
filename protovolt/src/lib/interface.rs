use defmt::*;

use crate::lib::{event::HardwareEvent};
use embassy_rp::gpio::{AnyPin, Input, Level, Output, Pull};


pub mod matrix {
    pub const N_ROWS: usize = 3;
    pub const N_COLS: usize = 3;

    pub const N_BUTTONS: usize = N_ROWS * N_COLS;

    pub const DEBOUNCE_THRESHOLD: u16 = 2;
    pub const POLL_TIME_MS: u64 = 10;
}

pub struct ButtonInterface<'a> {
    rows: [Output<'a>; matrix::N_ROWS],
    cols: [Input<'a>; matrix::N_COLS],
    current_state: [bool; matrix::N_BUTTONS],
    debounce: [u16; matrix::N_BUTTONS],
}

impl ButtonInterface<'_> {
    pub fn new(row_pins: [AnyPin; 3], col_pins: [AnyPin; 3]) -> Self {
        let row = row_pins.map(|p| Output::new(p, Level::High));
        let col = col_pins.map(|p| Input::new(p, Pull::Up));

        Self {
            rows: row,
            cols: col,
            current_state: [false; matrix::N_BUTTONS],
            debounce: [0; matrix::N_BUTTONS],
        }
    }

    pub fn poll(&mut self) -> Option<HardwareEvent> {
        for (i, row) in self.rows.iter_mut().enumerate() {
            row.set_low();

            for (j, col) in self.cols.iter_mut().enumerate() {
                let k = i * matrix::N_COLS + j;
                let state: &mut u16 = &mut self.debounce[k];

                if col.is_low() {
                    *state += 1;
                } else {
                    *state = 0;
                    self.current_state[k] = false;
                }

                if *state < matrix::DEBOUNCE_THRESHOLD {
                    continue;
                }

                self.current_state[k] = true;
            }

            row.set_high();
        }

        // info!("states {:?}", self.current_state);
        for (i, state) in self.current_state.iter().enumerate() {
            if *state == true{
                // info!("pressed {} time {}", i, self.debounce[i]);
                return match i {
                    0 => Some(HardwareEvent::ButtonUp),
                    1 => Some(HardwareEvent::ButtonDown),
                    2 => Some(HardwareEvent::ButtonLeft),
                    3 => Some(HardwareEvent::ButtonRight),
                    4 => Some(HardwareEvent::ButtonEnter),
                    5 => Some(HardwareEvent::ButtonSwitch),
                    6 => Some(HardwareEvent::ButtonSettings),
                    7 => Some(HardwareEvent::ButtonChannelA),
                    8 => Some(HardwareEvent::ButtonChannelB),
                    _ => None
                };
            }
        }

        None
    }
}
