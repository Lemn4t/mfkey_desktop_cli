use crate::core::model::MfClassicKey;
use colored::*;
use console::Term;
use dialoguer::{Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::sync::Mutex;
use std::time::Duration;

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

    fn colored(&self) -> bool {
        !self.opts.plain_ui
    }

    pub fn show_title(&self) {
        if self.opts.plain_ui {
            println!("MIFARE Classic Key Recovery Tool");
            println!("{}", "=".repeat(64));
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

        if self.colored() {
            println!("{}", art.cyan().bold());
            println!(
                "{}",
                "░▒▓█ Flipper Zero :: MIFARE Classic Key Recovery Tool █▓▒░".magenta()
            );
        } else {
            println!("{}", art);
            println!("░▒▓█ Flipper Zero :: MIFARE Classic Key Recovery Tool █▓▒░");
        }
        println!("{}\n", "═".repeat(64));
    }

    pub fn show_config(&self, input: &str, output: &str, dict_dir: Option<&str>) {
        if self.opts.plain_ui {
            println!("Input file:  {}", input);
            println!("Output file: {}", output);
            if let Some(d) = dict_dir {
                println!("Dict output dir: {}", d);
            }
            println!("{}\n", "=".repeat(64));
            return;
        }

        let lines = [
            format!("▸ Input file:  {}", input),
            format!("▸ Output file: {}", output),
        ];
        for l in &lines {
            if self.colored() {
                println!("{}", l.yellow());
            } else {
                println!("{}", l);
            }
        }
        if let Some(d) = dict_dir {
            let l = format!("▸ Dict dir:    {}", d);
            if self.colored() {
                println!("{}", l.yellow());
            } else {
                println!("{}", l);
            }
        }
        println!();
    }

    pub fn show_loading(&self, filename: &str) {
        if self.opts.plain_ui {
            println!("Loading nonces from {}...", filename);
        } else {
            println!("▪▫▪ Loading nonces from {}...", filename);
        }
    }

    pub fn show_status(&self, text: &str, color: Color) {
        if self.colored() {
            println!("{}", text.color(color));
        } else {
            println!("{}", text);
        }
    }

    pub fn show_status_detail(&self, prefix: &str, detail: &str, color: Color, bold: bool) {
        if self.colored() {
            let p = if bold {
                prefix.color(color).bold()
            } else {
                prefix.color(color)
            };
            println!("{} {}", p, detail);
        } else {
            println!("{} {}", prefix, detail);
        }
    }

    pub fn show_status_value_bold(&self, prefix: &str, value: &str, color: Color) {
        if self.colored() {
            println!("{} {}", prefix.color(color), value.bold());
        } else {
            println!("{} {}", prefix, value);
        }
    }

    pub fn show_detail_dimmed(&self, prefix: &str, detail: &str) {
        if self.colored() {
            println!("  {} {}", prefix.dimmed(), detail);
        } else {
            println!("  {} {}", prefix, detail);
        }
    }

    pub fn show_nonce_loaded(&self, index: usize, uid: u32, attack_type: &str) {
        if self.opts.plain_ui {
            println!(
                "Loaded nonce {}: UID=0x{:08X}, attack={}",
                index, uid, attack_type
            );
            return;
        }

        let tag = if self.colored() {
            if attack_type == "static_encrypted" {
                format!("[{}]", attack_type).yellow().to_string()
            } else {
                format!("[{}]", attack_type).green().to_string()
            }
        } else {
            format!("[{}]", attack_type)
        };
        println!("  └─ Loaded nonce {}: UID=0x{:08X} {}", index, uid, tag);
    }

    pub fn show_loading_complete(&self, total: usize) {
        if self.opts.plain_ui {
            println!("Total nonces loaded: {}\n", total);
        } else {
            let n = if self.colored() {
                total.to_string().bold().to_string()
            } else {
                total.to_string()
            };
            println!("  └─ Total nonces loaded: {}\n", n);
        }
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
        if self.opts.plain_ui {
            eprintln!("{}", msg);
        } else {
            eprintln!("{}", msg.yellow().bold());
        }
    }

    pub fn show_hardnested_note(&self, context: Option<&str>) {
        let msg = match context {
            Some(path) => format!(
                "Note: HardNested nonces were also found in {path} and were skipped — that attack is not supported (yet)."
            ),
            None => "Note: HardNested nonces were also found in this file and were skipped — that attack is not supported (yet).".to_string(),
        };
        if self.opts.plain_ui {
            println!("{}", msg);
        } else {
            println!("{}", msg.yellow());
        }
    }

    pub fn show_start(&self) {
        if self.opts.plain_ui {
            println!("Starting key recovery... (Press Ctrl+C to stop gracefully.)\n");
            return;
        }
        let msg = "▓▒░ Starting key recovery... ";
        if self.colored() {
            print!("{}", msg.magenta());
        } else {
            print!("{}", msg);
        }
        println!("(Press Ctrl+C to stop)");
        println!("{}", "─".repeat(64));
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
            use std::io::Write;
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
        let inner = self.inner.lock().unwrap();
        let print_line = |line: String| {
            if let Some(pb) = inner.bar.as_ref() {
                pb.println(line);
            } else {
                println!("{}", line);
            }
        };

        if self.opts.plain_ui {
            print_line(format!("Found key: {}", key.to_hex()));
            return;
        }

        let line = format!("✓ Found key: {}", key.to_hex());
        if self.colored() {
            print_line(line.green().to_string());
        } else {
            print_line(line);
        }
    }

    pub fn show_summary(&self, total_nonces: usize, found_keys: usize, candidate_keys: usize) {
        self.clear_progress();

        if self.opts.plain_ui {
            println!("\n{}", "=".repeat(64));
            println!("Key recovery completed!\n");
            println!("Summary:");
            println!("Total nonces processed: {}", total_nonces);
            println!("Keys found: {}", found_keys);
            println!("Candidate keys: {}", candidate_keys);
            return;
        }

        println!("{}", "═".repeat(64));
        let header = "▓▒░ Key Recovery Complete! ░▒▓";
        if self.colored() {
            println!("{}", header.magenta());
        } else {
            println!("{}", header);
        }

        println!("\nSummary:");
        println!("▸ Total nonces processed: {}", total_nonces);

        let kf = format!("▸ Keys found: {}", found_keys);
        let ck = format!("▸ Candidate keys: {}", candidate_keys);
        if self.colored() {
            println!("{}", kf.green());
            println!("{}", ck.yellow());
        } else {
            println!("{}", kf);
            println!("{}", ck);
        }
    }

    pub fn show_found_keys_list(&self, keys: &[MfClassicKey]) {
        if keys.is_empty() {
            return;
        }

        if self.opts.plain_ui {
            println!("\nFound Keys:");
            for k in keys {
                println!("  {}", k.to_hex());
            }
            return;
        }

        println!("\nFound Keys:");
        for k in keys {
            let line = format!("  ✓ {}", k.to_hex());
            if self.colored() {
                println!("{}", line.green());
            } else {
                println!("{}", line);
            }
        }
    }

    pub fn show_saved_files(&self, keys_file: Option<&str>, keys_count: usize) {
        println!("\nFiles saved:");
        if let Some(f) = keys_file
            && keys_count > 0
        {
            let line = format!("  ▸ {} ({} keys)", f, keys_count);
            if self.colored() {
                println!("{}", line.green());
            } else {
                println!("{}", line);
            }
        }
    }

    pub fn show_saved_dicts(&self, dicts: &[(String, usize)]) {
        if dicts.is_empty() {
            return;
        }

        println!("\nCandidate dictionaries ({} files):", dicts.len());
        for (path, count) in dicts {
            let filename = std::path::Path::new(path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let line = format!("  ▸ {} ({} candidates)", filename, count);
            if self.colored() {
                println!("{}", line.yellow());
            } else {
                println!("{}", line);
            }
        }
    }

    pub fn show_no_keys_found(&self) {
        if self.opts.plain_ui {
            println!("No keys were recovered. This could happen if:");
            println!("  * The nonces are invalid or corrupted");
            println!("  * The keyspace being searched doesn't contain the key");
            println!("  * The attack was interrupted before completion\n");
            return;
        }

        let line = "\n✗ No keys were recovered.";
        if self.colored() {
            println!("{}", line.red());
        } else {
            println!("{}", line);
        }
        println!("\nThis could happen if:");
        println!("  • The nonces are invalid or corrupted");
        println!("  • The keyspace being searched doesn't contain the key");
        println!("  • The attack was interrupted before completion\n");
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
