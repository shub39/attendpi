use rppal::gpio::{Gpio, InputPin, Level, OutputPin};
use std::thread::sleep;
use std::time::Duration;

const KEYS: [[char; 4]; 4] = [
    ['1', '2', '3', 'A'],
    ['4', '5', '6', 'B'],
    ['7', '8', '9', 'C'],
    ['*', '0', '#', 'D'],
];

const ROW_PINS: [u8; 4] = [5, 6, 13, 19];
const COL_PINS: [u8; 4] = [12, 16, 20, 21];

pub struct Keypad {
    rows: [OutputPin; 4],
    cols: [InputPin; 4],
}

impl Keypad {
    pub fn new() -> Self {
        let gpio = Gpio::new().unwrap();

        let rows: [OutputPin; 4] = ROW_PINS.map(|pin| {
            let mut out = gpio.get(pin).unwrap().into_output();
            out.set_low();
            out
        });

        let cols: [InputPin; 4] = COL_PINS.map(|pin| {
            let input = gpio.get(pin).unwrap().into_input_pulldown();
            input
        });

        let keypad = Keypad { rows, cols };
        keypad.log("Keypad Initialised", false);

        keypad
    }

    pub fn read_key(&mut self) -> Option<char> {
        for (i, row) in self.rows.iter_mut().enumerate() {
            row.set_high();
            sleep(Duration::from_millis(50));

            for (j, col) in self.cols.iter().enumerate() {
                if col.read() == Level::High {
                    row.set_low();
                    self.log(&format!("Key pressed: {}", KEYS[i][j]), false);
                    return Some(KEYS[i][j]);
                }
            }

            row.set_low();
        }

        None
    }

    fn log(&self, message: &str, warning: bool) {
        println!("{} Keypad: {}", if warning { "[WARNING]" } else { "[INFO]" }, message);
    }
}