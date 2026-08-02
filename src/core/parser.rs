use crate::core::ffi::{AttackType, prng_successor};
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

    let p64 = unsafe { prng_successor(nt0, 64) };
    let p64b = unsafe { prng_successor(nt1, 64) };

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
            let uid = nonce.uid;
            let name = nonce.attack_name();
            nonces.push(nonce);
            on_loaded(nonces.len(), uid, name);
        }
    }

    Ok((nonces, hardnested_detected))
}

#[cfg(test)]
#[path = "../tests/core_parser.rs"]
mod tests;
