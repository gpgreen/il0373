use core;
use interface::DisplayInterface;

trait Contains<C>
where
    C: Copy + PartialOrd,
{
    fn contains(&self, item: C) -> bool;
}

#[derive(Clone, Copy)]
pub enum DisplayResolution {
    R96x230,
    R96x252,
    R128x296,
    R160x296,
}

#[derive(Clone, Copy)]
pub enum DataPolarity {
    BWOnly,
    RedOnly,
    Both,
}

#[derive(Clone, Copy)]
pub enum DataInterval {
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
    V16,
    V17,
}

/// A command that can be issued to the controller.
#[derive(Clone, Copy)]
pub enum Command {
    /// Set the panel (PSR), overwritten by ResolutionSetting (TRES)
    PanelSetting(DisplayResolution),
    /// Gate scanning sequence and direction (PWR)
    PowerSetting(u8, u8, u8),
    /// Power OFF (POF)
    PowerOff,
    /// Power OFF Sequence
    /// Power ON (PON)
    PowerOn,
    /// Power ON Measure
    /// Booster Soft Start (BTST)
    BoosterSoftStart(u8, u8, u8),
    /// Deep Sleep
    DeepSleep,
    /// Data Start Transmission 1 (DTM1)
    /// Data Stop (DSP)
    DataStop,
    /// Display Refresh (DRF)
    DisplayRefresh,
    /// Data Start Transmission 2 (DTM2)
    /// VCOM LUT (LUTC)
    /// W2W LUT (LUTWW)
    /// B2W LUT (LUTBW/LUTR)
    /// B2B LUT (LUTBB/LUTB)
    /// PLL Control (PLL)
    PLLControl(u8),
    /// Temperature Sensor Calibration
    /// Temperature Sensor Enable
    /// Temperature Sensor Write
    /// Temperature Sensor Read
    /// VCOM and Data Interval Setting (CDI)
    VCOMDataIntervalSetting(u8, DataPolarity, DataInterval),
    /// Low Power Detection
    /// TCON Setting
    /// ResolutionSetting (TRES). Has higher priority than (PSR)
    ResolutionSetting(u8, u16),
    /// Revision
    /// Get Status
    /// Auto Measure VCOM
    /// VCOM Value
    /// VCM DC Setting (VDCS)
    VCMDCSetting(u8),
    // Partial Window
    // Partial In
    // Partial Out
    // Program Mode
    // Active Program
    // Read OTP Data
    // Cascade Setting
    // Force Temperature
}

/// Enumerates commands that can be sent to the controller that accept a slice argument buffer. This
/// is separated from `Command` so that the lifetime parameter of the argument buffer slice does
/// not pervade code which never invokes these two commands.
pub enum BufCommand<'buf> {
    /// Write to black/white RAM
    /// 1 = White
    /// 0 = Black
    WriteBlackData(&'buf [u8]),
    /// Write to red RAM
    /// 1 = Red
    /// 0 = Use contents of black/white RAM
    WriteRedData(&'buf [u8]),
}

/// Populates data buffer (array) and returns a pair (tuple) with command and
/// appropriately sized slice into populated buffer.
/// E.g.
///
/// let mut buf = [0u8; 5];
/// let (command, data) = pack!(buf, 0x3C, [0x12, 0x34]);
macro_rules! pack {
    ($buf:ident, $cmd:expr,[]) => {
        ($cmd, &$buf[..0])
    };
    ($buf:ident, $cmd:expr,[$arg0:expr]) => {{
        $buf[0] = $arg0;
        ($cmd, &$buf[..1])
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        ($cmd, &$buf[..2])
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr, $arg2:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        $buf[2] = $arg2;
        ($cmd, &$buf[..3])
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        $buf[2] = $arg2;
        $buf[3] = $arg3;
        ($cmd, &$buf[..4])
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        $buf[2] = $arg2;
        $buf[3] = $arg3;
        $buf[4] = $arg4;
        ($cmd, &$buf[..5])
    }};
}

impl Command {
    /// Execute the command, transmitting any associated data as well.
    pub fn execute<I: DisplayInterface>(&self, interface: &mut I) -> Result<(), I::Error> {
        use self::Command::*;

        let mut buf = [0u8; 5];
        let (command, data) = match *self {
            PanelSetting(resolution) => {
                let res = match resolution {
                    self::DisplayResolution::R96x230 => 0b0000_0000,
                    self::DisplayResolution::R96x252 => 0b0100_0000,
                    self::DisplayResolution::R128x296 => 0b1000_0000,
                    self::DisplayResolution::R160x296 => 0b1100_0000,
                };
                pack!(buf, 0x0, [res | 0b001111])
            }
            PowerSetting(vdh, vdl, vdhr) => {
                debug_assert!(vdh < 64);
                debug_assert!(vdl < 64);
                debug_assert!(vdhr < 64);
                pack!(buf, 0x1, [0x3, 0x0, vdh, vdl, vdhr])
            }
            PowerOff => {
                pack!(buf, 0x3, [])
            }
            PowerOn => {
                pack!(buf, 0x4, [])
            }
            BoosterSoftStart(phase_a, phase_b, phase_c) => {
                pack!(buf, 0x6, [phase_a, phase_b, phase_c])
            }
            DeepSleep => {
                pack!(buf, 0x8, [0xa5])
            }
            DataStop => {
                pack!(buf, 0x11, [])
            }
            DisplayRefresh => {
                pack!(buf, 0x12, [])
            }
            PLLControl(clock) => {
                pack!(buf, 0x30, [clock])
            }
            VCOMDataIntervalSetting(border_data, data_polarity, interval) => {
                debug_assert!(border_data < 4);
                let vbd = border_data << 6;
                let ddx = match data_polarity {
                    DataPolarity::BWOnly => 0b01_0000,
                    DataPolarity::RedOnly => 0b10_0000,
                    DataPolarity::Both => 0b11_0000,
                };
                let cdi = match interval {
                    DataInterval::V2 => 0b1111,
                    DataInterval::V3 => 0b1110,
                    DataInterval::V4 => 0b1101,
                    DataInterval::V5 => 0b1100,
                    DataInterval::V6 => 0b1011,
                    DataInterval::V7 => 0b1010,
                    DataInterval::V8 => 0b1001,
                    DataInterval::V9 => 0b1000,
                    DataInterval::V10 => 0b0111,
                    DataInterval::V11 => 0b0110,
                    DataInterval::V12 => 0b0101,
                    DataInterval::V13 => 0b0100,
                    DataInterval::V14 => 0b0011,
                    DataInterval::V15 => 0b0010,
                    DataInterval::V16 => 0b0001,
                    DataInterval::V17 => 0b0000,
                };
                pack!(buf, 0x50, [vbd | ddx | cdi])
            }
            ResolutionSetting(horiz, vertical) => {
                let vres_hi = ((vertical & 0x100) >> 8) as u8;
                let vres_lo = (vertical & 0xFF) as u8;
                pack!(buf, 0x61, [horiz, vres_hi, vres_lo])
            }
            VCMDCSetting(vcom_dc) => {
                debug_assert!(vcom_dc <= 0b11_1010);
                pack!(buf, 0x82, [vcom_dc])
            }
        };

        interface.send_command(command)?;
        if data.len() == 0 {
            Ok(())
        } else {
            interface.send_data(data)
        }
    }
}

impl<'buf> BufCommand<'buf> {
    /// Execute the command, transmitting the associated buffer as well.
    pub fn execute<I: DisplayInterface>(&self, interface: &mut I) -> Result<(), I::Error> {
        use self::BufCommand::*;

        let (command, data) = match self {
            WriteBlackData(buffer) => (0x10, buffer),
            WriteRedData(buffer) => (0x13, buffer),
        };

        interface.send_command(command)?;
        if data.len() == 0 {
            Ok(())
        } else {
            interface.send_data(data)
        }
    }
}

impl<C> Contains<C> for core::ops::Range<C>
where
    C: Copy + PartialOrd,
{
    fn contains(&self, item: C) -> bool {
        item >= self.start && item < self.end
    }
}

impl<C> Contains<C> for core::ops::RangeInclusive<C>
where
    C: Copy + PartialOrd,
{
    fn contains(&self, item: C) -> bool {
        item >= *self.start() && item <= *self.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockInterface {
        data: [u8; 256],
        offset: usize,
    }

    impl MockInterface {
        fn new() -> Self {
            MockInterface {
                data: [0; 256],
                offset: 0,
            }
        }

        fn write(&mut self, byte: u8) {
            self.data[self.offset] = byte;
            self.offset += 1;
        }

        fn data(&self) -> &[u8] {
            &self.data[0..self.offset]
        }
    }

    impl DisplayInterface for MockInterface {
        type Error = ();

        /// Send a command to the controller.
        ///
        /// Prefer calling `execute` on a [Commmand](../command/enum.Command.html) over calling this
        /// directly.
        fn send_command(&mut self, command: u8) -> Result<(), Self::Error> {
            self.write(command);
            Ok(())
        }

        /// Send data for a command.
        fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            for byte in data {
                self.write(*byte)
            }
            Ok(())
        }

        /// Reset the controller.
        fn reset<D: hal::blocking::delay::DelayMs<u8>>(&mut self, _delay: &mut D) {
            self.data = [0; 256];
            self.offset = 0;
        }

        /// Wait for the controller to indicate it is not busy.
        fn busy_wait(&self) {
            // nop
        }
    }

    #[test]
    fn test_command_execute() {
        let mut interface = MockInterface::new();
        let b = 0xCF;
        let command = Command::PanelSetting(DisplayResolution::R160x296);

        command.execute(&mut interface).unwrap();
        assert_eq!(interface.data(), &[0x00, b]);
    }
}
