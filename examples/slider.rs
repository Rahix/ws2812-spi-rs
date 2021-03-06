#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;
use ws2812_spi as ws2812;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use crate::hal::stm32;
use crate::ws2812::Ws2812;
use cortex_m::peripheral::Peripherals;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let gpioa = p.GPIOA.split();

        // Constrain clocking registers
        let rcc = p.RCC.constrain();

        // Configure clock to 8 MHz (i.e. the default) and freeze it
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        // Configure pins for SPI
        let sck = gpioa.pa5.into_alternate_af0();
        let miso = gpioa.pa6.into_alternate_af0();
        let mosi = gpioa.pa7.into_alternate_af0();

        // Configure SPI with 3Mhz rate
        let spi = Spi::spi1(
            p.SPI1,
            (sck, miso, mosi),
            ws2812::MODE,
            3_000_000.hz(),
            clocks,
        );
        const max: usize = 8;
        const color1: (u8, u8, u8) = (0x00, 0xc3 / 5, 0x36 / 5);
        const color2: (u8, u8, u8) = (0x00, 0x24 / 5, 0xb0 / 5);
        let mut data = [(0, 0, 0); max];
        let mut main = 0;
        let mut ws = Ws2812::new(spi);
        let mut up = true;
        loop {
            for i in 0..max {
                let distance = (main as i32 - i as i32).abs() as u8;
                let c1 = (
                    color1.0 as u32 * (max as u32 - distance as u32) / max as u32,
                    color1.1 as u32 * (max as u32 - distance as u32) / max as u32,
                    color1.2 as u32 * (max as u32 - distance as u32) / max as u32,
                );
                let c2 = (
                    color2.0 as u32 * distance as u32 / max as u32,
                    color2.1 as u32 * distance as u32 / max as u32,
                    color2.2 as u32 * distance as u32 / max as u32,
                );
                let ct = (
                    (c1.0 + c2.0) as u8,
                    (c1.1 + c2.1) as u8,
                    (c1.2 + c2.2) as u8,
                );
                data[i] = ct;
            }
            if up {
                if main == max - 1 {
                    up = false;
                    main -= 2;
                }
                main += 1;
            } else {
                if main == 0 {
                    up = true;
                    main += 2;
                }
                main -= 1;
            }
            ws.write(data.iter().map(|a| (a.0, a.1, a.2))).unwrap();
            delay.delay_ms(100 as u16);
        }
    }
    loop {
        continue;
    }
}
