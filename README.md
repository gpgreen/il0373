# IL0373 ePaper Display Driver

[![Crates.io](https://img.shields.io/crates/d/il0373.svg)](https://crates.io/crates/il0373)
[![Crates.io](https://img.shields.io/crates/v/il0373.svg)](https://crates.io/crates/il0373)
[![docs.rs](https://docs.rs/il0373/badge.svg)](https://docs.rs/il0373/)
[![dependency status](https://deps.rs/repo/github/[gpgreen/il0373/status.svg)](https://deps.rs/repo/github/gpgreen/il0373)

Rust driver for the [Dalian Good Displays IL0373][IL0373] e-Paper
display (EPD) controller. While it can be used stand-alone, it can
also support the [embedded-graphics] crate.

<img
src="https://raw.githubusercontent.com/gpgreen/il0373/main/nucleo-epaper.jpg"
width="640" alt="Photo of Adafruit 2.13 eink display on Nucleo-F103RB
dev board" />

## Description

This driver is intended to work on embedded platforms. It is `no_std`
compatible, builds on stable Rust, and only uses safe Rust. A feature
'graphics' enables support for the [embedded-graphics] library if that
is desired. It supports the 4-wire SPI interface. A feature `sram`
allows use of the SRAM device on the Adafruit display to store the
display buffer instead of using RAM on the MCU. This feature is
demonstrated in the [Nucleo-F103RB] example.

## Tested Devices

The library has been tested and confirmed working on these devices:

* Adafruit 2.13 Tri-Color eInk [Adafruit Tri-Color eInk] on [Nucleo-F103RB] dev board (pictured above)
* Adafruit 2.13 Tri-Color eInk [Adafruit Tri-Color eInk] on Raspberry Pi

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

`il0373` is licensed under either of

- Apache License, Version 2.0 [LICENSE-APACHE](LICENSE-APACHE)
- MIT License [LICENSE-MIT](LICENSE-MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[Adafruit Tri-Color eInk]: https://www.adafruit.com/product/4086
[cross]: https://github.com/rust-embedded/cross
[embedded-hal]: https://crates.io/crates/embedded-hal
[IL0373]: https://www.e-paper-display.com/download_detail/downloadsId%3d535.html
[Nucleo-F103RB]: https://github.com/gpgreen/il0373/tree/main/examples/stm32-eink
