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
}

use tps55289::*;

use crate::hal::device::I2cDeviceWithAddr;
use crate::hal::event::Channel;

pub trait Converter {
    async fn init(&mut self) -> Result<(), ()>;

    fn enable(&mut self) -> Result<(), ()>;
    fn disable(&mut self) -> Result<(), ()>;

    fn set_voltage(&mut self, voltage: f32) -> Result<(), ()>;
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
            Channel::A => ADDR,
            Channel::B => ADDR + 1,
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

        Timer::after_millis(1).await; // Await controller start after EN/UVLO pulled high

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
        self.i2c.write(&[MODE, 0b1010_0000]).map_err(|_| ())
    }

    fn disable(&mut self) -> Result<(), ()> {
        self.i2c.write(&[MODE, 0b0011_0000]).map_err(|_| ())
    }

    fn set_voltage(&mut self, voltage: f32) -> Result<(), ()> {
        let reference = (voltage * 0.0564 - 0.045) / 0.0005645;
        let reference = reference as u16;

        self.i2c
            .write(&[REF_LSB, (reference & 0xFF) as u8, (reference >> 8) as u8])
            .map_err(|_| ())?;

        let lsb = self.i2c.read_reg_byte(REF_LSB).map_err(|_| ())?;
        let msb = self.i2c.read_reg_byte(REF_MSB).map_err(|_| ())?;
        info!("enabled with REF_MSB, REF_LSB: {:08b} {:08b}", msb, lsb);

        Ok(())
    }
}
