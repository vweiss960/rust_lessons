#![cfg(feature = "rest-api")]
//! REST API server for querying packet analysis results
//!
//! Provides HTTP endpoints to retrieve flow statistics, gaps, and summary data
//! stored in the SQLite database.

use crate::db::{Database, DatabaseConfig};
use crate::types::FlowId;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

/// API request/response models
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowResponse {
    pub flow_id: String,
    pub packets_received: u64,
    pub gaps_detected: u64,
    pub total_lost_packets: u64,
    pub first_sequence: Option<u32>,
    pub last_sequence: Option<u32>,
    pub min_gap: Option<u32>,
    pub max_gap: Option<u32>,

    // Enhanced statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_mbps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_inter_arrival_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_inter_arrival_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_inter_arrival_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_distribution: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GapResponse {
    pub flow_id: String,
    pub expected_sequence: u32,
    pub received_sequence: u32,
    pub gap_size: u32,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct SummaryResponse {
    pub total_flows: i64,
    pub total_packets_received: i64,
    pub total_gaps_detected: i64,
    pub total_lost_packets: i64,
    pub max_gap_size: i64,

    // Enhanced statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_bandwidth_mbps: Option<f64>,
}

/// Query parameters for pagination
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Query parameters for advanced flow filtering
#[derive(Debug, Deserialize)]
pub struct FlowQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub min_bytes: Option<u64>,
    pub max_bytes: Option<u64>,
    pub min_bandwidth_mbps: Option<f64>,
    pub max_bandwidth_mbps: Option<f64>,
}

/// Shared database connection wrapped in Arc<Mutex<>>
pub type SharedDb = Arc<Mutex<Database>>;

/// Helper function to convert FlowStats to FlowResponse with calculated metrics
fn flow_stats_to_response(stats: &crate::types::FlowStats) -> FlowResponse {
    use std::time::SystemTime;

    // Calculate duration from timestamps
    let duration_seconds = stats
        .first_timestamp
        .zip(stats.last_timestamp)
        .and_then(|(first, last)| {
            last.duration_since(first)
                .ok()
                .map(|d| d.as_secs_f64())
        });

    // Calculate bandwidth in Mbps: (bytes * 8 bits/byte) / (seconds) / (1,000,000 bits/Mbps)
    let bandwidth_mbps = stats
        .total_bytes
        .gt(&0)
        .then_some(())
        .zip(duration_seconds)
        .map(|((), dur_secs)| {
            if dur_secs > 0.0 {
                (stats.total_bytes as f64 * 8.0) / dur_secs / 1_000_000.0
            } else {
                0.0
            }
        });

    // Convert Duration to milliseconds
    let min_inter_arrival_ms = stats.min_inter_arrival.map(|d| d.as_secs_f64() * 1000.0);
    let max_inter_arrival_ms = stats.max_inter_arrival.map(|d| d.as_secs_f64() * 1000.0);
    let avg_inter_arrival_ms = stats.avg_inter_arrival.map(|d| d.as_secs_f64() * 1000.0);

    // Format timestamps as ISO 8601 strings
    let first_timestamp = stats
        .first_timestamp
        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());
    let last_timestamp = stats
        .last_timestamp
        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

    // Convert protocol distribution to JSON value if present
    let protocol_distribution = if stats.protocol_distribution.is_empty() {
        None
    } else {
        serde_json::to_value(&stats.protocol_distribution).ok()
    };

    FlowResponse {
        flow_id: stats.flow_id.to_string(),
        packets_received: stats.packets_received,
        gaps_detected: stats.gaps_detected,
        total_lost_packets: stats.total_lost_packets,
        first_sequence: stats.first_sequence,
        last_sequence: stats.last_sequence,
        min_gap: stats.min_gap,
        max_gap: stats.max_gap,
        total_bytes: Some(stats.total_bytes),
        first_timestamp,
        last_timestamp,
        duration_seconds,
        bandwidth_mbps,
        min_inter_arrival_ms,
        max_inter_arrival_ms,
        avg_inter_arrival_ms,
        protocol_distribution,
    }
}

/// Create and start the REST API server
///
/// # Arguments
/// * `db_config` - Database configuration (SQLite path or PostgreSQL connection string)
/// * `listen_addr` - TCP address to listen on (e.g., "127.0.0.1:8080")
///
/// # Example
/// ```no_run
/// # use macsec_packet_analyzer::db::DatabaseConfig;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let db_config = DatabaseConfig::sqlite_default();
/// macsec_packet_analyzer::api::start_server(db_config, "127.0.0.1:8080").await?;
/// # Ok(())
/// # }
/// ```
pub async fn start_server(
    db_config: DatabaseConfig,
    listen_addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let mut db = Database::open(&db_config)?;
    db.initialize()?;

    let db: SharedDb = Arc::new(Mutex::new(db));

    // Build router with all endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/stats/summary", get(get_summary_stats))
        .route("/api/v1/flows", get(list_flows))
        .route("/api/v1/flows/:flow_id", get(get_flow_detail))
        .route("/api/v1/flows/:flow_id/gaps", get(get_flow_gaps))
        .with_state(db);

    // Start server
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    println!("REST API server listening on http://{}", listen_addr);
    println!("Available endpoints:");
    println!("  GET /health - Health check");
    println!("  GET /api/v1/stats/summary - Summary statistics with bandwidth metrics");
    println!("  GET /api/v1/flows - List all flows with enhanced statistics");
    println!("    Query params: limit, offset, min_bytes, max_bytes, min_bandwidth_mbps, max_bandwidth_mbps");
    println!("  GET /api/v1/flows/:flow_id - Get flow details with all metrics");
    println!("  GET /api/v1/flows/:flow_id/gaps - Get gaps for a flow");
    println!("    Note: Gap detection is only available for MACsec and IPsec flows");
    println!("          Generic L3 (TCP/UDP) flows will have 0 gaps detected");

    axum::serve(listener, app).await?;
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Get summary statistics across all flows including bandwidth metrics
async fn get_summary_stats(
    State(db): State<SharedDb>,
) -> Result<Json<SummaryResponse>, ApiError> {
    let db = db.lock().map_err(|_| ApiError::DatabaseLocked)?;
    let stats = db.get_summary_stats()?;

    // Calculate aggregate statistics from all flows for bandwidth
    let all_flows = db.get_flows(None, None)?;
    let total_bytes: u64 = all_flows.iter().map(|f| f.total_bytes).sum();

    // Calculate average bandwidth across all flows
    // Use the overall time span from earliest first_timestamp to latest last_timestamp
    let avg_bandwidth_mbps = if all_flows.is_empty() {
        None
    } else {
        // Find the earliest first_timestamp and latest last_timestamp across all flows
        let overall_first = all_flows
            .iter()
            .filter_map(|f| f.first_timestamp)
            .min();
        let overall_last = all_flows
            .iter()
            .filter_map(|f| f.last_timestamp)
            .max();

        let total_duration = overall_first
            .zip(overall_last)
            .and_then(|(first, last)| {
                last.duration_since(first)
                    .ok()
                    .map(|d| d.as_secs_f64())
            });

        if let Some(duration) = total_duration {
            if duration > 0.0 && total_bytes > 0 {
                Some((total_bytes as f64 * 8.0) / duration / 1_000_000.0)
            } else {
                None
            }
        } else {
            None
        }
    };

    Ok(Json(SummaryResponse {
        total_flows: stats.total_flows,
        total_packets_received: stats.total_packets_received,
        total_gaps_detected: stats.total_gaps_detected,
        total_lost_packets: stats.total_lost_packets,
        max_gap_size: stats.max_gap_size,
        total_bytes: if total_bytes > 0 { Some(total_bytes) } else { None },
        avg_bandwidth_mbps,
    }))
}

/// List all flows with pagination and optional filtering
async fn list_flows(
    State(db): State<SharedDb>,
    Query(params): Query<FlowQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let db = db.lock().map_err(|_| ApiError::DatabaseLocked)?;
    let flows = db.get_flows(params.limit, params.offset)?;

    let flow_responses: Vec<FlowResponse> = flows
        .into_iter()
        .map(|f| flow_stats_to_response(&f))
        .filter(|f| {
            // Apply byte filtering
            if let Some(min_bytes) = params.min_bytes {
                if f.total_bytes.unwrap_or(0) < min_bytes {
                    return false;
                }
            }
            if let Some(max_bytes) = params.max_bytes {
                if f.total_bytes.unwrap_or(0) > max_bytes {
                    return false;
                }
            }

            // Apply bandwidth filtering
            if let Some(min_bw) = params.min_bandwidth_mbps {
                if f.bandwidth_mbps.unwrap_or(0.0) < min_bw {
                    return false;
                }
            }
            if let Some(max_bw) = params.max_bandwidth_mbps {
                if f.bandwidth_mbps.unwrap_or(0.0) > max_bw {
                    return false;
                }
            }

            true
        })
        .collect();

    Ok(Json(json!({
        "count": flow_responses.len(),
        "flows": flow_responses
    })))
}

/// Get detailed statistics for a specific flow with enhanced metrics
async fn get_flow_detail(
    State(db): State<SharedDb>,
    Path(flow_id): Path<String>,
) -> Result<Json<FlowResponse>, ApiError> {
    let db = db.lock().map_err(|_| ApiError::DatabaseLocked)?;
    let flow_id = FlowId::new(flow_id);
    let stats = db
        .get_flow(&flow_id)?
        .ok_or(ApiError::FlowNotFound)?;

    Ok(Json(flow_stats_to_response(&stats)))
}

/// Get all sequence gaps for a specific flow
///
/// **Note**: Gap detection is only available for MACsec and IPsec flows.
/// Generic L3 (TCP/UDP) flows will always return an empty gaps array because:
/// - TCP sequence numbers track cumulative bytes, not packets
/// - TCP permits retransmissions and out-of-order delivery
/// - This causes unreliable gap detection (67%+ false positive rate)
///
/// For TCP/UDP flows, use packet counts and bandwidth metrics instead.
async fn get_flow_gaps(
    State(db): State<SharedDb>,
    Path(flow_id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Value>, ApiError> {
    let db = db.lock().map_err(|_| ApiError::DatabaseLocked)?;
    let flow_id = FlowId::new(flow_id);
    let gaps = db.get_flow_gaps(&flow_id, params.limit, params.offset)?;

    let gap_responses: Vec<GapResponse> = gaps
        .into_iter()
        .map(|g| GapResponse {
            flow_id: g.flow_id.to_string(),
            expected_sequence: g.expected,
            received_sequence: g.received,
            gap_size: g.gap_size,
            timestamp: chrono::DateTime::<chrono::Utc>::from(g.timestamp).to_rfc3339(),
        })
        .collect();

    Ok(Json(json!({
        "count": gap_responses.len(),
        "gaps": gap_responses
    })))
}

/// API error types
#[derive(Debug)]
pub enum ApiError {
    DatabaseError(String),
    DatabaseLocked,
    FlowNotFound,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = match self {
            ApiError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "error": "database_error",
                    "message": msg
                }),
            ),
            ApiError::DatabaseLocked => (
                StatusCode::SERVICE_UNAVAILABLE,
                json!({
                    "error": "database_locked",
                    "message": "Database is currently locked"
                }),
            ),
            ApiError::FlowNotFound => (
                StatusCode::NOT_FOUND,
                json!({
                    "error": "flow_not_found",
                    "message": "The requested flow was not found"
                }),
            ),
        };

        (status, Json(body)).into_response()
    }
}

impl From<crate::error::CaptureError> for ApiError {
    fn from(err: crate::error::CaptureError) -> Self {
        ApiError::DatabaseError(err.to_string())
    }
}
