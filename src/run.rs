use crate::core::attack_runner::{self, FileAttackOutcome};
use crate::core::model::save_keys_to_file;
use crate::core::paths::display_path;
use crate::ext::result::Rslt;
use crate::params::RunParams;
use crate::ui::Ui;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub fn run(ui: Arc<Ui>, params: RunParams, stop: Arc<AtomicBool>) -> Rslt<()> {
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
            return Err(e);
        }
    };

    let result = match outcome {
        FileAttackOutcome::NoUsableNonces {
            hardnested_detected,
        } => {
            if !hardnested_detected {
                ui.show_error("Failed to load nonces from file!");
            }
            return Err("no usable nonces".into());
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
        let dicts: Vec<(&str, usize)> = dict_outputs
            .iter()
            .map(|d| (d.path.as_str(), d.count))
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

    Ok(())
}
