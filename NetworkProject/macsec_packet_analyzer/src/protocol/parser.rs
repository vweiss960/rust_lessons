use crate::error::ParseError;
use crate::types::SequenceInfo;

/// Abstraction for protocols with sequence numbers
/// Allows different protocol parsers to be used interchangeably
pub trait SequenceParser {
    /// Parse sequence number and flow ID from raw packet data
    /// Returns Some(SequenceInfo) if the packet matches this protocol
    /// Returns None if the packet is not for this protocol
    fn parse_sequence(&self, data: &[u8]) -> Result<Option<SequenceInfo>, ParseError>;

    /// Check if packet matches this protocol (quick check before full parsing)
    fn matches(&self, data: &[u8]) -> bool;

    /// Get the name of this protocol for reporting
    fn protocol_name(&self) -> &str;
}
