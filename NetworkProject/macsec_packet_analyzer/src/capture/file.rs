use pcap::Capture;
use std::time::{Duration, UNIX_EPOCH};

use crate::error::CaptureError;
use crate::types::{CaptureStats, RawPacket};

use super::source::PacketSource;

/// File-based packet capture from a pcap file
pub struct FileCapture {
    capture: Capture<pcap::Offline>,
    packets_read: u64,
}

impl FileCapture {
    /// Open a pcap file for reading
    pub fn open(path: &str) -> Result<Self, CaptureError> {
        let capture = Capture::from_file(path)
            .map_err(|e| CaptureError::OpenFailed(format!("Failed to open {}: {}", path, e)))?;

        Ok(Self {
            capture,
            packets_read: 0,
        })
    }
}

impl PacketSource for FileCapture {
    fn next_packet(&mut self) -> Result<Option<RawPacket>, CaptureError> {
        match self.capture.next() {
            Ok(packet) => {
                self.packets_read += 1;

                // Convert pcap timestamp to SystemTime
                // pcap header contains seconds and microseconds since epoch
                let timestamp = UNIX_EPOCH
                    + Duration::from_secs(packet.header.ts.tv_sec as u64)
                    + Duration::from_micros(packet.header.ts.tv_usec as u64);

                Ok(Some(RawPacket {
                    data: packet.data.to_vec(),
                    timestamp,
                    length: packet.header.len as usize,
                }))
            }
            Err(pcap::Error::NoMorePackets) => Ok(None),
            Err(e) => Err(CaptureError::ReadFailed(format!(
                "Error reading pcap: {}",
                e
            ))),
        }
    }

    fn stats(&self) -> CaptureStats {
        CaptureStats {
            packets_received: self.packets_read,
            packets_dropped: 0, // File captures don't drop packets
        }
    }
}
