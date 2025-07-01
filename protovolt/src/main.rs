#![no_std]
#![no_main]

mod app;
mod task;
mod ui;
mod lib;

use core::cell::RefCell;

use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_rp::gpio::{Output, Pin};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::spi::{self, Spi};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex};
use embassy_sync::channel::{Channel, Sender};
use embassy_time::{Duration, Ticker, Timer};

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use lib::event::{InterfaceEvent, AppEvent, DisplayTask, HardwareEvent, Limits, Readout, Task};
use lib::interface::{ButtonsInterface, matrix};
use lib::display::DisplayInterface;

use task::{handle_display_task, handle_hardware_task};
use ui::{Ui, boot};
use ui::color_scheme::{self, CH_B_SELECTED};
use app::App;
use ui::controls;

use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Static channels
pub static INTERFACE_CHANNEL: Channel<ThreadModeRawMutex, InterfaceEvent, 32> = Channel::new();
pub static HARDWARE_CHANNEL: Channel<ThreadModeRawMutex, HardwareEvent, 32> = Channel::new();

// Multicore setup
static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static BUTTONS_INTERFACE: StaticCell<ButtonsInterface> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Buttons (moved to static so Core 1 owns them)
    let buttons = ButtonsInterface::new(
        [p.PIN_8.degrade(), p.PIN_9.degrade(), p.PIN_10.degrade()],
        [p.PIN_5.degrade(), p.PIN_6.degrade(), p.PIN_7.degrade()],
    );
    let buttons = BUTTONS_INTERFACE.init(buttons);

    // SPI display setup
    let spi = Spi::new_blocking(p.SPI0, p.PIN_18, p.PIN_19, p.PIN_20, spi::Config::default());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));
    let mut display = DisplayInterface::new(
        &spi_bus,
        p.PIN_17.degrade(),
        p.PIN_21.degrade(),
        p.PIN_28.degrade(),
    );
    let _backlight = Output::new(p.PIN_16, embassy_rp::gpio::Level::High);

    // App logic
    let mut a = App::default();
    let mut ui = Ui::new(&mut display.target);

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
            next_app_task = a.handle_event(AppEvent::Hardware(hw_event));
        } else if let Ok(ui_event) = INTERFACE_CHANNEL.try_receive() {
            next_app_task = a.handle_event(AppEvent::Interface(ui_event));
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
        channel.send(HardwareEvent::ReadoutAcquired(
            lib::event::Channel::A,
            Readout {
                voltage: v,
                current: c,
                power: p,
            },
        )).await;

        channel.send(HardwareEvent::ReadoutAcquired(
            lib::event::Channel::B,
            Readout {
                voltage: (v + 1.5).min(20.0),
                current: (c + 0.2).min(5.0),
                power: (p + 2.5).min(99.0),
            },
        )).await;

        v += 0.001;
        c += 0.005;
        p += 0.002;

        if v > 20.0 { v = 0.0; }
        if c > 5.0 { c = 0.0; }
        if p > 99.0 { p = 0.0; }

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
