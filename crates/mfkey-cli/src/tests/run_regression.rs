use super::*;
use crate::ui::{OutputMode, Ui, UiOptions};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

fn example(name: &str) -> String {
    format!("{}/../../examples/{}", env!("CARGO_MANIFEST_DIR"), name)
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mfkey_run_{}_{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn plain_ui() -> Arc<Ui> {
    Arc::new(Ui::new(UiOptions {
        mode: OutputMode::Plain,
    }))
}

fn has_dictionary(dir: &std::path::Path) -> bool {
    fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("mf_classic_dict_")
        })
}

#[test]
fn run_saves_found_keys_for_mfkey32() {
    let dir = temp_dir("mfkey32");
    let out = dir.join("keys.nfc");
    let params = RunParams {
        input_file: example("mfkey32.log"),
        output_file: out.to_string_lossy().to_string(),
        dict_output_dir: Some(dir.to_string_lossy().to_string()),
        plain_ui: true,
        accept_disclaimer: true,
    };

    run(plain_ui(), params, Arc::new(AtomicBool::new(false))).unwrap();

    assert_eq!(fs::read_to_string(&out).unwrap(), "A0A1A2A3A4A5\n");
    assert!(!has_dictionary(&dir));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn run_saves_dictionary_without_keys_file_for_static_encrypted() {
    let dir = temp_dir("static_encrypted");
    let out = dir.join("keys.nfc");
    let params = RunParams {
        input_file: example("static_encrypted.log"),
        output_file: out.to_string_lossy().to_string(),
        dict_output_dir: Some(dir.to_string_lossy().to_string()),
        plain_ui: true,
        accept_disclaimer: true,
    };

    run(plain_ui(), params, Arc::new(AtomicBool::new(false))).unwrap();

    assert!(!out.exists());
    let dict = dir.join("mf_classic_dict_929d09de.nfc");
    assert!(dict.exists());
    let bytes = fs::read(&dict).unwrap();
    assert_eq!(bytes.iter().filter(|&&b| b == b'\n').count(), 1_043_338);

    let _ = fs::remove_dir_all(&dir);
}
