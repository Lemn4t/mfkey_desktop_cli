use crate::core::engine::{self, DictOutput};
use crate::core::model::MfClassicKey;
use crate::core::parser;
use crate::core::state::AttackState;
use crate::ext::result::Rslt;
use crate::ui::Ui;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct FileAttackResult {
    pub found_keys: Vec<MfClassicKey>,
    pub candidate_total_count: usize,
    pub dict_outputs: Vec<DictOutput>,
}

pub enum FileAttackOutcome {
    NoUsableNonces { hardnested_detected: bool },
    Ran(FileAttackResult),
}

pub fn run_file_attack(
    ui: &Arc<Ui>,
    stop: &Arc<AtomicBool>,
    file_path: &str,
    dict_output_dir: Option<&str>,
    hardnested_context: Option<&str>,
    hardnested_skipping: bool,
) -> Rslt<FileAttackOutcome> {
    ui.show_loading(file_path);

    let ui_for_load = Arc::clone(ui);
    let (nonces, hardnested_detected) = parser::load_nested_nonces(file_path, |idx, uid, name| {
        ui_for_load.show_nonce_loaded(idx, uid, name);
    })?;

    if nonces.is_empty() {
        if hardnested_detected {
            ui.show_hardnested_unsupported(hardnested_context, hardnested_skipping);
        }
        return Ok(FileAttackOutcome::NoUsableNonces {
            hardnested_detected,
        });
    }

    ui.show_loading_complete(nonces.len());
    if hardnested_detected {
        ui.show_hardnested_note(hardnested_context);
    }
    ui.show_start();

    let mut attack_state = AttackState::new(Arc::clone(ui), Arc::clone(stop));

    let mut save_dict =
        |uid: u32, keys: &[(u8, MfClassicKey)], dir: Option<&str>| -> Option<String> {
            let path = candidate_dict_path(uid, dir);
            let path_str = path.to_string_lossy().to_string();
            match write_candidate_dict(&path, keys) {
                Ok(()) => Some(path_str),
                Err(_) => {
                    ui.show_error(&format!("Failed to create dictionary file: {}", path_str));
                    None
                }
            }
        };

    let (candidate_total_count, dict_outputs) =
        engine::run_attack(&mut attack_state, &nonces, dict_output_dir, &mut save_dict);

    let nonce_count = nonces.len();
    let found_count = attack_state.found_keys.len();
    ui.show_summary(nonce_count, found_count, candidate_total_count);

    Ok(FileAttackOutcome::Ran(FileAttackResult {
        found_keys: attack_state.found_keys.into_vec(),
        candidate_total_count,
        dict_outputs,
    }))
}

fn candidate_dict_path(uid: u32, output_dir: Option<&str>) -> PathBuf {
    let filename = format!("mf_classic_dict_{:08x}.nfc", uid);
    match output_dir {
        Some(dir) => Path::new(dir).join(&filename),
        None => Path::new(&filename).to_path_buf(),
    }
}

fn write_candidate_dict(path: &Path, keys: &[(u8, MfClassicKey)]) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    for (key_idx, k) in keys {
        let _ = writeln!(file, "{:02X}{}", key_idx, k.to_hex());
    }
    Ok(())
}
