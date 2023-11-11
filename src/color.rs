use embedded_graphics_core::pixelcolor::PixelColor;

/// Represents the state of a pixel in the display
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Black,
    White,
    Red,
}

impl PixelColor for Color {
    type Raw = ();
}
