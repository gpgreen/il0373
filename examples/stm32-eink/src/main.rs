#![no_std]
#![no_main]

// gud invocation: gdb-multiarch -x openocd.gdb -q target/thumbv7m-none-eabi/debug/stm32-eink

// pick a panicking behavior
//extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
//extern crate panic_abort; // requires nightly
//extern crate panic_itm; // logs messages over ITM; requires ITM support
extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
                                //use panic_semihosting;

use board::{
    hal::{adc::Adc, delay::Delay, gpio::*, pac, prelude::*, spi::*},
    spi::Mode,
};
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use heapless::consts::*;
use heapless::String;
use nucleo_f103rb as board;

use il0373::{
    Builder, Color, Dimensions, Display, Rotation, SpiBus, SramDisplayInterface, SramGraphicDisplay,
};

// Graphics
extern crate embedded_graphics;

use embedded_graphics::{
    fonts::{Font12x16, Font6x8, Text},
    prelude::*,
    primitives::Line,
    style::PrimitiveStyle,
    style::TextStyleBuilder,
};

const ROWS: u16 = 212;
const COLS: u8 = 104;

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store
    // the frozen frequencies in `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // afio
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    // gpioa
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    // configure Digital I/O pins
    let busy = gpioa.pa8.into_pull_up_input(&mut gpioa.crh);
    let dc = gpioc.pc7.into_push_pull_output(&mut gpioc.crl);
    let reset = gpioa.pa9.into_push_pull_output(&mut gpioa.crh);
    let display_pins = (busy, dc, reset);

    //configure adc
    let mut adc = Adc::adc1(dp.ADC1, &mut rcc.apb2, clocks);

    // spi pins
    let pins = (
        gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
        gpioa.pa6.into_floating_input(&mut gpioa.crl),
        gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
    );

    let epd_cs = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);
    let sram_cs = gpiob.pb10.into_push_pull_output(&mut gpiob.crh);
    let mut sdmmc_cs = gpiob.pb5.into_push_pull_output(&mut gpiob.crl);
    sdmmc_cs.set_high().unwrap();
    let cs_pins = (epd_cs, sram_cs);

    // configure spi1
    let spi = Spi::spi1(
        dp.SPI1,
        pins,
        &mut afio.mapr,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        4.mhz(),
        clocks,
        &mut rcc.apb2,
    );
    let spi_bus = SpiBus::new(spi, cs_pins);

    // Initialize display controller
    let mut delay = Delay::new(cp.SYST, clocks);

    let controller = SramDisplayInterface::new(spi_bus, display_pins);

    delay.delay_ms(800u32);

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

    let text_style_black = TextStyleBuilder::new(Font6x8)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();
    let text_style_red = TextStyleBuilder::new(Font12x16)
        .text_color(Color::Red)
        .background_color(Color::White)
        .build();

    // Check the temperature and display it, wait for 180s, and do it again
    loop {
        let temp = adc.read_temp();
        let mut status = String::<U32>::from("Nucleo-F103RB: ");
        status.push_str(&String::<U32>::from(temp)).ok();

        display.reset(&mut delay).ok();
        display.clear(Color::White).ok();
        Text::new(status.as_str(), Point::new(70, 49))
            .into_styled(text_style_black)
            .draw(&mut display)
            .ok();
        Text::new("Hello!", Point::new(120, 15))
            .into_styled(text_style_red)
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
        delay.delay_ms(180_000u32);
    }
}
