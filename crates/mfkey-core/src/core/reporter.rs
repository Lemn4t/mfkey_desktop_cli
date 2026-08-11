use crate::core::model::MfClassicKey;

pub trait Reporter: Send + Sync {
    fn loading(&self, _file_path: &str) {}

    fn nonce_loaded(&self, _index: usize, _uid: u32, _attack_type: &str) {}

    fn unrecognized_lines(&self, _count: usize) {}

    fn hardnested_loaded(&self, _count: usize, _targets: &[(u32, u8)]) {}

    fn loading_complete(&self, _total: usize) {}

    fn attack_start(&self) {}

    fn error(&self, _msg: &str) {}

    fn summary(&self, _total_nonces: usize, _found_keys: usize, _candidate_keys: usize) {}

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

    fn hardnested_result(&self, _label: &str, _key: Option<&MfClassicKey>) {}

    fn hardnested_end(&self) {}
}

#[cfg(test)]
pub struct NullReporter;

#[cfg(test)]
impl Reporter for NullReporter {
    fn begin_progress(&self, _total_nonces: usize) {}

    fn update_progress(
        &self,
        _nonce_current: usize,
        _nonce_total: usize,
        _msb_current: usize,
        _msb_total: usize,
        _stage_progress: f32,
        _uid: u32,
    ) {
    }

    fn found_key(&self, _key: &MfClassicKey) {}
}
