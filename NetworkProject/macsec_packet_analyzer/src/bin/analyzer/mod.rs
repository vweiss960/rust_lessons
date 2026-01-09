//! Packet analyzer module
//! Contains core analyzer logic and metrics processing

pub mod metrics;
pub mod debug_utils;

pub use metrics::{process_single_packet, TimingStats, ExecutionTimer};
pub use debug_utils::print_analysis_report;
