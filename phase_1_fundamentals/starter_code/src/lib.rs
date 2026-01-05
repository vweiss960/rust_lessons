/// Binary Protocol Parser
///
/// This module defines the core types and parsing functions for a simple binary protocol.
///
/// Protocol Format:
/// - Byte 0: Version (u8)
/// - Byte 1: Message Type (u8)
/// - Bytes 2-3: Message Length (u16, big-endian)
/// - Bytes 4..4+length: Payload (variable)
/// - Last byte: Checksum (u8, XOR of all payload bytes)

use std::fmt;
use std::error::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Custom error type for protocol parsing failures
#[derive(Debug)]
pub enum ParseError {
    // TODO: Add variants for different error types
    // Examples:
    // - InvalidVersion(u8)
    // - MessageTooShort
    // - ChecksumMismatch { expected: u8, got: u8 }
    // - PayloadTooLarge(usize)
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Implement Display for each error variant
        // Match on self and write descriptive messages
        write!(f, "Parse error")
    }
}

impl Error for ParseError {}

// ============================================================================
// Core Protocol Types
// ============================================================================

/// Represents a parsed binary protocol message
#[derive(Debug)]
pub struct Message {
    pub version: u8,
    pub message_type: u8,
    pub payload: Vec<u8>,
    pub checksum: u8,
}

impl Message {
    /// Creates a new message
    pub fn new(version: u8, message_type: u8, payload: Vec<u8>) -> Self {
        let checksum = calculate_checksum(&payload);
        Message {
            version,
            message_type,
            payload,
            checksum,
        }
    }

    /// Serializes the message to bytes in protocol format
    ///
    /// Returns a vector containing the complete protocol message
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Implement serialization
        // Format:
        // [version][message_type][length_hi][length_lo][payload...][checksum]
        // where length is big-endian u16 of payload length
        todo!()
    }

    /// Validates the message integrity
    ///
    /// Returns Ok(()) if valid, Err(ParseError) if invalid
    pub fn validate(&self) -> Result<(), ParseError> {
        // TODO: Verify checksum matches calculated value
        // Verify version is valid (>= 1)
        // Verify message_type is valid
        // Verify payload length is reasonable
        todo!()
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Pretty-print the message
        // Format: "Message v{version} type {type} payload {len} bytes checksum {cs}"
        write!(f, "Message")
    }
}

// ============================================================================
// Parsing Functions
// ============================================================================

/// Parses a byte slice into a Message
///
/// # Arguments
/// * `data` - The bytes to parse
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
pub fn parse(data: &[u8]) -> Result<Message, ParseError> {
    // TODO: Implement parsing
    // 1. Check minimum length (at least 5 bytes)
    // 2. Extract version from byte 0
    // 3. Extract message_type from byte 1
    // 4. Extract length from bytes 2-3 as big-endian u16
    // 5. Verify total length matches (5 + payload length + 1 for checksum)
    // 6. Extract payload
    // 7. Extract and verify checksum
    // 8. Create and return Message
    todo!()
}

/// Parses multiple messages from a byte stream
///
/// Reads messages sequentially until end of input or error
pub fn parse_multiple(data: &[u8]) -> Result<Vec<Message>, ParseError> {
    // TODO: Implement sequential parsing
    // 1. Create empty vector for messages
    // 2. Start at position 0
    // 3. Loop: parse message at current position
    // 4. Add to vector
    // 5. Move position past parsed message
    // 6. Continue until end of data
    todo!()
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Calculates the XOR checksum of a byte slice
fn calculate_checksum(data: &[u8]) -> u8 {
    // TODO: XOR all bytes together
    // Start with 0, XOR each byte
    todo!()
}

/// Converts a big-endian u16 from two bytes
fn bytes_to_u16(bytes: &[u8]) -> u16 {
    // TODO: Combine two bytes into u16 in big-endian format
    // Byte 0 is high byte, byte 1 is low byte
    todo!()
}

/// Converts a u16 to big-endian bytes
fn u16_to_bytes(value: u16) -> [u8; 2] {
    // TODO: Split u16 into two big-endian bytes
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add at least 8 unit tests
    // Test cases to include:
    // 1. Valid simple message
    // 2. Invalid version
    // 3. Message too short
    // 4. Checksum mismatch
    // 5. Payload extraction
    // 6. Multiple messages
    // 7. Empty payload
    // 8. Maximum payload size

    #[test]
    fn test_checksum_calculation() {
        // TODO: Test calculate_checksum
        // Example: checksum of [0x01, 0x02] should be 0x03
    }

    #[test]
    fn test_bytes_to_u16() {
        // TODO: Test big-endian conversion
        // Example: [0x00, 0x0A] should be 10
    }
}
