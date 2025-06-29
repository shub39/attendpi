mod sensors;

use crate::sensors::keypad::Keypad;
use sensors::r305_fingerprint_sensor::lib::FingerprintSensor;
use sensors::ssd1305_display::SSD1305Display;

fn main() {
    println!("Testing r305_fingerprint_sensor");
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

    let mut keypad = Keypad::new();

    let mut display = match SSD1305Display::new() {
        Ok(display) => display,
        Err(e) => {
            println!("Error Initialising display");
            return;
        }
    };

    display.draw(
        vec!["Hello World", "I'm shub39", "Serial Masochist"]
    );
}
