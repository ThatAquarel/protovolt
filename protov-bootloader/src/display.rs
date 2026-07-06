//! Hold ST7789 panel state through reset and re-enable scan-out of existing GRAM.
//!
//! GPIO idle levels and SPI register values match `protov-hal/src/hal/display.rs`.

use embassy_rp::gpio::{Drive, Level, Output};
use embassy_rp::peripherals::SPI0;
use embassy_rp::spi::{self, Phase, Polarity, Spi};
use embassy_time::{Duration, block_for};

const SPI_FREQ: u32 = 10_000_000;
const SPI_PHASE: Phase = Phase::CaptureOnSecondTransition;
const SPI_POLARITY: Polarity = Polarity::IdleHigh;

/// MADCTL for `Rotation::Deg270`, default RGB order — same as mipidsi in the HAL.
const MADCTL_DEG270: u8 = 0xA0;
/// COLMOD RGB565 (16 bpp DBI/DPI).
const COLMOD_RGB565: u8 = 0x55;

const CMD_SLPOUT: u8 = 0x11;
const CMD_DISPON: u8 = 0x29;
const CMD_MADCTL: u8 = 0x36;
const CMD_INVON: u8 = 0x21;
const CMD_COLMOD: u8 = 0x3A;
const CMD_NORON: u8 = 0x13;

pub struct DisplayHold {
    _rst: Output<'static>,
    _cs: Output<'static>,
    _dc: Output<'static>,
    _backlight: Output<'static>,
    _spi: Spi<'static, SPI0, spi::Blocking>,
}

impl DisplayHold {
    pub fn wake(
        rst: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_28>,
        cs: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_17>,
        dc: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_21>,
        sck: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_18>,
        mosi: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_19>,
        backlight: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_16>,
        spi0: embassy_rp::Peri<'static, SPI0>,
    ) -> Self {
        // Release reset before touching the bus (active-low).
        let _rst = Output::new(rst, Level::High);
        block_for(Duration::from_millis(10));

        let mut _cs = Output::new(cs, Level::High);
        let mut _dc = Output::new(dc, Level::Low);

        let mut config = spi::Config::default();
        config.frequency = SPI_FREQ;
        config.phase = SPI_PHASE;
        config.polarity = SPI_POLARITY;
        let mut _spi = Spi::new_blocking_txonly(spi0, sck, mosi, config);

        wake_panel(&mut _spi, &mut _dc, &mut _cs);

        let mut _backlight = Output::new(backlight, Level::High);
        _backlight.set_drive_strength(Drive::_8mA);

        Self {
            _rst,
            _cs,
            _dc,
            _backlight,
            _spi,
        }
    }
}

fn wake_panel(
    spi: &mut Spi<'static, SPI0, spi::Blocking>,
    dc: &mut Output<'static>,
    cs: &mut Output<'static>,
) {
    write_command(spi, dc, cs, CMD_SLPOUT);
    // block_for(Duration::from_millis(120));

    write_command_data(spi, dc, cs, CMD_MADCTL, &[MADCTL_DEG270]);
    write_command(spi, dc, cs, CMD_INVON);
    write_command_data(spi, dc, cs, CMD_COLMOD, &[COLMOD_RGB565]);

    write_command(spi, dc, cs, CMD_NORON);
    // block_for(Duration::from_millis(10));

    write_command(spi, dc, cs, CMD_DISPON);
    // block_for(Duration::from_millis(20));
}

fn write_command(
    spi: &mut Spi<'static, SPI0, spi::Blocking>,
    dc: &mut Output<'static>,
    cs: &mut Output<'static>,
    cmd: u8,
) {
    dc.set_low();
    cs.set_low();
    let _ = spi.blocking_write(&[cmd]);
    cs.set_high();
}

fn write_command_data(
    spi: &mut Spi<'static, SPI0, spi::Blocking>,
    dc: &mut Output<'static>,
    cs: &mut Output<'static>,
    cmd: u8,
    data: &[u8],
) {
    dc.set_low();
    cs.set_low();
    let _ = spi.blocking_write(&[cmd]);
    dc.set_high();
    let _ = spi.blocking_write(data);
    cs.set_high();
}
