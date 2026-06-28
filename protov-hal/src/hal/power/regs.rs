//! STUSB4500 I2C register map (ST USB_PD_defines_STUSB-GEN1S.h).

pub const I2C_ADDR: u8 = 0x28;

pub const REG_DEVICE_ID: u8 = 0x2F;
pub const STUSB4500_ID: u8 = 0x25;

pub const ALERT_STATUS_1: u8 = 0x0B;
pub const ALERT_PRT_STATUS: u8 = 0x02;
pub const PORT_STATUS: u8 = 0x0E;
pub const TYPEC_MONITORING_STATUS_1: u8 = 0x10;
pub const CC_STATUS: u8 = 0x11;
pub const PRT_STATUS: u8 = 0x16;
pub const STUSB_GEN1S_CMD_CTRL: u8 = 0x1A;
pub const STUSB_GEN1S_RESET_CTRL: u8 = 0x23;
pub const PE_FSM_STATE: u8 = 0x29;

pub const RX_BYTE_CNT: u8 = 0x30;
pub const RX_HEADER: u8 = 0x31;
pub const RX_DATA_OBJ: u8 = 0x33;

pub const TX_HEADER: u8 = 0x51;
pub const DPM_PDO_NUMB: u8 = 0x70;

pub const DPM_SNK_PDO1: u8 = 0x85;
pub const DPM_REQ_RDO: u8 = 0x91;

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

pub const FTP_READ: u8 = 0x00;
pub const FTP_WRITE_PL: u8 = 0x01;
pub const FTP_WRITE_SER: u8 = 0x02;
pub const FTP_ERASE_SECTOR: u8 = 0x05;
pub const FTP_PROG_SECTOR: u8 = 0x06;
pub const FTP_SOFT_PROG_SECTOR: u8 = 0x07;

pub const SECTOR_0: u8 = 0x01;
pub const SECTOR_1: u8 = 0x02;
pub const SECTOR_2: u8 = 0x04;
pub const SECTOR_3: u8 = 0x08;
pub const SECTOR_4: u8 = 0x10;

pub const PORT_ATTACHED: u8 = 0x01;
pub const CC_CONNECT: u8 = 0x01;

pub const PE_SNK_READY: u8 = 0x18;

pub const PD_HEADER_SOFTRESET: u16 = 0x000D;
pub const PD_MSG_SOURCE_CAPABILITIES: u8 = 0x01;
pub const PD_CMD_SEND_MESSAGE: u8 = 0x26;

pub const PRL_MSG_RECEIVED: u8 = 0x04;

pub const TLOAD_MS: u64 = 30;
pub const ATTACH_TIMEOUT_MS: u64 = 150;
pub const NEGOTIATE_TIMEOUT_MS: u64 = 5000;
pub const SRC_CAP_POLL_MS: u64 = 500;
pub const SRC_CAP_BURST_MS: u32 = 500;
pub const SRC_CAP_BURST_US: u32 = 200;
