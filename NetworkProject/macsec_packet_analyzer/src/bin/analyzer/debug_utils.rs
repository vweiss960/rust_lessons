//! Debug and reporting utilities for analysis output
//! Handles final analysis report printing and timing statistics

use crate::analyzer::TimingStats;
use macsec_packet_analyzer::analysis::flow::FlowTracker;
use macsec_packet_analyzer::protocol::ProtocolRegistry;
use std::sync::Arc;
use std::time::Instant;
use std::time::Duration;

/// Print final analysis report with statistics
pub fn print_analysis_report(
    tracker: &FlowTracker,
    registry: &Arc<ProtocolRegistry>,
    packet_count: u64,
    gap_count: u64,
    start_time: Instant,
    debug: bool,
    timing_stats: Option<Arc<TimingStats>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !debug {
        return Ok(());
    }

    let elapsed = start_time.elapsed();
    let pps = if elapsed.as_secs() > 0 {
        packet_count as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };

    let reg_stats = registry.get_stats();

    println!();
    println!("=== Analysis Complete ===");
    println!("Total packets analyzed: {}", packet_count);
    println!("Total gaps detected: {}", gap_count);
    println!("Elapsed time: {:.2}s", elapsed.as_secs_f64());
    println!("Packet rate: {:.0} pps", pps);
    println!();

    println!("=== Protocol Detection Stats ===");
    if reg_stats.cache_hits + reg_stats.cache_misses > 0 {
        let cache_hit_rate =
            (reg_stats.cache_hits as f64 / (reg_stats.cache_hits + reg_stats.cache_misses) as f64)
                * 100.0;
        println!("Cache hits: {} ({:.1}%)", reg_stats.cache_hits, cache_hit_rate);
    } else {
        println!("Cache hits: {}", reg_stats.cache_hits);
    }
    println!("Cache misses: {}", reg_stats.cache_misses);
    println!("EtherType fast path: {}", reg_stats.ethertype_fast_path);
    println!("Unknown protocol: {}", reg_stats.unknown_protocol);
    println!("Cache size: {}", reg_stats.cache_size);
    println!();

    let stats = tracker.get_stats();
    println!("Flows analyzed: {}", stats.len());
    println!();

    if !stats.is_empty() {
        println!(
            "{:<50} {:>15} {:>15} {:>15} {:>15}",
            "Flow ID", "Packets", "Bytes", "Gaps", "Bandwidth"
        );
        println!("{}", "-".repeat(110));

        for flow in stats {
            let bandwidth_mbps = if let (Some(first), Some(last)) =
                (flow.first_timestamp, flow.last_timestamp)
            {
                if let Ok(duration) = last.duration_since(first) {
                    let dur_secs = duration.as_secs_f64();
                    if dur_secs > 0.0 {
                        (flow.total_bytes as f64 * 8.0) / dur_secs / 1_000_000.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            };

            println!(
                "{:<50} {:>15} {:>15} {:>15} {:>14.2} Mbps",
                flow.flow_id.to_string(),
                flow.packets_received,
                flow.total_bytes,
                flow.gaps_detected,
                bandwidth_mbps
            );
        }

        println!();
        println!("Results saved to database. Query with:");
        println!("  cargo run --bin rest_api_server -- --db {}", "live.db");
    }

    // Display per-packet timing statistics if available
    if let Some(timing_stats_obj) = timing_stats {
        let (detect_avg, track_avg, total_avg) = timing_stats_obj.get_averages();
        println!();
        println!("=== Per-Packet Timing (Debug Mode) ===");
        println!("Protocol detection: {:.3}µs", detect_avg);
        println!("Flow tracking:      {:.3}µs", track_avg);
        println!("Measured subtotal:  {:.3}µs", detect_avg + track_avg);
        println!("Total (with overhead): {:.3}µs", total_avg);
        println!("Unaccounted overhead: {:.3}µs", total_avg - (detect_avg + track_avg));

        // Calculate I/O overhead
        let measured_us = detect_avg + track_avg;
        let total_us = 1_000_000.0 / pps.max(1.0);
        let overhead_us = total_us - measured_us;
        let overhead_pct = (overhead_us / total_us) * 100.0;

        println!();
        println!("=== I/O Overhead Analysis ===");
        println!("Measured processing: {:.3}µs", measured_us);
        println!("Total per-packet:    {:.3}µs", total_us);
        println!("I/O overhead:        {:.3}µs ({:.1}%)", overhead_us, overhead_pct);
        println!();
        let flow_count = tracker.get_stats().len();
        println!("Overhead sources:");
        println!("  - PCAP file I/O (reading packets from disk)");
        println!("  - Database persistence (async writes)");
        println!("  - Async event loop overhead (tokio scheduler)");
        println!("  - DashMap contention (lock-free lookups across {} flows)", flow_count);
    }

    Ok(())
}
