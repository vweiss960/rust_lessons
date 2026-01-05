use byteorder::{BigEndian, ByteOrder};

use crate::error::ParseError;
use crate::types::{FlowId, SequenceInfo};

use super::parser::SequenceParser;

/// MACsec packet parser
/// Parses the MACsec Security Tag (SecTag) to extract packet number and SCI
pub struct MACsecParser;

impl SequenceParser for MACsecParser {
    fn parse_sequence(&self, data: &[u8]) -> Result<Option<SequenceInfo>, ParseError> {
        // Quick protocol check
        if !self.matches(data) {
            return Ok(None);
        }

        // Validate minimum packet length
        // Minimum: Ethernet header (14) + SecTag headers (4) + Packet Number (4) + SCI (8)
        if data.len() < 30 {
            return Err(ParseError::PacketTooShort);
        }

        // MACsec frame format:
        // Bytes 0-5:     Destination MAC
        // Bytes 6-11:    Source MAC
        // Bytes 12-13:   EtherType (0x88E5 for MACsec)
        // Bytes 14:      TCI/AN flags
        // Bytes 15:      Short Length
        // Bytes 16-19:   Packet Number (4 bytes, big-endian) ← TARGET FIELD
        // Bytes 20-27:   SCI (8 bytes, big-endian) ← FLOW IDENTIFIER
        // Bytes 28+:     Encrypted Payload
        // Last 16:       ICV (Integrity Check Value)

        // Extract packet number at offset 16-19 (4 bytes, big-endian)
        let packet_number = BigEndian::read_u32(&data[16..20]);

        // Extract SCI (Secure Channel Identifier) at offset 20-27 (8 bytes, big-endian)
        let sci = BigEndian::read_u64(&data[20..28]);

        // Calculate payload length (total - Ethernet header - SecTag - ICV)
        // Assume ICV is always 16 bytes for standard MACsec
        let payload_length = if data.len() > 28 + 16 {
            data.len() - 28 - 16
        } else {
            0
        };

        Ok(Some(SequenceInfo {
            sequence_number: packet_number,
            flow_id: FlowId::MACsec { sci },
            payload_length,
        }))
    }

    fn matches(&self, data: &[u8]) -> bool {
        // Check minimum Ethernet frame size
        if data.len() < 14 {
            return false;
        }

        // Check EtherType 0x88E5 (MACsec) at offset 12-13
        data[12] == 0x88 && data[13] == 0xE5
    }

    fn protocol_name(&self) -> &str {
        "MACsec"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macsec_parser_valid_packet() {
        // Create a minimal valid MACsec packet
        let mut packet = vec![0u8; 45]; // Minimum size with payload

        // Set Ethernet header
        packet[0..6].copy_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]); // Dst MAC
        packet[6..12].copy_from_slice(&[0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB]); // Src MAC

        // Set EtherType 0x88E5
        packet[12] = 0x88;
        packet[13] = 0xE5;

        // Set TCI/AN and Short Length (can be anything valid)
        packet[14] = 0x00;
        packet[15] = 0x00;

        // Set Packet Number to 123 (big-endian)
        BigEndian::write_u32(&mut packet[16..20], 123);

        // Set SCI
        BigEndian::write_u64(&mut packet[20..28], 0x001122334455AABB);

        let parser = MACsecParser;
        let result = parser.parse_sequence(&packet).unwrap();

        assert!(result.is_some());
        let seq_info = result.unwrap();
        assert_eq!(seq_info.sequence_number, 123);
        assert!(matches!(seq_info.flow_id, FlowId::MACsec { sci: 0x001122334455AABB }));
    }

    #[test]
    fn test_macsec_parser_wrong_ethertype() {
        // Create packet with wrong EtherType
        let mut packet = vec![0u8; 30];
        packet[12] = 0x08; // IPv4
        packet[13] = 0x00;

        let parser = MACsecParser;
        let result = parser.parse_sequence(&packet).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_macsec_parser_too_short() {
        // Create packet that's too short
        let packet = vec![0u8; 10];

        let parser = MACsecParser;
        let result = parser.parse_sequence(&packet).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_macsec_parser_minimum_valid_size() {
        let mut packet = vec![0u8; 30]; // Minimum for valid MACsec

        // Set valid EtherType
        packet[12] = 0x88;
        packet[13] = 0xE5;

        // Set packet number
        BigEndian::write_u32(&mut packet[16..20], 42);
        BigEndian::write_u64(&mut packet[20..28], 0xAABBCCDDEEFF0011);

        let parser = MACsecParser;
        let result = parser.parse_sequence(&packet).unwrap();

        assert!(result.is_some());
        let seq_info = result.unwrap();
        assert_eq!(seq_info.sequence_number, 42);
    }
}
