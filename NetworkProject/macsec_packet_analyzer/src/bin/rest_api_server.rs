//! REST API server for packet analysis results
//!
//! This binary starts a REST API server that serves packet analysis statistics
//! from a SQLite database configured via config.json.
//!
//! Configuration:
//!   Create a config.json file in the current directory with:
//!   {
//!     "database": {"path": "your_database.db"},
//!     "server": {"host": "127.0.0.1", "port": 3000}
//!   }
//!
//! Usage:
//!   cargo build --bin rest_api_server --release
//!   ./target/release/rest_api_server
//!   ./target/release/rest_api_server --config config.json
//!   ./target/release/rest_api_server --db mydata.db
//!   ./target/release/rest_api_server --port 8080
//!
//! Examples:
//!   cargo run --bin rest_api_server
//!   cargo run --bin rest_api_server -- --db ./capture/results.db --port 8080
//!   cargo run --bin rest_api_server -- --config ./etc/config.json
//!
//! Then access the API at:
//!   http://localhost:3000/health
//!   http://localhost:3000/api/v1/stats/summary
//!   http://localhost:3000/api/v1/flows
//!   etc.

use macsec_packet_analyzer::db::DatabaseConfig;
use macsec_packet_analyzer::api;
use macsec_packet_analyzer::config::Config;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Load base configuration from file or defaults
    let mut config = Config::from_file_or_default("config.json");

    // Parse command line overrides
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                if i + 1 < args.len() {
                    config = Config::from_file_or_default(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: --config requires a path argument");
                    std::process::exit(1);
                }
            }
            "--db" | "--database" => {
                if i + 1 < args.len() {
                    config = config.with_db_path(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: --db requires a path argument");
                    std::process::exit(1);
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u16>() {
                        Ok(port) => config = config.with_port(port),
                        Err(_) => {
                            eprintln!("Error: Invalid port number: {}", args[i + 1]);
                            std::process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --port requires a number argument");
                    std::process::exit(1);
                }
            }
            "--host" => {
                if i + 1 < args.len() {
                    config = config.with_host(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: --host requires a value argument");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    let listen_addr = config.listen_addr();
    let db_path = config.database.path.clone();

    println!("Starting REST API server...");
    println!("Configuration:");
    println!("  Database: {} (SQLite)", db_path);
    println!("  Listen address: http://{}", listen_addr);
    println!();
    println!("Endpoints:");
    println!("  GET /health                       - Health check");
    println!("  GET /api/v1/stats/summary         - Summary statistics across all flows");
    println!("  GET /api/v1/flows                 - List all flows (with pagination)");
    println!("    ?limit=10&offset=0");
    println!("  GET /api/v1/flows/<flow_id>       - Get details for a specific flow");
    println!("  GET /api/v1/flows/<flow_id>/gaps  - Get all gaps for a specific flow");
    println!("    ?limit=10&offset=0");
    println!();

    // Use configured database path
    let db_config = DatabaseConfig::sqlite(db_path);

    // Start the REST API server
    api::start_server(db_config, &listen_addr).await?;

    Ok(())
}

/// Print help message
fn print_help() {
    eprintln!("REST API Server for Packet Analysis");
    eprintln!();
    eprintln!("Usage: rest_api_server [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --config <PATH>     Load configuration from JSON file (default: config.json)");
    eprintln!("  --db <PATH>         Override database path");
    eprintln!("  --port <NUM>        Override server port (default: 3000)");
    eprintln!("  --host <HOST>       Override server host (default: 127.0.0.1)");
    eprintln!("  --help, -h          Show this help message");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  rest_api_server");
    eprintln!("  rest_api_server --db ./results.db");
    eprintln!("  rest_api_server --config prod.json --port 8080");
    eprintln!("  rest_api_server --db /data/analysis.db --host 0.0.0.0 --port 8080");
}
