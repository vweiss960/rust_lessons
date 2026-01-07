use std::net::IpAddr;

use crate::error::ParseError;
use crate::types::{FlowId, SequenceInfo};
use super::parser::SequenceParser;

/// Generic L3 (Layer 3) packet parser for plain TCP/UDP traffic
/// Extracts 5-tuple flow information without sequence number tracking
///
/// Supported protocols:
/// - TCP (IP protocol 6): No gap detection (TCP byte-level sequences don't align with packet loss)
/// - UDP (IP protocol 17): No sequence number tracking
///
/// Note: Gap detection is intentionally disabled for generic L3 flows because:
/// - TCP sequence numbers track cumulative bytes, not packet counts
/// - TCP permits retransmissions and out-of-order delivery
/// - False positive gap rates are very high (67%+ on typical traffic)
///
/// To enable flow tracking without gap detection, this parser returns a "synthetic"
/// sequence number (0) for all packets. The FlowTracker detects GenericL3 flows and
/// skips gap detection, using only the 5-tuple for flow identification.
///
/// Packet structure:
/// - Ethernet (14 bytes)
/// - IPv4 header (20+ bytes)
/// - TCP/UDP header
pub struct GenericL3Parser;

// IP protocol numbers
const IP_PROTOCOL_TCP: u8 = 6;
const IP_PROTOCOL_UDP: u8 = 17;

impl SequenceParser for GenericL3Parser {
    fn parse_sequence(&self, data: &[u8]) -> Result<Option<SequenceInfo>, ParseError> {
        // Generic L3 flows: Extract 5-tuple for flow identification
        // Return synthetic sequence number (all zeros) to keep FlowTracker engaged
        // while disabling gap detection. FlowTracker detects GenericL3 flows and
        // skips gap analysis for them.

        // Quick protocol check
        if !self.matches(data) {
            return Ok(None);
        }

        // Validate minimum packet length
        // Minimum: Ethernet (14) + IPv4 (20) + TCP/UDP header (8)
        if data.len() < 42 {
            return Err(ParseError::PacketTooShort);
        }

        // Extract IPv4 header
        let ihl = (data[14] & 0x0f) as usize * 4;
        let ip_header_end = 14 + ihl;

        // Check we have enough data for the IP header
        if data.len() < ip_header_end {
            return Err(ParseError::PacketTooShort);
        }

        // Extract IP protocol type
        let protocol = data[23];

        // Extract source and destination IPs
        let src_ip = IpAddr::V4(std::net::Ipv4Addr::new(data[26], data[27], data[28], data[29]));
        let dst_ip = IpAddr::V4(std::net::Ipv4Addr::new(data[30], data[31], data[32], data[33]));

        // Get transport layer payload
        let transport_payload = &data[ip_header_end..];

        // Check we have at least port + port (4 bytes minimum)
        if transport_payload.len() < 4 {
            return Err(ParseError::PacketTooShort);
        }

        // Extract source and destination ports (same position in TCP and UDP)
        let src_port = u16::from_be_bytes([transport_payload[0], transport_payload[1]]);
        let dst_port = u16::from_be_bytes([transport_payload[2], transport_payload[3]]);

        // Calculate payload length for statistics (bytes after TCP/UDP header)
        let payload_length = match protocol {
            IP_PROTOCOL_TCP => {
                // Extract TCP header length (first nibble of byte 12) to find payload
                let tcp_header_len = if transport_payload.len() > 12 {
                    ((transport_payload[12] >> 4) as usize) * 4
                } else {
                    20 // Default TCP header size
                };

                if transport_payload.len() > tcp_header_len {
                    transport_payload.len() - tcp_header_len
                } else {
                    0
                }
            }
            IP_PROTOCOL_UDP => {
                // UDP header is 8 bytes
                if transport_payload.len() > 8 {
                    transport_payload.len() - 8
                } else {
                    0
                }
            }
            _ => 0,
        };

        // Return synthetic sequence number (0) for all packets
        // This allows FlowTracker to track the flow for statistics (bytes, packet count, bandwidth)
        // while gap detection is disabled in FlowTracker for GenericL3 flows
        Ok(Some(SequenceInfo {
            sequence_number: 0,  // Synthetic: not used for gap detection
            flow_id: FlowId::GenericL3 {
                src_ip,
                dst_ip,
                src_port,
                dst_port,
                protocol,
            },
            payload_length,
        }))
    }

    fn matches(&self, data: &[u8]) -> bool {
        // Minimum size: Ethernet (14) + IPv4 (20) + TCP/UDP header (8)
        if data.len() < 42 {
            return false;
        }

        // Check EtherType is IPv4 (0x0800)
        if data[12] != 0x08 || data[13] != 0x00 {
            return false;
        }

        // Check IP protocol is TCP (6) or UDP (17)
        // IP protocol field is at offset 23 (14 Ethernet + 9 into IP header)
        let protocol = data[23];
        protocol == IP_PROTOCOL_TCP || protocol == IP_PROTOCOL_UDP
    }

    fn protocol_name(&self) -> &str {
        "Generic-L3"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    /// Helper to create minimal valid TCP packet
    fn create_tcp_packet(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        seq: u32,
    ) -> Vec<u8> {
        let mut packet = Vec::new();

        // Ethernet header (14 bytes)
        packet.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]); // Dst MAC
        packet.extend_from_slice(&[0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB]); // Src MAC
        packet.extend_from_slice(&[0x08, 0x00]); // EtherType: IPv4

        // IPv4 header (20 bytes)
        packet.push(0x45); // Version 4, IHL 5
        packet.push(0x00); // DSCP/ECN
        let total_len: u16 = 20 + 20 + 10; // IP + TCP + payload
        packet.extend_from_slice(&total_len.to_be_bytes());
        packet.extend_from_slice(&[0x00, 0x00]); // Identification
        packet.extend_from_slice(&[0x00, 0x00]); // Flags
        packet.push(64); // TTL
        packet.push(IP_PROTOCOL_TCP); // Protocol: TCP
        packet.extend_from_slice(&[0x00, 0x00]); // Checksum
        packet.extend_from_slice(&src_ip);
        packet.extend_from_slice(&dst_ip);

        // TCP header (20 bytes minimum)
        packet.extend_from_slice(&src_port.to_be_bytes());
        packet.extend_from_slice(&dst_port.to_be_bytes());
        packet.extend_from_slice(&seq.to_be_bytes());
        packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // ACK
        packet.push(0x50); // Data offset (5 * 4 = 20 bytes)
        packet.push(0x00); // Flags
        packet.extend_from_slice(&[0xFF, 0xFF]); // Window
        packet.extend_from_slice(&[0x00, 0x00]); // Checksum
        packet.extend_from_slice(&[0x00, 0x00]); // Urgent pointer

        // Payload (10 bytes)
        packet.extend_from_slice(&[0u8; 10]);

        packet
    }

    /// Helper to create minimal valid UDP packet
    fn create_udp_packet(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
    ) -> Vec<u8> {
        let mut packet = Vec::new();

        // Ethernet header (14 bytes)
        packet.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        packet.extend_from_slice(&[0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB]);
        packet.extend_from_slice(&[0x08, 0x00]);

        // IPv4 header (20 bytes)
        packet.push(0x45);
        packet.push(0x00);
        let total_len: u16 = 20 + 8 + 10; // IP + UDP + payload
        packet.extend_from_slice(&total_len.to_be_bytes());
        packet.extend_from_slice(&[0x00, 0x00]);
        packet.extend_from_slice(&[0x00, 0x00]);
        packet.push(64);
        packet.push(IP_PROTOCOL_UDP); // Protocol: UDP
        packet.extend_from_slice(&[0x00, 0x00]);
        packet.extend_from_slice(&src_ip);
        packet.extend_from_slice(&dst_ip);

        // UDP header (8 bytes)
        packet.extend_from_slice(&src_port.to_be_bytes());
        packet.extend_from_slice(&dst_port.to_be_bytes());
        let udp_len: u16 = 8 + 10;
        packet.extend_from_slice(&udp_len.to_be_bytes());
        packet.extend_from_slice(&[0x00, 0x00]); // Checksum

        // Payload (10 bytes)
        packet.extend_from_slice(&[0u8; 10]);

        packet
    }

    #[test]
    fn test_generic_l3_parser_tcp() {
        let parser = GenericL3Parser;
        let packet = create_tcp_packet([192, 168, 1, 10], [10, 0, 0, 1], 12345, 80, 1000);

        // TCP gap detection is disabled: returns synthetic sequence 0
        let result = parser.parse_sequence(&packet).unwrap();
        assert!(result.is_some());

        let seq_info = result.unwrap();
        assert_eq!(seq_info.sequence_number, 0);  // Synthetic sequence

        match seq_info.flow_id {
            FlowId::GenericL3 {
                src_ip,
                dst_ip,
                src_port,
                dst_port,
                protocol,
            } => {
                assert_eq!(src_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
                assert_eq!(dst_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
                assert_eq!(src_port, 12345);
                assert_eq!(dst_port, 80);
                assert_eq!(protocol, IP_PROTOCOL_TCP);
            }
            _ => panic!("Expected GenericL3 flow ID"),
        }
    }

    #[test]
    fn test_generic_l3_parser_udp() {
        let parser = GenericL3Parser;
        let packet = create_udp_packet([192, 168, 1, 10], [10, 0, 0, 1], 53, 53);

        // UDP also returns synthetic sequence 0 (no gap detection for generic L3)
        let result = parser.parse_sequence(&packet).unwrap();
        assert!(result.is_some());

        let seq_info = result.unwrap();
        assert_eq!(seq_info.sequence_number, 0);  // Synthetic sequence

        match seq_info.flow_id {
            FlowId::GenericL3 {
                protocol,
                ..
            } => {
                assert_eq!(protocol, IP_PROTOCOL_UDP);
            }
            _ => panic!("Expected GenericL3 flow ID"),
        }
    }

    #[test]
    fn test_generic_l3_matches_tcp() {
        let parser = GenericL3Parser;
        let packet = create_tcp_packet([192, 168, 1, 10], [10, 0, 0, 1], 12345, 80, 1000);

        assert!(parser.matches(&packet));
    }

    #[test]
    fn test_generic_l3_matches_udp() {
        let parser = GenericL3Parser;
        let packet = create_udp_packet([192, 168, 1, 10], [10, 0, 0, 1], 53, 53);

        assert!(parser.matches(&packet));
    }

    #[test]
    fn test_generic_l3_wrong_protocol() {
        let parser = GenericL3Parser;
        let mut packet = create_tcp_packet([192, 168, 1, 10], [10, 0, 0, 1], 12345, 80, 1000);

        // Change protocol to ESP (50)
        packet[23] = 50;

        assert!(!parser.matches(&packet));
    }

    #[test]
    fn test_generic_l3_too_short() {
        let parser = GenericL3Parser;
        let packet = vec![0u8; 20];

        assert!(!parser.matches(&packet));
    }

    #[test]
    fn test_generic_l3_wrong_ethertype() {
        let parser = GenericL3Parser;
        let mut packet = create_tcp_packet([192, 168, 1, 10], [10, 0, 0, 1], 12345, 80, 1000);

        // Change EtherType to IPv6 (0x86DD)
        packet[12] = 0x86;
        packet[13] = 0xDD;

        assert!(!parser.matches(&packet));
    }
}
