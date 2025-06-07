//! This example shows how to use SPI (Serial Peripheral Interface) in the RP2040 chip.
//!
//! Example written for a display using the ST7789 chip. Possibly the Waveshare Pico-ResTouch
//! (https://www.waveshare.com/wiki/Pico-ResTouch-LCD-2.8)

#![no_std]
#![no_main]

mod measure;
mod buck_boost;


use panic_probe as _;

use core::cell::RefCell;

use defmt::*;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDeviceWithConfig;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
// use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{self, I2c};
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::{Delay, Timer};
use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
// use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;

// use embedded_hal::i2c::I2c;
use mipidsi::models::ST7789;
// use mipidsi::options::{Orientation, Rotation, ColorOrder};
use mipidsi::options::{Orientation, Rotation, ColorInversion};
use mipidsi::Builder;
// use display_interface::WriteOnlyDataCommand;
use {defmt_rtt as _, panic_probe as _};

use crate::measure::PowerMonitor;
use crate::buck_boost::BuckBoostConverter;

// use crate::touch::Touch;

// const DISPLAY_FREQ: u32 = 64_000_000;
const DISPLAY_FREQ: u32 = 64_000_000;
const TOUCH_FREQ: u32 = 200_000;

// type I2C1Bus = Mutex<NoopRawMutex, I2c<'static, I2C1, i2c::Async>>;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    // let bl = p.PIN_13;
    let rst = p.PIN_13;
    let display_cs = p.PIN_9;
    let dcx = p.PIN_8;
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let clk = p.PIN_10;
    // let touch_cs = p.PIN_16;
    //let touch_irq = p.PIN_17;

    // create SPI
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;
    let mut touch_config = spi::Config::default();
    touch_config.frequency = TOUCH_FREQ;
    touch_config.phase = spi::Phase::CaptureOnSecondTransition;
    touch_config.polarity = spi::Polarity::IdleHigh;

    let spi = Spi::new_blocking(p.SPI1, clk, mosi, miso, touch_config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(&spi_bus, Output::new(display_cs, Level::High), display_config);
    // let touch_spi = SpiDeviceWithConfig::new(&spi_bus, Output::new(touch_cs, Level::High), touch_config);

    // let mut touch = Touch::new(touch_spi);

    let dcx = Output::new(dcx, Level::Low);
    let rst = Output::new(rst, Level::Low);
    // dcx: 0 = command, 1 = data

    // Enable LCD backlight
    // let _bl = Output::new(bl, Level::High);

    // display interface abstraction from SPI and DC
    let di = SPIInterface::new(display_spi, dcx);

    // Define the display from the display interface and initialize it
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 320)
        .reset_pin(rst)
        // .color_order(ColorOrder::Bgr)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut Delay)
        .unwrap();

    // display.interface.write_command(0x3A).unwrap(); // COLMOD
    // display.interface.write_data(&[0x55]).unwrap(); // 16-bit/pixel (RGB565)

    display.clear(Rgb565::BLACK).unwrap();

    let raw_image_data = ImageRawLE::new(include_bytes!("../assets/ferris.raw"), 86);
    let ferris = Image::new(&raw_image_data, Point::new(34, 68));

    // Display the image
    ferris.draw(&mut display).unwrap();

    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
    Text::new(
        "Hello embedded_graphics \n + embassy + RP2040! aldskfjdslfjlj",
        Point::new(20, 200),
        style,
    )
    .draw(&mut display)
    .unwrap();

    // SETUP I2C

    let sda = p.PIN_14;
    let scl = p.PIN_15;

    let i2c_config = i2c::Config::default();
    let i2c = I2c::new_blocking(p.I2C1, scl, sda, i2c_config);
    let i2c_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(i2c));

    let measure_i2c = I2cDeviceWithConfig::new(&i2c_bus, i2c::Config::default());
    let buck_boost_i2c = I2cDeviceWithConfig::new(&i2c_bus, i2c::Config::default());

    let mut power_monitor = PowerMonitor::new(measure_i2c, 0u8);
    let enable = Output::new(p.PIN_0, Level::Low);
    let mut buck_boost = BuckBoostConverter::new(buck_boost_i2c, 0u8, enable);

    power_monitor.init().unwrap();
    buck_boost.init().unwrap();

    buck_boost.set_output_voltage(9.53, true).unwrap();
    buck_boost.enable().unwrap();
    
    loop {
        Timer::after_millis(500).await;

        let shunt_voltage = power_monitor.read_shunt_voltage().unwrap();
        let bus_voltage = power_monitor.read_bus_voltage().unwrap();
        let current = power_monitor.read_current().unwrap();
        let power = power_monitor.read_power().unwrap();

        // info!(
        // "Shunt Voltage: {} mV, Bus Voltage: {} V, Current: {} mA, Power: {} mW",
        //     shunt_voltage * 1000.0,     // V → mV
        //     bus_voltage,                // V
        //     current * 1000.0,           // A → mA
        //     power * 1000.0              // W → mW
        // );

        let shunt_mv = (shunt_voltage * 1000.0) as i32;
        let bus_v = (bus_voltage * 1000.0) as i32; // optionally scale to mV for consistency
        let current_ma = (current * 1000.0) as i32;
        let power_mw = (power * 1000.0) as i32;

        info!(
            "Shunt Voltage: {} mV, Bus Voltage: {} mV, Current: {} mA, Power: {} mW",
            shunt_mv, bus_v, current_ma, power_mw
        );



        // if let Some((x, y)) = touch.read() {
        //     let style = PrimitiveStyleBuilder::new().fill_color(Rgb565::BLUE).build();

        //     Rectangle::new(Point::new(x - 1, y - 1), Size::new(3, 3))
        //         .into_styled(style)
        //         .draw(&mut display)
        //         .unwrap();
        // }
    }
}
