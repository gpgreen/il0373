use color::Color;
use core::ops::{Deref, DerefMut};
use display::{Display, Rotation};
use interface::DisplayInterface;

/// A display that holds buffers for drawing into and updating the display from.
///
/// When the `graphics` feature is enabled `GraphicDisplay` implements the `DrawTarget` trait from
/// [embedded-graphics](https://crates.io/crates/embedded-graphics). This allows basic shapes and
/// text to be drawn on the display.
pub struct GraphicDisplay<'a, I>
where
    I: DisplayInterface,
{
    display: Display<I>,
    black_buffer: &'a mut [u8],
    red_buffer: &'a mut [u8],
}

impl<'a, I> GraphicDisplay<'a, I>
where
    I: DisplayInterface,
{
    /// Promote a `Display` to a `GraphicDisplay`.
    ///
    /// B/W and Red buffers for drawing into must be supplied. These should be `rows` * `cols` / `8` in
    /// length.
    pub fn new(display: Display<I>, black_buffer: &'a mut [u8], red_buffer: &'a mut [u8]) -> Self {
        GraphicDisplay {
            display,
            black_buffer,
            red_buffer,
        }
    }

    /// update the display
    pub fn update(&mut self) -> Result<(), I::Error> {
        let buf_limit = ((self.rows() * self.cols() as u16) as u32 / 8) as u16;
        // update black
        self.display
            .interface()
            .epd_update_data(0, buf_limit, self.black_buffer)
            .ok();
        // update red
        self.display
            .interface()
            .epd_update_data(1, buf_limit, self.red_buffer)
            .ok();
        self.display.signal_update()
    }

    /// Clear the buffers, filling them a single color.
    fn clear(&mut self, color: Color) -> Result<(), core::convert::Infallible> {
        let (black, red) = match color {
            Color::White => (0xFF, 0xFF),
            Color::Black => (0x00, 0xFF),
            Color::Red => (0xFF, 0x00),
        };

        for byte in &mut self.black_buffer.iter_mut() {
            *byte = black; // background_color.get_byte_value();
        }

        // TODO: Combine loops
        for byte in &mut self.red_buffer.iter_mut() {
            *byte = red; // background_color.get_byte_value();
        }
        Ok(())
    }

    /// set a pixel to a color
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) -> Result<(), core::convert::Infallible> {
        let (index, bit) = rotation(
            x,
            y,
            self.cols() as u32,
            self.rows() as u32,
            self.rotation(),
        );
        let index = index as usize;

        match color {
            Color::Black => {
                self.black_buffer[index] &= !bit;
                self.red_buffer[index] |= bit;
            }
            Color::White => {
                self.black_buffer[index] |= bit;
                self.red_buffer[index] |= bit;
            }
            Color::Red => {
                self.black_buffer[index] |= bit;
                self.red_buffer[index] &= !bit;
            }
        }
        Ok(())
    }
}

impl<'a, I> Deref for GraphicDisplay<'a, I>
where
    I: DisplayInterface,
{
    type Target = Display<I>;

    fn deref(&self) -> &Display<I> {
        &self.display
    }
}

impl<'a, I> DerefMut for GraphicDisplay<'a, I>
where
    I: DisplayInterface,
{
    fn deref_mut(&mut self) -> &mut Display<I> {
        &mut self.display
    }
}

fn rotation(x: u32, y: u32, width: u32, height: u32, rotation: Rotation) -> (u32, u8) {
    match rotation {
        Rotation::Rotate0 => (x / 8 + (width / 8) * y, 0x80 >> (x % 8)),
        Rotation::Rotate90 => ((width - 1 - y) / 8 + (width / 8) * x, 0x01 << (y % 8)),
        Rotation::Rotate180 => (
            ((width / 8) * height - 1) - (x / 8 + (width / 8) * y),
            0x01 << (x % 8),
        ),
        Rotation::Rotate270 => (y / 8 + (height - 1 - x) * (width / 8), 0x80 >> (y % 8)),
    }
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics;
#[cfg(feature = "graphics")]
use self::embedded_graphics::prelude::*;

#[cfg(feature = "graphics")]
impl<'a, I> DrawTarget<Color> for GraphicDisplay<'a, I>
where
    I: DisplayInterface,
{
    type Error = core::convert::Infallible;

    /// override the clear method
    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        self.clear(color)?;
        Ok(())
    }

    /// required method
    fn draw_pixel(
        &mut self,
        Pixel(Point { x, y }, color): Pixel<Color>,
    ) -> Result<(), Self::Error> {
        let sz = self.size();
        let x = x as u32;
        let y = y as u32;
        if x < sz.width && y < sz.height {
            self.set_pixel(x, y, color)?;
        }
        Ok(())
    }

    /// required method
    fn size(&self) -> Size {
        match self.rotation() {
            Rotation::Rotate0 | Rotation::Rotate180 => {
                Size::new(self.cols().into(), self.rows().into())
            }
            Rotation::Rotate90 | Rotation::Rotate270 => {
                Size::new(self.rows().into(), self.cols().into())
            }
        }
    }
}

/// A display that uses SRAM for backing buffers for drawing into and updating the display from.
///
/// When the `graphics` feature is enabled `GraphicDisplaySram` implements the `DrawTarget` trait from
/// [embedded-graphics](https://crates.io/crates/embedded-graphics). This allows basic shapes and
/// text to be drawn on the display.
#[cfg(feature = "sram")]
pub struct GraphicDisplaySram<I>
where
    I: DisplayInterface,
{
    display: Display<I>,
    buffer_size: u16,
    black_address: u16,
    red_address: u16,
}

#[cfg(feature = "sram")]
impl<I> GraphicDisplaySram<I>
where
    I: DisplayInterface,
{
    /// Promote a `Display` to a `GraphicDisplaySram`.
    pub fn new(display: Display<I>) -> Self {
        let sz = ((display.rows() * display.cols() as u16) as u32 / 8) as u16;
        GraphicDisplaySram {
            display: display,
            buffer_size: sz,
            black_address: 0,
            red_address: sz,
        }
    }

    /// update the display
    pub fn update(&mut self) -> Result<(), I::Error> {
        // update black
        self.display
            .interface()
            .sram_epd_update_data(0, self.buffer_size, self.black_address)?;
        // update red
        self.display
            .interface()
            .sram_epd_update_data(1, self.buffer_size, self.red_address)?;
        self.display.signal_update()
    }

    /// Clear the buffers, filling them a single color.
    fn clear(&mut self, color: Color) -> Result<(), I::Error> {
        let (black, red) = match color {
            Color::White => (0xFF, 0xFF),
            Color::Black => (0x00, 0xFF),
            Color::Red => (0xFF, 0x00),
        };

        self.display
            .interface()
            .sram_clear(self.black_address, self.buffer_size, black)?;
        self.display
            .interface()
            .sram_clear(self.red_address, self.buffer_size, red)?;
        Ok(())
    }

    /// set a pixel to a color
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) -> Result<(), I::Error> {
        let (index, bit) = rotation(
            x,
            y,
            self.cols() as u32,
            self.rows() as u32,
            self.rotation(),
        );
        let index = index as u16;

        // get the existing buffer bytes
        let mut buf: [u8; 1] = [0];
        self.display
            .interface()
            .sram_read(index + self.black_address, &mut buf)?;
        let mut black = buf[0];
        self.display
            .interface()
            .sram_read(index + self.red_address, &mut buf)?;
        let mut red = buf[0];
        match color {
            Color::Black => {
                black &= !bit;
                red |= bit;
            }
            Color::White => {
                black |= bit;
                red |= bit;
            }
            Color::Red => {
                black |= bit;
                red &= !bit;
            }
        }
        // write the new buffer bytes
        buf[0] = black;
        self.display
            .interface()
            .sram_write(index + self.black_address, &mut buf)?;
        buf[0] = red;
        self.display
            .interface()
            .sram_write(index + self.red_address, &mut buf)?;
        Ok(())
    }
}

#[cfg(feature = "sram")]
impl<I> Deref for GraphicDisplaySram<I>
where
    I: DisplayInterface,
{
    type Target = Display<I>;

    fn deref(&self) -> &Display<I> {
        &self.display
    }
}

#[cfg(feature = "sram")]
impl<I> DerefMut for GraphicDisplaySram<I>
where
    I: DisplayInterface,
{
    fn deref_mut(&mut self) -> &mut Display<I> {
        &mut self.display
    }
}

#[cfg(all(feature = "graphics", feature = "sram"))]
impl<I> DrawTarget<Color> for GraphicDisplaySram<I>
where
    I: DisplayInterface,
{
    type Error = I::Error;

    /// required method
    fn draw_pixel(
        &mut self,
        Pixel(Point { x, y }, color): Pixel<Color>,
    ) -> Result<(), Self::Error> {
        let sz = self.size();
        let x = x as u32;
        let y = y as u32;
        if x < sz.width && y < sz.height {
            self.set_pixel(x, y, color)?;
        }
        Ok(())
    }

    /// override the default
    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        self.clear(color)
    }

    /// required method
    fn size(&self) -> Size {
        match self.rotation() {
            Rotation::Rotate0 | Rotation::Rotate180 => {
                Size::new(self.cols().into(), self.rows().into())
            }
            Rotation::Rotate90 | Rotation::Rotate270 => {
                Size::new(self.rows().into(), self.cols().into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use self::embedded_graphics::{egrectangle, primitive_style};
    use super::*;
    use {Builder, Color, Dimensions, Display, DisplayInterface, GraphicDisplay, Rotation};

    const ROWS: u16 = 3;
    const COLS: u8 = 8;
    const BUFFER_SIZE: usize = (ROWS * COLS as u16) as usize / 8;

    struct MockInterface {}
    struct MockError {}

    impl MockInterface {
        fn new() -> Self {
            MockInterface {}
        }
    }

    impl DisplayInterface for MockInterface {
        type Error = MockError;

        fn reset<D: hal::blocking::delay::DelayMs<u8>>(&mut self, _delay: &mut D) {}

        fn send_command(&mut self, _command: u8) -> Result<(), Self::Error> {
            Ok(())
        }

        fn send_data(&mut self, _data: &[u8]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn busy_wait(&self) {}
    }

    fn build_mock_display() -> Display<MockInterface> {
        let interface = MockInterface::new();
        let dimensions = Dimensions {
            rows: ROWS,
            cols: COLS,
        };

        let config = Builder::new()
            .dimensions(dimensions)
            .rotation(Rotation::Rotate270)
            .build()
            .expect("invalid config");
        Display::new(interface, config)
    }

    #[test]
    fn clear_white() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut red_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut red_buffer);
            display.clear(Color::White);
        }

        assert_eq!(black_buffer, [0xFF, 0xFF, 0xFF]);
        assert_eq!(red_buffer, [0x00, 0x00, 0x00]);
    }

    #[test]
    fn clear_black() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut red_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut red_buffer);
            display.clear(Color::Black);
        }

        assert_eq!(black_buffer, [0x00, 0x00, 0x00]);
        assert_eq!(red_buffer, [0x00, 0x00, 0x00]);
    }

    #[test]
    fn clear_red() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut red_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut red_buffer);
            display.clear(Color::Red);
        }

        assert_eq!(black_buffer, [0xFF, 0xFF, 0xFF]);
        assert_eq!(red_buffer, [0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn draw_rect_white() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut red_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut red_buffer);

            egrectangle!(
                top_left = (0, 0),
                bottom_right = (2, 2),
                style = primitive_style!(stroke_color = Color::White, stroke_width = 1)
            )
            .draw(&mut display)
            .unwrap()
        }

        #[rustfmt::skip]
        assert_eq!(black_buffer, [0b11100000,
                                  0b10100000,
                                  0b11100000]);

        #[rustfmt::skip]
        assert_eq!(red_buffer,   [0b00000000,
                                  0b00000000,
                                  0b00000000]);
    }

    #[test]
    fn draw_rect_red() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut red_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut red_buffer);

            egrectangle!(
                top_left = (0, 0),
                bottom_right = (2, 2),
                style = primitive_style!(stroke_color = Color::Red, stroke_width = 1)
            )
            .draw(&mut display)
            .unwrap();
        }

        #[rustfmt::skip]
        assert_eq!(black_buffer, [0b11100000,
                                  0b10100000,
                                  0b11100000]);

        #[rustfmt::skip]
        assert_eq!(red_buffer,   [0b11100000,
                                  0b10100000,
                                  0b11100000]);
    }
}
