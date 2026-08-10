mod theme;

mod attack_report;
mod auto_report;

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

pub struct Ui {
    opts: UiOptions,
    inner: Mutex<UiInner>,
}

impl Ui {
    pub fn new(opts: UiOptions) -> Self {
        Ui {
            opts,
            inner: Mutex::new(UiInner { bar: None }),
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

    fn status_line(&self, text: &str, kind: MessageKind) {
        self.write_line(&theme::colored(text, kind, self.is_plain()));
    }

    fn with_glyph(&self, glyph: &str, text: &str) -> String {
        if self.is_plain() {
            text.to_string()
        } else {
            format!("{glyph} {text}")
        }
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

    pub fn show_error(&self, text: &str) {
        self.write_err_line(&theme::colored(text, MessageKind::Error, self.is_plain()));
    }

    pub fn show_interrupt(&self) {
        self.write_err_line("\n\nReceived interrupt signal. Stopping gracefully...");
    }

    pub fn show_detail(&self, text: &str) {
        self.write_line(&theme::indent(1, text));
    }

    pub fn begin_progress(&self, total_nonces: usize) {
        let mut inner = self.inner.lock().unwrap();
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

impl crate::core::reporter::Reporter for Ui {
    fn begin_progress(&self, total_nonces: usize) {
        Ui::begin_progress(self, total_nonces);
    }

    fn update_progress(
        &self,
        nonce_current: usize,
        nonce_total: usize,
        msb_current: usize,
        msb_total: usize,
        stage_progress: f32,
        uid: u32,
    ) {
        Ui::update_progress(
            self,
            nonce_current,
            nonce_total,
            msb_current,
            msb_total,
            stage_progress,
            uid,
        );
    }

    fn found_key(&self, key: &crate::core::model::MfClassicKey) {
        self.show_found_key(key);
    }
}

#[cfg(test)]
#[path = "../tests/ui_mod.rs"]
mod tests;
