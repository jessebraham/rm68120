#![no_std]

use display_interface::{DisplayError, WriteOnlyDataCommand};
pub use display_interface_parallel_gpio::{Generic16BitBus, PGPIO16BitInterface};
use embedded_hal::blocking::delay::DelayMs;

type Result<T, E = DisplayError> = core::result::Result<T, E>;

// ---------------------------------------------------------------------------
// Orientation

#[derive(Clone, Copy)]
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
