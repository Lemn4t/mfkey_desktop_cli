mod attack;
mod ffi;
mod model;
mod parser;
mod state;
mod ui;

use crate::model::MfClassicKey;
use crate::state::AttackState;
use crate::ui::{Ui, UiOptions};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const MFKEY_VERSION: &str = env!("CARGO_PKG_VERSION");
const MFKEY_NAME: &str = env!("CARGO_PKG_NAME");

struct Args {
    input_file: String,
    output_file: String,
    dict_output_dir: Option<String>,
    no_ui: bool,
}

fn print_usage(program: &str) {
    println!(
        "{} - Flipper Zero :: MIFARE Classic Key Recovery Tool",
        MFKEY_NAME
    );
    println!("Version {}\n", MFKEY_VERSION);
    println!(
        "Usage: {} [OPTIONS] <.nested.log/.mfkey32.log> [output_keys.nfc] [dict_output_dir]\n",
        program
    );
    println!("ARGUMENTS:");
    println!(
        "  .nested.log/.mfkey32.log        Input file containing one-time values for the attack"
    );
    println!(
        "  output_keys.txt                 Output file for recovered keys (default: mf_classic_dict_user.nfc)"
    );
    println!(
        "  dict_output_dir                 Directory for candidate key dictionaries (default: current dir)\n"
    );
    println!("OPTIONS:");
    println!("  -h, --help        Show this help message and exit");
    println!("  --no-ui           Disable UI and use simple text output");
    println!("  --version         Show version information");
}

fn parse_args() -> Option<Args> {
    let argv: Vec<String> = std::env::args().collect();
    let program = argv
        .first()
        .cloned()
        .unwrap_or_else(|| MFKEY_NAME.to_string());

    for a in argv.iter().skip(1) {
        if a == "-h" || a == "--help" {
            print_usage(&program);
            return None;
        }
        if a == "--version" {
            println!("{} version {}", MFKEY_NAME, MFKEY_VERSION);
            println!("Flipper Zero :: MIFARE Classic Key Recovery Tool");
            return None;
        }
    }

    let mut positionals: Vec<String> = Vec::new();
    let mut no_ui = false;
    for a in argv.iter().skip(1) {
        if a == "--no-ui" {
            no_ui = true;
        } else {
            positionals.push(a.clone());
        }
    }

    if positionals.is_empty() {
        print_usage(&program);
        return None;
    }

    let input_file = positionals[0].clone();
    let output_file = positionals
        .get(1)
        .cloned()
        .unwrap_or_else(|| "mf_classic_dict_user.nfc".to_string());
    let dict_output_dir = positionals.get(2).cloned();

    Some(Args {
        input_file,
        output_file,
        dict_output_dir,
        no_ui,
    })
}

fn save_keys_to_file(path: &str, keys: &[MfClassicKey]) -> std::io::Result<()> {
    if keys.is_empty() {
        return Ok(());
    }
    let mut file = File::create(path)?;
    for k in keys {
        writeln!(file, "{}", k.to_hex())?;
    }
    Ok(())
}

fn save_candidate_dict(uid: u32, keys: &[(u8, MfClassicKey)], output_dir: Option<&str>) -> String {
    let filename = format!("mf_classic_dict_{:08x}.nfc", uid);
    let path = match output_dir {
        Some(dir) => Path::new(dir).join(&filename),
        None => Path::new(&filename).to_path_buf(),
    };
    let path_str = path.to_string_lossy().to_string();

    if let Ok(mut file) = File::create(&path) {
        for (key_idx, k) in keys {
            let _ = writeln!(file, "{:02X}{}", key_idx, k.to_hex());
        }
    } else {
        eprintln!("Failed to create dictionary file: {}", path_str);
    }

    path_str
}

fn main() {
    let args = match parse_args() {
        Some(a) => a,
        None => process::exit(0),
    };

    let ui_opts = UiOptions {
        no_ui: args.no_ui,
        use_colors: !args.no_ui,
    };
    let ui = Arc::new(Ui::new(ui_opts));

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop_clone = Arc::clone(&stop);
        if let Err(e) = ctrlc::set_handler(move || {
            eprintln!("\n\nReceived interrupt signal. Stopping attack gracefully...");
            stop_clone.store(true, Ordering::SeqCst);
        }) {
            eprintln!("Warning: failed to set Ctrl+C handler: {}", e);
        }
    }

    ui.show_title();
    ui.show_config(
        &args.input_file,
        &args.output_file,
        args.dict_output_dir.as_deref(),
    );

    ui.show_loading(&args.input_file);

    let ui_for_load = Arc::clone(&ui);
    let nonces = match parser::load_nested_nonces(&args.input_file, |idx, uid, name| {
        ui_for_load.show_nonce_loaded(idx, uid, name);
    }) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to open file: {} ({})", args.input_file, e);
            process::exit(1);
        }
    };

    if nonces.is_empty() {
        eprintln!("Failed to load nonces from file!");
        process::exit(1);
    }

    ui.show_loading_complete(nonces.len());

    ui.show_start();

    let mut attack_state = AttackState::new(Arc::clone(&ui), Arc::clone(&stop), nonces.len());

    let mut save_dict = |uid: u32, keys: &[(u8, MfClassicKey)], dir: Option<&str>| -> String {
        save_candidate_dict(uid, keys, dir)
    };

    let (candidate_total_count, dict_outputs) = attack::run_attack(
        &mut attack_state,
        &nonces,
        args.dict_output_dir.as_deref(),
        &mut save_dict,
    );

    let found_count = attack_state.found_keys.len();
    ui.show_summary(nonces.len(), found_count, candidate_total_count);

    if found_count > 0 {
        ui.show_found_keys_list(&attack_state.found_keys);
        if let Err(e) = save_keys_to_file(&args.output_file, &attack_state.found_keys) {
            eprintln!("Failed to create output file: {} ({})", args.output_file, e);
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

    if !args.no_ui && (found_count > 0 || candidate_total_count > 0) {
        let confirmed = ui.confirm("Should I show the full path to the saved files?", false);

        if confirmed {
            if let Some(kf) = keys_file {
                if let Ok(abs) = std::fs::canonicalize(kf) {
                    println!("  keys: {}", abs.display());
                }
            }
            for d in &dict_outputs {
                if let Ok(abs) = std::fs::canonicalize(&d.path) {
                    println!("  dict (uid 0x{:08X}): {}", d.uid, abs.display());
                }
            }
        }
    }
}
