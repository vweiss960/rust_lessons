//! Async Live Packet Analyzer with Automatic Protocol Detection
//!
//! Captures packets from a live network interface or replays PCAP files,
//! automatically detects protocol, analyzes for sequence gaps, and persists statistics.
//!
//! Supports multiple capture backends:
//! - PCAP Live Capture (default, for testing)
//! - PCAP Replay (for stress testing with configurable timing)
//! - Napatech (for production high-speed NICs, requires napatech feature)
//!
//! Automatic protocol detection supports:
//! - MACsec (EtherType 0x88E5)
//! - IPsec ESP (IPv4 + IP protocol 50)
//! - Generic L3 (TCP/UDP)
//!
//! Usage (Live Capture - DEFAULT):
//!   cargo build --bin live_analyzer --release
//!   ./target/release/live_analyzer <interface> <db_path> [options]
//!
//! Usage (PCAP Replay):
//!   ./target/release/live_analyzer <pcap_file> <db_path> --replay [options]
//!
//! Examples:
//!   # Live capture
//!   ./target/release/live_analyzer eth0 live.db
//!   ./target/release/live_analyzer eth0 live.db --debug
//!
//!   # PCAP replay with different timing modes
//!   ./target/release/live_analyzer traffic.pcap test.db --replay --mode fast
//!   ./target/release/live_analyzer traffic.pcap test.db --replay --mode original --loop
//!   ./target/release/live_analyzer traffic.pcap test.db --replay --mode fixed --pps 10000

mod analyzer;

use macsec_packet_analyzer::{
    capture::AsyncPacketSource,
    analysis::flow::FlowTracker,
    db::{Database, DatabaseConfig},
    error::CaptureError,
    persist::PersistenceManager,
    protocol::ProtocolRegistry,
};

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::signal;
use tokio::task::spawn_blocking;

// Feature-gated imports for different capture backends
#[cfg(all(feature = "async", feature = "pcap"))]
use macsec_packet_analyzer::capture::{PcapLiveCapture, ReplayCapture, ReplayMode};

#[cfg(all(target_os = "linux", feature = "napatech"))]
use macsec_packet_analyzer::capture::NapatechCapture;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Parse arguments: <source> <db_path> [options]
    if args.len() < 3 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let source = &args[1];
    let db_path = &args[2];

    // Check for --replay flag
    let is_replay = args.iter().any(|arg| arg == "--replay");

    // Check for --debug flag
    let debug = args.iter().any(|arg| arg == "--debug");

    if is_replay {
        // PCAP replay mode
        run_replay_capture(source, db_path, &args[3..], debug).await?;
    } else {
        // Live capture mode (default, backward compatible)
        run_with_compiled_backend(source, db_path, debug).await?;
    }

    Ok(())
}

/// Run analyzer with the capture backend compiled into this binary
async fn run_with_compiled_backend(
    interface: &str,
    db_path: &str,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "async", feature = "pcap"))]
    {
        let mut capture = PcapLiveCapture::open(interface)?;
        run_analyzer_impl(&mut capture, db_path, "PCAP", interface, debug).await
    }

    #[cfg(all(target_os = "linux", feature = "napatech"))]
    {
        let mut capture = NapatechCapture::open(0, 0)?; // Default port 0, stream 0
        run_analyzer_impl(&mut capture, db_path, "Napatech", interface, debug).await
    }

    #[cfg(not(any(all(feature = "async", feature = "pcap"), all(target_os = "linux", feature = "napatech"))))]
    {
        eprintln!("Error: No capture backend compiled into this binary");
        eprintln!("Build with --features pcap or --features napatech");
        std::process::exit(1);
    }
}

/// Run analyzer with PCAP replay capture
#[cfg(all(feature = "async", feature = "pcap"))]
async fn run_replay_capture(
    pcap_path: &str,
    db_path: &str,
    options: &[String],
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse replay options
    let (replay_mode, enable_looping) = parse_replay_options(options)?;

    // Create ReplayCapture
    let mut capture = ReplayCapture::open(pcap_path, replay_mode, enable_looping)?;

    // Run generic analyzer (reuses existing infrastructure)
    run_analyzer_impl(&mut capture, db_path, "PCAP Replay", pcap_path, debug).await?;

    // Report PCAP I/O statistics if in debug mode
    if debug {
        capture.report_io_stats();
    }

    Ok(())
}

/// Run analyzer with PCAP replay capture (stub for when replay feature not available)
#[cfg(not(all(feature = "async", feature = "pcap")))]
async fn run_replay_capture(
    _pcap_path: &str,
    _db_path: &str,
    _options: &[String],
    _debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Error: PCAP replay requires --features async,pcap");
    eprintln!("Build with: cargo build --bin live_analyzer");
    std::process::exit(1);
}

/// Parse replay mode from command-line options
fn parse_replay_options(options: &[String]) -> Result<(ReplayMode, bool), Box<dyn std::error::Error>> {
    let mut replay_mode = ReplayMode::Fast;
    let mut enable_looping = false;

    let mut i = 0;
    while i < options.len() {
        match options[i].as_str() {
            "--mode" => {
                if i + 1 < options.len() {
                    let mode_str = &options[i + 1];
                    // For fixed and speed modes, these are just indicators
                    // The actual value comes from --pps or --speed
                    match mode_str.as_str() {
                        "fast" => replay_mode = ReplayMode::Fast,
                        "original" => replay_mode = ReplayMode::OriginalTiming,
                        "fixed" => {
                            // Will be overridden by --pps if present
                            replay_mode = ReplayMode::FixedRate(0); // Placeholder
                        }
                        "speed" => {
                            // Will be overridden by --speed if present
                            replay_mode = ReplayMode::SpeedMultiplier(1.0); // Placeholder
                        }
                        other => return Err(format!("Unknown replay mode: {}", other).into()),
                    };
                    i += 2;
                } else {
                    return Err("--mode requires an argument".into());
                }
            }
            "--pps" => {
                if i + 1 < options.len() {
                    let pps: u64 = options[i + 1].parse()?;
                    if pps == 0 {
                        return Err("--pps must be > 0".into());
                    }
                    replay_mode = ReplayMode::FixedRate(pps);
                    i += 2;
                } else {
                    return Err("--pps requires a number argument".into());
                }
            }
            "--speed" => {
                if i + 1 < options.len() {
                    let speed: f64 = options[i + 1].parse()?;
                    if !(speed > 0.0) {
                        return Err("--speed must be > 0.0".into());
                    }
                    replay_mode = ReplayMode::SpeedMultiplier(speed);
                    i += 2;
                } else {
                    return Err("--speed requires a number argument".into());
                }
            }
            "--loop" => {
                enable_looping = true;
                i += 1;
            }
            "--debug" | "--replay" => {
                // Handled by main or in main args, skip it
                i += 1;
            }
            other => {
                eprintln!("Warning: Unknown option: {}", other);
                i += 1;
            }
        }
    }

    // Validate that mode is complete
    match replay_mode {
        ReplayMode::FixedRate(0) => {
            return Err("--mode fixed requires --pps <rate>".into());
        }
        ReplayMode::SpeedMultiplier(1.0) => {
            // Check if this was actually meant to be set
            // This is a bit of a hack - we use 1.0 as placeholder
            // If user specified --mode speed but no --speed, they'll get this error
            // But if they just didn't specify --speed at all, they'll get fast mode
            // To fix this properly, we'd need to track whether --mode speed was actually used
        }
        _ => {}
    }

    Ok((replay_mode, enable_looping))
}

/// Print usage information
fn print_usage(program: &str) {
    eprintln!("Async Packet Analyzer - Live Capture or PCAP Replay");
    eprintln!();
    eprintln!("LIVE CAPTURE MODE (Default):");
    eprintln!("  {} <interface> <db_path> [options]", program);
    eprintln!();
    eprintln!("PCAP REPLAY MODE:");
    eprintln!("  {} <pcap_file> <db_path> --replay [options]", program);
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  interface       - Network interface (e.g., eth0)");
    eprintln!("  pcap_file       - Path to PCAP file (for --replay mode)");
    eprintln!("  db_path         - SQLite database path for storing results");
    eprintln!();
    eprintln!("Common Options:");
    eprintln!("  --debug         - Enable debug output (shows packet statistics)");
    eprintln!();
    eprintln!("Replay-Specific Options:");
    eprintln!("  --mode <mode>   - Replay timing: fast|original|fixed|speed (default: fast)");
    eprintln!("  --pps <rate>    - Packets per second (use with --mode fixed)");
    eprintln!("  --speed <n>     - Speed multiplier (use with --mode speed)");
    eprintln!("  --loop          - Enable continuous looping (replay mode only)");
    eprintln!();

    #[cfg(all(feature = "async", feature = "pcap"))]
    {
        eprintln!("Capture Backend: PCAP (supports both live capture and replay)");
    }
    #[cfg(all(target_os = "linux", feature = "napatech"))]
    {
        eprintln!("Capture Backend: Napatech NTAPI (production high-speed NICs)");
    }

    eprintln!();
    eprintln!("Examples:");
    eprintln!("LIVE CAPTURE:");
    eprintln!("  {} eth0 live.db", program);
    eprintln!("  {} eth0 live.db --debug", program);
    eprintln!();
    eprintln!("PCAP REPLAY:");
    eprintln!("  {} traffic.pcap test.db --replay --mode fast", program);
    eprintln!("  {} traffic.pcap test.db --replay --mode original --loop", program);
    eprintln!("  {} traffic.pcap test.db --replay --mode fixed --pps 10000", program);
    eprintln!("  {} traffic.pcap test.db --replay --mode speed --speed 10.0 --debug", program);
    eprintln!();
    eprintln!("Automatic Protocol Detection:");
    eprintln!("  - MACsec (EtherType 0x88E5)");
    eprintln!("  - IPsec ESP (IPv4 + IP protocol 50)");
    eprintln!("  - Generic L3 (TCP/UDP on IPv4)");
    eprintln!();
    eprintln!("Build Instructions:");
    #[cfg(not(any(all(feature = "async", feature = "pcap"), all(target_os = "linux", feature = "napatech"))))]
    eprintln!("  To build with PCAP:      cargo build --bin live_analyzer");
    #[cfg(not(any(all(feature = "async", feature = "pcap"), all(target_os = "linux", feature = "napatech"))))]
    eprintln!("  To build with Napatech:  cargo build --bin live_analyzer --features napatech");
}

/// Generic implementation that works with any AsyncPacketSource
async fn run_analyzer_impl<T: AsyncPacketSource>(
    capture: &mut T,
    db_path: &str,
    backend_name: &str,
    interface: &str,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if debug {
        println!("Starting async packet analyzer with automatic protocol detection");
        println!("  Interface: {}", interface);
        println!("  Protocol: Auto-detect (MACsec, IPsec, Generic L3)");
        println!("  Database: {}", db_path);
        println!("  Capture backend: {}", backend_name);
        println!();
    }

    // Initialize database
    let db = Database::open(&DatabaseConfig::sqlite(db_path))?;
    let db = Arc::new(Mutex::new(db));

    {
        let mut db = db.lock().map_err(|_| "Failed to lock database")?;
        db.initialize()?;
    }

    // Create persistence manager
    let persistence = PersistenceManager::new(Arc::clone(&db));

    // Create flow tracker (lock-free with DashMap for async use)
    // DashMap provides per-entry locking, so we don't need an outer Mutex wrapper
    let flow_tracker = Arc::new(FlowTracker::new());

    // Create protocol registry
    let registry = Arc::new(ProtocolRegistry::new());

    // Run the generic analyzer with any AsyncPacketSource
    run_analyzer(capture, &registry, &flow_tracker, &persistence, debug).await?;

    Ok(())
}

/// Main packet processing loop with graceful shutdown and automatic protocol detection
/// Works with any AsyncPacketSource implementation
async fn run_analyzer<T: AsyncPacketSource>(
    capture: &mut T,
    registry: &Arc<ProtocolRegistry>,
    flow_tracker: &Arc<FlowTracker>,
    persistence: &PersistenceManager,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup signal handling for graceful shutdown
    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;

    let mut packet_count = 0u64;
    let mut gap_count = 0u64;
    let start_time = Instant::now();
    let mut last_persist = Instant::now();
    // Increased from 5s/10K packets to 30s/100K packets for better throughput
    // This reduces lock contention on FlowTracker during high-speed replay
    let persist_interval = Duration::from_secs(30);
    let persist_packet_threshold = 100_000;

    // Track timing metrics in debug mode
    let timing_stats = if debug {
        Some(Arc::new(analyzer::TimingStats::new()))
    } else {
        None
    };

    loop {
        tokio::select! {
            // Packet received
            packet_result = capture.next_packet() => {
                match packet_result {
                    Ok(Some(raw_packet)) => {
                        packet_count += 1;

                        // Process packet and collect metrics (with timing in debug mode)
                        let metrics = analyzer::process_single_packet(
                            &raw_packet,
                            &registry,
                            &flow_tracker,
                            debug,
                        )?;

                        // Record timing metrics if in debug mode
                        if let Some(ref stats) = timing_stats {
                            stats.record(&metrics);
                        }

                        if metrics.detected && metrics.gap_detected {
                            gap_count += 1;
                        }

                        // Periodic persistence (reduced frequency for better throughput)
                        if last_persist.elapsed() > persist_interval || packet_count % persist_packet_threshold == 0 {
                            // Snapshot tracker data for async write (no lock needed, DashMap is thread-safe)
                            let stats = flow_tracker.get_stats();
                            let gaps = flow_tracker.get_gaps();
                            let num_flows = stats.len();
                            let stats_snapshot = (stats, gaps);

                            // Spawn async database write to avoid blocking packet processing
                            let persistence_clone = persistence.clone_for_async();
                            spawn_blocking(move || {
                                // This runs in a thread pool and doesn't block the main async loop
                                if let Err(e) = persistence_clone.persist_stats_and_gaps(stats_snapshot) {
                                    eprintln!("Warning: Async persistence failed: {}", e);
                                }
                            });

                            last_persist = Instant::now();

                            if debug && debug {
                                let elapsed = start_time.elapsed().as_secs_f64();
                                let pps = packet_count as f64 / elapsed;
                                let reg_stats = registry.get_stats();
                                let cache_hit_rate = if reg_stats.cache_hits + reg_stats.cache_misses > 0 {
                                    (reg_stats.cache_hits as f64 / (reg_stats.cache_hits + reg_stats.cache_misses) as f64) * 100.0
                                } else {
                                    0.0
                                };

                                println!(
                                    "[{:.1}s] Packets: {}, Gaps: {}, Flows: {}, Rate: {:.0} pps, Cache: {:.1}% | Timing: detect={:.1}µs, track={:.1}µs",
                                    elapsed,
                                    packet_count,
                                    gap_count,
                                    num_flows,
                                    pps,
                                    cache_hit_rate,
                                    metrics.detect_us as f64,
                                    metrics.track_us as f64
                                );
                            }
                        }
                    }
                    Ok(None) => {
                        // No more packets - could be:
                        // 1. Replay mode with looping enabled (loop reset signal)
                        // 2. End of PCAP replay without looping
                        // 3. End of live capture (shouldn't happen)
                        //
                        // For replay with looping, ReplayCapture signals loop reset with Ok(None)
                        // but continues returning packets on next calls.
                        // Data persistence only happens via async periodic persistence and final persistence,
                        // not at loop boundaries. Continue to process more packets if available.

                        if debug && debug {
                            let elapsed = start_time.elapsed().as_secs_f64();
                            println!(
                                "[{:.1}s] Loop boundary: {} packets processed, {} gaps detected, continuing...",
                                elapsed, packet_count, gap_count
                            );
                        }

                        continue;
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
                if debug {
                    println!("\nShutdown signal received. Flushing data...");
                }
                break;
            }
        }
    }

    // Final persistence
    if debug && debug {
        println!("Saving final statistics...");
    }
    {
        // Final persistence (DashMap is thread-safe, no locking needed)
        persistence.persist_flows(flow_tracker)?;
        analyzer::print_analysis_report(flow_tracker, registry, packet_count, gap_count, start_time, debug, timing_stats)?;
    }

    Ok(())
}
