use super::*;
use crate::core::reporter::NullReporter;
use std::mem::{align_of, offset_of, size_of};
use std::sync::atomic::AtomicBool;

unsafe extern "C" {
    fn hn_nonce_size() -> usize;
    fn hn_nonce_align() -> usize;
    fn hn_nonce_offset(field: i32) -> usize;
    fn hn_callbacks_size() -> usize;
    fn hn_callbacks_align() -> usize;
    fn hn_callbacks_offset(field: i32) -> usize;
}

#[test]
fn hn_nonce_layout_matches_c() {
    assert_eq!(size_of::<HnNonce>(), unsafe { hn_nonce_size() });
    assert_eq!(align_of::<HnNonce>(), unsafe { hn_nonce_align() });
    assert_eq!(offset_of!(HnNonce, nt_enc), unsafe { hn_nonce_offset(0) });
    assert_eq!(offset_of!(HnNonce, par), unsafe { hn_nonce_offset(1) });
}

#[test]
fn hn_callbacks_layout_matches_c() {
    assert_eq!(size_of::<HnCallbacks>(), unsafe { hn_callbacks_size() });
    assert_eq!(align_of::<HnCallbacks>(), unsafe { hn_callbacks_align() });
    assert_eq!(offset_of!(HnCallbacks, line), unsafe {
        hn_callbacks_offset(0)
    });
    assert_eq!(offset_of!(HnCallbacks, user), unsafe {
        hn_callbacks_offset(1)
    });
}

#[test]
#[ignore]
fn recovers_known_key_from_example_log() {
    let path = format!(
        "{}/../../examples/hard_nested.log",
        env!("CARGO_MANIFEST_DIR")
    );
    let set = crate::core::parser::load_nested_nonces(&path, |_, _, _| {}).unwrap();

    assert_eq!(set.hardnested.len(), 1740);

    let stop = AtomicBool::new(false);

    let keys = HardNestedSolver::run(&set.hardnested, &stop, &NullReporter);
    let hex: Vec<String> = keys.iter().map(|k| k.to_hex()).collect();

    assert_eq!(hex, vec!["759275927592".to_string()]);
}
