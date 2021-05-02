#![no_std]

// This has some nice examples of expected AT command usage:
// https://docs.espressif.com/projects/esp-at/en/latest/AT_Command_Examples/TCP-IP_AT_Examples.html

use embassy::io::{AsyncWriteExt, AsyncBufReadExt};

type ReadLine = [u8; 512];

#[derive(Debug)]
pub struct EspAt<T>
where
    T: AsyncWriteExt + AsyncBufReadExt + Unpin,
{
    uart: T,
}

impl<T> EspAt<T>
where
    T: AsyncWriteExt + AsyncBufReadExt + Unpin,
{
    pub fn new(uart: T) -> Self {
        Self { uart }
    }

    pub async fn set_wifi_mode(&mut self, mode: WifiMode) -> Result<(), GenericError> {
        match mode {
            WifiMode::Disabled         => self.write_line(b"AT+CWMODE=0,1").await?,
            WifiMode::Station          => self.write_line(b"AT+CWMODE=1,1").await?,
            WifiMode::SoftAP           => self.write_line(b"AT+CWMODE=2,1").await?,
            WifiMode::StationAndSoftAP => self.write_line(b"AT+CWMODE=3,1").await?,
        }
        let reply = self.read_line().await?;
        if &reply[0..2] == b"OK" {
            Ok(())
        } else {
            Err(GenericError::ATError(reply))
        }
    }

    async fn read_line(&mut self) -> Result<ReadLine, GenericError> { // TODO: probably need to return multiple lines or something
        let mut line = [0u8; 512];
        let mut end = 0;
        while end < 512 {
            end += match self.uart.read(&mut line[end..]).await {
                Ok(length) => length,
                Err(e) => { return Err(GenericError::EmbassyError(e)); }
            };

            if end >= 2 && line[end-2] == b'\r' && line[end-1] == b'\n' { // TODO: fix this check
                return Ok(line);
            }
        }
        Err(GenericError::ATResponseTooLong(line))
    }

    async fn write_line(&mut self, data: &[u8]) -> Result<(), GenericError> {
        self.write_data(data).await?;
        self.write_data(b"\r\n").await?;

        Ok(())
    }

    async fn write_data(&mut self, data: &[u8]) -> Result<(), GenericError> {
        match self.uart.write_all(data).await {
            Ok(()) => Ok(()),
            Err(e) => Err(GenericError::EmbassyError(e)),
        }
    }
}

/// `RXE` and `TXE` will be the `Error` type(s) of your serial port
/// implementation, as defined by `embedded_hal::serial::Read<u8>::Error`
/// and `embedded_hal::serial::Write<u8>::Error` respectively.
#[derive(Debug)] // TODO: This is bad right??
pub enum GenericError {
    EmbassyError(embassy::io::Error),
    ATError(ReadLine),
    ATResponseTooLong(ReadLine),
}

pub enum WifiMode {
    /// Completely disable wifi RF activity.
    Disabled,
    /// Act as a regular wifi client <https://en.wikipedia.org/wiki/Station_(networking)>
    Station,
    /// Act as a wifi Access Point <https://en.wikipedia.org/wiki/Wireless_access_point>
    SoftAP,
    /// Act as both a regular wifi client and a wifi Access Point
    StationAndSoftAP,
}
