use std::io;
use thiserror::Error;

// sensor specific errors
#[derive(Error, Debug)]
pub enum FingerprintError {
    #[error("No Fingerprint Sensors found")]
    NoFingerprintSensors,

    #[error("Serial port error: {0}")]
    Serial(#[from] io::Error),

    #[error("Invalid password or communication error")]
    Auth,

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Sensor returned error code: 0x{0:02X}")]
    SensorError(u8),

    #[error("Reached Maximum Retry limit")]
    MaxRetries,
}
