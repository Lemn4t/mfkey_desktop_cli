mod upload;

use crate::core::attack_runner::{self, FileAttackOutcome};
use crate::ext::result::{ResultExt, Rslt};
use crate::flipper::{FlipperSession, find};
use crate::ui::Ui;

use crate::params::AutoParams;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const NFC_DIR: &str = "/ext/nfc";

fn is_target_log(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".mfkey32.log") || lower.ends_with(".nested.log")
}

fn resolve_logs_dir(params: &AutoParams) -> Rslt<PathBuf> {
    let base: PathBuf = match params.out_dir.as_deref() {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().context("cwd error")?,
    };
    let logs_dir = base.join("mfkey_auto_data");
    fs::create_dir_all(&logs_dir).with_context(|| format!("cannot create {logs_dir:?}"))?;
    Ok(logs_dir)
}

fn discover_port(ui: &Ui, params: &AutoParams) -> Rslt<String> {
    if let Some(p) = params.port.as_deref() {
        ui.show_flipper_port(p);
        return Ok(p.to_string());
    }

    ui.show_searching_for_flipper();
    let flippers = find::find_all_flippers().context("cannot enumerate serial ports")?;

    let port = match flippers.len() {
        0 => {
            return Err(
                "Flipper Zero not found.\n  Hint: specify the port manually: --auto --port <PORT>\n  \
                 (Linux: /dev/ttyACM0, macOS: /dev/cu.usbmodemflip_*, Windows: COM3)"
                    .into(),
            );
        }
        1 => flippers.into_iter().next().unwrap().port,
        _ => {
            let labels: Vec<String> = flippers.iter().map(|f| f.label.clone()).collect();
            match ui.select_index("Multiple Flipper Zero devices found — select one:", &labels) {
                Some(idx) => flippers.into_iter().nth(idx).unwrap().port,
                None => {
                    let list = labels
                        .iter()
                        .map(|l| format!("  - {l}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    return Err(format!(
                        "Multiple Flipper Zero devices found; specify one with --auto --port <PORT>:\n{list}"
                    )
                    .into());
                }
            }
        }
    };

    ui.show_flipper_port(&port);
    Ok(port)
}

fn download_target_logs(
    ui: &Ui,
    sess: &mut FlipperSession,
    logs_dir: &Path,
) -> Rslt<(Vec<PathBuf>, Vec<String>)> {
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
        return Ok((Vec::new(), Vec::new()));
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

    Ok((local_logs, remote_logs))
}

fn run_attacks_over_logs(
    ui: &Arc<Ui>,
    stop: &Arc<AtomicBool>,
    local_logs: &[PathBuf],
    logs_dir: &Path,
) -> (BTreeSet<String>, Vec<PathBuf>) {
    let mut all_keys: BTreeSet<String> = BTreeSet::new();
    let mut local_dicts: Vec<PathBuf> = Vec::new();

    for log in local_logs {
        if stop.load(Ordering::SeqCst) {
            break;
        }

        let log_str = log.to_string_lossy().to_string();
        let dict_dir = logs_dir.to_string_lossy().to_string();

        let outcome = match attack_runner::run_file_attack(
            ui,
            stop,
            &log_str,
            Some(dict_dir.as_str()),
        ) {
            Ok(o) => o,
            Err(e) => {
                ui.show_error(&format!("Failed to parse {log_str}: {e}"));
                continue;
            }
        };

        let result = match outcome {
            FileAttackOutcome::NoUsableNonces => {
                ui.show_error(&format!("No nonces loaded from {log_str}, skipping."));
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

    (all_keys, local_dicts)
}

fn maybe_delete_remote_logs(ui: &Ui, sess: &mut FlipperSession, remote_logs: &[String]) {
    let should_delete = ui.confirm("Delete the original log files from the Flipper?", false);
    if should_delete {
        for remote in remote_logs {
            ui.show_deleting_from_device(remote);
            if let Err(e) = sess.storage_delete(remote, false) {
                ui.show_error(&format!("warning: failed to delete {remote}: {e}"));
            }
        }
    }
}

pub fn run_auto(ui: Arc<Ui>, params: AutoParams, stop: Arc<AtomicBool>) -> Rslt<()> {
    let logs_dir = resolve_logs_dir(&params)?;
    let port = discover_port(&ui, &params)?;

    ui.show_opening_session();
    let mut sess = FlipperSession::open(&port).context("cannot open RPC session")?;
    ui.show_session_ready();

    let (local_logs, remote_logs) = download_target_logs(&ui, &mut sess, &logs_dir)?;
    if local_logs.is_empty() {
        return Ok(());
    }

    ui.show_running_attack();
    let (all_keys, local_dicts) = run_attacks_over_logs(&ui, &stop, &local_logs, &logs_dir);

    upload::upload_dicts(&ui, &mut sess, &local_dicts);
    upload::merge_and_upload_keys(&ui, &mut sess, &all_keys, &logs_dir)?;

    if !all_keys.is_empty() {
        maybe_delete_remote_logs(&ui, &mut sess, &remote_logs);
    }

    Ok(())
}

#[cfg(test)]
#[path = "../tests/auto_regression.rs"]
mod tests;
