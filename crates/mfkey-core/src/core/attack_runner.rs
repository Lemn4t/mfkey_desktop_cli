use crate::core::hardnested::HardNestedSolver;
use crate::core::model::MfClassicKey;
use crate::core::outcome::AttackOutcome;
use crate::core::parser;
use crate::core::reporter::Reporter;
use crate::core::solver::Crapto1Solver;
use crate::ext::result::Rslt;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub enum FileAttackOutcome {
    NoUsableNonces,
    Ran(AttackOutcome),
}

pub fn run_file_attack(
    reporter: &Arc<dyn Reporter>,
    stop: &Arc<AtomicBool>,
    file_path: &str,
    dict_output_dir: Option<&str>,
) -> Rslt<FileAttackOutcome> {
    reporter.loading(file_path);

    let reporter_for_load = Arc::clone(reporter);
    let nonce_set = parser::load_nested_nonces(file_path, |idx, uid, name| {
        reporter_for_load.nonce_loaded(idx, uid, name);
    })?;
    let nonces = nonce_set.nonces;
    let hardnested = nonce_set.hardnested;

    if nonce_set.unrecognized > 0 {
        reporter.unrecognized_lines(nonce_set.unrecognized);
    }

    if nonces.is_empty() && hardnested.is_empty() {
        return Ok(FileAttackOutcome::NoUsableNonces);
    }

    if !hardnested.is_empty() {
        let mut targets: Vec<(u32, u8)> = Vec::new();
        for n in &hardnested {
            let target = (n.uid, n.key_idx);
            if !targets.contains(&target) {
                targets.push(target);
            }
        }
        reporter.hardnested_loaded(hardnested.len(), &targets);
    }

    let total = nonces.len() + hardnested.len();
    reporter.loading_complete(total);
    reporter.attack_start();

    let mut outcome = if nonces.is_empty() {
        AttackOutcome {
            found_keys: Vec::new(),
            candidate_total_count: 0,
            dict_outputs: Vec::new(),
        }
    } else {
        let mut save_dict =
            |uid: u32, keys: &[(u8, MfClassicKey)], dir: Option<&str>| -> Option<String> {
                let path = candidate_dict_path(uid, dir);
                let path_str = path.to_string_lossy().to_string();
                match write_candidate_dict(&path, keys) {
                    Ok(()) => Some(path_str),
                    Err(_) => {
                        reporter.error(&format!("Failed to create dictionary file: {}", path_str));
                        None
                    }
                }
            };

        Crapto1Solver::run(
            Arc::clone(reporter),
            Arc::clone(stop),
            &nonces,
            dict_output_dir,
            &mut save_dict,
        )
    };

    if !hardnested.is_empty() {
        let hardnested_keys = HardNestedSolver::run(&hardnested, stop, reporter.as_ref());
        for key in hardnested_keys {
            if !outcome.found_keys.contains(&key) {
                outcome.found_keys.push(key);
            }
        }
    }

    reporter.summary(total, outcome.found_keys.len(), outcome.candidate_total_count);

    Ok(FileAttackOutcome::Ran(outcome))
}

fn candidate_dict_path(uid: u32, output_dir: Option<&str>) -> PathBuf {
    let filename = format!("mf_classic_dict_{:08x}.nfc", uid);
    match output_dir {
        Some(dir) => Path::new(dir).join(&filename),
        None => Path::new(&filename).to_path_buf(),
    }
}

fn write_candidate_dict(path: &Path, keys: &[(u8, MfClassicKey)]) -> std::io::Result<()> {
    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    for (key_idx, k) in keys {
        writeln!(writer, "{:02X}{}", key_idx, k.to_hex())?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
#[path = "../tests/e2e_attack.rs"]
mod e2e_tests;
