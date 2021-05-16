#![no_std]

// This has some nice examples of expected AT command usage:
// https://docs.espressif.com/projects/esp-at/en/latest/AT_Command_Examples/TCP-IP_AT_Examples.html

use embassy::io::{AsyncWriteExt, AsyncBufReadExt};
use heapless::{Vec, String};
use core::fmt::Write;

const _WIFI_MTU: usize = 2304;
type ReplyData = Vec<u8, 512>;

#[derive(Debug)]
/// A driver to provide configuration and sending/receiving messages for the AT firmware running on any ESP wifi chip.
/// Important Wifi Terminology:
/// *   Station - A regular wifi client that connects to an access point to access the network <https://en.wikipedia.org/wiki/Station_(networking)>
/// *   AP - A wifi Access Point <https://en.wikipedia.org/wiki/Wireless_access_point>
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
    /// Construct a new EspAt instance.
    /// You are expected to regularly create and destroy instances so it is cheap to do so.
    pub fn new(uart: T) -> Self {
        Self { uart }
    }

    /// Set the wifi mode
    pub async fn set_wifi_mode(&mut self, mode: WifiMode) -> Result<(), GenericEspAtError> {
        match mode {
            WifiMode::Disabled         => self.write_line(b"AT+CWMODE=0,1").await?,
            WifiMode::Station          => self.write_line(b"AT+CWMODE=1,1").await?,
            WifiMode::SoftAP           => self.write_line(b"AT+CWMODE=2,1").await?,
            WifiMode::StationAndSoftAP => self.write_line(b"AT+CWMODE=3,1").await?,
        }
        self.read_reply().await?;
        Ok(())
    }

    /// Connect to the AP as configured in ConnectionConfig
    // TODO: return CWJAP<error code>
    pub async fn connect_to_access_point<'a>(&mut self, config: ConnectionConfig<'a>) -> Result<(), GenericEspAtError> {
        self.direct_write(b"AT+CWJAP=").await?;
        if let Some(ssid) = config.ssid {
            // TODO: Escape character syntax is needed if SSID or password contains special characterss: ",\\
            self.direct_write(ssid.as_bytes()).await?;
        }
        self.direct_write(b",").await?;
        if let Some(password) = config.password {
            self.direct_write(password.as_bytes()).await?;
        }
        self.direct_write(b",").await?;
        if let Some(bssid) = config.bssid {
            self.direct_write(bssid.as_bytes()).await?;
        }
        self.direct_write(b",").await?;
        if let Some(rssi) = config.rssi {
            self.direct_write(rssi.as_bytes()).await?;
        }
        self.direct_write(b",").await?;
        if let Some(pci_enable) = config.pci_enable {
            if pci_enable {
                self.direct_write(b"1").await?;
            } else {
                self.direct_write(b"2").await?;
            }
        }
        self.direct_write(b",").await?;
        if let Some(reconnect_interval) = config.reconnect_interval {
            // TODO: Should I be using write!
            // And/or is there a better way?
            let mut string = String::<32>::new();
            write!(string, "{}", reconnect_interval)
                .map_err(|_| GenericEspAtError::ATResponseInvalid)?;
            self.direct_write(string.as_bytes()).await?;
        }
        self.direct_write(b",").await?;
        if let Some(listen_interval) = config.listen_interval {
            let mut string = String::<32>::new();
            write!(string, "{}", listen_interval)
                .map_err(|_| GenericEspAtError::ATResponseInvalid)?;
            self.direct_write(string.as_bytes()).await?;
        }
        self.direct_write(b",").await?;
        if let Some(scan_mode) = config.scan_mode {
            let write = match scan_mode {
                ScanMode::Fast => b"0",
                ScanMode::AllChannel => b"1",
            };
            self.direct_write(write).await?;
        }
        self.direct_write(b",").await?;
        if let Some(timeout) = config.timeout {
            let mut string = String::<32>::new();
            write!(string, "{}", timeout)
                .map_err(|_| GenericEspAtError::ATResponseInvalid)?;
            self.direct_write(string.as_bytes()).await?;
        }
        self.direct_write(b",").await?;
        if let Some(pmf) = config.pmf {
            let write = match pmf {
                PMF::Disable => b"0",
                PMF::Enable  => b"0", // TODO: ??? what on earth is the difference between "0" and "bit 0"
                PMF::Require => b"1",
            };
            self.direct_write(write).await?;
        }
        Ok(())
    }

    /// Get the local addresses of the AP wifi connection
    pub async fn get_local_addresses_softap(&mut self) -> Result<LocalAddresses, GenericEspAtError> {
        self.write_line(b"AT+CIFSR").await?;
        let reply = self.read_reply().await?;

        Ok(LocalAddresses {
            ipv4:        EspAt::<T>::get_element_after(&reply, b"APIP")?,
            ipv6_local:  EspAt::<T>::get_element_after(&reply, b"APIP6LL")?,
            ipv6_global: EspAt::<T>::get_element_after(&reply, b"APIP6GL")?,
            mac:         EspAt::<T>::get_element_after(&reply, b"APMAC")?,
        })
    }

    /// Get the local addresses of the station wifi connection
    pub async fn get_local_addresses_station(&mut self) -> Result<LocalAddresses, GenericEspAtError> {
        self.write_line(b"AT+CIFSR").await?;
        let reply = self.read_reply().await?;

        Ok(LocalAddresses {
            ipv4:        EspAt::<T>::get_element_after(&reply, b"STAIP")?,
            ipv6_local:  EspAt::<T>::get_element_after(&reply, b"STAIP6LL")?,
            ipv6_global: EspAt::<T>::get_element_after(&reply, b"STAIP6GL")?,
            mac:         EspAt::<T>::get_element_after(&reply, b"STAMAC")?,
        })
    }

    /// Get the local addresses of the ethernet connection
    pub async fn get_local_addresses_ethernet(&mut self) -> Result<LocalAddresses, GenericEspAtError> {
        self.write_line(b"AT+CIFSR").await?;
        let reply = self.read_reply().await?;

        Ok(LocalAddresses {
            ipv4:        EspAt::<T>::get_element_after(&reply, b"ETHIP")?,
            ipv6_local:  EspAt::<T>::get_element_after(&reply, b"ETHIP6LL")?,
            ipv6_global: EspAt::<T>::get_element_after(&reply, b"ETHIP6GL")?,
            mac:         EspAt::<T>::get_element_after(&reply, b"ETHMAC")?,
        })
    }

    /// example calls:
    /// get_element_after("+CIFSR:APIP,foo\r\n+CIFSR:APIP6LL,bar", "APIP") -> "foo"
    /// get_element_after("+CIFSR:APIP,foo\r\n+CIFSR:APIP6LL,bar", "APIP6LL") -> "bar"
    fn get_element_after<const N: usize>(reply: &[u8], search_element: &[u8]) -> Result<Option<String<N>>, GenericEspAtError> {
        reply.windows(search_element.len())
            .position(|window| window == search_element)
            .map(|position| {
                let mut string = String::new();
                for string_char in &search_element[position+1..] {
                    if *string_char == b'\r' || *string_char == b',' {
                        break;
                    }
                    string.push(*string_char as char).map_err(|_| GenericEspAtError::ATResponseInvalid)?;
                }
                Ok(string)
            })
            .transpose()
    }

    /// Can only be used for responses that end in either OK or ERROR.
    async fn read_reply(&mut self) -> Result<ReplyData, GenericEspAtError> {
        let mut reply = ReplyData::new();
        let mut end = 0;
        while end < 512 {
            end += self.uart.read(&mut reply[end..]).await // sdfsdfk
                .map_err(|e| GenericEspAtError::EmbassyError(e))?;

            if reply.ends_with(b"OK\r\n") {
                reply.truncate(reply.len()-4);
                return Ok(reply);
            }
            if reply.ends_with(b"ERROR\r\n") {
                reply.truncate(reply.len()-7);
                return Err(GenericEspAtError::ATError(reply))
            }

            // TODO: Timeout if the reply doesnt finish in 10s
        }
        Err(GenericEspAtError::ATResponseTooLong(reply))
    }

    async fn write_line(&mut self, data: &[u8]) -> Result<(), GenericEspAtError> {
        self.direct_write(data).await?;
        self.direct_write(b"\r\n").await?;

        Ok(())
    }

    pub async fn direct_write(&mut self, data: &[u8]) -> Result<(), GenericEspAtError> {
        self.uart.write_all(data).await.map_err(|e| GenericEspAtError::EmbassyError(e))
    }

    pub async fn direct_read(&mut self) -> Result<ReplyData, GenericEspAtError> {
        let mut reply = ReplyData::new();
        self.uart.read_exact(&mut reply).await
            .map_err(|e| GenericEspAtError::EmbassyError(e))?;
        Ok(reply)
    }
}

// TODO: The error cases make the enum take up lots of memory.
// should I add dep on heap or remove ReplyData from the variants?
// If I dont find other reasons for needing a heap then I will just remove them.
#[derive(Debug)] // TODO: Is deriving Debug ok here?
pub enum GenericEspAtError {
    /// Embassy's UART implementation encountered an error
    EmbassyError(embassy::io::Error),
    /// The ESP responded with ERROR
    ATError(ReplyData),
    /// The ESP response was longer then our buffer size.
    /// This indicates that our buffer was too small or the ESP chip/firmware is broken.
    ATResponseTooLong(ReplyData),
    /// The ESP response was unexpected in some way.
    /// This indicates a bug in our parsing logic or the ESP chip/firmware is broken.
    ATResponseInvalid,
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

// TODO: Should all fields have an Option? i.e. if no value is provided will it use a previously saved non default value?
/// In most cases you just need to set the ssid and password
#[derive(Default)]
pub struct ConnectionConfig<'a> {
    /// SSID of the AP to connect to (maximum of 32 characters)
    pub ssid: Option<&'a str>,
    /// BSSID (MAC Address) of the AP to connect to. Required when there are two access
    /// points with the same SSID in range. (exactly 17 characters)
    pub bssid: Option<&'a str>,
    /// Password of the AP to connect to (maximum of 32 characters)
    pub password: Option<&'a str>,
    /// Received Signal Strength Indicator, the strength of the signal to be used when connecting to the AP
    pub rssi: Option<&'a str>,
    /// PCI authentication enable
    pub pci_enable: Option<bool>,
    /// The seconds between wifi reconnect attempts. Can be between 0 and 7200. Default is 1.
    /// When 0 will never attempt to reconnect.
    pub reconnect_interval: Option<u16>,
    /// The interval of listening to the AP's beacon. Can be between 1 and 100. Default is 3.
    pub listen_interval: Option<u16>,
    /// Scan Mode to use for finding the AP.
    pub scan_mode: Option<ScanMode>,
    /// Timeout in seconds for this command. Can be between 3 and 600. Default is 16.
    pub timeout: Option<u16>,
    /// Protected Management Frames. Default is PMF::Disable
    pub pmf: Option<PMF>,
}

/// Scan Mode
pub enum ScanMode {
    /// Fast scan. Will immediately connect to the first scanned AP.
    Fast,
    /// Will connect to the scanned AP with the strongest signal.
    AllChannel,
}

/// Procted management Frames
pub enum PMF {
    /// PMF is disabled
    Disable,
    /// PMF is enabled and preferred but not required
    Enable,
    /// PMF is enabled and required
    Require,
}

// TODO: Tentatively typing these fields as strings but maybe they should be [u8; 4] etc instead???
pub struct LocalAddresses {
    /// IPV4 address up to 15 characters
    pub ipv4: Option<String<15>>,
    /// IPV6 address exactly 39 characters
    pub ipv6_local: Option<String<39>>,
    /// IPV6 address exactly 39 characters
    pub ipv6_global: Option<String<39>>,
    /// MAC address exactly 17 characters
    pub mac: Option<String<17>>,
}
