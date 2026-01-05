use std::fmt;
use std::time::SystemTime;

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
pub enum FlowId {
    /// MACsec flow identified by Secure Channel Identifier (8 bytes)
    MACsec { sci: u64 },
    /// IPsec flow identified by SPI and destination IP (future)
    IPsec { spi: u32, dst_ip: [u8; 4] },
}

impl fmt::Display for FlowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlowId::MACsec { sci } => write!(f, "MACsec {{ sci: 0x{:016x} }}", sci),
            FlowId::IPsec { spi, dst_ip } => {
                write!(
                    f,
                    "IPsec {{ spi: 0x{:08x}, dst: {}.{}.{}.{} }}",
                    spi, dst_ip[0], dst_ip[1], dst_ip[2], dst_ip[3]
                )
            }
        }
    }
}

/// Gap detected in packet sequence
#[derive(Debug, Clone)]
pub struct SequenceGap {
    pub flow_id: FlowId,
    pub expected: u32,
    pub received: u32,
    pub gap_size: u32,
    pub timestamp: SystemTime,
}

/// Statistics for a single flow
#[derive(Debug, Clone)]
pub struct FlowStats {
    pub flow_id: FlowId,
    pub packets_received: u64,
    pub gaps_detected: u64,
    pub total_lost_packets: u64,
    pub first_sequence: Option<u32>,
    pub last_sequence: Option<u32>,
    pub min_gap: Option<u32>,
    pub max_gap: Option<u32>,
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
