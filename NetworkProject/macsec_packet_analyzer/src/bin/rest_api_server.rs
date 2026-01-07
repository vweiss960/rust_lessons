//! REST API server for packet analysis results
//!
//! This binary starts a REST API server that serves packet analysis statistics
//! from a SQLite database. It demonstrates database integration with the analyzer.
//!
//! Usage:
//!   cargo run --features "rest-api" --bin rest_api_server -- [port]
//!
//! Examples:
//!   cargo run --features "rest-api" --bin rest_api_server
//!   cargo run --features "rest-api" --bin rest_api_server -- 8080
//!
//! Then access the API at:
//!   http://localhost:3000/health
//!   http://localhost:3000/api/v1/stats/summary
//!   http://localhost:3000/api/v1/flows
//!   etc.

use macsec_packet_analyzer::db::DatabaseConfig;
use macsec_packet_analyzer::api;
use std::env;

#[cfg(feature = "rest-api")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get port from command line or use default
    let port = env::args()
        .nth(1)
        .unwrap_or_else(|| "3000".to_string());

    let listen_addr = format!("127.0.0.1:{}", port);

    println!("Starting REST API server...");
    println!("Database: analysis.db (SQLite)");
    println!("Listen address: http://{}", listen_addr);
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

    // Use default SQLite database
    let db_config = DatabaseConfig::sqlite_default();

    // Start the REST API server
    api::start_server(db_config, &listen_addr).await?;

    Ok(())
}

#[cfg(not(feature = "rest-api"))]
fn main() {
    eprintln!("This binary requires the 'rest-api' feature to be enabled.");
    eprintln!("Please build with: cargo build --features rest-api");
    std::process::exit(1);
}
