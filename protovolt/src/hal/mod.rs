use core::cell::RefCell;

use embassy_rp::{
    adc::{self, Adc, AdcPin},
    gpio::{AnyPin, Input, Pull},
    peripherals::{ADC_TEMP_SENSOR, PIN_14, PIN_15},
};
use embassy_sync::{
    blocking_mutex::{
        Mutex,
        raw::{NoopRawMutex, RawMutex, ThreadModeRawMutex},
    },
    channel::{Channel, Receiver, Sender},
};
use embassy_time::{Duration, Timer, Ticker, WithTimeout};
use embedded_hal::i2c::I2c;

use crate::{
    HardwareChannelSender, StaticI2c1,
    hal::{
        converter::{Converter, ConverterDevice},
        event::{Channel as OutputChannel, ChannelHardwareState, HardwareEvent},
        measure::{Measure, MeasureDevice},
        temperature::{Temperature, TemperatureDevice},
    },
};

pub mod backlight;
pub mod display;
pub mod event;
pub mod interface;
pub mod led;

mod device;

pub mod converter;
pub mod measure;
pub mod power;
pub mod temperature;

use event::Channel as ConverterChannel;

pub struct Hal<'a, M: RawMutex, BUS: I2c> {
    ch_a: ConverterDevice<'a, M, BUS>,
    ch_b: ConverterDevice<'a, M, BUS>,
}

impl<'a, M, BUS> Hal<'a, M, BUS>
where
    M: RawMutex,
    BUS: I2c + 'a,
{
    pub fn new(
        converter_bus: &'a Mutex<M, RefCell<BUS>>,
        ch_a_enable: AnyPin,
        ch_b_enable: AnyPin,
    ) -> Self {
        Self {
            ch_a: ConverterDevice::new(ch_a_enable, converter_bus, OutputChannel::A),
            ch_b: ConverterDevice::new(ch_b_enable, converter_bus, OutputChannel::B),
        }
    }

    pub async fn enable_sense(&mut self) {
        SENSE_CHANNEL.send(SenseEvent::Enable).await;
    }

    pub async fn enable_readout_loop(&mut self) {
        SENSE_CHANNEL.send(SenseEvent::StartReadoutLoop).await;
    }

    pub async fn enable_converter(&mut self) -> Result<(), ()> {
        self.ch_a.init().await?;
        self.ch_b.init().await
    }

    pub async fn update_converter_state(
        &mut self,
        channel: OutputChannel,
        active: bool,
    ) -> Result<(), ()> {
        let ch = match channel {
            OutputChannel::A => &mut self.ch_a,
            OutputChannel::B => &mut self.ch_b,
        };
        match active {
            true => ch.enable(),
            false => ch.disable(),
        }
    }

    pub fn poll_converter_status(&mut self, channel: OutputChannel) -> Result<ChannelHardwareState, ()> {
        match channel {
            OutputChannel::A => self.ch_a.get_status(),
            OutputChannel::B => self.ch_b.get_status(),
        }
    }

    pub async fn update_converter_voltage(
        &mut self,
        channel: OutputChannel,
        voltage: f32,
    ) -> Result<(), ()> {
        let voltage = (voltage * 1000.0) as u16;
        match channel {
            OutputChannel::A => self.ch_a.set_voltage(voltage).await,
            OutputChannel::B => self.ch_b.set_voltage(voltage).await,
        }
    }

    pub async fn update_converter_current(
        &mut self,
        channel: OutputChannel,
        current: f32,
    ) -> Result<(), ()> {
        let current = (current * 1000.0) as u16;
        match channel {
            OutputChannel::A => self.ch_a.set_current(current),
            OutputChannel::B => self.ch_b.set_current(current),
        }
    }

    pub fn dump_tps55289<const N: usize>(
        &mut self,
        channel: OutputChannel,
        buf: &mut heapless::String<N>,
    ) -> Result<(), ()> {
        match channel {
            OutputChannel::A => self.ch_a.dump_registers(buf),
            OutputChannel::B => self.ch_b.dump_registers(buf),
        }
    }
}

pub struct HalSense<'a, M: RawMutex, BUS: I2c> {
    ch_a: MeasureDevice<'a, M, BUS>,
    ch_b: MeasureDevice<'a, M, BUS>,
}

impl<'a, M, BUS> HalSense<'a, M, BUS>
where
    M: RawMutex,
    BUS: I2c + 'a,
{
    pub fn new(measure_bus: &'a Mutex<M, RefCell<BUS>>) -> Self {
        Self {
            ch_a: MeasureDevice::new(measure_bus, OutputChannel::A),
            ch_b: MeasureDevice::new(measure_bus, OutputChannel::B),
        }
    }

    pub fn dump_ina226<const N: usize>(
        &mut self,
        channel: OutputChannel,
        buf: &mut heapless::String<N>,
    ) -> Result<(), ()> {
        match channel {
            OutputChannel::A => self.ch_a.dump_registers(buf),
            OutputChannel::B => self.ch_b.dump_registers(buf),
        }
    }
}

pub enum SenseEvent {
    Enable,
    StartReadoutLoop,
}

pub static SENSE_CHANNEL: Channel<ThreadModeRawMutex, SenseEvent, 1> = Channel::new();

pub static INA226_DUMP_REQ: Channel<ThreadModeRawMutex, OutputChannel, 2> = Channel::new();
pub static INA226_DUMP_RESP: Channel<
    ThreadModeRawMutex,
    Result<heapless::String<{ crate::scpi::RESPONSE_BUF }>, ()>,
    2,
> = Channel::new();

async fn service_ina226_dump(sense: &mut HalSense<'static, NoopRawMutex, StaticI2c1>) {
    if let Ok(channel) = INA226_DUMP_REQ.try_receive() {
        let mut buf = heapless::String::<{ crate::scpi::RESPONSE_BUF }>::new();
        let result = sense.dump_ina226(channel, &mut buf).map(|_| buf);
        INA226_DUMP_RESP.send(result).await;
    }
}

#[embassy_executor::task]
pub async fn poll_sense(
    sense: &'static mut HalSense<'static, NoopRawMutex, StaticI2c1>,
    sense_channel: Receiver<'static, ThreadModeRawMutex, SenseEvent, 1>,
    data_channel: HardwareChannelSender,
) {
    match sense_channel.receive().await {
        SenseEvent::Enable => {}
        _ => return,
    };

    let (a, b) = (sense.ch_a.init(), sense.ch_b.init());
    if a.is_ok() && b.is_ok() {
        data_channel.send(HardwareEvent::SenseReady(Ok(()))).await;
    } else {
        data_channel.send(HardwareEvent::SenseReady(Err(()))).await;
        return;
    };

    loop {
        service_ina226_dump(sense).await;
        match sense_channel.try_receive() {
            Ok(SenseEvent::StartReadoutLoop) => break,
            Ok(_) => {}
            Err(_) => Timer::after_millis(1).await,
        }
    }

    let mut ticker = Ticker::every(Duration::from_hz(5)); // 100ms
    loop {
        service_ina226_dump(sense).await;

        let channels = [OutputChannel::A, OutputChannel::B];
        for event_ch in channels.iter() {
            let ch = match event_ch {
                OutputChannel::A => &mut sense.ch_a,
                OutputChannel::B => &mut sense.ch_b,
            };

            let v = ch.read_bus_voltage();
            let i: Result<f32, ()> = ch.read_current();
            let p = ch.read_power();

            if let (Ok(v), Ok(i), Ok(p)) = (v, i, p) {
                let _ = data_channel.try_send(HardwareEvent::ReadoutAcquired(
                    *event_ch,
                    event::Readout {
                        voltage: v,
                        current: i,
                        power: p,
                    },
                ));
            }
        }

        ticker.next().await;
    }
}

pub struct HalTempSense<'a> {
    temp_devices: TemperatureDevice<'a>,
}

impl<'a> HalTempSense<'a> {
    pub fn new(
        adc: Adc<'a, adc::Async>,
        ch_a_pin: impl AdcPin,
        ch_b_pin: impl AdcPin,
        mcu_pin: ADC_TEMP_SENSOR,
    ) -> Self {
        let ch_a = adc::Channel::new_pin(ch_a_pin, Pull::None);
        let ch_b = adc::Channel::new_pin(ch_b_pin, Pull::None);
        let mcu: adc::Channel<'_> = adc::Channel::new_temp_sensor(mcu_pin);

        Self {
            temp_devices: TemperatureDevice::new(adc, ch_a, ch_b, mcu),
        }
    }
}

#[embassy_executor::task]
pub async fn temp_sense(
    temp_sense: &'static mut HalTempSense<'static>,
    data_channel: HardwareChannelSender,
) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        let temp = temp_sense.temp_devices.read_temperature().await;
        if let Ok(reading) = temp {
            let _ = data_channel.try_send(HardwareEvent::TempAcquired(reading));
        }

        ticker.next().await;
    }
}

macro_rules! converter_irq_task {
    (
        $task_name:ident,
        $pin:ty,
        $channel:expr
    ) => {
        #[embassy_executor::task]
        pub async fn $task_name(
            data_channel: HardwareChannelSender,
            irq_pin: $pin,
        ) {
            let mut irq_pin = Input::new(irq_pin, Pull::None);
            let mut ticker = Ticker::every(Duration::from_hz(5)); // 100ms

            loop {
                let _ = irq_pin
                    .wait_for_falling_edge()
                    .await;

                defmt::warn!("converter IRQ triggered");

                data_channel
                    .send(HardwareEvent::PollConverterStatusInterrupt($channel))
                    .await;

                ticker.next().await;
            }
        }
    };
}

converter_irq_task!(
    converter_a_irq,
    PIN_15,
    ConverterChannel::A
);

converter_irq_task!(
    converter_b_irq,
    PIN_14,
    ConverterChannel::B
);