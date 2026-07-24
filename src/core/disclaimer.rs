use crate::core::config::Config;
use crate::ext::printing::PrintExt;
use crate::ext::result::ResultExt;
use crate::ui::UiOptions;
use colored::Color::{Blue, Green, Red, Yellow};
use colored::{Color, Colorize};
use std::io;

pub fn resolve_disclaimer_acceptance(opts: &UiOptions, force_accept: bool) -> bool {
    let mut config = Config::read();
    if config.disclaimer_accepted {
        return true;
    }
    if force_accept {
        config.accept_disclaimer_and_store().print_err();
        return true;
    }
    disclaimer(opts).println();
    let mut input = String::new();
    let result = loop {
        terms_acceptance_request(opts).print();
        let answer = io::stdin()
            .read_line(&mut input)
            .map(|_| input.trim())
            .unwrap_or("_");
        match answer {
            "y" | "Y" | "yes" | "YES" => {
                config.accept_disclaimer_and_store().print_err();
                break true;
            }
            "" | "n" | "N" | "no" | "NO" => break false,
            _ => {
                invalid_answer(opts).println();
                input.clear();
            }
        }
    };
    match result {
        true => red_pill_was_taken(opts).println(),
        false => blue_pill_was_taken(opts).println(),
    }
    result
}

pub fn disclaimer(opts: &UiOptions) -> String {
    let text = "\
+----------------------------------------------------------------------+
|                             DISCLAIMER                               |
+----------------------------------------------------------------------+
| This tool is intended exclusively for security research, education,  |
| and testing on cards you own or have explicit permission to analyze. |
| The author is not responsible for any misuse. Use at your own risk   |
| and in accordance with the laws of your jurisdiction.                |
+----------------------------------------------------------------------+";
    resolve(opts, Yellow, text)
}

fn terms_acceptance_request(opts: &UiOptions) -> String {
    resolve(opts, Yellow, "▸ Do you accept the terms of use? (y/N): ")
}

fn invalid_answer(opts: &UiOptions) -> String {
    resolve(opts, Red, "✗ Invalid answer")
}

fn blue_pill_was_taken(opts: &UiOptions) -> String {
    pill_was_taken(opts, resolve(opts, Blue, "Blue pill"))
}

fn red_pill_was_taken(opts: &UiOptions) -> String {
    pill_was_taken(opts, resolve(opts, Red, "Red pill"))
}

fn pill_was_taken(opts: &UiOptions, pill: String) -> String {
    let text = format!("✓ {} taken", pill);
    resolve(opts, Green, text.as_str())
}

fn resolve(opts: &UiOptions, color: Color, text: &str) -> String {
    match opts.plain_ui {
        true => text.clear(),
        false => text.color(color),
    }
    .to_string()
}
