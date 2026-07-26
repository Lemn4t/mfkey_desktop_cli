use crate::core::ffi::{AttackType, crypto1_prng_successor};
use crate::core::model::Nonce;
use crate::ext::result::Rslt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn binary_string_to_int(bin_str: &str) -> u8 {
    let mut result: u8 = 0;
    for c in bin_str.chars() {
        result <<= 1;
        if c == '1' {
            result |= 1;
        }
    }
    result
}

fn token_after<'a>(tokens: &'a [&'a str], key: &str) -> Option<&'a str> {
    tokens
        .iter()
        .position(|&t| t == key)
        .and_then(|i| tokens.get(i + 1).copied())
}

fn parse_hex_u32(s: &str) -> Option<u32> {
    u32::from_str_radix(s.trim(), 16).ok()
}

fn parse_mfkey32_line(tokens: &[&str]) -> Option<Nonce> {
    let uid = token_after(tokens, "cuid")
        .or_else(|| token_after(tokens, "uid"))
        .and_then(parse_hex_u32)?;
    let nt0 = token_after(tokens, "nt0").and_then(parse_hex_u32)?;
    let nr0_enc = token_after(tokens, "nr0").and_then(parse_hex_u32)?;
    let ar0_enc = token_after(tokens, "ar0").and_then(parse_hex_u32)?;
    let nt1 = token_after(tokens, "nt1").and_then(parse_hex_u32)?;
    let nr1_enc = token_after(tokens, "nr1").and_then(parse_hex_u32)?;
    let ar1_enc = token_after(tokens, "ar1").and_then(parse_hex_u32)?;

    let p64 = unsafe { crypto1_prng_successor(nt0, 64) };
    let p64b = unsafe { crypto1_prng_successor(nt1, 64) };

    Some(Nonce {
        attack: AttackType::Mfkey32,
        uid,
        nt0,
        nt1,
        uid_xor_nt0: uid ^ nt0,
        uid_xor_nt1: uid ^ nt1,
        nr0_enc,
        ar0_enc,
        nr1_enc,
        ar1_enc,
        p64,
        p64b,
        ..Default::default()
    })
}

fn parse_nested_line(tokens: &[&str]) -> Option<Nonce> {
    let sector_num: i64 = token_after(tokens, "Sec").and_then(|s| s.parse::<i64>().ok())?;
    let key_type: &str = token_after(tokens, "key")?;
    let key_b = key_type.eq_ignore_ascii_case("B");
    let key_idx = (sector_num * 2 + if key_b { 1 } else { 0 }) as u8;

    let uid = token_after(tokens, "cuid").and_then(parse_hex_u32)?;
    let nt0 = token_after(tokens, "nt0").and_then(parse_hex_u32)?;
    let ks1_1_enc = token_after(tokens, "ks0").and_then(parse_hex_u32)?;
    let par_1 = token_after(tokens, "par0").map(binary_string_to_int)?;

    let nt1 = token_after(tokens, "nt1").and_then(parse_hex_u32);
    let ks1 = token_after(tokens, "ks1").and_then(parse_hex_u32);
    let par1 = token_after(tokens, "par1").map(binary_string_to_int);

    let mut nonce = Nonce {
        attack: AttackType::StaticEncrypted,
        key_idx,
        uid,
        nt0,
        nt1: 0,
        uid_xor_nt0: uid ^ nt0,
        uid_xor_nt1: 0,
        ks1_1_enc,
        ks1_2_enc: 0,
        par_1,
        par_2: 0,
        ..Default::default()
    };

    if let (Some(nt1v), Some(ks1v), Some(par1v)) = (nt1, ks1, par1) {
        nonce.attack = AttackType::StaticNested;
        nonce.nt1 = nt1v;
        nonce.ks1_2_enc = ks1v;
        nonce.par_2 = par1v;
        nonce.uid_xor_nt1 = uid ^ nt1v;
    }

    Some(nonce)
}

fn is_hardnested_line(trimmed: &str, tokens: &[&str]) -> bool {
    tokens.contains(&"Sec")
        && tokens.contains(&"key")
        && tokens.contains(&"cuid")
        && tokens.contains(&"nt0")
        && tokens.contains(&"ks0")
        && tokens.contains(&"par0")
        && !trimmed.contains("dist")
}

pub fn load_nested_nonces<P, F>(path: P, mut on_loaded: F) -> Rslt<(Vec<Nonce>, bool)>
where
    P: AsRef<Path>,
    F: FnMut(usize, u32, &str),
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut nonces: Vec<Nonce> = Vec::new();
    let mut hardnested_detected = false;

    for line_res in reader.lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();

        let is_mfkey32 = tokens.contains(&"nr0")
            && tokens.contains(&"ar0")
            && tokens.contains(&"nr1")
            && tokens.contains(&"ar1");

        if !is_mfkey32 && is_hardnested_line(trimmed, &tokens) {
            hardnested_detected = true;
            continue;
        }

        let parsed = if is_mfkey32 {
            parse_mfkey32_line(&tokens)
        } else if trimmed.contains("dist 0") {
            parse_nested_line(&tokens)
        } else {
            None
        };

        if let Some(nonce) = parsed {
            nonces.push(nonce);
            let idx = nonces.len();
            let last = nonces.last().unwrap();
            on_loaded(idx, last.uid, last.attack_name());
        }
    }

    Ok((nonces, hardnested_detected))
}

#[cfg(test)]
mod tests {
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
    fn load_nested_nonces_mixes_static_and_hardnested_and_reports_both() {
        let content = "\
Sec 0 key A cuid 7a962390 nt0 aabbccdd nr0 11223344 ar0 55667788 nt1 aaaa1111 nr1 bbbb2222 ar1 cccc3333
Sec 7 key A cuid 7a962390 nt0 00000000 ks0 5b615df5 par0 0110
Sec 7 key A cuid 7a962390 nt0 00000000 ks0 fb193dbb par0 0100
Sec 0 key A cuid 7a962390 nt0 12345678 ks0 abcdef01 par0 0000 dist 0
";
        let path =
            std::env::temp_dir().join(format!("mfkey_parser_test_{}.log", std::process::id()));
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(content.as_bytes()).unwrap();
        }

        let mut loaded = Vec::new();
        let (nonces, hardnested_detected) = load_nested_nonces(&path, |idx, uid, name| {
            loaded.push((idx, uid, name.to_string()))
        })
        .unwrap();

        let _ = std::fs::remove_file(&path);

        assert_eq!(nonces.len(), 2);
        assert!(hardnested_detected);
        assert_eq!(loaded.len(), 2);
        assert_eq!(nonces[0].attack, AttackType::Mfkey32);
        assert_eq!(nonces[1].attack, AttackType::StaticEncrypted);
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

        let (nonces, hardnested_detected) = load_nested_nonces(&path, |_, _, _| {}).unwrap();

        let _ = std::fs::remove_file(&path);

        assert_eq!(nonces.len(), 1);
        assert!(!hardnested_detected);
    }
}
