use crate::flipper::Result;
use crate::flipper::ble::runtime::{BLE_RT, shared_adapter};
use crate::flipper::ble::{ADVERTISED_SERVICE, SERIAL_SERVICE};
use btleplug::api::{Central, Peripheral as _, ScanFilter};
use futures::StreamExt;
use std::time::Duration;

pub struct BleDevice {
    pub id: String,
    pub name: String,
    pub rssi: Option<i16>,
    pub paired: bool,
}

pub async fn list_ble_devices() -> Result<Vec<BleDevice>> {
    let Ok(adapter) = shared_adapter().await else {
        return Ok(vec![]);
    };

    let mut events = adapter
        .events()
        .await
        .map_err(super::ble_error)?;
    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(super::ble_error)?;

    let deadline = tokio::time::Instant::now() + Duration::from_millis(5000);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, events.next()).await {
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(_) => break,
        }
    }

    let _ = adapter.stop_scan().await;

    let peripherals = adapter
        .peripherals()
        .await
        .map_err(super::ble_error)?;

    let mut out = Vec::new();
    for p in peripherals {
        let props = match p.properties().await {
            Ok(Some(pr)) => pr,
            _ => continue,
        };
        let name = props.local_name.clone().unwrap_or_default();
        let matches_advertised = props.services.contains(&ADVERTISED_SERVICE);
        let matches_serial = props.services.contains(&SERIAL_SERVICE);
        let name_looks_like_flipper = name.to_lowercase().contains("flipper");
        if !matches_advertised && !matches_serial && !name_looks_like_flipper {
            continue;
        }
        out.push(BleDevice {
            id: p.id().to_string(),
            name: if name.is_empty() {
                "Flipper (unknown)".into()
            } else {
                name
            },
            rssi: props.rssi,
            paired: matches_serial,
        });
    }
    Ok(out)
}

pub fn list_ble_devices_blocking() -> Result<Vec<BleDevice>> {
    BLE_RT.block_on(list_ble_devices())
}
