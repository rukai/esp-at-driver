#[derive(Debug)]
pub struct EspAt<RX, TX>
where
    RX: embedded_hal::serial::Read<u8>,
    TX: embedded_hal::serial::Write<u8>,
{
    rx: RX,
    tx: TX,
}

impl<RX, TX> EspAt<RX, TX>
where
    RX: embedded_hal::serial::Read<u8>,
    TX: embedded_hal::serial::Write<u8>,
{
    pub fn new(rx: RX, tx: TX) -> Self {
        Self { rx, tx }
    }
}
