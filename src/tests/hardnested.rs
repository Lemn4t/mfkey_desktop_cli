use super::*;
use crate::ui::{OutputMode, Ui, UiOptions};
use std::sync::atomic::AtomicBool;

#[test]
#[ignore]
fn recovers_known_key_from_example_log() {
    let path = format!("{}/examples/hard_nested.log", env!("CARGO_MANIFEST_DIR"));
    let set = crate::core::parser::load_nested_nonces(&path, |_, _, _| {}).unwrap();

    assert!(set.hardnested_detected);
    assert_eq!(set.hardnested.len(), 1740);

    let ui = Ui::new(UiOptions {
        mode: OutputMode::Plain,
    });
    let stop = AtomicBool::new(false);

    let keys = HardNestedSolver::run(&set.hardnested, &stop, &ui);
    let hex: Vec<String> = keys.iter().map(|k| k.to_hex()).collect();

    assert_eq!(hex, vec!["759275927592".to_string()]);
}
