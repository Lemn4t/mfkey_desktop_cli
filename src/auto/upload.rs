use crate::ext::result::{ResultExt, Rslt};
use crate::flipper::FlipperSession;
use crate::ui::{MessageKind, Ui};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const ASSETS_DIR: &str = "/ext/nfc/assets";
const RESULT_REMOTE_NAME: &str = "mf_classic_dict_user.nfc";

pub fn upload_dicts(ui: &Ui, sess: &mut FlipperSession, local_dicts: &[PathBuf]) {
    if local_dicts.is_empty() {
        return;
    }

    let mut dicts = local_dicts.to_vec();
    dicts.sort();
    dicts.dedup();

    ui.show_status_detail(
        "↑ Uploading",
        &format!("{} candidate dict(s) to device...", dicts.len()),
        MessageKind::Info,
        false,
    );

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

        match sess.storage_write(&remote_dict, &data) {
            Ok(_) => ui.show_detail_dimmed("uploaded:", &remote_dict),
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
        ui.show_status("Attack found no keys.", MessageKind::Warning);
        return Ok(());
    }
    ui.show_status_detail(
        "✓ Found",
        &format!("{} key(s)", all_keys.len()),
        MessageKind::Success,
        false,
    );

    let result_path = logs_dir.join(RESULT_REMOTE_NAME);
    {
        let mut f =
            fs::File::create(&result_path).with_context(|| format!("create {result_path:?}"))?;
        for k in all_keys {
            writeln!(f, "{k}").context("write keys")?;
        }
    }
    ui.show_detail_dimmed("keys saved:", &result_path.display().to_string());

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
        ui.show_status_detail(
            "→",
            &format!("existing dict has {before} key(s); adding {added} new"),
            MessageKind::Info,
            false,
        );
    } else {
        ui.show_status_detail(
            "→",
            "no existing dict on device, creating a new one",
            MessageKind::Info,
            false,
        );
    }

    if had_existing && added == 0 {
        ui.show_status(
            "✓ Nothing new to upload (all keys already present).",
            MessageKind::Success,
        );
        ui.show_detail(&format!("local copies: {}", logs_dir.display()));
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
        ui.show_detail_dimmed("keys saved locally:", &result_path.display().to_string());
    }

    let _ = sess.storage_delete(&remote_out, false);
    ui.show_status_detail(
        "↑ Uploading",
        &format!("{remote_out} (full file)"),
        MessageKind::Info,
        false,
    );
    sess.storage_write(&remote_out, &upload)
        .with_context(|| format!("upload {remote_out}"))?;

    ui.show_status_detail(
        "✓ Done. Keys uploaded to",
        &remote_out,
        MessageKind::Success,
        true,
    );
    ui.show_detail(&format!("local copies: {}", logs_dir.display()));

    Ok(())
}
