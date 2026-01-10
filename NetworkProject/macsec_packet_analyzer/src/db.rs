#![cfg(any(feature = "rest-api", feature = "cli"))]
//! SQLite database layer for storing and querying packet analysis results
//!
//! Supports both SQLite (development) and PostgreSQL (future) backends.
//! Current implementation uses rusqlite for SQLite with chrono for timestamps.

use crate::error::CaptureError;
use crate::types::{FlowId, FlowStats, SequenceGap};
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use std::time::SystemTime;

/// Database configuration supporting multiple backends
#[derive(Clone, Debug)]
pub enum DatabaseConfig {
    /// SQLite: Local file database (for prototyping)
    SQLite {
        path: String,
    },
    /// PostgreSQL: Network database (for production)
    /// Not yet implemented
    #[allow(dead_code)]
    PostgreSQL {
        connection_string: String,
    },
}

impl DatabaseConfig {
    /// Create SQLite database config with default path
    pub fn sqlite_default() -> Self {
        Self::SQLite {
            path: "analysis.db".to_string(),
        }
    }

    /// Create SQLite database config with custom path
    pub fn sqlite(path: impl Into<String>) -> Self {
        Self::SQLite {
            path: path.into(),
        }
    }

    /// Create PostgreSQL database config (placeholder for future)
    #[allow(dead_code)]
    pub fn postgres(connection_string: impl Into<String>) -> Self {
        Self::PostgreSQL {
            connection_string: connection_string.into(),
        }
    }
}

/// Database abstraction layer
/// Currently implements SQLite backend via rusqlite
#[cfg(any(feature = "rest-api", feature = "cli"))]
pub struct Database {
    conn: rusqlite::Connection,
}

#[cfg(any(feature = "rest-api", feature = "cli"))]
impl Database {
    /// Open or create database connection
    pub fn open(config: &DatabaseConfig) -> Result<Self, CaptureError> {
        match config {
            DatabaseConfig::SQLite { path } => {
                let conn = rusqlite::Connection::open(path)
                    .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

                // Note: WAL mode and synchronous mode will be configured in the initialize() method
                // to avoid execution issues with PRAGMA statements in open()

                Ok(Self { conn })
            }
            DatabaseConfig::PostgreSQL { .. } => {
                Err(CaptureError::DatabaseError(
                    "PostgreSQL not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Initialize database schema (creates tables if not exist)
    pub fn initialize(&mut self) -> Result<(), CaptureError> {
        // Use execute_batch for all schema creation
        // Note: PRAGMAs with result-returning statements will be set separately
        let schema_sql = "
            CREATE TABLE IF NOT EXISTS flows (
                id TEXT PRIMARY KEY,
                first_sequence INTEGER,
                last_sequence INTEGER,
                packets_received INTEGER NOT NULL DEFAULT 0,
                gaps_detected INTEGER NOT NULL DEFAULT 0,
                total_lost_packets INTEGER NOT NULL DEFAULT 0,
                min_gap INTEGER,
                max_gap INTEGER,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sequence_gaps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                flow_id TEXT NOT NULL,
                expected_sequence INTEGER NOT NULL,
                received_sequence INTEGER NOT NULL,
                gap_size INTEGER NOT NULL,
                detected_at DATETIME NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(flow_id) REFERENCES flows(id)
            );

            CREATE TABLE IF NOT EXISTS flow_statistics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                flow_id TEXT NOT NULL UNIQUE,
                total_bytes INTEGER NOT NULL DEFAULT 0,
                first_timestamp TEXT,
                last_timestamp TEXT,
                min_inter_arrival_us INTEGER,
                max_inter_arrival_us INTEGER,
                avg_inter_arrival_us INTEGER,
                protocol_distribution TEXT,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(flow_id) REFERENCES flows(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_flows_created_at ON flows(created_at);
            CREATE INDEX IF NOT EXISTS idx_gaps_flow_id ON sequence_gaps(flow_id);
            CREATE INDEX IF NOT EXISTS idx_gaps_detected_at ON sequence_gaps(detected_at);
            CREATE INDEX IF NOT EXISTS idx_stats_flow_id ON flow_statistics(flow_id);
        ";

        self.conn
            .execute_batch(schema_sql)
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store flow statistics
    pub fn insert_flow(&mut self, stats: &FlowStats) -> Result<(), CaptureError> {
        let flow_id = stats.flow_id.to_string();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO flows (
                    id, first_sequence, last_sequence, packets_received,
                    gaps_detected, total_lost_packets, min_gap, max_gap, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)",
                rusqlite::params![
                    &flow_id,
                    stats.first_sequence,
                    stats.last_sequence,
                    stats.packets_received,
                    stats.gaps_detected,
                    stats.total_lost_packets,
                    stats.min_gap,
                    stats.max_gap,
                ],
            )
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store sequence gap detection
    pub fn insert_gap(&mut self, gap: &SequenceGap) -> Result<(), CaptureError> {
        let flow_id = gap.flow_id.to_string();
        let detected_at = DateTime::<Utc>::from(gap.timestamp)
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string();

        self.conn
            .execute(
                "INSERT INTO sequence_gaps (flow_id, expected_sequence, received_sequence, gap_size, detected_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![&flow_id, gap.expected, gap.received, gap.gap_size, &detected_at],
            )
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store enhanced statistics for a flow
    pub fn insert_statistics(&mut self, stats: &FlowStats) -> Result<(), CaptureError> {
        let flow_id = stats.flow_id.to_string();

        // Format timestamps as ISO 8601
        let first_timestamp = stats.first_timestamp.map(|t| {
            DateTime::<Utc>::from(t).to_rfc3339()
        });
        let last_timestamp = stats.last_timestamp.map(|t| {
            DateTime::<Utc>::from(t).to_rfc3339()
        });

        // Convert Duration to microseconds
        let min_inter_arrival_us = stats.min_inter_arrival.map(|d| d.as_micros() as i64);
        let max_inter_arrival_us = stats.max_inter_arrival.map(|d| d.as_micros() as i64);
        let avg_inter_arrival_us = stats.avg_inter_arrival.map(|d| d.as_micros() as i64);

        // Serialize protocol distribution as JSON
        let protocol_distribution = if stats.protocol_distribution.is_empty() {
            None
        } else {
            match serde_json::to_string(&stats.protocol_distribution) {
                Ok(json_str) => Some(json_str),
                Err(_) => None,
            }
        };

        self.conn
            .execute(
                "INSERT OR REPLACE INTO flow_statistics (
                    flow_id, total_bytes, first_timestamp, last_timestamp,
                    min_inter_arrival_us, max_inter_arrival_us, avg_inter_arrival_us,
                    protocol_distribution, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)",
                rusqlite::params![
                    &flow_id,
                    stats.total_bytes as i64,
                    first_timestamp,
                    last_timestamp,
                    min_inter_arrival_us,
                    max_inter_arrival_us,
                    avg_inter_arrival_us,
                    protocol_distribution,
                ],
            )
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get enhanced statistics for a specific flow
    pub fn get_statistics(&self, flow_id: &FlowId) -> Result<Option<FlowStatisticsRecord>, CaptureError> {
        let flow_id_str = flow_id.to_string();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT flow_id, total_bytes, first_timestamp, last_timestamp,
                        min_inter_arrival_us, max_inter_arrival_us, avg_inter_arrival_us,
                        protocol_distribution
                 FROM flow_statistics WHERE flow_id = ?1",
            )
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![&flow_id_str], |row| {
                Ok(FlowStatisticsRecord {
                    flow_id: row.get(0)?,
                    total_bytes: row.get(1)?,
                    first_timestamp: row.get(2)?,
                    last_timestamp: row.get(3)?,
                    min_inter_arrival_us: row.get(4)?,
                    max_inter_arrival_us: row.get(5)?,
                    avg_inter_arrival_us: row.get(6)?,
                    protocol_distribution: row.get(7)?,
                })
            })
            .optional()
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    /// Get flow statistics by ID
    pub fn get_flow(&self, flow_id: &FlowId) -> Result<Option<FlowStats>, CaptureError> {
        let flow_id_str = flow_id.to_string();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT f.id, f.first_sequence, f.last_sequence, f.packets_received,
                        f.gaps_detected, f.total_lost_packets, f.min_gap, f.max_gap,
                        s.total_bytes, s.first_timestamp, s.last_timestamp,
                        s.min_inter_arrival_us, s.max_inter_arrival_us, s.avg_inter_arrival_us,
                        s.protocol_distribution
                 FROM flows f
                 LEFT JOIN flow_statistics s ON f.id = s.flow_id
                 WHERE f.id = ?1",
            )
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![&flow_id_str], |row| {
                let total_bytes = row.get::<_, Option<i64>>(8)?.unwrap_or(0) as u64;
                let first_timestamp = row.get::<_, Option<String>>(9)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| SystemTime::from(dt.with_timezone(&Utc)));
                let last_timestamp = row.get::<_, Option<String>>(10)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| SystemTime::from(dt.with_timezone(&Utc)));
                let min_inter_arrival = row.get::<_, Option<i64>>(11)?
                    .map(|v| std::time::Duration::from_micros(v as u64));
                let max_inter_arrival = row.get::<_, Option<i64>>(12)?
                    .map(|v| std::time::Duration::from_micros(v as u64));
                let avg_inter_arrival = row.get::<_, Option<i64>>(13)?
                    .map(|v| std::time::Duration::from_micros(v as u64));
                let protocol_distribution_str = row.get::<_, Option<String>>(14)?;
                let protocol_distribution = protocol_distribution_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default();

                Ok(FlowStats {
                    flow_id: FlowId::new(row.get::<_, String>(0)?),
                    first_sequence: row.get(1)?,
                    last_sequence: row.get(2)?,
                    packets_received: row.get(3)?,
                    gaps_detected: row.get(4)?,
                    total_lost_packets: row.get(5)?,
                    min_gap: row.get(6)?,
                    max_gap: row.get(7)?,
                    total_bytes,
                    first_timestamp,
                    last_timestamp,
                    min_inter_arrival,
                    max_inter_arrival,
                    avg_inter_arrival,
                    protocol_distribution,
                })
            })
            .optional()
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    /// Get all flow statistics with optional filtering
    pub fn get_flows(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<FlowStats>, CaptureError> {
        let limit = limit.unwrap_or(100).min(1000); // Max 1000 results
        let offset = offset.unwrap_or(0).max(0);

        let mut stmt = self
            .conn
            .prepare(
                "SELECT f.id, f.first_sequence, f.last_sequence, f.packets_received,
                        f.gaps_detected, f.total_lost_packets, f.min_gap, f.max_gap,
                        s.total_bytes, s.first_timestamp, s.last_timestamp,
                        s.min_inter_arrival_us, s.max_inter_arrival_us, s.avg_inter_arrival_us,
                        s.protocol_distribution
                 FROM flows f
                 LEFT JOIN flow_statistics s ON f.id = s.flow_id
                 ORDER BY f.updated_at DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        let flows = stmt
            .query_map(rusqlite::params![limit, offset], |row| {
                let total_bytes = row.get::<_, Option<i64>>(8)?.unwrap_or(0) as u64;
                let first_timestamp = row.get::<_, Option<String>>(9)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| SystemTime::from(dt.with_timezone(&Utc)));
                let last_timestamp = row.get::<_, Option<String>>(10)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| SystemTime::from(dt.with_timezone(&Utc)));
                let min_inter_arrival = row.get::<_, Option<i64>>(11)?
                    .map(|v| std::time::Duration::from_micros(v as u64));
                let max_inter_arrival = row.get::<_, Option<i64>>(12)?
                    .map(|v| std::time::Duration::from_micros(v as u64));
                let avg_inter_arrival = row.get::<_, Option<i64>>(13)?
                    .map(|v| std::time::Duration::from_micros(v as u64));
                let protocol_distribution_str = row.get::<_, Option<String>>(14)?;
                let protocol_distribution = protocol_distribution_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default();

                Ok(FlowStats {
                    flow_id: FlowId::new(row.get::<_, String>(0)?),
                    first_sequence: row.get(1)?,
                    last_sequence: row.get(2)?,
                    packets_received: row.get(3)?,
                    gaps_detected: row.get(4)?,
                    total_lost_packets: row.get(5)?,
                    min_gap: row.get(6)?,
                    max_gap: row.get(7)?,
                    total_bytes,
                    first_timestamp,
                    last_timestamp,
                    min_inter_arrival,
                    max_inter_arrival,
                    avg_inter_arrival,
                    protocol_distribution,
                })
            })
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        Ok(flows)
    }

    /// Get gaps for a specific flow
    pub fn get_flow_gaps(
        &self,
        flow_id: &FlowId,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<SequenceGap>, CaptureError> {
        let flow_id_str = flow_id.to_string();
        let limit = limit.unwrap_or(100).min(1000);
        let offset = offset.unwrap_or(0).max(0);

        let mut stmt = self
            .conn
            .prepare(
                "SELECT flow_id, expected_sequence, received_sequence, gap_size, detected_at
                 FROM sequence_gaps
                 WHERE flow_id = ?1
                 ORDER BY detected_at DESC
                 LIMIT ?2 OFFSET ?3",
            )
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        let gaps = stmt
            .query_map(rusqlite::params![&flow_id_str, limit, offset], |row| {
                let detected_at_str: String = row.get(4)?;
                // Parse ISO 8601 format back to SystemTime
                let dt = chrono::DateTime::parse_from_rfc3339(&detected_at_str)
                    .ok()
                    .map(|dt| SystemTime::from(dt.with_timezone(&Utc)))
                    .unwrap_or(SystemTime::now());

                Ok(SequenceGap {
                    flow_id: FlowId::new(row.get::<_, String>(0)?),
                    expected: row.get(1)?,
                    received: row.get(2)?,
                    gap_size: row.get(3)?,
                    timestamp: dt,
                })
            })
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        Ok(gaps)
    }

    /// Get summary statistics across all flows including enhanced metrics
    pub fn get_summary_stats(&self) -> Result<SummaryStats, CaptureError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT COUNT(DISTINCT f.id) as flow_count,
                        COALESCE(SUM(f.packets_received), 0) as total_packets,
                        COALESCE(SUM(f.gaps_detected), 0) as total_gaps,
                        COALESCE(SUM(f.total_lost_packets), 0) as total_lost,
                        COALESCE(MAX(f.max_gap), 0) as max_gap_size
                 FROM flows f
                 LEFT JOIN flow_statistics s ON f.id = s.flow_id",
            )
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        stmt.query_row([], |row| {
            Ok(SummaryStats {
                total_flows: row.get(0)?,
                total_packets_received: row.get(1)?,
                total_gaps_detected: row.get(2)?,
                total_lost_packets: row.get(3)?,
                max_gap_size: row.get(4)?,
            })
        })
        .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))
    }

    /// Clear all data (useful for testing)
    #[allow(dead_code)]
    pub fn clear_all(&mut self) -> Result<(), CaptureError> {
        self.conn
            .execute("DELETE FROM flow_statistics", [])
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;
        self.conn
            .execute("DELETE FROM sequence_gaps", [])
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;
        self.conn
            .execute("DELETE FROM flows", [])
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

/// Summary statistics across all flows
#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "rest-api", serde(crate = "serde"))]
pub struct SummaryStats {
    pub total_flows: i64,
    pub total_packets_received: i64,
    pub total_gaps_detected: i64,
    pub total_lost_packets: i64,
    pub max_gap_size: i64,
}

/// Enhanced statistics for a single flow
/// Stored in normalized flow_statistics table
#[derive(Debug, Clone)]
pub struct FlowStatisticsRecord {
    pub flow_id: String,
    pub total_bytes: i64,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub min_inter_arrival_us: Option<i64>,
    pub max_inter_arrival_us: Option<i64>,
    pub avg_inter_arrival_us: Option<i64>,
    pub protocol_distribution: Option<String>, // JSON string
}
