pub mod find;
pub mod framing;
pub mod session;
pub mod storage;
pub mod transport;

pub use session::FlipperSession;

#[allow(unused_imports)]
pub use find::{find_all_flipper_ports, find_flipper_port};

use std::fmt;

#[derive(Debug)]
pub enum FlipperError {
    Io(std::io::Error),
    Serial(serialport::Error),
    Decode(prost::DecodeError),
    Encode(prost::EncodeError),
    CommandStatus(i32),
    NotFound,
    Timeout,
    Protocol(String),
}

impl fmt::Display for FlipperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlipperError::Io(e) => write!(f, "io error: {e}"),
            FlipperError::Serial(e) => write!(f, "serial error: {e}"),
            FlipperError::Decode(e) => write!(f, "protobuf decode error: {e}"),
            FlipperError::Encode(e) => write!(f, "protobuf encode error: {e}"),
            FlipperError::CommandStatus(c) => write!(f, "flipper command error, status={c}"),
            FlipperError::NotFound => write!(f, "Flipper Zero not found"),
            FlipperError::Timeout => write!(f, "operation timed out"),
            FlipperError::Protocol(s) => write!(f, "protocol error: {s}"),
        }
    }
}

impl std::error::Error for FlipperError {}

impl From<std::io::Error> for FlipperError {
    fn from(e: std::io::Error) -> Self {
        FlipperError::Io(e)
    }
}
impl From<serialport::Error> for FlipperError {
    fn from(e: serialport::Error) -> Self {
        FlipperError::Serial(e)
    }
}
impl From<prost::DecodeError> for FlipperError {
    fn from(e: prost::DecodeError) -> Self {
        FlipperError::Decode(e)
    }
}
impl From<prost::EncodeError> for FlipperError {
    fn from(e: prost::EncodeError) -> Self {
        FlipperError::Encode(e)
    }
}

pub type Result<T> = std::result::Result<T, FlipperError>;
