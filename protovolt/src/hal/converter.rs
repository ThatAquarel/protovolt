use defmt::*;
use embedded_hal::i2c::I2c;
use embedded_hal::digital::OutputPin;


#[allow(dead_code)]
mod lm51772 {
    // Addr/Slope --> GND: default addr
    pub const ADDR:u8 = 0x6A;

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


pub struct BuckBoostConverter<BUS: I2c, EN: OutputPin> {
    i2c: BUS,
    addr: u8,
    en: EN,
}

impl<BUS, EN> BuckBoostConverter<BUS, EN>
where
    BUS: I2c,
    EN: OutputPin
{
    // TODO: common abstract trait(?) for I2C drivers?
    pub fn new(i2c: BUS, addr_offset: u8, enable: EN) -> Self {
        Self {
            i2c,
            addr: ADDR + addr_offset,
            en: enable,
        }
    }

    // common trait for read_byte, read_word, etc.
    fn read_byte(&mut self, reg:u8) -> Result<u8, ()> {
        let mut byte= [0u8];
        self.i2c.write_read(self.addr, &[reg], &mut byte)
            .map_err(|_| ())?;

        Ok(u8::from_be_bytes(byte))
    }

    pub fn enable(&mut self)  -> Result<(), ()>{
        self.en.set_high().map_err(|_| ())
    }

    pub fn disable(&mut self)  -> Result<(), ()>{
        self.en.set_low().map_err(|_| ())
    }

    pub fn init(&mut self) -> Result<(), ()> {
        info!("disbling converter");
        
        // UVLO -> disable converter
        self.disable()?;
        info!("disabled");

        let status = self.read_byte(STATUS_BYTE).unwrap();
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

        self.i2c.write(self.addr, &[MFR_SPECIFIC_D0, d0]).unwrap();
        self.i2c.write(self.addr, &[MFR_SPECIFIC_D8, d8]).unwrap();
        self.i2c.write(self.addr, &[MFR_SPECIFIC_D9, d9]).unwrap();

        Ok(())
    }

    pub fn set_output_voltage(&mut self, voltage: f32, precise: bool) -> Result<(), ()>{
        let d8: u8 = self.read_byte(MFR_SPECIFIC_D8)?;
        let (d8, multiplier): (u8, u16) = if precise {
            (d8 & !0b1000_0000, (voltage / 0.010) as u16 + 1)
        } else {
            (d8 | 0b1000_0000, (voltage / 0.020) as u16 + 1)
        };

        let lsb = (multiplier & 0xFF) as u8;
        let msb = ((multiplier >> 8) & 0xFF) as u8;

        self.i2c.write(self.addr, &[MFR_SPECIFIC_D8, d8]).unwrap();
        self.i2c.write(self.addr, &[VOUT_TARGET1_LSB, lsb]).unwrap();
        self.i2c.write(self.addr, &[VOUT_TARGET1_MSB, msb]).unwrap();

        Ok(())
    }
}