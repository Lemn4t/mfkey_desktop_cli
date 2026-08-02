use super::transport::Transport;
use super::{FlipperError, Result};
use crate::pb;
use prost::Message;
use std::time::{Duration, Instant};

pub fn read_message(t: &mut Transport, timeout: Duration) -> Result<pb::Main> {
    let deadline = Instant::now() + timeout;

    let mut len: u64 = 0;
    let mut shift = 0u32;
    loop {
        let b = t.read_u8(deadline)?;
        len |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(FlipperError::Protocol("varint length too long".into()));
        }
    }

    if len == 0 {
        return Err(FlipperError::Protocol("zero-length frame".into()));
    }
    if len > 1_000_000 {
        return Err(FlipperError::Protocol(format!("frame too large: {len}")));
    }

    let body = t.read_exact(len as usize, deadline)?;

    let msg = pb::Main::decode(&body[..]).map_err(FlipperError::Decode)?;
    Ok(msg)
}

pub fn write_message(t: &mut Transport, msg: &pb::Main) -> Result<()> {
    let mut buf = Vec::with_capacity(msg.encoded_len() + 8);
    msg.encode_length_delimited(&mut buf)
        .map_err(FlipperError::Encode)?;
    t.write_all(&buf)?;
    Ok(())
}
