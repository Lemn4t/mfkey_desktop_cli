use super::transport::RpcTransport;
use super::{FlipperError, Result};
use crate::pb;
use prost::Message;
use std::time::{Duration, Instant};

const MAX_FRAME_LEN: u64 = 1_000_000;

fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
    out
}

fn decode_varint<F>(mut next: F) -> Result<u64>
where
    F: FnMut() -> Result<u8>,
{
    let mut len: u64 = 0;
    let mut shift = 0u32;
    loop {
        let b = next()?;
        len |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(FlipperError::Protocol("varint length too long".into()));
        }
    }
    Ok(len)
}

fn validate_frame_len(len: u64) -> Result<u64> {
    if len == 0 {
        return Err(FlipperError::Protocol("zero-length frame".into()));
    }
    if len > MAX_FRAME_LEN {
        return Err(FlipperError::Protocol(format!("frame too large: {len}")));
    }
    Ok(len)
}

pub fn read_message(t: &mut dyn RpcTransport, timeout: Duration) -> Result<pb::Main> {
    let deadline = Instant::now() + timeout;

    let len = validate_frame_len(decode_varint(|| t.read_u8(deadline))?)?;

    let body = t.read_exact(len as usize, deadline)?;

    let msg = pb::Main::decode(&body[..]).map_err(FlipperError::Decode)?;
    Ok(msg)
}

pub fn write_message(t: &mut dyn RpcTransport, msg: &pb::Main) -> Result<()> {
    let mut body = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut body).map_err(FlipperError::Encode)?;

    let mut buf = encode_varint(body.len() as u64);
    buf.extend_from_slice(&body);
    t.write_all(&buf)?;
    Ok(())
}

#[cfg(test)]
#[path = "../tests/flipper_framing.rs"]
mod tests;
