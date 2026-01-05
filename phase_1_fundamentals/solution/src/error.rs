//! Custom error types for protocol parsing
//!
//! This module defines the ParseError enum and implements the Error trait,
//! enabling clear, domain-specific error reporting.

use std::error::Error;
use std::fmt;

/// Represents failures that can occur during protocol parsing
///
/// Each variant includes relevant context to help debug parsing issues.
#[derive(Debug)]
pub enum ParseError {
    /// Message data is shorter than the minimum required (5 bytes)
    MessageTooShort { actual: usize },

    /// Protocol version is not supported (only version 1 is valid)
    InvalidVersion { version: u8 },

    /// Extracted payload length exceeds remaining data
    IncompletPayload { expected: usize, actual: usize },

    /// Checksum verification failed
    ChecksumMismatch { expected: u8, calculated: u8 },

    /// Payload size exceeds reasonable limits
    PayloadTooLarge { size: usize, max: usize },
}

impl fmt::Display for ParseError {
    /// Formats the error as a human-readable message
    ///
    /// This is called when the error is printed with {} formatting
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::MessageTooShort { actual } => {
                write!(
                    f,
                    "Message data too short: {} bytes (minimum 5 required)",
                    actual
                )
            }
            ParseError::InvalidVersion { version } => {
                write!(
                    f,
                    "Unsupported protocol version: {} (only version 1 is supported)",
                    version
                )
            }
            ParseError::IncompletPayload { expected, actual } => {
                write!(
                    f,
                    "Incomplete payload: expected {} bytes, but only {} available",
                    expected, actual
                )
            }
            ParseError::ChecksumMismatch {
                expected,
                calculated,
            } => {
                write!(
                    f,
                    "Checksum mismatch: expected 0x{:02X}, but calculated 0x{:02X}",
                    expected, calculated
                )
            }
            ParseError::PayloadTooLarge { size, max } => {
                write!(
                    f,
                    "Payload too large: {} bytes (maximum {} allowed)",
                    size, max
                )
            }
        }
    }
}

/// Error trait implementation
///
/// This empty implementation is required for ParseError to be a proper Error type.
/// The Display implementation above provides the error message.
impl Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_message_too_short() {
        let err = ParseError::MessageTooShort { actual: 3 };
        assert_eq!(
            err.to_string(),
            "Message data too short: 3 bytes (minimum 5 required)"
        );
    }

    #[test]
    fn test_error_display_invalid_version() {
        let err = ParseError::InvalidVersion { version: 5 };
        assert!(err.to_string().contains("version: 5"));
        assert!(err.to_string().contains("only version 1"));
    }

    #[test]
    fn test_error_display_checksum_mismatch() {
        let err = ParseError::ChecksumMismatch {
            expected: 0xAB,
            calculated: 0xCD,
        };
        assert!(err.to_string().contains("Checksum mismatch"));
        assert!(err.to_string().contains("0xAB"));
    }
}
