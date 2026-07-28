mod upload;

use crate::core::attack_runner::{self, FileAttackOutcome};
use crate::ext::result::{ResultExt, Rslt};
use crate::flipper::{FlipperSession, find};
use crate::ui::{MessageKind, Ui};

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
            ui.show_status("→ Searching for Flipper Zero...", MessageKind::Info);
            find::find_flipper_port().map_err(|e| {
                format!(
                    "{e}\n  Hint: specify the port manually: --auto --port <PORT>\n  \
                     (Linux: /dev/ttyACM0, macOS: /dev/cu.usbmodemflip_*, Windows: COM3)"
                )
            })?
        }
    };
    ui.show_status_value_bold("✓ Flipper port:", &port, MessageKind::Success);

    ui.show_status("→ Opening RPC session...", MessageKind::Info);
    let mut sess = FlipperSession::open(&port).context("cannot open RPC session")?;
    ui.show_status("✓ RPC session is up (ping OK)", MessageKind::Success);

    ui.show_status_detail("→ Listing", NFC_DIR, MessageKind::Info, false);
    let entries = sess
        .storage_list(NFC_DIR)
        .with_context(|| format!("cannot list {NFC_DIR}"))?;

    let targets: Vec<String> = entries
        .into_iter()
        .filter(|e| !e.is_dir && is_target_log(&e.name))
        .map(|e| e.name)
        .collect();

    if targets.is_empty() {
        ui.show_status(
            "No logs (*.mfkey32.log / *.nested.log) found in /ext/nfc. Nothing to attack.",
            MessageKind::Warning,
        );
        return Ok(());
    }
    ui.show_status_detail(
        "✓ Found",
        &format!("{} file(s): {}", targets.len(), targets.join(", ")),
        MessageKind::Success,
        false,
    );

    let mut local_logs: Vec<PathBuf> = Vec::new();
    let mut remote_logs: Vec<String> = Vec::new();
    for name in &targets {
        let remote = format!("{NFC_DIR}/{name}");
        ui.show_status_detail("↓ Downloading", &remote, MessageKind::Info, false);

        let data = sess
            .storage_read(&remote)
            .with_context(|| format!("read {remote}"))?;

        let local = logs_dir.join(name);
        fs::write(&local, &data).with_context(|| format!("write {local:?}"))?;
        ui.show_detail_dimmed(
            "saved",
            &format!("{} ({} bytes)", local.display(), data.len()),
        );
        local_logs.push(local);
        remote_logs.push(remote);
    }

    ui.show_status("→ Running attack...", MessageKind::Info);

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
                ui.show_status_detail("✗ Deleting from device", remote, MessageKind::Info, false);
                if let Err(e) = sess.storage_delete(remote, false) {
                    ui.show_error(&format!("warning: failed to delete {remote}: {e}"));
                }
            }
        }
    }

    Ok(())
}
