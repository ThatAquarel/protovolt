use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::usb::Driver;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::UsbDevice;
use static_cell::StaticCell;

use crate::config::{MANUFACTURER, PRODUCT_NAME, SERIAL_NUMBER, USB_MAX_POWER_MA, USB_PID, USB_VID};
use crate::scpi::parser::{parse_command, ScpiCommand};
use crate::scpi::{LINE_BUF, SCPI_CMD, SCPI_RESP};

/// Bus power budget in the configuration descriptor (embassy-usb: milliamps).
/// ProtoV is PD-powered; keep this modest for the RP2040 USB PHY only.
pub const USB_ENUM_GRACE_MS: u64 = 250;

type MyDriver = Driver<'static, embassy_rp::peripherals::USB>;
type MyUsbDevice = UsbDevice<'static, MyDriver>;
type MyCdcClass = CdcAcmClass<'static, MyDriver>;

pub struct UsbCdcResources {
    pub class: &'static mut MyCdcClass,
    pub usb: MyUsbDevice,
}

pub fn build_usb_cdc(driver: MyDriver) -> UsbCdcResources {
    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    static STATE: StaticCell<State> = StaticCell::new();
    static CLASS: StaticCell<MyCdcClass> = StaticCell::new();

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

    UsbCdcResources { class, usb }
}

pub fn spawn_usb_tasks(spawner: &Spawner, resources: UsbCdcResources) {
    unwrap!(spawner.spawn(usb_task(resources.usb)));
    unwrap!(spawner.spawn(scpi_task(resources.class)));
}

#[embassy_executor::task]
async fn usb_task(mut usb: MyUsbDevice) -> ! {
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

#[embassy_executor::task]
async fn scpi_task(class: &'static mut MyCdcClass) -> ! {
    let mut line_buf = heapless::String::<LINE_BUF>::new();
    
    loop {
        class.wait_connection().await;
        
        info!("SCPI USB connected");
        line_buf.clear();

        loop {
            match read_scpi_session(class, &mut line_buf).await {
                Ok(()) => {}
                Err(Disconnected) => {
                    info!("SCPI USB disconnected");
                    break;
                }
            }
        }
    }
}

async fn read_scpi_session(
    class: &mut MyCdcClass,
    line_buf: &mut heapless::String<LINE_BUF>,
) -> Result<(), Disconnected> {
    let mut packet = [0u8; 64];
    let n = class.read_packet(&mut packet).await?;
    push_bytes(line_buf, &packet[..n]);

    while let Some(line) = take_complete_line(line_buf) {
        if let Some(cmd) = parse_command(&line) {
            SCPI_CMD.send(cmd).await;
        } else {
            continue;
        }
        let response = SCPI_RESP.receive().await;
        if let Some(text) = response.text {
            write_response(class, text.as_bytes()).await?;
        }
    }

    Ok(())
}

fn push_bytes(buf: &mut heapless::String<LINE_BUF>, data: &[u8]) {
    for &b in data {
        if b == b'\n' || b == b'\r' {
            let _ = buf.push('\n');
            continue;
        }
        if (b as char).is_ascii() {
            let _ = buf.push(b as char);
        }
    }
}

fn take_complete_line(buf: &mut heapless::String<LINE_BUF>) -> Option<heapless::String<LINE_BUF>> {
    let newline_pos = buf.as_str().find('\n')?;
    let mut line = heapless::String::<LINE_BUF>::new();
    let _ = line.push_str(buf.as_str()[..newline_pos].trim());
    let mut rest = heapless::String::<LINE_BUF>::new();
    let rest_start = newline_pos + 1;
    let _ = rest.push_str(buf.as_str()[rest_start..].trim_start());
    buf.clear();
    let _ = buf.push_str(rest.as_str());
    Some(line)
}

async fn write_response(class: &mut MyCdcClass, text: &[u8]) -> Result<(), Disconnected> {
    const MAX_PACKET: usize = 64;

    let mut offset = 0;
    while offset < text.len() {
        let end = (offset + MAX_PACKET).min(text.len());
        class.write_packet(&text[offset..end]).await?;
        offset = end;
    }

    class.write_packet(b"\n").await?;
    Ok(())
}
