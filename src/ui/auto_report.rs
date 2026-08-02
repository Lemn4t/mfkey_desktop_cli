use super::{MessageKind, Ui};

fn render_dict_merge_detail(existing_count: Option<usize>, added: usize) -> String {
    match existing_count {
        Some(before) => format!("existing dict has {before} key(s); adding {added} new"),
        None => "no existing dict on device, creating a new one".to_string(),
    }
}

impl Ui {
    pub fn show_searching_for_flipper(&self) {
        self.status_line("→ Searching for Flipper Zero...", MessageKind::Info);
    }

    pub fn show_flipper_port(&self, port: &str) {
        self.status_value_line("✓ Flipper port:", port, MessageKind::Success);
    }

    pub fn show_opening_session(&self) {
        self.status_line("→ Opening RPC session...", MessageKind::Info);
    }

    pub fn show_session_ready(&self) {
        self.status_line("✓ RPC session is up (ping OK)", MessageKind::Success);
    }

    pub fn show_listing_dir(&self, dir: &str) {
        self.status_detail_line("→ Listing", dir, MessageKind::Info, false);
    }

    pub fn show_no_logs_found(&self, dir: &str) {
        self.status_line(
            &format!("No logs (*.mfkey32.log / *.nested.log) found in {dir}. Nothing to attack."),
            MessageKind::Warning,
        );
    }

    pub fn show_logs_found(&self, names: &[String]) {
        self.status_detail_line(
            "✓ Found",
            &format!("{} file(s): {}", names.len(), names.join(", ")),
            MessageKind::Success,
            false,
        );
    }

    pub fn show_downloading(&self, remote: &str) {
        self.status_detail_line("↓ Downloading", remote, MessageKind::Info, false);
    }

    pub fn show_saved_local_log(&self, path: &std::path::Path, bytes: usize) {
        self.dimmed_detail_line("saved", &format!("{} ({} bytes)", path.display(), bytes));
    }

    pub fn show_running_attack(&self) {
        self.status_line("→ Running attack...", MessageKind::Info);
    }

    pub fn show_deleting_from_device(&self, remote: &str) {
        self.status_detail_line("✗ Deleting from device", remote, MessageKind::Info, false);
    }

    pub fn show_uploading_dicts(&self, count: usize) {
        self.status_detail_line(
            "↑ Uploading",
            &format!("{count} candidate dict(s) to device..."),
            MessageKind::Info,
            false,
        );
    }

    pub fn show_dict_uploaded(&self, remote: &str) {
        self.dimmed_detail_line("uploaded:", remote);
    }

    pub fn show_no_keys_in_attack(&self) {
        self.status_line("Attack found no keys.", MessageKind::Warning);
    }

    pub fn show_keys_found_count(&self, count: usize) {
        self.status_detail_line(
            "✓ Found",
            &format!("{count} key(s)"),
            MessageKind::Success,
            false,
        );
    }

    pub fn show_keys_saved(&self, path: &std::path::Path) {
        self.dimmed_detail_line("keys saved:", &path.display().to_string());
    }

    pub fn show_dict_merge_status(&self, existing_count: Option<usize>, added: usize) {
        let detail = render_dict_merge_detail(existing_count, added);
        self.status_detail_line("→", &detail, MessageKind::Info, false);
    }

    pub fn show_nothing_new_to_upload(&self) {
        self.status_line(
            "✓ Nothing new to upload (all keys already present).",
            MessageKind::Success,
        );
    }

    pub fn show_local_copies(&self, dir: &std::path::Path) {
        self.show_detail(&format!("local copies: {}", dir.display()));
    }

    pub fn show_keys_saved_locally(&self, path: &std::path::Path) {
        self.dimmed_detail_line("keys saved locally:", &path.display().to_string());
    }

    pub fn show_uploading_full_dict(&self, remote: &str) {
        self.status_detail_line(
            "↑ Uploading",
            &format!("{remote} (full file)"),
            MessageKind::Info,
            false,
        );
    }

    pub fn show_upload_done(&self, remote: &str) {
        self.status_detail_line(
            "✓ Done. Keys uploaded to",
            remote,
            MessageKind::Success,
            true,
        );
    }
}

#[cfg(test)]
#[path = "../tests/ui_auto_report.rs"]
mod tests;
