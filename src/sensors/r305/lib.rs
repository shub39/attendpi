use crate::sensors::r305::error::FingerprintError;
use crate::sensors::r305::protocol::*;

use serialport::{self, DataBits, ErrorKind, FlowControl, Parity, SerialPort, StopBits};
use std::io::{self, Read, Write};
use std::time::Duration;

// view system info
#[derive(Debug, PartialEq, Eq)]
pub struct SystemParameters {
    pub status_register: u16,
    pub system_id: u16,
    pub storage_capacity: u16,
    pub security_level: u16,
    pub device_address: u32,
    pub packet_length: u16,
    pub baud_rate: u16,
}

pub struct FingerprintSensor {
    port: Box<dyn SerialPort>,
    address: u32,
    password: u32
}

impl FingerprintSensor {

    //makes a new fingerprint sensor struct after verifying password
    pub fn new(baud_rate: u32, address: u32, password: u32) -> Result<Self, FingerprintError> {
        if baud_rate < 9600 || baud_rate > 57600 {
            serialport::Error::new(ErrorKind::InvalidInput, "Baud rate must be between 9600 and 57600");
        }

        let ports = serialport::available_ports().map_err(|_| FingerprintError::NoFingerprintSensors)?;
        let port_name = if ports.is_empty() {
            return Err(FingerprintError::NoFingerprintSensors);
        } else {
            ports.first().unwrap().port_name.clone()
        };

        let port = serialport::new(&port_name, baud_rate)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .timeout(Duration::from_secs(2))
            .open()
            .map_err(|_| FingerprintError::NoFingerprintSensors)?;

        let mut sensor = FingerprintSensor { port, address, password };

        match sensor.verify_password() {
            Ok(_) => {
                sensor.log("Sensor initialised and password verified", false);
                Ok(sensor)
            },
            Err(_) => Err(FingerprintError::Auth)
        }
    }

    pub fn get_system_parameters(&mut self) -> Result<SystemParameters, FingerprintError> {
        let command_payload = vec![FINGERPRINT_GET_SYSTEM_PARAMETERS];

        let command_packet = self.build_packet(COMMAND_PACKET, &command_payload);
        self.send_command(&command_packet)?;

        let full = self.receive_exact()?;
        let (received_packet_type, received_payload) = self.parse_response_packet(&full)?;


        if received_packet_type != ACK_PACKET {
            return Err(FingerprintError::Protocol("Recieved packet is not ack packet!".to_string()));
        }

        if received_payload.is_empty() || received_payload[0] != FINGERPRINT_OK {
            return Err(FingerprintError::Protocol("Received payload is not OK".to_string()));
        }

        if received_payload.len() < 17 {
            return Err(FingerprintError::Protocol("Received payload too short".to_string()));
        }

        let status_register = u16::from_be_bytes([received_payload[1], received_payload[2]]);
        let system_id = u16::from_be_bytes([received_payload[3], received_payload[4]]);
        let storage_capacity = u16::from_be_bytes([received_payload[5], received_payload[6]]);
        let security_level = u16::from_be_bytes([received_payload[7], received_payload[8]]);

        let device_address = u32::from_be_bytes([
            received_payload[9],
            received_payload[10],
            received_payload[11],
            received_payload[12],
        ]);

        let packet_length = u16::from_be_bytes([received_payload[13], received_payload[14]]);
        let baud_rate = u16::from_be_bytes([received_payload[15], received_payload[16]]);

        Ok(SystemParameters {
            status_register,
            system_id,
            storage_capacity,
            security_level,
            device_address,
            packet_length,
            baud_rate,
        })
    }

    // enrolls a new fingerprint
    pub fn enroll(&mut self, id: u16) -> Result<(), FingerprintError> {
        self.get_image()?;
        self.image2tz(1)?;

        self.log("Remove Finger...", false);
        std::thread::sleep(Duration::from_secs(1));

        self.log("Place the same finger again...", false);
        self.get_image()?;
        self.image2tz(2)?;

        self.create_model()?;
        self.store_model(id)?;

        self.log(&format!("Fingerprint enrolled at : {}", id), false);
        Ok(())
    }

    // drops all stored templates
    pub fn delete_all(&mut self) -> Result<(), FingerprintError> {
        let packet = self.build_packet(COMMAND_PACKET, &[FINGERPRINT_CLEAR_DATABASE]);
        self.send_command(&packet)?;
        let response = self.receive_exact()?;
        let (_, payload) = self.parse_response_packet(&response)?;
        self.expect_ok(&payload)?;
        self.log("Fingerprint Database deleted", false);
        Ok(())
    }

    // searches for the fingerprint
    pub fn search(&mut self) -> Result<Option<u16>, FingerprintError> {
        self.get_image()?;
        self.image2tz(1)?;

        let payload = [
            FINGERPRINT_SEARCH_TEMPLATE,
            0x01,
            0x00, 0x00,
            0x00, 0xA3
        ];
        let packet = self.build_packet(COMMAND_PACKET, &payload);
        self.send_command(&packet)?;
        let response = self.receive_exact()?;
        let (_, payload) = self.parse_response_packet(&response)?;

        if payload[0] == FINGERPRINT_OK {
            let id = u16::from_be_bytes([payload[1], payload[2]]);
            let _score = u16::from_be_bytes([payload[3], payload[4]]);
            Ok(Some(id))
        } else if payload[0] == FINGERPRINT_ERROR_NO_TEMPLATE_FOUND {
            Ok(None)
        } else {
            Err(FingerprintError::SensorError(payload[0]))
        }
    }


    // util to unwrap payload
    fn expect_ok(&self, payload: &[u8]) -> Result<(), FingerprintError> {
        match payload.get(0) {
            Some(&FINGERPRINT_OK) => Ok(()),
            Some(code) => Err(FingerprintError::SensorError(*code)),
            None => Err(FingerprintError::Protocol("Got empty payload".to_string())),
        }
    }

    // converts image to template
    fn image2tz(&mut self, slot: u8) -> Result<(), FingerprintError> {
        let payload = [FINGERPRINT_CONVERT_IMAGE, slot];
        let packet = self.build_packet(COMMAND_PACKET, &payload);
        self.send_command(&packet)?;
        let response = self.receive_exact()?;
        let (_, payload) = self.parse_response_packet(&response)?;
        self.expect_ok(&payload)?;
        Ok(())
    }

    // reads image from sensor
    fn get_image(&mut self) -> Result<(), FingerprintError> {
        let max_retries = 10;
        for retry_no in 0..max_retries {
            let packet = self.build_packet(COMMAND_PACKET, &[FINGERPRINT_READ_IMAGE]);
            self.send_command(&packet)?;
            let response = self.receive_exact()?;
            let (_, payload) = self.parse_response_packet(&response)?;

            match payload.get(0) {
                Some(&FINGERPRINT_OK) => {
                    self.log("Image Captured", false);
                    return Ok(());
                }
                Some(&0x02) => {
                    self.log(&format!("No finger detected. Retry no: {}/10", retry_no), true);
                    std::thread::sleep(Duration::from_millis(500));
                    continue;
                }
                Some(&code) => return Err(FingerprintError::SensorError(code)),
                None => return Err(FingerprintError::Protocol("Got empty payload".to_string())),
            }
        }

        Err(FingerprintError::MaxRetries)
    }

    // creates template
    fn create_model(&mut self) -> Result<(), FingerprintError> {
        let packet = self.build_packet(COMMAND_PACKET, &[FINGERPRINT_CREATE_TEMPLATE]);
        self.send_command(&packet)?;
        let response = self.receive_exact()?;
        let (_, payload) = self.parse_response_packet(&response)?;
        self.expect_ok(&payload)?;
        Ok(())
    }

    // stores template
    fn store_model(&mut self, id: u16) -> Result<(), FingerprintError> {
        let id_bytes = id.to_be_bytes();
        let payload = [FINGERPRINT_STORE_TEMPLATE, 0x01, id_bytes[0], id_bytes[1]];
        let packet = self.build_packet(COMMAND_PACKET, &payload);
        self.send_command(&packet)?;
        let response = self.receive_exact()?;
        let (_, payload) = self.parse_response_packet(&response)?;
        self.expect_ok(&payload)?;
        Ok(())
    }

    //verifies password
    fn verify_password(&mut self) -> Result<bool, FingerprintError> {
        let password_bytes = self.password.to_be_bytes();

        let command_payload = vec![
            FINGERPRINT_VERIFY_PASSWORD,
            password_bytes[0],
            password_bytes[1],
            password_bytes[2],
            password_bytes[3],
        ];

        let command_packet = self.build_packet(COMMAND_PACKET, &command_payload);

        self.send_command(&command_packet)?;

        let mut response_buffer = vec![0; 32];
        let bytes_read = self.read_response(&mut response_buffer)?;

        if bytes_read == 0 {
            return Err(FingerprintError::Protocol("Got empty response".to_string()));
        }

        let (received_packet_type, received_payload) = self.parse_response_packet(&response_buffer[..bytes_read])?;

        if received_packet_type != ACK_PACKET {
            return Err(FingerprintError::Protocol("Got wrong ACK packet".to_string()));
        }

        if received_payload.is_empty() {
            return Err(FingerprintError::Protocol("Got empty payload".to_string()));
        }

        match received_payload[0] {
            FINGERPRINT_OK => {
                self.log("Password Verified", false);
                Ok(true)
            },
            FINGERPRINT_ERROR_WRONG_PASSWORD => {
                self.log("Invalid Password", true);
                Ok(false)
            }
            FINGERPRINT_ERROR_COMMUNICATION => {
                Err(FingerprintError::Protocol("Communication failed".to_string()))
            },
            FINGERPRINT_ADDR_CODE => {
                Err(FingerprintError::Auth)
            }
            other => Err(FingerprintError::Protocol(format!("Received unknown payload: {:?}", other))),
        }
    }

    //helper to send packet to sensor
    fn send_command(&mut self, command: &[u8]) -> io::Result<()> {
        self.port.write_all(command)?;
        self.port.flush()?;
        Ok(())
    }

    //recieve exact no of bytes
    fn receive_exact(&mut self) -> Result<Vec<u8>, FingerprintError> {
        let mut header = [0u8; 9];
        self.port.read_exact(&mut header)?;

        let length = u16::from_be_bytes([header[7], header[8]]) as usize;
        let mut body = vec![0u8; length];
        self.port.read_exact(&mut body)?;

        let mut full = header.to_vec();
        full.extend(body);
        Ok(full)
    }

    //helper to read resposne from sensor
    fn read_response(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        buffer.fill(0);
        let bytes_read = self.port.read(buffer)?;
        Ok(bytes_read)
    }

    //calculates checksum for the packet
    fn calculate_checksum(packet_type: u8, payload: &[u8]) -> u16 {
        let mut sum: u16 = packet_type as u16;
        sum = sum.wrapping_add((payload.len() + 2) as u16);

        for &byte in payload {
            sum = sum.wrapping_add(byte as u16);
        }
        sum
    }

    //builds the packet
    fn build_packet(&self, packet_type: u8, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push(PACKET_START_CODE_1);
        packet.push(PACKET_START_CODE_2);
        packet.extend_from_slice(&self.address.to_be_bytes());
        packet.push(packet_type);
        let packet_length = (payload.len() + 2) as u16;
        packet.extend_from_slice(&packet_length.to_be_bytes());
        packet.extend_from_slice(payload);
        let checksum = Self::calculate_checksum(packet_type, payload);
        packet.extend_from_slice(&checksum.to_be_bytes());

        packet
    }

    //parses packet response
    fn parse_response_packet(&self, raw_bytes: &[u8]) -> Result<(u8, Vec<u8>), FingerprintError> {
        if raw_bytes.len() < 12 {
            return Err(FingerprintError::Protocol("Received Packet too short".to_string()));
        }

        if raw_bytes[0] != PACKET_START_CODE_1 || raw_bytes[1] != PACKET_START_CODE_2 {
            return Err(FingerprintError::Protocol("Invalid start code in response".to_string()));
        }

        let received_address = u32::from_be_bytes([raw_bytes[2], raw_bytes[3], raw_bytes[4], raw_bytes[5]]);
        if received_address != self.address {
            return Err(FingerprintError::Auth);
        }

        let packet_type = raw_bytes[6];
        let packet_length = u16::from_be_bytes([raw_bytes[7], raw_bytes[8]]);

        let expected_payload_len = (packet_length - 2) as usize;

        if raw_bytes.len() < 9 + expected_payload_len + 2 {
            return Err(FingerprintError::Protocol("Received packet too short".to_string()));
        }

        let payload_start_idx = 9;
        let payload_end_idx = payload_start_idx + expected_payload_len;
        let payload = raw_bytes[payload_start_idx..payload_end_idx].to_vec();

        let received_checksum = u16::from_be_bytes([raw_bytes[payload_end_idx], raw_bytes[payload_end_idx + 1]]);
        let calculated_checksum = Self::calculate_checksum(packet_type, &payload);

        if received_checksum != calculated_checksum {
            return Err(FingerprintError::Protocol("Checksum mismatch".to_string()));
        }

        Ok((packet_type, payload))
    }

    fn log(&self, message: &str, warning: bool) {
        println!("{} FingerprintSensor: {}", if warning { "[WARNING]" } else { "[INFO]" }, message);
    }
}