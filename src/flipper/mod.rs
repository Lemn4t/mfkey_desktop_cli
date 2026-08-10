pub mod ble;
pub mod find;
pub mod framing;
pub mod session;
pub mod storage;
pub mod transport;

pub use session::FlipperSession;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlipperError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serial error: {0}")]
    Serial(#[from] serialport::Error),
    #[error("protobuf decode error: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("protobuf encode error: {0}")]
    Encode(#[from] prost::EncodeError),
    #[error("flipper command error, status={0}")]
    CommandStatus(i32),
    #[error("operation timed out")]
    Timeout,
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("bluetooth error: {0}")]
    Ble(String),
}

pub type Result<T> = std::result::Result<T, FlipperError>;
