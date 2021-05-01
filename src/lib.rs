#![no_std]

// This has some nice examples of expected AT command usage:
// https://docs.espressif.com/projects/esp-at/en/latest/AT_Command_Examples/TCP-IP_AT_Examples.html

// Ah, the hal's directly implement these traits:
// https://docs.rs/stm32f30x-hal/0.2.0/stm32f30x_hal/serial/struct.Rx.html

// TODO: Can we avoid using the block! macro?
use nb::block;

type ReadString = heapless::String<heapless::consts::U512>;

#[derive(Debug)]
pub struct EspAt<RX, TX>
where
    TX: embedded_hal::serial::Write<u8>,
    RX: embedded_hal::serial::Read<u8>,
{
    tx: TX,
    rx: RX,
}

impl<RX, TX> EspAt<RX, TX>
where
    TX: embedded_hal::serial::Write<u8>,
    RX: embedded_hal::serial::Read<u8>,
{
    pub fn new(tx: TX, rx: RX) -> Self {
        Self { tx, rx }
    }

    pub fn set_wifi_mode(&mut self, mode: WifiMode) -> Result<(), GenericError<TX::Error, RX::Error>> {
        match mode {
            WifiMode::Disabled         => self.write_line(b"AT+CWMODE=0,1")?,
            WifiMode::Station          => self.write_line(b"AT+CWMODE=1,1")?,
            WifiMode::SoftAP           => self.write_line(b"AT+CWMODE=2,1")?,
            WifiMode::StationAndSoftAP => self.write_line(b"AT+CWMODE=3,1")?,
        }
        let reply = self.read_line()?;
        if reply == "OK" {
            Ok(())
        } else {
            Err(GenericError::ATError(reply))
        }
    }

    fn read_line(&mut self) -> Result<ReadString, GenericError<TX::Error, RX::Error>> {
        let mut line = ReadString::new();
        let mut prev_value = 'a';
        loop {
            let value = match block!(self.rx.read()) {
                Ok(word) => word as char,
                Err(e) => { return Err(GenericError::ReadError(e)); }
            };

            if value != '\r' && value != '\n' {
                if let Err(()) = line.push(value) {
                    return Err(GenericError::ATResponseTooLong(line));
                }
            }
            else if prev_value == '\r' && value == '\n' {
                return Ok(line);
            }
            prev_value = value;
        }
    }

    fn write_line(&mut self, data: &[u8]) -> Result<(), GenericError<TX::Error, RX::Error>> {
        for x in data {
            self.write_byte(*x)?;
        }
        self.write_byte(b'\r')?;
        self.write_byte(b'\n')?;

        Ok(())
    }

    fn write_byte(&mut self, data: u8) -> Result<(), GenericError<TX::Error, RX::Error>> {
        match block!(self.tx.write(data)) {
            Ok(()) => Ok(()),
            Err(e) => Err(GenericError::WriteError(e)),
        }
    }
}

/// `RXE` and `TXE` will be the `Error` type(s) of your serial port
/// implementation, as defined by `embedded_hal::serial::Read<u8>::Error`
/// and `embedded_hal::serial::Write<u8>::Error` respectively.
#[derive(Debug)] // TODO: This is bad right??
pub enum GenericError<TXE, RXE> {
    WriteError(TXE),
    ReadError(RXE),
    ATError(ReadString),
    ATResponseTooLong(ReadString),
}

pub enum WifiMode {
    /// Completely disable wifi RF activity.
    Disabled,
    /// Act as a regular wifi client https://en.wikipedia.org/wiki/Station_(networking)
    Station,
    /// Act as a wifi Access Point https://en.wikipedia.org/wiki/Wireless_access_point
    SoftAP,
    /// Act as both a regular wifi client and a wifi Access Point
    StationAndSoftAP,
}
