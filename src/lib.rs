#![no_std]

//! IL0373 ePaper Display Driver
//!
//! For a complete example see
//! [the nucleo-f103rb example](https://github.com/gpgreen/stm32_eink/blob/master/src/main.rs).
//!
//! ### Usage
//!
//! To control a display you will need:
//!
//! * An [Interface] to the controller
//! * A [display configuration][Config]
//! * A [Display]
//!
//! The [Interface] captures the details of the hardware connection to
//! the IL0373 controller. This includes an SPI device and some GPIO
//! pins. The IL0373 can control many different displays that vary in
//! dimensions, rotation, and driving characteristics. The [Config]
//! captures these details.  To aid in constructing the [Config] there
//! is a [Builder] interface. Finally when you have an interface and a
//! [Config] a [Display] instance can be created.
//!
//!
//! Optionally the [Display] can be promoted to a [GraphicDisplay],
//! which allows it to use the functionality from the
//! [embedded-graphics crate][embedded-graphics]. The plain display
//! only provides the ability to update the display by passing
//! black/white and red buffers.
//!
//!
//! This driver can work with an SRAM device, to store the display
//! buffer on that device, to reduce the memory footprint on the
//! MCU. The SRAM device must be on the the same SPI port as the
//! il0373. For this option, use the feature `sram`. Instead of using
//! a [Interface] and [GraphicDisplay], use a [SpiBus], and an
//! associated [SramDisplayInterface], then a [SramGraphicDisplay].
//!
//!
//! To update the display you will typically follow this flow:
//!
//! 1. [reset](display/struct.Display.html#method.reset)
//! 1. [clear](graphics/struct.GraphicDisplay.html#method.clear)
//! 1. [update](graphics/struct.GraphicDisplay.html#method.update)
//! 1. [sleep](display/struct.Display.html#method.deep_sleep)
//!
//! [Interface]: interface/struct.Interface.html
//! [SpiBus]: interface/struct.SpiBus.html
//! [SramDisplayInterface]: interface/struct.SramDisplayInterface.html
//! [Display]: display/struct.Display.html
//! [GraphicDisplay]: display/struct.GraphicDisplay.html
//! [SramGraphicDisplay]: display/struct.SramGraphicDisplay.html
//! [Config]: config/struct.Config.html
//! [Builder]: config/struct.Builder.html
//! [embedded-graphics]: https://crates.io/crates/embedded-graphics

extern crate embedded_hal as hal;

#[cfg(test)]
#[macro_use]
extern crate std;

mod color;
pub mod command;
pub mod config;
pub mod display;
pub mod graphics;
pub mod interface;

pub use color::Color;
pub use config::Builder;
pub use display::{Dimensions, Display, Rotation};
pub use graphics::GraphicDisplay;
#[cfg(feature = "sram")]
pub use graphics::SramGraphicDisplay;
pub use interface::DisplayInterface;
pub use interface::Interface;
#[cfg(feature = "sram")]
pub use interface::SpiBus;
#[cfg(feature = "sram")]
pub use interface::SramDisplayInterface;
