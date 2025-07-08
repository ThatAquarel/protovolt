#![no_std]
#![no_main]

mod app;
mod hal;
mod task;
mod ui;

use core::cell::RefCell;

use defmt::*;
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_executor::{Executor, Spawner};
use embassy_rp::gpio::{Output, Pin};
use embassy_rp::i2c::I2c;
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_rp::peripherals::{I2C0, I2C1, PIO0};
use embassy_rp::pio;
use embassy_rp::pio::Pio;
use embassy_rp::spi::{self, Spi};
use embassy_rp::{bind_interrupts, i2c};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex};

use embassy_sync::blocking_mutex::Mutex as I2cMutex;

use embassy_sync::channel::{Channel, Sender};
use embassy_time::{Duration, Ticker};

use hal::display::DisplayInterface;
use hal::event::{AppEvent, HardwareEvent, InterfaceEvent, Readout, Task};
use hal::interface::{ButtonsInterface, matrix};

use app::App;
use task::{handle_display_task, handle_hardware_task};
use ui::Ui;

use crate::hal::led::LedsInterface;
use crate::hal::{ChannelDevices, Hal};

use crate::hal::event::Channel as OutputChannel;

use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

// Static channels
pub static INTERFACE_CHANNEL: Channel<ThreadModeRawMutex, InterfaceEvent, 32> = Channel::new();
pub static HARDWARE_CHANNEL: Channel<ThreadModeRawMutex, HardwareEvent, 32> = Channel::new();

// Multicore setup
static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

static BUTTONS_INTERFACE: StaticCell<ButtonsInterface> = StaticCell::new();

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Power hardware initialization
    let i2c0 = I2c::new_blocking(p.I2C0, p.PIN_1, p.PIN_0, i2c::Config::default());
    let i2c0_bus: Mutex<NoopRawMutex, _> = I2cMutex::new(RefCell::new(i2c0));
    let i2c1 = I2c::new_blocking(p.I2C1, p.PIN_23, p.PIN_22, i2c::Config::default());
    let i2c1_bus: Mutex<NoopRawMutex, _> = I2cMutex::new(RefCell::new(i2c1));

    // let pd = I2cDevice::new(&i2c0_bus);
    let hal = Hal::new(&i2c0_bus, &i2c1_bus, p.PIN_25.degrade(), p.PIN_24.degrade());


    // Buttons (moved to static so Core 1 owns them)
    let buttons = ButtonsInterface::new(
        [p.PIN_8.degrade(), p.PIN_9.degrade(), p.PIN_10.degrade()],
        [p.PIN_5.degrade(), p.PIN_6.degrade(), p.PIN_7.degrade()],
    );
    let buttons = BUTTONS_INTERFACE.init(buttons);

    // SPI display setup
    let spi = Spi::new_blocking(p.SPI0, p.PIN_18, p.PIN_19, p.PIN_20, spi::Config::default());
    let spi_shared: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));
    let mut display = DisplayInterface::new(
        &spi_shared,
        p.PIN_17.degrade(),
        p.PIN_21.degrade(),
        p.PIN_28.degrade(),
    );
    let _backlight = Output::new(p.PIN_16, embassy_rp::gpio::Level::High);

    // Interfacing LEDs setup
    let pio = Pio::new(p.PIO0, Irqs);
    let leds = LedsInterface::new(pio, p.DMA_CH0, p.PIN_11);

    // App logic
    let mut app = App::default();
    let mut ui = Ui::new(&mut display.target, leds);

    // Start core 1 and spawn poll_interface there
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                unwrap!(spawner.spawn(poll_interface(buttons, INTERFACE_CHANNEL.sender())));
            });
        },
    );

    // Start application
    let hw_sender = HARDWARE_CHANNEL.sender();
    hw_sender.send(HardwareEvent::PowerOn).await;
    let int_sender = INTERFACE_CHANNEL.sender();

    let mut ticker = Ticker::every(Duration::from_hz(100));
    loop {
        let mut next_app_task = None;
        if let Ok(hw_event) = HARDWARE_CHANNEL.try_receive() {
            next_app_task = app.handle_event(AppEvent::Hardware(hw_event));
        } else if let Ok(ui_event) = INTERFACE_CHANNEL.try_receive() {
            next_app_task = app.handle_event(AppEvent::Interface(ui_event));
        }

        if let Some(app_task) = next_app_task {
            for task in app_task {
                match task {
                    Task::Hardware(hw_task) => {
                        handle_hardware_task(hw_task, spawner, &hw_sender, &int_sender).await;
                    }
                    Task::Display(disp_task) => {
                        handle_display_task(disp_task, &mut ui, &hw_sender, &int_sender).await
                    }
                }
            }
        }

        ticker.next().await;
    }
}

#[embassy_executor::task]
pub async fn poll_readout(channel: Sender<'static, ThreadModeRawMutex, HardwareEvent, 32>) {
    let mut ticker = Ticker::every(Duration::from_hz(5)); // 100ms
    let mut v: f32 = 10.0;
    let mut c: f32 = 5.0;
    let mut p: f32 = 0.0;

    loop {
        channel
            .send(HardwareEvent::ReadoutAcquired(
                hal::event::Channel::A,
                Readout {
                    voltage: v,
                    current: c,
                    power: p,
                },
            ))
            .await;

        channel
            .send(HardwareEvent::ReadoutAcquired(
                hal::event::Channel::B,
                Readout {
                    voltage: (v + 1.5).min(20.0),
                    current: (c + 0.2).min(5.0),
                    power: (p + 2.5).min(99.0),
                },
            ))
            .await;

        v += 0.001;
        c += 0.005;
        p += 0.002;

        if v > 20.0 {
            v = 0.0;
        }
        if c > 5.0 {
            c = 0.0;
        }
        if p > 99.0 {
            p = 0.0;
        }

        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn poll_interface(
    buttons: &'static mut ButtonsInterface<'static>,
    channel: Sender<'static, ThreadModeRawMutex, InterfaceEvent, 32>,
) {
    let mut ticker = Ticker::every(Duration::from_millis(matrix::POLL_TIME_MS));
    loop {
        if let Some(event) = buttons.poll() {
            channel.send(event).await;
        }
        ticker.next().await;
    }
}
