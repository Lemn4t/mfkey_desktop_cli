use crate::flipper::ble::runtime::BLE_RT;
use crate::flipper::transport::RpcTransport;
use crate::flipper::{FlipperError, Result};
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

const MAX_WRITE_LEN: usize = 160;
const FLOW_WAIT_TIMEOUT: Duration = Duration::from_secs(5);
const FLOW_POLL_INTERVAL: Duration = Duration::from_millis(10);

pub struct RxBuffer {
    pub bytes: VecDeque<u8>,
    pub closed: bool,
    pub close_reason: Option<String>,
}

pub struct RxShared {
    pub inner: Mutex<RxBuffer>,
    pub cv: Condvar,
}

impl RxShared {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(RxBuffer {
                bytes: VecDeque::new(),
                closed: false,
                close_reason: None,
            }),
            cv: Condvar::new(),
        })
    }

    pub fn push(&self, bytes: &[u8]) {
        let mut g = self.lock_recover();
        g.bytes.extend(bytes);
        self.cv.notify_all();
    }

    pub fn close(&self, reason: impl Into<String>) {
        let mut g = self.lock_recover();
        if !g.closed {
            g.closed = true;
            g.close_reason = Some(reason.into());
        }
        self.cv.notify_all();
    }

    pub fn is_closed(&self) -> bool {
        self.lock_recover().closed
    }

    fn lock_recover(&self) -> MutexGuard<'_, RxBuffer> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub struct BleTransport {
    peripheral: Peripheral,
    rx_char: Characteristic,
    rx: Arc<RxShared>,
    overflow: Arc<AtomicU32>,
    cancel: Option<oneshot::Sender<()>>,
}

impl BleTransport {
    pub fn new(
        peripheral: Peripheral,
        rx_char: Characteristic,
        rx: Arc<RxShared>,
        overflow: Arc<AtomicU32>,
        cancel: oneshot::Sender<()>,
    ) -> Self {
        Self {
            peripheral,
            rx_char,
            rx,
            overflow,
            cancel: Some(cancel),
        }
    }

    fn wait_free(&self, needed: u32) -> Result<()> {
        let deadline = Instant::now() + FLOW_WAIT_TIMEOUT;
        loop {
            if self.rx.is_closed() {
                return Err(FlipperError::Ble("BLE transport closed during write".into()));
            }
            if self.overflow.load(Ordering::SeqCst) >= needed {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(FlipperError::Timeout);
            }
            std::thread::sleep(FLOW_POLL_INTERVAL);
        }
    }
}

impl RpcTransport for BleTransport {
    fn read_u8(&mut self, deadline: Instant) -> Result<u8> {
        Ok(self.read_exact(1, deadline)?[0])
    }

    fn read_exact(&mut self, len: usize, deadline: Instant) -> Result<Vec<u8>> {
        if len == 0 {
            return Ok(Vec::new());
        }

        let mut g = self.rx.lock_recover();
        while g.bytes.len() < len && !g.closed {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(FlipperError::Timeout);
            }
            let (ng, wr) = self
                .rx
                .cv
                .wait_timeout(g, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            g = ng;
            if wr.timed_out() && g.bytes.len() < len && !g.closed {
                return Err(FlipperError::Timeout);
            }
        }

        if g.bytes.len() < len {
            let reason = g
                .close_reason
                .clone()
                .unwrap_or_else(|| "BLE transport closed".into());
            return Err(FlipperError::Ble(reason));
        }

        Ok(g.bytes.drain(..len).collect())
    }

    fn write_all(&mut self, data: &[u8]) -> Result<()> {
        for piece in data.chunks(MAX_WRITE_LEN) {
            let n = piece.len() as u32;
            self.wait_free(n)?;
            BLE_RT
                .block_on(
                    self.peripheral
                        .write(&self.rx_char, piece, WriteType::WithResponse),
                )
                .map_err(|e| FlipperError::Ble(e.to_string()))?;
            let _ = self
                .overflow
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                    Some(v.saturating_sub(n))
                });
        }
        Ok(())
    }
}

impl Drop for BleTransport {
    fn drop(&mut self) {
        if let Some(tx) = self.cancel.take() {
            let _ = tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rxbuffer_accumulates_across_pushes() {
        let rx = RxShared::new();
        rx.push(&[1, 2, 3]);
        rx.push(&[4, 5]);
        let g = rx.inner.lock().unwrap();
        assert_eq!(
            g.bytes.iter().copied().collect::<Vec<u8>>(),
            vec![1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn rxbuffer_close_sets_flag_and_reason_once() {
        let rx = RxShared::new();
        rx.close("boom");
        rx.close("second");
        assert!(rx.is_closed());
        let g = rx.inner.lock().unwrap();
        assert_eq!(g.close_reason.as_deref(), Some("boom"));
    }
}
