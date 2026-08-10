use super::*;
use crate::ui::{OutputMode, Ui, UiOptions};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

fn example(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/examples/{}", env!("CARGO_MANIFEST_DIR"), name))
}

fn plain_ui() -> Arc<Ui> {
    Arc::new(Ui::new(UiOptions {
        mode: OutputMode::Plain,
    }))
}

#[test]
fn aggregates_keys_and_skips_hardnested() {
    let dir = std::env::temp_dir().join(format!("mfkey_auto_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let logs = [
        example("mfkey32.log"),
        example("static_nested.log"),
        example("hard_nested.log"),
    ];
    let ui = plain_ui();
    let stop = Arc::new(AtomicBool::new(false));

    let (all_keys, local_dicts) = run_attacks_over_logs(&ui, &stop, &logs, &dir);

    let keys: Vec<String> = all_keys.into_iter().collect();
    assert_eq!(
        keys,
        vec!["112244556600".to_string(), "A0A1A2A3A4A5".to_string()]
    );
    assert!(local_dicts.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}
