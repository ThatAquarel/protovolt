use embassy_rp::adc::{self, Adc};
use micromath::F32Ext;


#[allow(dead_code)]
mod temp_ntc {
    pub const BETA: f32 = 3435.0;
    pub const ADC_MAX: f32 = 4095.0;
    pub const R_FIXED: f32 = 10_000.0; // 10k to GND
    pub const R0: f32 = 10_000.0;      // 10k @ 25C
    pub const T0: f32 = 298.15;        // 25C in Kelvin
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TemperatureReading {
    pub ch_a: f32,
    pub ch_b: f32,
    pub mcu: f32,
}

pub trait Temperature<'a> {
    async fn read_temperature(&mut self) -> Result<TemperatureReading, ()>;
}

pub struct TemperatureDevice<'a> {
    adc: Adc<'a, adc::Async>,
    ch_a: adc::Channel<'a>,
    ch_b: adc::Channel<'a>,
    mcu: adc::Channel<'a>,
}

impl<'a> TemperatureDevice<'a> {
    pub fn new(
        adc: Adc<'a, adc::Async>,
        ch_a: adc::Channel<'a>,
        ch_b: adc::Channel<'a>,
        mcu: adc::Channel<'a>,
    ) -> Self {
        Self {
            adc: adc,
            ch_a: ch_a,
            ch_b: ch_b,
            mcu: mcu,
        }
    }
}

impl<'a> Temperature<'a> for TemperatureDevice<'a> {
    async fn read_temperature(&mut self) -> Result<TemperatureReading, ()> {
        let adc = &mut self.adc;

        let ch_a_level = adc.read(&mut self.ch_a).await.map_err(|_| ())?;
        let ch_b_level = adc.read(&mut self.ch_b).await.map_err(|_| ())?;
        let mcu_level = adc.read(&mut self.mcu).await.map_err(|_| ())?;

        Ok(TemperatureReading {
            ch_a: ch_convert_to_celsius(ch_a_level),
            ch_b: ch_convert_to_celsius(ch_b_level),
            mcu: mcu_convert_to_celsius(mcu_level)
        })
    }
}

// TODO: https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/adc.rs
fn mcu_convert_to_celsius(raw_temp: u16) -> f32 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    let temp = 27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721;
    let sign = if temp < 0.0 { -1.0 } else { 1.0 };
    let rounded_temp_x10: i16 = ((temp * 10.0) + 0.5 * sign) as i16;
    (rounded_temp_x10 as f32) / 10.0
}

pub fn ch_convert_to_celsius(raw: u16) -> f32 {
    use temp_ntc::*;

    let raw = raw.max(1) as f32;
    let v_ratio = raw / ADC_MAX;
    let r_ntc = R_FIXED * (1.0 / v_ratio - 1.0);
    let temp_k = 1.0 / ( (1.0 / T0) + (1.0 / BETA) * (r_ntc / R0).ln() );
    temp_k - 273.15
}
