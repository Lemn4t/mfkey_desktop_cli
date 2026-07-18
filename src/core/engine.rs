use crate::core::ffi::{AttackType, CCallbacks, CNonce, MF_CLASSIC_KEY_SIZE, crypto1_recover};
use crate::core::model::{MfClassicKey, Nonce};
use crate::core::state::{AttackContext, AttackState, TaskState};
use rayon::prelude::*;
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
        state.ctx.ui.show_found_key(&key);
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
    state.ctx.ui.update_progress(
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

fn process_one(ctx: &AttackContext, nonce: &Nonce, ks2: u32, in_: u32, uid: u32) -> TaskResult {
    if ctx.should_stop() {
        return TaskResult {
            found: Vec::new(),
            candidates: Vec::new(),
        };
    }

    let mut ts = TaskState::new(ctx);
    ts.current_uid = uid;

    let c_nonce: CNonce = nonce.to_c();
    let cb = make_callbacks(&mut ts);
    unsafe {
        crypto1_recover(
            &c_nonce as *const CNonce,
            ks2,
            in_,
            &cb as *const CCallbacks,
        );
    }

    ctx.processed.fetch_add(1, Ordering::Relaxed);

    TaskResult {
        found: ts.found_keys,
        candidates: ts.candidate_keys,
    }
}

pub type SaveDictFn<'a> = dyn FnMut(u32, &[(u8, MfClassicKey)], Option<&str>) -> String + 'a;

pub fn run_attack(
    state: &mut AttackState,
    nonces: &[Nonce],
    dict_output_dir: Option<&str>,
    save_dict: &mut SaveDictFn,
) -> (usize, Vec<DictOutput>) {
    let ctx = AttackContext::new(Arc::clone(&state.ui), Arc::clone(&state.stop), nonces.len());

    state.ui.begin_progress(nonces.len());

    {
        let results: Vec<TaskResult> = nonces
            .par_iter()
            .filter(|n| n.attack == AttackType::Mfkey32)
            .map(|nonce| {
                let ks2 = nonce.ar0_enc ^ nonce.p64;
                process_one(&ctx, nonce, ks2, 0, nonce.uid)
            })
            .collect();

        for r in &results {
            state.merge_found(&r.found);
        }
    }

    {
        let results: Vec<TaskResult> = nonces
            .par_iter()
            .filter(|n| n.attack == AttackType::StaticNested)
            .map(|nonce| {
                let ks_enc = nonce.ks1_2_enc;
                let nt_xor_uid = nonce.uid_xor_nt1;
                process_one(&ctx, nonce, ks_enc, nt_xor_uid, nonce.uid)
            })
            .collect();

        for r in &results {
            state.merge_found(&r.found);
        }
    }

    let mut unique_uids: Vec<u32> = Vec::new();
    for n in nonces.iter() {
        if n.attack == AttackType::StaticEncrypted && !unique_uids.contains(&n.uid) {
            unique_uids.push(n.uid);
        }
    }

    let mut dict_outputs: Vec<DictOutput> = Vec::new();
    let mut candidate_total_count: usize = 0;

    for &uid in unique_uids.iter() {
        if ctx.should_stop() {
            break;
        }

        let group: Vec<&Nonce> = nonces
            .iter()
            .filter(|n| n.attack == AttackType::StaticEncrypted && n.uid == uid)
            .collect();
        if group.is_empty() {
            continue;
        }

        let results: Vec<TaskResult> = group
            .par_iter()
            .map(|nonce| {
                let ks_enc = nonce.ks1_1_enc;
                let nt_xor_uid = nonce.uid_xor_nt0;
                process_one(&ctx, nonce, ks_enc, nt_xor_uid, uid)
            })
            .collect();

        state.clear_candidates();
        for r in &results {
            state.merge_candidates(&r.candidates);
            state.merge_found(&r.found);
        }

        if !state.candidate_keys.is_empty() {
            let count = state.candidate_keys.len();
            let path = save_dict(uid, &state.candidate_keys, dict_output_dir);
            dict_outputs.push(DictOutput { uid, count, path });
            candidate_total_count += count;
        }
        state.clear_candidates();
    }

    (candidate_total_count, dict_outputs)
}
