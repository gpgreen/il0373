#!/bin/bash
cargo +nightly-2021-01-07 build --target avr-atmega328p.json -Z build-std=core --example arduino_uno --release
avrdude -v -patmega328p -carduino -P"/dev/ttyACM0" -D "-Uflash:w:target/avr-atmega328p/release/examples/arduino_uno.elf:e"
