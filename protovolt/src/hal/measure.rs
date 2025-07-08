use core::cell::RefCell;

use defmt::*;
use embassy_sync::blocking_mutex::{raw::RawMutex, Mutex};
use embedded_hal::i2c::I2c;

#[allow(dead_code)]
mod ina226 {
    // A0, A1 --> GND: default addr
    pub const ADDR: u8 = 0x40;

    // REGISTERS
    pub const CONFIG: u8 = 0x00;

    pub const SHUNT_VOLTAGE: u8 = 0x01;
    pub const BUS_VOLTAGE: u8 = 0x02;
    pub const POWER: u8 = 0x03;
    pub const CURRENT: u8 = 0x04;

    pub const CALIBRATION: u8 = 0x05;
    pub const ENABLE: u8 = 0x06;
    pub const ALERT_LIMIT: u8 = 0x07;

    pub const MANUFACTURER_ID: u8 = 0xFE;
    pub const DIE_ID: u8 = 0xFF;
}

const R_SHUNT: f32 = 0.010; // 10mR
const I_MAX: f32 = 5.00; // 5A limit
// TODO: move all into hardware file

const CURRENT_LSB: f32 = I_MAX / ((1 << 15) as f32);
const POWER_LSB: f32 = CURRENT_LSB * 25.0;
const CAL: [u8; 2] = compute_cal(CURRENT_LSB, R_SHUNT);
const fn compute_cal(current_lsb: f32, shunt_resistance: f32) -> [u8; 2] {
    let cal = 0.00512 / (current_lsb * shunt_resistance);

    // TODO: figure out rounding or truncating?
    // also, uncertainty computation on either method. (currently truncating)
    (cal as u16).to_be_bytes()
}

use ina226::*;

use crate::hal::{device::I2cDeviceWithAddr, event::Channel};

pub trait Measure {
    fn init(&mut self) -> Result<(), ()>;
    fn read_shunt_voltage(&mut self) -> Result<f32, ()>;
    fn read_bus_voltage(&mut self) -> Result<f32, ()>;
    fn read_current(&mut self) -> Result<f32, ()>;
    fn read_power(&mut self) -> Result<f32, ()>;
}

pub struct MeasureDevice<'a, M: RawMutex, BUS: I2c> {
    i2c: I2cDeviceWithAddr<'a, M, BUS>
}

impl <'a, M, BUS> MeasureDevice<'a, M, BUS>
where 
    M: RawMutex,
    BUS: I2c + 'a,
{
    pub fn new(mutex: &'a Mutex<M, RefCell<BUS>>, channel: Channel) -> Self {
        let address = match channel {
            Channel::A => ADDR,
            Channel::B => ADDR + 1,
        };

        Self {
            i2c: I2cDeviceWithAddr::new(mutex, address)
        }
    }
}

impl<'a, M, BUS> Measure for MeasureDevice<'a, M, BUS>
where 
    M: RawMutex,
    BUS: I2c + 'a,
{
    fn init(&mut self) -> Result<(), ()>{
        let mut manufacturer_id = [0u8; 2];
        self.i2c
            .write_read( &[MANUFACTURER_ID], &mut manufacturer_id)
            .map_err(|_| ())?;

        let id = u16::from_be_bytes(manufacturer_id);
        if id != 0x5449 {
            error!("Manufacturer ID mismatch: got 0x{:04X}", id);
            return Err(());
        }
        info!("verfied manufaturer id:  got 0x{:04X}", id);
        // TODO: verify DIE_ID also

        info!("cal 0 {}", CAL[0]);
        info!("cal 1 {}", CAL[1]);

        self.i2c
            .write( &[CALIBRATION, CAL[0], CAL[1]])
            .map_err(|_| ())?;

        // TODO: verify CAL is correctly written

        Ok(())
    }

    fn read_shunt_voltage(&mut self) -> Result<f32, ()> {
        let reg = self.i2c.read_reg_word(SHUNT_VOLTAGE).map_err(|_| ())?;
        Ok((reg as i16 as f32) * 2.5e-6)
    }

    fn read_bus_voltage(&mut self) -> Result<f32, ()> {
        let reg = self.i2c.read_reg_word(BUS_VOLTAGE).map_err(|_| ())?;
        Ok((reg as f32) * 1.25e-3)
    }

    fn read_current(&mut self) -> Result<f32, ()> {
        let reg = self.i2c.read_reg_word(CURRENT).map_err(|_| ())?;
        Ok((reg as i16 as f32) * CURRENT_LSB)
        // TODO: move 1.25mV LSB out into INA226 constants
    }

    fn read_power(&mut self) -> Result<f32, ()> {
        let reg = self.i2c.read_reg_word(POWER).map_err(|_| ())?;
        Ok((reg as f32) * POWER_LSB)
        // TODO: move 1.25mV LSB out into INA226 constants
    }
}
