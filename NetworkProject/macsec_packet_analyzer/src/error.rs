use thiserror::Error;

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Failed to open capture: {0}")]
    OpenFailed(String),

    #[error("Failed to read packet: {0}")]
    ReadFailed(String),

    #[error("No more packets")]
    NoMorePackets,

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("XDP not available: {0}")]
    XdpNotAvailable(String),

    #[error("AF_PACKET error: {0}")]
    AfPacketError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Packet too short for protocol")]
    PacketTooShort,

    #[error("Invalid protocol format: {0}")]
    InvalidFormat(String),
}

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Capture error: {0}")]
    Capture(#[from] CaptureError),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
}
