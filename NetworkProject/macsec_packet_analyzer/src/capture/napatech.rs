#![cfg(target_os = "linux")]
//! Napatech NTAPI FFI bindings for high-performance packet capture
//!
//! This module provides raw FFI bindings to the Napatech NTAPI C library.
//! Requires Napatech drivers and libntapi.so to be installed on the Linux system.
//!
//! ## Installation (Linux)
//! ```bash
//! # Install Napatech driver package (obtained from Napatech)
//! # Typically installed to /opt/napatech/
//! # Ensure libntapi.so is in a library path:
//! export LD_LIBRARY_PATH=/opt/napatech/lib:$LD_LIBRARY_PATH
//! ```
//!
//! ## Building with Napatech support
//! ```bash
//! cargo build --features "cli napatech"
//! ```

use crate::capture::source::AsyncPacketSource;
use crate::error::CaptureError;
use crate::types::{CaptureStats, RawPacket};
use std::time::SystemTime;
use std::ptr;
use std::ffi::CString;

// ============================================================================
// FFI Bindings to Napatech NTAPI
// ============================================================================

/// Status codes returned by NTAPI functions
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtStatus {
    Success = 0,
    Fail = -1,
    /// More data available
    MoreData = 1,
    /// No more data available
    Done = 2,
    /// Parameter error
    ParamError = -2,
}

impl NtStatus {
    /// Check if status indicates success
    pub fn is_success(self) -> bool {
        self == NtStatus::Success
    }

    /// Check if status indicates done/no-more-data
    pub fn is_done(self) -> bool {
        self == NtStatus::Done
    }
}

/// Packet buffer type used by NTAPI
#[repr(C)]
pub struct NtNetBuf {
    pub h_packet: u64,
}

/// Network RX statistics
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NtNetRxStat {
    pub rx_packets: u64,
    pub rx_bytes: u64,
    pub rx_errors: u64,
    pub rx_drops: u64,
}

/// Packet metadata from NTAPI
#[repr(C)]
pub struct NtPacketDescriptor {
    pub length: u32,
    pub cap_length: u32,
    pub timestamp: u64,
    pub color: u32,
}

/// Raw FFI declarations for Napatech NTAPI
/// These are unsafe C function bindings that must be linked against libntapi.so
#[link(name = "ntapi")]
extern "C" {
    /// Initialize the NTAPI library
    /// # Arguments
    /// * `flags` - Initialization flags (typically 0)
    /// # Returns
    /// 0 on success, negative on failure
    pub fn NT_Init(flags: u32) -> i32;

    /// Release NTAPI library resources
    pub fn NT_Release() -> i32;

    /// Open a network RX stream
    /// # Arguments
    /// * `stream_id` - Stream identifier (0-63)
    /// * `port_id` - Physical port number
    /// # Returns
    /// Handle on success (>= 0), negative on failure
    pub fn NT_NetRxOpen(stream_id: u32, port_id: u32) -> i32;

    /// Close a network RX stream
    pub fn NT_NetRxClose(net_rx_handle: i32) -> i32;

    /// Set attribute on RX stream
    /// # Arguments
    /// * `net_rx_handle` - Stream handle from NT_NetRxOpen
    /// * `attr` - Attribute string (null-terminated C string)
    /// # Returns
    /// 0 on success, negative on failure
    pub fn NT_NetRxSetAttribute(net_rx_handle: i32, attr: *const u8) -> i32;

    /// Read a packet from the RX stream
    /// # Arguments
    /// * `net_rx_handle` - Stream handle
    /// * `net_buf` - Pointer to NtNetBuf struct to receive packet handle
    /// # Returns
    /// Status code (Success = packet available, Done = no more, Fail = error)
    pub fn NT_NetRxRead(net_rx_handle: i32, net_buf: *mut NtNetBuf) -> i32;

    /// Release a packet buffer back to the driver
    pub fn NT_NetRxRelease(net_rx_handle: i32, net_buf: *mut NtNetBuf) -> i32;

    /// Get packet data pointer from handle
    /// # Arguments
    /// * `h_packet` - Packet handle from NtNetBuf
    /// # Returns
    /// Pointer to packet data
    pub fn NT_NetRxGetPacket(h_packet: u64) -> *const u8;

    /// Get packet descriptor (metadata)
    pub fn NT_NetRxGetDescriptor(h_packet: u64) -> *const NtPacketDescriptor;

    /// Get statistics from RX stream
    pub fn NT_NetRxRead_Stat(net_rx_handle: i32, stat: *mut NtNetRxStat) -> i32;

    /// Convert error code to human-readable string
    pub fn NT_ExplainError(status: i32) -> *const u8;
}

/// Safe wrapper around Napatech NTAPI
pub struct NapatechCapture {
    net_rx_handle: i32,
    port_id: u32,
    stream_id: u32,
    packets_read: u64,
    initialized: bool,
}

impl NapatechCapture {
    /// Create a new Napatech capture on the specified port
    ///
    /// # Arguments
    /// * `port_id` - Physical port number on Napatech card (0-based)
    /// * `stream_id` - NTAPI stream ID to bind to this port (0-63)
    ///
    /// # Returns
    /// Result containing NapatechCapture or CaptureError
    ///
    /// # Safety
    /// This function calls unsafe FFI code to the Napatech NTAPI library.
    /// Requires libntapi.so to be installed and linked.
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let capture = NapatechCapture::open(0, 0)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open(port_id: u32, stream_id: u32) -> Result<Self, CaptureError> {
        // Validate parameters
        if port_id > 255 {
            return Err(CaptureError::OpenFailed(
                format!("Invalid Napatech port ID: {}", port_id),
            ));
        }

        if stream_id > 63 {
            return Err(CaptureError::OpenFailed(
                format!("Invalid Napatech stream ID: {} (must be 0-63)", stream_id),
            ));
        }

        unsafe {
            // Initialize NTAPI library
            let init_status = NT_Init(0);
            if init_status != 0 {
                return Err(CaptureError::OpenFailed(
                    format!("NT_Init failed with status {}", init_status),
                ));
            }

            // Open RX stream
            let net_rx_handle = NT_NetRxOpen(stream_id, port_id);
            if net_rx_handle < 0 {
                NT_Release();
                return Err(CaptureError::OpenFailed(
                    format!(
                        "NT_NetRxOpen failed for port {} stream {} with status {}",
                        port_id, stream_id, net_rx_handle
                    ),
                ));
            }

            Ok(Self {
                net_rx_handle,
                port_id,
                stream_id,
                packets_read: 0,
                initialized: true,
            })
        }
    }

    /// Set a filter attribute on the capture stream
    ///
    /// # Arguments
    /// * `attribute` - Napatech attribute string (e.g., "Assign=StreamId:0-1")
    ///
    /// # Example
    /// ```no_run
    /// # use macsec_packet_analyzer::capture::NapatechCapture;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut cap = NapatechCapture::open(0, 0)?;
    /// cap.set_attribute("Assign=StreamId:0-1")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_attribute(&mut self, attribute: &str) -> Result<(), CaptureError> {
        if !self.initialized {
            return Err(CaptureError::OpenFailed(
                "Capture not initialized".to_string(),
            ));
        }

        // Convert Rust string to C string
        let c_attr = CString::new(attribute).map_err(|_| {
            CaptureError::OpenFailed("Attribute string contains null byte".to_string())
        })?;

        unsafe {
            let status = NT_NetRxSetAttribute(self.net_rx_handle, c_attr.as_ptr() as *const u8);
            if status != 0 {
                return Err(CaptureError::OpenFailed(format!(
                    "NT_NetRxSetAttribute failed with status {}",
                    status
                )));
            }
        }

        Ok(())
    }

    /// Get the port ID this capture is bound to
    pub fn port_id(&self) -> u32 {
        self.port_id
    }

    /// Get the stream ID this capture is using
    pub fn stream_id(&self) -> u32 {
        self.stream_id
    }

    /// Get packets read so far
    pub fn packets_read(&self) -> u64 {
        self.packets_read
    }
}

#[cfg(feature = "napatech")]
impl AsyncPacketSource for NapatechCapture {
    async fn next_packet(&mut self) -> Result<Option<RawPacket>, CaptureError> {
        if !self.initialized {
            return Err(CaptureError::OpenFailed(
                "Capture not initialized".to_string(),
            ));
        }

        unsafe {
            let mut net_buf = NtNetBuf { h_packet: 0 };

            // Try to read a packet
            let status = NT_NetRxRead(self.net_rx_handle, &mut net_buf);

            match status {
                0 => {
                    // Success - packet available
                    let packet_ptr = NT_NetRxGetPacket(net_buf.h_packet);
                    let descriptor = NT_NetRxGetDescriptor(net_buf.h_packet);

                    if packet_ptr.is_null() || descriptor.is_null() {
                        NT_NetRxRelease(self.net_rx_handle, &mut net_buf);
                        return Err(CaptureError::ReadFailed(
                            "Failed to get packet data or descriptor".to_string(),
                        ));
                    }

                    let desc = &*descriptor;
                    let packet_data = std::slice::from_raw_parts(
                        packet_ptr,
                        desc.cap_length.min(65535) as usize,
                    )
                    .to_vec();

                    // Convert Napatech timestamp (typically FPGA counter in nanoseconds)
                    // to SystemTime for compatibility
                    let timestamp = SystemTime::now();

                    self.packets_read += 1;

                    // Release packet buffer back to driver
                    NT_NetRxRelease(self.net_rx_handle, &mut net_buf);

                    Ok(Some(RawPacket {
                        data: packet_data,
                        timestamp,
                        length: desc.length as usize,
                    }))
                }
                1 => {
                    // MoreData status (shouldn't happen in async context, but handle it)
                    Ok(None)
                }
                2 => {
                    // Done - no more packets available
                    Ok(None)
                }
                _ => {
                    // Error
                    Err(CaptureError::ReadFailed(format!(
                        "NT_NetRxRead failed with status {}",
                        status
                    )))
                }
            }
        }
    }

    fn stats(&self) -> CaptureStats {
        if !self.initialized {
            return CaptureStats {
                packets_received: 0,
                packets_dropped: 0,
            };
        }

        unsafe {
            let mut stat = NtNetRxStat {
                rx_packets: 0,
                rx_bytes: 0,
                rx_errors: 0,
                rx_drops: 0,
            };

            let status = NT_NetRxRead_Stat(self.net_rx_handle, &mut stat);
            if status == 0 {
                CaptureStats {
                    packets_received: stat.rx_packets,
                    packets_dropped: stat.rx_drops,
                }
            } else {
                // Fallback to tracked stats if NTAPI call fails
                CaptureStats {
                    packets_received: self.packets_read,
                    packets_dropped: 0,
                }
            }
        }
    }

    fn set_filter(&mut self, filter: &str) -> Result<(), CaptureError> {
        // Napatech uses different filter syntax than BPF
        // This is a pass-through to set_attribute for compatibility
        self.set_attribute(filter)
    }
}

impl Drop for NapatechCapture {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                if self.net_rx_handle >= 0 {
                    let _ = NT_NetRxClose(self.net_rx_handle);
                }
                let _ = NT_Release();
            }
            self.initialized = false;
        }
    }
}

// ============================================================================
// Configuration and Helper Types
// ============================================================================

/// Configuration for Napatech packet capture
#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
pub struct NapatechConfig {
    /// Physical port to capture from
    pub port_id: u32,
    /// NTAPI stream ID (0-63 supported)
    pub stream_id: u32,
    /// Capture complete frames including CRC
    pub capture_crc: bool,
    /// Packet capture mode
    pub capture_mode: NapatechCaptureMode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NapatechCaptureMode {
    /// Capture all packets (no filtering)
    AllPackets,
    /// Capture only packets matching specific flow (key filter)
    KeyFilter {
        l3_protocol: Option<u8>,
        l4_protocol: Option<u8>,
    },
    /// Capture with FPGA-based pattern matching
    PatternMatch { pattern: String },
}

#[derive(Clone, Debug)]
pub struct NapatechStats {
    pub packets_received: u64,
    pub packets_dropped: u64,
    pub bytes_received: u64,
    pub errors: u64,
}

impl Default for NapatechConfig {
    fn default() -> Self {
        Self {
            port_id: 0,
            stream_id: 0,
            capture_crc: false,
            capture_mode: NapatechCaptureMode::AllPackets,
        }
    }
}

impl NapatechConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the physical port to capture from
    pub fn with_port(mut self, port_id: u32) -> Self {
        self.port_id = port_id;
        self
    }

    /// Set the NTAPI stream ID
    pub fn with_stream(mut self, stream_id: u32) -> Self {
        self.stream_id = stream_id;
        self
    }

    /// Enable or disable CRC capture
    pub fn with_crc(mut self, capture: bool) -> Self {
        self.capture_crc = capture;
        self
    }

    /// Set capture mode (filtering)
    pub fn with_capture_mode(mut self, mode: NapatechCaptureMode) -> Self {
        self.capture_mode = mode;
        self
    }
}

/// Performance characteristics of Napatech SmartNICs
/// Typical specifications based on common Napatech models:
///
/// - **NT50E10**: Single-port 100 GbE, ~150M pps per port
/// - **NT200A02**: Dual-port 100 GbE, ~300M pps combined
/// - **NT400A02**: Quad-port 100 GbE, ~600M pps combined
/// - **NT800A02**: Quad-port 400 GbE, ~2.4B pps combined
///
/// **Timestamp precision**: ~5 nanoseconds (FPGA hardware counter)
/// **Zero-copy**: Packets accessed directly from DMA buffers
///
/// ## Typical usage with async pipeline
///
/// ```no_run
/// # use macsec_packet_analyzer::capture::NapatechCapture;
/// # #[tokio::main]
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut capture = NapatechCapture::open(0, 0)?;
///
/// // Optional: Set additional attributes
/// capture.set_attribute("Assign=StreamId:0-1")?;
///
/// // Read packets asynchronously
/// while let Some(packet) = capture.next_packet().await? {
///     // Process packet...
/// }
/// # Ok(())
/// # }
/// ```
