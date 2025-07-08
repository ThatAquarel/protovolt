use core::cell::RefCell;

use defmt::*;
use embassy_rp::gpio::{AnyPin, Level, Output};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embedded_hal::i2c::I2c;

#[allow(dead_code)]
mod lm51772 {
    // Addr/Slope --> GND: default addr
    pub const ADDR: u8 = 0x6A;

    // REGISTERS
    pub const CLEAR_FAULTS: u8 = 0x03;

    pub const ILIM_THRESHOLD: u8 = 0x0A;

    pub const VOUT_TARGET1_LSB: u8 = 0x0C;
    pub const VOUT_TARGET1_MSB: u8 = 0x0D;

    pub const USB_PD_STATUS_0: u8 = 0x21;
    pub const STATUS_BYTE: u8 = 0x78;
    pub const USB_PD_CONTROL_0: u8 = 0x81;

    pub const MFR_SPECIFIC_D0: u8 = 0xD0;
    pub const MFR_SPECIFIC_D1: u8 = 0xD1;
    pub const MFR_SPECIFIC_D2: u8 = 0xD2;
    pub const MFR_SPECIFIC_D3: u8 = 0xD3;
    pub const MFR_SPECIFIC_D4: u8 = 0xD4;
    pub const MFR_SPECIFIC_D5: u8 = 0xD5;
    pub const MFR_SPECIFIC_D6: u8 = 0xD6;
    pub const MFR_SPECIFIC_D7: u8 = 0xD7;
    pub const MFR_SPECIFIC_D8: u8 = 0xD8;
    pub const MFR_SPECIFIC_D9: u8 = 0xD9;

    pub const IVP_VOLTAGE: u8 = 0xDA;
}

use lm51772::*;

use crate::hal::device::I2cDeviceWithAddr;
use crate::hal::event::Channel;

pub trait Converter {
    fn init(&mut self) -> Result<(), ()>;

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
    fn init(&mut self) -> Result<(), ()> {
        info!("disbling converter");

        // UVLO -> disable converter
        self.disable()?;
        info!("disabled");

        let status = self.i2c.read_reg_byte(STATUS_BYTE).map_err(|_| ())?;

        // VOUT_OV fault, VIN_UV fault
        if status != 0x48 {
            error!("buck boost unresonsive: status got 0x{:04X}", status);
            return Err(());
        }
        info!("verified status 0x{:04X}", status);

        // D0, D9 reg config.
        let d0: u8 = 0b00110011;
        let d8: u8 = 0b00001100;
        let d9: u8 = 0b00000000;

        self.i2c.write(&[MFR_SPECIFIC_D0, d0]).map_err(|_| ())?;
        self.i2c.write(&[MFR_SPECIFIC_D8, d8]).map_err(|_| ())?;
        self.i2c.write(&[MFR_SPECIFIC_D9, d9]).map_err(|_| ())?;

        Ok(())
    }

    fn enable(&mut self) -> Result<(), ()> {
        self.en.set_high();
        Ok(())
    }

    fn disable(&mut self) -> Result<(), ()> {
        self.en.set_low();
        Ok(())
    }

    fn set_voltage(&mut self, voltage: f32) -> Result<(), ()> {
        let d8: u8 = self.i2c.read_reg_byte(MFR_SPECIFIC_D8).map_err(|_| ())?;

        let precise = false;

        let (d8, multiplier): (u8, u16) = if precise {
            (d8 & !0b1000_0000, (voltage / 0.010) as u16 + 1)
        } else {
            (d8 | 0b1000_0000, (voltage / 0.020) as u16 + 1)
        };

        let lsb = (multiplier & 0xFF) as u8;
        let msb = ((multiplier >> 8) & 0xFF) as u8;

        self.i2c.write(&[MFR_SPECIFIC_D8, d8]).map_err(|_| ())?;
        self.i2c.write(&[VOUT_TARGET1_LSB, lsb]).map_err(|_| ())?;
        self.i2c.write(&[VOUT_TARGET1_MSB, msb]).map_err(|_| ())?;

        Ok(())
    }
}
