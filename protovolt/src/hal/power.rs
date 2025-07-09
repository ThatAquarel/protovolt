use core::cell::RefCell;

use embassy_sync::blocking_mutex::{Mutex, raw::RawMutex};
use embedded_hal::i2c::I2c;

use crate::hal::device::I2cDeviceWithAddr;

mod stusb4500 {
    pub const ADDR: u8 = 0x28;

    // Additional con~stants from your C defines
    pub const DEFAULT: u8 = 0xFF;

    pub const FTP_CUST_PASSWORD_REG: u8 = 0x95;
    pub const FTP_CUST_PASSWORD: u8 = 0x47;

    pub const FTP_CTRL_0: u8 = 0x96;
    pub const FTP_CUST_PWR: u8 = 0x80;
    pub const FTP_CUST_RST_N: u8 = 0x40;
    pub const FTP_CUST_REQ: u8 = 0x10;
    pub const FTP_CUST_SECT: u8 = 0x07;

    pub const FTP_CTRL_1: u8 = 0x97;
    pub const FTP_CUST_SER: u8 = 0xF8;
    pub const FTP_CUST_OPCODE: u8 = 0x07;

    pub const RW_BUFFER: u8 = 0x53;
    pub const TX_HEADER_LOW: u8 = 0x51;
    pub const PD_COMMAND_CTRL: u8 = 0x1A;
    pub const DPM_PDO_NUMB: u8 = 0x70;

    pub const READ: u8 = 0x00;
    pub const WRITE_PL: u8 = 0x01;
    pub const WRITE_SER: u8 = 0x02;
    pub const ERASE_SECTOR: u8 = 0x05;
    pub const PROG_SECTOR: u8 = 0x06;
    pub const SOFT_PROG_SECTOR: u8 = 0x07;

    pub const SECTOR_0: u8 = 0x01;
    pub const SECTOR_1: u8 = 0x02;
    pub const SECTOR_2: u8 = 0x04;
    pub const SECTOR_3: u8 = 0x08;
    pub const SECTOR_4: u8 = 0x10;
}

pub trait PowerDelivery {
    /// Initialize device on given I2C address (default 0x28)
    fn begin(&mut self, device_address: u8) -> Result<(), ()>;

    /// Read the NVM memory from the device
    fn read(&mut self) -> Result<(), ()>;

    /// Write NVM settings to the device
    fn write(&mut self, default_vals: u8) -> Result<(), ()>;

    /// Get voltage for given PDO number (1 to 3)
    fn get_voltage(&mut self, pdo_numb: u8) -> Result<f32, ()>;

    /// Get current for given PDO number (1 to 3)
    fn get_current(&mut self, pdo_numb: u8) -> Result<f32, ()>;

    // /// Get over-voltage lockout (5-20%) for given PDO number
    fn get_upper_voltage_limit(&self, pdo_numb: u8) -> Result<u8, ()>;

    // /// Get under-voltage lockout (5-20%), PDO1 fixed at 3.3V
    fn get_lower_voltage_limit(&self, pdo_numb: u8) -> Result<u8, ()>;

    // /// Get global flexible current value common to all PDOs
    fn get_flex_current(&self) -> Result<f32, ()>;

    // /// Get number of sink PDOs configured
    fn get_pdo_number(&mut self) -> Result<u8, ()>;

    // /// Check if external power source is available and sufficient
    fn get_external_power(&self) -> Result<u8, ()>;

    // /// Check if sink supports data communication
    fn get_usb_comm_capable(&self) -> Result<u8, ()>;

    // /// Get POWER_OK_CFG parameter (0..3)
    fn get_config_ok_gpio(&self) -> Result<u8, ()>;

    // /// Get GPIO pin configuration (0..3)
    fn get_gpio_ctrl(&self) -> Result<u8, ()>;

    // /// Get POWER_ONLY_ABOVE_5V parameter (0 or 1)
    fn get_power_above_5v_only(&self) -> Result<u8, ()>;

    // /// Get REQ_SRC_CURRENT parameter (0 or 1)
    fn get_req_src_current(&self) -> Result<u8, ()>;

    // /// Set voltage for given PDO number
    fn set_voltage(&mut self, pdo_numb: u8, voltage: f32) -> Result<(), ()>;

    // /// Set current for given PDO number
    fn set_current(&mut self, pdo_numb: u8, current: f32) -> Result<(), ()>;

    // /// Set over-voltage lockout for given PDO number
    // fn set_upper_voltage_limit(&mut self, pdo_numb: u8, value: u8) -> Result<(), ()>;

    // /// Set under-voltage lockout for given PDO number
    // fn set_lower_voltage_limit(&mut self, pdo_numb: u8, value: u8) -> Result<(), ()>;

    // /// Set global flexible current value common to all PDOs
    // fn set_flex_current(&mut self, value: f32) -> Result<(), ()>;

    // /// Set number of sink PDOs
    fn set_pdo_number(&mut self, value: u8) -> Result<(), ()>;

    // /// Set external power parameter
    // fn set_external_power(&mut self, value: u8) -> Result<(), ()>;

    // /// Set USB_COMM_CAPABLE parameter
    // fn set_usb_comm_capable(&mut self, value: u8) -> Result<(), ()>;

    // /// Set POWER_OK_CFG parameter
    // fn set_config_ok_gpio(&mut self, value: u8) -> Result<(), ()>;

    // /// Set GPIO pin configuration
    // fn set_gpio_ctrl(&mut self, value: u8) -> Result<(), ()>;

    // /// Set POWER_ONLY_ABOVE_5V parameter
    // fn set_power_above_5v_only(&mut self, value: u8) -> Result<(), ()>;

    // /// Set REQ_SRC_CURRENT parameter
    // fn set_req_src_current(&mut self, value: u8) -> Result<(), ()>;

    // /// Perform a soft reset forcing re-negotiation
    // fn soft_reset(&mut self) -> Result<(), ()>;
}

use stusb4500::*;

pub struct PowerDeliveryDevice<'a, M: RawMutex, BUS>
where
    BUS: I2c + 'a,
{
    i2c: I2cDeviceWithAddr<'a, M, BUS>,

    // buffer to hold 5 sectors * 8 bytes each as in C++ example
    sector: [[u8; 8]; 5],

    // flag to avoid redundant reading
    read_sectors: bool,
}

impl<'a, M, BUS> PowerDeliveryDevice<'a, M, BUS>
where
    M: RawMutex,
    BUS: I2c + 'a,
{
    pub fn new(mutex: &'a Mutex<M, RefCell<BUS>>) -> Self {
        Self {
            i2c: I2cDeviceWithAddr::new(mutex, ADDR),
            sector: [[0u8; 8]; 5],
            read_sectors: false,
        }
    }

    // Helper: write to I2C register
    fn i2c_write(&mut self, reg: u8, data: &[u8]) -> Result<(), ()> {
        let mut buf = [0u8; 9];
        buf[0] = reg;
        buf[1..=data.len()].copy_from_slice(data);

        self.i2c.write(&buf[..=data.len()]).map_err(|_| ())?;
        // Small delay after write (like delay(1) in C++)
        cortex_m::asm::delay(8_000);
        Ok(())
    }

    // Helper: read from I2C register
    fn i2c_read(&mut self, reg: u8, buffer: &mut [u8]) -> Result<(), ()> {
        self.i2c.write(&[reg]).map_err(|_| ())?;
        self.i2c.read(buffer).map_err(|_| ())?;
        Ok(())
    }

    // Helper: read PDO (1-3)
    fn read_pdo(&mut self, pdo_numb: u8) -> Result<u32, ()> {
        let addr = 0x85 + (pdo_numb - 1) * 4;
        let mut buf = [0u8; 4];
        self.i2c_read(addr, &mut buf)?;
        let pdo = u32::from_le_bytes(buf);
        Ok(pdo)
    }

    // Helper: write PDO (1-3)
    fn write_pdo(&mut self, pdo_numb: u8, pdo_data: u32) -> Result<(), ()> {
        let addr = 0x85 + (pdo_numb - 1) * 4;
        let buf = pdo_data.to_le_bytes();
        self.i2c_write(addr, &buf)
    }

    // Enter Write Mode (like CUST_EnterWriteMode)
    fn enter_write_mode(&mut self, erased_sector: u8) -> Result<(), ()> {
        self.i2c_write(FTP_CUST_PASSWORD_REG, &[FTP_CUST_PASSWORD])?;
        self.i2c_write(RW_BUFFER, &[0])?;

        self.i2c_write(FTP_CTRL_0, &[0])?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N])?;

        let opcode = ((erased_sector << 3) & FTP_CUST_SER) | (WRITE_SER & FTP_CUST_OPCODE);
        self.i2c_write(FTP_CTRL_1, &[opcode])?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ])?;

        // wait for completion
        let mut buf = [0u8];
        loop {
            self.i2c_read(FTP_CTRL_0, &mut buf)?;
            if (buf[0] & FTP_CUST_REQ) == 0 {
                break;
            }
            cortex_m::asm::delay(4_000_000);
        }

        // soft program opcode
        self.i2c_write(FTP_CTRL_1, &[SOFT_PROG_SECTOR & FTP_CUST_OPCODE])?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ])?;

        loop {
            self.i2c_read(FTP_CTRL_0, &mut buf)?;
            if (buf[0] & FTP_CUST_REQ) == 0 {
                break;
            }
            cortex_m::asm::delay(4_000_000);
        }

        // erase sectors opcode
        self.i2c_write(FTP_CTRL_1, &[ERASE_SECTOR & FTP_CUST_OPCODE])?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ])?;

        loop {
            self.i2c_read(FTP_CTRL_0, &mut buf)?;
            if (buf[0] & FTP_CUST_REQ) == 0 {
                break;
            }
            cortex_m::asm::delay(4_000_000);
        }

        Ok(())
    }

    // Exit Test Mode (like CUST_ExitTestMode)
    fn exit_test_mode(&mut self) -> Result<(), ()> {
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_RST_N])?;
        self.i2c_write(FTP_CUST_PASSWORD_REG, &[0])?;
        Ok(())
    }

    // Write sector (like CUST_WriteSector)
    fn write_sector(&mut self, sector_num: u8, sector_data: &[u8]) -> Result<(), ()> {
        assert_eq!(sector_data.len(), 8);

        self.i2c_write(RW_BUFFER, sector_data)?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N])?;
        self.i2c_write(FTP_CTRL_1, &[WRITE_PL & FTP_CUST_OPCODE])?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ])?;

        let mut buf = [0u8];
        loop {
            self.i2c_read(FTP_CTRL_0, &mut buf)?;
            if (buf[0] & FTP_CUST_REQ) == 0 {
                break;
            }
            cortex_m::asm::delay(4_000_000);
        }

        self.i2c_write(FTP_CTRL_1, &[PROG_SECTOR & FTP_CUST_OPCODE])?;
        self.i2c_write(
            FTP_CTRL_0,
            &[(sector_num & FTP_CUST_SECT) | FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ],
        )?;

        loop {
            self.i2c_read(FTP_CTRL_0, &mut buf)?;
            if (buf[0] & FTP_CUST_REQ) == 0 {
                break;
            }
            cortex_m::asm::delay(4_000_000);
        }

        Ok(())
    }
}

impl<'a, M, BUS> PowerDelivery for PowerDeliveryDevice<'a, M, BUS>
where
    M: RawMutex,
    BUS: I2c + 'a,
{
    fn begin(&mut self, device_address: u8) -> Result<(), ()> {
        Ok(())
    }

    fn read(&mut self) -> Result<(), ()> {
        let mut buffer = [0u8; 1];
        self.read_sectors = true;

        // Enter Read Mode
        buffer[0] = FTP_CUST_PASSWORD; // Password 0x47
        self.i2c_write(FTP_CUST_PASSWORD_REG, &buffer)?;

        buffer[0] = 0;
        self.i2c_write(FTP_CTRL_0, &buffer)?;

        buffer[0] = FTP_CUST_PWR | FTP_CUST_RST_N;
        self.i2c_write(FTP_CTRL_0, &buffer)?;

        // Read 5 sectors
        for i in 0..5usize {
            buffer[0] = FTP_CUST_PWR | FTP_CUST_RST_N;
            self.i2c_write(FTP_CTRL_0, &buffer)?;

            buffer[0] = READ & FTP_CUST_OPCODE;
            self.i2c_write(FTP_CTRL_1, &buffer)?;

            buffer[0] = (i as u8 & FTP_CUST_SECT) | FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ;
            self.i2c_write(FTP_CTRL_0, &buffer)?;

            // Wait for execution
            loop {
                self.i2c_read(FTP_CTRL_0, &mut buffer)?;
                if (buffer[0] & FTP_CUST_REQ) == 0 {
                    break;
                }
                cortex_m::asm::delay(4_000_000);
            }

            let mut temp_sector = [0u8; 8];
            self.i2c_read(RW_BUFFER, &mut temp_sector)?;
            self.sector[i] = temp_sector;
        }

        // Exit test mode
        self.exit_test_mode()?;

        // Load values from sector buffer into internal PDO representation
        // (Assuming you have setVoltage and setCurrent implemented)

        // PDO number: bits 1 and 2 of sector[3][2]
        self.set_pdo_number((self.sector[3][2] & 0x06) >> 1)?;

        // PDO1 - fixed 5V, current decoded per logic
        self.set_voltage(1, 5.0)?;

        let current_value = (self.sector[3][2] & 0xF0) >> 4;
        if current_value == 0 {
            self.set_current(1, 0.0)?;
        } else if current_value < 11 {
            self.set_current(1, current_value as f32 * 0.25 + 0.25)?;
        } else {
            self.set_current(1, current_value as f32 * 0.50 - 2.50)?;
        }

        // PDO2 voltage: ((sector[4][1] << 2) + (sector[4][0] >> 6)) / 20.0
        let voltage_2 =
            (((self.sector[4][1] as u16) << 2) + ((self.sector[4][0] as u16) >> 6)) as f32 / 20.0;
        self.set_voltage(2, voltage_2)?;

        // PDO2 current
        let current_value = self.sector[3][4] & 0x0F;
        if current_value == 0 {
            self.set_current(2, 0.0)?;
        } else if current_value < 11 {
            self.set_current(2, current_value as f32 * 0.25 + 0.25)?;
        } else {
            self.set_current(2, current_value as f32 * 0.50 - 2.50)?;
        }

        // PDO3 voltage: (((sector[4][3] & 0x03) << 8) + sector[4][2]) / 20.0
        let voltage_3 =
            ((((self.sector[4][3] & 0x03) as u16) << 8) + self.sector[4][2] as u16) as f32 / 20.0;
        self.set_voltage(3, voltage_3)?;

        // PDO3 current
        let current_value = (self.sector[3][5] & 0xF0) >> 4;
        if current_value == 0 {
            self.set_current(3, 0.0)?;
        } else if current_value < 11 {
            self.set_current(3, current_value as f32 * 0.25 + 0.25)?;
        } else {
            self.set_current(3, current_value as f32 * 0.50 - 2.50)?;
        }

        Ok(())
    }

    fn write(&mut self, default_vals: u8) -> Result<(), ()> {
        if default_vals == 0 {
            let mut nvm_current = [0u8; 3];
            let mut voltage = [0f32; 3];
            let mut digital_voltage: u32;

            // Load current and voltage values from PDO registers
            for i in 0..3usize {
                let pdo_data = self.read_pdo((i + 1).try_into().unwrap())?;
                let mut current = (pdo_data & 0x3FF) as f32 * 0.01;
                if current > 5.0 {
                    current = 5.0; // constrain current
                }

                // Convert current to 4-bit NVM representation
                nvm_current[i] = if current < 0.5 {
                    0
                } else if current <= 3.0 {
                    (4.0 * current - 1.0) as u8
                } else {
                    (2.0 * current + 5.0) as u8
                };

                digital_voltage = ((pdo_data >> 10) & 0x3FF) as u32;
                voltage[i] = digital_voltage as f32 / 20.0;

                // Clamp voltage between 5 and 20V
                if voltage[i] < 5.0 {
                    voltage[i] = 5.0;
                } else if voltage[i] > 20.0 {
                    voltage[i] = 20.0;
                }
            }

            // Update sector buffer currents for PDO1-3
            self.sector[3][2] = (self.sector[3][2] & 0x0F) | (nvm_current[0] << 4);
            self.sector[3][4] = (self.sector[3][4] & 0xF0) | nvm_current[1];
            self.sector[3][5] = (self.sector[3][5] & 0x0F) | (nvm_current[2] << 4);

            // PDO1 voltage fixed at 5V, skip voltage update

            // PDO2 voltage (10-bit) split into sector bytes
            digital_voltage = (voltage[1] * 20.0) as u32;
            self.sector[4][0] =
                (self.sector[4][0] & 0x3F) | (((digital_voltage & 0x03) << 6) as u8);
            self.sector[4][1] = (digital_voltage >> 2) as u8;

            // PDO3 voltage (10-bit) split into sector bytes
            digital_voltage = (voltage[2] * 20.0) as u32;
            self.sector[4][2] = (digital_voltage & 0xFF) as u8;
            self.sector[4][3] = (self.sector[4][3] & 0xFC) | ((digital_voltage >> 8) as u8 & 0x03);

            // Read current PDO number from device register
            let mut buf = [0u8; 1];
            self.i2c_read(DPM_PDO_NUMB, &mut buf)?;

            // Load PDO number into sector 3, byte 2 bits 2:3
            self.sector[3][2] = (self.sector[3][2] & 0xF9) | ((buf[0] << 1) & 0x06);

            // Write all sectors back to device
            self.enter_write_mode(SECTOR_0 | SECTOR_1 | SECTOR_2 | SECTOR_3 | SECTOR_4)?;
            for sector_idx in 0..5 {
                // let sector_slice = &mut self.sector[sector_idx];

                let sector = self.sector[sector_idx];
                self.write_sector(sector_idx as u8, &sector)?;
            }
            self.exit_test_mode()?;
        } else {
            let default_sector: [[u8; 8]; 5] = [
                [0x00, 0x00, 0xB0, 0xAA, 0x00, 0x45, 0x00, 0x00],
                [0x10, 0x40, 0x9C, 0x1C, 0xFF, 0x01, 0x3C, 0xDF],
                [0x02, 0x40, 0x0F, 0x00, 0x32, 0x00, 0xFC, 0xF1],
                [0x00, 0x19, 0x56, 0xAF, 0xF5, 0x35, 0x5F, 0x00],
                [0x00, 0x4B, 0x90, 0x21, 0x43, 0x00, 0x40, 0xFB],
            ];

            self.enter_write_mode(SECTOR_0 | SECTOR_1 | SECTOR_2 | SECTOR_3 | SECTOR_4)?;
            for sector_idx in 0..5 {
                self.write_sector(sector_idx as u8, &default_sector[sector_idx])?;
            }
            self.exit_test_mode()?;
        }
        Ok(())
    }

    fn get_voltage(&mut self, pdo_numb: u8) -> Result<f32, ()> {
        let pdo_data = self.read_pdo(pdo_numb)?;
        let voltage_bits = (pdo_data >> 10) & 0x3FF;
        Ok((voltage_bits as f32) / 20.0)
    }

    fn get_current(&mut self, pdo_numb: u8) -> Result<f32, ()> {
        let pdo_data = self.read_pdo(pdo_numb)?;
        let current_bits = pdo_data & 0x3FF;
        Ok((current_bits as f32) * 0.01)
    }

    fn get_lower_voltage_limit(&self, pdo_numb: u8) -> Result<u8, ()> {
        match pdo_numb {
            1 => Ok(0), // PDO1 is fixed at 3.3V, so no undervoltage control
            2 => Ok((self.sector[3][4] >> 4) + 5),
            3 => Ok((self.sector[3][6] & 0x0F) + 5),
            _ => Err(()),
        }
    }

    fn get_upper_voltage_limit(&self, pdo_numb: u8) -> Result<u8, ()> {
        match pdo_numb {
            1 => Ok((self.sector[3][3] >> 4) + 5),
            2 => Ok((self.sector[3][5] & 0x0F) + 5),
            3 => Ok((self.sector[3][6] >> 4) + 5),
            _ => Err(()),
        }
    }

        fn get_flex_current(&self) -> Result<f32, ()> {
        let digital_value: u16 =
            (((self.sector[4][4] & 0x0F) as u16) << 6) + (((self.sector[4][3] & 0xFC) as u16) >> 2);
        Ok(digital_value as f32 / 100.0)
    }

    fn get_pdo_number(&mut self) -> Result<u8, ()> {
        let mut buffer = [0u8];
        self.i2c_read(DPM_PDO_NUMB, &mut buffer)?;
        Ok(buffer[0] & 0x07)
    }

    fn get_external_power(&self) -> Result<u8, ()> {
        Ok((self.sector[3][2] & 0x08) >> 3)
    }

    fn get_usb_comm_capable(&self) -> Result<u8, ()> {
        Ok(self.sector[3][2] & 0x01)
    }

    fn get_config_ok_gpio(&self) -> Result<u8, ()> {
        Ok((self.sector[4][4] & 0x60) >> 5)
    }

    fn get_gpio_ctrl(&self) -> Result<u8, ()> {
        Ok((self.sector[1][0] & 0x30) >> 4)
    }

    fn get_power_above_5v_only(&self) -> Result<u8, ()> {
        Ok((self.sector[4][6] & 0x08) >> 3)
    }

    fn get_req_src_current(&self) -> Result<u8, ()> {
        Ok((self.sector[4][6] & 0x10) >> 4)
    }

    fn set_voltage(&mut self, mut pdo_numb: u8, mut voltage: f32) -> Result<(), ()> {
        if pdo_numb < 1 {
            pdo_numb = 1;
        } else if pdo_numb > 3 {
            pdo_numb = 3;
        }

        // Constrain voltage to 5-20V
        if voltage < 5.0 {
            voltage = 5.0;
        } else if voltage > 20.0 {
            voltage = 20.0;
        }

        // PDO1 voltage fixed at 5V
        if pdo_numb == 1 {
            voltage = 5.0;
        }

        // Convert voltage to 10-bit format (scale by 20)
        let voltage_bits = (voltage * 20.0) as u32;

        // Read current PDO value
        let mut pdo_data = self.read_pdo(pdo_numb)?;

        // Clear bits 10:19 (0xFFC00 mask)
        pdo_data &= !0xFFC00;

        // Set new voltage bits
        pdo_data |= voltage_bits << 10;

        // Write updated PDO value back
        self.write_pdo(pdo_numb, pdo_data)
    }

    fn set_current(&mut self, pdo_numb: u8, current: f32) -> Result<(), ()> {
        // Convert current from amps to the 10-bit integer representation
        let mut int_current = (current / 0.01) as u32;

        // Mask to 10 bits
        int_current &= 0x3FF;

        // Read current PDO value
        let mut pdo_data = self.read_pdo(pdo_numb)?;

        // Clear bits 0:9
        pdo_data &= !0x3FF;

        // Set new current bits
        pdo_data |= int_current;

        // Write updated PDO value back
        self.write_pdo(pdo_numb, pdo_data)
    }

    fn set_pdo_number(&mut self, mut value: u8) -> Result<(), ()> {
        if value > 3 {
            value = 3;
        }

        let buf = [value];
        self.i2c_write(DPM_PDO_NUMB, &buf)
    }
}
