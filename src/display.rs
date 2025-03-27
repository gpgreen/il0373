use crate::command::{Command, DataInterval, DataPolarity};
use crate::config::Config;
use crate::interface::DisplayInterface;
use hal;

// Max display resolution is 160x296
/// The maximum number of rows supported by the controller
pub const MAX_GATE_OUTPUTS: u16 = 296;
/// The maximum number of cols supported by the controller
pub const MAX_SOURCE_OUTPUTS: u8 = 160;

/// Represents the dimensions of the display.
pub struct Dimensions {
    /// The number of rows the display has.
    ///
    /// Must be less than or equal to MAX_GATE_OUTPUTS.
    pub rows: u16,
    /// The number of columns the display has.
    ///
    /// Must be less than or equal to MAX_SOURCE_OUTPUTS.
    pub cols: u8,
}

/// Represents the physical rotation of the display relative to the native orientation.
///
/// For example the native orientation of the Inky pHAT display is a tall (portrait) 104x212
/// display. `Rotate270` can be used to make it the right way up when attached to a Raspberry Pi
/// Zero with the ports on the top.
#[derive(Clone, Copy)]
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl Default for Rotation {
    /// Default is no rotation (`Rotate0`).
    fn default() -> Self {
        Rotation::Rotate0
    }
}

/// A configured display with a hardware interface.
pub struct Display<I>
where
    I: DisplayInterface,
{
    interface: I,
    config: Config,
}

impl<I> Display<I>
where
    I: DisplayInterface,
{
    /// Create a new display instance from a DisplayInterface and Config.
    ///
    /// The `Config` is typically created with `config::Builder`.
    pub fn new(interface: I, config: Config) -> Self {
        Self { interface, config }
    }

    /// Perform a hardware reset
    ///
    /// This will wake a controller that has previously entered deep sleep.
    pub fn reset<D: hal::delay::DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        self.interface.reset(delay);
        self.init(delay)
    }

    /// Initialize the controller
    fn init<D: hal::delay::DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        self.config.power_setting.execute(&mut self.interface)?;
        self.config
            .booster_soft_start
            .execute(&mut self.interface)?;
        Command::PowerOn.execute(&mut self.interface)?;
        delay.delay_ms(200);
        self.config.panel_setting.execute(&mut self.interface)?;
        Command::VCOMDataIntervalSetting(0x0, DataPolarity::Both, DataInterval::V10)
            .execute(&mut self.interface)?;
        self.config.pll.execute(&mut self.interface)?;
        Command::VCMDCSetting(0xA).execute(&mut self.interface)?;
        delay.delay_ms(20);
        Command::ResolutionSetting(self.config.dimensions.cols, self.config.dimensions.rows)
            .execute(&mut self.interface)?;
        Ok(())
    }

    /// Tell the hardware to update the display
    pub fn signal_update(&mut self) -> Result<(), I::Error> {
        // Kick off the display update
        Command::DisplayRefresh.execute(&mut self.interface)
    }

    fn power_down(&mut self) -> Result<(), I::Error> {
        self.interface.busy_wait();
        Command::VCOMDataIntervalSetting(0x0, DataPolarity::BWOnly, DataInterval::V10)
            .execute(&mut self.interface)?;
        Command::VCMDCSetting(0).execute(&mut self.interface)?;
        Command::PowerOff.execute(&mut self.interface)
    }

    /// Enter deep sleep mode.
    ///
    /// This puts the display controller into a low power mode. `reset` must be called to wake it
    /// from sleep.
    pub fn deep_sleep(&mut self) -> Result<(), I::Error> {
        self.power_down()?;
        Command::DeepSleep.execute(&mut self.interface)
    }

    /// Returns the number of rows the display has.
    pub fn rows(&self) -> u16 {
        self.config.dimensions.rows
    }

    /// Returns the number of columns the display has.
    pub fn cols(&self) -> u8 {
        self.config.dimensions.cols
    }

    /// Returns the rotation the display was configured with.
    pub fn rotation(&self) -> Rotation {
        self.config.rotation
    }

    /// returns the interface
    pub fn interface(&mut self) -> &mut I {
        &mut self.interface
    }
}
