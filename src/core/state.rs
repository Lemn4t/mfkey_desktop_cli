use crate::core::model::MfClassicKey;
use crate::core::reporter::Reporter;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct DedupVec<T> {
    items: Vec<T>,
    seen: HashSet<T>,
}

impl<T: Eq + Hash + Copy> DedupVec<T> {
    pub fn new() -> Self {
        DedupVec {
            items: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn push(&mut self, item: T) -> bool {
        if self.seen.insert(item) {
            self.items.push(item);
            true
        } else {
            false
        }
    }

    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.seen.clear();
    }

    pub fn into_vec(self) -> Vec<T> {
        self.items
    }
}

impl<T: Eq + Hash + Copy> Default for DedupVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for DedupVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.items
    }
}

pub struct AttackContext {
    pub stop: Arc<AtomicBool>,
    pub processed: AtomicUsize,
    pub total_nonces: usize,
    pub reporter: Arc<dyn Reporter>,
    pub found_seen: Mutex<HashSet<MfClassicKey>>,
}

impl AttackContext {
    pub fn new(reporter: Arc<dyn Reporter>, stop: Arc<AtomicBool>, total_nonces: usize) -> Self {
        AttackContext {
            stop,
            processed: AtomicUsize::new(0),
            total_nonces,
            reporter,
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
    pub found_keys: DedupVec<MfClassicKey>,
    pub candidate_keys: DedupVec<(u8, MfClassicKey)>,

    pub ctx: &'ctx AttackContext,
    pub total_nonces: usize,
}

impl<'ctx> TaskState<'ctx> {
    pub fn new(ctx: &'ctx AttackContext) -> Self {
        TaskState {
            found_keys: DedupVec::new(),
            candidate_keys: DedupVec::new(),
            total_nonces: ctx.total_nonces,
            ctx,
        }
    }

    pub fn add_found_key(&mut self, key: MfClassicKey) {
        self.found_keys.push(key);
    }

    pub fn add_candidate_key(&mut self, key_idx: u8, key: MfClassicKey) {
        self.candidate_keys.push((key_idx, key));
    }

    #[inline]
    pub fn should_stop(&self) -> bool {
        self.ctx.should_stop()
    }
}

pub struct AttackState {
    pub found_keys: DedupVec<MfClassicKey>,
    pub candidate_keys: DedupVec<(u8, MfClassicKey)>,

    pub stop: Arc<AtomicBool>,
    pub reporter: Arc<dyn Reporter>,
}

impl AttackState {
    pub fn new(reporter: Arc<dyn Reporter>, stop: Arc<AtomicBool>) -> Self {
        AttackState {
            found_keys: DedupVec::new(),
            candidate_keys: DedupVec::new(),
            stop,
            reporter,
        }
    }

    pub fn clear_candidates(&mut self) {
        self.candidate_keys.clear();
    }

    pub fn merge_found(&mut self, keys: &[MfClassicKey]) {
        self.found_keys.extend(keys.iter().copied());
    }

    pub fn merge_candidates(&mut self, cands: &[(u8, MfClassicKey)]) {
        self.candidate_keys.extend(cands.iter().copied());
    }
}

#[cfg(test)]
#[path = "../tests/core_state.rs"]
mod tests;
