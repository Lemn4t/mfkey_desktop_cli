use uuid::{Uuid, uuid};

pub mod connection;
pub mod runtime;
pub mod scanner;
pub mod transport;

pub const SERIAL_SERVICE: Uuid = uuid!("8fe5b3d5-2e7f-4a98-2a48-7acc60fe0000");
pub const ADVERTISED_SERVICE: Uuid = uuid!("00003083-0000-1000-8000-00805f9b34fb");
pub const TX_CHAR: Uuid = uuid!("19ed82ae-ed21-4c9d-4145-228e61fe0000");
pub const RX_CHAR: Uuid = uuid!("19ed82ae-ed21-4c9d-4145-228e62fe0000");
pub const OVERFLOW_CHAR: Uuid = uuid!("19ed82ae-ed21-4c9d-4145-228e63fe0000");
pub const RPC_STATE_CHAR: Uuid = uuid!("19ed82ae-ed21-4c9d-4145-228e64fe0000");

pub const FLIPPER_NAME_PREFIX: &str = "Flipper ";
