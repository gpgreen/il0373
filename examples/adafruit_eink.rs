extern crate embedded_graphics;
extern crate il0373;
extern crate linux_embedded_hal;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, Triangle},
    text::{Alignment, Text},
};
use il0373::{Builder, Color, Dimensions, Display, GraphicDisplay, Interface, Rotation};
use linux_embedded_hal::{
    spidev::{SpiModeFlags, SpidevOptions},
    sysfs_gpio::Direction,
    SpidevBus, SysfsPin,
};

fn main() -> Result<(), std::convert::Infallible> {
    // Configure SPI
    let mut spi = SpidevBus::open("/dev/spidev0.0").expect("SPI device");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("SPI configuration");

    // https://pinout.xyz/pinout/inky_phat
    // Configure Digital I/O Pins
    let cs = SysfsPin::new(8); // BCM8
    cs.export().expect("cs export");
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).expect("CS Direction");
    cs.set_value(1).expect("CS Value set to 1");

    let busy = SysfsPin::new(17); // BCM17
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");

    let dc = SysfsPin::new(22); // BCM22
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out).expect("dc Direction");
    dc.set_value(1).expect("dc Value set to 1");

    let reset = SysfsPin::new(27); // BCM27
    reset.export().expect("reset export");
    while !reset.is_exported() {}
    reset
        .set_direction(Direction::Out)
        .expect("reset Direction");
    reset.set_value(1).expect("reset Value set to 1");

    let pins = (cs, busy, dc, reset);

    // need some buffers
    let mut black = [0u8; 212 * 104 / 8];
    let mut red = [0u8; 212 * 104 / 8];

    let config = Builder::new()
        .dimensions(Dimensions {
            rows: 212,
            cols: 104,
        })
        .rotation(Rotation::Rotate270)
        .build()
        .ok()
        .unwrap();

    // interface
    let controller = Interface::new(spi, pins);

    // display
    let display = Display::new(controller, config);

    // promote display to a GraphicDisplay
    let mut display = GraphicDisplay::new(display, &mut black, &mut red);

    // Create styles used by the drawing operations.
    let thin_stroke = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .build();
    let thick_stroke = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(3)
        .build();
    let fill = PrimitiveStyleBuilder::new()
        .fill_color(Color::Black)
        .build();
    let text_style = MonoTextStyle::new(&FONT_6X9, Color::Red);

    let yoffset = 10;

    // Draw a 3px wide outline around the display.
    Rectangle::new(
        Point::zero(),
        Size::new(display.size().width - 1, display.size().height - 1),
    )
    .into_styled(thick_stroke)
    .draw(&mut display)?;

    // Draw a triangle.
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(thin_stroke)
    .draw(&mut display)?;

    // Draw a filled square
    Rectangle::new(
        Point::new(52, yoffset),
        Size::new(52 + 16, 16 + yoffset as u32),
    )
    .into_styled(fill)
    .draw(&mut display)?;

    // Draw a circle with a 3px wide stroke.
    Circle::new(Point::new(96, yoffset + 8), 8)
        .into_styled(thick_stroke)
        .draw(&mut display)?;

    // Draw centered text.
    let text = "embedded-graphics";
    let width = text.len() as i32 * 6;
    Text::with_alignment(
        text,
        Point::new(64 - width / 2, 40),
        text_style,
        Alignment::Center,
    )
    .draw(&mut display)?;

    display.update().ok();
    display.deep_sleep().ok();

    Ok(())
}
