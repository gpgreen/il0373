# IL0373 ePaper Display Driver

Rust driver for the [Solomon Systech IL0373][IL0373] e-Paper display (EPD)
controller, for use with [embedded-hal].

[![Build Status](https://travis-ci.org/gpgreen/il0373.svg?branch=master)](https://travis-ci.org/gpgreen/il0373)
[![codecov](https://codecov.io/gh/gpgreen/il0373/branch/master/graph/badge.svg)](https://codecov.io/gh/gpgreen/il0373)
[![crates.io](https://img.shields.io/crates/v/ssd1675.svg)](https://crates.io/crates/ssd1675)
[![Documentation](https://docs.rs/ssd1675/badge.svg)][crate-docs]

<img src="https://raw.githubusercontent.com/gpgreen/il0373/master/IMG_2198.jpg" width="459" alt="Photo of Inky pHAT ePaper display on Raspberry Pi Zero W" />

## Description

This driver is intended to work on embedded platforms using the `embedded-hal`
trait library. It is `no_std` compatible, builds on stable Rust, and only uses
safe Rust. It supports the 4-wire SPI interface.

## Tested Devices

The library has been tested and confirmed working on these devices:

* Adafruit 2.13 Tri-Color eInk [Adafruit Tri-Color eInk] on Nucleo F103RB dev board (pictured above)

## Examples

**Note:** To build the examples the `examples` feature needs to be enabled. E.g.

    cargo build --release --examples --features examples

### Nucleo-F103RB with Adafruit 2.13 eInk

The [Raspberry Pi Inky pHAT
example](https://github.com/gpgreen/il0373/blob/master/examples/raspberry_pi_inky_phat.rs),
shows how to display information on an [Adafruit Tri-Color eInk] using this crate. The photo
at the top of the page shows this example in action.

## Credits

* [SSD1675 EPD driver](https://github.com/wezm/ssd1675)

## License

`il0373` is licensed under the `GNU General Public License v3.0 or later`. See [LICENSE](LICENSE) for more info.

[crate-docs]: https://docs.rs/il0373
[cross]: https://github.com/rust-embedded/cross
[embedded-hal]: https://crates.io/crates/embedded-hal
[Adafruit Tri-Color eInk]: https://www.adafruit.com/product/4086
[LICENSE]: https://github.com/gpgreen/il0373/blob/master/LICENSE
[IL0373]: http://www.solomon-systech.com/en/product/advanced-display/bistable-display-driver-ic/IL0373/
