# IL0373 ePaper Display Driver

Rust driver for the [Dalian Good Displays IL0373][IL0373] e-Paper display (EPD)
controller, for use with [embedded-hal].

[![Build Status](https://travis-ci.org/gpgreen/il0373.svg?branch=main)](https://travis-ci.org/gpgreen/il0373)
[![codecov](https://codecov.io/gh/gpgreen/il0373/branch/main/graph/badge.svg)](https://codecov.io/gh/gpgreen/il0373)
[![crates.io](https://img.shields.io/crates/v/il0373.svg)](https://crates.io/crates/il0373)
[![Documentation](https://docs.rs/il0373/badge.svg)][crate-docs]

<img
src="https://raw.githubusercontent.com/gpgreen/il0373/main/nucleo-epaper.jpg"
width="640" alt="Photo of Adafruit 2.13 eink display on Nucleo-F103RB
dev board" />

## Description

This driver is intended to work on embedded platforms using the `embedded-hal`
trait library. It is `no_std` compatible, builds on stable Rust, and only uses
safe Rust. It supports the 4-wire SPI interface. A feature `sram`
allows use of the SRAM device on the Adafruit display to store the
display buffer instead of using RAM on the MCU. This feature is
demonstrated in the [Nucleo-F103RB] example.

## Tested Devices

The library has been tested and confirmed working on these devices:

* Adafruit 2.13 Tri-Color eInk [Adafruit Tri-Color eInk] on [Nucleo-F103RB] dev board (pictured above)

## Examples

**Note:** To build the examples the `examples` feature needs to be enabled. E.g.

    cargo build --release --examples --features examples

### Nucleo-F103RB with Adafruit 2.13 eInk

The [Nucleo-F103RB with Adafruit 2.13 example](https://github.com/gpgreen/il0373/tree/main/examples/stm32-eink),
shows how to display information on an [Adafruit Tri-Color eInk] using this crate. The photo
at the top of the page shows this example in action.

### Raspberry Pi with Adafruit 2.13 eInk

The [Raspberry Pi Adafruit 2.13 example](https://github.com/gpgreen/il0373/blob/main/examples/adafruit_eink.rs),
shows how to display information on an [Adafruit Tri-Color eInk] using this crate.

## Credits

* [embedded-graphics](https://crates.io/crates/embedded-graphics)
* [SSD1675 EPD driver](https://github.com/wezm/ssd1675)

## License

`il0373` is licensed under the `GNU General Public License v3.0 or later`. See [LICENSE](LICENSE) for more info.

[Adafruit Tri-Color eInk]: https://www.adafruit.com/product/4086
[crate-docs]: https://docs.rs/il0373
[cross]: https://github.com/rust-embedded/cross
[embedded-hal]: https://crates.io/crates/embedded-hal
[IL0373]: https://www.e-paper-display.com/download_detail/downloadsId%3d535.html
[LICENSE]: https://github.com/gpgreen/il0373/blob/main/LICENSE
[Nucleo-F103RB]: https://github.com/gpgreen/il0373/tree/main/examples/stm32-eink
