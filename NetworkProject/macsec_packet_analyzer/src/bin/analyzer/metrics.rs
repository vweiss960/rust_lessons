//! Performance metrics and packet processing helpers
//! Provides utilities for measuring packet processing time in debug mode

use macsec_packet_analyzer::{
    analysis::flow::FlowTracker,
    protocol::ProtocolRegistry,
    types::{AnalyzedPacket, ProcessMetrics, RawPacket},
};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Instant, Duration}; // Duration used in ExecutionTimer

/// Aggregated timing statistics (thread-safe for accumulation)
pub struct TimingStats {
    total_detect_us: Mutex<u128>,
    total_track_us: Mutex<u128>,
    total_process_us: Mutex<u128>,  // Total time including overhead between operations
    packet_count: Mutex<u64>,
}

impl TimingStats {
    pub fn new() -> Self {
        Self {
            total_detect_us: Mutex::new(0),
            total_track_us: Mutex::new(0),
            total_process_us: Mutex::new(0),
            packet_count: Mutex::new(0),
        }
    }

    /// Record metrics from a single packet
    pub fn record(&self, metrics: &ProcessMetrics) {
        if let Ok(mut detect) = self.total_detect_us.lock() {
            *detect += metrics.detect_us;
        }
        if let Ok(mut track) = self.total_track_us.lock() {
            *track += metrics.track_us;
        }
        if let Ok(mut process) = self.total_process_us.lock() {
            *process += metrics.total_us;
        }
        if let Ok(mut count) = self.packet_count.lock() {
            *count += 1;
        }
    }

    /// Get average timing (in microseconds)
    /// Returns: (detect_avg, track_avg, total_avg)
    pub fn get_averages(&self) -> (f64, f64, f64) {
        let count = self.packet_count.lock().map(|c| *c).unwrap_or(0) as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let detect_total = self.total_detect_us.lock().map(|d| *d).unwrap_or(0) as f64;
        let track_total = self.total_track_us.lock().map(|t| *t).unwrap_or(0) as f64;
        let process_total = self.total_process_us.lock().map(|p| *p).unwrap_or(0) as f64;
        let detect_avg = detect_total / count;
        let track_avg = track_total / count;
        let process_avg = process_total / count;
        (detect_avg, track_avg, process_avg)
    }
}

/// Process a single packet and return metrics
/// In non-debug mode, timing fields are always zero (minimal overhead)
#[inline]
pub fn process_single_packet(
    raw_packet: &RawPacket,
    registry: &Arc<ProtocolRegistry>,
    flow_tracker: &Arc<FlowTracker>,
    debug: bool,
) -> Result<ProcessMetrics, Box<dyn std::error::Error>> {
    // Measure total time including function call overhead
    let total_start = if debug { Some(Instant::now()) } else { None };

    let detect_start = if debug { Some(Instant::now()) } else { None };
    let seq_info = registry.detect_and_parse(&raw_packet.data)?;
    let detect_us = detect_start.map(|s| s.elapsed().as_micros()).unwrap_or(0);

    let mut metrics = ProcessMetrics {
        detected: seq_info.is_some(),
        gap_detected: false,
        detect_us,
        track_us: 0,
        total_us: 0,
    };

    if let Some(seq_info) = seq_info {
        let analyzed = AnalyzedPacket {
            sequence_number: seq_info.sequence_number,
            flow_id: seq_info.flow_id,
            timestamp: raw_packet.timestamp,
            payload_length: seq_info.payload_length,
        };

        let track_start = if debug { Some(Instant::now()) } else { None };
        metrics.gap_detected = flow_tracker.process_packet(analyzed).is_some();
        metrics.track_us = track_start.map(|s| s.elapsed().as_micros()).unwrap_or(0);
    }

    // Calculate total time
    if let Some(timer) = total_start {
        metrics.total_us = timer.elapsed().as_micros();
    }

    Ok(metrics)
}

/// Track I/O and overhead timing phases
/// Helps identify where time is spent outside of packet processing
#[derive(Debug)]
pub struct ExecutionTimer {
    phase_name: &'static str,
    start: Instant,
}

impl ExecutionTimer {
    pub fn start(phase: &'static str) -> Self {
        Self {
            phase_name: phase,
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_us(&self) -> f64 {
        self.start.elapsed().as_micros() as f64
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    pub fn report(&self) {
        let elapsed = self.start.elapsed();
        if elapsed.as_secs() > 0 {
            println!("  [{}] {:.2}s", self.phase_name, elapsed.as_secs_f64());
        } else if elapsed.as_millis() > 0 {
            println!("  [{}] {:.1}ms", self.phase_name, elapsed.as_secs_f64() * 1000.0);
        } else {
            println!("  [{}] {:.1}µs", self.phase_name, elapsed.as_micros() as f64);
        }
    }
}
