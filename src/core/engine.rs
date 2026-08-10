use crate::core::ffi::{AttackType, CCallbacks, CNonce, MF_CLASSIC_KEY_SIZE, crapto1_recover};
use crate::core::model::{MfClassicKey, Nonce};
use crate::core::state::{AttackContext, AttackState, TaskState};
use rayon::prelude::*;
use std::collections::HashMap;
use std::os::raw::{c_float, c_int, c_void};
use std::slice;
use std::sync::Arc;
use std::sync::atomic::Ordering;

extern "C" fn cb_found_key(key6: *const u8, user: *mut c_void) {
    if key6.is_null() || user.is_null() {
        return;
    }
    let state = unsafe { &mut *(user as *mut TaskState) };
    let s = unsafe { slice::from_raw_parts(key6, MF_CLASSIC_KEY_SIZE) };
    let key = MfClassicKey::from_slice(s);

    state.add_found_key(key);

    if state.ctx.register_found(key) {
        state.ctx.reporter.found_key(&key);
    }
}

extern "C" fn cb_candidate_key(key6: *const u8, key_idx: u8, user: *mut c_void) {
    if key6.is_null() || user.is_null() {
        return;
    }
    let state = unsafe { &mut *(user as *mut TaskState) };
    let s = unsafe { slice::from_raw_parts(key6, MF_CLASSIC_KEY_SIZE) };
    state.add_candidate_key(key_idx, MfClassicKey::from_slice(s));
}

extern "C" fn cb_progress(
    msb_round: u32,
    total_rounds: u32,
    stage_progress: c_float,
    uid: u32,
    user: *mut c_void,
) {
    if user.is_null() {
        return;
    }
    let state = unsafe { &*(user as *const TaskState) };
    let done = state.ctx.processed.load(Ordering::Relaxed);
    state.ctx.reporter.update_progress(
        done,
        state.total_nonces,
        msb_round as usize,
        total_rounds as usize,
        stage_progress,
        uid,
    );
}

extern "C" fn cb_should_stop(user: *mut c_void) -> c_int {
    if user.is_null() {
        return 0;
    }
    let state = unsafe { &*(user as *const TaskState) };
    if state.should_stop() { 1 } else { 0 }
}

fn make_callbacks(state: &mut TaskState) -> CCallbacks {
    CCallbacks {
        found_key: Some(cb_found_key),
        candidate_key: Some(cb_candidate_key),
        progress: Some(cb_progress),
        should_stop: Some(cb_should_stop),
        user: state as *mut TaskState as *mut c_void,
    }
}

pub struct DictOutput {
    pub uid: u32,
    pub count: usize,
    pub path: String,
}

struct TaskResult {
    found: Vec<MfClassicKey>,
    candidates: Vec<(u8, MfClassicKey)>,
}

fn process_one(ctx: &AttackContext, nonce: &Nonce, ks2: u32, in_: u32) -> TaskResult {
    if ctx.should_stop() {
        return TaskResult {
            found: Vec::new(),
            candidates: Vec::new(),
        };
    }

    let mut ts = TaskState::new(ctx);

    let c_nonce: CNonce = nonce.to_c();
    let cb = make_callbacks(&mut ts);
    unsafe {
        crapto1_recover(
            &c_nonce as *const CNonce,
            ks2,
            in_,
            &cb as *const CCallbacks,
        );
    }

    ctx.processed.fetch_add(1, Ordering::Relaxed);

    TaskResult {
        found: ts.found_keys.into_vec(),
        candidates: ts.candidate_keys.into_vec(),
    }
}

fn derive_ks_mfkey32(nonce: &Nonce) -> (u32, u32) {
    (nonce.ar0_enc ^ nonce.p64, 0)
}

fn derive_ks_static_nested(nonce: &Nonce) -> (u32, u32) {
    (nonce.ks1_2_enc, nonce.uid_xor_nt1)
}

fn derive_ks_static_encrypted(nonce: &Nonce) -> (u32, u32) {
    (nonce.ks1_1_enc, nonce.uid_xor_nt0)
}

#[derive(Clone, Copy)]
struct AttackPass {
    attack: AttackType,
    derive_ks: fn(&Nonce) -> (u32, u32),
}

const SIMPLE_PASSES: [AttackPass; 2] = [
    AttackPass {
        attack: AttackType::Mfkey32,
        derive_ks: derive_ks_mfkey32,
    },
    AttackPass {
        attack: AttackType::StaticNested,
        derive_ks: derive_ks_static_nested,
    },
];

fn run_pass(
    ctx: &AttackContext,
    state: &mut AttackState,
    nonces: &[Nonce],
    attack_type: AttackType,
    derive_ks: impl Fn(&Nonce) -> (u32, u32) + Sync,
) {
    let results: Vec<TaskResult> = nonces
        .par_iter()
        .filter(|n| n.attack == attack_type)
        .map(|nonce| {
            let (ks2, in_) = derive_ks(nonce);
            process_one(ctx, nonce, ks2, in_)
        })
        .collect();

    for r in &results {
        state.merge_found(&r.found);
    }
}

pub type SaveDictFn<'a> =
    dyn FnMut(u32, &[(u8, MfClassicKey)], Option<&str>) -> Option<String> + 'a;

fn group_static_encrypted_by_uid(nonces: &[Nonce]) -> Vec<(u32, Vec<&Nonce>)> {
    let mut order: Vec<u32> = Vec::new();
    let mut groups: HashMap<u32, Vec<&Nonce>> = HashMap::new();
    for n in nonces.iter() {
        if n.attack == AttackType::StaticEncrypted {
            groups
                .entry(n.uid)
                .or_insert_with(|| {
                    order.push(n.uid);
                    Vec::new()
                })
                .push(n);
        }
    }
    order
        .into_iter()
        .map(|uid| {
            let group = groups.remove(&uid).unwrap_or_default();
            (uid, group)
        })
        .collect()
}

fn process_uid_group(
    ctx: &AttackContext,
    state: &mut AttackState,
    uid: u32,
    group: &[&Nonce],
    dict_output_dir: Option<&str>,
    save_dict: &mut SaveDictFn,
) -> (usize, Option<DictOutput>) {
    let results: Vec<TaskResult> = group
        .par_iter()
        .map(|nonce| {
            let (ks2, in_) = derive_ks_static_encrypted(nonce);
            process_one(ctx, nonce, ks2, in_)
        })
        .collect();

    state.clear_candidates();
    for r in &results {
        state.merge_candidates(&r.candidates);
        state.merge_found(&r.found);
    }

    let mut count = 0;
    let mut output = None;
    if !state.candidate_keys.is_empty() {
        count = state.candidate_keys.len();
        if let Some(path) = save_dict(uid, &state.candidate_keys, dict_output_dir) {
            output = Some(DictOutput { uid, count, path });
        }
    }
    state.clear_candidates();

    (count, output)
}

pub fn run_attack(
    state: &mut AttackState,
    nonces: &[Nonce],
    dict_output_dir: Option<&str>,
    save_dict: &mut SaveDictFn,
) -> (usize, Vec<DictOutput>) {
    let ctx = AttackContext::new(
        Arc::clone(&state.reporter),
        Arc::clone(&state.stop),
        nonces.len(),
    );

    state.reporter.begin_progress(nonces.len());

    for pass in SIMPLE_PASSES {
        run_pass(&ctx, state, nonces, pass.attack, pass.derive_ks);
    }

    let mut dict_outputs: Vec<DictOutput> = Vec::new();
    let mut candidate_total_count: usize = 0;

    for (uid, group) in group_static_encrypted_by_uid(nonces) {
        if ctx.should_stop() {
            break;
        }

        let (count, output) =
            process_uid_group(&ctx, state, uid, &group, dict_output_dir, save_dict);
        candidate_total_count += count;
        if let Some(o) = output {
            dict_outputs.push(o);
        }
    }

    (candidate_total_count, dict_outputs)
}

#[cfg(test)]
#[path = "../tests/engine_passes.rs"]
mod tests;
