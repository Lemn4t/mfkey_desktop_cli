use crate::core::model::MfClassicKey;
use crate::ui::Ui;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

fn accumulate<T: Eq + Hash + Copy>(items: &mut Vec<T>, seen: &mut HashSet<T>, item: T) -> bool {
    if seen.insert(item) {
        items.push(item);
        true
    } else {
        false
    }
}

pub struct AttackContext {
    pub stop: Arc<AtomicBool>,
    pub processed: AtomicUsize,
    pub total_nonces: usize,
    pub ui: Arc<Ui>,
    pub found_seen: Mutex<HashSet<MfClassicKey>>,
}

impl AttackContext {
    pub fn new(ui: Arc<Ui>, stop: Arc<AtomicBool>, total_nonces: usize) -> Self {
        AttackContext {
            stop,
            processed: AtomicUsize::new(0),
            total_nonces,
            ui,
            found_seen: Mutex::new(HashSet::new()),
        }
    }

    #[inline]
    pub fn should_stop(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    pub fn register_found(&self, key: MfClassicKey) -> bool {
        let mut set = self.found_seen.lock().unwrap();
        set.insert(key)
    }
}

pub struct TaskState<'ctx> {
    pub found_keys: Vec<MfClassicKey>,
    found_set: HashSet<MfClassicKey>,

    pub candidate_keys: Vec<(u8, MfClassicKey)>,
    candidate_set: HashSet<(u8, MfClassicKey)>,

    pub ctx: &'ctx AttackContext,
    pub total_nonces: usize,
}

impl<'ctx> TaskState<'ctx> {
    pub fn new(ctx: &'ctx AttackContext) -> Self {
        TaskState {
            found_keys: Vec::new(),
            found_set: HashSet::new(),
            candidate_keys: Vec::new(),
            candidate_set: HashSet::new(),
            total_nonces: ctx.total_nonces,
            ctx,
        }
    }

    pub fn add_found_key(&mut self, key: MfClassicKey) {
        accumulate(&mut self.found_keys, &mut self.found_set, key);
    }

    pub fn add_candidate_key(&mut self, key_idx: u8, key: MfClassicKey) {
        accumulate(
            &mut self.candidate_keys,
            &mut self.candidate_set,
            (key_idx, key),
        );
    }

    #[inline]
    pub fn should_stop(&self) -> bool {
        self.ctx.should_stop()
    }
}

pub struct AttackState {
    pub found_keys: Vec<MfClassicKey>,
    found_set: HashSet<MfClassicKey>,

    pub candidate_keys: Vec<(u8, MfClassicKey)>,
    candidate_set: HashSet<(u8, MfClassicKey)>,

    pub stop: Arc<AtomicBool>,
    pub ui: Arc<Ui>,
}

impl AttackState {
    pub fn new(ui: Arc<Ui>, stop: Arc<AtomicBool>) -> Self {
        AttackState {
            found_keys: Vec::new(),
            found_set: HashSet::new(),
            candidate_keys: Vec::new(),
            candidate_set: HashSet::new(),
            stop,
            ui,
        }
    }

    pub fn add_candidate_key(&mut self, key_idx: u8, key: MfClassicKey) {
        accumulate(
            &mut self.candidate_keys,
            &mut self.candidate_set,
            (key_idx, key),
        );
    }

    pub fn clear_candidates(&mut self) {
        self.candidate_keys.clear();
        self.candidate_set.clear();
    }

    pub fn merge_found(&mut self, keys: &[MfClassicKey]) {
        for &k in keys {
            accumulate(&mut self.found_keys, &mut self.found_set, k);
        }
    }

    pub fn merge_candidates(&mut self, cands: &[(u8, MfClassicKey)]) {
        for &(idx, k) in cands {
            self.add_candidate_key(idx, k);
        }
    }
}
