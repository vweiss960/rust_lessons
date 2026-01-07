#![cfg_attr(not(all(feature = "cli", feature = "rest-api")), allow(dead_code))]
//! Packet analyzer with database persistence
//!
//! This example demonstrates analyzing packets and persisting results to SQLite database.
//! The results can then be queried via the REST API server.
//!
//! Usage:
//!   cargo run --features "cli,rest-api" --bin analyze_with_db -- [pcap_file]
//!
//! Example:
//!   cargo run --features "cli,rest-api" --bin analyze_with_db -- macsec_traffic.pcap
//!
//! Then run the REST API server to query the results:
//!   cargo run --features "rest-api" --bin rest_api_server

#[cfg(all(feature = "cli", feature = "rest-api"))]
use macsec_packet_analyzer::{
    analysis::flow::FlowTracker,
    capture::{FileCapture, PacketSource},
    db::{Database, DatabaseConfig},
    error::CaptureError,
    protocol::{MACsecParser, SequenceParser},
    persist::PersistenceManager,
};
#[cfg(all(feature = "cli", feature = "rest-api"))]
use std::env;
#[cfg(all(feature = "cli", feature = "rest-api"))]
use std::sync::Arc;

#[cfg(all(feature = "cli", feature = "rest-api"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get pcap file path from command line or use default
    let pcap_file = env::args()
        .nth(1)
        .unwrap_or_else(|| "macsec_traffic.pcap".to_string());

    println!("Analyzing packets from: {}", pcap_file);
    println!("Persisting results to: analysis.db\n");

    // Initialize database
    let db_config = DatabaseConfig::sqlite_default();
    let mut db = Database::open(&db_config)?;
    db.initialize()?;
    println!("✓ Database initialized");

    let db = Arc::new(std::sync::Mutex::new(db));
    let persistence = PersistenceManager::new(Arc::clone(&db));

    // Create flow tracker and parser
    let tracker = FlowTracker::new();
    let parser = MACsecParser;

    // Open pcap file
    let mut source = FileCapture::open(&pcap_file)?;
    println!("✓ Pcap file opened");

    let mut packet_count = 0u64;
    let mut gap_count = 0u64;

    // Process each packet
    loop {
        match source.next_packet() {
            Ok(Some(raw_packet)) => {
                packet_count += 1;

                // Parse packet
                if let Ok(Some(seq_info)) = parser.parse_sequence(&raw_packet.data) {
                    let analyzed = macsec_packet_analyzer::types::AnalyzedPacket {
                        sequence_number: seq_info.sequence_number,
                        flow_id: seq_info.flow_id,
                        timestamp: raw_packet.timestamp,
                        payload_length: seq_info.payload_length,
                    };

                    // Process and detect gaps
                    if let Some(gap) = tracker.process_packet(analyzed) {
                        gap_count += 1;
                        println!(
                            "Gap {}: Expected {}, received {} (size: {})",
                            gap_count, gap.expected, gap.received, gap.gap_size
                        );
                    }
                }

                // Periodic stats
                if packet_count % 10000 == 0 {
                    println!("Processed: {} packets, {} gaps detected", packet_count, gap_count);
                }
            }
            Ok(None) => break, // No more packets
            Err(CaptureError::NoMorePackets) => break,
            Err(e) => {
                eprintln!("Error reading packet: {}", e);
                break;
            }
        }
    }

    println!("\n✓ Analysis complete: {} packets, {} gaps", packet_count, gap_count);

    // Persist all results to database
    println!("Persisting results to database...");
    persistence.persist_flows(&tracker)?;
    println!("✓ Results persisted to database\n");

    // Print summary
    println!("=== Analysis Summary ===");
    let stats = tracker.get_stats();
    println!("Flows detected: {}", stats.len());

    for flow_stat in stats {
        println!("\nFlow: {}", flow_stat.flow_id);
        println!("  Packets received: {}", flow_stat.packets_received);
        println!("  Gaps detected: {}", flow_stat.gaps_detected);
        println!("  Total lost packets: {}", flow_stat.total_lost_packets);

        if let (Some(first), Some(last)) = (flow_stat.first_sequence, flow_stat.last_sequence) {
            println!("  Sequence range: {} - {}", first, last);
        }
        if let Some(min) = flow_stat.min_gap {
            println!("  Min gap size: {}", min);
        }
        if let Some(max) = flow_stat.max_gap {
            println!("  Max gap size: {}", max);
        }
    }

    // Print database summary stats
    println!("\n=== Database Summary ===");
    {
        let db = db.lock().map_err(|_| "Failed to lock database")?;
        let summary = db.get_summary_stats()?;
        println!("Total flows in DB: {}", summary.total_flows);
        println!("Total packets: {}", summary.total_packets_received);
        println!("Total gaps: {}", summary.total_gaps_detected);
        println!("Total lost packets: {}", summary.total_lost_packets);
        println!("Max gap size: {}", summary.max_gap_size);
    }

    println!("\nYou can now query the results using the REST API server:");
    println!("  cargo run --features rest-api --bin rest_api_server");

    Ok(())
}

#[cfg(not(all(feature = "cli", feature = "rest-api")))]
fn main() {
    eprintln!("This binary requires both 'cli' and 'rest-api' features to be enabled.");
    eprintln!("Please build with: cargo build --features cli,rest-api");
    std::process::exit(1);
}
