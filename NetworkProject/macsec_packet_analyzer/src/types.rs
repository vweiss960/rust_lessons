use std::fmt;
use std::time::SystemTime;
use std::net::IpAddr;
use std::collections::HashMap;
use std::time::Duration;

#[cfg(feature = "rest-api")]
use serde::{Deserialize, Serialize};

/// Raw packet data with metadata from capture source
#[derive(Debug, Clone)]
pub struct RawPacket {
    pub data: Vec<u8>,
    pub timestamp: SystemTime,
    pub length: usize,
}

/// Information extracted from a sequenced packet
#[derive(Debug, Clone)]
pub struct SequenceInfo {
    pub sequence_number: u32,
    pub flow_id: FlowId,
    pub payload_length: usize,
}

/// Packet analyzed with sequence and flow information
#[derive(Debug, Clone)]
pub struct AnalyzedPacket {
    pub sequence_number: u32,
    pub flow_id: FlowId,
    pub timestamp: SystemTime,
    pub payload_length: usize,
}

/// Flow identifier - protocol-specific
#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "rest-api", derive(Serialize, Deserialize))]
pub enum FlowId {
    /// MACsec flow identified by Secure Channel Identifier (8 bytes)
    MACsec { sci: u64 },

    /// IPsec ESP flow identified by SPI and destination IP
    /// SPI (Security Parameter Index) is the primary flow identifier
    /// dst_ip disambiguates when same SPI is used for multiple tunnels
    IPsec {
        spi: u32,
        dst_ip: IpAddr,
    },

    /// Generic L3 flow identified by 5-tuple
    /// Used for plain TCP/UDP traffic (non-encrypted)
    GenericL3 {
        src_ip: IpAddr,
        dst_ip: IpAddr,
        src_port: u16,
        dst_port: u16,
        protocol: u8,  // 6=TCP, 17=UDP
    },
}

impl FlowId {
    /// Create a FlowId from a string representation
    pub fn new(s: impl Into<String>) -> Self {
        let s = s.into();
        if s.starts_with("MACsec") {
            // Parse "MACsec { sci: 0x... }"
            if let Some(hex_str) = s.split("0x").nth(1) {
                if let Ok(sci) = u64::from_str_radix(hex_str.trim_end_matches(" }"), 16) {
                    return FlowId::MACsec { sci };
                }
            }
            FlowId::MACsec { sci: 0 }
        } else if s.starts_with("IPsec") {
            // Parse "IPsec { spi: 0x..., dst: ... }"
            // Simple parsing for now, can enhance later
            FlowId::IPsec {
                spi: 0,
                dst_ip: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            }
        } else if s.starts_with("TCP") || s.starts_with("UDP") {
            // Parse "TCP { ip:port -> ip:port }"
            // Simple fallback
            FlowId::GenericL3 {
                src_ip: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
                dst_ip: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
                src_port: 0,
                dst_port: 0,
                protocol: 6,
            }
        } else {
            FlowId::MACsec { sci: 0 }
        }
    }
}

impl fmt::Display for FlowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlowId::MACsec { sci } => {
                write!(f, "MACsec {{ sci: 0x{:016x} }}", sci)
            }
            FlowId::IPsec { spi, dst_ip } => {
                write!(f, "IPsec {{ spi: 0x{:08x}, dst: {} }}", spi, dst_ip)
            }
            FlowId::GenericL3 {
                src_ip,
                dst_ip,
                src_port,
                dst_port,
                protocol,
            } => {
                let proto_name = match *protocol {
                    6 => "TCP",
                    17 => "UDP",
                    _ => "Unknown",
                };
                write!(
                    f,
                    "{} {{ {}:{} -> {}:{} }}",
                    proto_name, src_ip, src_port, dst_ip, dst_port
                )
            }
        }
    }
}

/// Gap detected in packet sequence
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rest-api", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "rest-api", serde(crate = "serde"))]
pub struct SequenceGap {
    pub flow_id: FlowId,
    pub expected: u32,
    pub received: u32,
    pub gap_size: u32,
    #[cfg_attr(feature = "rest-api", serde(serialize_with = "serialize_systemtime"))]
    pub timestamp: SystemTime,
}

/// Statistics for a single flow
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rest-api", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "rest-api", serde(crate = "serde"))]
pub struct FlowStats {
    pub flow_id: FlowId,

    // Existing gap detection stats
    pub packets_received: u64,
    pub gaps_detected: u64,
    pub total_lost_packets: u64,
    pub first_sequence: Option<u32>,
    pub last_sequence: Option<u32>,
    pub min_gap: Option<u32>,
    pub max_gap: Option<u32>,

    // Enhanced statistics
    pub total_bytes: u64,
    #[cfg_attr(feature = "rest-api", serde(serialize_with = "serialize_systemtime_option"))]
    pub first_timestamp: Option<SystemTime>,
    #[cfg_attr(feature = "rest-api", serde(serialize_with = "serialize_systemtime_option"))]
    pub last_timestamp: Option<SystemTime>,
    pub min_inter_arrival: Option<Duration>,
    pub max_inter_arrival: Option<Duration>,
    pub avg_inter_arrival: Option<Duration>,

    // Protocol distribution (IP protocol number -> packet count)
    // For MACsec/IPsec: encrypted payload, so empty
    // For GenericL3: already in FlowId, so this is for inner protocols if needed
    #[cfg_attr(feature = "rest-api", serde(skip))]  // Skip HashMap in JSON
    pub protocol_distribution: HashMap<u8, u64>,
}

/// Serialize SystemTime to ISO 8601 string for REST API
#[cfg(feature = "rest-api")]
fn serialize_systemtime<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use chrono::{DateTime, Utc};
    let dt: DateTime<Utc> = (*time).into();
    serializer.serialize_str(&dt.to_rfc3339())
}

/// Serialize Option<SystemTime> to ISO 8601 string for REST API
#[cfg(feature = "rest-api")]
fn serialize_systemtime_option<S>(time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match time {
        Some(t) => {
            use chrono::{DateTime, Utc};
            let dt: DateTime<Utc> = (*t).into();
            serializer.serialize_str(&dt.to_rfc3339())
        }
        None => serializer.serialize_none(),
    }
}

/// Metrics from processing a single packet
/// Used for performance profiling in debug mode
#[derive(Debug)]
pub struct ProcessMetrics {
    pub detected: bool,     // Protocol was detected
    pub gap_detected: bool, // Gap was found
    pub detect_us: u128,    // Protocol detection time in microseconds (debug only)
    pub track_us: u128,     // Flow tracking time in microseconds (debug only)
    pub total_us: u128,     // Total processing time including overhead (debug only)
}

/// Statistics from packet capture source
#[derive(Debug, Clone)]
pub struct CaptureStats {
    pub packets_received: u64,
    pub packets_dropped: u64,
}

/// Complete analysis report
#[derive(Debug)]
pub struct AnalysisReport {
    pub total_packets: u64,
    pub protocol: String,
    pub gaps: Vec<SequenceGap>,
    pub flow_stats: Vec<FlowStats>,
}

impl AnalysisReport {
    pub fn new(protocol: String) -> Self {
        Self {
            total_packets: 0,
            protocol,
            gaps: Vec::new(),
            flow_stats: Vec::new(),
        }
    }
}
