use super::*;

use crate::core::reporter::{NullReporter, Reporter};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn example(name: &str) -> String {
    format!("{}/../../examples/{}", env!("CARGO_MANIFEST_DIR"), name)
}

fn temp_dict_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "mfkey_e2e_{}_{}_{:p}",
        tag,
        std::process::id(),
        &tag as *const _
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_fixture(fixture: &str, dict_dir: &Path) -> FileAttackOutcome {
    let reporter: Arc<dyn Reporter> = Arc::new(NullReporter);
    let stop = Arc::new(AtomicBool::new(false));
    run_file_attack(
        &reporter,
        &stop,
        &example(fixture),
        Some(&dict_dir.to_string_lossy()),
    )
    .expect("run_file_attack should succeed on a readable fixture")
}

fn found_hex(result: &AttackOutcome) -> Vec<String> {
    result.found_keys.iter().map(|k| k.to_hex()).collect()
}

#[test]
fn mfkey32_recovers_single_known_key() {
    let dir = temp_dict_dir("mfkey32");
    let outcome = run_fixture("mfkey32.log", &dir);

    match outcome {
        FileAttackOutcome::Ran(r) => {
            assert_eq!(found_hex(&r), vec!["A0A1A2A3A4A5"]);
            assert_eq!(r.candidate_total_count, 0);
            assert!(r.dict_outputs.is_empty());
        }
        FileAttackOutcome::NoUsableNonces => panic!("expected Ran outcome"),
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn static_nested_recovers_single_known_key() {
    let dir = temp_dict_dir("static_nested");
    let outcome = run_fixture("static_nested.log", &dir);

    match outcome {
        FileAttackOutcome::Ran(r) => {
            assert_eq!(found_hex(&r), vec!["112244556600"]);
            assert_eq!(r.candidate_total_count, 0);
            assert!(r.dict_outputs.is_empty());
        }
        FileAttackOutcome::NoUsableNonces => panic!("expected Ran outcome"),
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn static_encrypted_produces_frozen_candidate_dictionary() {
    let dir = temp_dict_dir("static_encrypted");
    let outcome = run_fixture("static_encrypted.log", &dir);

    match outcome {
        FileAttackOutcome::Ran(r) => {
            assert!(
                r.found_keys.is_empty(),
                "static_encrypted yields candidates, not found keys"
            );
            assert_eq!(r.candidate_total_count, 1_043_338);
            assert_eq!(r.dict_outputs.len(), 1);

            let dict = &r.dict_outputs[0];
            assert_eq!(dict.uid, 0x929d_09de);
            assert_eq!(dict.count, 1_043_338);

            let filename = Path::new(&dict.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            assert_eq!(filename, "mf_classic_dict_929d09de.nfc");

            let bytes = fs::read(&dict.path).expect("dictionary file must exist");
            let newlines = bytes.iter().filter(|&&b| b == b'\n').count();
            assert_eq!(newlines, 1_043_338, "line count must be stable");
            assert!(
                !bytes.windows(2).any(|w| w == b"\r\n"),
                "dictionary must use LF line endings"
            );
            assert_eq!(
                fnv1a64(&bytes),
                0x608a_85a8_56c0_1d50,
                "dictionary content (bytes + order) must be identical to baseline"
            );
        }
        FileAttackOutcome::NoUsableNonces => panic!("expected Ran outcome"),
    }

    let _ = fs::remove_dir_all(&dir);
}

