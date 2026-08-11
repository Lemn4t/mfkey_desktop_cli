use crate::flipper::{FlipperError, Result};
use btleplug::api::Manager as _;
use btleplug::platform::{Adapter, Manager};
use once_cell::sync::Lazy;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::OnceCell;

pub static BLE_RT: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("ble-rt")
        .build()
        .expect("failed to build BLE tokio runtime")
});

static ADAPTER: OnceCell<Adapter> = OnceCell::const_new();

pub async fn shared_adapter() -> Result<Adapter> {
    ADAPTER
        .get_or_try_init(|| async {
            let manager = Manager::new()
                .await
                .map_err(|e| FlipperError::Ble(e.to_string()))?;
            let adapters = manager
                .adapters()
                .await
                .map_err(|e| FlipperError::Ble(e.to_string()))?;
            adapters
                .into_iter()
                .next()
                .ok_or_else(|| FlipperError::Ble("no BLE adapter available".into()))
        })
        .await
        .cloned()
}
