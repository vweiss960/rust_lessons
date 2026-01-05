#![cfg_attr(not(feature = "cli"), allow(dead_code))]

#[cfg(feature = "cli")]
use std::env;
#[cfg(feature = "cli")]
use std::time::SystemTime;

#[cfg(feature = "cli")]
use macsec_packet_analyzer::{
    analysis::PacketAnalyzer, capture::FileCapture, protocol::MACsecParser,
};

#[cfg(feature = "cli")]
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

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("This binary requires the 'cli' feature to be enabled.");
    eprintln!("Please build with: cargo build --features cli");
    std::process::exit(1);
}
