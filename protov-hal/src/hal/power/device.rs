use defmt::info;
use embassy_time::{Duration, Ticker};

use core::cell::RefCell;

use embassy_sync::blocking_mutex::{Mutex, raw::RawMutex};
use embedded_hal::i2c::I2c;

use crate::hal::device::I2cDeviceWithAddr;
use crate::hal::event::Limits;

use super::nvm_baseline::MANUFACTURING_SECTORS;
use super::pdo::{self, IndexedSourcePdo, SinkSlots};
use super::regs::*;

pub struct PowerDeliveryDevice<'a, M: RawMutex, BUS>
where
    BUS: I2c + 'a,
{
    pub(super) i2c: I2cDeviceWithAddr<'a, M, BUS>,
    pub(super) sector: [[u8; 8]; 5],
    pub(super) read_sectors: bool,
    source_caps: heapless::Vec<IndexedSourcePdo, 7>,
    saw_ps_rdy: bool,
}

impl<'a, M, BUS> PowerDeliveryDevice<'a, M, BUS>
where
    M: RawMutex,
    BUS: I2c + 'a,
{
    pub fn new(mutex: &'a Mutex<M, RefCell<BUS>>) -> Self {
        Self {
            i2c: I2cDeviceWithAddr::new(mutex, I2C_ADDR),
            sector: [[0u8; 8]; 5],
            read_sectors: false,
            source_caps: heapless::Vec::new(),
            saw_ps_rdy: false,
        }
    }

    pub fn i2c_write(&mut self, reg: u8, data: &[u8]) -> Result<(), ()> {
        let mut buf = [0u8; 9];
        buf[0] = reg;
        buf[1..=data.len()].copy_from_slice(data);
        self.i2c.write(&buf[..=data.len()]).map_err(|_| ())?;
        cortex_m::asm::delay(8_000);
        Ok(())
    }

    pub fn i2c_read(&mut self, reg: u8, buffer: &mut [u8]) -> Result<(), ()> {
        self.i2c.write(&[reg]).map_err(|_| ())?;
        self.i2c.read(buffer).map_err(|_| ())?;
        Ok(())
    }

    pub fn read_pdo_raw(&mut self, pdo_numb: u8) -> Result<u32, ()> {
        let addr = DPM_SNK_PDO1 + (pdo_numb - 1) * 4;
        let mut buf = [0u8; 4];
        self.i2c_read(addr, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn write_pdo_raw(&mut self, pdo_numb: u8, pdo_data: u32) -> Result<(), ()> {
        let addr = DPM_SNK_PDO1 + (pdo_numb - 1) * 4;
        self.i2c_write(addr, &pdo_data.to_le_bytes())
    }

    pub fn is_attached(&mut self) -> Result<bool, ()> {
        let mut buf = [0u8];
        self.i2c_read(PORT_STATUS, &mut buf)?;
        Ok(buf[0] & PORT_ATTACHED != 0)
    }

    pub fn cc_status(&mut self) -> Result<u8, ()> {
        let mut buf = [0u8];
        self.i2c_read(CC_STATUS, &mut buf)?;
        Ok(buf[0])
    }

    pub fn typec_current_at_5v(&mut self) -> f32 {
        let cc = self.cc_status().unwrap_or(0);
        let cc1 = cc & 0x03;
        let cc2 = (cc >> 2) & 0x03;
        let state = cc1.max(cc2);
        match state {
            3 => 3.0,
            2 => 1.5,
            _ => 0.5,
        }
    }

    pub fn pe_state(&mut self) -> Result<u8, ()> {
        let mut buf = [0u8];
        self.i2c_read(PE_FSM_STATE, &mut buf)?;
        Ok(buf[0])
    }

    pub fn vbus_ready(&mut self) -> Result<bool, ()> {
        let mut buf = [0u8];
        self.i2c_read(TYPEC_MONITORING_STATUS_1, &mut buf)?;
        Ok(buf[0] & 0x08 != 0)
    }

    pub fn has_nvm_cache(&self) -> bool {
        self.read_sectors
    }

    /// Compare runtime sink PDO registers against the manufacturing baseline (no FTP read).
    pub fn registers_need_baseline(&mut self) -> bool {
        let Ok(count) = self.get_pdo_number() else {
            return true;
        };
        if count != 3 {
            return true;
        }

        let checks = [(1, 5.0), (2, 15.0), (3, 20.0)];
        for (slot, expected_v) in checks {
            let Ok(v) = self.get_voltage(slot) else {
                return true;
            };
            if (v - expected_v).abs() > 0.15 {
                return true;
            }
        }

        false
    }

    pub fn read_active_contract(&mut self) -> Result<Option<Limits>, ()> {
        if self.pe_state()? != PE_SNK_READY {
            return Ok(None);
        }

        let (raw, limits, pos) = self.read_contract()?;
        if pos == 0 || limits.current <= 0.0 {
            return Ok(None);
        }

        info!(
            "[pd] active contract from RDO pos={} raw=0x{:08x}",
            pos, raw
        );
        Ok(Some(limits))
    }

    fn highest_sink_pdo_voltage(&mut self) -> f32 {
        let mut best = 5.0f32;
        for slot in 1..=3u8 {
            if let Ok(v) = self.get_voltage(slot) {
                if v > best {
                    best = v;
                }
            }
        }
        best
    }

    fn contract_voltage(&mut self, pos: u8) -> f32 {
        if let Some(v) = IndexedSourcePdo::voltage_for_rdo_pos(&self.source_caps, pos) {
            return v;
        }
        if self.pe_state().ok() == Some(PE_SNK_READY) {
            return self.highest_sink_pdo_voltage();
        }
        5.0
    }

    pub fn read_contract(&mut self) -> Result<(u32, Limits, u8), ()> {
        let mut buf = [0u8; 4];
        self.i2c_read(DPM_REQ_RDO, &mut buf)?;
        let raw = u32::from_le_bytes(buf);
        let (op_a, max_a, pos) = pdo::decode_rdo_currents(raw);
        let current = op_a.max(max_a);
        let voltage = self.contract_voltage(pos);

        Ok((raw, Limits { voltage, current }, pos))
    }

    pub fn read_rdo(&mut self) -> Result<(u32, f32, f32, u8), ()> {
        let (raw, limits, pos) = self.read_contract()?;
        Ok((raw, limits.voltage, limits.current, pos))
    }

    pub fn is_snk_ready(&mut self) -> bool {
        self.pe_state().ok() == Some(PE_SNK_READY)
    }

    pub fn saw_ps_rdy(&self) -> bool {
        self.saw_ps_rdy
    }

    fn note_control_message(&mut self, msg_type: u8) {
        if msg_type == PD_MSG_PS_RDY || msg_type == PD_MSG_ACCEPT {
            self.saw_ps_rdy = true;
        }
    }

    pub fn set_source_caps(&mut self, caps: &[IndexedSourcePdo]) {
        self.source_caps.clear();
        for cap in caps {
            let _ = self.source_caps.push(*cap);
        }
    }

    pub fn source_caps(&self) -> &[IndexedSourcePdo] {
        &self.source_caps
    }

    /// Tight spin-poll for source capabilities (STUSB4500 RX buffer ~3 ms lifetime).
    pub async fn burst_poll_source_capabilities(
        &mut self,
        ms: u32,
    ) -> heapless::Vec<IndexedSourcePdo, 7> {
        let iterations = ms * 1000 / SRC_CAP_BURST_US;
        let mut ticker = Ticker::every(Duration::from_micros(SRC_CAP_BURST_US as u64));

        for _ in 0..iterations {
            match self.try_read_source_capabilities() {
                Ok(Some(caps)) => return caps,
                Ok(None) => {}
                Err(()) => {}
            }
            ticker.next().await;
        }
        heapless::Vec::new()
    }

    fn parse_rx_header(header: [u8; 2]) -> (u8, u8) {
        let header_word = u16::from_le_bytes(header);
        let msg_type = (header_word & 0x1F) as u8;
        let num_obj = ((header_word >> 12) & 0x07) as u8;
        (msg_type, num_obj)
    }

    fn prt_message_pending(&mut self) -> Result<bool, ()> {
        let mut prt = [0u8];
        self.i2c_read(PRT_STATUS, &mut prt)?;
        Ok(prt[0] & PRL_MSG_RECEIVED != 0)
    }

    /// USB PD Soft Reset (not STUSB4500 SW_RESET register — that drops VBUS).
    pub fn pd_soft_reset(&mut self) -> Result<(), ()> {
        let mut port = [0u8];
        self.i2c_read(PORT_STATUS, &mut port)?;
        if port[0] & PORT_ATTACHED == 0 {
            return Err(());
        }

        let header = PD_HEADER_SOFTRESET.to_le_bytes();
        self.i2c_write(TX_HEADER, &header)?;
        self.i2c_write(STUSB_GEN1S_CMD_CTRL, &[PD_CMD_SEND_MESSAGE])
    }

    /// Read source capabilities from the RX buffer if a Source_Capabilities message is pending.
    pub fn try_read_source_capabilities(
        &mut self,
    ) -> Result<Option<heapless::Vec<IndexedSourcePdo, 7>>, ()> {
        if !self.prt_message_pending()? {
            return Ok(None);
        }

        let mut header = [0u8; 2];
        self.i2c_read(RX_HEADER, &mut header)?;
        let (msg_type, num_obj) = Self::parse_rx_header(header);
        info!("[pd] rx msg type={} objs={}", msg_type, num_obj);

        if num_obj == 0 {
            self.note_control_message(msg_type);
            return Ok(None);
        }

        let byte_count = num_obj as usize * 4;
        let mut rx_cnt = [0u8];
        self.i2c_read(RX_BYTE_CNT, &mut rx_cnt)?;

        let mut data = [0u8; 28];
        self.i2c_read(RX_DATA_OBJ, &mut data[..byte_count.min(28)])?;

        if msg_type != PD_MSG_SOURCE_CAPABILITIES {
            return Ok(None);
        }

        if rx_cnt[0] != 0 && rx_cnt[0] != byte_count as u8 {
            return Ok(None);
        }

        let mut caps = heapless::Vec::new();
        for i in 0..num_obj as usize {
            let offset = i * 4;
            let raw = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            if let Some(pdo) = pdo::decode_fixed_src_pdo(raw) {
                let _ = caps.push(IndexedSourcePdo {
                    index: (i + 1) as u8,
                    pdo,
                });
            }
        }

        if caps.is_empty() {
            Ok(None)
        } else {
            Ok(Some(caps))
        }
    }

    pub fn program_sink_slots(&mut self, slots: &SinkSlots) -> Result<(), ()> {
        self.set_pdo_number(3)?;
        self.set_voltage_current(1, slots.pdo1.voltage_v, slots.pdo1.current_a)?;
        self.set_voltage_current(2, slots.pdo2.voltage_v, slots.pdo2.current_a)?;
        self.set_voltage_current(3, slots.pdo3.voltage_v, slots.pdo3.current_a)?;
        Ok(())
    }

    pub fn set_voltage_current(
        &mut self,
        pdo_numb: u8,
        voltage: f32,
        current: f32,
    ) -> Result<(), ()> {
        let raw = pdo::encode_fixed_sink_pdo(if pdo_numb == 1 { 5.0 } else { voltage }, current);
        self.write_pdo_raw(pdo_numb, raw)
    }

    pub fn set_pdo_number(&mut self, mut value: u8) -> Result<(), ()> {
        if value > 3 {
            value = 3;
        }
        self.i2c_write(DPM_PDO_NUMB, &[value])
    }

    pub fn get_voltage(&mut self, pdo_numb: u8) -> Result<f32, ()> {
        let pdo_data = self.read_pdo_raw(pdo_numb)?;
        Ok(((pdo_data >> 10) & 0x3FF) as f32 / 20.0)
    }

    pub fn get_current(&mut self, pdo_numb: u8) -> Result<f32, ()> {
        let pdo_data = self.read_pdo_raw(pdo_numb)?;
        Ok((pdo_data & 0x3FF) as f32 * 0.01)
    }

    pub fn get_pdo_number(&mut self) -> Result<u8, ()> {
        let mut buffer = [0u8];
        self.i2c_read(DPM_PDO_NUMB, &mut buffer)?;
        Ok(buffer[0] & 0x07)
    }

    pub fn get_lower_voltage_limit(&self, pdo_numb: u8) -> Result<u8, ()> {
        match pdo_numb {
            1 => Ok(0),
            2 => Ok((self.sector[3][4] >> 4) + 5),
            3 => Ok((self.sector[3][6] & 0x0F) + 5),
            _ => Err(()),
        }
    }

    pub fn get_upper_voltage_limit(&self, pdo_numb: u8) -> Result<u8, ()> {
        match pdo_numb {
            1 => Ok((self.sector[3][3] >> 4) + 5),
            2 => Ok((self.sector[3][5] & 0x0F) + 5),
            3 => Ok((self.sector[3][6] >> 4) + 5),
            _ => Err(()),
        }
    }

    pub fn get_flex_current(&self) -> Result<f32, ()> {
        let digital_value: u16 =
            (((self.sector[4][4] & 0x0F) as u16) << 6) + (((self.sector[4][3] & 0xFC) as u16) >> 2);
        Ok(digital_value as f32 / 100.0)
    }

    pub fn get_external_power(&self) -> Result<u8, ()> {
        Ok((self.sector[3][2] & 0x08) >> 3)
    }

    pub fn get_usb_comm_capable(&self) -> Result<u8, ()> {
        Ok(self.sector[3][2] & 0x01)
    }

    pub fn get_config_ok_gpio(&self) -> Result<u8, ()> {
        Ok((self.sector[4][4] & 0x60) >> 5)
    }

    pub fn get_gpio_ctrl(&self) -> Result<u8, ()> {
        Ok((self.sector[1][0] & 0x30) >> 4)
    }

    pub fn get_power_above_5v_only(&self) -> Result<u8, ()> {
        Ok((self.sector[4][6] & 0x08) >> 3)
    }

    pub fn get_req_src_current(&self) -> Result<u8, ()> {
        Ok((self.sector[4][6] & 0x10) >> 4)
    }

    pub fn needs_baseline_nvm(&mut self) -> bool {
        self.registers_need_baseline()
    }

    pub fn program_manufacturing_baseline(&mut self) -> Result<(), ()> {
        info!("[pd] programming manufacturing NVM baseline");
        self.enter_write_mode(SECTOR_0 | SECTOR_1 | SECTOR_2 | SECTOR_3 | SECTOR_4)?;
        for (idx, sector) in MANUFACTURING_SECTORS.iter().enumerate() {
            self.write_sector(idx as u8, sector)?;
        }
        self.exit_test_mode()?;
        self.sector = MANUFACTURING_SECTORS;
        self.read_sectors = true;
        Ok(())
    }

    pub fn read_nvm(&mut self) -> Result<(), ()> {
        let mut buffer = [0u8; 1];
        self.read_sectors = true;

        buffer[0] = FTP_CUST_PASSWORD;
        self.i2c_write(FTP_CUST_PASSWORD_REG, &buffer)?;
        buffer[0] = 0;
        self.i2c_write(FTP_CTRL_0, &buffer)?;
        buffer[0] = FTP_CUST_PWR | FTP_CUST_RST_N;
        self.i2c_write(FTP_CTRL_0, &buffer)?;

        for i in 0..5usize {
            buffer[0] = FTP_CUST_PWR | FTP_CUST_RST_N;
            self.i2c_write(FTP_CTRL_0, &buffer)?;
            buffer[0] = FTP_READ & FTP_CUST_OPCODE;
            self.i2c_write(FTP_CTRL_1, &buffer)?;
            buffer[0] = (i as u8 & FTP_CUST_SECT) | FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ;
            self.i2c_write(FTP_CTRL_0, &buffer)?;

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

        self.exit_test_mode()?;
        self.sync_pdo_registers_from_nvm()
    }

    fn sync_pdo_registers_from_nvm(&mut self) -> Result<(), ()> {
        self.set_pdo_number((self.sector[3][2] & 0x06) >> 1)?;
        self.set_voltage_current(1, 5.0, decode_nvm_current((self.sector[3][2] & 0xF0) >> 4))?;

        let v2 =
            (((self.sector[4][1] as u16) << 2) + ((self.sector[4][0] as u16) >> 6)) as f32 / 20.0;
        self.set_voltage_current(2, v2, decode_nvm_current(self.sector[3][4] & 0x0F))?;

        let v3 =
            ((((self.sector[4][3] & 0x03) as u16) << 8) + self.sector[4][2] as u16) as f32 / 20.0;
        self.set_voltage_current(3, v3, decode_nvm_current((self.sector[3][5] & 0xF0) >> 4))?;
        Ok(())
    }

    fn enter_write_mode(&mut self, erased_sector: u8) -> Result<(), ()> {
        self.i2c_write(FTP_CUST_PASSWORD_REG, &[FTP_CUST_PASSWORD])?;
        self.i2c_write(RW_BUFFER, &[0])?;
        self.i2c_write(FTP_CTRL_0, &[0])?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N])?;

        let opcode = ((erased_sector << 3) & FTP_CUST_SER) | (FTP_WRITE_SER & FTP_CUST_OPCODE);
        self.i2c_write(FTP_CTRL_1, &[opcode])?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ])?;
        self.wait_ftp_request()?;

        self.i2c_write(FTP_CTRL_1, &[FTP_SOFT_PROG_SECTOR & FTP_CUST_OPCODE])?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ])?;
        self.wait_ftp_request()?;

        self.i2c_write(FTP_CTRL_1, &[FTP_ERASE_SECTOR & FTP_CUST_OPCODE])?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ])?;
        self.wait_ftp_request()?;
        Ok(())
    }

    fn exit_test_mode(&mut self) -> Result<(), ()> {
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_RST_N])?;
        self.i2c_write(FTP_CUST_PASSWORD_REG, &[0])?;
        Ok(())
    }

    fn write_sector(&mut self, sector_num: u8, sector_data: &[u8]) -> Result<(), ()> {
        assert_eq!(sector_data.len(), 8);
        self.i2c_write(RW_BUFFER, sector_data)?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N])?;
        self.i2c_write(FTP_CTRL_1, &[FTP_WRITE_PL & FTP_CUST_OPCODE])?;
        self.i2c_write(FTP_CTRL_0, &[FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ])?;
        self.wait_ftp_request()?;

        self.i2c_write(FTP_CTRL_1, &[FTP_PROG_SECTOR & FTP_CUST_OPCODE])?;
        self.i2c_write(
            FTP_CTRL_0,
            &[(sector_num & FTP_CUST_SECT) | FTP_CUST_PWR | FTP_CUST_RST_N | FTP_CUST_REQ],
        )?;
        self.wait_ftp_request()?;
        Ok(())
    }

    fn wait_ftp_request(&mut self) -> Result<(), ()> {
        let mut buf = [0u8];
        loop {
            self.i2c_read(FTP_CTRL_0, &mut buf)?;
            if (buf[0] & FTP_CUST_REQ) == 0 {
                break;
            }
            cortex_m::asm::delay(4_000_000);
        }
        Ok(())
    }

    pub fn limits_from_typec(&mut self) -> Limits {
        Limits {
            voltage: 5.0,
            current: self.typec_current_at_5v(),
        }
    }
}

fn decode_nvm_current(nibble: u8) -> f32 {
    if nibble == 0 {
        0.0
    } else if nibble < 11 {
        nibble as f32 * 0.25 + 0.25
    } else {
        nibble as f32 * 0.50 - 2.50
    }
}
