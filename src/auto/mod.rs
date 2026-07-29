mod upload;

use crate::core::attack_runner::{self, FileAttackOutcome};
use crate::ext::result::{ResultExt, Rslt};
use crate::flipper::{FlipperSession, find};
use crate::ui::Ui;

use crate::params::AutoParams;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const NFC_DIR: &str = "/ext/nfc";

fn is_target_log(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".mfkey32.log") || lower.ends_with(".nested.log")
}

pub fn run_auto(ui: Arc<Ui>, params: AutoParams, stop: Arc<AtomicBool>) -> Rslt<()> {
    let base: PathBuf = match params.out_dir.as_deref() {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().context("cwd error")?,
    };
    let logs_dir = base.join("mfkey_auto_data");
    fs::create_dir_all(&logs_dir).with_context(|| format!("cannot create {logs_dir:?}"))?;

    let port = match params.port.as_deref() {
        Some(p) => p.to_string(),
        None => {
            ui.show_searching_for_flipper();
            find::find_flipper_port().map_err(|e| {
                format!(
                    "{e}\n  Hint: specify the port manually: --auto --port <PORT>\n  \
                     (Linux: /dev/ttyACM0, macOS: /dev/cu.usbmodemflip_*, Windows: COM3)"
                )
            })?
        }
    };
    ui.show_flipper_port(&port);

    ui.show_opening_session();
    let mut sess = FlipperSession::open(&port).context("cannot open RPC session")?;
    ui.show_session_ready();

    ui.show_listing_dir(NFC_DIR);
    let entries = sess
        .storage_list(NFC_DIR)
        .with_context(|| format!("cannot list {NFC_DIR}"))?;

    let targets: Vec<String> = entries
        .into_iter()
        .filter(|e| !e.is_dir && is_target_log(&e.name))
        .map(|e| e.name)
        .collect();

    if targets.is_empty() {
        ui.show_no_logs_found(NFC_DIR);
        return Ok(());
    }
    ui.show_logs_found(&targets);

    let mut local_logs: Vec<PathBuf> = Vec::new();
    let mut remote_logs: Vec<String> = Vec::new();
    for name in &targets {
        let remote = format!("{NFC_DIR}/{name}");
        ui.show_downloading(&remote);

        let data = sess
            .storage_read(&remote)
            .with_context(|| format!("read {remote}"))?;

        let local = logs_dir.join(name);
        fs::write(&local, &data).with_context(|| format!("write {local:?}"))?;
        ui.show_saved_local_log(&local, data.len());
        local_logs.push(local);
        remote_logs.push(remote);
    }

    ui.show_running_attack();

    let mut all_keys: BTreeSet<String> = BTreeSet::new();
    let mut local_dicts: Vec<PathBuf> = Vec::new();

    for log in &local_logs {
        if stop.load(Ordering::SeqCst) {
            break;
        }

        let log_str = log.to_string_lossy().to_string();
        let dict_dir = logs_dir.to_string_lossy().to_string();

        let outcome = match attack_runner::run_file_attack(
            &ui,
            &stop,
            &log_str,
            Some(dict_dir.as_str()),
            Some(&log_str),
            true,
        ) {
            Ok(o) => o,
            Err(e) => {
                ui.show_error(&format!("Failed to parse {log_str}: {e}"));
                continue;
            }
        };

        let result = match outcome {
            FileAttackOutcome::NoUsableNonces {
                hardnested_detected,
            } => {
                if !hardnested_detected {
                    ui.show_error(&format!("No nonces loaded from {log_str}, skipping."));
                }
                continue;
            }
            FileAttackOutcome::Ran(r) => r,
        };

        for k in &result.found_keys {
            all_keys.insert(k.to_hex().to_uppercase());
        }
        for d in &result.dict_outputs {
            local_dicts.push(PathBuf::from(&d.path));
        }
    }

    upload::upload_dicts(&ui, &mut sess, &local_dicts);
    upload::merge_and_upload_keys(&ui, &mut sess, &all_keys, &logs_dir)?;

    if !all_keys.is_empty() {
        let should_delete = ui.confirm("Delete the original log files from the Flipper?", false);
        if should_delete {
            for remote in &remote_logs {
                ui.show_deleting_from_device(remote);
                if let Err(e) = sess.storage_delete(remote, false) {
                    ui.show_error(&format!("warning: failed to delete {remote}: {e}"));
                }
            }
        }
    }

    Ok(())
}
