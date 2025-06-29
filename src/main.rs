mod r305;

use r305::lib::FingerprintSensor;

fn main() {
    println!("Testing r305");
    let baud_rate: u32 = 57600;
    let address: u32 = 0xFFFFFFFF;
    let password: u32 = 0x00000000;

    let sensor = FingerprintSensor::new(
        baud_rate,
        address,
        password
    );

    let mut fingerprint = match sensor {
        Ok(sensor) => sensor,
        Err(e) => {
            println!("Error initialising {}", e);
            return;
        }
    };
    
    match fingerprint.verify_password() {
        Ok(info) => {
            println!("{:?}", info);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}
