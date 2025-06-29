mod r305;

use r305::lib::FingerprintSensor;

fn main() {
    println!("Testing r305");
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
    
    let system_parameters = fingerprint_sensor.get_system_parameters().ok();
    dbg!(system_parameters);
}
