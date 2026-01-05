pub mod flow;

#[cfg(feature = "cli")]
use crate::capture::PacketSource;
use crate::error::AnalysisError;
use crate::protocol::SequenceParser;
use crate::types::{AnalyzedPacket, AnalysisReport};

use self::flow::FlowTracker;

/// Generic packet analyzer that works with any combination of:
/// - Capture sources (file, live interface)
/// - Protocol parsers (MACsec, IPsec)
#[cfg(feature = "cli")]
pub struct PacketAnalyzer<S: PacketSource, P: SequenceParser> {
    source: S,
    parser: P,
    flow_tracker: FlowTracker,
}

#[cfg(feature = "cli")]
impl<S: PacketSource, P: SequenceParser> PacketAnalyzer<S, P> {
    pub fn new(source: S, parser: P) -> Self {
        Self {
            source,
            parser,
            flow_tracker: FlowTracker::new(),
        }
    }

    /// Run the analysis on all packets from the source
    pub fn analyze(&mut self) -> Result<AnalysisReport, AnalysisError> {
        let mut total_packets = 0;
        let mut gaps = Vec::new();

        // Process all packets from source
        while let Some(raw_packet) = self.source.next_packet()? {
            total_packets += 1;

            // Try to parse the packet
            if let Some(seq_info) = self.parser.parse_sequence(&raw_packet.data)? {
                // Create analyzed packet
                let analyzed = AnalyzedPacket {
                    sequence_number: seq_info.sequence_number,
                    flow_id: seq_info.flow_id,
                    timestamp: raw_packet.timestamp,
                    payload_length: seq_info.payload_length,
                };

                // Track the packet and detect gaps
                if let Some(gap) = self.flow_tracker.process_packet(analyzed) {
                    gaps.push(gap);
                }
            }
        }

        // Get flow statistics
        let flow_stats = self.flow_tracker.get_stats();

        let report = AnalysisReport {
            total_packets,
            protocol: self.parser.protocol_name().to_string(),
            gaps,
            flow_stats,
        };

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::source::PacketSource;
    use crate::protocol::parser::SequenceParser;
    use crate::types::{CaptureStats, RawPacket, SequenceInfo};
    use std::time::SystemTime;

    // Mock capture source for testing
    struct MockSource {
        packets: Vec<Vec<u8>>,
        index: usize,
    }

    impl MockSource {
        fn new(packets: Vec<Vec<u8>>) -> Self {
            Self { packets, index: 0 }
        }
    }

    impl PacketSource for MockSource {
        fn next_packet(&mut self) -> Result<Option<RawPacket>, crate::error::CaptureError> {
            if self.index < self.packets.len() {
                let data = self.packets[self.index].clone();
                self.index += 1;
                Ok(Some(RawPacket {
                    data,
                    timestamp: SystemTime::now(),
                    length: 0,
                }))
            } else {
                Ok(None)
            }
        }

        fn stats(&self) -> CaptureStats {
            CaptureStats {
                packets_received: self.index as u64,
                packets_dropped: 0,
            }
        }
    }

    // Mock parser for testing
    struct MockParser;

    impl SequenceParser for MockParser {
        fn parse_sequence(&self, data: &[u8]) -> Result<Option<SequenceInfo>, crate::error::ParseError> {
            // Simple mock: first byte is sequence number, second byte is flow id
            if data.len() < 2 {
                return Ok(None);
            }

            Ok(Some(SequenceInfo {
                sequence_number: data[0] as u32,
                flow_id: crate::types::FlowId::MACsec {
                    sci: data[1] as u64,
                },
                payload_length: data.len() - 2,
            }))
        }

        fn matches(&self, _data: &[u8]) -> bool {
            true
        }

        fn protocol_name(&self) -> &str {
            "Mock"
        }
    }

    #[test]
    fn test_analyzer_basic() {
        let packets = vec![
            vec![1, 1], // seq=1, flow=1
            vec![2, 1], // seq=2, flow=1
            vec![3, 1], // seq=3, flow=1
        ];

        let source = MockSource::new(packets);
        let parser = MockParser;
        let mut analyzer = PacketAnalyzer::new(source, parser);

        let report = analyzer.analyze().unwrap();
        assert_eq!(report.total_packets, 3);
        assert_eq!(report.gaps.len(), 0);
        assert_eq!(report.flow_stats.len(), 1);
    }

    #[test]
    fn test_analyzer_with_gaps() {
        let packets = vec![
            vec![1, 1], // seq=1, flow=1
            vec![2, 1], // seq=2, flow=1
            vec![4, 1], // seq=4, flow=1 (gap: missing 3)
        ];

        let source = MockSource::new(packets);
        let parser = MockParser;
        let mut analyzer = PacketAnalyzer::new(source, parser);

        let report = analyzer.analyze().unwrap();
        assert_eq!(report.total_packets, 3);
        assert_eq!(report.gaps.len(), 1);
        assert_eq!(report.gaps[0].expected, 3);
        assert_eq!(report.gaps[0].received, 4);
    }
}
