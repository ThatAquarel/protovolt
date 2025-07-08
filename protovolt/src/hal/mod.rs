use core::{cell::RefCell, str::EncodeUtf16};

use embassy_rp::gpio::AnyPin;
use embassy_sync::blocking_mutex::{Mutex, raw::RawMutex};
use embedded_hal::{digital::OutputPin, i2c::I2c};

use crate::hal::{converter::ConverterDevice, event::Channel, measure::{Measure, MeasureDevice}};

pub mod display;
pub mod event;
pub mod interface;
pub mod led;

mod device;

pub mod converter;
pub mod measure;

pub struct ChannelDevices<'a, M: RawMutex, BUS0: I2c, BUS1: I2c> {
    measure: MeasureDevice<'a, M, BUS1>,
    converter: ConverterDevice<'a, M, BUS0>,
}

impl<'a, M, BUS0, BUS1> ChannelDevices<'a, M, BUS0, BUS1>
where
    M: RawMutex,
    BUS0: I2c + 'a,
    BUS1: I2c + 'a,
{
    pub fn new(
        ch: Channel,
        converter_bus: &'a Mutex<M, RefCell<BUS0>>,
        measure_bus: &'a Mutex<M, RefCell<BUS1>>,
        enable_pin: AnyPin,
    ) -> Self {
        let measure = MeasureDevice::new(measure_bus, ch);
        let converter = ConverterDevice::new(enable_pin, converter_bus, ch);

        Self {
            measure: measure,
            converter: converter,
        }
    }
}

pub struct Hal<'a, M: RawMutex, BUS0: I2c, BUS1: I2c> {
    ch_a: ChannelDevices<'a, M, BUS0, BUS1>,
    ch_b: ChannelDevices<'a, M, BUS0, BUS1>,
}

impl<'a, M, BUS0, BUS1> Hal<'a, M, BUS0, BUS1>
where
    M: RawMutex,
    BUS0: I2c + 'a,
    BUS1: I2c + 'a,
{
    pub fn new(
        converter_bus: &'a Mutex<M, RefCell<BUS0>>,
        measure_bus: &'a Mutex<M, RefCell<BUS1>>,
        ch_a_enable: AnyPin,
        ch_b_enable: AnyPin,
    ) -> Self {
        Self {
            ch_a: ChannelDevices::new(Channel::A, &converter_bus, &measure_bus, ch_a_enable),
            ch_b: ChannelDevices::new(Channel::B, &converter_bus, &measure_bus, ch_b_enable),
        }
    }

    pub fn enable_sense(&mut self) -> Result<(), ()> {
        self.ch_a.measure.init()?;
        self.ch_b.measure.init()
    }
}
