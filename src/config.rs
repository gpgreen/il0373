use command::{Command, DisplayResolution};
use display::{self, Dimensions, Rotation};

/// Builder for constructing a display Config.
///
/// Dimensions must supplied, all other settings will use a default value if not supplied. However
/// it's likely that LUT values will need to be supplied to successfully use a display.
///
/// ### Example
///
/// ```
/// use il0373::{Builder, Dimensions, Rotation};
///
/// let config = Builder::new()
///     .dimensions(Dimensions {
///         rows: 216,
///         cols: 106,
///     })
///     .rotation(Rotation::Rotate270)
///     .build()
///     .expect("invalid configuration");
/// ```
pub struct Builder {
    power_setting: Command,
    booster_soft_start: Command,
    panel_setting: Command,
    pll: Command,
    dimensions: Option<Dimensions>,
    rotation: Rotation,
}

/// Error returned if Builder configuration is invalid.
///
/// Currently only returned if a configuration is built without dimensions.
#[derive(Debug)]
pub struct BuilderError {}

/// Display configuration.
///
/// Passed to Display::new. Use `Builder` to construct a `Config`.
pub struct Config {
    pub(crate) power_setting: Command,
    pub(crate) booster_soft_start: Command,
    pub(crate) panel_setting: Command,
    pub(crate) pll: Command,
    pub(crate) dimensions: Dimensions,
    pub(crate) rotation: Rotation,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            power_setting: Command::PowerSetting(0x2b, 0x2b, 0x9),
            booster_soft_start: Command::BoosterSoftStart(0x17, 0x17, 0x17),
            panel_setting: Command::PanelSetting(DisplayResolution::R160x296), // 0xCF
	    pll: Command::PLLControl(0x29),				  // 0x29
            dimensions: None,
            rotation: Rotation::default(),
        }
    }
}

impl Builder {
    /// Create a new Builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the panel
    ///
    /// Defaults to 160x296. Corresponds to command 0x0.
    pub fn panel_setting(self, res: DisplayResolution) -> Self {
        Self {
            panel_setting: Command::PanelSetting(res),
            ..self
        }
    }

    /// Set the power
    ///
    /// Defaults to 0x2b, 0x2b, 0x9. Corresponds to command 0x1.
    pub fn power_setting(self, vdh: u8, vdl: u8, vdhr: u8) -> Self {
        Self {
            power_setting: Command::PowerSetting(vdh, vdl, vdhr),
            ..self
        }
    }

    /// Set the booster power settings
    ///
    /// Defaults to 0x17, 0x17, 0x17. Corresponds to command 0x6.
    pub fn booster_soft_start(self, vhh: u8, vhl: u8, vhgl: u8) -> Self {
        Self {
            booster_soft_start: Command::BoosterSoftStart(vhh, vhl, vhgl),
            ..self
        }
    }

    /// Set the Clock
    ///
    /// Defaults to 0x29. Corresponds to command 0x30.
    pub fn pll(self, value: u8) -> Self {
        Self {
            pll: Command::PLLControl(value),
            ..self
        }
    }

    /// Set the display dimensions.
    ///
    /// There is no default for this setting. The dimensions must be set for the builder to
    /// successfully build a Config.
    pub fn dimensions(self, dimensions: Dimensions) -> Self {
        assert!(
            dimensions.cols % 8 == 0,
            "columns must be evenly divisible by 8"
        );
        assert!(
            dimensions.rows <= display::MAX_GATE_OUTPUTS,
            "rows must be less than MAX_GATE_OUTPUTS"
        );
        assert!(
            dimensions.cols <= display::MAX_SOURCE_OUTPUTS,
            "cols must be less than MAX_SOURCE_OUTPUTS"
        );

        Self {
            dimensions: Some(dimensions),
            ..self
        }
    }

    /// Set the display rotation.
    ///
    /// Defaults to no rotation (`Rotation::Rotate0`). Use this to translate between the physical
    /// rotation of the display and how the data is displayed on the display.
    pub fn rotation(self, rotation: Rotation) -> Self {
        Self { rotation, ..self }
    }

    /// Build the display Config.
    ///
    /// Will fail if dimensions are not set.
    pub fn build(self) -> Result<Config, BuilderError> {
        Ok(Config {
            power_setting: self.power_setting,
            booster_soft_start: self.booster_soft_start,
            panel_setting: self.panel_setting,
	    pll: self.pll,
            dimensions: self.dimensions.ok_or_else(|| BuilderError {})?,
            rotation: self.rotation,
        })
    }
}
