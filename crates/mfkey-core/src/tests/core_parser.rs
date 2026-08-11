use super::*;
use std::io::Write;

fn tokens_of(line: &str) -> Vec<&str> {
    line.split_whitespace().collect()
}

#[test]
fn binary_string_to_int_parses_msb_first() {
    assert_eq!(binary_string_to_int("0000"), 0);
    assert_eq!(binary_string_to_int("1111"), 15);
    assert_eq!(binary_string_to_int("0110"), 6);
    assert_eq!(binary_string_to_int("1000"), 8);
}

#[test]
fn token_after_finds_the_following_token() {
    let tokens = tokens_of("Sec 7 key A cuid 7a962390 nt0 00000000");
    assert_eq!(token_after(&tokens, "cuid"), Some("7a962390"));
    assert_eq!(token_after(&tokens, "nt0"), Some("00000000"));
}

#[test]
fn token_after_returns_none_when_key_missing_or_last() {
    let tokens = tokens_of("Sec 7 key A cuid");
    assert_eq!(token_after(&tokens, "nt0"), None);
    assert_eq!(token_after(&tokens, "cuid"), None);
}

#[test]
fn parse_hex_u32_accepts_valid_hex_and_rejects_garbage() {
    assert_eq!(parse_hex_u32("7a962390"), Some(0x7a962390));
    assert_eq!(parse_hex_u32("  1A2b "), Some(0x1a2b));
    assert_eq!(parse_hex_u32("not_hex"), None);
}

#[test]
fn is_hardnested_line_matches_the_real_capture_format() {
    let line = "Sec 7 key A cuid 7a962390 nt0 00000000 ks0 5b615df5 par0 0110";
    assert!(is_hardnested_line(line, &tokens_of(line)));
}

#[test]
fn is_hardnested_line_rejects_static_lines_with_dist() {
    let line = "Sec 0 key A cuid 7a962390 nt0 12345678 ks0 abcdef01 par0 0000 dist 0";
    assert!(!is_hardnested_line(line, &tokens_of(line)));
}

#[test]
fn is_hardnested_line_rejects_incomplete_lines() {
    let line = "Sec 7 key A cuid 7a962390 nt0 00000000";
    assert!(!is_hardnested_line(line, &tokens_of(line)));
}

#[test]
fn parse_nested_line_without_second_nonce_is_static_encrypted() {
    let line = "Sec 0 key A cuid 7a962390 nt0 12345678 ks0 abcdef01 par0 0000 dist 0";
    let nonce = parse_nested_line(&tokens_of(line)).expect("should parse");
    assert_eq!(nonce.attack, AttackType::StaticEncrypted);
    assert_eq!(nonce.uid, 0x7a962390);
    assert_eq!(nonce.key_idx, 0);
}

#[test]
fn parse_nested_line_with_second_nonce_is_static_nested() {
    let line = "Sec 1 key B cuid 7a962390 nt0 12345678 ks0 abcdef01 par0 0000 \
                 nt1 87654321 ks1 10fedcba par1 1111 dist 0";
    let nonce = parse_nested_line(&tokens_of(line)).expect("should parse");
    assert_eq!(nonce.attack, AttackType::StaticNested);
    assert_eq!(nonce.key_idx, 3);
}

#[test]
fn parse_nested_line_missing_required_field_returns_none() {
    let line = "Sec 0 key A cuid 7a962390 nt0 12345678 dist 0";
    assert!(parse_nested_line(&tokens_of(line)).is_none());
}

#[test]
fn load_nested_nonces_accepts_weak_nested_with_nonzero_dist() {
    let content = "Sec 5 key A cuid 7c30d979 nt0 214904f0 ks0 c03823d1 par0 0000 \
                    nt1 f69baa3a ks1 8a2107ad par1 1010 dist 12\n";
    let path = std::env::temp_dir().join(format!(
        "mfkey_parser_weak_{}.log",
        std::process::id()
    ));
    {
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    let set = load_nested_nonces(&path, |_, _, _| {}).unwrap();

    let _ = std::fs::remove_file(&path);

    assert_eq!(set.nonces.len(), 1);
    assert_eq!(set.nonces[0].attack, AttackType::StaticNested);
    assert_eq!(set.nonces[0].key_idx, 10);
    assert!(set.hardnested.is_empty());
}

#[test]
fn load_nested_nonces_counts_unrecognized_lines() {
    let content = "this is not a nonce line\n\
                    Sec 0 key A cuid 7a962390 nt0 12345678 ks0 abcdef01 par0 0000 dist 0\n\
                    garbage garbage garbage\n";
    let path = std::env::temp_dir().join(format!(
        "mfkey_parser_unrecognized_{}.log",
        std::process::id()
    ));
    {
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    let set = load_nested_nonces(&path, |_, _, _| {}).unwrap();

    let _ = std::fs::remove_file(&path);

    assert_eq!(set.nonces.len(), 1);
    assert_eq!(set.unrecognized, 2);
}

#[test]
fn parse_mfkey32_line_extracts_all_fields() {
    let line = "Sec 0 key A cuid 7a962390 nt0 aabbccdd nr0 11223344 ar0 55667788 nt1 aaaa1111 nr1 bbbb2222 ar1 cccc3333";
    let nonce = parse_mfkey32_line(&tokens_of(line)).expect("should parse");
    assert_eq!(nonce.attack, AttackType::Mfkey32);
    assert_eq!(nonce.uid, 0x7a962390);
    assert_eq!(nonce.nt0, 0xaabbccdd);
    assert_eq!(nonce.nr0_enc, 0x11223344);
    assert_eq!(nonce.ar0_enc, 0x55667788);
    assert_eq!(nonce.nt1, 0xaaaa1111);
    assert_eq!(nonce.nr1_enc, 0xbbbb2222);
    assert_eq!(nonce.ar1_enc, 0xcccc3333);
    assert_eq!(nonce.uid_xor_nt0, 0x7a962390 ^ 0xaabbccdd);
    assert_eq!(nonce.uid_xor_nt1, 0x7a962390 ^ 0xaaaa1111);
}

#[test]
fn parse_mfkey32_line_accepts_uid_alias() {
    let line = "uid 7a962390 nt0 aabbccdd nr0 11223344 ar0 55667788 nt1 aaaa1111 nr1 bbbb2222 ar1 cccc3333";
    let nonce = parse_mfkey32_line(&tokens_of(line)).expect("should parse");
    assert_eq!(nonce.uid, 0x7a962390);
}

#[test]
fn parse_mfkey32_line_missing_field_returns_none() {
    let line = "cuid 7a962390 nt0 aabbccdd nr0 11223344 ar0 55667788";
    assert!(parse_mfkey32_line(&tokens_of(line)).is_none());
}

#[test]
fn load_nested_nonces_mixes_static_and_hardnested_and_reports_both() {
    let content = "\
Sec 0 key A cuid 7a962390 nt0 aabbccdd nr0 11223344 ar0 55667788 nt1 aaaa1111 nr1 bbbb2222 ar1 cccc3333
Sec 7 key A cuid 7a962390 nt0 00000000 ks0 5b615df5 par0 0110
Sec 7 key A cuid 7a962390 nt0 00000000 ks0 fb193dbb par0 0100
Sec 0 key A cuid 7a962390 nt0 12345678 ks0 abcdef01 par0 0000 dist 0
";
    let path = std::env::temp_dir().join(format!("mfkey_parser_test_{}.log", std::process::id()));
    {
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    let mut loaded = Vec::new();
    let set = load_nested_nonces(&path, |idx, uid, name| {
        loaded.push((idx, uid, name.to_string()))
    })
    .unwrap();

    let _ = std::fs::remove_file(&path);

    assert_eq!(set.nonces.len(), 2);
    assert!(!set.hardnested.is_empty());
    assert_eq!(loaded.len(), 2);
    assert_eq!(set.nonces[0].attack, AttackType::Mfkey32);
    assert_eq!(set.nonces[1].attack, AttackType::StaticEncrypted);

    assert_eq!(set.hardnested.len(), 2);
    assert_eq!(set.hardnested[0].uid, 0x7a962390);
    assert_eq!(set.hardnested[0].nt0, 0x00000000);
    assert_eq!(set.hardnested[0].ks0, 0x5b615df5);
    assert_eq!(set.hardnested[0].par0, 0b0110);
}

#[test]
fn load_nested_nonces_reports_no_hardnested_when_none_present() {
    let content = "Sec 0 key A cuid 7a962390 nt0 12345678 ks0 abcdef01 par0 0000 dist 0\n";
    let path = std::env::temp_dir().join(format!(
        "mfkey_parser_test_clean_{}.log",
        std::process::id()
    ));
    {
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    let set = load_nested_nonces(&path, |_, _, _| {}).unwrap();

    let _ = std::fs::remove_file(&path);

    assert_eq!(set.nonces.len(), 1);
    assert!(set.hardnested.is_empty());
}
