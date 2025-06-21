use core::cell::{Ref, RefCell};

use display_interface_spi::SPIInterface;
// use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
// use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_rp::Peripheral;
use embassy_rp::gpio::Output;
use embassy_rp::spi::{self, Instance, Spi};
use embassy_rp::{
    gpio::{AnyPin, Level},
    spi::{ClkPin, MisoPin, MosiPin},
};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;
use mipidsi::models::ST7789;
use mipidsi::options::{Orientation, Rotation};
use mipidsi::{Builder, Display};

mod st7789 {
    use embassy_rp::spi;
    use mipidsi::options::{ColorInversion, Rotation};

    pub const SPI_FREQ: u32 = 64_00_000;
    pub const SPI_PHASE: spi::Phase = spi::Phase::CaptureOnSecondTransition;
    pub const SPI_POLARITY: spi::Polarity = spi::Polarity::IdleHigh;

    pub const WIDTH: u16 = 240;
    pub const HEIGHT: u16 = 320;
    pub const COLOR_INVERSION: ColorInversion = ColorInversion::Inverted;
    pub const ORIENTATION: Rotation = Rotation::Deg270;
}

pub struct DisplayInterface<'d, T: Instance> {
    spi_bus: &'d Mutex<NoopRawMutex, RefCell<Spi<'d, T, spi::Blocking>>>,
    pub target: Display<
        SPIInterface<
            SpiDeviceWithConfig<'d, NoopRawMutex, Spi<'d, T, spi::Blocking>, Output<'d>>,
            Output<'d>,
        >,
        ST7789,
        Output<'d>,
    >,
}

impl<'d, T> DisplayInterface<'d, T>
where
    T: Instance,
{
    pub fn new(
        spi_bus: &'d Mutex<NoopRawMutex, RefCell<Spi<'d, T, spi::Blocking>>>,
        cs: impl Peripheral<P = AnyPin> + 'd,
        rs: impl Peripheral<P = AnyPin> + 'd,
        rst: impl Peripheral<P = AnyPin> + 'd,
    ) -> Self {
        let cs = Output::new(cs, Level::High);
        let rs = Output::new(rs, Level::Low);
        let rst = Output::new(rst, Level::Low);

        let mut display_config = spi::Config::default();
        display_config.frequency = st7789::SPI_FREQ;
        display_config.phase = st7789::SPI_PHASE;
        display_config.polarity = st7789::SPI_POLARITY;

        let display_spi = SpiDeviceWithConfig::new(&spi_bus, cs, display_config);
        let display_interface = SPIInterface::new(display_spi, rs);

        let display = Builder::new(ST7789, display_interface)
            .display_size(st7789::WIDTH, st7789::HEIGHT)
            .reset_pin(rst)
            .invert_colors(st7789::COLOR_INVERSION)
            .orientation(Orientation::new().rotate(st7789::ORIENTATION))
            .init(&mut Delay)
            .unwrap();
        // TODO: error management for display
        // In reality, if the display fails, then there are bigger issues at hand

        Self {
            spi_bus: spi_bus,
            target: display,
        }
    }
}
