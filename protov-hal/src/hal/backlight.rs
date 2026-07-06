use embassy_rp::Peri;
use embassy_rp::peripherals;
use embassy_rp::pwm::{Config, Pwm};
use embedded_hal::pwm::SetDutyCycle;

pub struct Backlight<'d> {
    pwm: Pwm<'d>,
    /// Configured brightness (`SYST:LCD:BRIG`, factory default, etc.).
    level: u8,
}

impl<'d> Backlight<'d> {
    pub fn new(
        slice: Peri<'d, peripherals::PWM_SLICE0>,
        pin: Peri<'d, peripherals::PIN_16>,
    ) -> Self {
        let mut config = Config::default();
        config.top = 255;
        config.compare_a = 0;
        config.enable = true;

        Self {
            pwm: Pwm::new_output_a(slice, pin, config),
            level: 0,
        }
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn set_brightness(&mut self, level: u8) {
        self.level = level;
        self.apply(level);
    }

    /// Run `f` with the panel powered off, then restore the configured brightness.
    pub fn while_suppressed<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.apply(0);
        let result = f();
        self.apply(self.level);
        result
    }

    fn apply(&mut self, level: u8) {
        let _ = self.pwm.set_duty_cycle(level as u16);
    }
}
