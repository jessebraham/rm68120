#![no_std]

use core::iter::once;

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
pub use display_interface_parallel_gpio::{Generic16BitBus, PGPIO16BitInterface};
use embedded_hal::blocking::delay::DelayMs;

type Result<T = (), E = DisplayError> = core::result::Result<T, E>;

// ---------------------------------------------------------------------------
// Orientation

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    Landscape,
    LandscapeFlipped,
    Portrait,
    PortraitFlipped,
}

impl Orientation {
    pub fn is_landscape(&self) -> bool {
        match self {
            Orientation::Landscape | Orientation::LandscapeFlipped => true,
            _ => false,
        }
    }

    pub fn is_portrait(&self) -> bool {
        match self {
            Orientation::Portrait | Orientation::PortraitFlipped => true,
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Command

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Nop                        = 0x0000,
    SoftReset                  = 0x0100,
    GetDisplayId               = 0x0400,
    GetDsiErr                  = 0x0500,
    GetPowerMode               = 0x0A00,
    GetAddressMode             = 0x0B00,
    GetPixelFormat             = 0x0C00,
    GetDisplayMode             = 0x0D00,
    GetSignalMode              = 0x0E00,
    GetDiagnosticResult        = 0x0F00,
    EnterSleepMode             = 0x1000,
    ExitSleepMode              = 0x1100,
    EnterPartialMode           = 0x1200,
    EnterNormalMode            = 0x1300,
    ExitInvertMode             = 0x2000,
    EnterInvertMode            = 0x2100,
    SetAllPixelOff             = 0x2200,
    SetAllPixelOn              = 0x2300,
    GammaCurveSelect           = 0x2600,
    SetDisplayOff              = 0x2800,
    SetDisplayOn               = 0x2900,
    SetColumnAddress           = 0x2A00,
    SetPageAddress             = 0x2B00,
    WriteMemoryStart           = 0x2C00,
    ReadMemoryStart            = 0x2E00,
    SetPartialArea             = 0x3000,
    SetScrollArea              = 0x3300,
    SetTearOff                 = 0x3400,
    SetTearOn                  = 0x3500,
    SetAddressMode             = 0x3600,
    ExitIdleMode               = 0x3800,
    EnterIdleMode              = 0x3900,
    SetPixelFormat             = 0x3A00,
    WriteMemoryContinue        = 0x3C00,
    ReadMemoryContinue         = 0x3E00,
    SetTearScanline            = 0x4400,
    GetScanline                = 0x4500,
    SetDeepStandbyMode         = 0x4F00,
    SetProfileValueForDisplay  = 0x5000,
    SetDisplayBrightness       = 0x5100,
    GetDisplayBrightness       = 0x5200,
    SetControlDisplay          = 0x5300,
    GetControlDisplay          = 0x5400,
    SetCabcMode                = 0x5500,
    GetCabcMode                = 0x5600,
    SetHysteresis              = 0x5700,
    SetGammaSetting            = 0x5800,
    GetFsValueMsbs             = 0x5A00,
    GetFsValueLsbs             = 0x5B00,
    GetMedianFilterFsValueMsbs = 0x5C00,
    GetMedianFilterFsValueLsbs = 0x5D00,
    SetCabcMinBrightness       = 0x5E00,
    GetCabcMinBrightness       = 0x5F00,
    SetLightSensorCoefficient  = 0x6500,
    GetLsccMsbs                = 0x6600,
    GetLsccLsbs                = 0x6700,
    GetBlackWhiteLowBit        = 0x7000,
    GetBkx                     = 0x7100,
    GetBky                     = 0x7200,
    GetWx                      = 0x7300,
    GetWy                      = 0x7400,
    GetRedGreenLowBit          = 0x7500,
    GetRx                      = 0x7600,
    GetRy                      = 0x7700,
    GetGx                      = 0x7800,
    GetGy                      = 0x7900,
    GetBlueAcolorLowBit        = 0x7A00,
    GetBx                      = 0x7B00,
    GetBy                      = 0x7C00,
    GetAx                      = 0x7D00,
    GetAy                      = 0x7E00,
    ReadDdbStart               = 0xA100,
    ReadDdbContinue            = 0xA800,
    ReadFirstChecksum          = 0xAA00,
    ReadContinueChecksum       = 0xAF00,
    ReadId1                    = 0xDA00,
    ReadId2                    = 0xDB00,
    ReadId3                    = 0xDC00,
}

// ---------------------------------------------------------------------------
// Driver

// NOTE:
//
// The following pins are required in order to drive the display using 8080
// parallell communication:
//
//   - 16 data lines       (LCD_D0 - LCD_D15)
//   - 1 bit clock signal  (LCD_WR)
//   - data/command signal (LCD_DC/LCD_RS)
//
// These pins should be encapsulated by the `I` (for interface) generic type.
pub struct Rm68120<I, D> {
    interface: I,
    delay: D,
    width: usize,
    height: usize,
    orientation: Orientation,
}

impl<I, D> Rm68120<I, D>
where
    I: WriteOnlyDataCommand,
    D: DelayMs<u32>,
{
    /// Construct the driver without any side effects
    pub fn new(
        interface: I,
        delay: D,
        width: usize,
        height: usize,
        orientation: Orientation,
    ) -> Self {
        Self {
            interface,
            delay,
            width,
            height,
            orientation,
        }
    }

    /// Enable the display
    pub fn enable(&mut self) -> Result<()> {
        self.command(Command::SetDisplayOn)
    }

    /// Disble the display
    pub fn disable(&mut self) -> Result<()> {
        self.command(Command::SetDisplayOff)
    }

    /// Get the current screen orientation
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Get the current screen width
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the current screen height
    pub fn height(&self) -> usize {
        self.height
    }

    // PRIVATE FUNCTIONS

    fn command(&mut self, command: Command) -> Result {
        self.interface
            .send_commands(DataFormat::U16BEIter(&mut once(command as u16)))?;

        Ok(())
    }

    fn write_iter<IT>(&mut self, data: IT) -> Result
    where
        IT: IntoIterator<Item = u16>,
    {
        // FIXME: do I need to send a command first?
        self.interface
            .send_data(DataFormat::U16BEIter(&mut data.into_iter()))?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Builder

const DEFAULT_WIDTH: usize = 800;
const DEFAULT_HEIGHT: usize = 480;

pub struct Rm68120Builder {
    width: usize,
    height: usize,
    orientation: Orientation,
}

impl Rm68120Builder {
    /// Construct a new driver builder
    pub fn new() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            orientation: Orientation::Landscape,
        }
    }

    /// Set the initial width and height of the display
    pub fn with_dimensions(&mut self, width: usize, height: usize) -> &mut Self {
        self.width = width;
        self.height = height;

        self
    }

    /// Set the initial orientation of the display
    pub fn with_orientation(&mut self, orientation: Orientation) -> &mut Self {
        self.orientation = orientation;

        self
    }

    /// Construct the driver using the provided interface
    pub fn build<I, D>(&self, interface: I, delay: D) -> Rm68120<I, D>
    where
        I: WriteOnlyDataCommand,
        D: DelayMs<u32>,
    {
        Rm68120::new(interface, delay, self.width, self.height, self.orientation)
    }
}
