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
