use super::theme::{self, glyph};
use super::{MessageKind, OutputMode, Ui};
use crate::core::model::MfClassicKey;

fn render_config(
    mode: OutputMode,
    input: &str,
    output: &str,
    dict_dir: Option<&str>,
) -> Vec<String> {
    let plain = mode.is_plain();
    if plain {
        let mut lines = vec![
            format!("Input file:  {}", input),
            format!("Output file: {}", output),
        ];
        if let Some(d) = dict_dir {
            lines.push(format!("Dict output dir: {}", d));
        }
        lines.push(format!("{}\n", theme::heavy_rule(plain)));
        return lines;
    }

    let mut lines = vec![
        theme::accent(&format!("{} Input file:  {}", glyph::BULLET, input), plain),
        theme::accent(&format!("{} Output file: {}", glyph::BULLET, output), plain),
    ];
    if let Some(d) = dict_dir {
        lines.push(theme::accent(
            &format!("{} Dict dir:    {}", glyph::BULLET, d),
            plain,
        ));
    }
    lines.push(String::new());
    lines
}

fn render_nonce_loaded(mode: OutputMode, index: usize, uid: u32, attack_type: &str) -> String {
    let plain = mode.is_plain();
    if plain {
        return format!(
            "Loaded nonce {}: UID=0x{:08X}, attack={}",
            index, uid, attack_type
        );
    }

    let kind = if attack_type == "static_encrypted" {
        MessageKind::Warning
    } else {
        MessageKind::Success
    };
    let tag = theme::colored(&format!("[{}]", attack_type), kind, plain);
    format!(
        "{} Loaded nonce {}: UID=0x{:08X} {}",
        theme::indent(1, glyph::TREE),
        index,
        uid,
        tag
    )
}

fn render_loading_complete(mode: OutputMode, total: usize) -> String {
    let plain = mode.is_plain();
    if plain {
        return format!("Total nonces loaded: {}\n", total);
    }
    let n = theme::emphasis(&total.to_string(), plain);
    format!(
        "{} Total nonces loaded: {}\n",
        theme::indent(1, glyph::TREE),
        n
    )
}

fn render_hardnested_unsupported(
    mode: OutputMode,
    context: Option<&str>,
    skipping: bool,
) -> String {
    let suffix = if skipping { ", skipping." } else { "." };
    let msg = match context {
        Some(path) => format!(
            "HardNested nonces detected in {path} — this attack is not supported (yet){suffix}"
        ),
        None => {
            format!("HardNested nonces detected — this attack is not supported (yet){suffix}")
        }
    };
    theme::colored_bold(&msg, MessageKind::Warning, mode.is_plain())
}

fn render_hardnested_note(mode: OutputMode, context: Option<&str>) -> String {
    let msg = match context {
        Some(path) => format!(
            "Note: HardNested nonces were also found in {path} and were skipped — that attack is not supported (yet)."
        ),
        None => "Note: HardNested nonces were also found in this file and were skipped — that attack is not supported (yet).".to_string(),
    };
    theme::colored(&msg, MessageKind::Warning, mode.is_plain())
}

fn render_start(mode: OutputMode) -> Vec<String> {
    let plain = mode.is_plain();
    if plain {
        return vec!["Starting key recovery... (Press Ctrl+C to stop gracefully.)\n".to_string()];
    }
    let msg = theme::block_style(
        &format!("{} Starting key recovery... ", glyph::BLOCK),
        plain,
    );
    vec![
        format!("{}(Press Ctrl+C to stop)", msg),
        theme::light_rule(),
    ]
}

fn render_summary(
    mode: OutputMode,
    total_nonces: usize,
    found_keys: usize,
    candidate_keys: usize,
) -> Vec<String> {
    let plain = mode.is_plain();
    if plain {
        return vec![
            format!("\n{}", theme::heavy_rule(plain)),
            "Key recovery completed!\n".to_string(),
            "Summary:".to_string(),
            format!("Total nonces processed: {}", total_nonces),
            format!("Keys found: {}", found_keys),
            format!("Candidate keys: {}", candidate_keys),
        ];
    }

    let header = format!(
        "{} Key Recovery Complete! {}",
        glyph::BLOCK,
        glyph::BLOCK_END
    );
    let kf = format!("{} Keys found: {}", glyph::BULLET, found_keys);
    let ck = format!("{} Candidate keys: {}", glyph::BULLET, candidate_keys);
    vec![
        theme::heavy_rule(plain),
        theme::block_style(&header, plain),
        "\nSummary:".to_string(),
        format!("{} Total nonces processed: {}", glyph::BULLET, total_nonces),
        theme::colored(&kf, MessageKind::Success, plain),
        theme::accent(&ck, plain),
    ]
}

fn render_found_keys_list(mode: OutputMode, keys: &[MfClassicKey]) -> Vec<String> {
    if keys.is_empty() {
        return Vec::new();
    }
    let plain = mode.is_plain();

    let mut lines = vec!["\nFound Keys:".to_string()];
    for k in keys {
        let hex = k.to_hex();
        let line = if plain {
            hex
        } else {
            theme::colored(
                &format!("{} {}", glyph::CHECK, hex),
                MessageKind::Success,
                plain,
            )
        };
        lines.push(theme::indent(1, &line));
    }
    lines
}

fn render_saved_files(mode: OutputMode, keys_file: Option<&str>, keys_count: usize) -> Vec<String> {
    let mut lines = vec!["\nFiles saved:".to_string()];
    if let Some(f) = keys_file
        && keys_count > 0
    {
        let line = format!("{} {} ({} keys)", glyph::BULLET, f, keys_count);
        lines.push(theme::indent(
            1,
            &theme::colored(&line, MessageKind::Success, mode.is_plain()),
        ));
    }
    lines
}

fn render_saved_dicts(mode: OutputMode, dicts: &[(String, usize)]) -> Vec<String> {
    if dicts.is_empty() {
        return Vec::new();
    }
    let plain = mode.is_plain();

    let mut lines = vec![format!("\nCandidate dictionaries ({} files):", dicts.len())];
    for (path, count) in dicts {
        let filename = std::path::Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        let line = format!("{} {} ({} candidates)", glyph::BULLET, filename, count);
        lines.push(theme::indent(1, &theme::accent(&line, plain)));
    }
    lines
}

fn render_no_keys_found(mode: OutputMode) -> Vec<String> {
    let plain = mode.is_plain();
    if plain {
        return vec![
            "No keys were recovered. This could happen if:".to_string(),
            "  * The nonces are invalid or corrupted".to_string(),
            "  * The keyspace being searched doesn't contain the key".to_string(),
            "  * The attack was interrupted before completion\n".to_string(),
        ];
    }

    vec![
        theme::colored(
            &format!("\n{} No keys were recovered.", glyph::CROSS),
            MessageKind::Error,
            plain,
        ),
        "\nThis could happen if:".to_string(),
        format!("  {} The nonces are invalid or corrupted", glyph::DOT),
        format!(
            "  {} The keyspace being searched doesn't contain the key",
            glyph::DOT
        ),
        format!(
            "  {} The attack was interrupted before completion\n",
            glyph::DOT
        ),
    ]
}

impl Ui {
    pub fn show_config(&self, input: &str, output: &str, dict_dir: Option<&str>) {
        for line in render_config(self.opts.mode, input, output, dict_dir) {
            self.write_line(&line);
        }
    }

    pub fn show_loading(&self, filename: &str) {
        if self.is_plain() {
            self.write_line(&format!("Loading nonces from {}...", filename));
        } else {
            self.write_line(&format!(
                "{} Loading nonces from {}...",
                glyph::LOADING,
                filename
            ));
        }
    }

    pub fn show_nonce_loaded(&self, index: usize, uid: u32, attack_type: &str) {
        self.write_line(&render_nonce_loaded(
            self.opts.mode,
            index,
            uid,
            attack_type,
        ));
    }

    pub fn show_loading_complete(&self, total: usize) {
        self.write_line(&render_loading_complete(self.opts.mode, total));
    }

    pub fn show_hardnested_unsupported(&self, context: Option<&str>, skipping: bool) {
        self.write_err_line(&render_hardnested_unsupported(
            self.opts.mode,
            context,
            skipping,
        ));
    }

    pub fn show_hardnested_note(&self, context: Option<&str>) {
        self.write_line(&render_hardnested_note(self.opts.mode, context));
    }

    pub fn show_start(&self) {
        for line in render_start(self.opts.mode) {
            self.write_line(&line);
        }
    }

    pub fn show_found_key(&self, key: &MfClassicKey) {
        let plain = self.is_plain();
        if plain {
            self.write_line(&format!("Found key: {}", key.to_hex()));
            return;
        }

        let line = format!("{} Found key: {}", glyph::CHECK, key.to_hex());
        self.write_line(&theme::colored(&line, MessageKind::Success, plain));
    }

    pub fn show_summary(&self, total_nonces: usize, found_keys: usize, candidate_keys: usize) {
        self.clear_progress();
        for line in render_summary(self.opts.mode, total_nonces, found_keys, candidate_keys) {
            self.write_line(&line);
        }
    }

    pub fn show_found_keys_list(&self, keys: &[MfClassicKey]) {
        for line in render_found_keys_list(self.opts.mode, keys) {
            self.write_line(&line);
        }
    }

    pub fn show_saved_files(&self, keys_file: Option<&str>, keys_count: usize) {
        for line in render_saved_files(self.opts.mode, keys_file, keys_count) {
            self.write_line(&line);
        }
    }

    pub fn show_saved_dicts(&self, dicts: &[(String, usize)]) {
        for line in render_saved_dicts(self.opts.mode, dicts) {
            self.write_line(&line);
        }
    }

    pub fn show_no_keys_found(&self) {
        for line in render_no_keys_found(self.opts.mode) {
            self.write_line(&line);
        }
    }
}

#[cfg(test)]
#[path = "../tests/ui_attack_report.rs"]
mod tests;
