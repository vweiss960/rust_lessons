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
