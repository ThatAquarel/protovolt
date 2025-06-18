//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

mod app;
mod ui;

mod lib;

use defmt::*;

use app::App;
use embassy_executor::Spawner;
use embassy_rp::{gpio::Pin, pac::Interrupt::PIO0_IRQ_0};
use embassy_time::{Duration, Ticker, Timer};
use lib::event::HardwareEvent;
use lib::interface::{ButtonInterface, matrix};


use embassy_rp::gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let interface = ButtonInterface::new(
        [p.PIN_8.degrade(), p.PIN_9.degrade(), p.PIN_10.degrade()],
        [p.PIN_5.degrade(), p.PIN_6.degrade(), p.PIN_7.degrade()],
    );
    
    unwrap!(spawner.spawn(poll_interface(interface)));
}

#[embassy_executor::task(pool_size = 1)]
async fn poll_interface(mut interface: ButtonInterface<'static>) {
    let mut ticker = Ticker::every(Duration::from_millis(matrix::POLL_TIME_MS));

    loop {
        {
            if let Some(event) = interface.poll() {
                info!("event recv");
            }
        }

        ticker.next().await;
    }
}
