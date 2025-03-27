use crate::color::Color;
use crate::display::{Display, Rotation};
use crate::interface::DisplayInterface;
use core::ops::{Deref, DerefMut};

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

// return index into array and bit position in that index
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
extern crate embedded_graphics_core;
#[cfg(feature = "graphics")]
use self::embedded_graphics_core::prelude::*;

#[cfg(feature = "graphics")]
impl<'a, I> DrawTarget for GraphicDisplay<'a, I>
where
    I: DisplayInterface,
{
    type Color = Color;
    type Error = core::convert::Infallible;

    /// override the clear method
    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        self.clear(color)?;
        Ok(())
    }

    /// required method
    fn draw_iter<ITR>(&mut self, pixels: ITR) -> Result<(), Self::Error>
    where
        ITR: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            self.set_pixel(point.x as u32, point.y as u32, color)?;
        }
        Ok(())
    }
}

impl<'a, I> OriginDimensions for GraphicDisplay<'a, I>
where
    I: DisplayInterface,
{
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
/// When the `graphics` feature is enabled `SramGraphicDisplay` implements the `DrawTarget` trait from
/// [embedded-graphics](https://crates.io/crates/embedded-graphics). This allows basic shapes and
/// text to be drawn on the display.
#[cfg(feature = "sram")]
pub struct SramGraphicDisplay<I>
where
    I: DisplayInterface,
{
    display: Display<I>,
    buffer_size: u16,
    black_address: u16,
    red_address: u16,
}

#[cfg(feature = "sram")]
impl<I> SramGraphicDisplay<I>
where
    I: DisplayInterface,
{
    /// Promote a `Display` to a `SramGraphicDisplay`.
    pub fn new(display: Display<I>) -> Self {
        let sz = ((display.rows() * display.cols() as u16) as u32 / 8) as u16;
        SramGraphicDisplay {
            display,
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
        let mut black: [u8; 1] = [0];
        self.display
            .interface()
            .sram_read(index + self.black_address, &mut black)?;
        let mut red: [u8; 1] = [0];
        self.display
            .interface()
            .sram_read(index + self.red_address, &mut red)?;
        match color {
            Color::Black => {
                black[0] &= !bit;
                red[0] |= bit;
            }
            Color::White => {
                black[0] |= bit;
                red[0] |= bit;
            }
            Color::Red => {
                black[0] |= bit;
                red[0] &= !bit;
            }
        }
        // write the new buffer bytes
        self.display
            .interface()
            .sram_write(index + self.black_address, &mut black)?;
        self.display
            .interface()
            .sram_write(index + self.red_address, &mut red)?;
        Ok(())
    }
}

#[cfg(feature = "sram")]
impl<I> Deref for SramGraphicDisplay<I>
where
    I: DisplayInterface,
{
    type Target = Display<I>;

    fn deref(&self) -> &Display<I> {
        &self.display
    }
}

#[cfg(feature = "sram")]
impl<I> DerefMut for SramGraphicDisplay<I>
where
    I: DisplayInterface,
{
    fn deref_mut(&mut self) -> &mut Display<I> {
        &mut self.display
    }
}

#[cfg(all(feature = "graphics", feature = "sram"))]
impl<I> DrawTarget for SramGraphicDisplay<I>
where
    I: DisplayInterface,
{
    type Color = Color;
    type Error = I::Error;

    /// required method
    fn draw_iter<ITR>(&mut self, pixels: ITR) -> Result<(), Self::Error>
    where
        ITR: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            self.set_pixel(point.x as u32, point.y as u32, color)?;
        }
        Ok(())
    }

    /// override the default
    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        self.clear(color)
    }
}

#[cfg(all(feature = "graphics", feature = "sram"))]
impl<I> OriginDimensions for SramGraphicDisplay<I>
where
    I: DisplayInterface,
{
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
    use super::*;
    use embedded_graphics::{
        prelude::*,
        primitives::{PrimitiveStyleBuilder, Rectangle},
    };
    use {Builder, Color, Dimensions, Display, DisplayInterface, GraphicDisplay};

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

        fn epd_update_data(
            &mut self,
            _layer: u8,
            _nbytes: u16,
            _buf: &[u8],
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        #[cfg(feature = "sram")]
        fn sram_read(&mut self, _address: u16, _data: &mut [u8]) -> Result<(), Self::Error> {
            Ok(())
        }

        #[cfg(feature = "sram")]
        fn sram_write(&mut self, _address: u16, _data: &[u8]) -> Result<(), Self::Error> {
            Ok(())
        }

        #[cfg(feature = "sram")]
        fn sram_clear(&mut self, _address: u16, _nbytes: u16, _val: u8) -> Result<(), Self::Error> {
            Ok(())
        }

        #[cfg(feature = "sram")]
        fn sram_epd_update_data(
            &mut self,
            _layer: u8,
            _nbytes: u16,
            _start_address: u16,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    fn build_mock_display() -> Display<MockInterface> {
        let interface = MockInterface::new();
        let dimensions = Dimensions {
            rows: ROWS,
            cols: COLS,
        };

        let config = Builder::new()
            .dimensions(dimensions)
            .build()
            .expect("invalid config");
        Display::new(interface, config)
    }

    #[test]
    fn rotation_0() {
        let rotation_data: [(u32, u32, u32, u8); 8] = [
            (0, 0, 0, 0x80),
            (103, 0, 12, 0x1),
            (103, 211, 2755, 0x1),
            (0, 211, 2743, 0x80),
            (1, 0, 0, 0x40),
            (2, 0, 0, 0x20),
            (3, 0, 0, 0x10),
            (4, 0, 0, 0x8),
        ];
        for (x, y, index, bit) in rotation_data.iter() {
            assert_eq!(
                (*index, *bit),
                super::rotation(*x, *y, 104, 212, Rotation::Rotate0)
            );
        }
    }

    #[test]
    fn rotation_270() {
        let rotation_data: [(u32, u32, u32, u8); 8] = [
            (0, 0, 2743, 0x80),
            (211, 0, 0, 0x80),
            (211, 103, 12, 0x1),
            (0, 103, 2755, 0x1),
            (7, 0, 2652, 0x80),
            (0, 1, 2743, 0x40),
            (0, 2, 2743, 0x20),
            (1, 8, 2731, 0x80),
        ];
        for (x, y, index, bit) in rotation_data.iter() {
            assert_eq!(
                (*index, *bit),
                super::rotation(*x, *y, 104, 212, Rotation::Rotate270)
            );
        }
    }

    #[test]
    fn clear_white() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut red_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut red_buffer);
            display.clear(Color::White).unwrap();
        }

        assert_eq!(black_buffer, [0xFF, 0xFF, 0xFF]);
        assert_eq!(red_buffer, [0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn clear_black() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut red_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut red_buffer);
            display.clear(Color::Black).unwrap();
        }

        assert_eq!(black_buffer, [0x00, 0x00, 0x00]);
        assert_eq!(red_buffer, [0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn clear_red() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut red_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut red_buffer);
            display.clear(Color::Red).unwrap();
        }

        assert_eq!(black_buffer, [0xFF, 0xFF, 0xFF]);
        assert_eq!(red_buffer, [0x00, 0x00, 0x00]);
    }

    #[test]
    fn draw_rect_white() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut red_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut red_buffer);
            // display.clear(Color::Black).unwrap();
            let style = PrimitiveStyleBuilder::new()
                .stroke_color(Color::White)
                .stroke_width(1)
                .build();
            Rectangle::new(Point::new(0, 0), Size::new(3, 3))
                .into_styled(style)
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

    #[test]
    fn draw_rect_red() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut red_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut red_buffer);

            let style = PrimitiveStyleBuilder::new()
                .stroke_color(Color::White)
                .stroke_width(1)
                .build();
            Rectangle::new(Point::new(0, 0), Size::new(3, 3))
                .into_styled(style)
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
