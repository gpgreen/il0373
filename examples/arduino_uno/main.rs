#![no_std]
#![no_main]

extern crate il0373;
extern crate arduino_uno;
extern crate panic_halt;
extern crate ufmt;

use arduino_uno::prelude::*;
use arduino_uno::spi;
use arduino_uno::Delay;

use il0373::{Builder, Color, Dimensions, Display, GraphicDisplay, Rotation};

// Graphics
extern crate embedded_graphics;

use embedded_graphics::{
    prelude::*,
    fonts::{Font6x8, Text},
    style::TextStyleBuilder,
};

const ROWS: u16 = 212;
const COLS: u8 = 104;

#[arduino_uno::entry]
fn main() -> ! {
    let dp = arduino_uno::Peripherals::take().unwrap();

    let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

    // setup serial interface for text output
    let mut serial = arduino_uno::Serial::new(
	dp.USART0,
	pins.d0,
	pins.d1.into_output(&mut pins.ddr),
	115200.into_baudrate(),
    );

    // Configure SPI
    let (spi, _) = spi::Spi::new(
	dp.SPI,
	pins.d13.into_output(&mut pins.ddr),
	pins.d11.into_output(&mut pins.ddr),
	pins.d12.into_pull_up_input(&mut pins.ddr),
	pins.d10.into_output(&mut pins.ddr),
	spi::Settings::default(),
    );

    // https://learn.adafruit.com/adafruit-eink-display-breakouts/pinouts
    // Configure Digital I/O Pins
    let busy = pins.d7.into_pull_up_input(&mut pins.ddr);
    let dc = pins.d9.into_output(&mut pins.ddr);
    let reset = pins.d5.into_output(&mut pins.ddr);

    ufmt::uwriteln!(serial, "Pins configured\r").void_unwrap();
    
    // Initialise display controller
    let mut delay = Delay::new();
    
    let controller = il0373::Interface::new(spi, busy, dc, reset);
    ufmt::uwriteln!(serial, "Controller configured\r").void_unwrap();

//    let mut black_buffer = [0u8; ROWS as usize * COLS as usize / 8];
//    let mut red_buffer = [0u8; ROWS as usize * COLS as usize / 8];
    let mut black_buffer = [0u8; 2756];
    let mut red_buffer = [0u8; 2756];
    ufmt::uwriteln!(serial, "Buffers configured\r").void_unwrap();
    delay.delay_ms(800 as u16);
    
    let config = Builder::new()
        .dimensions(Dimensions {
            rows: ROWS,
            cols: COLS,
        })
        .rotation(Rotation::Rotate270)
        .build().ok().unwrap();

    let display = Display::new(controller, config);
    ufmt::uwriteln!(serial, "Display configured\r").void_unwrap();

    let mut display = GraphicDisplay::new(display, &mut black_buffer, &mut red_buffer);
    ufmt::uwriteln!(serial, "Graphics Display configured\r").void_unwrap();

    let text_style = TextStyleBuilder::new(Font6x8)
	.text_color(Color::Black)
	.background_color(Color::White)
	.build();
    ufmt::uwriteln!(serial, "Text Style configured\r").void_unwrap();
    
    // Main loop. Displays CPU temperature, uname, and uptime every minute with a red Raspberry Pi
    // header.
    loop {
        display.reset(&mut delay).ok();

        display.clear(Color::White);
	Text::new("CPU Temp:", Point::new(20, 30))
	    .into_styled(text_style)
	    .draw(&mut display).ok();

        display.update(&mut delay).ok();
        display.deep_sleep().ok();

        delay.delay_ms(1000 as u16);
    }
}

/*
fn read_cpu_temp() -> Result<f64, io::Error> {
    fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?
        .trim()
        .parse::<i32>()
        .map(|temp| temp as f64 / 1000.)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err)) 
}
*/
