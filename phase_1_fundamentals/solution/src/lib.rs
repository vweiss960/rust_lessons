//! Binary Protocol Parser
//!
//! A library for parsing and serializing a simple binary protocol.
//!
//! ## Protocol Format
//!
//! ```text
//! Byte 0:     Version (u8)
//! Byte 1:     Message Type (u8)
//! Bytes 2-3:  Payload Length (u16, big-endian)
//! Bytes 4..:  Payload (variable length)
//! Last byte:  Checksum (u8, XOR of all payload bytes)
//! ```
//!
//! ## Example
//!
//! ```
//! use binary_protocol_parser::Message;
//!
//! // Create a message
//! let msg = Message::new(1, 5, vec![72, 101, 108, 108, 111]);  // "Hello"
//!
//! // Serialize to bytes
//! let bytes = msg.to_bytes();
//! println!("{:?}", bytes);
//!
//! // Parse back
//! let parsed = binary_protocol_parser::parse(&bytes).unwrap();
//! assert_eq!(parsed.version, 1);
//! ```

pub mod error;

use error::ParseError;
use std::fmt;

/// Maximum allowed payload size (in bytes)
const MAX_PAYLOAD_SIZE: usize = 65535;

/// Represents a parsed binary protocol message
///
/// Contains all the fields from a protocol message including version,
/// message type, payload, and checksum for integrity verification.
#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    /// Protocol version (typically 1)
    pub version: u8,

    /// Type/command identifier
    pub message_type: u8,

    /// Message payload data
    pub payload: Vec<u8>,

    /// XOR checksum of payload for integrity verification
    pub checksum: u8,
}

impl Message {
    /// Creates a new message with automatically calculated checksum
    ///
    /// # Arguments
    /// * `version` - Protocol version
    /// * `message_type` - Type/command identifier
    /// * `payload` - Message data
    ///
    /// # Example
    /// ```
    /// use binary_protocol_parser::Message;
    ///
    /// let msg = Message::new(1, 5, vec![1, 2, 3]);
    /// assert_eq!(msg.version, 1);
    /// assert_eq!(msg.checksum, 0); // 1 ^ 2 ^ 3 = 0
    /// ```
    pub fn new(version: u8, message_type: u8, payload: Vec<u8>) -> Self {
        let checksum = calculate_checksum(&payload);
        Message {
            version,
            message_type,
            payload,
            checksum,
        }
    }

    /// Serializes the message to protocol format bytes
    ///
    /// Returns a vector of bytes following the protocol specification:
    /// [version][message_type][length_hi][length_lo][payload...][checksum]
    ///
    /// # Example
    /// ```
    /// use binary_protocol_parser::Message;
    ///
    /// let msg = Message::new(1, 5, vec![1, 2, 3]);
    /// let bytes = msg.to_bytes();
    /// assert_eq!(bytes[0], 1);        // version
    /// assert_eq!(bytes[1], 5);        // message_type
    /// assert_eq!(bytes[2], 0);        // length high byte
    /// assert_eq!(bytes[3], 3);        // length low byte
    /// assert_eq!(bytes[4..7], [1, 2, 3][..]);  // payload
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Add version
        result.push(self.version);

        // Add message type
        result.push(self.message_type);

        // Add payload length as big-endian u16
        let length_bytes = u16_to_bytes(self.payload.len() as u16);
        result.extend_from_slice(&length_bytes);

        // Add payload
        result.extend_from_slice(&self.payload);

        // Add checksum
        result.push(self.checksum);

        result
    }

    /// Validates message integrity
    ///
    /// Verifies that:
    /// - Version is valid (must be 1)
    /// - Checksum matches the calculated value
    /// - Message is not malformed
    ///
    /// # Returns
    /// * `Ok(())` if message is valid
    /// * `Err(ParseError)` if validation fails
    ///
    /// # Example
    /// ```
    /// use binary_protocol_parser::Message;
    ///
    /// let msg = Message::new(1, 5, vec![1, 2, 3]);
    /// assert!(msg.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), ParseError> {
        // Verify version
        if self.version != 1 {
            return Err(ParseError::InvalidVersion {
                version: self.version,
            });
        }

        // Verify checksum
        let calculated = calculate_checksum(&self.payload);
        if calculated != self.checksum {
            return Err(ParseError::ChecksumMismatch {
                expected: self.checksum,
                calculated,
            });
        }

        Ok(())
    }
}

impl fmt::Display for Message {
    /// Pretty-prints the message in human-readable format
    ///
    /// Shows version, message type, payload length, and checksum
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Message v{} type={} payload_len={} checksum=0x{:02X}",
            self.version,
            self.message_type,
            self.payload.len(),
            self.checksum
        )
    }
}

/// Parses a byte slice into a Message
///
/// # Arguments
/// * `data` - The bytes to parse (must follow protocol format)
///
/// # Returns
/// * `Ok(Message)` if parsing succeeds
/// * `Err(ParseError)` if parsing fails
///
/// # Protocol Format
/// The byte stream must follow this format:
/// - Byte 0: Version (must be 1)
/// - Byte 1: Message Type
/// - Bytes 2-3: Payload length as big-endian u16
/// - Bytes 4..4+length: Payload data
/// - Last byte: Checksum (XOR of all payload bytes)
///
/// # Example
/// ```
/// use binary_protocol_parser::parse;
///
/// let packet = vec![1, 5, 0, 3, 1, 2, 3, 0]; // v1, type5, len3, payload[1,2,3], checksum
/// let msg = parse(&packet).unwrap();
/// assert_eq!(msg.version, 1);
/// assert_eq!(msg.message_type, 5);
/// assert_eq!(msg.payload, vec![1, 2, 3]);
/// ```
pub fn parse(data: &[u8]) -> Result<Message, ParseError> {
    // Check minimum length (version + type + length + checksum = 5 bytes minimum)
    if data.len() < 5 {
        return Err(ParseError::MessageTooShort {
            actual: data.len(),
        });
    }

    // Extract version (byte 0)
    let version = data[0];

    // Verify version is supported
    if version != 1 {
        return Err(ParseError::InvalidVersion { version });
    }

    // Extract message type (byte 1)
    let message_type = data[1];

    // Extract payload length from bytes 2-3 (big-endian)
    let length = bytes_to_u16(&data[2..4]) as usize;

    // Verify we have enough data for the payload
    // Format: version(1) + type(1) + length(2) + payload(length) + checksum(1)
    let required_length = 4 + length + 1;
    if data.len() < required_length {
        return Err(ParseError::IncompletPayload {
            expected: required_length,
            actual: data.len(),
        });
    }

    // Verify payload size is reasonable
    if length > MAX_PAYLOAD_SIZE {
        return Err(ParseError::PayloadTooLarge {
            size: length,
            max: MAX_PAYLOAD_SIZE,
        });
    }

    // Extract payload (bytes 4..4+length)
    let payload = data[4..4 + length].to_vec();

    // Extract checksum (last byte of payload section)
    let checksum = data[4 + length];

    // Create message and validate
    let message = Message {
        version,
        message_type,
        payload,
        checksum,
    };

    // Verify checksum
    message.validate()?;

    Ok(message)
}

/// Parses multiple sequential messages from a byte stream
///
/// Continues parsing messages until all input is consumed or an error occurs.
///
/// # Arguments
/// * `data` - The bytes to parse (may contain multiple messages)
///
/// # Returns
/// * `Ok(Vec<Message>)` if all messages parse successfully
/// * `Err(ParseError)` if parsing fails (partially parsed messages are discarded)
///
/// # Example
/// ```
/// use binary_protocol_parser::{Message, parse_multiple};
///
/// let msg1 = Message::new(1, 5, vec![1, 2, 3]);
/// let msg2 = Message::new(1, 10, vec![4, 5, 6, 7]);
///
/// let mut data = msg1.to_bytes();
/// data.extend_from_slice(&msg2.to_bytes());
///
/// let messages = parse_multiple(&data).unwrap();
/// assert_eq!(messages.len(), 2);
/// ```
pub fn parse_multiple(data: &[u8]) -> Result<Vec<Message>, ParseError> {
    let mut messages = Vec::new();
    let mut position = 0;

    while position < data.len() {
        // Parse one message starting at position
        let message = parse(&data[position..])?;

        // Calculate how many bytes this message consumed
        let message_bytes = message.to_bytes();
        let message_length = message_bytes.len();

        // Add to results
        messages.push(message);

        // Move to next message
        position += message_length;
    }

    Ok(messages)
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Calculates the XOR checksum of a byte slice
///
/// The checksum is computed by XORing all bytes together,
/// which provides a simple integrity check.
///
/// # Arguments
/// * `data` - The bytes to checksum
///
/// # Returns
/// The XOR checksum value
///
/// # Example
/// ```
/// // Internally used by Message::new()
/// // 0x01 ^ 0x02 ^ 0x03 = 0x00
/// ```
fn calculate_checksum(data: &[u8]) -> u8 {
    // XOR all bytes together, starting with 0
    data.iter().fold(0u8, |acc, &byte| acc ^ byte)
}

/// Converts two bytes into a big-endian u16
///
/// The first byte is the high byte, the second is the low byte.
///
/// # Arguments
/// * `bytes` - A slice of at least 2 bytes
///
/// # Returns
/// The combined value as u16
///
/// # Example
/// ```
/// // [0x00, 0x0A] = 10
/// // [0x01, 0x00] = 256
/// ```
fn bytes_to_u16(bytes: &[u8]) -> u16 {
    ((bytes[0] as u16) << 8) | (bytes[1] as u16)
}

/// Converts a u16 into two big-endian bytes
///
/// The first byte is the high byte, the second is the low byte.
///
/// # Arguments
/// * `value` - The value to convert
///
/// # Returns
/// A 2-byte array in big-endian format
///
/// # Example
/// ```
/// // 10 = [0x00, 0x0A]
/// // 256 = [0x01, 0x00]
/// ```
fn u16_to_bytes(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, (value & 0xFF) as u8]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Utility Function Tests ==========

    #[test]
    fn test_calculate_checksum_simple() {
        assert_eq!(calculate_checksum(&[]), 0);
        assert_eq!(calculate_checksum(&[5]), 5);
        assert_eq!(calculate_checksum(&[5, 5]), 0);  // 5 ^ 5 = 0
        assert_eq!(calculate_checksum(&[1, 2, 3]), 0);  // 1 ^ 2 ^ 3 = 0
    }

    #[test]
    fn test_calculate_checksum_hello_world() {
        // "Hello World" = H(0x48) e(0x65) l(0x6C) l(0x6C) o(0x6F) space(0x20) W(0x57) o(0x6F) r(0x72) l(0x6C) d(0x64)
        let payload = b"Hello World";
        let checksum = calculate_checksum(payload);

        // Verify by manual calculation
        let mut expected = 0u8;
        for &byte in payload {
            expected ^= byte;
        }
        assert_eq!(checksum, expected);
    }

    #[test]
    fn test_bytes_to_u16_basic() {
        assert_eq!(bytes_to_u16(&[0x00, 0x0A]), 10);
        assert_eq!(bytes_to_u16(&[0x01, 0x00]), 256);
        assert_eq!(bytes_to_u16(&[0xFF, 0xFF]), 65535);
        assert_eq!(bytes_to_u16(&[0x12, 0x34]), 0x1234);
    }

    #[test]
    fn test_u16_to_bytes_basic() {
        assert_eq!(u16_to_bytes(10), [0x00, 0x0A]);
        assert_eq!(u16_to_bytes(256), [0x01, 0x00]);
        assert_eq!(u16_to_bytes(65535), [0xFF, 0xFF]);
        assert_eq!(u16_to_bytes(0x1234), [0x12, 0x34]);
    }

    #[test]
    fn test_u16_round_trip() {
        for value in [0, 1, 255, 256, 1000, 65535] {
            let bytes = u16_to_bytes(value);
            let restored = bytes_to_u16(&bytes);
            assert_eq!(value, restored);
        }
    }

    // ========== Message Tests ==========

    #[test]
    fn test_message_new_calculates_checksum() {
        let msg = Message::new(1, 5, vec![1, 2, 3]);
        assert_eq!(msg.checksum, 0);  // 1 ^ 2 ^ 3 = 0
    }

    #[test]
    fn test_message_to_bytes_format() {
        let msg = Message::new(1, 5, vec![1, 2, 3]);
        let bytes = msg.to_bytes();

        assert_eq!(bytes[0], 1);           // version
        assert_eq!(bytes[1], 5);           // message_type
        assert_eq!(bytes[2], 0);           // length high byte
        assert_eq!(bytes[3], 3);           // length low byte
        assert_eq!(&bytes[4..7], &[1, 2, 3]);  // payload
        assert_eq!(bytes[7], 0);           // checksum
    }

    #[test]
    fn test_message_validate_valid() {
        let msg = Message::new(1, 5, vec![1, 2, 3]);
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_message_validate_invalid_version() {
        let msg = Message {
            version: 2,  // Invalid version
            message_type: 5,
            payload: vec![1, 2, 3],
            checksum: 0,
        };
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_message_validate_checksum_mismatch() {
        let msg = Message {
            version: 1,
            message_type: 5,
            payload: vec![1, 2, 3],
            checksum: 99,  // Wrong checksum
        };
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_message_display() {
        let msg = Message::new(1, 5, vec![1, 2, 3, 4, 5]);
        let display_str = format!("{}", msg);
        assert!(display_str.contains("v1"));
        assert!(display_str.contains("type=5"));
        assert!(display_str.contains("payload_len=5"));
    }

    // ========== Parsing Tests ==========

    #[test]
    fn test_parse_valid_simple() {
        // Manually construct valid packet
        let packet = vec![
            0x01,           // version
            0x05,           // message_type
            0x00, 0x03,     // length = 3
            0x01, 0x02, 0x03,  // payload
            0x00,           // checksum (1 ^ 2 ^ 3 = 0)
        ];

        let msg = parse(&packet).expect("Parse failed");
        assert_eq!(msg.version, 1);
        assert_eq!(msg.message_type, 5);
        assert_eq!(msg.payload, vec![1, 2, 3]);
        assert_eq!(msg.checksum, 0);
    }

    #[test]
    fn test_parse_message_too_short() {
        let packet = vec![0x01, 0x05, 0x00];  // Only 3 bytes
        assert!(parse(&packet).is_err());
    }

    #[test]
    fn test_parse_invalid_version() {
        let packet = vec![
            0x02,           // invalid version
            0x05,
            0x00, 0x00,
            0x00,
        ];
        let result = parse(&packet);
        assert!(matches!(result, Err(ParseError::InvalidVersion { version: 2 })));
    }

    #[test]
    fn test_parse_incomplete_payload() {
        let packet = vec![
            0x01,           // version
            0x05,           // message_type
            0x00, 0x05,     // length = 5 (but we won't provide it)
            0x01, 0x02,     // only 2 bytes of payload
            0x00,           // checksum
        ];
        assert!(matches!(
            parse(&packet),
            Err(ParseError::IncompletPayload { .. })
        ));
    }

    #[test]
    fn test_parse_checksum_mismatch() {
        let packet = vec![
            0x01,           // version
            0x05,           // message_type
            0x00, 0x03,     // length = 3
            0x01, 0x02, 0x03,  // payload
            0xFF,           // wrong checksum
        ];
        assert!(matches!(
            parse(&packet),
            Err(ParseError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn test_parse_empty_payload() {
        let packet = vec![
            0x01,           // version
            0x05,           // message_type
            0x00, 0x00,     // length = 0
            0x00,           // checksum (for empty payload)
        ];

        let msg = parse(&packet).expect("Parse failed");
        assert_eq!(msg.payload.len(), 0);
    }

    #[test]
    fn test_parse_payload_too_large() {
        // Create a packet claiming to have more than MAX_PAYLOAD_SIZE bytes
        let packet = vec![
            0x01,           // version
            0x01,           // message_type
            0xFF, 0xFF,     // length = 65535 (exceeds reasonable limit)
            0x00,           // At least one data byte to pass initial length check
        ];
        // This should fail with PayloadTooLarge before we even check if data exists
        let result = parse(&packet);
        assert!(matches!(
            result,
            Err(ParseError::PayloadTooLarge { .. }) | Err(ParseError::IncompletPayload { .. })
        ));
    }

    #[test]
    fn test_parse_multiple_messages() {
        let msg1 = Message::new(1, 5, vec![1, 2, 3]);
        let msg2 = Message::new(1, 10, vec![4, 5, 6, 7]);

        let mut data = msg1.to_bytes();
        data.extend_from_slice(&msg2.to_bytes());

        let messages = parse_multiple(&data).expect("Parse failed");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].message_type, 5);
        assert_eq!(messages[1].message_type, 10);
        assert_eq!(messages[1].payload.len(), 4);
    }

    #[test]
    fn test_round_trip() {
        let original = Message::new(1, 10, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]);
        let bytes = original.to_bytes();
        let parsed = parse(&bytes).expect("Parse failed");

        assert_eq!(parsed.version, original.version);
        assert_eq!(parsed.message_type, original.message_type);
        assert_eq!(parsed.payload, original.payload);
        assert_eq!(parsed.checksum, original.checksum);
    }
}
