use super::*;
use std::mem::{align_of, offset_of, size_of};

unsafe extern "C" {
    fn crapto1_cnonce_size() -> usize;
    fn crapto1_cnonce_align() -> usize;
    fn crapto1_cnonce_offset(field: i32) -> usize;
    fn crapto1_attack_value(which: i32) -> i32;
}

#[test]
fn cnonce_size_and_align_match_c() {
    assert_eq!(size_of::<CNonce>(), unsafe { crapto1_cnonce_size() });
    assert_eq!(align_of::<CNonce>(), unsafe { crapto1_cnonce_align() });
}

#[test]
fn cnonce_field_offsets_match_c() {
    let fields: [(usize, i32); 17] = [
        (offset_of!(CNonce, attack), 0),
        (offset_of!(CNonce, key_idx), 1),
        (offset_of!(CNonce, uid), 2),
        (offset_of!(CNonce, nt0), 3),
        (offset_of!(CNonce, nt1), 4),
        (offset_of!(CNonce, uid_xor_nt0), 5),
        (offset_of!(CNonce, uid_xor_nt1), 6),
        (offset_of!(CNonce, p64), 7),
        (offset_of!(CNonce, p64b), 8),
        (offset_of!(CNonce, nr0_enc), 9),
        (offset_of!(CNonce, ar0_enc), 10),
        (offset_of!(CNonce, nr1_enc), 11),
        (offset_of!(CNonce, ar1_enc), 12),
        (offset_of!(CNonce, ks1_1_enc), 13),
        (offset_of!(CNonce, ks1_2_enc), 14),
        (offset_of!(CNonce, par_1), 15),
        (offset_of!(CNonce, par_2), 16),
    ];

    for (rust_offset, field) in fields {
        assert_eq!(
            rust_offset,
            unsafe { crapto1_cnonce_offset(field) },
            "offset mismatch for field index {field}"
        );
    }
}

#[test]
fn attack_type_values_match_c() {
    assert_eq!(AttackType::Mfkey32 as i32, unsafe {
        crapto1_attack_value(0)
    });
    assert_eq!(AttackType::StaticNested as i32, unsafe {
        crapto1_attack_value(1)
    });
    assert_eq!(AttackType::StaticEncrypted as i32, unsafe {
        crapto1_attack_value(2)
    });
}
