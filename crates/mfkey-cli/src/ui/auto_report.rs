use super::theme::glyph;
use super::{MessageKind, Ui};

fn render_dict_merge_detail(existing_count: Option<usize>, added: usize) -> String {
    match existing_count {
        Some(before) => format!("existing dict has {before} key(s); adding {added} new"),
        None => "no existing dict on device, creating a new one".to_string(),
    }
}

impl Ui {
    pub fn show_searching_for_flipper(&self) {
        let text = self.with_glyph(glyph::ARROW, "Searching for Flipper Zero...");
        self.status_line(&text, MessageKind::Info);
    }

    pub fn show_flipper_port(&self, port: &str) {
        let prefix = self.with_glyph(glyph::CHECK, "Flipper port:");
        self.status_value_line(&prefix, port, MessageKind::Success);
    }

    pub fn show_scanning_ble(&self) {
        let text = self.with_glyph(glyph::ARROW, "Scanning for Flipper Zero over BLE...");
        self.status_line(&text, MessageKind::Info);
    }

    pub fn show_ble_device(&self, label: &str) {
        let prefix = self.with_glyph(glyph::CHECK, "Flipper (BLE):");
        self.status_value_line(&prefix, label, MessageKind::Success);
    }

    pub fn show_opening_session(&self) {
        let text = self.with_glyph(glyph::ARROW, "Opening RPC session...");
        self.status_line(&text, MessageKind::Info);
    }

    pub fn show_session_ready(&self) {
        let text = self.with_glyph(glyph::CHECK, "RPC session is up (ping OK)");
        self.status_line(&text, MessageKind::Success);
    }

    pub fn show_listing_dir(&self, dir: &str) {
        let prefix = self.with_glyph(glyph::ARROW, "Listing");
        self.status_detail_line(&prefix, dir, MessageKind::Info, false);
    }

    pub fn show_no_logs_found(&self, dir: &str) {
        self.status_line(
            &format!("No logs (*.mfkey32.log / *.nested.log) found in {dir}. Nothing to attack."),
            MessageKind::Warning,
        );
    }

    pub fn show_logs_found(&self, names: &[String]) {
        let prefix = self.with_glyph(glyph::CHECK, "Found");
        self.status_detail_line(
            &prefix,
            &format!("{} file(s): {}", names.len(), names.join(", ")),
            MessageKind::Success,
            false,
        );
    }

    pub fn show_downloading(&self, remote: &str) {
        let prefix = self.with_glyph(glyph::DOWN, "Downloading");
        self.status_detail_line(&prefix, remote, MessageKind::Info, false);
    }

    pub fn show_saved_local_log(&self, path: &std::path::Path, bytes: usize) {
        self.dimmed_detail_line("saved", &format!("{} ({} bytes)", path.display(), bytes));
    }

    pub fn show_running_attack(&self) {
        let text = self.with_glyph(glyph::ARROW, "Running attack...");
        self.status_line(&text, MessageKind::Info);
    }

    pub fn show_deleting_from_device(&self, remote: &str) {
        let prefix = self.with_glyph(glyph::CROSS, "Deleting from device");
        self.status_detail_line(&prefix, remote, MessageKind::Info, false);
    }

    pub fn show_uploading_dicts(&self, count: usize) {
        let prefix = self.with_glyph(glyph::UP, "Uploading");
        self.status_detail_line(
            &prefix,
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
        let prefix = self.with_glyph(glyph::CHECK, "Found");
        self.status_detail_line(
            &prefix,
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
        if self.is_plain() {
            self.status_line(&detail, MessageKind::Info);
        } else {
            self.status_detail_line(glyph::ARROW, &detail, MessageKind::Info, false);
        }
    }

    pub fn show_nothing_new_to_upload(&self) {
        let text = self.with_glyph(
            glyph::CHECK,
            "Nothing new to upload (all keys already present).",
        );
        self.status_line(&text, MessageKind::Success);
    }

    pub fn show_local_copies(&self, dir: &std::path::Path) {
        self.show_detail(&format!("local copies: {}", dir.display()));
    }

    pub fn show_keys_saved_locally(&self, path: &std::path::Path) {
        self.dimmed_detail_line("keys saved locally:", &path.display().to_string());
    }

    pub fn show_uploading_full_dict(&self, remote: &str) {
        let prefix = self.with_glyph(glyph::UP, "Uploading");
        self.status_detail_line(
            &prefix,
            &format!("{remote} (full file)"),
            MessageKind::Info,
            false,
        );
    }

    pub fn show_upload_done(&self, remote: &str) {
        let prefix = self.with_glyph(glyph::CHECK, "Done. Keys uploaded to");
        self.status_detail_line(&prefix, remote, MessageKind::Success, true);
    }
}

#[cfg(test)]
#[path = "../tests/ui_auto_report.rs"]
mod tests;
