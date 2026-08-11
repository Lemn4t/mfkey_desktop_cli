use crate::flipper::FlipperError;
use uuid::{Uuid, uuid};

pub mod connection;
pub mod runtime;
pub mod scanner;
pub mod transport;

pub(crate) fn ble_error(raw: impl std::fmt::Display) -> FlipperError {
    let s = raw.to_string();
    let lower = s.to_lowercase();

    let hint = if lower.contains("0x800710df") || lower.contains("not ready") {
        Some("Bluetooth appears to be off or not ready — turn Bluetooth on and try again")
    } else if lower.contains("not paired")
        || lower.contains("pairing")
        || lower.contains("authentication")
        || lower.contains("0x800700b7")
    {
        Some("the Flipper is not paired — pair it in your operating system's Bluetooth settings first")
    } else if lower.contains("no ble adapter") || lower.contains("no adapter") {
        Some("no Bluetooth adapter was found on this system")
    } else if lower.contains("access is denied") || lower.contains("0x80070005") {
        Some("access denied — check this app's Bluetooth permissions in the OS settings")
    } else {
        None
    };

    match hint {
        Some(h) => FlipperError::Ble(format!("{h} ({s})")),
        None => FlipperError::Ble(s),
    }
}

pub const SERIAL_SERVICE: Uuid = uuid!("8fe5b3d5-2e7f-4a98-2a48-7acc60fe0000");
pub const ADVERTISED_SERVICE: Uuid = uuid!("00003083-0000-1000-8000-00805f9b34fb");
pub const TX_CHAR: Uuid = uuid!("19ed82ae-ed21-4c9d-4145-228e61fe0000");
pub const RX_CHAR: Uuid = uuid!("19ed82ae-ed21-4c9d-4145-228e62fe0000");
pub const OVERFLOW_CHAR: Uuid = uuid!("19ed82ae-ed21-4c9d-4145-228e63fe0000");
pub const RPC_STATE_CHAR: Uuid = uuid!("19ed82ae-ed21-4c9d-4145-228e64fe0000");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ble_error_hints_bluetooth_off_and_keeps_original() {
        let m = ble_error("Error { code: HRESULT(0x800710DF), ... }").to_string();
        assert!(m.to_lowercase().contains("bluetooth"));
        assert!(m.contains("0x800710DF"));
    }

    #[test]
    fn ble_error_hints_pairing() {
        let m = ble_error("The device is not paired.").to_string();
        assert!(m.to_lowercase().contains("pair"));
    }

    #[test]
    fn ble_error_passes_unknown_through() {
        let m = ble_error("some weird error").to_string();
        assert_eq!(m, "bluetooth error: some weird error");
    }
}
