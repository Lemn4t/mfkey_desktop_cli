mod theme;

use crate::core::model::MfClassicKey;
use console::Term;
use dialoguer::{Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::sync::Mutex;
use std::time::Duration;
use theme::glyph;

use theme::MessageKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Fancy,
    Plain,
}

impl OutputMode {
    pub fn is_plain(self) -> bool {
        matches!(self, OutputMode::Plain)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiOptions {
    pub mode: OutputMode,
}

impl Default for UiOptions {
    fn default() -> Self {
        UiOptions {
            mode: OutputMode::Plain,
        }
    }
}

pub fn resolve_output_mode(requested_plain: bool) -> OutputMode {
    let plain = requested_plain
        || std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty())
        || !console::user_attended();
    if plain {
        OutputMode::Plain
    } else {
        OutputMode::Fancy
    }
}

struct UiInner {
    bar: Option<ProgressBar>,
    total_nonces: usize,
}

struct Progress {
    current: usize,
    total: usize,
}

impl Progress {
    fn pct(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.current as f32 / self.total as f32 * 100.0
        }
    }
}

fn render_title(mode: OutputMode) -> Vec<String> {
    let plain = mode.is_plain();
    if plain {
        return vec![
            "MIFARE Classic Key Recovery Tool".to_string(),
            theme::heavy_rule(plain),
        ];
    }

    let art = r#"
███╗   ███╗███████╗██╗  ██╗███████╗██╗   ██╗
████╗ ████║██╔════╝██║ ██╔╝██╔════╝╚██╗ ██╔╝
██╔████╔██║█████╗  █████╔╝ █████╗   ╚████╔╝
██║╚██╔╝██║██╔══╝  ██╔═██╗ ██╔══╝    ╚██╔╝
██║ ╚═╝ ██║██║     ██║  ██╗███████╗   ██║
╚═╝     ╚═╝╚═╝     ╚═╝  ╚═╝╚══════╝   ╚═╝   "#;

    vec![
        theme::banner_title(art, plain),
        theme::block_style(
            &format!(
                "{} Flipper Zero :: MIFARE Classic Key Recovery Tool {}",
                glyph::BANNER_LEFT,
                glyph::BANNER_RIGHT
            ),
            plain,
        ),
        format!("{}\n", theme::heavy_rule(plain)),
    ]
}

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

fn render_dict_merge_detail(existing_count: Option<usize>, added: usize) -> String {
    match existing_count {
        Some(before) => format!("existing dict has {before} key(s); adding {added} new"),
        None => "no existing dict on device, creating a new one".to_string(),
    }
}

pub struct Ui {
    opts: UiOptions,
    inner: Mutex<UiInner>,
}

impl Ui {
    pub fn new(opts: UiOptions) -> Self {
        Ui {
            opts,
            inner: Mutex::new(UiInner {
                bar: None,
                total_nonces: 0,
            }),
        }
    }

    pub fn is_plain(&self) -> bool {
        self.opts.mode.is_plain()
    }

    fn write_line(&self, line: &str) {
        let inner = self.inner.lock().unwrap();
        if let Some(pb) = inner.bar.as_ref() {
            pb.println(line);
        } else {
            println!("{line}");
        }
    }

    fn write_err_line(&self, line: &str) {
        let inner = self.inner.lock().unwrap();
        if let Some(pb) = inner.bar.as_ref() {
            pb.suspend(|| eprintln!("{line}"));
        } else {
            eprintln!("{line}");
        }
    }

    pub fn show_title(&self) {
        let plain = self.is_plain();
        if !plain {
            let term = Term::stdout();
            let _ = term.clear_screen();
        }
        for line in render_title(self.opts.mode) {
            self.write_line(&line);
        }
    }

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

    fn status_line(&self, text: &str, kind: MessageKind) {
        self.write_line(&theme::colored(text, kind, self.is_plain()));
    }

    fn status_detail_line(&self, prefix: &str, detail: &str, kind: MessageKind, bold: bool) {
        let plain = self.is_plain();
        let styled_prefix = if bold {
            theme::colored_bold(prefix, kind, plain)
        } else {
            theme::colored(prefix, kind, plain)
        };
        self.write_line(&format!("{} {}", styled_prefix, detail));
    }

    fn status_value_line(&self, prefix: &str, value: &str, kind: MessageKind) {
        let plain = self.is_plain();
        self.write_line(&format!(
            "{} {}",
            theme::colored(prefix, kind, plain),
            theme::emphasis(value, plain)
        ));
    }

    fn dimmed_detail_line(&self, prefix: &str, detail: &str) {
        let plain = self.is_plain();
        let line = format!("{} {}", theme::muted(prefix, plain), detail);
        self.write_line(&theme::indent(1, &line));
    }

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

    pub fn show_error(&self, text: &str) {
        self.write_err_line(&theme::colored(text, MessageKind::Error, self.is_plain()));
    }

    pub fn show_interrupt(&self) {
        self.write_err_line("\n\nReceived interrupt signal. Stopping gracefully...");
    }

    pub fn show_detail(&self, text: &str) {
        self.write_line(&theme::indent(1, text));
    }

    pub fn show_start(&self) {
        for line in render_start(self.opts.mode) {
            self.write_line(&line);
        }
    }

    pub fn begin_progress(&self, total_nonces: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.total_nonces = total_nonces;
        if inner.bar.is_some() {
            return;
        }
        let pb = ProgressBar::new(total_nonces.max(1) as u64);
        if self.is_plain() {
            let style = ProgressStyle::with_template("{msg}")
                .unwrap_or_else(|_| ProgressStyle::default_bar());
            pb.set_style(style);
        } else {
            let style = ProgressStyle::with_template(
                "{prefix:.bold.dim} [{bar:30.cyan/blue}] {pos}/{len} {msg}",
            )
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("█▓░");
            pb.set_style(style);
            pb.set_prefix("▸ Attacking");
            pb.enable_steady_tick(Duration::from_millis(120));
        }
        inner.bar = Some(pb);
    }

    pub fn update_progress(
        &self,
        nonce_current: usize,
        nonce_total: usize,
        msb_current: usize,
        msb_total: usize,
        stage_progress: f32,
        uid: u32,
    ) {
        let inner = self.inner.lock().unwrap();
        let Some(pb) = inner.bar.as_ref() else {
            return;
        };
        pb.set_position(nonce_current.min(nonce_total) as u64);

        let msg = if self.is_plain() {
            let nonce_pct = Progress {
                current: nonce_current,
                total: nonce_total,
            }
            .pct();
            let msb_pct = Progress {
                current: msb_current,
                total: msb_total,
            }
            .pct();
            format!(
                "Progress: Nonce {}/{} ({:.1}%) | MSB {}/{} ({:.1}%) | Current {:.1}%",
                nonce_current,
                nonce_total,
                nonce_pct,
                msb_current,
                msb_total,
                msb_pct,
                stage_progress
            )
        } else {
            format!("UID 0x{:08X} | MSB {}/{}", uid, msb_current, msb_total)
        };
        pb.set_message(msg);
    }

    pub fn clear_progress(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(pb) = inner.bar.take() {
            pb.finish_and_clear();
        }
        inner.total_nonces = 0;
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

    pub fn show_disclaimer(&self, text: &str) {
        self.write_line(&theme::accent(text, self.is_plain()));
    }

    pub fn show_pill_taken(&self, accepted: bool) {
        let text = if accepted {
            format!("{} Red pill taken", glyph::CHECK)
        } else {
            format!("{} Blue pill taken", glyph::CHECK)
        };
        self.write_line(&theme::colored(
            &text,
            MessageKind::Success,
            self.is_plain(),
        ));
    }

    pub fn confirm(&self, prompt: &str, default_yes: bool) -> bool {
        if !self.is_plain() {
            let theme = ColorfulTheme::default();
            let items = &["Yes", "No"];
            let default = if default_yes { 0 } else { 1 };
            let choice = Select::with_theme(&theme)
                .with_prompt(prompt)
                .items(items)
                .default(default)
                .interact()
                .unwrap_or(1);
            return choice == 0;
        }

        let hint = if default_yes {
            "(Y/n, Enter = Yes)"
        } else {
            "(y/N, Enter = No)"
        };
        loop {
            print!("{prompt} {hint}: ");
            let _ = std::io::stdout().flush();

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                return default_yes;
            }

            match input.trim() {
                "y" | "Y" | "yes" | "YES" => return true,
                "n" | "N" | "no" | "NO" => return false,
                "" => return default_yes,
                _ => println!("Invalid answer, please type y or n."),
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/ui_mod.rs"]
mod tests;
