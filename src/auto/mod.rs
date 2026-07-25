mod upload;

use crate::core::disclaimer::resolve_disclaimer_acceptance;
use crate::core::engine;
use crate::core::model::MfClassicKey;
use crate::core::parser;
use crate::core::state::AttackState;
use crate::flipper::{FlipperSession, find};
use crate::ui::{Ui, UiOptions};

use colored::Color;
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const NFC_DIR: &str = "/ext/nfc";

fn is_target_log(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".mfkey32.log") || lower.ends_with(".nested.log")
}

pub fn run_auto(
    port_override: Option<&str>,
    out_dir: Option<&Path>,
    plain_ui: bool,
    auto_accept_disclaimer: bool,
) -> Result<(), String> {
    let ui_opts = UiOptions { plain_ui };
    let ui = Arc::new(Ui::new(ui_opts));
    if !resolve_disclaimer_acceptance(&ui_opts, auto_accept_disclaimer) {
        return Ok(());
    }
    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop_clone = Arc::clone(&stop);
        let _ = ctrlc::set_handler(move || {
            eprintln!("\n\nReceived interrupt signal. Stopping gracefully...");
            stop_clone.store(true, Ordering::SeqCst);
        });
    }

    let base: PathBuf = match out_dir {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().map_err(|e| format!("cwd error: {e}"))?,
    };
    let logs_dir = base.join("mfkey_auto_data");
    fs::create_dir_all(&logs_dir).map_err(|e| format!("cannot create {logs_dir:?}: {e}"))?;

    let port = match port_override {
        Some(p) => p.to_string(),
        None => {
            ui.show_status("→ Searching for Flipper Zero...", Color::Cyan);
            find::find_flipper_port().map_err(|e| {
                format!(
                    "{e}\n  Hint: specify the port manually: --auto --port <PORT>\n  \
                     (Linux: /dev/ttyACM0, macOS: /dev/cu.usbmodemflip_*, Windows: COM3)"
                )
            })?
        }
    };
    ui.show_status_value_bold("✓ Flipper port:", &port, Color::Green);

    ui.show_status("→ Opening RPC session...", Color::Cyan);
    let mut sess =
        FlipperSession::open(&port).map_err(|e| format!("cannot open RPC session: {e}"))?;
    ui.show_status("✓ RPC session is up (ping OK)", Color::Green);

    ui.show_status_detail("→ Listing", NFC_DIR, Color::Cyan, false);
    let entries = sess
        .storage_list(NFC_DIR)
        .map_err(|e| format!("cannot list {NFC_DIR}: {e}"))?;

    let targets: Vec<String> = entries
        .into_iter()
        .filter(|e| !e.is_dir && is_target_log(&e.name))
        .map(|e| e.name)
        .collect();

    if targets.is_empty() {
        ui.show_status(
            "No logs (*.mfkey32.log / *.nested.log) found in /ext/nfc. Nothing to attack.",
            Color::Yellow,
        );
        return Ok(());
    }
    ui.show_status_detail(
        "✓ Found",
        &format!("{} file(s): {}", targets.len(), targets.join(", ")),
        Color::Green,
        false,
    );

    let mut local_logs: Vec<PathBuf> = Vec::new();
    let mut remote_logs: Vec<String> = Vec::new();
    for name in &targets {
        let remote = format!("{NFC_DIR}/{name}");
        ui.show_status_detail("↓ Downloading", &remote, Color::Cyan, false);

        let data = sess
            .storage_read(&remote)
            .map_err(|e| format!("read {remote}: {e}"))?;

        let local = logs_dir.join(name);
        fs::write(&local, &data).map_err(|e| format!("write {local:?}: {e}"))?;
        ui.show_detail_dimmed(
            "saved",
            &format!("{} ({} bytes)", local.display(), data.len()),
        );
        local_logs.push(local);
        remote_logs.push(remote);
    }

    ui.show_status("→ Running attack...", Color::Cyan);

    let mut all_keys: BTreeSet<String> = BTreeSet::new();
    let mut local_dicts: Vec<PathBuf> = Vec::new();

    for log in &local_logs {
        if stop.load(Ordering::SeqCst) {
            break;
        }

        let log_str = log.to_string_lossy().to_string();
        ui.show_loading(&log_str);

        let ui_for_load = Arc::clone(&ui);
        let (nonces, hardnested_detected) =
            match parser::load_nested_nonces(&log_str, |idx, uid, name| {
                ui_for_load.show_nonce_loaded(idx, uid, name);
            }) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Failed to parse {log_str}: {e}");
                    continue;
                }
            };

        if nonces.is_empty() {
            if hardnested_detected {
                ui.show_hardnested_unsupported(Some(&log_str), true);
            } else {
                eprintln!("No nonces loaded from {log_str}, skipping.");
            }
            continue;
        }

        ui.show_loading_complete(nonces.len());

        if hardnested_detected {
            ui.show_hardnested_note(Some(&log_str));
        }

        ui.show_start();

        let mut attack_state = AttackState::new(Arc::clone(&ui), Arc::clone(&stop), nonces.len());

        let dict_dir = logs_dir.to_string_lossy().to_string();
        let mut save_dict = |uid: u32, keys: &[(u8, MfClassicKey)], dir: Option<&str>| -> String {
            save_candidate_dict(uid, keys, dir, &mut local_dicts)
        };

        let (candidate_total_count, _dict_outputs) = engine::run_attack(
            &mut attack_state,
            &nonces,
            Some(dict_dir.as_str()),
            &mut save_dict,
        );

        let found = attack_state.found_keys.len();
        ui.show_summary(nonces.len(), found, candidate_total_count);

        for k in &attack_state.found_keys {
            all_keys.insert(k.to_hex().to_uppercase());
        }
    }

    upload::upload_dicts(&ui, &mut sess, &local_dicts);
    upload::merge_and_upload_keys(&ui, &mut sess, &all_keys, &logs_dir)?;

    if !all_keys.is_empty() {
        let should_delete = ui.confirm("Delete the original log files from the Flipper?", false);
        if should_delete {
            for remote in &remote_logs {
                ui.show_status_detail("✗ Deleting from device", remote, Color::Cyan, false);
                if let Err(e) = sess.storage_delete(remote, false) {
                    eprintln!("warning: failed to delete {remote}: {e}");
                }
            }
        }
    }

    Ok(())
}

fn save_candidate_dict(
    uid: u32,
    keys: &[(u8, MfClassicKey)],
    output_dir: Option<&str>,
    sink: &mut Vec<PathBuf>,
) -> String {
    let filename = format!("mf_classic_dict_{:08x}.nfc", uid);
    let path = match output_dir {
        Some(dir) => Path::new(dir).join(&filename),
        None => Path::new(&filename).to_path_buf(),
    };
    let path_str = path.to_string_lossy().to_string();

    if let Ok(mut file) = fs::File::create(&path) {
        for (key_idx, k) in keys {
            let _ = writeln!(file, "{:02X}{}", key_idx, k.to_hex());
        }
        sink.push(path.clone());
    } else {
        eprintln!("Failed to create dictionary file: {}", path_str);
    }

    path_str
}
