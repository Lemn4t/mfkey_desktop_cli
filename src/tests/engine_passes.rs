use super::*;

fn nonce(attack: AttackType, uid: u32) -> Nonce {
    Nonce {
        attack,
        uid,
        ..Default::default()
    }
}

#[test]
fn groups_static_encrypted_in_first_seen_uid_order() {
    let nonces = vec![
        nonce(AttackType::StaticEncrypted, 0xAA),
        nonce(AttackType::Mfkey32, 0xFF),
        nonce(AttackType::StaticEncrypted, 0xBB),
        nonce(AttackType::StaticEncrypted, 0xAA),
    ];

    let groups = group_static_encrypted_by_uid(&nonces);

    let uids: Vec<u32> = groups.iter().map(|(uid, _)| *uid).collect();
    assert_eq!(uids, vec![0xAA, 0xBB], "UID order must be first-seen");
    assert_eq!(groups[0].1.len(), 2, "both 0xAA nonces grouped together");
    assert_eq!(groups[1].1.len(), 1);
}

#[test]
fn ignores_nonces_that_are_not_static_encrypted() {
    let nonces = vec![
        nonce(AttackType::Mfkey32, 1),
        nonce(AttackType::StaticNested, 2),
    ];

    assert!(group_static_encrypted_by_uid(&nonces).is_empty());
}

#[test]
fn derive_ks_formulas_match_baseline() {
    let n = Nonce {
        ar0_enc: 0x1111_1111,
        p64: 0x2222_2222,
        ks1_2_enc: 0x3333_3333,
        uid_xor_nt1: 0x4444_4444,
        ks1_1_enc: 0x5555_5555,
        uid_xor_nt0: 0x6666_6666,
        ..Default::default()
    };

    assert_eq!(derive_ks_mfkey32(&n), (0x1111_1111 ^ 0x2222_2222, 0));
    assert_eq!(derive_ks_static_nested(&n), (0x3333_3333, 0x4444_4444));
    assert_eq!(derive_ks_static_encrypted(&n), (0x5555_5555, 0x6666_6666));
}
