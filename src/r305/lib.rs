use serialport::{self, DataBits, ErrorKind, FlowControl, Parity, SerialPort, StopBits};
use std::io::{self, Read, Write};
use std::time::Duration;
use crate::r305::protocol::*;

pub struct FingerprintSensor {
    port: Box<dyn SerialPort>,
    address: u32,
    password: u32,
    baud_rate: u32,
    port_name: String
}

impl FingerprintSensor {
    pub fn new(
        baud_rate: u32,
        address: u32,
        password: u32,
    ) -> Result<Self, serialport::Error> {
        if baud_rate < 9600 || baud_rate > 57600 {
            serialport::Error::new(ErrorKind::InvalidInput, "Baud rate must be between 9600 and 57600");
        }
        
        let ports = serialport::available_ports()?;
        let port_name = if ports.is_empty() {
            return Err(serialport::Error::new(ErrorKind::NoDevice, "No available ports"));
        } else { 
            ports.first().unwrap().port_name.clone()
        };

        let port = serialport::new(&port_name, baud_rate)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .timeout(Duration::from_millis(2000))
            .open()?;

        Ok(
            FingerprintSensor {
                port,
                address,
                password,
                baud_rate,
                port_name,
            }
        )
    }

    pub fn send_command(&mut self, command: &[u8]) -> io::Result<()> {
        println!("Sending: {:02X?}", command); 
        self.port.write_all(command)?;
        self.port.flush()?;
        Ok(())
    }

    pub fn read_response(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        buffer.fill(0);
        let bytes_read = self.port.read(buffer)?;
        println!("Received: {:02X?}", &buffer[..bytes_read]);
        Ok(bytes_read)
    }

    fn calculate_checksum(packet_type: u8, payload: &[u8]) -> u16 {
        let mut sum: u16 = packet_type as u16;
        sum = sum.wrapping_add((payload.len() + 2) as u16); 

        for &byte in payload {
            sum = sum.wrapping_add(byte as u16);
        }
        sum
    }

    fn build_packet(&self, packet_type: u8, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push(PACKET_START_CODE_1);
        packet.push(PACKET_START_CODE_2);
        packet.extend_from_slice(&self.address.to_be_bytes()); 
        packet.push(packet_type);
        let packet_length = (payload.len() + 2) as u16;
        packet.extend_from_slice(&packet_length.to_be_bytes()); // Big Endian
        packet.extend_from_slice(payload);
        let checksum = Self::calculate_checksum(packet_type, payload);
        packet.extend_from_slice(&checksum.to_be_bytes()); // Big Endian

        packet
    }

    fn parse_response_packet(&self, raw_bytes: &[u8]) -> Result<(u8, Vec<u8>), Box<dyn std::error::Error>> {
        if raw_bytes.len() < 12 { 
            return Err("Received packet too short.".into());
        }

        if raw_bytes[0] != PACKET_START_CODE_1 || raw_bytes[1] != PACKET_START_CODE_2 {
            return Err("Invalid start code in response.".into());
        }

        let received_address = u32::from_be_bytes([raw_bytes[2], raw_bytes[3], raw_bytes[4], raw_bytes[5]]);
        if received_address != self.address {
            return Err("Received packet with wrong device address.".into());
        }

        let packet_type = raw_bytes[6];
        let packet_length = u16::from_be_bytes([raw_bytes[7], raw_bytes[8]]);

        let expected_payload_len = (packet_length - 2) as usize;

        if raw_bytes.len() < 9 + expected_payload_len + 2 { 
            return Err(format!("Received packet shorter than expected payload length. Expected at least {} bytes, got {}.", 9 + expected_payload_len + 2, raw_bytes.len()).into());
        }

        let payload_start_idx = 9;
        let payload_end_idx = payload_start_idx + expected_payload_len;
        let payload = raw_bytes[payload_start_idx..payload_end_idx].to_vec();

        let received_checksum = u16::from_be_bytes([raw_bytes[payload_end_idx], raw_bytes[payload_end_idx + 1]]);
        let calculated_checksum = Self::calculate_checksum(packet_type, &payload);

        if received_checksum != calculated_checksum {
            println!("Checksum mismatch! Expected: {:04X}, Got: {:04X}", calculated_checksum, received_checksum);
            return Err("Checksum mismatch in received packet.".into());
        }

        Ok((packet_type, payload))
    }

    pub fn verify_password(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let password_bytes = self.password.to_be_bytes();

        let command_payload = vec![
            FINGERPRINT_VERIFYPASSWORD,
            password_bytes[0], // MSB
            password_bytes[1],
            password_bytes[2],
            password_bytes[3], // LSB
        ];

        let command_packet = self.build_packet(COMMAND_PACKET, &command_payload);

        self.send_command(&command_packet)?;

        let mut response_buffer = vec![0; 32]; // ACK packet is typically small
        let bytes_read = self.read_response(&mut response_buffer)?;

        if bytes_read == 0 {
            return Err("No response received for verify password.".into());
        }

        let (received_packet_type, received_payload) = self.parse_response_packet(&response_buffer[..bytes_read])?;

        if received_packet_type != ACK_PACKET {
            return Err("The received packet is not an ACK packet!".into());
        }

        if received_payload.is_empty() {
            return Err("ACK packet has no payload.".into());
        }

        match received_payload[0] {
            FINGERPRINT_OK => {
                println!("Password verification successful!");
                Ok(true)
            },
            FINGERPRINT_ERROR_WRONGPASSWORD => {
                println!("Password verification failed: Wrong password.");
                Ok(false) // Pyfingerprint returns False here
            },
            FINGERPRINT_ERROR_COMMUNICATION => {
                Err("Communication error during password verification.".into())
            },
            FINGERPRINT_ADDRCODE => {
                Err("The device address is wrong.".into())
            }
        }
    }
}