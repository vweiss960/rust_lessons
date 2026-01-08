//! Async Live Packet Analyzer with Automatic Protocol Detection
//!
//! Captures packets from a live network interface, automatically detects protocol,
//! analyzes them for sequence gaps, and persists statistics to a database.
//!
//! Automatic protocol detection supports:
//! - MACsec (EtherType 0x88E5)
//! - IPsec ESP (IPv4 + IP protocol 50)
//! - Generic L3 (TCP/UDP)
//!
//! Usage:
//!   cargo build --bin live_analyzer --release
//!   ./target/release/live_analyzer <interface> <db_path> <capture_method>
//!
//! Examples:
//!   ./target/release/live_analyzer eth0 live.db pcap
//!   ./target/release/live_analyzer eth1 analysis.db pcap

use macsec_packet_analyzer::{
    capture::{AsyncPacketSource, PcapLiveCapture},
    analysis::flow::FlowTracker,
    db::{Database, DatabaseConfig},
    error::CaptureError,
    persist::PersistenceManager,
    protocol::ProtocolRegistry,
    types::AnalyzedPacket,
};

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <interface> <db_path> <capture_method>", args[0]);
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  interface       - Network interface (e.g., eth0)");
        eprintln!("  db_path         - SQLite database path for storing results");
        eprintln!("  capture_method  - Capture method: pcap");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} eth0 live.db pcap", args[0]);
        eprintln!();
        eprintln!("Automatic Protocol Detection:");
        eprintln!("  - MACsec (EtherType 0x88E5)");
        eprintln!("  - IPsec ESP (IPv4 + IP protocol 50)");
        eprintln!("  - Generic L3 (TCP/UDP on IPv4)");
        std::process::exit(1);
    }

    let interface = &args[1];
    let db_path = &args[2];
    let capture_method = args[3].to_lowercase();

    // Validate capture method
    if capture_method == "af_packet" {
        eprintln!("Error: AF_PACKET capture is not available (pre-existing compilation issue in crate)");
        eprintln!("Please use 'pcap' as the capture method instead");
        std::process::exit(1);
    }

    if !["pcap"].contains(&capture_method.as_str()) {
        eprintln!("Error: Unknown capture method '{}'. Use: pcap", capture_method);
        std::process::exit(1);
    }

    println!("Starting async packet analyzer with automatic protocol detection");
    println!("  Interface: {}", interface);
    println!("  Protocol: Auto-detect (MACsec, IPsec, Generic L3)");
    println!("  Database: {}", db_path);
    println!("  Capture: {}", capture_method);
    println!();

    // Initialize database
    let db = Database::open(&DatabaseConfig::sqlite(db_path))?;
    let db = Arc::new(Mutex::new(db));

    {
        let mut db = db.lock().map_err(|_| "Failed to lock database")?;
        db.initialize()?;
    }

    // Create persistence manager
    let persistence = PersistenceManager::new(Arc::clone(&db));

    // Create flow tracker
    let flow_tracker = Arc::new(Mutex::new(FlowTracker::new()));

    // Create protocol registry
    let registry = Arc::new(ProtocolRegistry::new());

    // Run the analyzer with PCAP
    analyze_with_pcap(interface, &registry, &flow_tracker, &persistence).await?;

    Ok(())
}

/// Analyze packets using PCAP capture
async fn analyze_with_pcap(
    interface: &str,
    registry: &Arc<ProtocolRegistry>,
    flow_tracker: &Arc<Mutex<FlowTracker>>,
    persistence: &PersistenceManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut capture = PcapLiveCapture::open(interface)?;

    println!("PCAP capture started on interface '{}'", interface);
    println!("Press Ctrl+C to stop and save results");
    println!();

    run_analyzer(&mut capture, registry, flow_tracker, persistence).await
}

/// Main packet processing loop with graceful shutdown and automatic protocol detection
async fn run_analyzer(
    capture: &mut PcapLiveCapture,
    registry: &Arc<ProtocolRegistry>,
    flow_tracker: &Arc<Mutex<FlowTracker>>,
    persistence: &PersistenceManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup signal handling for graceful shutdown
    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;

    let mut packet_count = 0u64;
    let mut gap_count = 0u64;
    let start_time = Instant::now();
    let mut last_persist = Instant::now();
    let persist_interval = Duration::from_secs(5);

    loop {
        tokio::select! {
            // Packet received
            packet_result = capture.next_packet() => {
                match packet_result {
                    Ok(Some(raw_packet)) => {
                        packet_count += 1;

                        // Automatic protocol detection using registry
                        if let Some(seq_info) = registry.detect_and_parse(&raw_packet.data)? {
                            // Create analyzed packet
                            let analyzed = AnalyzedPacket {
                                sequence_number: seq_info.sequence_number,
                                flow_id: seq_info.flow_id,
                                timestamp: raw_packet.timestamp,
                                payload_length: seq_info.payload_length,
                            };

                            // Track the packet and detect gaps
                            let mut tracker = flow_tracker.lock().map_err(|_| "Failed to lock flow tracker")?;
                            if tracker.process_packet(analyzed).is_some() {
                                gap_count += 1;
                            }
                        }

                        // Periodic persistence
                        if last_persist.elapsed() > persist_interval || packet_count % 10000 == 0 {
                            let tracker = flow_tracker.lock().map_err(|_| "Failed to lock flow tracker")?;
                            persistence.persist_flows(&tracker)?;
                            last_persist = Instant::now();

                            let elapsed = start_time.elapsed().as_secs_f64();
                            let pps = packet_count as f64 / elapsed;
                            let reg_stats = registry.get_stats();
                            let cache_hit_rate = if reg_stats.cache_hits + reg_stats.cache_misses > 0 {
                                (reg_stats.cache_hits as f64 / (reg_stats.cache_hits + reg_stats.cache_misses) as f64) * 100.0
                            } else {
                                0.0
                            };

                            println!(
                                "[{:.1}s] Packets: {}, Gaps: {}, Flows: {}, Rate: {:.0} pps, Cache: {:.1}%",
                                elapsed,
                                packet_count,
                                gap_count,
                                tracker.get_stats().len(),
                                pps,
                                cache_hit_rate
                            );
                        }
                    }
                    Ok(None) => {
                        // No more packets (shouldn't happen with live capture)
                        break;
                    }
                    Err(CaptureError::NoMorePackets) => {
                        // Expected end of capture
                        break;
                    }
                    Err(e) => {
                        eprintln!("Capture error: {}", e);
                        break;
                    }
                }
            }

            // Ctrl+C received
            _ = sigint.recv() => {
                println!("\nShutdown signal received. Flushing data...");
                break;
            }
        }
    }

    // Final persistence
    println!("Saving final statistics...");
    {
        let tracker = flow_tracker.lock().map_err(|_| "Failed to lock flow tracker")?;
        persistence.persist_flows(&tracker)?;
        print_analysis_report(&tracker, registry, packet_count, gap_count, start_time)?;
    }

    Ok(())
}

/// Print final analysis report with statistics
fn print_analysis_report(
    tracker: &FlowTracker,
    registry: &Arc<ProtocolRegistry>,
    packet_count: u64,
    gap_count: u64,
    start_time: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
