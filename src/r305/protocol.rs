pub const PACKET_START_CODE_1: u8 = 0xEF;
pub const PACKET_START_CODE_2: u8 = 0x01;

// Packet types
pub const COMMAND_PACKET: u8 = 0x01;
pub const ACK_PACKET: u8 = 0x07; 
const DATA_PACKET: u8 = 0x02;
const END_DATA_PACKET: u8 = 0x08;

// Command Codes (from R305 manual/pyfingerprint equivalent)
pub const FINGERPRINT_VERIFYPASSWORD: u8 = 0x13; // Command for password verification
pub const FINGERPRINT_GET_DEVICE_INFO: u8 = 0x04; // Corrected command for Get Device Info based on typical R305 protocol

// Acknowledgment Codes (Payload[0] of ACK_PACKET)
pub const FINGERPRINT_OK: u8 = 0x00;
const FINGERPRINT_ERROR_COMMUNICATION: u8 = 0x01;
const FINGERPRINT_ERROR_WRONGPASSWORD: u8 = 0x02; // General "wrong password" error from sensor
const FINGERPRINT_NO_FINGER: u8 = 0x03; // No finger on sensor
const FINGERPRINT_ENROLL_FAILED: u8 = 0x05; // Enrollment failed
const FINGERPRINT_ADDRCODE: u8 = 0x14;