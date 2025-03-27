#![no_std]
#![no_main]

use crate::board::{adc, gpio, rcc, spi};
use embassy_executor::Spawner;
use embassy_stm32 as board;
use embassy_stm32::time::Hertz;
use embassy_time::{Delay, Timer};
use heapless::consts::*;
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

use il0373::{
    Builder, Color, Dimensions, Display, Rotation, SpiSramBus, SramDisplayInterface,
    SramGraphicDisplay,
};

// Graphics
extern crate embedded_graphics;

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X9},
        MonoTextStyle,
    },
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::Text,
};

const ROWS: u16 = 212;
const COLS: u8 = 104;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hse = Some(rcc::Hse {
        freq: Hertz(8_000_000),
        // Oscillator for bluepill, Bypass for nucleos.
        mode: rcc::HseMode::Bypass,
    });
    config.rcc.pll = Some(rcc::Pll {
        src: rcc::PllSource::HSE,
        prediv: rcc::PllPreDiv::DIV1,
        mul: rcc::PllMul::MUL7,
    });
    config.rcc.sys = rcc::Sysclk::PLL1_P;
    config.rcc.ahb_pre = rcc::AHBPrescaler::DIV1;
    config.rcc.apb1_pre = rcc::APBPrescaler::DIV2;
    config.rcc.apb2_pre = rcc::APBPrescaler::DIV2;
    config.rcc.adc_pre = rcc::ADCPrescaler::DIV2;
    let p = embassy_stm32::init(config);

    // configure Digital I/O pins
    let busy = gpio::Input::new(p.PA8, gpio::Pull::Up); // pull-up input
    let dc = gpio::Output::new(p.PC7, gpio::Level::High, gpio::Speed::Low);
    let reset = gpio::Output::new(p.PA9, gpio::Level::High, gpio::Speed::Low);

    let display_pins = (busy, dc, reset);

    //configure adc
    let mut adc = adc::Adc::new(p.ADC1);
    let mut temp_ch = adc.enable_temperature();

    let epd_cs = gpio::Output::new(p.PB6, gpio::Level::High, gpio::Speed::Low);
    let sram_cs = gpio::Output::new(p.PB10, gpio::Level::High, gpio::Speed::Low);
    let mut sdmmc_cs = gpio::Output::new(p.PB5, gpio::Level::High, gpio::Speed::Low);
    sdmmc_cs.set_high();
    let cs_pins = (epd_cs, sram_cs);

    // configure spi1
    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz(4_000_000);
    spi_config.mode = spi::Mode {
        polarity: spi::Polarity::IdleLow,
        phase: spi::Phase::CaptureOnFirstTransition,
    };

    let spi = spi::Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_config);
    let spi_bus = SpiSramBus::new(spi, cs_pins);

    // Initialize display controller
    let controller = SramDisplayInterface::new(spi_bus, display_pins);

    Timer::after_millis(800).await;

    let config = Builder::new()
        .dimensions(Dimensions {
            rows: ROWS,
            cols: COLS,
        })
        .rotation(Rotation::Rotate270)
        .build()
        .ok()
        .unwrap();

    let display = Display::new(controller, config);

    let mut display = SramGraphicDisplay::new(display);

    let text_style_black = MonoTextStyle::new(&FONT_6X9, Color::Black);
    let text_style_red = MonoTextStyle::new(&FONT_10X20, Color::Red);

    // Check the temperature and display it, wait for 180s, and do it again
    let mut delay = Delay;
    loop {
        let temp = adc.read(&mut temp_ch).await;
        let mut status = String::<U32>::from("Nucleo-F103RB: ");
        status.push_str(&String::<U32>::from(temp)).ok();

        display.reset(&mut delay).ok();
        display.clear(Color::White).ok();
        Text::new(status.as_str(), Point::new(70, 52), text_style_black)
            .draw(&mut display)
            .ok();
        Text::new("Hello!", Point::new(120, 15), text_style_red)
            .draw(&mut display)
            .ok();
        Line::new(Point::new(10, 10), Point::new(100, 96))
            .into_styled(PrimitiveStyle::with_stroke(Color::Black, 5))
            .draw(&mut display)
            .ok();
        Line::new(Point::new(10, 96), Point::new(100, 10))
            .into_styled(PrimitiveStyle::with_stroke(Color::Black, 5))
            .draw(&mut display)
            .ok();
        Line::new(Point::new(55, 10), Point::new(55, 96))
            .into_styled(PrimitiveStyle::with_stroke(Color::Red, 5))
            .draw(&mut display)
            .ok();
        display.update().ok();
        display.deep_sleep().ok();

        // adafruit says to only update the display every 180 seconds
        // or risk damaging the display
        Timer::after_millis(180_000).await;
    }
}
