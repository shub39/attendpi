pub const PACKET_START_CODE_1: u8 = 0xEF;
pub const PACKET_START_CODE_2: u8 = 0x01;

//packet types
pub const COMMAND_PACKET: u8 = 0x01;
pub const ACK_PACKET: u8 = 0x07; 

//command Codes (from R305 manual/pyfingerprint equivalent)
pub const FINGERPRINT_VERIFY_PASSWORD: u8 = 0x13; 
pub const FINGERPRINT_GET_SYSTEM_PARAMETERS: u8 = 0x0F;
pub const FINGERPRINT_CLEAR_DATABASE: u8 = 0x0D;
pub const FINGERPRINT_READ_IMAGE: u8 = 0x01;
pub const FINGERPRINT_CONVERT_IMAGE: u8 = 0x02;
pub const FINGERPRINT_CREATE_TEMPLATE: u8 = 0x05;
pub const FINGERPRINT_STORE_TEMPLATE: u8 = 0x06;
pub const FINGERPRINT_SEARCH_TEMPLATE: u8 = 0x04;
pub const FINGERPRINT_ERROR_NO_TEMPLATE_FOUND: u8 = 0x09;

//acknowledgment Codes (Payload[0] of ACK_PACKET)
pub const FINGERPRINT_OK: u8 = 0x00;
pub const FINGERPRINT_ERROR_COMMUNICATION: u8 = 0x01;
pub const FINGERPRINT_ERROR_WRONG_PASSWORD: u8 = 0x02; 
pub const FINGERPRINT_ADDR_CODE: u8 = 0x14;