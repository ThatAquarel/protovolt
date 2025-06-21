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
use embedded_graphics::prelude::*;
use embedded_hal::spi::SpiBus;
use lib::event::InterfaceEvent;
use lib::interface::{ButtonsInterface, matrix};

use crate::lib::display::DisplayInterface;
use crate::lib::event::{AppEvent, DisplayTask, HardwareEvent, HardwareTask, Readout};
use crate::ui::boot;
use crate::ui::controls;
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

    // State Machine
    let mut a = App::new();

    unwrap!(spawner.spawn(poll_interface(buttons, INTERFACE_CHANNEL.sender())));
    // unwrap!(spawner.spawn(poll_readout(HARDWARE_CHANNEL.sender())));

    let hw_sender = HARDWARE_CHANNEL.sender();

    hw_sender.send(HardwareEvent::PowerOn).await;

    let mut ticker = Ticker::every(Duration::from_hz(1000));

    loop {
        let mut next_app_task = None;

        if let Ok(hw_event) = HARDWARE_CHANNEL.try_receive() {
            next_app_task = a.handle_event(AppEvent::Hardware(hw_event));
        } else if let Ok(ui_event) = INTERFACE_CHANNEL.try_receive() {
            next_app_task = a.handle_event(AppEvent::Interface(ui_event));
        }

        if let Some(task) = next_app_task {
            if let Some(hardware_task) = task.hardware {
                match hardware_task {
                    HardwareTask::EnablePowerDelivery => {
                        Timer::after_millis(10).await;

                        hw_sender
                            .send(HardwareEvent::PowerDeliveryReady)
                            .await;
                    }
                    HardwareTask::EnableSense => {
                        Timer::after_millis(10).await;

                        hw_sender.send(HardwareEvent::SenseReady).await;
                    }
                    HardwareTask::EnableConverter => {
                        Timer::after_millis(10).await;

                        hw_sender.send(HardwareEvent::ConverterReady).await;
                    }
                    HardwareTask::EnableReadoutLoop => {
                        info!("enable readout loop");
                        unwrap!(spawner.spawn(poll_readout(HARDWARE_CHANNEL.sender())));
                    }

                    HardwareTask::DelayedHardwareEvent(duration, event) => {
                        Timer::after(duration).await;
                        hw_sender.send(event).await;
                    }
                }
            }
            if let Some(display_task) = task.display {
                match display_task {
                    DisplayTask::SetupSplash => {
                        controls::clear(&mut target);
                        // draw_channel_background(&mut target, Rgb565::RED);
                        boot::draw_splash_screen(&mut target);
                    }
                    DisplayTask::ConfirmPowerDelivery => {
                        boot::draw_splash_text(&mut target, 0, "INPUT", "PD 20V 5A", true);
                    }
                    DisplayTask::ConfirmSense => {
                        boot::draw_splash_text(&mut target, 1, "SENSE", "CH A ERR", false);
                    }
                    DisplayTask::ConfirmConverter => {
                        boot::draw_splash_text(&mut target, 2, "CONVERTER", "CH A, CH B", true);
                    }
                    DisplayTask::SetupMain => {
                        controls::clear(&mut target);
                        controls::draw_power_header(&mut target).unwrap();
                        controls::draw_buttons(&mut target).unwrap();

                        let mut ch_a_section = target.translated(Point::new(0, 40));
                        controls::draw_channel_background(&mut ch_a_section, Rgb565::CSS_TOMATO)
                            .unwrap();
                        controls::draw_header_text(&mut ch_a_section, "CHANNEL A").unwrap();
                        controls::draw_units(&mut ch_a_section).unwrap();
                        controls::draw_submeasurement(&mut ch_a_section).unwrap();
                        // controls::draw_measurements(&mut ch_a_section).unwrap();

                        let mut ch_b_section = target.translated(Point::new(163, 40));
                        controls::draw_channel_background(
                            &mut ch_b_section,
                            Rgb565::CSS_ROYAL_BLUE,
                        )
                        .unwrap();
                        controls::draw_header_text(&mut ch_b_section, "CHANNEL B").unwrap();
                        controls::draw_units(&mut ch_b_section).unwrap();
                        controls::draw_submeasurement(&mut ch_b_section).unwrap();
                        // controls::draw_measurements(&mut ch_b_section).unwrap();
                    }
                    DisplayTask::UpdateReadout(channel, readout) => {
                        let mut section = match channel {
                            lib::event::Channel::A => target.translated(Point::new(0, 40)),
                            lib::event::Channel::B => target.translated(Point::new(163, 40)),
                        };

                        controls::draw_measurements(&mut section, readout);
                    }
                    _ => {}
                }
            }
        }

        ticker.next().await;
    }
}

// #[embassy_executor::task]
// async fn poll_readout(channel: Sender<'static, ThreadModeRawMutex, HardwareEvent, 32>) {
//     let mut ticker = Ticker::every(Duration::from_hz(10));
    
//     loop {
//         channel.send(HardwareEvent::ReadoutAcquired(
//             lib::event::Channel::A,
//             Readout {
//                 voltage: 5.001,
//                 current: 0.122,
//                 power: 0.056,
//             },
//         )).await;

//         channel.send(HardwareEvent::ReadoutAcquired(
//             lib::event::Channel::B,
//             Readout {
//                 voltage: 12.123,
//                 current: 0.879,
//                 power: 0.548,
//             },
//         )).await;

//         ticker.next().await;
//     }
// }

#[embassy_executor::task]
async fn poll_readout(channel: Sender<'static, ThreadModeRawMutex, HardwareEvent, 32>) {
    let mut ticker = Ticker::every(Duration::from_hz(10)); // 100ms
    let mut v: f32 = 0.0;
    let mut c: f32 = 0.0;
    let mut p: f32 = 0.0;

    loop {
        // Send mock readout for Channel A
        channel
            .send(HardwareEvent::ReadoutAcquired(
                lib::event::Channel::A,
                Readout {
                    voltage: v,
                    current: c,
                    power: p,
                },
            ))
            .await;

        // Send mock readout for Channel B (add small offset for variety)
        channel
            .send(HardwareEvent::ReadoutAcquired(
                lib::event::Channel::B,
                Readout {
                    voltage: (v + 1.5).min(20.0),
                    current: (c + 0.2).min(5.0),
                    power: (p + 2.5).min(99.0),
                },
            ))
            .await;

        // Increment values slightly
        v += 0.5;
        c += 0.1;
        p += 1.0;

        // Loop back when max is reached
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
