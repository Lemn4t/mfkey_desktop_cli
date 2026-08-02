use super::*;

#[test]
fn from_slice_copies_six_bytes() {
    let key = MfClassicKey::from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    assert_eq!(key.data, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
}

#[test]
fn from_slice_ignores_trailing_bytes() {
    let key = MfClassicKey::from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x99, 0x88]);
    assert_eq!(key.data, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
}

#[test]
fn to_hex_is_uppercase_zero_padded() {
    let key = MfClassicKey::from_slice(&[0x00, 0x0A, 0xFF, 0x10, 0x01, 0xB2]);
    assert_eq!(key.to_hex(), "000AFF1001B2");
}

#[test]
fn to_hex_all_zero_and_all_ff() {
    assert_eq!(MfClassicKey::from_slice(&[0; 6]).to_hex(), "000000000000");
    assert_eq!(
        MfClassicKey::from_slice(&[0xFF; 6]).to_hex(),
        "FFFFFFFFFFFF"
    );
}

#[test]
fn save_keys_to_file_writes_one_hex_per_line() {
    let keys = [
        MfClassicKey::from_slice(&[0x11; 6]),
        MfClassicKey::from_slice(&[0x22; 6]),
    ];
    let path = std::env::temp_dir().join(format!("mfkey_model_test_{}.txt", std::process::id()));
    let path_str = path.to_string_lossy().to_string();
    save_keys_to_file(&path_str, &keys).unwrap();
    let contents = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(contents, "111111111111\n222222222222\n");
}

#[test]
fn save_keys_to_file_skips_empty_without_creating_file() {
    let path = std::env::temp_dir().join(format!("mfkey_model_empty_{}.txt", std::process::id()));
    let path_str = path.to_string_lossy().to_string();
    save_keys_to_file(&path_str, &[]).unwrap();
    assert!(!path.exists());
}
