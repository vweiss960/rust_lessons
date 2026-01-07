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
}

/// Query parameters for pagination
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Shared database connection wrapped in Arc<Mutex<>>
pub type SharedDb = Arc<Mutex<Database>>;

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
    println!("  GET /api/v1/stats/summary - Summary statistics");
    println!("  GET /api/v1/flows - List all flows (with pagination)");
    println!("  GET /api/v1/flows/:flow_id - Get flow details");
    println!("  GET /api/v1/flows/:flow_id/gaps - Get gaps for a flow");

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

/// Get summary statistics across all flows
async fn get_summary_stats(
    State(db): State<SharedDb>,
) -> Result<Json<SummaryResponse>, ApiError> {
    let db = db.lock().map_err(|_| ApiError::DatabaseLocked)?;
    let stats = db.get_summary_stats()?;

    Ok(Json(SummaryResponse {
        total_flows: stats.total_flows,
        total_packets_received: stats.total_packets_received,
        total_gaps_detected: stats.total_gaps_detected,
        total_lost_packets: stats.total_lost_packets,
        max_gap_size: stats.max_gap_size,
    }))
}

/// List all flows with pagination
async fn list_flows(
    State(db): State<SharedDb>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Value>, ApiError> {
    let db = db.lock().map_err(|_| ApiError::DatabaseLocked)?;
    let flows = db.get_flows(params.limit, params.offset)?;

    let flow_responses: Vec<FlowResponse> = flows
        .into_iter()
        .map(|f| FlowResponse {
            flow_id: f.flow_id.to_string(),
            packets_received: f.packets_received,
            gaps_detected: f.gaps_detected,
            total_lost_packets: f.total_lost_packets,
            first_sequence: f.first_sequence,
            last_sequence: f.last_sequence,
            min_gap: f.min_gap,
            max_gap: f.max_gap,
        })
        .collect();

    Ok(Json(json!({
        "count": flow_responses.len(),
        "flows": flow_responses
    })))
}

/// Get detailed statistics for a specific flow
async fn get_flow_detail(
    State(db): State<SharedDb>,
    Path(flow_id): Path<String>,
) -> Result<Json<FlowResponse>, ApiError> {
    let db = db.lock().map_err(|_| ApiError::DatabaseLocked)?;
    let flow_id = FlowId::new(flow_id);
    let stats = db
        .get_flow(&flow_id)?
        .ok_or(ApiError::FlowNotFound)?;

    Ok(Json(FlowResponse {
        flow_id: stats.flow_id.to_string(),
        packets_received: stats.packets_received,
        gaps_detected: stats.gaps_detected,
        total_lost_packets: stats.total_lost_packets,
        first_sequence: stats.first_sequence,
        last_sequence: stats.last_sequence,
        min_gap: stats.min_gap,
        max_gap: stats.max_gap,
    }))
}

/// Get all sequence gaps for a specific flow
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
