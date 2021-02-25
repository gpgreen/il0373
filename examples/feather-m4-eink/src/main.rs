#![no_std]
#![no_main]

// gud invocation: gdb-multiarch -x jlink.gdb -q target/thumbv7em-none-eabihf/debug/feather-m4-eink

// pick a panicking behavior
//extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
//extern crate panic_abort; // requires nightly
//extern crate panic_itm; // logs messages over ITM; requires ITM support
extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use feather_m4::{
    adc::Adc, adc::DeviceSensor, clock, delay::Delay, pac::gclk::pchctrl::GEN_A,
    pac::CorePeripherals, pac::Peripherals, prelude::*, spi_master, Pins,
};
use heapless::consts::*;
use heapless::String;

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

#[feather_m4::entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = CorePeripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let mut dp = Peripherals::take().unwrap();

    // the clock
    let mut clocks = clock::GenericClockController::with_external_32kosc(
        dp.GCLK,
        &mut dp.MCLK,
        &mut dp.OSC32KCTRL,
        &mut dp.OSCCTRL,
        &mut dp.NVMCTRL,
    );

    let mut pins = Pins::new(dp.PORT);

    // configure Digital I/O pins
    let busy = pins.d12.into_floating_input(&mut pins.port);
    let dc = pins.d6.into_push_pull_output(&mut pins.port);
    let reset = pins.d11.into_push_pull_output(&mut pins.port);
    let display_pins = (busy, dc, reset);

    let epd_cs = pins.d5.into_push_pull_output(&mut pins.port);
    let sram_cs = pins.d9.into_push_pull_output(&mut pins.port);
    let mut sdmmc_cs = pins.d10.into_push_pull_output(&mut pins.port);
    sdmmc_cs.set_high().unwrap();
    let cs_pins = (epd_cs, sram_cs);

    // configure spi3
    let spi = spi_master(
        &mut clocks,
        4_000_000u32.hz(),
        dp.SERCOM1,
        &mut dp.MCLK,
        pins.sck,
        pins.mosi,
        pins.miso,
        &mut pins.port,
    );

    // configure adc
    let mut adc = Adc::adc0(dp.ADC0, &mut dp.MCLK, &mut clocks, GEN_A::GCLK11);

    let mut delay = Delay::new(cp.SYST, &mut clocks);

    // configure display
    let spi_bus = SpiBus::new(spi, cs_pins);
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
        let vcore = adc.read_device_sensors(DeviceSensor::SCALEDCOREVCC);
        let vbat = adc.read_device_sensors(DeviceSensor::SCALEDVBAT);
        let vio = adc.read_device_sensors(DeviceSensor::SCALEDIOVCC);
        let status = String::<U32>::from("Feather-M4: ");
        let mut vc = String::<U32>::from("vcore:");
        vc.push_str(&String::<U32>::from(vcore)).ok();
        let mut vb = String::<U32>::from("vbat:");
        vb.push_str(&String::<U32>::from(vbat)).ok();
        let mut vi = String::<U32>::from("vio:");
        vi.push_str(&String::<U32>::from(vio)).ok();

        display.reset(&mut delay).ok();
        display.clear(Color::White).ok();
        Text::new("Hello!", Point::new(120, 15))
            .into_styled(text_style_red)
            .draw(&mut display)
            .ok();
        Text::new(status.as_str(), Point::new(70, 49))
            .into_styled(text_style_black)
            .draw(&mut display)
            .ok();
        Text::new(vc.as_str(), Point::new(105, 59))
            .into_styled(text_style_black)
            .draw(&mut display)
            .ok();
        Text::new(vb.as_str(), Point::new(105, 69))
            .into_styled(text_style_black)
            .draw(&mut display)
            .ok();
        Text::new(vi.as_str(), Point::new(105, 79))
            .into_styled(text_style_black)
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
        for _i in 0..6 {
            delay.delay_ms(30_000u32);
        }
    }
}
