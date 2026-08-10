use crate::ext::result::{ResultExt, Rslt};
use crate::flipper::FlipperSession;
use crate::ui::Ui;
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const ASSETS_DIR: &str = "/ext/nfc/assets";
const RESULT_REMOTE_NAME: &str = "mf_classic_dict_user.nfc";
const TRANSFER_BAR_THRESHOLD: usize = 64 * 1024;

fn parse_existing_keys(data: &[u8]) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for line in String::from_utf8_lossy(data).lines() {
        let l = line.trim();
        if !l.is_empty() {
            set.insert(l.to_uppercase());
        }
    }
    set
}

fn merge_key_sets(
    existing: &BTreeSet<String>,
    new: &BTreeSet<String>,
) -> (usize, BTreeSet<String>) {
    let mut merged = existing.clone();
    let before = merged.len();
    for k in new {
        merged.insert(k.clone());
    }
    let added = merged.len() - before;
    (added, merged)
}

pub fn upload_dicts(ui: &Ui, sess: &mut FlipperSession, local_dicts: &[PathBuf]) {
    if local_dicts.is_empty() {
        return;
    }

    let mut dicts = local_dicts.to_vec();
    dicts.sort();
    dicts.dedup();

    ui.show_uploading_dicts(dicts.len());

    for dict_path in &dicts {
        let file_name = match dict_path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => {
                ui.show_error(&format!(
                    "warning: skipping dict with invalid name: {dict_path:?}"
                ));
                continue;
            }
        };

        let data = match fs::read(dict_path) {
            Ok(d) => d,
            Err(e) => {
                ui.show_error(&format!(
                    "warning: cannot read local dict {dict_path:?}: {e}"
                ));
                continue;
            }
        };

        let remote_dict = format!("{ASSETS_DIR}/{file_name}");

        let _ = sess.storage_delete(&remote_dict, false);

        let show_progress = data.len() > TRANSFER_BAR_THRESHOLD;
        if show_progress {
            ui.begin_transfer(&file_name, data.len());
        }
        let res =
            sess.storage_write_with_progress(&remote_dict, &data, |sent, _| ui.update_transfer(sent));
        if show_progress {
            ui.end_transfer();
        }

        match res {
            Ok(_) => ui.show_dict_uploaded(&remote_dict),
            Err(e) => ui.show_error(&format!("warning: upload {remote_dict} failed: {e}")),
        }
    }
}

pub fn merge_and_upload_keys(
    ui: &Ui,
    sess: &mut FlipperSession,
    all_keys: &BTreeSet<String>,
    logs_dir: &Path,
) -> Rslt<()> {
    if all_keys.is_empty() {
        ui.show_no_keys_in_attack();
        return Ok(());
    }
    ui.show_keys_found_count(all_keys.len());

    let result_path = logs_dir.join(RESULT_REMOTE_NAME);
    {
        let mut f =
            fs::File::create(&result_path).with_context(|| format!("create {result_path:?}"))?;
        for k in all_keys {
            writeln!(f, "{k}").context("write keys")?;
        }
    }
    ui.show_keys_saved(&result_path);

    let remote_out = format!("{ASSETS_DIR}/{RESULT_REMOTE_NAME}");

    let existing = sess
        .storage_read(&remote_out)
        .ok()
        .filter(|d| !d.is_empty());
    let had_existing = existing.is_some();

    let existing_keys = existing
        .as_deref()
        .map(parse_existing_keys)
        .unwrap_or_default();
    let before = existing_keys.len();
    let (added, final_keys) = merge_key_sets(&existing_keys, all_keys);

    ui.show_dict_merge_status(had_existing.then_some(before), added);

    if had_existing && added == 0 {
        ui.show_nothing_new_to_upload();
        ui.show_local_copies(logs_dir);
        return Ok(());
    }

    let mut body = final_keys.into_iter().collect::<Vec<_>>().join("\n");
    body.push('\n');
    let upload = body.into_bytes();

    if let Err(e) = fs::write(&result_path, &upload) {
        ui.show_error(&format!(
            "warning: cannot write local copy {result_path:?}: {e}"
        ));
    } else {
        ui.show_keys_saved_locally(&result_path);
    }

    let _ = sess.storage_delete(&remote_out, false);
    ui.show_uploading_full_dict(&remote_out);

    let show_progress = upload.len() > TRANSFER_BAR_THRESHOLD;
    if show_progress {
        ui.begin_transfer(RESULT_REMOTE_NAME, upload.len());
    }
    let res =
        sess.storage_write_with_progress(&remote_out, &upload, |sent, _| ui.update_transfer(sent));
    if show_progress {
        ui.end_transfer();
    }
    res.with_context(|| format!("upload {remote_out}"))?;

    ui.show_upload_done(&remote_out);
    ui.show_local_copies(logs_dir);

    Ok(())
}

#[cfg(test)]
#[path = "../tests/auto_upload.rs"]
mod tests;
