mod auto;
mod core;
pub mod ext;
mod flipper;
mod generated_pb;
mod params;
mod ui;

pub use generated_pb::*;

use crate::core::attack_runner::{self, FileAttackOutcome};
use crate::core::disclaimer::resolve_disclaimer_acceptance;
use crate::core::model::MfClassicKey;
use crate::ext::result::Rslt;
use crate::params::{Params, RunParams};
use crate::ui::{Ui, UiOptions};
use std::fs::File;
use std::io::Write;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn save_keys_to_file(path: &str, keys: &[MfClassicKey]) -> Rslt<()> {
    if keys.is_empty() {
        return Ok(());
    }
    let mut file = File::create(path)?;
    for k in keys {
        writeln!(file, "{}", k.to_hex())?;
    }
    Ok(())
}

fn display_path(p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = s.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        s.into_owned()
    }
}

fn main() {
    match params::parse() {
        Params::Auto(auto_params) => {
            let plain_ui = auto_params.plain_ui;
            match auto::run_auto(
                auto_params.port.as_deref(),
                auto_params.out_dir.as_deref(),
                plain_ui,
                auto_params.accept_disclaimer,
            ) {
                Ok(()) => process::exit(0),
                Err(e) => {
                    let ui = Ui::new(UiOptions { plain_ui });
                    ui.show_error(&format!("AUTO failed: {e}"));
                    process::exit(1);
                }
            }
        }
        Params::Run(args) => run(args),
    }
}

fn run(args: RunParams) {
    let ui_opts = UiOptions {
        plain_ui: args.plain_ui,
    };
    let ui = Arc::new(Ui::new(ui_opts));
    if !resolve_disclaimer_acceptance(&ui, args.accept_disclaimer) {
        return;
    }
    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop_clone = Arc::clone(&stop);
        let ui_for_ctrlc = Arc::clone(&ui);
        if let Err(e) = ctrlc::set_handler(move || {
            ui_for_ctrlc.show_interrupt();
            stop_clone.store(true, Ordering::SeqCst);
        }) {
            ui.show_error(&format!("Warning: failed to set Ctrl+C handler: {e}"));
        }
    }

    ui.show_title();
    ui.show_config(
        &args.input_file,
        &args.output_file,
        args.dict_output_dir.as_deref(),
    );

    let outcome = match attack_runner::run_file_attack(
        &ui,
        &stop,
        &args.input_file,
        args.dict_output_dir.as_deref(),
        None,
        false,
    ) {
        Ok(o) => o,
        Err(e) => {
            ui.show_error(&format!("Failed to open file: {} ({})", args.input_file, e));
            process::exit(1);
        }
    };

    let result = match outcome {
        FileAttackOutcome::NoUsableNonces {
            hardnested_detected,
        } => {
            if !hardnested_detected {
                ui.show_error("Failed to load nonces from file!");
            }
            process::exit(1);
        }
        FileAttackOutcome::Ran(r) => r,
    };

    let found_count = result.found_keys.len();
    let candidate_total_count = result.candidate_total_count;
    let dict_outputs = result.dict_outputs;

    if found_count > 0 {
        ui.show_found_keys_list(&result.found_keys);
        if let Err(e) = save_keys_to_file(&args.output_file, &result.found_keys) {
            ui.show_error(&format!(
                "Failed to create output file: {} ({})",
                args.output_file, e
            ));
        }
    }

    let keys_file = if found_count > 0 {
        Some(args.output_file.as_str())
    } else {
        None
    };
    ui.show_saved_files(keys_file, found_count);

    if !dict_outputs.is_empty() {
        let dicts: Vec<(String, usize)> = dict_outputs
            .iter()
            .map(|d| (d.path.clone(), d.count))
            .collect();
        ui.show_saved_dicts(&dicts);
    }

    if found_count == 0 && candidate_total_count == 0 {
        ui.show_no_keys_found();
    }

    if !args.plain_ui && (found_count > 0 || candidate_total_count > 0) {
        let confirmed = ui.confirm("Should I show the full path to the saved files?", false);

        if confirmed {
            if let Some(kf) = keys_file
                && let Ok(abs) = std::fs::canonicalize(kf)
            {
                ui.show_detail(&format!("keys: {}", display_path(&abs)));
            }
            for d in &dict_outputs {
                if let Ok(abs) = std::fs::canonicalize(&d.path) {
                    ui.show_detail(&format!(
                        "dict (uid 0x{:08X}): {}",
                        d.uid,
                        display_path(&abs)
                    ));
                }
            }
        }
    }
}
