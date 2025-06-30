use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Text, TextStyleBuilder},
};
use linux_embedded_hal::I2cdev;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use std::io::Error;

pub struct SSD1305Display {
    device:
        Ssd1306<I2CInterface<I2cdev>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
}

impl SSD1305Display {
    pub fn new() -> Result<Self, Error> {
        let i2c = I2cdev::new("/dev/i2c-1")?;

        let interface = I2CDisplayInterface::new(i2c);

        let connection = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        let display = SSD1305Display { device: connection };

        display.log("Display Initialized", false);
        Ok(display)
    }

    pub fn draw(&mut self, data: Vec<&str>) {
        self.device.init().unwrap();
        self.device.set_brightness(Brightness::BRIGHTEST).unwrap();
        self.device.clear(BinaryColor::Off).unwrap();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        let mut current_display_line = 20;
        let line_height = 10;

        for item in data.iter().take(3) {
            let y_in_buffer = current_display_line * 2;

            Text::with_text_style(
                item.to_uppercase().as_str(),
                Point::new(5, y_in_buffer),
                text_style,
                TextStyleBuilder::new().build(),
            )
                .draw(&mut self.device)
                .unwrap();

            current_display_line += line_height / 2;

            self.log(item, false);
        }

        self.device.flush().unwrap();
    }

    pub fn cleanup(&mut self) {
        self.device.clear(BinaryColor::Off).unwrap();
        self.device.flush().unwrap();
        self.log("Cleaned up the screen", false);
    }

    fn log(&self, message: &str, warning: bool) {
        println!(
            "{} Display: {}",
            if warning { "[WARNING]" } else { "[INFO]" },
            message
        );
    }
}
