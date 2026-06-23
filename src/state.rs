use crate::model::MfClassicKey;
use crate::ui::Ui;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct AttackState {
    pub found_keys: Vec<MfClassicKey>,
    found_set: HashSet<MfClassicKey>,

    pub candidate_keys: Vec<(u8, MfClassicKey)>,
    candidate_set: HashSet<(u8, MfClassicKey)>,
    pub stop: Arc<AtomicBool>,

    pub global_current_nonce: usize,
    pub global_total_nonces: usize,

    pub ui: Arc<Ui>,
}

impl AttackState {
    pub fn new(ui: Arc<Ui>, stop: Arc<AtomicBool>, total_nonces: usize) -> Self {
        AttackState {
            found_keys: Vec::new(),
            found_set: HashSet::new(),
            candidate_keys: Vec::new(),
            candidate_set: HashSet::new(),
            stop,
            global_current_nonce: 0,
            global_total_nonces: total_nonces,
            ui,
        }
    }

    pub fn add_found_key(&mut self, key: MfClassicKey) -> bool {
        if self.found_set.insert(key) {
            self.found_keys.push(key);
            true
        } else {
            false
        }
    }

    pub fn add_candidate_key(&mut self, key_idx: u8, key: MfClassicKey) -> bool {
        if self.candidate_set.insert((key_idx, key)) {
            self.candidate_keys.push((key_idx, key));
            true
        } else {
            false
        }
    }

    pub fn clear_candidates(&mut self) {
        self.candidate_keys.clear();
        self.candidate_set.clear();
    }

    pub fn should_stop(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }
}
