use crate::hal::event::{Change, Channel, InterfaceEvent};
use embassy_rp::gpio::{AnyPin, Input, Level, Output, Pull};
use embassy_time::Duration;

pub mod matrix {
    pub const N_ROWS: usize = 3;
    pub const N_COLS: usize = 3;

    pub const N_BUTTONS: usize = N_ROWS * N_COLS;

    pub const DEBOUNCE_THRESHOLD: u16 = 3;
    pub const POLL_TIME_MS: u64 = 10;
}

pub struct ButtonsInterface<'a> {
    rows: [Output<'a>; matrix::N_ROWS],
    cols: [Input<'a>; matrix::N_COLS],
    current_state: [bool; matrix::N_BUTTONS],
    prev_state: [bool; matrix::N_BUTTONS],
    debounce: [u16; matrix::N_BUTTONS],
}

impl ButtonsInterface<'_> {
    pub fn new(row_pins: [AnyPin; 3], col_pins: [AnyPin; 3]) -> Self {
        let row = row_pins.map(|p| Output::new(p, Level::High));
        let col = col_pins.map(|p| Input::new(p, Pull::Up));

        Self {
            rows: row,
            cols: col,
            current_state: [false; matrix::N_BUTTONS],
            prev_state: [false; matrix::N_BUTTONS],
            debounce: [0; matrix::N_BUTTONS],
        }
    }

    pub fn poll(&mut self) -> Option<InterfaceEvent> {
        for (i, row) in self.rows.iter_mut().enumerate() {
            row.set_low();
            embassy_time::block_for(Duration::from_micros(5));
            // allievate weak internal pullups and line capacitance.

            for (j, col) in self.cols.iter_mut().enumerate() {
                let k = i * matrix::N_COLS + j;
                let debounce: &mut u16 = &mut self.debounce[k];

                if col.is_low() {
                    *debounce += 1;
                } else {
                    *debounce = 0;
                }

                self.current_state[k] = false;

                if *debounce < matrix::DEBOUNCE_THRESHOLD {
                    continue;
                }

                self.current_state[k] = true;
            }

            row.set_high();
        }

        let mut button_event = None;

        for (i, state) in self.current_state.iter().enumerate() {
            let change = if *state == true && self.prev_state[i] == false {
                Some(Change::Pressed)
            } else if *state == false && self.prev_state[i] == true {
                Some(Change::Released)
            } else {
                None
            };

            if let Some(change) = change {
                match change {
                    Change::Pressed => {defmt::info!("SW{} press", i+1);}
                    Change::Released=> {defmt::info!("SW{} release", i+1);}
                };

                button_event = match i {
                    0 => Some(InterfaceEvent::ButtonSettings(change)),
                    1 => Some(InterfaceEvent::ButtonSwitch(change)),
                    2 => Some(InterfaceEvent::ButtonEnter(change)),

                    3 => emit_on_press(change, InterfaceEvent::ButtonRight),
                    6 => emit_on_press(change, InterfaceEvent::ButtonLeft),

                    4 => emit_on_press(change, InterfaceEvent::ButtonUp),
                    5 => emit_on_press(change, InterfaceEvent::ButtonDown),

                    7 => emit_on_press(change, InterfaceEvent::ButtonChannel(Channel::B)),
                    8 => emit_on_press(change, InterfaceEvent::ButtonChannel(Channel::A)),
                    _ => None,
                };
            }
        }

        self.prev_state = self.current_state;

        button_event
    }
}

fn emit_on_press(change: Change, event: InterfaceEvent) -> Option<InterfaceEvent> {
    match change {
        Change::Pressed => Some(event),
        Change::Released => None,
    }
}
