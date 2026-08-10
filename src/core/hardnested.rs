use crate::core::model::{HardNestedNonce, MfClassicKey};
use std::collections::BTreeMap;
use std::os::raw::{c_int, c_void};

#[repr(C)]
struct HnNonce {
    nt_enc: u32,
    par: u8,
}

unsafe extern "C" {
    fn hardnested_recover(
        uid: u32,
        key_type: u8,
        nonces: *const HnNonce,
        count: u32,
        cb: *const c_void,
        out_key: *mut u64,
    ) -> c_int;
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

#[allow(dead_code)]
pub struct HardNestedSolver;

#[allow(dead_code)]
impl HardNestedSolver {
    pub fn run(nonces: &[HardNestedNonce]) -> Vec<MfClassicKey> {
        let mut groups: BTreeMap<(u32, u8), Vec<HnNonce>> = BTreeMap::new();
        for n in nonces {
            let key_type = n.key_idx & 1;
            groups.entry((n.uid, key_type)).or_default().push(HnNonce {
                nt_enc: n.nt0 ^ n.ks0,
                par: n.par0,
            });
        }

        let mut found: Vec<MfClassicKey> = Vec::new();
        for ((uid, key_type), group) in groups {
            let mut out_key: u64 = 0;
            let res = unsafe {
                hardnested_recover(
                    uid,
                    key_type,
                    group.as_ptr(),
                    group.len() as u32,
                    std::ptr::null(),
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
