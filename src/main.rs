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
    let params = params::parse();
    let opts = UiOptions {
        plain_ui: ui::should_use_plain_mode(params.plain_ui()),
    };
    let ui = Ui::new(opts);
    if !resolve_disclaimer_acceptance(&ui, params.accept_disclaimer()) {
        return;
    }
    let ui = Arc::new(ui);
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let handler_ui = ui.clone();
    {
        if let Err(e) = ctrlc::set_handler(move || {
            handler_ui.show_interrupt();
            stop_clone.store(true, Ordering::SeqCst);
        }) {
            ui.show_error(&format!("Warning: failed to set Ctrl+C handler: {e}"));
        }
    }
    ui.show_title();

    match params {
        Params::Auto(params) => match auto::run_auto(ui.clone(), params, stop) {
            Ok(()) => process::exit(0),
            Err(e) => {
                ui.show_error(&format!("AUTO failed: {e}"));
                process::exit(1);
            }
        },
        Params::Run(params) => run(ui, params, stop),
    }
}

fn run(ui: Arc<Ui>, params: RunParams, stop: Arc<AtomicBool>) {
    ui.show_config(
        &params.input_file,
        &params.output_file,
        params.dict_output_dir.as_deref(),
    );

    let outcome = match attack_runner::run_file_attack(
        &ui,
        &stop,
        &params.input_file,
        params.dict_output_dir.as_deref(),
        None,
        false,
    ) {
        Ok(o) => o,
        Err(e) => {
            ui.show_error(&format!(
                "Failed to open file: {} ({})",
                params.input_file, e
            ));
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
        if let Err(e) = save_keys_to_file(&params.output_file, &result.found_keys) {
            ui.show_error(&format!(
                "Failed to create output file: {} ({})",
                params.output_file, e
            ));
        }
    }

    let keys_file = if found_count > 0 {
        Some(params.output_file.as_str())
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

    if !ui.is_plain() && (found_count > 0 || candidate_total_count > 0) {
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
