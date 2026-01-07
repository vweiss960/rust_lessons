use std::net::IpAddr;

use crate::error::ParseError;
use crate::types::{FlowId, SequenceInfo};
use super::parser::SequenceParser;

/// IPsec ESP (Encapsulating Security Payload) packet parser
/// Extracts sequence numbers from ESP header
///
/// Packet structure:
/// - Ethernet (14 bytes)
/// - IPv4 header (20+ bytes)
/// - ESP header:
///   - SPI (4 bytes)
///   - Sequence Number (4 bytes) ← TARGET
///   - Encrypted payload
///   - ESP trailer (variable)
///   - ICV (Integrity Check Value, 12-32 bytes)
pub struct IPsecParser;

// ESP protocol number in IP header
const IP_PROTOCOL_ESP: u8 = 50;

impl SequenceParser for IPsecParser {
    fn parse_sequence(&self, data: &[u8]) -> Result<Option<SequenceInfo>, ParseError> {
        // Quick protocol check
        if !self.matches(data) {
            return Ok(None);
        }

        // Validate minimum packet length
        // Minimum: Ethernet (14) + IPv4 header (20) + ESP header (8)
        if data.len() < 42 {
            return Err(ParseError::PacketTooShort);
        }

        // Extract IPv4 header to get destination IP and find ESP payload
        // IPv4 header length is in first nibble of byte 0 (offset 14 for Ethernet)
        let ihl = (data[14] & 0x0f) as usize * 4;
        let ip_header_end = 14 + ihl;

        // Check we have enough data for the IP header
        if data.len() < ip_header_end {
            return Err(ParseError::PacketTooShort);
        }

        // Extract destination IP (bytes 30-33 in standard IPv4)
        // Destination IP is at offset 14 + 16 = 30
        let dst_ip = IpAddr::V4(std::net::Ipv4Addr::new(
            data[30],
            data[31],
            data[32],
            data[33],
        ));

        // ESP payload starts after IP header
        let esp_payload = &data[ip_header_end..];

        // Check we have at least SPI + Sequence Number (8 bytes)
        if esp_payload.len() < 8 {
            return Err(ParseError::PacketTooShort);
        }

        // Extract SPI (4 bytes, big-endian)
        let spi = u32::from_be_bytes([
            esp_payload[0],
            esp_payload[1],
            esp_payload[2],
            esp_payload[3],
        ]);

        // Extract Sequence Number (4 bytes, big-endian)
        let sequence_number = u32::from_be_bytes([
            esp_payload[4],
            esp_payload[5],
            esp_payload[6],
            esp_payload[7],
        ]);

        // Calculate payload length (encrypted portion + trailer + ICV)
        // This is everything after the 8-byte ESP header
        let payload_length = esp_payload.len() - 8;

        Ok(Some(SequenceInfo {
            sequence_number,
            flow_id: FlowId::IPsec { spi, dst_ip },
            payload_length,
        }))
    }

    fn matches(&self, data: &[u8]) -> bool {
        // Minimum size: Ethernet (14) + IPv4 (20) + ESP header (8)
        if data.len() < 42 {
            return false;
        }

        // Check EtherType is IPv4 (0x0800)
        if data.len() < 14 || data[12] != 0x08 || data[13] != 0x00 {
            return false;
        }

        // Check IP protocol is ESP (50)
        // IP protocol field is at offset 23 (14 Ethernet + 9 into IP header)
        if data.len() < 24 {
            return false;
        }

        data[23] == IP_PROTOCOL_ESP
    }

    fn protocol_name(&self) -> &str {
        "IPsec-ESP"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    /// Helper to create minimal valid ESP packet
    fn create_esp_packet(spi: u32, seq: u32, dst_ip: [u8; 4]) -> Vec<u8> {
        let mut packet = Vec::new();

        // Ethernet header (14 bytes)
        packet.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]); // Dst MAC
        packet.extend_from_slice(&[0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB]); // Src MAC
        packet.extend_from_slice(&[0x08, 0x00]); // EtherType: IPv4

        // IPv4 header (20 bytes minimum)
        packet.push(0x45); // Version 4, IHL 5
        packet.push(0x00); // DSCP/ECN
        let total_len: u16 = 20 + 8 + 16; // IP header + ESP header + some payload
        packet.extend_from_slice(&total_len.to_be_bytes());
        packet.extend_from_slice(&[0x00, 0x00]); // Identification
        packet.extend_from_slice(&[0x00, 0x00]); // Flags + Fragment offset
        packet.push(64); // TTL
        packet.push(IP_PROTOCOL_ESP); // Protocol: ESP (50)
        packet.extend_from_slice(&[0x00, 0x00]); // Checksum (not validated)
        packet.extend_from_slice(&[192, 168, 1, 1]); // Source IP
        packet.extend_from_slice(&dst_ip); // Destination IP

        // ESP header (8 bytes)
        packet.extend_from_slice(&spi.to_be_bytes());
        packet.extend_from_slice(&seq.to_be_bytes());

        // Encrypted payload + ICV (16 bytes dummy)
        packet.extend_from_slice(&[0u8; 16]);

        packet
    }

    #[test]
    fn test_ipsec_parser_valid_packet() {
        let parser = IPsecParser;
        let packet = create_esp_packet(0x12345678, 42, [10, 0, 0, 1]);

        let result = parser.parse_sequence(&packet).unwrap();
        assert!(result.is_some());

        let seq_info = result.unwrap();
        assert_eq!(seq_info.sequence_number, 42);

        match seq_info.flow_id {
            FlowId::IPsec { spi, dst_ip } => {
                assert_eq!(spi, 0x12345678);
                assert_eq!(dst_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
            }
            _ => panic!("Expected IPsec flow ID"),
        }
    }

    #[test]
    fn test_ipsec_parser_wrong_protocol() {
        let parser = IPsecParser;
        let mut packet = create_esp_packet(0x12345678, 42, [10, 0, 0, 1]);

        // Change IP protocol from ESP (50) to TCP (6)
        packet[23] = 6;

        let result = parser.parse_sequence(&packet).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_ipsec_parser_too_short() {
        let parser = IPsecParser;
        let packet = vec![0u8; 20]; // Too short

        let result = parser.parse_sequence(&packet).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_ipsec_matches() {
        let parser = IPsecParser;
        let packet = create_esp_packet(0x12345678, 42, [10, 0, 0, 1]);

        assert!(parser.matches(&packet));
    }

    #[test]
    fn test_ipsec_sequence_wraparound() {
        let parser = IPsecParser;
        let packet = create_esp_packet(0xAABBCCDD, u32::MAX, [172, 16, 0, 1]);

        let result = parser.parse_sequence(&packet).unwrap();
        assert!(result.is_some());

        let seq_info = result.unwrap();
        assert_eq!(seq_info.sequence_number, u32::MAX);
    }

    #[test]
    fn test_ipsec_parser_multiple_flows() {
        let parser = IPsecParser;

        // Create two packets with different SPIs
        let packet1 = create_esp_packet(0x11111111, 100, [10, 0, 0, 1]);
        let packet2 = create_esp_packet(0x22222222, 200, [10, 0, 0, 1]);

        let result1 = parser.parse_sequence(&packet1).unwrap().unwrap();
        let result2 = parser.parse_sequence(&packet2).unwrap().unwrap();

        // Both should parse, but have different flow IDs
        match result1.flow_id {
            FlowId::IPsec { spi, .. } => assert_eq!(spi, 0x11111111),
            _ => panic!("Expected IPsec flow ID"),
        }

        match result2.flow_id {
            FlowId::IPsec { spi, .. } => assert_eq!(spi, 0x22222222),
            _ => panic!("Expected IPsec flow ID"),
        }
    }

    #[test]
    fn test_ipsec_payload_length() {
        let parser = IPsecParser;
        let packet = create_esp_packet(0x12345678, 42, [10, 0, 0, 1]);

        let result = parser.parse_sequence(&packet).unwrap().unwrap();
        // Payload length should be 16 (the dummy data we added)
        assert_eq!(result.payload_length, 16);
    }

    #[test]
    fn test_ipsec_wrong_ethertype() {
        let parser = IPsecParser;
        let mut packet = create_esp_packet(0x12345678, 42, [10, 0, 0, 1]);

        // Change EtherType to IPv6 (0x86DD)
        packet[12] = 0x86;
        packet[13] = 0xDD;

        assert!(!parser.matches(&packet));
    }
}
