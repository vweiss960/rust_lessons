#![cfg(all(feature = "async", feature = "pcap"))]

use crate::capture::source::AsyncPacketSource;
use crate::error::CaptureError;
use crate::types::{CaptureStats, RawPacket};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, UNIX_EPOCH};

pub struct PcapLiveCapture {
    capture: Arc<Mutex<pcap::Capture<pcap::Active>>>,
    packets_read: u64,
}

impl PcapLiveCapture {
    pub fn open(interface: &str) -> Result<Self, CaptureError> {
        let capture = pcap::Capture::from_device(interface)
            .map_err(|e: pcap::Error| CaptureError::OpenFailed(format!("Device {}: {}", interface, e)))?
            .promisc(true)
            .snaplen(65535)
            .timeout(100) // 100ms timeout for responsive async
            .open()
            .map_err(|e: pcap::Error| CaptureError::OpenFailed(e.to_string()))?;

        Ok(Self {
            capture: Arc::new(Mutex::new(capture)),
            packets_read: 0,
        })
    }
}

impl AsyncPacketSource for PcapLiveCapture {
    async fn next_packet(&mut self) -> Result<Option<RawPacket>, CaptureError> {
        // Use spawn_blocking for pcap's blocking read
        let capture: Arc<Mutex<pcap::Capture<pcap::Active>>> = Arc::clone(&self.capture);
        let result = tokio::task::spawn_blocking(move || {
            let mut cap = capture.lock().unwrap();
            match cap.next() {
                Ok(packet) => {
                    let tv_sec = packet.header.ts.tv_sec;
                    let tv_usec = packet.header.ts.tv_usec;
                    let len = packet.header.len;
                    let data = packet.data.to_vec();
                    Ok((data, tv_sec, tv_usec, len))
                }
                Err(pcap::Error::TimeoutExpired) => Err(CaptureError::NoMorePackets),
                Err(e) => Err(CaptureError::ReadFailed(e.to_string())),
            }
        })
        .await
        .map_err(|e: tokio::task::JoinError| CaptureError::ReadFailed(e.to_string()))?;

        match result {
            Ok((data, tv_sec, tv_usec, len)) => {
                self.packets_read += 1;
                let timestamp = UNIX_EPOCH
                    + Duration::from_secs(tv_sec as u64)
                    + Duration::from_micros(tv_usec as u64);

                Ok(Some(RawPacket {
                    data,
                    timestamp,
                    length: len as usize,
                }))
            }
            Err(CaptureError::NoMorePackets) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn stats(&self) -> CaptureStats {
        let mut cap = self.capture.lock().unwrap();
        match cap.stats() {
            Ok(stats) => CaptureStats {
                packets_received: stats.received as u64,
                packets_dropped: stats.dropped as u64,
            },
            Err(_) => CaptureStats {
                packets_received: self.packets_read,
                packets_dropped: 0,
            },
        }
    }

    fn set_filter(&mut self, filter: &str) -> Result<(), CaptureError> {
        let mut cap = self.capture.lock().unwrap();
        cap.filter(filter)
            .map_err(|e| CaptureError::OpenFailed(format!("BPF filter failed: {}", e)))
    }
}
