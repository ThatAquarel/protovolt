//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

mod app;
mod ui;

mod lib;

use core::cell::RefCell;

use defmt::*;

use app::App;

use embassy_executor::Spawner;
use embassy_rp::spi::{self, Spi};
use embassy_rp::{gpio::Pin, pac::Interrupt::PIO0_IRQ_0};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_time::{Duration, Ticker, Timer};

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_hal::spi::SpiBus;
use lib::event::InterfaceEvent;
use lib::interface::{ButtonsInterface, matrix};

use crate::lib::display::DisplayInterface;
use crate::lib::event::{AppEvent, DisplayTask, HardwareEvent};
use crate::ui::controls::{clear, draw_channel_background, draw_self_check};
use embassy_rp::gpio::{Level, Output};

use {defmt_rtt as _, panic_probe as _};

static INTERFACE_CHANNEL: Channel<ThreadModeRawMutex, InterfaceEvent, 32> = Channel::new();
static HARDWARE_CHANNEL: Channel<ThreadModeRawMutex, HardwareEvent, 32> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Buttons
    let buttons = ButtonsInterface::new(
        [p.PIN_8.degrade(), p.PIN_9.degrade(), p.PIN_10.degrade()],
        [p.PIN_5.degrade(), p.PIN_6.degrade(), p.PIN_7.degrade()],
    );

    // Display
    let spi = Spi::new_blocking(p.SPI0, p.PIN_18, p.PIN_19, p.PIN_20, spi::Config::default());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display = DisplayInterface::new(
        &spi_bus,
        p.PIN_17.degrade(),
        p.PIN_21.degrade(),
        p.PIN_28.degrade(),
    );
    let mut target = display.target;

    clear(&mut target);
    draw_self_check(&mut target);


    // State Machine
    let mut a = App::new();

    unwrap!(spawner.spawn(poll_interface(buttons, INTERFACE_CHANNEL.sender())));

    loop {
        let mut next_app_task = None;

        if let Ok(hw_event) = HARDWARE_CHANNEL.try_receive() {
            next_app_task = a.handle_event(AppEvent::Hardware(hw_event));
        } else if let Ok(ui_event) = INTERFACE_CHANNEL.try_receive() {
            next_app_task = a.handle_event(AppEvent::Interface(ui_event));
        }

        if let Some(task) = next_app_task {
            if let Some(hardware_task) = task.hardware{
                
            }
            if let Some(display_task) = task.display {
                match display_task {
                    DisplayTask::SetupSplash => {
                        clear(&mut target);
                        draw_channel_background(&mut target, Rgb565::RED);
                        // draw_self_check(&mut target);
                    }
                    DisplayTask::ConfirmPowerDelivery => {
                        
                    }
                    DisplayTask::ConfirmSense => {
                        
                    }
                    DisplayTask::ConfirmConverter => {
                        
                    }
                    _ => {}
                }
            }
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
async fn poll_interface(
    mut buttons: ButtonsInterface<'static>,
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
