use crate::sensors::keypad::Keypad;
use crate::sensors::r305_fingerprint_sensor::lib::FingerprintSensor;
use crate::sensors::ssd1305_display::SSD1305Display;
use std::thread::sleep;
use std::time::Duration;

// test fingerprint, keypad and display
pub fn test(
    fingerprint_sensor: &mut FingerprintSensor,
    display: &mut SSD1305Display,
    keypad: &mut Keypad,
) {
    let mut index = 1;
    let mut exit_flag = false;

    loop {
        if exit_flag {
            exit_flag = false;
            break;
        }

        display.draw(vec!["Enrolling", &format!("Fingerprint {}", index)]);

        match fingerprint_sensor.enroll(index) {
            Ok(_) => {
                display.draw(vec!["Enrolled!"]);
                sleep(Duration::from_secs(1));

                display.draw(vec!["1: Continue", "Any: Exit"]);
                loop {
                    match keypad.read_key() {
                        None => {}
                        Some(key) => {
                            if key == '1' {
                                index += 1;
                                break;
                            } else {
                                exit_flag = true;
                                break;
                            }
                        }
                    }
                }
            }
            Err(_) => {
                display.draw(vec!["Error", "Retrying"]);
                sleep(Duration::from_millis(500));
            }
        }
    }

    loop {
        if exit_flag { break; }

        display.draw(vec!["Detecting Fingerprints..."]);

        match fingerprint_sensor.search() {
            Ok(index) => {
                if index.is_some() {
                    display.draw(vec!["Detected Fingerprint", &format!("{}", index.unwrap())]);
                    sleep(Duration::from_secs(1));
                    display.draw(vec!["1: Continue", "Any: Exit"]);

                    loop {
                        match keypad.read_key() {
                            None => {}
                            Some(key) => {
                                if key == '1' {
                                    break;
                                } else {
                                    exit_flag = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }

    display.cleanup();
}