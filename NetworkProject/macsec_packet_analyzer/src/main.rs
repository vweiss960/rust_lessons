#![cfg_attr(not(feature = "cli"), allow(dead_code))]

// Synchronous version - when async feature is disabled
#[cfg(all(feature = "cli", not(feature = "async")))]
use std::env;
#[cfg(all(feature = "cli", not(feature = "async")))]
use std::time::SystemTime;

#[cfg(all(feature = "cli", not(feature = "async")))]
use macsec_packet_analyzer::{
    analysis::PacketAnalyzer, capture::FileCapture, protocol::MACsecParser,
};

#[cfg(all(feature = "cli", not(feature = "async")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get pcap file path from command line or use default
    let pcap_file = env::args()
        .nth(1)
        .unwrap_or_else(|| "macsec_traffic.pcap".to_string());

    println!("Analyzing MACsec packets from: {}\n", pcap_file);

    // Create capture and parser
    let source = FileCapture::open(&pcap_file)?;
    let parser = MACsecParser;

    // Create and run analyzer
    let mut analyzer = PacketAnalyzer::new(source, parser);
    let report = analyzer.analyze()?;

    // Print analysis results
    println!("Analysis Report:");
    println!("================");
    println!("Total packets processed: {}", report.total_packets);
    println!("Protocol: {}", report.protocol);
    println!("Flows detected: {}\n", report.flow_stats.len());

    // Print per-flow statistics
    for flow_stat in &report.flow_stats {
        println!("Flow: {}", flow_stat.flow_id);
        println!("  Packets received: {}", flow_stat.packets_received);
        println!("  Gaps detected: {}", flow_stat.gaps_detected);
        println!("  Lost packets (due to gaps): {}", flow_stat.total_lost_packets);

        if let Some(first) = flow_stat.first_sequence {
            if let Some(last) = flow_stat.last_sequence {
                println!("  Sequence range: {} - {}", first, last);
            }
        }

        if let Some(min) = flow_stat.min_gap {
            println!("  Min gap size: {}", min);
        }
        if let Some(max) = flow_stat.max_gap {
            println!("  Max gap size: {}", max);
        }
        println!();
    }

    // Print detailed gap list if any gaps were detected
    if !report.gaps.is_empty() {
        println!("Gaps Detected:");
        println!("==============");
        for (i, gap) in report.gaps.iter().enumerate() {
            let timestamp = gap
                .timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);

            println!(
                "  Gap {}: Flow {} - Expected seq {}, received {} (gap size: {})",
                i + 1,
                gap.flow_id,
                gap.expected,
                gap.received,
                gap.gap_size
            );
            println!("    Timestamp: {:.6}s", timestamp);
        }
    } else {
        println!("No gaps detected - all packets were received in order.");
    }

    Ok(())
}

// Async version - when async feature is enabled
#[cfg(all(feature = "cli", feature = "async"))]
use std::env;
#[cfg(all(feature = "cli", feature = "async"))]
use std::sync::Arc;

#[cfg(all(feature = "cli", feature = "async"))]
use crossbeam::channel::bounded;

#[cfg(all(feature = "cli", feature = "async"))]
use macsec_packet_analyzer::{
    analysis::flow::FlowTracker, protocol::{MACsecParser, SequenceParser},
    types::{AnalyzedPacket, RawPacket},
};

#[cfg(all(feature = "cli", feature = "async"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pcap_file = env::args()
        .nth(1)
        .unwrap_or_else(|| "macsec_traffic.pcap".to_string());

    println!("Starting continuous packet analysis from: {}\n", pcap_file);

    // Three-stage pipeline with bounded channels (backpressure)
    let (_raw_tx, raw_rx) = bounded::<RawPacket>(10000); // Capture → Parser
    let (analyzed_tx, analyzed_rx) = bounded::<AnalyzedPacket>(10000); // Parser → Analyzer

    // Shared flow tracker (concurrent access via DashMap)
    let tracker = Arc::new(FlowTracker::new());

    // Stage 1: Capture thread (packet producer)
    let capture_handle = tokio::task::spawn_blocking(move || {
        // TODO: Use async capture source (pcap_live, af_packet, xdp)
        // For now, placeholder
        println!("Capture thread started (placeholder)");
        // Future: while let Some(packet) = source.next_packet().await { raw_tx.send(packet); }
    });

    // Stage 2: Parser thread (packet transformer)
    let parser_handle = tokio::spawn(async move {
        let parser = MACsecParser;
        let mut parsed_count = 0u64;

        while let Ok(raw_packet) = raw_rx.recv() {
            if let Ok(Some(seq_info)) = parser.parse_sequence(&raw_packet.data) {
                let analyzed = AnalyzedPacket {
                    sequence_number: seq_info.sequence_number,
                    flow_id: seq_info.flow_id,
                    timestamp: raw_packet.timestamp,
                    payload_length: seq_info.payload_length,
                };

                if analyzed_tx.send(analyzed).is_err() {
                    break; // Downstream closed
                }
                parsed_count += 1;
            }
        }
        println!("Parser thread finished: {} packets parsed", parsed_count);
    });

    // Stage 3: Analyzer thread (gap detector)
    let tracker_clone = Arc::clone(&tracker);
    let analyzer_handle = tokio::spawn(async move {
        let mut packet_count = 0u64;
        let mut gap_count = 0u64;

        while let Ok(analyzed) = analyzed_rx.recv() {
            packet_count += 1;

            // Process concurrently (no &mut needed with DashMap)
            if let Some(gap) = tracker_clone.process_packet(analyzed) {
                gap_count += 1;
                println!(
                    "Gap {}: Expected {}, received {} (size: {})",
                    gap_count, gap.expected, gap.received, gap.gap_size
                );
            }

            // Periodic stats (every 10k packets)
            if packet_count % 10000 == 0 {
                println!(
                    "Progress: {} packets processed, {} gaps detected",
                    packet_count, gap_count
                );
            }
        }

        println!(
            "Analyzer thread finished: {} packets, {} gaps",
            packet_count, gap_count
        );
    });

    // Wait for all threads to complete
    let _ = tokio::join!(capture_handle, parser_handle, analyzer_handle);

    // Print final statistics
    println!("\n=== Final Report ===");
    let stats = tracker.get_stats();
    for flow_stat in stats {
        println!("\nFlow: {}", flow_stat.flow_id);
        println!("  Packets received: {}", flow_stat.packets_received);
        println!("  Gaps detected: {}", flow_stat.gaps_detected);
        println!("  Lost packets: {}", flow_stat.total_lost_packets);
        if let (Some(first), Some(last)) =
            (flow_stat.first_sequence, flow_stat.last_sequence)
        {
            println!("  Sequence range: {} - {}", first, last);
        }
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("This binary requires the 'cli' feature to be enabled.");
    eprintln!("Please build with: cargo build --features cli");
    std::process::exit(1);
}
