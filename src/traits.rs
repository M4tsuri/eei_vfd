use core::marker::Sized;
use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::*,
};

/// All commands need to have this trait which gives the address of the command
/// which needs to be send via SPI with activated CommandsPin (Data/Command Pin in CommandMode)
pub(crate) trait Command {
    fn address(self) -> u8;
}

pub(crate) trait EEIInit<SPI, CS, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    /// This initialises the display and powers it up
    ///
    /// This function is already called from
    ///  - [new()](EEIDisplay::new())
    ///  - [`wake_up`]
    ///
    ///
    /// This function calls [reset](EEIDisplay::reset),
    /// so you don't need to call reset your self when trying to wake your device up
    /// after setting it to sleep.
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error>;
}

/// All the functions to interact with the EEI VFDs
pub trait EEIDisplay<SPI, CS, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    /// The Color Type used by the Display
    type DisplayColor;
    /// Creates a new driver from a SPI peripheral, CS Pin, Busy InputPin, DC
    ///
    /// This already initialises the device.
    fn new(
        spi: &mut SPI,
        cs: CS,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error>
    where
        Self: Sized;

    /// Let the device enter deep-sleep mode to save power.
    ///
    /// The deep sleep mode returns to standby with a hardware reset.
    fn sleep(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error>;

    /// Wakes the device up from sleep
    ///
    /// Also reintialises the device if necessary.
    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error>;

    /// Get the width of the display
    fn width(&self) -> u32;

    /// Get the height of the display
    fn height(&self) -> u32;

    /// set brightness of screen
    fn set_brightness(&mut self, spi: &mut SPI, val: u32) -> Result<(), SPI::Error>;

    /// Transmit a full frame to the SRAM of the EPD
    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error>;

    /// Transmits partial data to the SRAM of the EPD
    ///
    /// (x,y) is the top left corner
    ///
    /// BUFFER needs to be of size: width / 8 * height !
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error>;

    /// Clears the frame buffer on the VFD with the declared background color
    ///
    fn clear_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error>;
}
