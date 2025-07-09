use core::cell::RefCell;

use embassy_embedded_hal::shared_bus::{I2cDeviceError, blocking::i2c::I2cDevice};
use embassy_sync::blocking_mutex::{Mutex, raw::RawMutex};
use embedded_hal::i2c::I2c;

pub struct I2cDeviceWithAddr<'a, M: RawMutex, BUS: I2c> {
    bus: I2cDevice<'a, M, BUS>,
    address: u8,
}

impl<'a, M: RawMutex, BUS: I2c> I2cDeviceWithAddr<'a, M, BUS> {
    pub fn new(mutex: &'a Mutex<M, RefCell<BUS>>, address: u8) -> Self {
        Self {
            bus: I2cDevice::new(mutex),
            address: address,
        }
    }

    pub fn read_reg_word(&mut self, reg: u8) -> Result<u16, I2cDeviceError<BUS::Error>> {
        let mut word = [0u8; 2];
        self.bus.write_read(self.address, &[reg], &mut word)?;
        Ok(u16::from_be_bytes(word))
    }

    pub fn read_reg_byte(&mut self, reg: u8) -> Result<u8, I2cDeviceError<BUS::Error>> {
        let mut byte = [0u8];
        self.bus.write_read(self.address, &[reg], &mut byte)?;
        Ok(u8::from_be_bytes(byte))
    }

    pub fn write(&mut self, bytes: &[u8]) -> Result<(), I2cDeviceError<BUS::Error>> {
        self.bus.write(self.address, bytes)
    }

    pub fn read(&mut self, read: &mut [u8]) -> Result<(), I2cDeviceError<BUS::Error>> {
        self.bus.read(self.address, read)
    }

    pub fn write_read(
        &mut self,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), I2cDeviceError<BUS::Error>> {
        self.bus.write_read(self.address, wr_buffer, rd_buffer)
    }
}
