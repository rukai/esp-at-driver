use esp_at_driver::{EspAt, WifiMode};

use embedded_hal_mock::serial::{Mock, Transaction};

#[test]
fn test_constructing() {
    let mut serial = Mock::new(&[]);
    EspAt::new(serial.clone(), serial.clone());
    serial.done();
}

#[test]
fn test_set_wifi_mode_disable() {
    let mut serial = Mock::new(&[
        Transaction::write_many(b"AT+CWMODE=0,1\r\n"),
        Transaction::read_many(b"OK\r\n"),
    ]);

    let mut esp_at = EspAt::new(serial.clone(), serial.clone());
    esp_at.set_wifi_mode(WifiMode::Disabled).unwrap();
    serial.done();
}
