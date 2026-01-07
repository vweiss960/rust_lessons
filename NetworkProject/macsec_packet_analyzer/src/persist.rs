#![cfg(any(feature = "rest-api", feature = "cli"))]
//! Database persistence layer for packet analysis results
//!
//! Provides utilities to persist flow statistics and gap detections
//! from the FlowTracker into the SQLite database.

use crate::analysis::flow::FlowTracker;
use crate::db::Database;
use crate::error::CaptureError;
use std::sync::{Arc, Mutex};

/// Persistence manager for syncing analysis results to database
pub struct PersistenceManager {
    db: Arc<Mutex<Database>>,
}

impl PersistenceManager {
    /// Create a new persistence manager with an initialized database
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    /// Persist all current flow statistics and gaps to database
    /// Call this periodically during analysis or at the end
    pub fn persist_flows(&self, tracker: &FlowTracker) -> Result<(), CaptureError> {
        let mut db = self.db.lock().map_err(|_| {
            CaptureError::DatabaseError("Failed to lock database".to_string())
        })?;

        // Get all flow stats and persist them
        let stats = tracker.get_stats();
        for flow_stat in stats {
            db.insert_flow(&flow_stat)?;
            // Also persist enhanced statistics
            db.insert_statistics(&flow_stat)?;
        }

        // Get all gaps and persist them
        let gaps = tracker.get_gaps();
        for gap in gaps {
            db.insert_gap(&gap)?;
        }

        Ok(())
    }

    /// Persist statistics for a single flow
    pub fn persist_flow(&self, tracker: &FlowTracker, flow_id: &crate::types::FlowId) -> Result<(), CaptureError> {
        let stats = tracker.get_stats();
        if let Some(flow_stat) = stats.iter().find(|s| &s.flow_id == flow_id) {
            let mut db = self.db.lock().map_err(|_| {
                CaptureError::DatabaseError("Failed to lock database".to_string())
            })?;
            db.insert_flow(flow_stat)?;
            db.insert_statistics(flow_stat)?;
        }
        Ok(())
    }

    /// Flush all pending data to database (same as persist_flows for SQLite)
    pub fn flush(&self, tracker: &FlowTracker) -> Result<(), CaptureError> {
        self.persist_flows(tracker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseConfig;

    #[test]
    fn test_create_persistence_manager() -> Result<(), CaptureError> {
        let db = Database::open(&DatabaseConfig::sqlite(":memory:"))?;
        let db = Arc::new(Mutex::new(db));
        let _manager = PersistenceManager::new(db);
        Ok(())
    }
}
