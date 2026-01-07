use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, Duration};

#[cfg(feature = "async")]
use dashmap::DashMap;

use crate::types::{AnalyzedPacket, FlowId, FlowStats, SequenceGap};

/// Tracks packet sequences for multiple flows with reordering support
#[cfg(not(feature = "async"))]
pub struct FlowTracker {
    flows: HashMap<FlowId, FlowState>,
    #[allow(dead_code)]
    reorder_window_size: u32,
}

/// Concurrent flow tracker using DashMap for lock-free access
#[cfg(feature = "async")]
pub struct FlowTracker {
    flows: DashMap<FlowId, FlowState>,
    #[allow(dead_code)]
    reorder_window_size: u32,
}

/// Internal state for a single flow
struct FlowState {
    highest_sequence: Option<u32>,
    /// Buffer for out-of-order packets: sequence -> packet
    reorder_buffer: BTreeMap<u32, AnalyzedPacket>,
    /// Expected next sequence number (for normal forward flow)
    expected_sequence: Option<u32>,
    packets_received: u64,
    gaps: Vec<SequenceGap>,
    first_sequence: Option<u32>,
    last_sequence: Option<u32>,
    min_gap: Option<u32>,
    max_gap: Option<u32>,

    // Enhanced statistics
    total_bytes: u64,
    first_timestamp: Option<SystemTime>,
    last_timestamp: Option<SystemTime>,
    previous_timestamp: Option<SystemTime>,  // For inter-arrival calculation
    min_inter_arrival_us: Option<u64>,       // Microseconds
    max_inter_arrival_us: Option<u64>,
    total_inter_arrival_us: u64,             // For average calculation
    inter_arrival_count: u64,                // Number of inter-arrival measurements
    protocol_distribution: HashMap<u8, u64>, // For GenericL3 flows
}

impl FlowState {
    fn new() -> Self {
        Self {
            highest_sequence: None,
            reorder_buffer: BTreeMap::new(),
            expected_sequence: None,
            packets_received: 0,
            gaps: Vec::new(),
            first_sequence: None,
            last_sequence: None,
            min_gap: None,
            max_gap: None,
            // Enhanced statistics initialization
            total_bytes: 0,
            first_timestamp: None,
            last_timestamp: None,
            previous_timestamp: None,
            min_inter_arrival_us: None,
            max_inter_arrival_us: None,
            total_inter_arrival_us: 0,
            inter_arrival_count: 0,
            protocol_distribution: HashMap::new(),
        }
    }
}

#[cfg(not(feature = "async"))]
impl FlowTracker {
    pub fn new() -> Self {
        Self::with_window_size(32)
    }

    /// Create tracker with custom reordering window size
    pub fn with_window_size(window_size: u32) -> Self {
        Self {
            flows: HashMap::new(),
            reorder_window_size: window_size,
        }
    }

    /// Process a packet and detect gaps
    /// Returns Some(gap) if a gap is detected, None otherwise
    pub fn process_packet(&mut self, packet: AnalyzedPacket) -> Option<SequenceGap> {
        let flow_id = packet.flow_id.clone();

        // Ensure flow exists
        self.flows
            .entry(flow_id.clone())
            .or_insert_with(FlowState::new);

        let mut gap = None;

        // Get flow state and process packet
        {
            let state = self.flows.get_mut(&flow_id).unwrap();
            state.packets_received += 1;

            // Track bytes received
            state.total_bytes += packet.payload_length as u64;

            // Track inter-arrival times
            if let Some(previous) = state.last_timestamp {
                if let Ok(duration) = packet.timestamp.duration_since(previous) {
                    let duration_us = duration.as_micros() as u64;

                    // Update min/max inter-arrival times
                    if state.min_inter_arrival_us.is_none() || duration_us < state.min_inter_arrival_us.unwrap() {
                        state.min_inter_arrival_us = Some(duration_us);
                    }
                    if state.max_inter_arrival_us.is_none() || duration_us > state.max_inter_arrival_us.unwrap() {
                        state.max_inter_arrival_us = Some(duration_us);
                    }

                    state.total_inter_arrival_us += duration_us;
                    state.inter_arrival_count += 1;
                }
            }

            // Track timestamps
            if state.first_timestamp.is_none() {
                state.first_timestamp = Some(packet.timestamp);
            }
            state.last_timestamp = Some(packet.timestamp);

            // Track protocol distribution for GenericL3 flows
            if let FlowId::GenericL3 { protocol, .. } = &flow_id {
                *state.protocol_distribution.entry(*protocol).or_insert(0) += 1;
            }

            // Skip gap detection for GenericL3 flows
            // GenericL3Parser returns synthetic sequence numbers (all zeros)
            // to enable flow tracking without gap detection
            if let FlowId::GenericL3 { .. } = &flow_id {
                return None;
            }

            // Record first sequence number
            if state.first_sequence.is_none() {
                state.first_sequence = Some(packet.sequence_number);
                state.expected_sequence = Some(packet.sequence_number.wrapping_add(1));
                state.highest_sequence = Some(packet.sequence_number);
                state.last_sequence = Some(packet.sequence_number);
                return None;
            }

            let current_seq = packet.sequence_number;
            let highest = state.highest_sequence.unwrap();
            state.last_sequence = Some(current_seq);

            // Check if this is the next expected packet
            if let Some(expected) = state.expected_sequence {
                if current_seq == expected {
                    // Packet is in order, advance expected
                    state.expected_sequence = Some(expected.wrapping_add(1));
                    state.highest_sequence = Some(current_seq);
                    return None;
                }
            }

            // Out-of-order packet
            if current_seq > highest {
                // Packet is ahead of all others we've seen
                // This is where we first detect missing packets
                let expected = state.expected_sequence.unwrap_or_else(|| highest.wrapping_add(1));

                if current_seq != expected {
                    // We have a gap: expected the next sequential, but got something higher
                    // Count missing packets: how many are between expected and current_seq
                    let gap_size = if current_seq > expected {
                        current_seq.wrapping_sub(expected)
                    } else {
                        // Handle wraparound
                        u32::MAX.wrapping_sub(expected).wrapping_add(current_seq).wrapping_add(1)
                    };

                    // Report the gap ONCE: from expected to current_seq
                    gap = Some(SequenceGap {
                        flow_id: flow_id.clone(),
                        expected,
                        received: current_seq,
                        gap_size,
                        timestamp: SystemTime::now(),
                    });

                    // Update expected to skip over the gap
                    state.expected_sequence = Some(current_seq.wrapping_add(1));
                }

                state.reorder_buffer.insert(current_seq, packet);
                state.highest_sequence = Some(current_seq);
            } else if current_seq < highest {
                // Out-of-order packet (arrived late)
                if !state.reorder_buffer.contains_key(&current_seq) {
                    // Check if this fills a gap
                    if let Some(expected) = state.expected_sequence {
                        if current_seq == expected {
                            // This packet fills the gap! Advance expected
                            state.expected_sequence = Some(expected.wrapping_add(1));
                        }
                    }
                    state.reorder_buffer.insert(current_seq, packet);
                }
            }
        }

        // Record gap if detected
        if let Some(ref gap_info) = gap {
            self.record_gap(&flow_id, gap_info.clone());
        }

        gap
    }

    /// Process the reorder buffer to see if gaps can be filled
    #[allow(dead_code)]
    fn process_reorder_buffer(&mut self, flow_id: &FlowId) -> Option<SequenceGap> {
        let state = self.flows.get_mut(flow_id)?;

        // Check if the highest buffered sequence is now expected
        if let Some((&highest_buffered, _)) = state.reorder_buffer.iter().next_back() {
            if let Some(expected) = state.expected_sequence {
                if highest_buffered == expected {
                    // We can process the buffer
                    // Remove and process sequential packets from buffer
                    let mut processed_gap = None;

                    while let Some((&seq, _packet)) = state.reorder_buffer.iter().next() {
                        if let Some(exp) = state.expected_sequence {
                            if seq == exp {
                                state.reorder_buffer.remove(&seq);
                                state.expected_sequence = Some(exp.wrapping_add(1));
                                state.highest_sequence = Some(seq);

                                // If we filled a gap, record it
                                if processed_gap.is_none()
                                    && exp > seq.wrapping_sub(1) + 1
                                {
                                    processed_gap = None; // Gap was filled
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    return processed_gap;
                }
            }
        }

        None
    }

    /// Detect a gap between expected and received sequence
    #[allow(dead_code)]
    fn detect_gap(
        &self,
        state: &FlowState,
        flow_id: &FlowId,
        received: u32,
        last_seen: u32,
    ) -> Option<SequenceGap> {
        let expected = if let Some(exp) = state.expected_sequence {
            exp
        } else {
            last_seen.wrapping_add(1)
        };

        if received != expected {
            // Calculate gap size
            let gap_size = if received > expected {
                received.wrapping_sub(expected)
            } else {
                // Handle wraparound
                (u32::MAX).wrapping_sub(expected).wrapping_add(received).wrapping_add(1)
            };

            let gap = SequenceGap {
                flow_id: flow_id.clone(),
                expected,
                received,
                gap_size,
                timestamp: SystemTime::now(),
            };

            return Some(gap);
        }

        None
    }

    /// Get statistics for all flows
    pub fn get_stats(&self) -> Vec<FlowStats> {
        self.flows
            .iter()
            .map(|(flow_id, state)| {
                let mut total_lost = 0u64;
                for gap in &state.gaps {
                    total_lost += gap.gap_size as u64;
                }

                // Calculate average inter-arrival time
                let avg_inter_arrival = if state.inter_arrival_count > 0 {
                    Some(Duration::from_micros(
                        state.total_inter_arrival_us / state.inter_arrival_count,
                    ))
                } else {
                    None
                };

                // Convert microseconds back to Duration for min/max
                let min_inter_arrival = state.min_inter_arrival_us.map(Duration::from_micros);
                let max_inter_arrival = state.max_inter_arrival_us.map(Duration::from_micros);

                FlowStats {
                    flow_id: flow_id.clone(),
                    packets_received: state.packets_received,
                    gaps_detected: state.gaps.len() as u64,
                    total_lost_packets: total_lost,
                    first_sequence: state.first_sequence,
                    last_sequence: state.last_sequence,
                    min_gap: state.min_gap,
                    max_gap: state.max_gap,
                    // Enhanced statistics
                    total_bytes: state.total_bytes,
                    first_timestamp: state.first_timestamp,
                    last_timestamp: state.last_timestamp,
                    min_inter_arrival,
                    max_inter_arrival,
                    avg_inter_arrival,
                    protocol_distribution: state.protocol_distribution.clone(),
                }
            })
            .collect()
    }

    /// Get all detected gaps
    pub fn get_gaps(&self) -> Vec<SequenceGap> {
        self.flows
            .values()
            .flat_map(|state| state.gaps.clone())
            .collect()
    }

    /// Record a gap detection (called internally)
    fn record_gap(&mut self, flow_id: &FlowId, gap: SequenceGap) {
        if let Some(state) = self.flows.get_mut(flow_id) {
            // Update min/max gap
            if state.min_gap.is_none() || gap.gap_size < state.min_gap.unwrap() {
                state.min_gap = Some(gap.gap_size);
            }
            if state.max_gap.is_none() || gap.gap_size > state.max_gap.unwrap() {
                state.max_gap = Some(gap.gap_size);
            }

            state.gaps.push(gap);
        }
    }
}

#[cfg(feature = "async")]
impl FlowTracker {
    /// Create a new concurrent flow tracker
    pub fn new() -> Self {
        Self::with_window_size(32)
    }

    /// Create tracker with custom reordering window size
    pub fn with_window_size(window_size: u32) -> Self {
        Self {
            flows: DashMap::new(),
            reorder_window_size: window_size,
        }
    }

    /// Process packet concurrently (lock-free with DashMap)
    pub fn process_packet(&self, packet: AnalyzedPacket) -> Option<SequenceGap> {
        let flow_id = packet.flow_id.clone();

        // DashMap handles locking internally per flow
        let mut state = self.flows
            .entry(flow_id.clone())
            .or_insert_with(FlowState::new);

        let mut gap = None;

        state.packets_received += 1;

        // Track bytes received
        state.total_bytes += packet.payload_length as u64;

        // Track inter-arrival times
        if let Some(previous) = state.last_timestamp {
            if let Ok(duration) = packet.timestamp.duration_since(previous) {
                let duration_us = duration.as_micros() as u64;

                // Update min/max inter-arrival times
                if state.min_inter_arrival_us.is_none() || duration_us < state.min_inter_arrival_us.unwrap() {
                    state.min_inter_arrival_us = Some(duration_us);
                }
                if state.max_inter_arrival_us.is_none() || duration_us > state.max_inter_arrival_us.unwrap() {
                    state.max_inter_arrival_us = Some(duration_us);
                }

                state.total_inter_arrival_us += duration_us;
                state.inter_arrival_count += 1;
            }
        }

        // Track timestamps
        if state.first_timestamp.is_none() {
            state.first_timestamp = Some(packet.timestamp);
        }
        state.last_timestamp = Some(packet.timestamp);

        // Track protocol distribution for GenericL3 flows
        if let FlowId::GenericL3 { protocol, .. } = &flow_id {
            *state.protocol_distribution.entry(*protocol).or_insert(0) += 1;
        }

        // Skip gap detection for GenericL3 flows
        // GenericL3Parser returns synthetic sequence numbers (all zeros)
        // to enable flow tracking without gap detection
        if let FlowId::GenericL3 { .. } = &flow_id {
            return None;
        }

        // Record first sequence
        if state.first_sequence.is_none() {
            state.first_sequence = Some(packet.sequence_number);
            state.expected_sequence = Some(packet.sequence_number.wrapping_add(1));
            state.highest_sequence = Some(packet.sequence_number);
            state.last_sequence = Some(packet.sequence_number);
            return None;
        }

        let current_seq = packet.sequence_number;
        let highest = state.highest_sequence.unwrap();
        state.last_sequence = Some(current_seq);

        // Check if next expected
        if let Some(expected) = state.expected_sequence {
            if current_seq == expected {
                state.expected_sequence = Some(expected.wrapping_add(1));
                state.highest_sequence = Some(current_seq);
                return None;
            }
        }

        // Out-of-order packet
        if current_seq > highest {
            let expected = state.expected_sequence.unwrap_or_else(|| highest.wrapping_add(1));

            if current_seq != expected {
                let gap_size = if current_seq > expected {
                    current_seq.wrapping_sub(expected)
                } else {
                    u32::MAX.wrapping_sub(expected).wrapping_add(current_seq).wrapping_add(1)
                };

                gap = Some(SequenceGap {
                    flow_id: flow_id.clone(),
                    expected,
                    received: current_seq,
                    gap_size,
                    timestamp: SystemTime::now(),
                });

                state.expected_sequence = Some(current_seq.wrapping_add(1));
            }

            state.reorder_buffer.insert(current_seq, packet);
            state.highest_sequence = Some(current_seq);
        } else if current_seq < highest {
            if !state.reorder_buffer.contains_key(&current_seq) {
                if let Some(expected) = state.expected_sequence {
                    if current_seq == expected {
                        state.expected_sequence = Some(expected.wrapping_add(1));
                    }
                }
                state.reorder_buffer.insert(current_seq, packet);
            }
        }

        // Record gap if detected
        if let Some(ref gap_info) = gap {
            // Update min/max gap stats
            if state.min_gap.is_none() || gap_info.gap_size < state.min_gap.unwrap() {
                state.min_gap = Some(gap_info.gap_size);
            }
            if state.max_gap.is_none() || gap_info.gap_size > state.max_gap.unwrap() {
                state.max_gap = Some(gap_info.gap_size);
            }
            state.gaps.push(gap_info.clone());
        }

        gap
    }

    /// Get statistics for all flows (concurrent-safe)
    pub fn get_stats(&self) -> Vec<FlowStats> {
        self.flows
            .iter()
            .map(|entry| {
                let flow_id = entry.key();
                let state = entry.value();

                let mut total_lost = 0u64;
                for gap in &state.gaps {
                    total_lost += gap.gap_size as u64;
                }

                // Calculate average inter-arrival time
                let avg_inter_arrival = if state.inter_arrival_count > 0 {
                    Some(Duration::from_micros(
                        state.total_inter_arrival_us / state.inter_arrival_count,
                    ))
                } else {
                    None
                };

                // Convert microseconds back to Duration for min/max
                let min_inter_arrival = state.min_inter_arrival_us.map(Duration::from_micros);
                let max_inter_arrival = state.max_inter_arrival_us.map(Duration::from_micros);

                FlowStats {
                    flow_id: flow_id.clone(),
                    packets_received: state.packets_received,
                    gaps_detected: state.gaps.len() as u64,
                    total_lost_packets: total_lost,
                    first_sequence: state.first_sequence,
                    last_sequence: state.last_sequence,
                    min_gap: state.min_gap,
                    max_gap: state.max_gap,
                    // Enhanced statistics
                    total_bytes: state.total_bytes,
                    first_timestamp: state.first_timestamp,
                    last_timestamp: state.last_timestamp,
                    min_inter_arrival,
                    max_inter_arrival,
                    avg_inter_arrival,
                    protocol_distribution: state.protocol_distribution.clone(),
                }
            })
            .collect()
    }

    /// Get all detected gaps (concurrent-safe)
    pub fn get_gaps(&self) -> Vec<SequenceGap> {
        self.flows
            .iter()
            .flat_map(|entry| entry.value().gaps.clone())
            .collect()
    }
}

#[cfg(not(feature = "async"))]
impl Default for FlowTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "async")]
impl Default for FlowTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_packet(seq: u32, flow_id: FlowId) -> AnalyzedPacket {
        AnalyzedPacket {
            sequence_number: seq,
            flow_id,
            timestamp: SystemTime::now(),
            payload_length: 100,
        }
    }

    #[test]
    fn test_sequential_packets_no_gap() {
        let mut tracker = FlowTracker::new();
        let flow = FlowId::MACsec { sci: 0x1234 };

        // Process sequential packets
        let gap1 = tracker.process_packet(create_packet(1, flow.clone()));
        let gap2 = tracker.process_packet(create_packet(2, flow.clone()));
        let gap3 = tracker.process_packet(create_packet(3, flow.clone()));

        assert!(gap1.is_none());
        assert!(gap2.is_none());
        assert!(gap3.is_none());

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].packets_received, 3);
        assert_eq!(stats[0].gaps_detected, 0);
    }

    #[test]
    fn test_gap_detection() {
        let mut tracker = FlowTracker::new();
        let flow = FlowId::MACsec { sci: 0x1234 };

        // Process packets with gap
        tracker.process_packet(create_packet(1, flow.clone()));
        tracker.process_packet(create_packet(2, flow.clone()));
        let gap = tracker.process_packet(create_packet(4, flow.clone())); // Missing 3

        assert!(gap.is_some());
        let gap_info = gap.unwrap();
        assert_eq!(gap_info.expected, 3);
        assert_eq!(gap_info.received, 4);
        assert_eq!(gap_info.gap_size, 1);
    }

    #[test]
    fn test_multiple_flows() {
        let mut tracker = FlowTracker::new();
        let flow1 = FlowId::MACsec { sci: 0x1111 };
        let flow2 = FlowId::MACsec { sci: 0x2222 };

        // Two independent flows
        tracker.process_packet(create_packet(1, flow1.clone()));
        tracker.process_packet(create_packet(1, flow2.clone()));
        tracker.process_packet(create_packet(2, flow1.clone()));
        tracker.process_packet(create_packet(2, flow2.clone()));

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 2);

        for stat in stats {
            assert_eq!(stat.packets_received, 2);
            assert_eq!(stat.gaps_detected, 0);
        }
    }

    #[test]
    fn test_wraparound_detection() {
        let mut tracker = FlowTracker::new();
        let flow = FlowId::MACsec { sci: 0x1234 };

        // Test sequence near wraparound
        tracker.process_packet(create_packet(u32::MAX, flow.clone()));
        // Next expected would be 0
        tracker.process_packet(create_packet(1, flow.clone()));

        let stats = tracker.get_stats();
        assert_eq!(stats[0].packets_received, 2);
        // Gap should be detected (expected 0, got 1)
        assert_eq!(stats[0].gaps_detected, 1);
    }

    #[test]
    fn test_total_bytes_tracking() {
        let mut tracker = FlowTracker::new();
        let flow = FlowId::MACsec { sci: 0x5678 };

        // Create packets with known payload lengths
        let mut pkt1 = create_packet(1, flow.clone());
        pkt1.payload_length = 100;
        let mut pkt2 = create_packet(2, flow.clone());
        pkt2.payload_length = 200;
        let mut pkt3 = create_packet(3, flow.clone());
        pkt3.payload_length = 150;

        tracker.process_packet(pkt1);
        tracker.process_packet(pkt2);
        tracker.process_packet(pkt3);

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].total_bytes, 450); // 100 + 200 + 150
    }

    #[test]
    fn test_timestamp_tracking() {
        let mut tracker = FlowTracker::new();
        let flow = FlowId::MACsec { sci: 0x9abc };

        let now = SystemTime::now();
        let mut pkt1 = create_packet(1, flow.clone());
        pkt1.timestamp = now;
        let mut pkt2 = create_packet(2, flow.clone());
        pkt2.timestamp = now;

        tracker.process_packet(pkt1);
        tracker.process_packet(pkt2);

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].first_timestamp, Some(now));
        assert_eq!(stats[0].last_timestamp, Some(now));
    }

    #[test]
    fn test_inter_arrival_time_tracking() {
        let mut tracker = FlowTracker::new();
        let flow = FlowId::MACsec { sci: 0xdef0 };

        let base_time = SystemTime::UNIX_EPOCH;

        // Create packets with 1ms apart (1000 microseconds)
        let mut pkt1 = create_packet(1, flow.clone());
        pkt1.timestamp = base_time + std::time::Duration::from_micros(0);

        let mut pkt2 = create_packet(2, flow.clone());
        pkt2.timestamp = base_time + std::time::Duration::from_micros(1000);

        let mut pkt3 = create_packet(3, flow.clone());
        pkt3.timestamp = base_time + std::time::Duration::from_micros(3000); // 2ms from pkt2

        tracker.process_packet(pkt1);
        tracker.process_packet(pkt2);
        tracker.process_packet(pkt3);

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 1);

        // Should have min=1000us, max=2000us, avg=1500us
        assert_eq!(stats[0].min_inter_arrival, Some(Duration::from_micros(1000)));
        assert_eq!(stats[0].max_inter_arrival, Some(Duration::from_micros(2000)));
        assert_eq!(stats[0].avg_inter_arrival, Some(Duration::from_micros(1500)));
    }

    #[test]
    fn test_single_packet_no_inter_arrival() {
        let mut tracker = FlowTracker::new();
        let flow = FlowId::MACsec { sci: 0x1234 };

        tracker.process_packet(create_packet(1, flow.clone()));

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].packets_received, 1);
        // No inter-arrival times should be recorded for first packet
        assert_eq!(stats[0].min_inter_arrival, None);
        assert_eq!(stats[0].max_inter_arrival, None);
        assert_eq!(stats[0].avg_inter_arrival, None);
    }

    #[test]
    fn test_protocol_distribution_tracking() {
        let mut tracker = FlowTracker::new();

        // Create GenericL3 flows with different protocols
        let tcp_flow = FlowId::GenericL3 {
            src_ip: "192.168.1.1".parse().unwrap(),
            dst_ip: "192.168.1.2".parse().unwrap(),
            src_port: 5000,
            dst_port: 80,
            protocol: 6, // TCP
        };

        let mut pkt1 = create_packet(1, tcp_flow.clone());
        let mut pkt2 = create_packet(2, tcp_flow.clone());
        let mut pkt3 = create_packet(3, tcp_flow.clone());

        tracker.process_packet(pkt1);
        tracker.process_packet(pkt2);
        tracker.process_packet(pkt3);

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].packets_received, 3);

        // Protocol distribution should contain TCP (6) -> 3 packets
        let tcp_count = stats[0].protocol_distribution.get(&6);
        assert_eq!(tcp_count, Some(&3));
    }

    #[test]
    fn test_multiple_flows_independent_statistics() {
        let mut tracker = FlowTracker::new();
        let flow1 = FlowId::MACsec { sci: 0x1111 };
        let flow2 = FlowId::MACsec { sci: 0x2222 };

        let base_time = SystemTime::UNIX_EPOCH;

        // Flow 1: 3 packets, 100 bytes each
        for i in 1..=3 {
            let mut pkt = create_packet(i, flow1.clone());
            pkt.payload_length = 100;
            pkt.timestamp = base_time + Duration::from_millis(i as u64);
            tracker.process_packet(pkt);
        }

        // Flow 2: 2 packets, 200 bytes each
        for i in 1..=2 {
            let mut pkt = create_packet(i, flow2.clone());
            pkt.payload_length = 200;
            pkt.timestamp = base_time + Duration::from_millis(i as u64 * 10);
            tracker.process_packet(pkt);
        }

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 2);

        // Verify statistics are independent
        for stat in stats {
            if stat.flow_id == flow1 {
                assert_eq!(stat.packets_received, 3);
                assert_eq!(stat.total_bytes, 300);
            } else if stat.flow_id == flow2 {
                assert_eq!(stat.packets_received, 2);
                assert_eq!(stat.total_bytes, 400);
            }
        }
    }

    #[test]
    fn test_combined_statistics_with_gaps() {
        let mut tracker = FlowTracker::new();
        let flow = FlowId::MACsec { sci: 0xabcd };

        let base_time = SystemTime::UNIX_EPOCH;

        // Packet 1
        let mut pkt1 = create_packet(1, flow.clone());
        pkt1.payload_length = 150;
        pkt1.timestamp = base_time;
        tracker.process_packet(pkt1);

        // Packet 2 (after 2ms) - continue sequence
        let mut pkt2 = create_packet(2, flow.clone());
        pkt2.payload_length = 150;
        pkt2.timestamp = base_time + Duration::from_millis(2);
        tracker.process_packet(pkt2);

        // Packet 4 (after 4ms) - gap detected
        let mut pkt3 = create_packet(4, flow.clone());
        pkt3.payload_length = 150;
        pkt3.timestamp = base_time + Duration::from_millis(4);
        tracker.process_packet(pkt3);

        // Packet 5
        let mut pkt4 = create_packet(5, flow.clone());
        pkt4.payload_length = 150;
        pkt4.timestamp = base_time + Duration::from_millis(5);
        tracker.process_packet(pkt4);

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 1);

        // Basic stats
        assert_eq!(stats[0].packets_received, 4);
        assert_eq!(stats[0].total_bytes, 600); // 4 * 150
        assert_eq!(stats[0].gaps_detected, 1);

        // Inter-arrival times: 2ms, 2ms, 1ms -> min=1000us, max=2000us, avg=1666us
        assert_eq!(stats[0].min_inter_arrival, Some(Duration::from_millis(1)));
        assert_eq!(stats[0].max_inter_arrival, Some(Duration::from_millis(2)));
    }
}
