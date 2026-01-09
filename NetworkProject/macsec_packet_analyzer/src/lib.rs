// Analysis module available for both cli and async features
#[cfg(any(feature = "cli", feature = "async"))]
pub mod analysis;

// Capture module available for both cli and async features
#[cfg(any(feature = "cli", feature = "async"))]
pub mod capture;

pub mod error;
pub mod protocol;
pub mod types;

// Database module available for CLI file analysis and REST API
#[cfg(any(feature = "rest-api", feature = "cli"))]
pub mod db;

// REST API module for serving metrics
#[cfg(feature = "rest-api")]
pub mod api;

// Database persistence integration available for CLI file analysis and REST API
#[cfg(any(feature = "rest-api", feature = "cli"))]
pub mod persist;

// Configuration management for REST API
#[cfg(feature = "rest-api")]
pub mod config;

// Re-export commonly used public API
#[cfg(feature = "cli")]
pub use analysis::PacketAnalyzer;

#[cfg(feature = "async")]
pub use analysis::flow::FlowTracker;

#[cfg(feature = "cli")]
pub use capture::FileCapture;

#[cfg(any(feature = "cli", feature = "async"))]
pub use capture::PacketSource;

#[cfg(feature = "async")]
pub use capture::AsyncPacketSource;

#[cfg(all(feature = "async", feature = "pcap"))]
pub use capture::PcapLiveCapture;

#[cfg(all(target_os = "linux", feature = "async"))]
pub use capture::{AfPacketCapture, XdpCapture};

#[cfg(all(target_os = "linux", feature = "napatech"))]
pub use capture::{NapatechCapture, NapatechConfig, NapatechCaptureMode, NapatechStats};

#[cfg(all(feature = "async", feature = "pcap"))]
pub use capture::{ReplayCapture, ReplayMode};

pub use error::{AnalysisError, CaptureError, ParseError};
pub use protocol::{MACsecParser, SequenceParser, ProtocolRegistry, RegistryStats};
pub use types::{AnalyzedPacket, AnalysisReport, FlowId, FlowStats, SequenceGap};
