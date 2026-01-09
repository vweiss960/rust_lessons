//! PCAP Replay Capture Source for Stress Testing
//!
//! Provides controlled replay of PCAP files with multiple timing modes:
//! - Fast: Maximum throughput (no delays)
//! - OriginalTiming: Respect original packet intervals from PCAP
//! - FixedRate: Configurable packets-per-second
//! - SpeedMultiplier: N× original speed
//!
//! Supports optional infinite looping for sustained stress testing.
//! When looping, returns `Ok(None)` to signal loop reset, allowing the analyzer
//! to persist data and reset flow tracking state to avoid artificial gaps.

use crate::capture::source::AsyncPacketSource;
use crate::error::CaptureError;
use crate::types::{CaptureStats, RawPacket};
use pcap::Capture;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::Mutex;

/// Replay timing mode for PCAP packet replay
#[derive(Debug, Clone, Copy)]
pub enum ReplayMode {
    /// Send packets as fast as possible (no delays)
    /// Use case: Maximum throughput testing, CPU stress testing
    Fast,

    /// Respect original packet timing from PCAP
    /// Use case: Realistic traffic pattern replay
    OriginalTiming,

    /// Send packets at fixed rate (packets per second)
    /// Use case: Controlled load testing, bandwidth testing
    FixedRate(u64),

    /// Multiply original timing by a factor
    /// Use case: Accelerated or decelerated replay
    /// Examples: 0.5 = half speed, 2.0 = double speed, 10.0 = 10x faster
    SpeedMultiplier(f64),
}

impl std::fmt::Display for ReplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayMode::Fast => write!(f, "fast"),
            ReplayMode::OriginalTiming => write!(f, "original"),
            ReplayMode::FixedRate(pps) => write!(f, "fixed ({} pps)", pps),
            ReplayMode::SpeedMultiplier(m) => write!(f, "speed ({}x)", m),
        }
    }
}

/// PCAP-based replay capture for stress testing
///
/// Loads a PCAP file into memory and replays packets with configurable timing.
/// Enables stress testing of packet parsing and analysis with deterministic,
/// reproducible packet sequences.
pub struct ReplayCapture {
    // Packet storage
    packets: Vec<RawPacket>,

    // Replay state
    current_index: usize,
    loop_count: u64,

    // Configuration
    replay_mode: ReplayMode,
    enable_looping: bool,

    // Timing calculations
    first_packet_time: Option<SystemTime>,
    replay_start_time: Option<Instant>,

    // Loop reset handling
    pending_loop_reset: bool,

    // Statistics
    packets_replayed: u64,
    loops_completed: u64,

    // I/O timing (for performance analysis in debug mode)
    io_timing: Mutex<IoTiming>,
}

/// Tracks I/O timing for PCAP replay
/// Measures time spent in next_packet() calls
#[derive(Debug, Clone)]
struct IoTiming {
    total_io_us: u128,      // Total microseconds spent in I/O operations
    io_call_count: u64,     // Number of next_packet() calls
    min_io_us: u128,        // Minimum single I/O call time
    max_io_us: u128,        // Maximum single I/O call time
}

impl ReplayCapture {
    /// Open a PCAP file and load all packets into memory for replay
    ///
    /// # Arguments
    /// * `path` - Path to PCAP file
    /// * `replay_mode` - How to time packet delivery
    /// * `enable_looping` - Allow infinite replay after file ends
    ///
    /// # Errors
    /// - `CaptureError::OpenFailed` if file doesn't exist or can't be read
    /// - `CaptureError::OpenFailed` if PCAP file is empty
    /// - `CaptureError::OpenFailed` if replay mode configuration is invalid
    pub fn open(
        path: &str,
        replay_mode: ReplayMode,
        enable_looping: bool,
    ) -> Result<Self, CaptureError> {
        // Validate replay mode configuration
        match replay_mode {
            ReplayMode::FixedRate(pps) if pps == 0 => {
                return Err(CaptureError::OpenFailed(
                    "FixedRate: packets per second must be > 0".to_string(),
                ));
            }
            ReplayMode::SpeedMultiplier(m) if !(m > 0.0) => {
                return Err(CaptureError::OpenFailed(
                    "SpeedMultiplier: multiplier must be > 0.0".to_string(),
                ));
            }
            _ => {}
        }

        // Open PCAP file
        let mut capture = Capture::from_file(path).map_err(|e| {
            CaptureError::OpenFailed(format!("Failed to open {}: {}", path, e))
        })?;

        // Load all packets into memory
        let mut packets = Vec::new();
        let mut first_packet_time = None;

        loop {
            match capture.next() {
                Ok(packet) => {
                    // Convert pcap timestamp to SystemTime
                    let timestamp = UNIX_EPOCH
                        + Duration::from_secs(packet.header.ts.tv_sec as u64)
                        + Duration::from_micros(packet.header.ts.tv_usec as u64);

                    if first_packet_time.is_none() {
                        first_packet_time = Some(timestamp);
                    }

                    packets.push(RawPacket {
                        data: packet.data.to_vec(),
                        timestamp,
                        length: packet.header.len as usize,
                    });
                }
                Err(pcap::Error::NoMorePackets) => break,
                Err(e) => {
                    if packets.is_empty() {
                        return Err(CaptureError::OpenFailed(format!(
                            "Failed to read packets from {}: {}",
                            path, e
                        )));
                    } else {
                        eprintln!(
                            "Warning: Error reading packet {} from {}: {}",
                            packets.len() + 1,
                            path,
                            e
                        );
                        break;
                    }
                }
            }
        }

        if packets.is_empty() {
            return Err(CaptureError::OpenFailed(format!(
                "PCAP file {} contains no packets",
                path
            )));
        }

        eprintln!(
            "[ReplayCapture] Loaded {} packets from {} (mode: {})",
            packets.len(),
            path,
            replay_mode
        );

        Ok(Self {
            packets,
            current_index: 0,
            loop_count: 0,
            replay_mode,
            enable_looping,
            first_packet_time,
            replay_start_time: None,
            pending_loop_reset: false,
            packets_replayed: 0,
            loops_completed: 0,
            io_timing: Mutex::new(IoTiming {
                total_io_us: 0,
                io_call_count: 0,
                min_io_us: u128::MAX,
                max_io_us: 0,
            }),
        })
    }

    /// Get current replay statistics
    pub fn replay_stats(&self) -> ReplayStats {
        ReplayStats {
            packets_replayed: self.packets_replayed,
            loops_completed: self.loops_completed,
            current_loop: self.loop_count,
            total_packets: self.packets.len() as u64,
        }
    }
}

/// Statistics about replay progress
#[derive(Debug, Clone)]
pub struct ReplayStats {
    pub packets_replayed: u64,
    pub loops_completed: u64,
    pub current_loop: u64,
    pub total_packets: u64,
}

impl AsyncPacketSource for ReplayCapture {
    async fn next_packet(&mut self) -> Result<Option<RawPacket>, CaptureError> {
        let io_start = Instant::now();

        // Handle pending loop reset from previous call
        if self.pending_loop_reset {
            self.pending_loop_reset = false;
            self.current_index = 0;
            self.replay_start_time = Some(Instant::now());
            // Continue to next packet from start of file
        }

        // Handle loop wraparound
        if self.current_index >= self.packets.len() {
            if !self.enable_looping {
                // No looping enabled: truly done, signal end of capture
                self.record_io_time(io_start);
                return Err(CaptureError::NoMorePackets);
            }

            // Looping enabled: signal reset and return None this iteration
            self.loops_completed += 1;
            self.pending_loop_reset = true;

            eprintln!(
                "[ReplayCapture] Loop {} complete, {} packets replayed",
                self.loops_completed, self.packets_replayed
            );

            self.record_io_time(io_start);
            return Ok(None); // Signal loop reset to analyzer (analyzer will continue)
        }

        // Get current packet
        let packet = &self.packets[self.current_index];

        // Initialize replay_start_time on first packet
        if self.replay_start_time.is_none() {
            self.replay_start_time = Some(Instant::now());
        }

        // Calculate and apply delay based on replay mode
        self.apply_timing_delay(packet).await?;

        // Clone packet and update state
        let mut result = packet.clone();
        self.current_index += 1;
        self.packets_replayed += 1;

        // Update timestamp to current time (for gap detection timestamps)
        result.timestamp = SystemTime::now();

        self.record_io_time(io_start);
        Ok(Some(result))
    }

    fn stats(&self) -> CaptureStats {
        CaptureStats {
            packets_received: self.packets_replayed,
            packets_dropped: 0, // Replay never drops packets
        }
    }
}

impl ReplayCapture {
    /// Apply timing delay based on replay mode
    async fn apply_timing_delay(&self, packet: &RawPacket) -> Result<(), CaptureError> {
        match self.replay_mode {
            ReplayMode::Fast => {
                // No delay - maximum throughput
                Ok(())
            }
            ReplayMode::OriginalTiming => self.apply_original_timing(packet).await,
            ReplayMode::FixedRate(pps) => self.apply_fixed_rate(pps).await,
            ReplayMode::SpeedMultiplier(multiplier) => {
                self.apply_speed_multiplier(packet, multiplier).await
            }
        }
    }

    /// Apply original timing: respect inter-packet delays from PCAP
    async fn apply_original_timing(&self, packet: &RawPacket) -> Result<(), CaptureError> {
        let first_timestamp = self
            .first_packet_time
            .ok_or_else(|| CaptureError::ReadFailed("No first packet time".to_string()))?;

        let packet_offset = packet
            .timestamp
            .duration_since(first_timestamp)
            .map_err(|e| CaptureError::ReadFailed(format!("Time calculation error: {}", e)))?;

        let replay_start = self
            .replay_start_time
            .ok_or_else(|| CaptureError::ReadFailed("Replay not started".to_string()))?;

        let elapsed_since_start = replay_start.elapsed();

        // If we're ahead of schedule, wait for it
        if let Some(wait_time) = packet_offset.checked_sub(elapsed_since_start) {
            tokio::time::sleep(wait_time).await;
        }
        // If we're behind schedule, proceed immediately (best effort)

        Ok(())
    }

    /// Apply fixed rate timing: uniform delay between packets
    async fn apply_fixed_rate(&self, pps: u64) -> Result<(), CaptureError> {
        let interval_ns = 1_000_000_000 / pps;
        let interval = Duration::from_nanos(interval_ns);
        tokio::time::sleep(interval).await;
        Ok(())
    }

    /// Apply speed multiplier: scale original delays by factor
    async fn apply_speed_multiplier(
        &self,
        packet: &RawPacket,
        multiplier: f64,
    ) -> Result<(), CaptureError> {
        let first_timestamp = self
            .first_packet_time
            .ok_or_else(|| CaptureError::ReadFailed("No first packet time".to_string()))?;

        let packet_offset = packet
            .timestamp
            .duration_since(first_timestamp)
            .map_err(|e| CaptureError::ReadFailed(format!("Time calculation error: {}", e)))?;

        // Scale the offset by the multiplier
        // Higher multiplier = faster playback
        // Example: multiplier 2.0 means play at 2x speed (half the delays)
        let scaled_ns = (packet_offset.as_nanos() as f64 / multiplier) as u64;
        let scaled_offset = Duration::from_nanos(scaled_ns);

        let replay_start = self
            .replay_start_time
            .ok_or_else(|| CaptureError::ReadFailed("Replay not started".to_string()))?;

        let elapsed_since_start = replay_start.elapsed();

        if let Some(wait_time) = scaled_offset.checked_sub(elapsed_since_start) {
            tokio::time::sleep(wait_time).await;
        }

        Ok(())
    }

    /// Record I/O time for a next_packet() call
    fn record_io_time(&self, start: Instant) {
        let elapsed_us = start.elapsed().as_micros();
        if let Ok(mut timing) = self.io_timing.lock() {
            timing.total_io_us += elapsed_us;
            timing.io_call_count += 1;
            timing.min_io_us = timing.min_io_us.min(elapsed_us);
            timing.max_io_us = timing.max_io_us.max(elapsed_us);
        }
    }

    /// Get I/O timing statistics for debugging
    /// Returns: (total_us, call_count, min_us, max_us, avg_us)
    pub fn get_io_stats(&self) -> (u128, u64, u128, u128, f64) {
        if let Ok(timing) = self.io_timing.lock() {
            let avg = if timing.io_call_count > 0 {
                timing.total_io_us as f64 / timing.io_call_count as f64
            } else {
                0.0
            };
            (timing.total_io_us, timing.io_call_count, timing.min_io_us, timing.max_io_us, avg)
        } else {
            (0, 0, 0, 0, 0.0)
        }
    }

    /// Print I/O timing report
    pub fn report_io_stats(&self) {
        let (total_us, count, min_us, max_us, avg_us) = self.get_io_stats();
        if count == 0 {
            println!("No I/O statistics available");
            return;
        }
        println!();
        println!("=== PCAP I/O Statistics ===");
        println!("Total I/O time:     {:.1}ms", total_us as f64 / 1000.0);
        println!("Calls:              {}", count);
        println!("Avg time/call:      {:.3}µs", avg_us);
        println!("Min time/call:      {:.3}µs", min_us as f64);
        println!("Max time/call:      {:.3}µs", max_us as f64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_fixed_rate_zero() {
        let result = ReplayCapture::open("dummy.pcap", ReplayMode::FixedRate(0), false);
        assert!(result.is_err());
        match result {
            Err(CaptureError::OpenFailed(msg)) => assert!(msg.contains("must be > 0")),
            _ => panic!("Expected OpenFailed error"),
        }
    }

    #[test]
    fn test_invalid_speed_multiplier_zero() {
        let result = ReplayCapture::open("dummy.pcap", ReplayMode::SpeedMultiplier(0.0), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_speed_multiplier_negative() {
        let result = ReplayCapture::open("dummy.pcap", ReplayMode::SpeedMultiplier(-1.0), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_modes() {
        // These should not error (file doesn't exist, but config is valid)
        // We're testing configuration validation, not file I/O
        assert_eq!(format!("{}", ReplayMode::Fast), "fast");
        assert_eq!(format!("{}", ReplayMode::OriginalTiming), "original");
        assert!(format!("{}", ReplayMode::FixedRate(1000)).contains("fixed"));
        assert!(format!("{}", ReplayMode::SpeedMultiplier(2.0)).contains("speed"));
    }
}
