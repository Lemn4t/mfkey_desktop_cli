use crate::core::model::MfClassicKey;

pub trait Reporter: Send + Sync {
    fn begin_progress(&self, total_nonces: usize);

    fn update_progress(
        &self,
        nonce_current: usize,
        nonce_total: usize,
        msb_current: usize,
        msb_total: usize,
        stage_progress: f32,
        uid: u32,
    );

    fn found_key(&self, key: &MfClassicKey);
}
