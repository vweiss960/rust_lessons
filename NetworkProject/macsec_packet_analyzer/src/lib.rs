#[cfg(feature = "cli")]
pub mod analysis;
#[cfg(feature = "cli")]
pub mod capture;
pub mod error;
pub mod protocol;
pub mod types;

// Re-export commonly used public API
#[cfg(feature = "cli")]
pub use analysis::PacketAnalyzer;
#[cfg(feature = "cli")]
pub use capture::{FileCapture, PacketSource};
pub use error::{AnalysisError, CaptureError, ParseError};
pub use protocol::{MACsecParser, SequenceParser};
pub use types::{AnalyzedPacket, AnalysisReport, FlowId, FlowStats, SequenceGap};
