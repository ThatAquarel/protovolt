use core::cell::RefCell;

use defmt::*;
use embassy_rp::gpio::{AnyPin, Level, Output};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::Timer;
use embedded_hal::i2c::I2c;

#[allow(dead_code)]
mod tps55289 {
    pub const ADDR: u8 = 0x74;

    // REGISTERS
    pub const REF_LSB: u8 = 0x00;
    pub const REF_MSB: u8 = 0x01;
    pub const IOUT_LIMIT: u8 = 0x02;
    pub const VOUT_SR: u8 = 0x03;
    pub const VOUT_FS: u8 = 0x04;
    pub const CDC: u8 = 0x05;
    pub const MODE: u8 = 0x06;
    pub const STATUS: u8 = 0x07;

    pub fn cc_discharge_time(v_i: u16, v_f: u16) -> u64 {
        // capacitance 590uF on output
        // + extra
        590 * (v_i as u64 - v_f as u64) / 100 + 20000
    }
}

use tps55289::*;

use crate::hal::device::I2cDeviceWithAddr;
use crate::hal::event::Channel;

pub trait Converter {
    async fn init(&mut self) -> Result<(), ()>;

    fn enable(&mut self) -> Result<(), ()>;
    fn disable(&mut self) -> Result<(), ()>;

    fn get_enabled(&mut self) -> Result<bool, ()>;
    fn get_voltage(&mut self) -> Result<u16, ()>;

    async fn set_voltage(&mut self, voltage: u16) -> Result<(), ()>;
    fn set_current(&mut self, current: u16) -> Result<(), ()>;
}

pub struct ConverterDevice<'a, M: RawMutex, BUS: I2c> {
    i2c: I2cDeviceWithAddr<'a, M, BUS>,
    en: Output<'a>,
}

impl<'a, M, BUS> ConverterDevice<'a, M, BUS>
where
    M: RawMutex,
    BUS: I2c + 'a,
{
    pub fn new(enable_pin: AnyPin, mutex: &'a Mutex<M, RefCell<BUS>>, channel: Channel) -> Self {
        let en = Output::new(enable_pin, Level::Low);

        let address = match channel {
            Channel::A => ADDR + 1,
            Channel::B => ADDR,
        };

        Self {
            i2c: I2cDeviceWithAddr::new(mutex, address),
            en: en,
        }
    }
}

impl<'a, M, BUS> Converter for ConverterDevice<'a, M, BUS>
where
    M: RawMutex,
    BUS: I2c + 'a,
{
    async fn init(&mut self) -> Result<(), ()> {
        self.en.set_high();

        Timer::after_millis(100).await; // Await controller start after EN/UVLO pulled high

        info!("start converter");

        let mut regs = [0u8; 8];
        self.i2c.write_read(&[REF_LSB], &mut regs).map_err(|_| {
            warn!("start converter, i2c read error");
            ()
        })?;

        info!("regs {}", regs);

        let (mode, status) = (
            regs[MODE as usize] & 0b1110_0010,
            regs[STATUS as usize] & 0b1110_0011,
        );
        if mode == 32 && status == 1 {
            info!("verified mode, status: 0b{:08b} 0b{:08b}", mode, status);
        } else {
            warn!("invalid mode, status");
            return Err(());
        }

        self.disable()?;

        Ok(())
    }

    fn enable(&mut self) -> Result<(), ()> {
        self.i2c.write(&[MODE, 0b1011_0000]).map_err(|_| ())
    }

    fn disable(&mut self) -> Result<(), ()> {
        // self.set_voltage(0)?;
        self.i2c.write(&[MODE, 0b0011_0000]).map_err(|_| ())
    }

    fn get_enabled(&mut self) -> Result<bool, ()> {
        let reg = self.i2c.read_reg_byte(MODE).map_err(|_| ())?;
        let enabled = reg & (1 << 7) != 0;

        Ok(enabled)
    }

    fn get_voltage(&mut self) -> Result<u16, ()> {
        let feedback_reg = self.i2c.read_reg_byte(VOUT_FS).map_err(|_| ())?;
        let feedback_divisor = match feedback_reg & 3 {
            0 => 625u32,
            1 => 1250u32,
            2 => 1875u32,
            _ => 2500u32,
        };

        let mut read = [0u8; 2];
        self.i2c.write_read(&[REF_LSB], &mut read).map_err(|_| ())?;
        let (lsb, msb) = (read[0], read[1]);

        let voltage_reg = (msb as u32) << 8 | lsb as u32;

        let vref = voltage_reg * 1129 / 2;
        let conv = (vref + 45_000) * feedback_divisor / 141_000;

        Ok(conv as u16)
    }

    /// Set TPS55289 output voltage
    /// * voltage: mV
    async fn set_voltage(&mut self, voltage: u16) -> Result<(), ()> {
        let (feedback_divisor, feedback_reg) = match voltage {
            200..=5000 => (625u32, 0u8),
            5001..=10000 => (1250u32, 1u8),
            10001..=15000 => (1875u32, 2u8),
            15001..=20000 => (2500u32, 3u8),
            _ => return Err(()),
        };

        let conv = voltage as u32 * 1000; // mV -> uV
        let vref = conv * 141 / feedback_divisor - 45_000;
        let reg = (vref * 2 / 1129) as u16;

        let (msb, lsb) = ((reg >> 8) as u8, reg as u8 & 0xFF);

        let was_enabled = self.get_enabled()?;
        let prev_voltage = self.get_voltage()?;

        self.disable()?;
        if was_enabled && (prev_voltage > voltage) {
            let us_delay = cc_discharge_time(prev_voltage, voltage);
            info!(
                "v initial {} mV, v final {} mV, discharge time {} us",
                prev_voltage, voltage, us_delay
            );
            Timer::after_micros(us_delay).await;
        }

        self.i2c.write(&[VOUT_FS, feedback_reg]).map_err(|_| ())?;
        self.i2c.write(&[REF_LSB, lsb, msb]).map_err(|_| ())?;

        if was_enabled {
            self.enable()?;
        };

        // let buf = ((msb as u16) << 8) | (lsb as u16);
        info!(
            "set voltage {} mV: MSB {:08b} LSB {:08b}",
            voltage, msb, lsb
        );

        Ok(())
    }

    /// Set TPS55289 output current limit
    /// * current: mA
    fn set_current(&mut self, current: u16) -> Result<(), ()> {
        let enable = 1u8 << 7;
        let current_limit_setting = (current / 50) as u8 & !enable;
        let reg = enable | current_limit_setting;

        self.i2c.write(&[IOUT_LIMIT, reg]).map_err(|_| ())?;

        info!("set current {} mA: REG {:08b}", current, reg);

        Ok(())
    }
}
