use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::usb::Driver;
use embassy_usb::UsbDevice;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use static_cell::StaticCell;

use protov_core::dfu::parse_definite_block_at;
use protov_core::scpi::parser::{self, FWUP_DATA_PREFIX, ScpiCommand};
use protov_core::scpi::LINE_BUF;
use protov_nvm::FWUP_MAX_BLOCK_LEN;

use crate::config::{
    MANUFACTURER, PRODUCT_NAME, SERIAL_NUMBER, USB_MAX_POWER_MA, USB_PID, USB_VID,
};
use crate::scpi::{set_serial_connected, SCPI_CMD, SCPI_RESP};

pub const USB_ENUM_GRACE_MS: u64 = 250;

type RpUsbDriver = Driver<'static, embassy_rp::peripherals::USB>;
type UsbDeviceStack = UsbDevice<'static, RpUsbDriver>;
type CdcSerialPort = CdcAcmClass<'static, RpUsbDriver>;

pub struct ScpiUsbStack {
    pub class: &'static mut CdcSerialPort,
    pub usb: UsbDeviceStack,
}

static mut FWUP_PAYLOAD: [u8; FWUP_MAX_BLOCK_LEN] = [0; FWUP_MAX_BLOCK_LEN];
static mut FWUP_PAYLOAD_LEN: usize = 0;

pub fn fwup_payload() -> &'static [u8] {
    unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(FWUP_PAYLOAD).cast::<u8>(),
            FWUP_PAYLOAD_LEN,
        )
    }
}

pub fn fwup_payload_len() -> usize {
    unsafe { FWUP_PAYLOAD_LEN }
}

pub fn build_usb_cdc(driver: RpUsbDriver) -> ScpiUsbStack {
    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    static STATE: StaticCell<State> = StaticCell::new();
    static CLASS: StaticCell<CdcSerialPort> = StaticCell::new();

    let mut config = embassy_usb::Config::new(USB_VID, USB_PID);
    config.manufacturer = Some(MANUFACTURER);
    config.product = Some(PRODUCT_NAME);
    config.serial_number = Some(SERIAL_NUMBER);
    config.max_power = USB_MAX_POWER_MA;
    config.max_packet_size_0 = 64;

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 128]),
    );

    let class = CdcAcmClass::new(&mut builder, STATE.init(State::new()), 64);
    let class = CLASS.init(class);
    let usb = builder.build();

    ScpiUsbStack { class, usb }
}

pub fn spawn_usb_tasks(spawner: &Spawner, resources: ScpiUsbStack) {
    spawner.spawn(unwrap!(usb_task(resources.usb)));
    spawner.spawn(unwrap!(scpi_task(resources.class)));
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDeviceStack) -> ! {
    usb.run().await
}

struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("USB buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

enum RxMode {
    Ascii,
    FwupPayload {
        payload_len: usize,
        received: usize,
    },
}

struct ScpiReader {
    buf: heapless::Vec<u8, LINE_BUF>,
    mode: RxMode,
}

impl ScpiReader {
    fn new() -> Self {
        Self {
            buf: heapless::Vec::new(),
            mode: RxMode::Ascii,
        }
    }

    fn push(&mut self, data: &[u8]) {
        for &b in data {
            let _ = self.buf.push(b);
        }
    }

    fn wants_more_usb(&self) -> bool {
        match self.mode {
            RxMode::FwupPayload { payload_len, received } => received < payload_len,
            RxMode::Ascii => {
                try_fwup_header(&self.buf).is_some() || fwup_data_pending(&self.buf)
            }
        }
    }

    async fn drain(&mut self, class: &mut CdcSerialPort) -> Result<(), Disconnected> {
        loop {
            match self.mode {
                RxMode::Ascii => {
                    // Detect FWUP binary blocks before newline splitting — payload bytes
                    // may contain 0x0A and must not be parsed as ASCII SCPI lines.
                    if let Some((header_end, payload_len)) = try_fwup_header(&self.buf) {
                        if payload_len > FWUP_MAX_BLOCK_LEN {
                            self.buf.clear();
                            break;
                        }
                        let inline = self.buf.len().saturating_sub(header_end);
                        let take = inline.min(payload_len);
                        if take > 0 {
                            unsafe {
                                FWUP_PAYLOAD[..take].copy_from_slice(
                                    &self.buf.as_slice()[header_end..header_end + take],
                                );
                            }
                        }
                        consume_front(&mut self.buf, header_end + take);
                        if take == payload_len {
                            self.finish_fwup_block(class, payload_len).await?;
                        } else {
                            self.mode = RxMode::FwupPayload {
                                payload_len,
                                received: take,
                            };
                        }
                        continue;
                    }

                    if fwup_data_pending(&self.buf) {
                        if self.buf.len() >= LINE_BUF {
                            self.buf.clear();
                        }
                        break;
                    }

                    if let Some(newline) = self.buf.iter().position(|&b| b == b'\n') {
                        let mut line_bytes = heapless::Vec::<u8, LINE_BUF>::new();
                        for &b in &self.buf[..=newline] {
                            let _ = line_bytes.push(b);
                        }
                        consume_front(&mut self.buf, newline + 1);
                        let line = trim_line(&line_bytes);
                        if !line.is_empty() {
                            self.dispatch_line(class, line).await?;
                        }
                        continue;
                    }

                    if self.buf.len() >= LINE_BUF {
                        self.buf.clear();
                    }
                    break;
                }
                RxMode::FwupPayload {
                    payload_len,
                    received,
                } => {
                    if self.buf.is_empty() {
                        break;
                    }
                    let need = payload_len - received;
                    let take = need.min(self.buf.len());
                    unsafe {
                        FWUP_PAYLOAD[received..received + take]
                            .copy_from_slice(&self.buf.as_slice()[..take]);
                    }
                    consume_front(&mut self.buf, take);
                    let received = received + take;
                    if received == payload_len {
                        self.mode = RxMode::Ascii;
                        self.finish_fwup_block(class, payload_len).await?;
                        continue;
                    }
                    self.mode = RxMode::FwupPayload {
                        payload_len,
                        received,
                    };
                    break;
                }
            }
        }
        Ok(())
    }

    async fn finish_fwup_block(
        &mut self,
        class: &mut CdcSerialPort,
        payload_len: usize,
    ) -> Result<(), Disconnected> {
        unsafe {
            FWUP_PAYLOAD_LEN = payload_len;
        }
        self.dispatch_cmd(class, ScpiCommand::FwupData).await
    }

    async fn dispatch_line(
        &mut self,
        class: &mut CdcSerialPort,
        line: &str,
    ) -> Result<(), Disconnected> {
        if let Some(cmd) = parser::parse_command(line) {
            if matches!(cmd, ScpiCommand::FwupData) {
                return Ok(());
            }
            self.dispatch_cmd(class, cmd).await?;
        }
        Ok(())
    }

    async fn dispatch_cmd(
        &mut self,
        class: &mut CdcSerialPort,
        cmd: ScpiCommand,
    ) -> Result<(), Disconnected> {
        SCPI_CMD.send(cmd).await;
        let response = SCPI_RESP.receive().await;
        match response.text {
            Some(text) => write_response(class, text.as_bytes()).await?,
            None => write_response(class, b"ERR\n").await?,
        }
        Ok(())
    }
}

#[embassy_executor::task]
async fn scpi_task(class: &'static mut CdcSerialPort) -> ! {
    let mut reader = ScpiReader::new();

    loop {
        class.wait_connection().await;
        set_serial_connected(true);

        info!("SCPI USB connected");
        reader.buf.clear();
        reader.mode = RxMode::Ascii;

        loop {
            match read_scpi_session(class, &mut reader).await {
                Ok(()) => {}
                Err(Disconnected) => {
                    set_serial_connected(false);
                    info!("SCPI USB disconnected");
                    break;
                }
            }
        }
    }
}

async fn read_scpi_session(
    class: &mut CdcSerialPort,
    reader: &mut ScpiReader,
) -> Result<(), Disconnected> {
    let mut packet = [0u8; 64];
    loop {
        let n = class.read_packet(&mut packet).await?;
        reader.push(&packet[..n]);
        reader.drain(class).await?;

        if reader.wants_more_usb() {
            continue;
        }
        break;
    }
    Ok(())
}

fn consume_front(buf: &mut heapless::Vec<u8, LINE_BUF>, n: usize) {
    let len = buf.len();
    if n >= len {
        buf.clear();
        return;
    }
    for i in 0..len - n {
        buf[i] = buf[i + n];
    }
    buf.truncate(len - n);
}

fn trim_line(line_bytes: &[u8]) -> &str {
    let end = line_bytes
        .iter()
        .position(|&b| b == b'\n' || b == b'\r')
        .unwrap_or(line_bytes.len());
    core::str::from_utf8(&line_bytes[..end])
        .ok()
        .map(str::trim)
        .unwrap_or("")
}

fn try_fwup_header(buf: &[u8]) -> Option<(usize, usize)> {
    let prefix = FWUP_DATA_PREFIX.as_bytes();
    if buf.len() < prefix.len() || !buf.starts_with(prefix) {
        return None;
    }
    let mut i = prefix.len();
    if buf.get(i) == Some(&b' ') {
        i += 1;
    }
    if buf.get(i) != Some(&b'#') {
        return None;
    }
    let (hdr_len, payload_len) = parse_definite_block_at(&buf[i..])?;
    Some((i + hdr_len, payload_len))
}

/// True when buf holds a partial or complete `SYST:FWUP:DATA` prefix (binary block incoming).
fn fwup_data_pending(buf: &[u8]) -> bool {
    let prefix = FWUP_DATA_PREFIX.as_bytes();
    if buf.is_empty() {
        return false;
    }
    if buf.len() < prefix.len() {
        return prefix.starts_with(buf);
    }
    buf.starts_with(prefix)
}

async fn write_response(class: &mut CdcSerialPort, text: &[u8]) -> Result<(), Disconnected> {
    const MAX_PACKET: usize = 64;

    let mut offset = 0;
    while offset < text.len() {
        let end = (offset + MAX_PACKET).min(text.len());
        class.write_packet(&text[offset..end]).await?;
        offset = end;
    }

    if !text.ends_with(b"\n") {
        class.write_packet(b"\n").await?;
    }
    Ok(())
}
