use super::framing::{read_message, write_message};
use super::transport::{RpcTransport, SerialTransport};
use super::{FlipperError, Result};
use crate::{pb, pb_system};
use std::time::Duration;

pub struct FlipperSession {
    pub(crate) t: Box<dyn RpcTransport>,
    next_id: u32,
}

impl FlipperSession {
    pub fn open(port_name: &str) -> Result<Self> {
        let mut t = SerialTransport::open(port_name)?;
        t.start_rpc_session()?;
        Self::from_transport(Box::new(t))
    }

    pub fn from_transport(t: Box<dyn RpcTransport>) -> Result<Self> {
        let mut s = Self { t, next_id: 1 };
        s.ping()?;
        Ok(s)
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == 0 {
            self.next_id = 1;
        }
        id
    }

    fn send(&mut self, content: pb::main::Content) -> Result<u32> {
        let id = self.alloc_id();
        let msg = pb::Main {
            command_id: id,
            command_status: 0,
            has_next: false,
            content: Some(content),
        };
        write_message(&mut *self.t, &msg)?;
        Ok(id)
    }

    fn recv_for(&mut self, id: u32, timeout: Duration) -> Result<pb::Main> {
        loop {
            let msg = read_message(&mut *self.t, timeout)?;
            if msg.command_id == id {
                return Ok(msg);
            }
        }
    }

    fn check_status(msg: &pb::Main) -> Result<()> {
        if msg.command_status != 0 {
            return Err(FlipperError::CommandStatus(msg.command_status));
        }
        Ok(())
    }

    pub fn request(&mut self, content: pb::main::Content, timeout: Duration) -> Result<pb::Main> {
        let id = self.send(content)?;
        let resp = self.recv_for(id, timeout)?;
        Self::check_status(&resp)?;
        Ok(resp)
    }

    pub fn request_stream(
        &mut self,
        content: pb::main::Content,
        timeout: Duration,
    ) -> Result<Vec<pb::main::Content>> {
        let id = self.send(content)?;
        let mut parts = Vec::new();
        loop {
            let resp = self.recv_for(id, timeout)?;
            Self::check_status(&resp)?;
            let has_next = resp.has_next;
            if let Some(c) = resp.content {
                parts.push(c);
            }
            if !has_next {
                break;
            }
        }
        Ok(parts)
    }

    pub fn ping(&mut self) -> Result<()> {
        let content = pb::main::Content::SystemPingRequest(pb_system::PingRequest::default());
        let resp = self.request(content, Duration::from_secs(3))?;
        match resp.content {
            Some(pb::main::Content::SystemPingResponse(_)) => Ok(()),
            _ => Err(FlipperError::Protocol("unexpected ping response".into())),
        }
    }

    pub(crate) fn alloc_id_pub(&mut self) -> u32 {
        self.alloc_id()
    }
    pub(crate) fn recv_for_pub(&mut self, id: u32, timeout: Duration) -> Result<pb::Main> {
        self.recv_for(id, timeout)
    }
}
