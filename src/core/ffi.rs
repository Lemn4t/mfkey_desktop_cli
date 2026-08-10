use std::os::raw::{c_float, c_int, c_void};

pub const MF_CLASSIC_KEY_SIZE: usize = 6;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackType {
    #[default]
    Mfkey32 = 0,
    StaticNested = 1,
    StaticEncrypted = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CNonce {
    pub attack: i32,
    pub key_idx: u8,
    pub uid: u32,
    pub nt0: u32,
    pub nt1: u32,
    pub uid_xor_nt0: u32,
    pub uid_xor_nt1: u32,

    pub p64: u32,
    pub p64b: u32,
    pub nr0_enc: u32,
    pub ar0_enc: u32,
    pub nr1_enc: u32,
    pub ar1_enc: u32,

    pub ks1_1_enc: u32,
    pub ks1_2_enc: u32,
    pub par_1: u8,
    pub par_2: u8,
}

impl Default for CNonce {
    fn default() -> Self {
        CNonce {
            attack: AttackType::StaticEncrypted as i32,
            key_idx: 0,
            uid: 0,
            nt0: 0,
            nt1: 0,
            uid_xor_nt0: 0,
            uid_xor_nt1: 0,
            p64: 0,
            p64b: 0,
            nr0_enc: 0,
            ar0_enc: 0,
            nr1_enc: 0,
            ar1_enc: 0,
            ks1_1_enc: 0,
            ks1_2_enc: 0,
            par_1: 0,
            par_2: 0,
        }
    }
}

#[repr(C)]
pub struct CCallbacks {
    pub found_key: Option<extern "C" fn(key6: *const u8, user: *mut c_void)>,
    pub candidate_key: Option<extern "C" fn(key6: *const u8, key_idx: u8, user: *mut c_void)>,
    pub progress: Option<
        extern "C" fn(
            msb_round: u32,
            total_rounds: u32,
            stage_progress: c_float,
            uid: u32,
            user: *mut c_void,
        ),
    >,
    pub should_stop: Option<extern "C" fn(user: *mut c_void) -> c_int>,
    pub user: *mut c_void,
}

unsafe extern "C" {
    pub fn crapto1_recover(n: *const CNonce, ks2: u32, in_: u32, cb: *const CCallbacks) -> bool;
    pub fn prng_successor(x: u32, n: u32) -> u32;
}

#[cfg(test)]
#[path = "../tests/ffi_contract.rs"]
mod tests;
