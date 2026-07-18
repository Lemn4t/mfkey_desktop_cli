use crate::flipper::FlipperSession;
use colored::Colorize;
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const ASSETS_DIR: &str = "/ext/nfc/assets";
const RESULT_REMOTE_NAME: &str = "mf_classic_dict_user.nfc";

pub fn upload_dicts(sess: &mut FlipperSession, local_dicts: &[PathBuf]) {
    if local_dicts.is_empty() {
        return;
    }

    let mut dicts = local_dicts.to_vec();
    dicts.sort();
    dicts.dedup();

    println!(
        "{} {} candidate dict(s) to device...",
        "↑ Uploading".cyan(),
        dicts.len()
    );

    for dict_path in &dicts {
        let file_name = match dict_path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => {
                eprintln!("warning: skipping dict with invalid name: {dict_path:?}");
                continue;
            }
        };

        let data = match fs::read(dict_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("warning: cannot read local dict {dict_path:?}: {e}");
                continue;
            }
        };

        let remote_dict = format!("{ASSETS_DIR}/{file_name}");

        let _ = sess.storage_delete(&remote_dict, false);

        match sess.storage_write(&remote_dict, &data) {
            Ok(_) => println!("  {} {}", "uploaded:".dimmed(), remote_dict),
            Err(e) => eprintln!("warning: upload {remote_dict} failed: {e}"),
        }
    }
}

pub fn merge_and_upload_keys(
    sess: &mut FlipperSession,
    all_keys: &BTreeSet<String>,
    logs_dir: &Path,
) -> Result<(), String> {
    if all_keys.is_empty() {
        println!("{}", "Attack found no keys.".yellow());
        return Ok(());
    }
    println!("{} {} key(s)", "✓ Found".green(), all_keys.len());

    let result_path = logs_dir.join(RESULT_REMOTE_NAME);
    {
        let mut f =
            fs::File::create(&result_path).map_err(|e| format!("create {result_path:?}: {e}"))?;
        for k in all_keys {
            writeln!(f, "{k}").map_err(|e| format!("write keys: {e}"))?;
        }
    }
    println!("  {} {}", "keys saved:".dimmed(), result_path.display());

    let remote_out = format!("{ASSETS_DIR}/{RESULT_REMOTE_NAME}");

    let mut final_keys: BTreeSet<String> = BTreeSet::new();

    let existing = sess
        .storage_read(&remote_out)
        .ok()
        .filter(|d| !d.is_empty());
    let had_existing = existing.is_some();

    if let Some(data) = &existing {
        for line in String::from_utf8_lossy(data).lines() {
            let l = line.trim();
            if !l.is_empty() {
                final_keys.insert(l.to_uppercase());
            }
        }
    }

    let before = final_keys.len();
    for k in all_keys {
        final_keys.insert(k.clone());
    }
    let added = final_keys.len() - before;

    if had_existing {
        println!(
            "{} existing dict has {} key(s); adding {} new",
            "→".cyan(),
            before,
            added
        );
    } else {
        println!(
            "{} no existing dict on device, creating a new one",
            "→".cyan()
        );
    }

    if had_existing && added == 0 {
        println!(
            "{}",
            "✓ Nothing new to upload (all keys already present).".green()
        );
        println!("  local copies: {}", logs_dir.display());
        return Ok(());
    }

    let mut body = final_keys.into_iter().collect::<Vec<_>>().join("\n");
    body.push('\n');
    let upload = body.into_bytes();

    if let Err(e) = fs::write(&result_path, &upload) {
        eprintln!("warning: cannot write local copy {result_path:?}: {e}");
    } else {
        println!(
            "  {} {}",
            "keys saved locally:".dimmed(),
            result_path.display()
        );
    }

    let _ = sess.storage_delete(&remote_out, false);
    println!("{} {} (full file)", "↑ Uploading".cyan(), remote_out);
    sess.storage_write(&remote_out, &upload)
        .map_err(|e| format!("upload {remote_out}: {e}"))?;

    println!(
        "{} {}",
        "✓ Done. Keys uploaded to".green().bold(),
        remote_out
    );
    println!("  local copies: {}", logs_dir.display());

    Ok(())
}
