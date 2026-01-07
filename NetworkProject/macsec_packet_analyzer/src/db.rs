#![cfg(feature = "rest-api")]
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
#[cfg(feature = "rest-api")]
pub struct Database {
    conn: rusqlite::Connection,
}

#[cfg(feature = "rest-api")]
impl Database {
    /// Open or create database connection
    pub fn open(config: &DatabaseConfig) -> Result<Self, CaptureError> {
        match config {
            DatabaseConfig::SQLite { path } => {
                let conn = rusqlite::Connection::open(path)
                    .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

                // Enable foreign keys
                conn.execute("PRAGMA foreign_keys = ON", [])
                    .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

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
        // Create flows table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS flows (
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
                )",
                [],
            )
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

        // Create sequence_gaps table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS sequence_gaps (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    flow_id TEXT NOT NULL,
                    expected_sequence INTEGER NOT NULL,
                    received_sequence INTEGER NOT NULL,
                    gap_size INTEGER NOT NULL,
                    detected_at DATETIME NOT NULL,
                    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY(flow_id) REFERENCES flows(id)
                )",
                [],
            )
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

        // Create indices for common queries
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_flows_created_at ON flows(created_at)",
                [],
            )
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_gaps_flow_id ON sequence_gaps(flow_id)",
                [],
            )
            .map_err(|e| CaptureError::DatabaseError(e.to_string()))?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_gaps_detected_at ON sequence_gaps(detected_at)",
                [],
            )
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

    /// Get flow statistics by ID
    pub fn get_flow(&self, flow_id: &FlowId) -> Result<Option<FlowStats>, CaptureError> {
        let flow_id_str = flow_id.to_string();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, first_sequence, last_sequence, packets_received,
                        gaps_detected, total_lost_packets, min_gap, max_gap
                 FROM flows WHERE id = ?1",
            )
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![&flow_id_str], |row| {
                Ok(FlowStats {
                    flow_id: FlowId::new(row.get::<_, String>(0)?),
                    first_sequence: row.get(1)?,
                    last_sequence: row.get(2)?,
                    packets_received: row.get(3)?,
                    gaps_detected: row.get(4)?,
                    total_lost_packets: row.get(5)?,
                    min_gap: row.get(6)?,
                    max_gap: row.get(7)?,
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
                "SELECT id, first_sequence, last_sequence, packets_received,
                        gaps_detected, total_lost_packets, min_gap, max_gap
                 FROM flows
                 ORDER BY updated_at DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e: rusqlite::Error| CaptureError::DatabaseError(e.to_string()))?;

        let flows = stmt
            .query_map(rusqlite::params![limit, offset], |row| {
                Ok(FlowStats {
                    flow_id: FlowId::new(row.get::<_, String>(0)?),
                    first_sequence: row.get(1)?,
                    last_sequence: row.get(2)?,
                    packets_received: row.get(3)?,
                    gaps_detected: row.get(4)?,
                    total_lost_packets: row.get(5)?,
                    min_gap: row.get(6)?,
                    max_gap: row.get(7)?,
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

    /// Get summary statistics across all flows
    pub fn get_summary_stats(&self) -> Result<SummaryStats, CaptureError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT COUNT(*) as flow_count,
                        COALESCE(SUM(packets_received), 0) as total_packets,
                        COALESCE(SUM(gaps_detected), 0) as total_gaps,
                        COALESCE(SUM(total_lost_packets), 0) as total_lost,
                        COALESCE(MAX(max_gap), 0) as max_gap_size
                 FROM flows",
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
