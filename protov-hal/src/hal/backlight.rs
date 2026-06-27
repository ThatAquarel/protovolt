use embassy_rp::peripherals;
use embassy_rp::pwm::{Config, Pwm};
use embedded_hal::pwm::SetDutyCycle;

pub struct Backlight<'d> {
    pwm: Pwm<'d>,
}

impl<'d> Backlight<'d> {
    pub fn new(slice: peripherals::PWM_SLICE0, pin: peripherals::PIN_16) -> Self {
        let mut config = Config::default();
        config.top = 255;
        config.compare_a = 255;
        config.enable = true;

        Self {
            pwm: Pwm::new_output_a(slice, pin, config),
        }
    }

    pub fn set_brightness(&mut self, level: u8) {
        let _ = self.pwm.set_duty_cycle(level as u16);
    }
}
