use embassy_rp::{
    Peri,
    dma::ChannelInstance,
    interrupt,
    pio::{Instance, Pio, PioPin},
    pio_programs::ws2812::{Grb, PioWs2812, PioWs2812Program},
};
use smart_leds::RGB8;

use crate::hal::led::ws2812::LED_COUNT;

use {defmt_rtt as _, panic_probe as _};

pub mod ws2812 {
    pub const LED_COUNT: usize = 7;
}

pub struct LedsInterface<'a, PIO: Instance> {
    led: PioWs2812<'a, PIO, 0, LED_COUNT, Grb>,
    data: [RGB8; LED_COUNT],
}

pub enum LedsColor {
    Settings(RGB8), //0
    Switch(RGB8),   //1
    Enter(RGB8),    //2

    ChannelA(RGB8, RGB8), //5, 6
    ChannelB(RGB8, RGB8), //3, 4
}

impl<'a, PIO> LedsInterface<'a, PIO>
where
    PIO: Instance,
{
    pub fn new<DMA, PIN, I>(
        pio: Pio<'a, PIO>,
        dma: Peri<'a, DMA>,
        irq: I,
        pin: Peri<'a, PIN>,
    ) -> Self
    where
        DMA: ChannelInstance,
        PIN: PioPin,
        I: interrupt::typelevel::Binding<DMA::Interrupt, embassy_rp::dma::InterruptHandler<DMA>>
            + 'a,
    {
        let Pio {
            mut common, sm0, ..
        } = pio;

        let program = PioWs2812Program::new(&mut common);
        let ws2812 = PioWs2812::new(&mut common, sm0, dma, irq, pin, &program);

        Self {
            led: ws2812,
            data: [RGB8::default(); LED_COUNT],
        }
    }

    pub async fn refresh(&mut self) {
        self.led.write(&self.data).await;
    }

    pub fn update_color(&mut self, color: LedsColor) {
        let d = &mut self.data;
        match color {
            LedsColor::Settings(c) => d[0] = c,
            LedsColor::Switch(c) => d[1] = c,
            LedsColor::Enter(c) => d[2] = c,
            LedsColor::ChannelA(c_a, c_b) => (d[5], d[6]) = (c_a, c_b),
            LedsColor::ChannelB(c_a, c_b) => (d[3], d[4]) = (c_a, c_b),
        }
    }

    pub async fn update_refresh(&mut self, color: LedsColor) {
        self.update_color(color);
        self.refresh().await;
    }
}
