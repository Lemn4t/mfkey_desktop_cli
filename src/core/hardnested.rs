use crate::core::model::{HardNestedNonce, MfClassicKey};
use crate::core::reporter::Reporter;
use std::collections::BTreeMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

#[repr(C)]
struct HnNonce {
    nt_enc: u32,
    par: u8,
}

#[repr(C)]
struct HnCallbacks {
    line: Option<extern "C" fn(*const c_char, *mut c_void)>,
    user: *mut c_void,
}

unsafe extern "C" {
    fn hardnested_recover(
        uid: u32,
        key_type: u8,
        nonces: *const HnNonce,
        count: u32,
        cb: *const HnCallbacks,
        out_key: *mut u64,
    ) -> c_int;
}

extern "C" fn line_trampoline(text: *const c_char, user: *mut c_void) {
    if text.is_null() || user.is_null() {
        return;
    }
    let reporter = unsafe { *(user as *const &dyn Reporter) };
    let line = unsafe { CStr::from_ptr(text) }.to_string_lossy();
    reporter.hardnested_line(&line);
}

fn key_from_u64(key: u64) -> MfClassicKey {
    let bytes = [
        (key >> 40) as u8,
        (key >> 32) as u8,
        (key >> 24) as u8,
        (key >> 16) as u8,
        (key >> 8) as u8,
        key as u8,
    ];
    MfClassicKey::from_slice(&bytes)
}

pub struct HardNestedSolver;

impl HardNestedSolver {
    pub fn run(
        nonces: &[HardNestedNonce],
        stop: &AtomicBool,
        reporter: &dyn Reporter,
    ) -> Vec<MfClassicKey> {
        let mut groups: BTreeMap<(u32, u8), Vec<HnNonce>> = BTreeMap::new();
        for n in nonces {
            let key_type = n.key_idx & 1;
            groups.entry((n.uid, key_type)).or_default().push(HnNonce {
                nt_enc: n.nt0 ^ n.ks0,
                par: n.par0,
            });
        }

        let reporter_ref: &dyn Reporter = reporter;
        let user = &reporter_ref as *const &dyn Reporter as *mut c_void;
        let cb = HnCallbacks {
            line: Some(line_trampoline),
            user,
        };

        let mut found: Vec<MfClassicKey> = Vec::new();
        for ((uid, key_type), group) in groups {
            if stop.load(Ordering::SeqCst) {
                break;
            }

            let mut out_key: u64 = 0;
            let res = unsafe {
                hardnested_recover(
                    uid,
                    key_type,
                    group.as_ptr(),
                    group.len() as u32,
                    &cb as *const HnCallbacks,
                    &mut out_key as *mut u64,
                )
            };
            if res == 1 {
                found.push(key_from_u64(out_key));
            }
        }
        found
    }
}

#[cfg(test)]
#[path = "../tests/hardnested.rs"]
mod tests;
