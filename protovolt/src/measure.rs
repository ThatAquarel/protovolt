use defmt::*;
use embedded_hal::i2c::I2c;


#[allow(dead_code)]
mod ina226 {
    // A0, A1 --> GND: default addr
    pub const ADDR: u8 = 0x40;

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


const R_SHUNT:f32 = 0.010;  // 10mR
const I_MAX: f32 = 5.00; // 5A limit
// TODO: move all into hardware file

const CURRENT_LSB: f32 = I_MAX / ((1 << 15) as f32);
const POWER_LSB: f32 = CURRENT_LSB * 25.0;
const CAL: [u8;2] = compute_cal(CURRENT_LSB, R_SHUNT);
const fn compute_cal(current_lsb: f32, shunt_resistance: f32) -> [u8;2] {
    let cal = 0.00512 / (current_lsb * shunt_resistance);

    // TODO: figure out rounding or truncating?
    // also, uncertainty computation on either method. (currently truncating)
    (cal as u16).to_be_bytes()
}

use ina226::*;

pub struct PowerMonitor<I2C: I2c> {
    i2c: I2C,
    addr: u8,
}

impl<I2C> PowerMonitor<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C, addr_offset: u8) -> Self {
        Self {
            i2c,
            addr: ADDR + addr_offset,
        }
    }

    pub fn init(&mut self) -> Result<(), ()> {
        let mut manufacturer_id = [0u8; 2];
        self.i2c
            .write_read(self.addr, &[MANUFACTURER_ID], &mut manufacturer_id)
            .map_err(|_| ())?;

        let id = u16::from_be_bytes(manufacturer_id);
        if id != 0x5449 {
            error!("Manufacturer ID mismatch: got 0x{:04X}", id);
            return Err(());
        }
        info!("verfied manufaturer id:  got 0x{:04X}", id);
        // TODO: verify DIE_ID also

        self.i2c.write(self.addr, &[CALIBRATION, CAL[0], CAL[1]]).unwrap();
        // TODO: verify CAL is correctly written

        Ok(())

    }

    fn read_word(&mut self, reg: u8) -> Result<u16, ()> {
        let mut word = [0u8; 2];
        self.i2c.write_read(self.addr, &[reg], &mut word).map_err(|_| ())?;

        Ok(u16::from_be_bytes(word))
    }

    pub fn read_shunt_voltage(&mut self) -> Result<f32, ()> {
        let reg = self.read_word(SHUNT_VOLTAGE)?;
        Ok((reg as f32) * 2.5e-6)
        // TODO: move 2.5uV LSB out into INA226 constants
    }

    pub fn read_bus_voltage(&mut self) -> Result<f32, ()> {
        let reg = self.read_word(BUS_VOLTAGE)?;
        Ok((reg as f32) * 1.25e-3)
        // TODO: move 1.25mV LSB out into INA226 constants
    }

    pub fn read_current(&mut self) -> Result<f32, ()> {
        let reg = self.read_word(BUS_VOLTAGE)?;
        Ok((reg as f32) * CURRENT_LSB)
        // TODO: move 1.25mV LSB out into INA226 constants
    }

    pub fn read_power(&mut self) -> Result<f32, ()> {
        let reg = self.read_word(BUS_VOLTAGE)?;
        Ok((reg as f32) * POWER_LSB)
        // TODO: move 1.25mV LSB out into INA226 constants
    }
}
