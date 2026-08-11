use crate::flipper::ble::runtime::{BLE_RT, shared_adapter};
use crate::flipper::ble::transport::{BleTransport, RxShared};
use crate::flipper::ble::{OVERFLOW_CHAR, RPC_STATE_CHAR, RX_CHAR, TX_CHAR};
use crate::flipper::{FlipperError, Result};
use btleplug::api::{
    Central, CentralEvent, CharPropFlags, Characteristic, Peripheral as _, ScanFilter,
    ValueNotification,
};
use btleplug::platform::{Adapter, Peripheral};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::sync::oneshot;

fn map(e: impl std::fmt::Display) -> FlipperError {
    super::ble_error(e)
}

fn parse_flow_ctrl(bytes: &[u8]) -> Option<u32> {
    match bytes.len() {
        2 => Some(u16::from_le_bytes([bytes[0], bytes[1]]) as u32),
        4 => Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
        _ => None,
    }
}

async fn find_by_id(adapter: &Adapter, id: &str) -> Result<Option<Peripheral>> {
    let peripherals = adapter.peripherals().await.map_err(map)?;
    for p in peripherals {
        if p.id().to_string() == id {
            return Ok(Some(p));
        }
    }
    Ok(None)
}

async fn scan_for_id(adapter: &Adapter, target: &str) -> Result<Peripheral> {
    let mut events = adapter.events().await.map_err(map)?;
    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(map)?;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(6);
    let outcome = loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break Err(FlipperError::Ble("BLE peripheral not found after scan".into()));
        }
        match tokio::time::timeout(remaining, events.next()).await {
            Ok(Some(CentralEvent::DeviceDiscovered(pid)))
            | Ok(Some(CentralEvent::DeviceUpdated(pid))) => {
                if pid.to_string() == target
                    && let Some(p) = find_by_id(adapter, target).await?
                {
                    break Ok(p);
                }
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => {
                break Err(FlipperError::Ble("BLE peripheral not found after scan".into()));
            }
        }
    };
    let _ = adapter.stop_scan().await;
    outcome
}

fn find_char(peripheral: &Peripheral, uuid: uuid::Uuid) -> Result<Characteristic> {
    peripheral
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == uuid)
        .ok_or_else(|| FlipperError::Ble(format!("BLE characteristic {uuid} not found on device")))
}

pub fn connect_ble_blocking(id: &str) -> Result<BleTransport> {
    BLE_RT.block_on(connect_ble_async(id.to_string()))
}

async fn connect_ble_async(id: String) -> Result<BleTransport> {
    let adapter = shared_adapter().await?;

    let peripheral = match find_by_id(&adapter, &id).await? {
        Some(p) => p,
        None => scan_for_id(&adapter, &id).await?,
    };

    let already_connected = peripheral.is_connected().await.unwrap_or(false);
    if !already_connected {
        match tokio::time::timeout(Duration::from_secs(15), peripheral.connect()).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(map(e)),
            Err(_) => {
                let _ = peripheral.disconnect().await;
                return Err(FlipperError::Ble(
                    "BLE connect timed out — is the Flipper paired in the OS Bluetooth settings and in range?".into(),
                ));
            }
        }
    }

    peripheral.discover_services().await.map_err(map)?;

    let tx_char = find_char(&peripheral, TX_CHAR)?;
    let rx_char = find_char(&peripheral, RX_CHAR)?;
    let overflow_char = find_char(&peripheral, OVERFLOW_CHAR)?;
    let rpc_state_char = find_char(&peripheral, RPC_STATE_CHAR)?;

    for c in [&tx_char, &overflow_char, &rpc_state_char] {
        if c.properties.contains(CharPropFlags::NOTIFY)
            || c.properties.contains(CharPropFlags::INDICATE)
        {
            peripheral.subscribe(c).await.map_err(map)?;
        }
    }

    let rx = RxShared::new();

    let initial_free = if overflow_char.properties.contains(CharPropFlags::READ) {
        match peripheral.read(&overflow_char).await {
            Ok(bytes) => parse_flow_ctrl(&bytes).unwrap_or(1024),
            Err(_) => 1024,
        }
    } else {
        1024
    };
    let overflow = Arc::new(AtomicU32::new(initial_free));

    let stream = peripheral.notifications().await.map_err(map)?;

    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

    let peripheral_task = peripheral.clone();
    let rx_task = Arc::clone(&rx);
    let overflow_task = Arc::clone(&overflow);

    BLE_RT.spawn(async move {
        run_notification_task(peripheral_task, rx_task, overflow_task, stream, cancel_rx).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    Ok(BleTransport::new(
        peripheral, rx_char, rx, overflow, cancel_tx,
    ))
}

async fn run_notification_task(
    peripheral: Peripheral,
    rx: Arc<RxShared>,
    overflow: Arc<AtomicU32>,
    mut stream: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
    mut cancel_rx: oneshot::Receiver<()>,
) {
    let reason: String = loop {
        tokio::select! {
            _ = &mut cancel_rx => break "disconnect requested".into(),
            next = stream.next() => {
                let Some(n) = next else {
                    break "BLE notifications stream ended".into();
                };
                if n.uuid == TX_CHAR {
                    rx.push(&n.value);
                } else if n.uuid == OVERFLOW_CHAR
                    && let Some(v) = parse_flow_ctrl(&n.value)
                {
                    overflow.store(v, Ordering::SeqCst);
                }
            }
        }
    };

    let _ = peripheral.disconnect().await;
    rx.close(reason);
}
