use crate::traits::Command;
use core::marker::PhantomData;
use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::*,
};

/// The Connection Interface of all (?) EEI VFD
///
pub(crate) struct DisplayInterface<SPI, CS, RST, DELAY> {
    /// SPI
    _spi: PhantomData<SPI>,
    /// DELAY
    _delay: PhantomData<DELAY>,
    /// CS for SPI
    cs: CS,
    /// Pin for Resetting
    rst: RST,
}

impl<SPI, CS, RST, DELAY> DisplayInterface<SPI, CS, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    pub fn new(cs: CS, rst: RST) -> Self {
        DisplayInterface {
            _spi: PhantomData::default(),
            _delay: PhantomData::default(),
            cs,
            rst,
        }
    }

    /// Basic function for sending [Commands](Command).
    ///
    fn cmd<T: Command>(&mut self, spi: &mut SPI, command: T) -> Result<(), SPI::Error> {
        // Transfer the command over spi
        let addr = command.address();
        self.write(spi, &[addr.reverse_bits()])
    }

    /// Basic function for sending an array of u8-values of data over spi
    ///
    pub(crate) fn args(
        &mut self, 
        spi: &mut SPI, 
        args: impl IntoIterator<Item = u8>
    ) -> Result<(), SPI::Error> {
        args.into_iter().try_for_each(|val| {
            self.write(spi, &[val.reverse_bits()])
        })
    }

    pub(crate) fn data(
        &mut self, 
        spi: &mut SPI, 
        data: impl IntoIterator<Item = u8>
    ) -> Result<(), SPI::Error> {
        data.into_iter().try_for_each(|val| {
            self.write(spi, &[val])
        })
    }

    /// Basic function for sending [Commands](Command) and the data belonging to it.
    ///
    /// TODO: directly use ::write? cs wouldn't needed to be changed twice than
    pub(crate) fn cmd_with_data<T: Command>(
        &mut self,
        spi: &mut SPI,
        command: T,
        args: impl IntoIterator<Item = u8>,
        data: impl IntoIterator<Item = u8>,
    ) -> Result<(), SPI::Error> {
        // this is nessessary for shifting out the previous frame when communicating
        // with high frequency
        let _ = self.cs.set_high();
        // activate spi with cs low
        let _ = self.cs.set_low();

        self.cmd(spi, command)?;
        self.args(spi, args)?;
        self.data(spi, data)?;

        let _ = self.cs.set_high();

        Ok(())
    }

    /// Basic function for sending [Commands](Command) and the data belonging to it.
    ///
    /// TODO: directly use ::write? cs wouldn't needed to be changed twice than
    pub(crate) fn cmd_with_arg<T: Command>(
        &mut self,
        spi: &mut SPI,
        command: T,
        args: impl IntoIterator<Item = u8>,
    ) -> Result<(), SPI::Error> {
        // this is nessessary for shifting out the previous frame when communicating
        // with high frequency
        let _ = self.cs.set_high();
        // activate spi with cs low
        let _ = self.cs.set_low();

        self.cmd(spi, command)?;
        self.args(spi, args)?;

        let _ = self.cs.set_high();

        Ok(())
    }

    /// Basic function for sending the same byte of data (one u8) multiple times over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](ConnectionInterface::command())
    #[allow(unused)]
    pub(crate) fn data_x_times(
        &mut self,
        spi: &mut SPI,
        val: u8,
        repetitions: u32,
    ) -> Result<(), SPI::Error> {
        // Transfer data (u8) over spi
        for _ in 0..repetitions {
            self.write(spi, &[val])?;
        }
        Ok(())
    }

    // spi write helper/abstraction function
    fn write(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        // transfer spi data
        // Be careful!! Linux has a default limit of 4096 bytes per spi transfer
        // see https://raspberrypi.stackexchange.com/questions/65595/spi-transfer-fails-with-buffer-size-greater-than-4096
        if cfg!(target_os = "linux") {
            for data_chunk in data.chunks(4096) {
                spi.write(data_chunk)?;
            }
        } else {
            spi.write(data)?;
        }

        Ok(())
    }

    /// Resets the device.
    ///
    /// Often used to awake the module from deep sleep. See [VFD256x50::sleep()](Epd4in2::sleep())
    ///
    /// The timing of keeping the reset pin low seems to be important and different per device.
    /// Most displays seem to require keeping it low for 10ms, but the 7in5_v2 only seems to reset
    /// properly with 2ms
    pub(crate) fn reset(&mut self, delay: &mut DELAY, duration: u8) {
        let _ = self.rst.set_low();
        delay.delay_ms(duration);
        let _ = self.rst.set_high();
        delay.delay_ms(1)
    }
}
