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

    fn hardnested_begin(&self, _index: usize, _total: usize, _label: &str) {}

    fn hardnested_line(&self, _line: &str) {}

    fn hardnested_end(&self) {}
}
