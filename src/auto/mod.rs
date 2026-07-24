mod upload;

use crate::core::disclaimer::resolve_disclaimer_acceptance;
use crate::core::engine;
use crate::core::model::MfClassicKey;
use crate::core::parser;
use crate::core::state::AttackState;
use crate::flipper::{FlipperSession, find};
use crate::ui::{Ui, UiOptions};

use colored::Colorize;
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
            println!("{}", "→ Searching for Flipper Zero...".cyan());
            find::find_flipper_port().map_err(|e| {
                format!(
                    "{e}\n  Hint: specify the port manually: --auto --port <PORT>\n  \
                     (Linux: /dev/ttyACM0, macOS: /dev/cu.usbmodemflip_*, Windows: COM3)"
                )
            })?
        }
    };
    println!("{} {}", "✓ Flipper port:".green(), port.bold());

    println!("{}", "→ Opening RPC session...".cyan());
    let mut sess =
        FlipperSession::open(&port).map_err(|e| format!("cannot open RPC session: {e}"))?;
    println!("{}", "✓ RPC session is up (ping OK)".green());

    println!("{} {}", "→ Listing".cyan(), NFC_DIR);
    let entries = sess
        .storage_list(NFC_DIR)
        .map_err(|e| format!("cannot list {NFC_DIR}: {e}"))?;

    let targets: Vec<String> = entries
        .into_iter()
        .filter(|e| !e.is_dir && is_target_log(&e.name))
        .map(|e| e.name)
        .collect();

    if targets.is_empty() {
        println!(
            "{}",
            "No logs (*.mfkey32.log / *.nested.log) found in /ext/nfc. Nothing to attack.".yellow()
        );
        return Ok(());
    }
    println!(
        "{} {} file(s): {}",
        "✓ Found".green(),
        targets.len(),
        targets.join(", ")
    );

    let mut local_logs: Vec<PathBuf> = Vec::new();
    for name in &targets {
        let remote = format!("{NFC_DIR}/{name}");
        println!("{} {}", "↓ Downloading".cyan(), remote);

        let data = sess
            .storage_read(&remote)
            .map_err(|e| format!("read {remote}: {e}"))?;

        let local = logs_dir.join(name);
        fs::write(&local, &data).map_err(|e| format!("write {local:?}: {e}"))?;
        println!(
            "  {} {} ({} bytes)",
            "saved".dimmed(),
            local.display(),
            data.len()
        );
        local_logs.push(local);

        println!("{} {}", "✗ Deleting from device".cyan(), remote);
        sess.storage_delete(&remote, false)
            .map_err(|e| format!("delete {remote}: {e}"))?;
    }

    println!("{}", "→ Running attack...".cyan());

    let mut all_keys: BTreeSet<String> = BTreeSet::new();
    let mut local_dicts: Vec<PathBuf> = Vec::new();

    for log in &local_logs {
        if stop.load(Ordering::SeqCst) {
            break;
        }

        let log_str = log.to_string_lossy().to_string();
        ui.show_loading(&log_str);

        let ui_for_load = Arc::clone(&ui);
        let nonces = match parser::load_nested_nonces(&log_str, |idx, uid, name| {
            ui_for_load.show_nonce_loaded(idx, uid, name);
        }) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to parse {log_str}: {e}");
                continue;
            }
        };

        if nonces.is_empty() {
            eprintln!("No nonces loaded from {log_str}, skipping.");
            continue;
        }

        ui.show_loading_complete(nonces.len());
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

    upload::upload_dicts(&mut sess, &local_dicts);
    upload::merge_and_upload_keys(&mut sess, &all_keys, &logs_dir)?;

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
