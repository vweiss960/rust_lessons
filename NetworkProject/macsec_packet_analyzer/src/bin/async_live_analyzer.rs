//! Async Live Packet Analyzer
//!
//! Captures packets from a live network interface, analyzes them for sequence gaps,
//! and persists statistics to a database. Supports configurable capture method
//! (PCAP or AF_PACKET on Linux) and protocol parsing (MACsec, IPsec, Generic L3).
//!
//! Usage:
//!   cargo build --bin async_live_analyzer --release
//!   ./target/release/async_live_analyzer <interface> <protocol> <db_path> <capture_method>
//!
//! Examples:
//!   ./target/release/async_live_analyzer eth0 macsec live.db pcap
//!   ./target/release/async_live_analyzer eth0 ipsec live.db af_packet
//!   ./target/release/async_live_analyzer eth0 generic live.db pcap

use macsec_packet_analyzer::{
    capture::{AsyncPacketSource, PcapLiveCapture},
    analysis::flow::FlowTracker,
    db::{Database, DatabaseConfig},
    error::CaptureError,
    persist::PersistenceManager,
    protocol::{MACsecParser, IPsecParser, GenericL3Parser, SequenceParser},
    types::{AnalyzedPacket, FlowId},
};

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage: {} <interface> <protocol> <db_path> <capture_method>", args[0]);
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  interface       - Network interface (e.g., eth0)");
        eprintln!("  protocol        - Protocol parser: macsec, ipsec, or generic");
        eprintln!("  db_path         - SQLite database path for storing results");
        eprintln!("  capture_method  - Capture method: pcap or af_packet (Linux only)");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} eth0 macsec live.db pcap", args[0]);
        eprintln!("  {} eth0 ipsec live.db af_packet", args[0]);
        eprintln!("  {} eth0 generic live.db pcap", args[0]);
        std::process::exit(1);
    }

    let interface = &args[1];
    let protocol = args[2].to_lowercase();
    let db_path = &args[3];
    let capture_method = args[4].to_lowercase();

    // Validate protocol
    if !["macsec", "ipsec", "generic"].contains(&protocol.as_str()) {
        eprintln!("Error: Unknown protocol '{}'. Use: macsec, ipsec, or generic", protocol);
        std::process::exit(1);
    }

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

    println!("Starting async packet analyzer");
    println!("  Interface: {}", interface);
    println!("  Protocol: {}", protocol);
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

    // Run the analyzer with PCAP
    analyze_with_pcap(&protocol, interface, &flow_tracker, &persistence).await?;

    Ok(())
}

/// Analyze packets using PCAP capture
async fn analyze_with_pcap(
    protocol: &str,
    interface: &str,
    flow_tracker: &Arc<Mutex<FlowTracker>>,
    persistence: &PersistenceManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut capture = PcapLiveCapture::open(interface)?;

    println!("PCAP capture started on interface '{}'", interface);
    println!("Press Ctrl+C to stop and save results");
    println!();

    run_analyzer(protocol, &mut capture, flow_tracker, persistence).await
}

/// Main packet processing loop with graceful shutdown - dispatches to protocol-specific version
async fn run_analyzer(
    protocol: &str,
    capture: &mut PcapLiveCapture,
    flow_tracker: &Arc<Mutex<FlowTracker>>,
    persistence: &PersistenceManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup signal handling for graceful shutdown
    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;

    let mut packet_count = 0u64;
    let mut gap_count = 0u64;
    let start_time = Instant::now();
    let mut last_persist = Instant::now();
    let persist_interval = Duration::from_secs(5); // Persist every 5 seconds or 10k packets

    // Create parser upfront based on protocol
    match protocol {
        "macsec" => {
            run_analyzer_with_parser(
                &MACsecParser,
                capture,
                flow_tracker,
                persistence,
                &mut sigint,
                &mut packet_count,
                &mut gap_count,
                start_time,
                &mut last_persist,
                persist_interval,
            )
            .await?;
        }
        "ipsec" => {
            run_analyzer_with_parser(
                &IPsecParser,
                capture,
                flow_tracker,
                persistence,
                &mut sigint,
                &mut packet_count,
                &mut gap_count,
                start_time,
                &mut last_persist,
                persist_interval,
            )
            .await?;
        }
        "generic" => {
            run_analyzer_with_parser(
                &GenericL3Parser,
                capture,
                flow_tracker,
                persistence,
                &mut sigint,
                &mut packet_count,
                &mut gap_count,
                start_time,
                &mut last_persist,
                persist_interval,
            )
            .await?;
        }
        _ => unreachable!(),
    }

    // Final persistence
    println!("Saving final statistics...");
    {
        let tracker = flow_tracker.lock().map_err(|_| "Failed to lock flow tracker")?;
        persistence.persist_flows(&tracker)?;
        print_analysis_report(&tracker, packet_count, gap_count, start_time)?;
    }

    Ok(())
}

/// Generic packet processing loop for any parser type
async fn run_analyzer_with_parser<P: SequenceParser>(
    parser: &P,
    capture: &mut PcapLiveCapture,
    flow_tracker: &Arc<Mutex<FlowTracker>>,
    persistence: &PersistenceManager,
    sigint: &mut tokio::signal::unix::Signal,
    packet_count: &mut u64,
    gap_count: &mut u64,
    start_time: Instant,
    last_persist: &mut Instant,
    persist_interval: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        tokio::select! {
            // Packet received
            packet_result = capture.next_packet() => {
                match packet_result {
                    Ok(Some(raw_packet)) => {
                        *packet_count += 1;

                        // Try to parse the packet
                        if let Some(seq_info) = parser.parse_sequence(&raw_packet.data)? {
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
                                *gap_count += 1;
                            }
                        }

                        // Periodic persistence
                        if last_persist.elapsed() > persist_interval || *packet_count % 10000 == 0 {
                            let tracker = flow_tracker.lock().map_err(|_| "Failed to lock flow tracker")?;
                            persistence.persist_flows(&tracker)?;
                            *last_persist = Instant::now();

                            let elapsed = start_time.elapsed().as_secs_f64();
                            let pps = *packet_count as f64 / elapsed;
                            println!(
                                "[{:.1}s] Packets: {}, Gaps: {}, Flows: {}, Rate: {:.0} pps",
                                elapsed,
                                *packet_count,
                                *gap_count,
                                tracker.get_stats().len(),
                                pps
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

    Ok(())
}

/// Print final analysis report with statistics
fn print_analysis_report(
    tracker: &FlowTracker,
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

    println!();
    println!("=== Analysis Complete ===");
    println!("Total packets analyzed: {}", packet_count);
    println!("Total gaps detected: {}", gap_count);
    println!("Elapsed time: {:.2}s", elapsed.as_secs_f64());
    println!("Packet rate: {:.0} pps", pps);
    println!();

    let stats = tracker.get_stats();
    println!("Flows analyzed: {}", stats.len());
    println!();

    if !stats.is_empty() {
        println!("{:<50} {:>15} {:>15} {:>15} {:>15}", "Flow ID", "Packets", "Bytes", "Gaps", "Bandwidth");
        println!("{}", "-".repeat(110));

        for flow in stats {
            let bandwidth_mbps = if let (Some(first), Some(last)) = (flow.first_timestamp, flow.last_timestamp) {
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
        println!("  cargo run --features rest-api --bin api_server");
    }

    Ok(())
}
