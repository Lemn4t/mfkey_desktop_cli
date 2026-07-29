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

#[derive(Debug, Clone, Copy)]
pub struct UiOptions {
    pub plain_ui: bool,
}

impl Default for UiOptions {
    fn default() -> Self {
        UiOptions { plain_ui: true }
    }
}

pub fn should_use_plain_mode(requested_plain: bool) -> bool {
    requested_plain
        || std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty())
        || !console::user_attended()
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
        self.opts.plain_ui
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
        let plain = self.opts.plain_ui;
        if plain {
            self.write_line("MIFARE Classic Key Recovery Tool");
            self.write_line(&theme::heavy_rule(plain));
            return;
        }

        let term = Term::stdout();
        let _ = term.clear_screen();

        let art = r#"
███╗   ███╗███████╗██╗  ██╗███████╗██╗   ██╗
████╗ ████║██╔════╝██║ ██╔╝██╔════╝╚██╗ ██╔╝
██╔████╔██║█████╗  █████╔╝ █████╗   ╚████╔╝
██║╚██╔╝██║██╔══╝  ██╔═██╗ ██╔══╝    ╚██╔╝
██║ ╚═╝ ██║██║     ██║  ██╗███████╗   ██║
╚═╝     ╚═╝╚═╝     ╚═╝  ╚═╝╚══════╝   ╚═╝   "#;

        self.write_line(&theme::banner_title(art, plain));
        self.write_line(&theme::block_style(
            &format!(
                "{} Flipper Zero :: MIFARE Classic Key Recovery Tool {}",
                glyph::BANNER_LEFT,
                glyph::BANNER_RIGHT
            ),
            plain,
        ));
        self.write_line(&format!("{}\n", theme::heavy_rule(plain)));
    }

    pub fn show_config(&self, input: &str, output: &str, dict_dir: Option<&str>) {
        let plain = self.opts.plain_ui;
        if plain {
            self.write_line(&format!("Input file:  {}", input));
            self.write_line(&format!("Output file: {}", output));
            if let Some(d) = dict_dir {
                self.write_line(&format!("Dict output dir: {}", d));
            }
            self.write_line(&format!("{}\n", theme::heavy_rule(plain)));
            return;
        }

        let lines = [
            format!("{} Input file:  {}", glyph::BULLET, input),
            format!("{} Output file: {}", glyph::BULLET, output),
        ];
        for l in &lines {
            self.write_line(&theme::accent(l, plain));
        }
        if let Some(d) = dict_dir {
            let l = format!("{} Dict dir:    {}", glyph::BULLET, d);
            self.write_line(&theme::accent(&l, plain));
        }
        self.write_line("");
    }

    pub fn show_loading(&self, filename: &str) {
        if self.opts.plain_ui {
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
        self.write_line(&theme::colored(text, kind, self.opts.plain_ui));
    }

    fn status_detail_line(&self, prefix: &str, detail: &str, kind: MessageKind, bold: bool) {
        let plain = self.opts.plain_ui;
        let styled_prefix = if bold {
            theme::colored_bold(prefix, kind, plain)
        } else {
            theme::colored(prefix, kind, plain)
        };
        self.write_line(&format!("{} {}", styled_prefix, detail));
    }

    fn status_value_line(&self, prefix: &str, value: &str, kind: MessageKind) {
        let plain = self.opts.plain_ui;
        self.write_line(&format!(
            "{} {}",
            theme::colored(prefix, kind, plain),
            theme::emphasis(value, plain)
        ));
    }

    fn dimmed_detail_line(&self, prefix: &str, detail: &str) {
        let plain = self.opts.plain_ui;
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
        let detail = match existing_count {
            Some(before) => format!("existing dict has {before} key(s); adding {added} new"),
            None => "no existing dict on device, creating a new one".to_string(),
        };
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
        let plain = self.opts.plain_ui;
        if plain {
            self.write_line(&format!(
                "Loaded nonce {}: UID=0x{:08X}, attack={}",
                index, uid, attack_type
            ));
            return;
        }

        let kind = if attack_type == "static_encrypted" {
            MessageKind::Warning
        } else {
            MessageKind::Success
        };
        let tag = theme::colored(&format!("[{}]", attack_type), kind, plain);
        self.write_line(&format!(
            "{} Loaded nonce {}: UID=0x{:08X} {}",
            theme::indent(1, glyph::TREE),
            index,
            uid,
            tag
        ));
    }

    pub fn show_loading_complete(&self, total: usize) {
        let plain = self.opts.plain_ui;
        if plain {
            self.write_line(&format!("Total nonces loaded: {}\n", total));
            return;
        }
        let n = theme::emphasis(&total.to_string(), plain);
        self.write_line(&format!(
            "{} Total nonces loaded: {}\n",
            theme::indent(1, glyph::TREE),
            n
        ));
    }

    pub fn show_hardnested_unsupported(&self, context: Option<&str>, skipping: bool) {
        let suffix = if skipping { ", skipping." } else { "." };
        let msg = match context {
            Some(path) => format!(
                "HardNested nonces detected in {path} — this attack is not supported (yet){suffix}"
            ),
            None => {
                format!("HardNested nonces detected — this attack is not supported (yet){suffix}")
            }
        };
        self.write_err_line(&theme::colored_bold(
            &msg,
            MessageKind::Warning,
            self.opts.plain_ui,
        ));
    }

    pub fn show_hardnested_note(&self, context: Option<&str>) {
        let msg = match context {
            Some(path) => format!(
                "Note: HardNested nonces were also found in {path} and were skipped — that attack is not supported (yet)."
            ),
            None => "Note: HardNested nonces were also found in this file and were skipped — that attack is not supported (yet).".to_string(),
        };
        self.write_line(&theme::colored(
            &msg,
            MessageKind::Warning,
            self.opts.plain_ui,
        ));
    }

    pub fn show_error(&self, text: &str) {
        self.write_err_line(&theme::colored(
            text,
            MessageKind::Error,
            self.opts.plain_ui,
        ));
    }

    pub fn show_interrupt(&self) {
        self.write_err_line("\n\nReceived interrupt signal. Stopping gracefully...");
    }

    pub fn show_detail(&self, text: &str) {
        self.write_line(&theme::indent(1, text));
    }

    pub fn show_start(&self) {
        let plain = self.opts.plain_ui;
        if plain {
            self.write_line("Starting key recovery... (Press Ctrl+C to stop gracefully.)\n");
            return;
        }
        let msg = theme::block_style(
            &format!("{} Starting key recovery... ", glyph::BLOCK),
            plain,
        );
        self.write_line(&format!("{}(Press Ctrl+C to stop)", msg));
        self.write_line(&theme::light_rule());
    }

    pub fn begin_progress(&self, total_nonces: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.total_nonces = total_nonces;
        if inner.bar.is_some() {
            return;
        }
        let pb = ProgressBar::new(total_nonces.max(1) as u64);
        if self.opts.plain_ui {
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

        let msg = if self.opts.plain_ui {
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
        let plain = self.opts.plain_ui;
        if plain {
            self.write_line(&format!("Found key: {}", key.to_hex()));
            return;
        }

        let line = format!("{} Found key: {}", glyph::CHECK, key.to_hex());
        self.write_line(&theme::colored(&line, MessageKind::Success, plain));
    }

    pub fn show_summary(&self, total_nonces: usize, found_keys: usize, candidate_keys: usize) {
        self.clear_progress();
        let plain = self.opts.plain_ui;

        if plain {
            self.write_line(&format!("\n{}", theme::heavy_rule(plain)));
            self.write_line("Key recovery completed!\n");
            self.write_line("Summary:");
            self.write_line(&format!("Total nonces processed: {}", total_nonces));
            self.write_line(&format!("Keys found: {}", found_keys));
            self.write_line(&format!("Candidate keys: {}", candidate_keys));
            return;
        }

        self.write_line(&theme::heavy_rule(plain));
        let header = format!(
            "{} Key Recovery Complete! {}",
            glyph::BLOCK,
            glyph::BLOCK_END
        );
        self.write_line(&theme::block_style(&header, plain));

        self.write_line("\nSummary:");
        self.write_line(&format!(
            "{} Total nonces processed: {}",
            glyph::BULLET,
            total_nonces
        ));

        let kf = format!("{} Keys found: {}", glyph::BULLET, found_keys);
        let ck = format!("{} Candidate keys: {}", glyph::BULLET, candidate_keys);
        self.write_line(&theme::colored(&kf, MessageKind::Success, plain));
        self.write_line(&theme::accent(&ck, plain));
    }

    pub fn show_found_keys_list(&self, keys: &[MfClassicKey]) {
        if keys.is_empty() {
            return;
        }
        let plain = self.opts.plain_ui;

        self.write_line("\nFound Keys:");
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
            self.write_line(&theme::indent(1, &line));
        }
    }

    pub fn show_saved_files(&self, keys_file: Option<&str>, keys_count: usize) {
        self.write_line("\nFiles saved:");
        if let Some(f) = keys_file
            && keys_count > 0
        {
            let line = format!("{} {} ({} keys)", glyph::BULLET, f, keys_count);
            self.write_line(&theme::indent(
                1,
                &theme::colored(&line, MessageKind::Success, self.opts.plain_ui),
            ));
        }
    }

    pub fn show_saved_dicts(&self, dicts: &[(String, usize)]) {
        if dicts.is_empty() {
            return;
        }
        let plain = self.opts.plain_ui;

        self.write_line(&format!(
            "\nCandidate dictionaries ({} files):",
            dicts.len()
        ));
        for (path, count) in dicts {
            let filename = std::path::Path::new(path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let line = format!("{} {} ({} candidates)", glyph::BULLET, filename, count);
            self.write_line(&theme::indent(1, &theme::accent(&line, plain)));
        }
    }

    pub fn show_no_keys_found(&self) {
        let plain = self.opts.plain_ui;
        if plain {
            self.write_line("No keys were recovered. This could happen if:");
            self.write_line("  * The nonces are invalid or corrupted");
            self.write_line("  * The keyspace being searched doesn't contain the key");
            self.write_line("  * The attack was interrupted before completion\n");
            return;
        }

        let line = format!("\n{} No keys were recovered.", glyph::CROSS);
        self.write_line(&theme::colored(&line, MessageKind::Error, plain));
        self.write_line("\nThis could happen if:");
        self.write_line(&format!(
            "  {} The nonces are invalid or corrupted",
            glyph::DOT
        ));
        self.write_line(&format!(
            "  {} The keyspace being searched doesn't contain the key",
            glyph::DOT
        ));
        self.write_line(&format!(
            "  {} The attack was interrupted before completion\n",
            glyph::DOT
        ));
    }

    pub fn show_disclaimer(&self, text: &str) {
        self.write_line(&theme::accent(text, self.opts.plain_ui));
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
            self.opts.plain_ui,
        ));
    }

    pub fn confirm(&self, prompt: &str, default_yes: bool) -> bool {
        if !self.opts.plain_ui {
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
