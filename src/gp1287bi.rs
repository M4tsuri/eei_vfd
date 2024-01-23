//! A simple Driver for the gp1287bi display for SPI

use embedded_hal::{delay::DelayNs, digital::*, spi::SpiDevice};

use crate::interface::DisplayInterface;
use crate::traits::{EEIDisplay, EEIInit};

/// Width of gp1287bi in pixels
pub const WIDTH: u32 = 56;
/// Height of gp1287bi in pixels
pub const HEIGHT: u32 = 256;
/// Default Background Color (white)
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::Dark;
const NUM_DISPLAY_BITS: u32 = WIDTH * HEIGHT / 8;

use crate::color::Color;

pub(crate) mod command;

#[cfg(feature = "graphics")]
mod graphics;

use self::command::Command;
#[cfg(feature = "graphics")]
pub use self::graphics::Display256x50;

/// vfd1in02 driver
pub struct VFD256x50<SPI, CS, RST, DELAY> {
    interface: DisplayInterface<SPI, CS, RST, DELAY>,
}

impl<SPI, CS, RST, DELAY> EEIInit<SPI, CS, RST, DELAY> for VFD256x50<SPI, CS, RST, DELAY>
where
    SPI: SpiDevice,
    CS: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // Based on the spec (not public accessible)
        self.interface.reset(delay, 1);

        // software reset
        self.command(spi, Command::Reset)?;

        // set VFD mode
        self.cmd_with_args(spi, Command::VFDModeSetting, [0x02, 0x00])?;

        // set display area
        self.cmd_with_args(
            spi,
            Command::DisplayAreaSetting,
            [0xFF, 0x31, 0x00, 0x20, 0x00, 0x00, 0x80],
        )?;

        // set internal speed
        self.cmd_with_args(spi, Command::InternalSpeedSetting, [0x20, 0x3F, 0x00, 0x01])?;

        // set brightness
        self.set_brightness(spi, 0x30)?;

        // clear gram
        self.command(spi, Command::ClearGRAM)?;
        delay.delay_ms(10);

        // offset: no offset
        self.cmd_with_args(spi, Command::DisplayPosition1Offset, [0x00, 0x04])?;
        self.cmd_with_args(spi, Command::DisplayPosition2Offset, [0x00, 0x3c])?;

        // unknown
        self.cmd_with_args(spi, Command::UnknownInit, [0x00])?;

        // set display mode
        self.cmd_with_args(spi, Command::DisplayModeSetting, [0x00])?;

        // set frame sync
        self.cmd_with_args(spi, Command::FrameSyncSetting, [0x00])
    }
}

impl<SPI, CS, RST, DELAY> EEIDisplay<SPI, CS, RST, DELAY> for VFD256x50<SPI, CS, RST, DELAY>
where
    SPI: SpiDevice,
    CS: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    type DisplayColor = Color;
    fn new(spi: &mut SPI, cs: CS, rst: RST, delay: &mut DELAY) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(cs, rst);

        let mut vfd = VFD256x50 { interface };

        vfd.init(spi, delay)?;

        Ok(vfd)
    }

    fn set_brightness(&mut self, spi: &mut SPI, val: u32) -> Result<(), SPI::Error> {
        self.cmd_with_args(
            spi,
            Command::BrightnessSetting,
            [((val >> 8) as u8) & 0b11, val as u8],
        )
    }

    fn sleep(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.command(spi, Command::Sleep)
    }

    fn wake_up(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.command(spi, Command::WakeUp)
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        _delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.cmd_with_data(
            spi,
            Command::WriteGRAM,
            [0x00, 0x04, 0x37],
            buffer.iter().copied(),
        )
    }

    #[allow(unused)]
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        unimplemented!()
    }

    fn clear_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // Clear the black
        self.command(spi, Command::ClearGRAM)?;
        delay.delay_ms(10);
        Ok(())
    }
}

impl<SPI, CS, RST, DELAY> VFD256x50<SPI, CS, RST, DELAY>
where
    SPI: SpiDevice,
    CS: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn command(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.cmd_with_args(spi, command, [])
    }

    fn cmd_with_args(
        &mut self,
        spi: &mut SPI,
        command: Command,
        args: impl IntoIterator<Item = u8>,
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_arg(spi, command, args)
    }

    fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        args: impl IntoIterator<Item = u8>,
        data: impl IntoIterator<Item = u8>,
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, command, args, data)
    }
}
