mod sensors;
mod test;

use crate::sensors::keypad::Keypad;
use crate::test::test;
use sensors::r305_fingerprint_sensor::lib::FingerprintSensor;
use sensors::ssd1305_display::SSD1305Display;

fn main() {
    let baud_rate: u32 = 57600;
    let address: u32 = 0xFFFFFFFF;
    let password: u32 = 0x00000000;

    let mut fingerprint_sensor = match FingerprintSensor::new(baud_rate, address, password) {
        Ok(sensor) => sensor,
        Err(e) => {
            println!("Error initialising {}", e);
            return;
        }
    };

    fingerprint_sensor.delete_all().unwrap();

    let mut keypad = Keypad::new();

    let mut display = match SSD1305Display::new() {
        Ok(display) => display,
        Err(e) => {
            println!("Error Initialising display {}", e);
            return;
        }
    };

    test(
        &mut fingerprint_sensor,
        &mut display,
        &mut keypad,
    )
}
