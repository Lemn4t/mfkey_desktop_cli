use super::{FlipperError, Result};
use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::{Duration, Instant};

const TIMEOUT_DRAIN: Duration = Duration::from_millis(50);
const TIMEOUT_NORMAL: Duration = Duration::from_secs(5);

pub trait RpcTransport {
    fn read_u8(&mut self, deadline: Instant) -> Result<u8>;
    fn read_exact(&mut self, len: usize, deadline: Instant) -> Result<Vec<u8>>;
    fn write_all(&mut self, data: &[u8]) -> Result<()>;
}

fn is_timeout(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::TimedOut || e.raw_os_error() == Some(121)
}

pub struct SerialTransport {
    pub port: Box<dyn SerialPort>,
}

impl SerialTransport {
    pub fn open(port_name: &str) -> Result<Self> {
        let mut port = serialport::new(port_name, 230_400)
            .timeout(TIMEOUT_DRAIN)
            .open()?;

        let _ = port.write_data_terminal_ready(true);
        let _ = port.write_request_to_send(true);

        Ok(Self { port })
    }

    pub fn start_rpc_session(&mut self) -> Result<()> {
        self.drain_until_silent(Duration::from_millis(300));

        self.port.set_timeout(TIMEOUT_NORMAL)?;
        self.port.write_all(b"start_rpc_session\r")?;
        self.port.flush()?;

        let echo_deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if self.read_u8(echo_deadline)? == b'\n' {
                break;
            }
        }

        self.drain_until_silent(Duration::from_millis(400));

        Ok(())
    }

    fn drain_until_silent(&mut self, quiet: Duration) -> Vec<u8> {
        let _ = self.port.set_timeout(Duration::from_millis(50));

        let mut acc = Vec::new();
        let mut buf = [0u8; 256];
        let mut last_data = Instant::now();

        loop {
            match self.port.read(&mut buf) {
                Ok(0) => {}
                Ok(n) => {
                    acc.extend_from_slice(&buf[..n]);
                    last_data = Instant::now();
                }
                Err(ref e) if is_timeout(e) => {
                    if last_data.elapsed() >= quiet {
                        break;
                    }
                }
                Err(_) => break,
            }
            if last_data.elapsed() >= quiet {
                break;
            }
        }

        acc
    }

    fn poll_read(&mut self, buf: &mut [u8], deadline: Instant) -> Result<usize> {
        loop {
            match self.port.read(buf) {
                Ok(0) => {}
                Ok(n) => return Ok(n),
                Err(ref e) if is_timeout(e) => {}
                Err(e) => return Err(FlipperError::Io(e)),
            }
            if Instant::now() >= deadline {
                return Err(FlipperError::Timeout);
            }
        }
    }
}

impl RpcTransport for SerialTransport {
    fn read_u8(&mut self, deadline: Instant) -> Result<u8> {
        let mut b = [0u8; 1];
        self.poll_read(&mut b, deadline)?;
        Ok(b[0])
    }

    fn read_exact(&mut self, len: usize, deadline: Instant) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(len);
        let mut tmp = vec![0u8; len];
        while out.len() < len {
            let want = len - out.len();
            let n = self.poll_read(&mut tmp[..want], deadline)?;
            out.extend_from_slice(&tmp[..n]);
            if Instant::now() >= deadline && out.len() < len {
                return Err(FlipperError::Timeout);
            }
        }
        Ok(out)
    }

    fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.port.write_all(data)?;
        self.port.flush()?;
        Ok(())
    }
}
