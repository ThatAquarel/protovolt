use defmt::*;
use embassy_rp::{
    dma::Channel, peripherals::PIO0, pio::{Instance, Pio, PioPin}, pio_programs::ws2812::{PioWs2812, PioWs2812Program}
};
use smart_leds::RGB8;

use crate::lib::led::ws2812::LED_COUNT;

use {defmt_rtt as _, panic_probe as _};

pub mod ws2812 {
    pub const LED_COUNT: usize = 7;
}

// pub struct LedInterface<'a, PIO: Instance, DMA: Channel, PIN: PioPin> {
pub struct LedsInterface<'a, PIO: Instance> {
    // pio: Pio<'a, PIO>,
    // dma: DMA,
    // pin: PIN,
    led: PioWs2812<'a, PIO, 0, LED_COUNT>,
    data: [RGB8; LED_COUNT],
}

pub enum LedsColor {
    Settings(RGB8), //0
    Switch(RGB8),   //1
    Enter(RGB8),    //2

    ChannelA(RGB8, RGB8), //3, 4
    ChannelB(RGB8, RGB8), //5, 6
}

impl<'a, PIO> LedsInterface<'a, PIO>
where
    PIO: Instance,
{
    pub fn new<DMA, PIN>(pio: Pio<'a, PIO>, dma: DMA, pin: PIN) -> Self
    where
        DMA: Channel,
        PIN: PioPin,
    {
        let Pio {
            mut common, sm0, ..
        } = pio;

        let program = PioWs2812Program::new(&mut common);
        let ws2812: PioWs2812<'a, PIO, 0, LED_COUNT> =
            PioWs2812::<PIO, 0, LED_COUNT>::new(&mut common, sm0, dma, pin, &program);

        Self {
            led: ws2812,
            data: [RGB8::default(); LED_COUNT],
        }
    }

    async fn refresh(&mut self) {
        self.led.write(&self.data).await;
    }

    pub async fn update_color(&mut self, color: LedsColor) {
        let d = &mut self.data;
        match color {
            LedsColor::Settings(c) => d[0] = c,
            LedsColor::Switch(c) => d[1] = c,
            LedsColor::Enter(c) => d[2] = c,
            LedsColor::ChannelA(c_a, c_b) => (d[5], d[6]) = (c_a, c_b),
            LedsColor::ChannelB(c_a, c_b) => (d[3], d[4]) = (c_a, c_b),
        }

        self.refresh().await;
    }
}
