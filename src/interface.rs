use command::BufCommand;
use core::fmt::Debug;
use hal;

// Sample code from Good Displays says to hold for 10ms
const RESET_DELAY_MS: u8 = 10;

/// Trait implemented by displays to provide implementation of core functionality.
pub trait DisplayInterface {
    type Error;

    /// Send a command to the controller.
    ///
    /// Prefer calling `execute` on a [Commmand](../command/enum.Command.html) over calling this
    /// directly.
    fn send_command(&mut self, command: u8) -> Result<(), Self::Error>;

    /// Send data for a command.
    fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Reset the controller.
    fn reset<D: hal::blocking::delay::DelayMs<u8>>(&mut self, delay: &mut D);

    /// Wait for the controller to indicate it is not busy.
    fn busy_wait(&self);

    //----- Following is only for buffers in RAM
    /// copy display buffer data to epd
    fn epd_update_data(&mut self, layer: u8, nbytes: u16, buf: &[u8]) -> Result<(), Self::Error>;

    //----- Following is only for buffers in SRAM
    /// copy display buffer data to epd from sram
    #[cfg(feature = "sram")]
    fn sram_epd_update_data(
        &mut self,
        layer: u8,
        nbytes: u16,
        start_address: u16,
    ) -> Result<(), Self::Error>;

    /// read data from sram
    #[cfg(feature = "sram")]
    fn sram_read(&mut self, address: u16, data: &mut [u8]) -> Result<(), Self::Error>;

    /// write data to sram
    #[cfg(feature = "sram")]
    fn sram_write(&mut self, address: u16, data: &[u8]) -> Result<(), Self::Error>;

    /// set area in sram to a value
    #[cfg(feature = "sram")]
    fn sram_clear(&mut self, address: u16, nbytes: u16, val: u8) -> Result<(), Self::Error>;
}

/// The hardware interface to a display.
///
/// ### Example
///
/// This example uses the Linux implementation of the embedded HAL traits to build a display
/// interface. For a complete example see [the Raspberry Pi Inky pHAT example](https://github.com/wezm/ssd1675/blob/master/examples/raspberry_pi_inky_phat.rs).
///
/// ```ignore
/// extern crate linux_embedded_hal;
/// use linux_embedded_hal::spidev::{self, SpidevOptions};
/// use linux_embedded_hal::sysfs_gpio::Direction;
/// use linux_embedded_hal::Delay;
/// use linux_embedded_hal::{Pin, Spidev};
///
/// extern crate ssd1675;
/// use ssd1675::{Builder, Color, Dimensions, Display, GraphicDisplay, Rotation};
///
/// // Configure SPI
/// let mut spi = Spidev::open("/dev/spidev0.0").expect("SPI device");
/// let options = SpidevOptions::new()
///     .bits_per_word(8)
///     .max_speed_hz(4_000_000)
///     .mode(spidev::SPI_MODE_0)
///     .build();
/// spi.configure(&options).expect("SPI configuration");
///
/// // https://pinout.xyz/pinout/inky_phat
/// // Configure Digital I/O Pins
/// let cs = Pin::new(8); // BCM8
/// cs.export().expect("cs export");
/// while !cs.is_exported() {}
/// cs.set_direction(Direction::Out).expect("CS Direction");
/// cs.set_value(1).expect("CS Value set to 1");
///
/// let busy = Pin::new(17); // BCM17
/// busy.export().expect("busy export");
/// while !busy.is_exported() {}
/// busy.set_direction(Direction::In).expect("busy Direction");
///
/// let dc = Pin::new(22); // BCM22
/// dc.export().expect("dc export");
/// while !dc.is_exported() {}
/// dc.set_direction(Direction::Out).expect("dc Direction");
/// dc.set_value(1).expect("dc Value set to 1");
///
/// let reset = Pin::new(27); // BCM27
/// reset.export().expect("reset export");
/// while !reset.is_exported() {}
/// reset
///     .set_direction(Direction::Out)
///     .expect("reset Direction");
/// reset.set_value(1).expect("reset Value set to 1");
///
/// // Build the interface from the pins and SPI device
/// let controller = ssd1675::Interface::new(spi, cs, busy, dc, reset);

pub struct Interface<SPI, CS, BUSY, DC, RESET> {
    /// SPI interface
    spi: SPI,
    /// Chip Select, low active (output)
    cs: CS,
    /// Active low busy pin (input)
    busy: BUSY,
    /// Data/Command Control Pin (High for data, Low for command) (output)
    dc: DC,
    /// Pin for resetting the controller (output)
    reset: RESET,
}

impl<SPI, CS, BUSY, DC, RESET> Interface<SPI, CS, BUSY, DC, RESET>
where
    SPI: hal::blocking::spi::Write<u8>,
    CS: hal::digital::v2::OutputPin,
    BUSY: hal::digital::v2::InputPin,
    DC: hal::digital::v2::OutputPin,
    RESET: hal::digital::v2::OutputPin,
{
    /// Create a new Interface from embedded hal traits.
    pub fn new(spi: SPI, pins: (CS, BUSY, DC, RESET)) -> Self {
        Self {
            spi: spi,
            cs: pins.0,
            busy: pins.1,
            dc: pins.2,
            reset: pins.3,
        }
    }

    /// release the spi and pins
    pub fn release(self) -> (SPI, (CS, BUSY, DC, RESET)) {
        (self.spi, (self.cs, self.busy, self.dc, self.reset))
    }

    fn write(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.cs.set_low().ok();
        // Linux has a default limit of 4096 bytes per SPI transfer
        // https://github.com/torvalds/linux/blob/ccda4af0f4b92f7b4c308d3acc262f4a7e3affad/drivers/spi/spidev.c#L93
        if cfg!(target_os = "linux") {
            for data_chunk in data.chunks(4096) {
                self.spi.write(data_chunk)?;
            }
        } else {
            self.spi.write(data)?;
        }

        // Release the controller
        self.cs.set_high().ok();

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RESET> DisplayInterface for Interface<SPI, CS, BUSY, DC, RESET>
where
    SPI: hal::blocking::spi::Write<u8>,
    CS: hal::digital::v2::OutputPin,
    CS::Error: Debug,
    BUSY: hal::digital::v2::InputPin,
    DC: hal::digital::v2::OutputPin,
    DC::Error: Debug,
    RESET: hal::digital::v2::OutputPin,
    RESET::Error: Debug,
{
    type Error = SPI::Error;

    fn reset<D: hal::blocking::delay::DelayMs<u8>>(&mut self, delay: &mut D) {
        // do a hardware reset 3 times
        self.reset.set_low().unwrap();
        delay.delay_ms(RESET_DELAY_MS);
        self.reset.set_high().unwrap();
        delay.delay_ms(RESET_DELAY_MS);
        self.reset.set_low().unwrap();
        delay.delay_ms(RESET_DELAY_MS);
        self.reset.set_high().unwrap();
        delay.delay_ms(RESET_DELAY_MS);
        self.reset.set_low().unwrap();
        delay.delay_ms(RESET_DELAY_MS);
        self.reset.set_high().unwrap();
        delay.delay_ms(RESET_DELAY_MS);
    }

    fn send_command(&mut self, command: u8) -> Result<(), Self::Error> {
        self.dc.set_low().unwrap();
        self.write(&[command])?;
        self.dc.set_high().unwrap();
        Ok(())
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_high().unwrap();
        self.write(data)
    }

    #[cfg(feature = "sram")]
    fn sram_read(&mut self, _address: u16, _data: &mut [u8]) -> Result<(), Self::Error> {
        panic!()
    }

    #[cfg(feature = "sram")]
    fn sram_write(&mut self, _address: u16, _data: &[u8]) -> Result<(), Self::Error> {
        panic!()
    }

    #[cfg(feature = "sram")]
    fn sram_clear(&mut self, _address: u16, _nbytes: u16, _val: u8) -> Result<(), Self::Error> {
        panic!()
    }

    #[cfg(feature = "sram")]
    fn sram_epd_update_data(
        &mut self,
        _layer: u8,
        _nbytes: u16,
        _start_address: u16,
    ) -> Result<(), Self::Error> {
        panic!()
    }

    fn epd_update_data(&mut self, layer: u8, nbytes: u16, buf: &[u8]) -> Result<(), Self::Error> {
        let sz: usize = nbytes.into();
        if layer == 0 {
            BufCommand::WriteBlackData(&buf[..sz]).execute(self)
        } else {
            BufCommand::WriteRedData(&buf[..sz]).execute(self)
        }
    }

    fn busy_wait(&self) {
        while match self.busy.is_high() {
            Ok(x) => x,
            _ => false,
        } {}
    }
}

//const MCPSRAM_RDSR: u8 = 0x05;
#[cfg(feature = "sram")]
const MCPSRAM_READ: u8 = 0x03;
#[cfg(feature = "sram")]
const MCPSRAM_WRITE: u8 = 0x02;
#[cfg(feature = "sram")]
const MCPSRAM_WRSR: u8 = 0x01;
#[cfg(feature = "sram")]
const K640_SEQUENTIAL_MODE: u8 = 1 << 6;

#[cfg(feature = "sram")]
pub struct SpiBus<SPI, EPDCS, SRAMCS> {
    spi: SPI,
    epd_cs: EPDCS,
    sram_cs: SRAMCS,
}

#[cfg(feature = "sram")]
impl<SPI, EPDCS, SRAMCS> SpiBus<SPI, EPDCS, SRAMCS>
where
    SPI: hal::spi::FullDuplex<u8>,
    EPDCS: hal::digital::v2::OutputPin,
    SRAMCS: hal::digital::v2::OutputPin,
{
    /// create a new SpiBus from embedded hal traits
    pub fn new(spi: SPI, mut pins: (EPDCS, SRAMCS)) -> SpiBus<SPI, EPDCS, SRAMCS> {
        pins.0.set_high().ok();
        pins.1.set_high().ok();
        SpiBus {
            spi: spi,
            epd_cs: pins.0,
            sram_cs: pins.1,
        }
    }

    /// release the spi and cs pins
    pub fn release(self) -> (SPI, (EPDCS, SRAMCS)) {
        (self.spi, (self.epd_cs, self.sram_cs))
    }

    /// initialize sram device
    pub fn sram_init(&mut self) -> Result<(), SPI::Error> {
        self.sram_cs.set_low().ok();
        self.write(&[0xFF, 0xFF, 0xFF])?;
        self.sram_cs.set_high().ok();
        Ok(())
    }

    /// set sram device to sequential
    pub fn sram_seq(&mut self) -> Result<(), SPI::Error> {
        self.sram_cs.set_low().ok();
        self.write(&[MCPSRAM_WRSR, K640_SEQUENTIAL_MODE])?;
        self.sram_cs.set_high().ok();
        Ok(())
    }

    /// write to the sram
    pub fn sram_write(&mut self, address: u16, data: &[u8]) -> Result<(), SPI::Error> {
        self.sram_cs.set_low().ok();
        let cmd: [u8; 3] = [MCPSRAM_WRITE, (address >> 8) as u8, (address & 0xFF) as u8];
        self.write(&cmd)?;
        self.write(data)?;
        self.sram_cs.set_high().ok();
        Ok(())
    }

    /// read the sram
    pub fn sram_read(&mut self, address: u16, data: &mut [u8]) -> Result<(), SPI::Error> {
        self.sram_cs.set_low().ok();
        let cmd: [u8; 3] = [MCPSRAM_READ, (address >> 8) as u8, (address & 0xFF) as u8];
        self.write(&cmd)?;
        self.transfer(data)?;
        self.sram_cs.set_high().ok();
        Ok(())
    }

    /// erase buffer in sram
    pub fn sram_erase(&mut self, address: u16, len: u16, val: u8) -> Result<(), SPI::Error> {
        self.sram_cs.set_low().ok();
        let cmd: [u8; 3] = [MCPSRAM_WRITE, (address >> 8) as u8, (address & 0xFF) as u8];
        self.write(&cmd)?;
        for _i in 0..len {
            nb::block!(self.spi.send(val))?;
            nb::block!(self.spi.read())?;
        }
        self.sram_cs.set_high().ok();
        Ok(())
    }

    /// start a buffer transfer from the SRAM to the EPD. This needs the beginning address
    /// in the SRAM, and the location where they will be sent in the EPD.
    /// While the location is sent to the EPD, the first byte will be pulled from
    /// the SRAM at the address specified, this is passed to the sram_epd_move_body fn
    pub fn sram_epd_move_header(
        &mut self,
        address: u16,
        epd_location: u8,
    ) -> Result<u8, SPI::Error> {
        self.sram_cs.set_low().ok();
        // send address and get first byte of data
        let cmd: [u8; 3] = [MCPSRAM_READ, (address >> 8) as u8, (address & 0xFF) as u8];
        self.write(&cmd)?;
        self.epd_cs.set_low().ok();
        nb::block!(self.spi.send(epd_location))?;
        let c = nb::block!(self.spi.read())?;
        Ok(c)
    }

    /// given the first byte from SRAM from sram_epd_move_header, transfer the rest
    /// of the bytes to the EPD. These functions are split up because another pin
    /// must be pulled low between them in the protocol
    pub fn sram_epd_move_body(&mut self, ch: u8, data_len: u16) -> Result<(), SPI::Error> {
        let mut c = ch;
        for _i in 0..data_len {
            nb::block!(self.spi.send(c))?;
            c = nb::block!(self.spi.read())?;
        }
        self.epd_cs.set_high().ok();
        self.sram_cs.set_high().ok();
        Ok(())
    }
    /// write to the epaper display
    pub fn epd_write(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.epd_cs.set_low().ok();
        for byte in data.iter() {
            nb::block!(self.spi.send(*byte))?;
            nb::block!(self.spi.read())?;
        }
        self.epd_cs.set_high().ok();
        Ok(())
    }

    /// low level method to transfer a data array, used by sram and epaper devices
    fn transfer(&mut self, data: &mut [u8]) -> Result<(), SPI::Error> {
        for byte in data.iter_mut() {
            nb::block!(self.spi.send(*byte))?;
            *byte = nb::block!(self.spi.read())?;
        }
        Ok(())
    }

    /// low level method to transfer a data array, used by sram and epaper devices
    fn write(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        for byte in data.iter() {
            nb::block!(self.spi.send(*byte))?;
            nb::block!(self.spi.read())?;
        }
        Ok(())
    }
}

#[cfg(feature = "sram")]
pub struct SramDisplayInterface<SPI, EPDCS, SRAMCS, BUSY, DC, RESET> {
    spi_bus: SpiBus<SPI, EPDCS, SRAMCS>,
    busy: BUSY,
    dc: DC,
    reset: RESET,
}

#[cfg(feature = "sram")]
impl<SPI, EPDCS, SRAMCS, BUSY, DC, RESET> SramDisplayInterface<SPI, EPDCS, SRAMCS, BUSY, DC, RESET>
where
    SPI: hal::spi::FullDuplex<u8>,
    EPDCS: hal::digital::v2::OutputPin,
    SRAMCS: hal::digital::v2::OutputPin,
    BUSY: hal::digital::v2::InputPin,
    DC: hal::digital::v2::OutputPin,
    RESET: hal::digital::v2::OutputPin,
{
    /// create a display interface from the embedded hal
    pub fn new(
        spi_bus: SpiBus<SPI, EPDCS, SRAMCS>,
        mut pins: (BUSY, DC, RESET),
    ) -> SramDisplayInterface<SPI, EPDCS, SRAMCS, BUSY, DC, RESET> {
        // dc inactive low
        pins.1.set_low().ok();
        // reset inactive high
        pins.2.set_high().ok();
        SramDisplayInterface {
            spi_bus: spi_bus,
            busy: pins.0,
            dc: pins.1,
            reset: pins.2,
        }
    }

    /// release the spibus and all the associated pins
    pub fn release(self) -> (SpiBus<SPI, EPDCS, SRAMCS>, (BUSY, DC, RESET)) {
        (self.spi_bus, (self.busy, self.dc, self.reset))
    }
}

#[cfg(feature = "sram")]
impl<SPI, EPDCS, SRAMCS, BUSY, DC, RESET> DisplayInterface
    for SramDisplayInterface<SPI, EPDCS, SRAMCS, BUSY, DC, RESET>
where
    SPI: hal::spi::FullDuplex<u8>,
    EPDCS: hal::digital::v2::OutputPin,
    SRAMCS: hal::digital::v2::OutputPin,
    BUSY: hal::digital::v2::InputPin,
    DC: hal::digital::v2::OutputPin,
    RESET: hal::digital::v2::OutputPin,
{
    type Error = SPI::Error;

    fn send_command(&mut self, command: u8) -> Result<(), Self::Error> {
        self.dc.set_low().ok();
        self.spi_bus.epd_write(&[command])
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_high().ok();
        self.spi_bus.epd_write(data)
    }

    fn reset<D: hal::blocking::delay::DelayMs<u8>>(&mut self, delay: &mut D) {
        // setup the sram
        self.spi_bus.sram_init().ok();

        // do a hardware reset 3 times
        self.reset.set_low().ok();
        delay.delay_ms(RESET_DELAY_MS);
        self.reset.set_high().ok();
        delay.delay_ms(RESET_DELAY_MS);
        self.reset.set_low().ok();
        delay.delay_ms(RESET_DELAY_MS);
        self.reset.set_high().ok();
        delay.delay_ms(RESET_DELAY_MS);
        self.reset.set_low().ok();
        delay.delay_ms(RESET_DELAY_MS);
        self.reset.set_high().ok();
        delay.delay_ms(RESET_DELAY_MS);

        self.spi_bus.sram_seq().ok();
    }

    fn busy_wait(&self) {
        while match self.busy.is_high() {
            Ok(x) => x,
            _ => false,
        } {}
    }

    fn epd_update_data(
        &mut self,
        _layer: u8,
        _nbytes: u16,
        _buf: &[u8],
    ) -> Result<(), Self::Error> {
        panic!()
    }

    fn sram_read(&mut self, address: u16, data: &mut [u8]) -> Result<(), Self::Error> {
        self.spi_bus.sram_read(address, data)
    }

    fn sram_write(&mut self, address: u16, data: &[u8]) -> Result<(), Self::Error> {
        self.spi_bus.sram_write(address, data)
    }

    fn sram_clear(&mut self, address: u16, nbytes: u16, val: u8) -> Result<(), Self::Error> {
        self.spi_bus.sram_erase(address, nbytes, val)
    }

    fn sram_epd_update_data(
        &mut self,
        layer: u8,
        nbytes: u16,
        start_address: u16,
    ) -> Result<(), Self::Error> {
        let epd_location = if layer == 0 { 0x10 } else { 0x13 };
        self.dc.set_low().ok();
        let ch = self
            .spi_bus
            .sram_epd_move_header(start_address, epd_location)?;
        self.dc.set_high().ok();
        self.spi_bus.sram_epd_move_body(ch, nbytes)
    }
}
