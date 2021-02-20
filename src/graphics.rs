use color::Color;
use core::ops::{Deref, DerefMut};
use display::{Display, Rotation};
use hal;
use interface::DisplayInterface;

/// A display that holds buffers for drawing into and updating the display from.
///
/// When the `graphics` feature is enabled `GraphicDisplay` implements the `Draw` trait from
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
    /// B/W and Red buffers for drawing into must be supplied. These should be `rows` * `cols` in
    /// length.
    pub fn new(display: Display<I>, black_buffer: &'a mut [u8], red_buffer: &'a mut [u8]) -> Self {
        GraphicDisplay {
            display,
            black_buffer,
            red_buffer,
        }
    }

    /// Update the display by writing the buffers to the controller.
    pub fn update<D: hal::blocking::delay::DelayMs<u8>>(
        &mut self,
        delay: &mut D,
    ) -> Result<(), I::Error> {
        self.display
            .update(self.black_buffer, self.red_buffer, delay)
    }

    /// Clear the buffers, filling them a single color.
    pub fn clear(&mut self, color: Color) {
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
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
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

    fn draw_pixel(
        &mut self,
        Pixel(Point { x, y }, color): Pixel<Color>,
    ) -> Result<(), Self::Error> {
        let sz = self.size();
        let x = x as u32;
        let y = y as u32;
        if x < sz.width && y < sz.height {
            self.set_pixel(x, y, color)
        }
        Ok(())
    }

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
