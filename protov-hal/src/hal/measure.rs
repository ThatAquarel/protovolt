use core::cell::RefCell;

use defmt::*;
use embassy_sync::blocking_mutex::{Mutex, raw::RawMutex};
use embedded_hal::i2c::I2c;

use crate::config::HARDWARE_PROFILE;

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

const CURRENT_LSB: f32 = HARDWARE_PROFILE.ina226_current_lsb();
const POWER_LSB: f32 = HARDWARE_PROFILE.ina226_power_lsb();
const CAL: [u8; 2] = HARDWARE_PROFILE.ina226_cal_reg();

use ina226::*;

use crate::hal::{device::I2cDeviceWithAddr, event::Channel};

pub trait Measure {
    fn init(&mut self) -> Result<(), ()>;

    #[allow(dead_code)]
    fn read_shunt_voltage(&mut self) -> Result<f32, ()>;

    fn read_bus_voltage(&mut self) -> Result<f32, ()>;
    fn read_current(&mut self) -> Result<f32, ()>;
    fn read_power(&mut self) -> Result<f32, ()>;

    fn dump_registers<const N: usize>(&mut self, buf: &mut heapless::String<N>) -> Result<(), ()>;
}

pub struct MeasureDevice<'a, M: RawMutex, BUS: I2c> {
    i2c: I2cDeviceWithAddr<'a, M, BUS>,
}

impl<'a, M, BUS> MeasureDevice<'a, M, BUS>
where
    M: RawMutex,
    BUS: I2c + 'a,
{
    pub fn new(mutex: &'a Mutex<M, RefCell<BUS>>, channel: Channel) -> Self {
        let address = match channel {
            Channel::A => ADDR + 1,
            Channel::B => ADDR,
        };

        Self {
            i2c: I2cDeviceWithAddr::new(mutex, address),
        }
    }
}

impl<'a, M, BUS> Measure for MeasureDevice<'a, M, BUS>
where
    M: RawMutex,
    BUS: I2c + 'a,
{
    fn init(&mut self) -> Result<(), ()> {
        let mut manufacturer_id = [0u8; 2];
        self.i2c
            .write_read(&[MANUFACTURER_ID], &mut manufacturer_id)
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
            .write(&[CALIBRATION, CAL[0], CAL[1]])
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
    }

    fn read_power(&mut self) -> Result<f32, ()> {
        let reg = self.i2c.read_reg_word(POWER).map_err(|_| ())?;
        Ok((reg as f32) * POWER_LSB)
    }

    fn dump_registers<const N: usize>(&mut self, buf: &mut heapless::String<N>) -> Result<(), ()> {
        let addr = self.i2c.address();
        let _ = buf.push_str("INA226 @ 0x");
        push_hex_u8(buf, addr);
        let _ = buf.push_str(" (I2C1)|");

        let regs: [(u8, &str); 10] = [
            (CONFIG, "CONFIG"),
            (SHUNT_VOLTAGE, "SHUNT_V"),
            (BUS_VOLTAGE, "BUS_V"),
            (POWER, "POWER"),
            (CURRENT, "CURRENT"),
            (CALIBRATION, "CALIBRATION"),
            (ENABLE, "ENABLE"),
            (ALERT_LIMIT, "ALERT_LIMIT"),
            (MANUFACTURER_ID, "MANUF_ID"),
            (DIE_ID, "DIE_ID"),
        ];

        for (i, (reg, name)) in regs.iter().enumerate() {
            if i > 0 {
                let _ = buf.push('|');
            }
            let value = self.i2c.read_reg_word(*reg).map_err(|_| ())?;
            push_reg_line(buf, *reg, name, value);
        }

        Ok(())
    }
}

fn push_hex_u8<const N: usize>(buf: &mut heapless::String<N>, value: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let _ = buf.push(HEX[(value >> 4) as usize] as char);
    let _ = buf.push(HEX[(value & 0x0F) as usize] as char);
}

fn push_reg_line<const N: usize>(buf: &mut heapless::String<N>, reg: u8, name: &str, value: u16) {
    push_hex_u8(buf, reg);
    let _ = buf.push(' ');
    let _ = buf.push_str(name);
    let pad = name.len().max(12);
    for _ in name.len()..pad {
        let _ = buf.push(' ');
    }
    let _ = buf.push(' ');
    push_hex_u16(buf, value);
}

fn push_hex_u16<const N: usize>(buf: &mut heapless::String<N>, value: u16) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let _ = buf.push(HEX[((value >> 12) & 0xF) as usize] as char);
    let _ = buf.push(HEX[((value >> 8) & 0xF) as usize] as char);
    let _ = buf.push(HEX[((value >> 4) & 0xF) as usize] as char);
    let _ = buf.push(HEX[(value & 0xF) as usize] as char);
}
