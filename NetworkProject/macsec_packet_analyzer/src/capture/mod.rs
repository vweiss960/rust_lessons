pub mod source;

#[cfg(feature = "cli")]
pub mod file;

#[cfg(all(feature = "async", feature = "pcap"))]
pub mod pcap_live;

#[cfg(all(target_os = "linux", feature = "async"))]
pub mod af_packet;

#[cfg(all(target_os = "linux", feature = "async"))]
pub mod xdp;

#[cfg(all(target_os = "linux", feature = "napatech"))]
pub mod napatech;

#[cfg(all(feature = "async", feature = "pcap"))]
pub mod replay;

pub use source::PacketSource;

#[cfg(feature = "async")]
pub use source::AsyncPacketSource;

#[cfg(feature = "cli")]
pub use file::FileCapture;

#[cfg(all(feature = "async", feature = "pcap"))]
pub use pcap_live::PcapLiveCapture;

#[cfg(all(target_os = "linux", feature = "async"))]
pub use af_packet::AfPacketCapture;

#[cfg(all(target_os = "linux", feature = "async"))]
pub use xdp::XdpCapture;

#[cfg(all(target_os = "linux", feature = "napatech"))]
pub use napatech::{NapatechCapture, NapatechConfig, NapatechCaptureMode, NapatechStats};

#[cfg(all(feature = "async", feature = "pcap"))]
pub use replay::{ReplayCapture, ReplayMode};
