use crate::ffi::{AttackType, CCallbacks, CNonce, MF_CLASSIC_KEY_SIZE, crypto1_recover};
use crate::model::{MfClassicKey, Nonce};
use crate::state::AttackState;
use std::os::raw::{c_float, c_int, c_void};
use std::slice;

extern "C" fn cb_found_key(key6: *const u8, user: *mut c_void) {
    if key6.is_null() || user.is_null() {
        return;
    }
    let state = unsafe { &mut *(user as *mut AttackState) };
    let slice = unsafe { slice::from_raw_parts(key6, MF_CLASSIC_KEY_SIZE) };
    let key = MfClassicKey::from_slice(slice);
    if state.add_found_key(key) {
        state.ui.show_found_key(&key);
    }
}

extern "C" fn cb_candidate_key(key6: *const u8, key_idx: u8, user: *mut c_void) {
    if key6.is_null() || user.is_null() {
        return;
    }
    let state = unsafe { &mut *(user as *mut AttackState) };
    let slice = unsafe { slice::from_raw_parts(key6, MF_CLASSIC_KEY_SIZE) };
    let key = MfClassicKey::from_slice(slice);
    state.add_candidate_key(key_idx, key);
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
    let state = unsafe { &mut *(user as *mut AttackState) };
    state.ui.update_progress(
        state.global_current_nonce,
        state.global_total_nonces,
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
    let state = unsafe { &*(user as *const AttackState) };
    if state.should_stop() { 1 } else { 0 }
}

fn make_callbacks(state: &mut AttackState) -> CCallbacks {
    CCallbacks {
        found_key: Some(cb_found_key),
        candidate_key: Some(cb_candidate_key),
        progress: Some(cb_progress),
        should_stop: Some(cb_should_stop),
        user: state as *mut AttackState as *mut c_void,
    }
}

fn run_recover(state: &mut AttackState, nonce: &Nonce, ks2: u32, in_: u32) -> bool {
    let c_nonce: CNonce = nonce.to_c();
    let cb = make_callbacks(state);
    unsafe {
        crypto1_recover(
            &c_nonce as *const CNonce,
            ks2,
            in_,
            &cb as *const CCallbacks,
        )
    }
}

pub struct DictOutput {
    pub uid: u32,
    pub count: usize,
    pub path: String,
}

pub fn run_attack(
    state: &mut AttackState,
    nonces: &[Nonce],
    dict_output_dir: Option<&str>,
    save_dict: &mut dyn FnMut(u32, &[(u8, MfClassicKey)], Option<&str>) -> String,
) -> (usize, Vec<DictOutput>) {
    let total_nonces = nonces.len();
    state.global_total_nonces = total_nonces;

    let mut processed_total: usize = 0;

    for nonce in nonces.iter() {
        if state.should_stop() {
            break;
        }
        if nonce.attack != AttackType::Mfkey32 {
            continue;
        }

        processed_total += 1;
        state.global_current_nonce = processed_total;
        state.global_total_nonces = total_nonces;

        let ks2 = nonce.ar0_enc ^ nonce.p64;
        let in_ = 0u32;

        run_recover(state, nonce, ks2, in_);
    }

    for nonce in nonces.iter() {
        if state.should_stop() {
            break;
        }
        if nonce.attack != AttackType::StaticNested {
            continue;
        }

        processed_total += 1;
        state.global_current_nonce = processed_total;
        state.global_total_nonces = total_nonces;

        let ks_enc = nonce.ks1_2_enc;
        let nt_xor_uid = nonce.uid_xor_nt1;
        run_recover(state, nonce, ks_enc, nt_xor_uid);
    }

    let mut unique_uids: Vec<u32> = Vec::new();
    for nonce in nonces.iter() {
        if nonce.attack != AttackType::StaticEncrypted {
            continue;
        }
        if !unique_uids.contains(&nonce.uid) {
            unique_uids.push(nonce.uid);
        }
    }

    let mut dict_outputs: Vec<DictOutput> = Vec::new();
    let mut candidate_total_count: usize = 0;

    for &uid in unique_uids.iter() {
        if state.should_stop() {
            break;
        }

        let group_total = nonces
            .iter()
            .filter(|n| n.attack == AttackType::StaticEncrypted && n.uid == uid)
            .count();
        if group_total == 0 {
            continue;
        }

        state.clear_candidates();

        for nonce in nonces.iter() {
            if state.should_stop() {
                break;
            }
            if nonce.attack != AttackType::StaticEncrypted || nonce.uid != uid {
                continue;
            }

            processed_total += 1;
            state.global_current_nonce = processed_total;
            state.global_total_nonces = total_nonces;

            let ks_enc = nonce.ks1_1_enc;
            let nt_xor_uid = nonce.uid_xor_nt0;
            run_recover(state, nonce, ks_enc, nt_xor_uid);
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
