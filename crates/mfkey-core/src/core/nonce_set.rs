use crate::core::model::{HardNestedNonce, Nonce};

#[derive(Debug, Clone, Default)]
pub struct NonceSet {
    pub nonces: Vec<Nonce>,
    pub hardnested: Vec<HardNestedNonce>,
    pub unrecognized: usize,
}
