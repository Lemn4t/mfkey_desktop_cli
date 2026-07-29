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

struct UiInner {
    bar: Option<ProgressBar>,
    total_nonces: usize,
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

    pub fn show_title(&self) {
        let plain = self.opts.plain_ui;
        if plain {
            println!("MIFARE Classic Key Recovery Tool");
            println!("{}", theme::heavy_rule(plain));
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

        println!("{}", theme::banner_title(art, plain));
        println!(
            "{}",
            theme::block_style(
                &format!(
                    "{} Flipper Zero :: MIFARE Classic Key Recovery Tool {}",
                    glyph::BANNER_LEFT,
                    glyph::BANNER_RIGHT
                ),
                plain
            )
        );
        println!("{}\n", theme::heavy_rule(plain));
    }

    pub fn show_config(&self, input: &str, output: &str, dict_dir: Option<&str>) {
        let plain = self.opts.plain_ui;
        if plain {
            println!("Input file:  {}", input);
            println!("Output file: {}", output);
            if let Some(d) = dict_dir {
                println!("Dict output dir: {}", d);
            }
            println!("{}\n", theme::heavy_rule(plain));
            return;
        }

        let lines = [
            format!("{} Input file:  {}", glyph::BULLET, input),
            format!("{} Output file: {}", glyph::BULLET, output),
        ];
        for l in &lines {
            println!("{}", theme::accent(l, plain));
        }
        if let Some(d) = dict_dir {
            let l = format!("{} Dict dir:    {}", glyph::BULLET, d);
            println!("{}", theme::accent(&l, plain));
        }
        println!();
    }

    pub fn show_loading(&self, filename: &str) {
        if self.opts.plain_ui {
            println!("Loading nonces from {}...", filename);
        } else {
            println!("{} Loading nonces from {}...", glyph::LOADING, filename);
        }
    }

    fn status_line(&self, text: &str, kind: MessageKind) {
        println!("{}", theme::colored(text, kind, self.opts.plain_ui));
    }

    fn status_detail_line(&self, prefix: &str, detail: &str, kind: MessageKind, bold: bool) {
        let plain = self.opts.plain_ui;
        let styled_prefix = if bold {
            theme::colored_bold(prefix, kind, plain)
        } else {
            theme::colored(prefix, kind, plain)
        };
        println!("{} {}", styled_prefix, detail);
    }

    fn status_value_line(&self, prefix: &str, value: &str, kind: MessageKind) {
        let plain = self.opts.plain_ui;
        println!(
            "{} {}",
            theme::colored(prefix, kind, plain),
            theme::emphasis(value, plain)
        );
    }

    fn dimmed_detail_line(&self, prefix: &str, detail: &str) {
        let plain = self.opts.plain_ui;
        let line = format!("{} {}", theme::muted(prefix, plain), detail);
        println!("{}", theme::indent(1, &line));
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
            println!(
                "Loaded nonce {}: UID=0x{:08X}, attack={}",
                index, uid, attack_type
            );
            return;
        }

        let kind = if attack_type == "static_encrypted" {
            MessageKind::Warning
        } else {
            MessageKind::Success
        };
        let tag = theme::colored(&format!("[{}]", attack_type), kind, plain);
        println!(
            "{} Loaded nonce {}: UID=0x{:08X} {}",
            theme::indent(1, glyph::TREE),
            index,
            uid,
            tag
        );
    }

    pub fn show_loading_complete(&self, total: usize) {
        let plain = self.opts.plain_ui;
        if plain {
            println!("Total nonces loaded: {}\n", total);
            return;
        }
        let n = theme::emphasis(&total.to_string(), plain);
        println!(
            "{} Total nonces loaded: {}\n",
            theme::indent(1, glyph::TREE),
            n
        );
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
        eprintln!(
            "{}",
            theme::colored_bold(&msg, MessageKind::Warning, self.opts.plain_ui)
        );
    }

    pub fn show_hardnested_note(&self, context: Option<&str>) {
        let msg = match context {
            Some(path) => format!(
                "Note: HardNested nonces were also found in {path} and were skipped — that attack is not supported (yet)."
            ),
            None => "Note: HardNested nonces were also found in this file and were skipped — that attack is not supported (yet).".to_string(),
        };
        println!(
            "{}",
            theme::colored(&msg, MessageKind::Warning, self.opts.plain_ui)
        );
    }

    pub fn show_error(&self, text: &str) {
        eprintln!(
            "{}",
            theme::colored(text, MessageKind::Error, self.opts.plain_ui)
        );
    }

    pub fn show_interrupt(&self) {
        eprintln!("\n\nReceived interrupt signal. Stopping gracefully...");
    }

    pub fn show_detail(&self, text: &str) {
        println!("{}", theme::indent(1, text));
    }

    pub fn show_start(&self) {
        let plain = self.opts.plain_ui;
        if plain {
            println!("Starting key recovery... (Press Ctrl+C to stop gracefully.)\n");
            return;
        }
        let msg = format!("{} Starting key recovery... ", glyph::BLOCK);
        print!("{}", theme::block_style(&msg, plain));
        println!("(Press Ctrl+C to stop)");
        println!("{}", theme::light_rule());
    }

    pub fn begin_progress(&self, total_nonces: usize) {
        if self.opts.plain_ui {
            return;
        }
        let mut inner = self.inner.lock().unwrap();
        inner.total_nonces = total_nonces;
        if inner.bar.is_some() {
            return;
        }
        let pb = ProgressBar::new(total_nonces.max(1) as u64);
        let style = ProgressStyle::with_template(
            "{prefix:.bold.dim} [{bar:30.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("█▓░");
        pb.set_style(style);
        pb.set_prefix("▸ Attacking");
        pb.enable_steady_tick(Duration::from_millis(120));
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
        if self.opts.plain_ui {
            let nonce_pct = if nonce_total > 0 {
                nonce_current as f32 / nonce_total as f32 * 100.0
            } else {
                0.0
            };
            let msb_pct = if msb_total > 0 {
                msb_current as f32 / msb_total as f32 * 100.0
            } else {
                0.0
            };
            print!(
                "\rProgress: Nonce {}/{} ({:.1}%) | MSB {}/{} ({:.1}%) | Current {:.1}%   ",
                nonce_current,
                nonce_total,
                nonce_pct,
                msb_current,
                msb_total,
                msb_pct,
                stage_progress
            );
            let _ = std::io::stdout().flush();
            return;
        }

        let inner = self.inner.lock().unwrap();
        if let Some(pb) = inner.bar.as_ref() {
            pb.set_position(nonce_current.min(nonce_total) as u64);
            let msg = format!("UID 0x{:08X} | MSB {}/{}", uid, msb_current, msb_total);
            pb.set_message(msg);
        }
    }

    pub fn clear_progress(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(pb) = inner.bar.take() {
            pb.finish_and_clear();
        }
        inner.total_nonces = 0;

        if self.opts.plain_ui {
            println!();
        }
    }

    pub fn show_found_key(&self, key: &MfClassicKey) {
        let plain = self.opts.plain_ui;
        let inner = self.inner.lock().unwrap();
        let print_line = |line: String| {
            if let Some(pb) = inner.bar.as_ref() {
                pb.println(line);
            } else {
                println!("{}", line);
            }
        };

        if plain {
            print_line(format!("Found key: {}", key.to_hex()));
            return;
        }

        let line = format!("{} Found key: {}", glyph::CHECK, key.to_hex());
        print_line(theme::colored(&line, MessageKind::Success, plain));
    }

    pub fn show_summary(&self, total_nonces: usize, found_keys: usize, candidate_keys: usize) {
        self.clear_progress();
        let plain = self.opts.plain_ui;

        if plain {
            println!("\n{}", theme::heavy_rule(plain));
            println!("Key recovery completed!\n");
            println!("Summary:");
            println!("Total nonces processed: {}", total_nonces);
            println!("Keys found: {}", found_keys);
            println!("Candidate keys: {}", candidate_keys);
            return;
        }

        println!("{}", theme::heavy_rule(plain));
        let header = format!(
            "{} Key Recovery Complete! {}",
            glyph::BLOCK,
            glyph::BLOCK_END
        );
        println!("{}", theme::block_style(&header, plain));

        println!("\nSummary:");
        println!("{} Total nonces processed: {}", glyph::BULLET, total_nonces);

        let kf = format!("{} Keys found: {}", glyph::BULLET, found_keys);
        let ck = format!("{} Candidate keys: {}", glyph::BULLET, candidate_keys);
        println!("{}", theme::colored(&kf, MessageKind::Success, plain));
        println!("{}", theme::accent(&ck, plain));
    }

    pub fn show_found_keys_list(&self, keys: &[MfClassicKey]) {
        if keys.is_empty() {
            return;
        }
        let plain = self.opts.plain_ui;

        println!("\nFound Keys:");
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
            println!("{}", theme::indent(1, &line));
        }
    }

    pub fn show_saved_files(&self, keys_file: Option<&str>, keys_count: usize) {
        println!("\nFiles saved:");
        if let Some(f) = keys_file
            && keys_count > 0
        {
            let line = format!("{} {} ({} keys)", glyph::BULLET, f, keys_count);
            println!(
                "{}",
                theme::indent(
                    1,
                    &theme::colored(&line, MessageKind::Success, self.opts.plain_ui)
                )
            );
        }
    }

    pub fn show_saved_dicts(&self, dicts: &[(String, usize)]) {
        if dicts.is_empty() {
            return;
        }
        let plain = self.opts.plain_ui;

        println!("\nCandidate dictionaries ({} files):", dicts.len());
        for (path, count) in dicts {
            let filename = std::path::Path::new(path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let line = format!("{} {} ({} candidates)", glyph::BULLET, filename, count);
            println!("{}", theme::indent(1, &theme::accent(&line, plain)));
        }
    }

    pub fn show_no_keys_found(&self) {
        let plain = self.opts.plain_ui;
        if plain {
            println!("No keys were recovered. This could happen if:");
            println!("  * The nonces are invalid or corrupted");
            println!("  * The keyspace being searched doesn't contain the key");
            println!("  * The attack was interrupted before completion\n");
            return;
        }

        let line = format!("\n{} No keys were recovered.", glyph::CROSS);
        println!("{}", theme::colored(&line, MessageKind::Error, plain));
        println!("\nThis could happen if:");
        println!("  {} The nonces are invalid or corrupted", glyph::DOT);
        println!(
            "  {} The keyspace being searched doesn't contain the key",
            glyph::DOT
        );
        println!(
            "  {} The attack was interrupted before completion\n",
            glyph::DOT
        );
    }

    pub fn show_disclaimer(&self, text: &str) {
        println!("{}", theme::accent(text, self.opts.plain_ui));
    }

    pub fn show_pill_taken(&self, accepted: bool) {
        let text = if accepted {
            format!("{} Red pill taken", glyph::CHECK)
        } else {
            format!("{} Blue pill taken", glyph::CHECK)
        };
        println!(
            "{}",
            theme::colored(&text, MessageKind::Success, self.opts.plain_ui)
        );
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
