//! Per-revision analog constants (converter trim, current sense shunt, etc.).
//!
//! Enable exactly one `hw-*` feature on `protov-core` / `protov-hal`. `HARDWARE_REVISION`
//! in [`crate::config::product`] tracks the selected profile.

#[cfg(feature = "hw-a0")]
mod a0;
#[cfg(feature = "hw-a1")]
mod a1;
#[cfg(feature = "hw-a2")]
mod a2;

#[derive(Clone, Copy, Debug)]
pub struct HardwareProfile {
    pub revision: &'static str,
    /// Added to TPS55289 feedback readback; subtracted on voltage set (mV).
    pub converter_voltage_offset_mv: u16,
    /// TPS55289 IOUT_LIMIT register LSB size (mA per count).
    pub converter_current_scale_ma: f32,
    /// INA226 shunt resistance (Ω).
    pub ina226_r_shunt_ohm: f32,
    /// INA226 calibration full-scale current (A).
    pub ina226_i_max_a: f32,
}

impl HardwareProfile {
    pub const fn ina226_current_lsb(self) -> f32 {
        self.ina226_i_max_a / ((1 << 15) as f32)
    }

    pub const fn ina226_power_lsb(self) -> f32 {
        self.ina226_current_lsb() * 25.0
    }

    pub const fn ina226_cal_reg(self) -> [u8; 2] {
        let cal = 0.00512 / (self.ina226_current_lsb() * self.ina226_r_shunt_ohm);
        (cal as u16).to_be_bytes()
    }
}

#[cfg(feature = "hw-a1")]
pub use a1::PROFILE;

#[cfg(feature = "hw-a2")]
pub use a2::PROFILE;

#[cfg(feature = "hw-a0")]
pub use a0::PROFILE;

#[cfg(any(
    all(feature = "hw-a0", feature = "hw-a1"),
    all(feature = "hw-a0", feature = "hw-a2"),
    all(feature = "hw-a1", feature = "hw-a2"),
    all(feature = "hw-a0", feature = "hw-a1", feature = "hw-a2"),
))]
compile_error!("enable only one hardware profile feature: hw-a0, hw-a1, or hw-a2");

#[cfg(not(any(feature = "hw-a0", feature = "hw-a1", feature = "hw-a2")))]
compile_error!("select a hardware profile feature: hw-a0, hw-a1, or hw-a2");

#[cfg(test)]
mod tests;
