use crate::error::CaptureError;
use crate::types::{CaptureStats, RawPacket};

/// Abstraction over packet capture sources (file or live interface)
/// Allows the analyzer to work with different packet sources without knowing the details
pub trait PacketSource {
    /// Get the next packet from the capture source
    /// Returns Some(RawPacket) if a packet is available
    /// Returns None when no more packets are available
    fn next_packet(&mut self) -> Result<Option<RawPacket>, CaptureError>;

    /// Get statistics from the capture source
    fn stats(&self) -> CaptureStats;
}

/// Async packet source for high-performance concurrent processing
#[cfg(feature = "async")]
pub trait AsyncPacketSource: Send + Sync {
    /// Get next packet asynchronously
    async fn next_packet(&mut self) -> Result<Option<RawPacket>, CaptureError>;

    /// Get capture statistics
    fn stats(&self) -> CaptureStats;

    /// Optional: Set BPF filter (for live captures)
    fn set_filter(&mut self, _filter: &str) -> Result<(), CaptureError> {
        Err(CaptureError::UnsupportedOperation(
            "BPF filtering not supported by this source".to_string(),
        ))
    }
}
