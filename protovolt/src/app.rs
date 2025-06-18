use crate::lib::event::HardwareEvent;

pub struct App {
    pub selected_channel: Channel,
    pub ch_a: ChannelState,
    pub ch_b: ChannelState,
}

pub struct ChannelState {
    pub enable: bool,
    pub v_set: f32,
    pub i_set: f32,
}

enum Channel {
    A,
    B
}

impl App {
    pub fn new() -> Self {
        Self { 
            selected_channel: Channel::A,
            ch_a: ChannelState { enable: false, v_set: 5.000, i_set: 1.00 },
            ch_b: ChannelState { enable: false, v_set: 3.300, i_set: 1.00 }
         }
    }

    pub fn handle_event(&mut self, event: HardwareEvent) {
        match event {
            HardwareEvent::ButtonUp => {
                // self.led_on = !self.led_on;
            }
            _ => {}
        }
    }
}
