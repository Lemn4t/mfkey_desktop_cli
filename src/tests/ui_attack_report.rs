use super::*;
use crate::core::model::MfClassicKey;

fn key(bytes: [u8; 6]) -> MfClassicKey {
    MfClassicKey::from_slice(&bytes)
}

fn force_color() {
    colored::control::set_override(true);
}

#[test]
fn hardnested_result_plain_shows_key_or_not_found() {
    let k = key([0x75, 0x92, 0x75, 0x92, 0x75, 0x92]);
    let ok = render_hardnested_result(
        OutputMode::Plain,
        "UID 0xDA505F80 sector 0 key A",
        Some(&k),
    );
    assert!(ok.contains("759275927592"));
    assert!(ok.contains("sector 0"));

    let no = render_hardnested_result(OutputMode::Plain, "UID 0x00000000 sector 1 key B", None);
    assert!(no.to_lowercase().contains("no key found"));
    assert!(no.contains("sector 1"));
}

#[test]
fn unrecognized_lines_plain_mentions_count() {
    let s = render_unrecognized_lines(OutputMode::Plain, 3);
    assert!(s.contains("3 unrecognized line(s)"));
}

#[test]
fn hardnested_loaded_plain_lists_count_and_targets() {
    let lines = render_hardnested_loaded(OutputMode::Plain, 1740, &[(0xDA50_5F80, 0)]);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "Detected 1740 HardNested nonce(s) — 1 target(s)");
    assert!(lines[1].contains("UID 0xDA505F80"));
    assert!(lines[1].contains("sector 0"));
    assert!(lines[1].contains("key A"));
}

#[test]
fn hardnested_loaded_maps_key_idx_to_sector_and_key_type() {
    let lines = render_hardnested_loaded(OutputMode::Plain, 4, &[(1, 3)]);
    assert!(lines[1].contains("sector 1"));
    assert!(lines[1].contains("key B"));
}

#[test]
fn config_plain_has_exact_labels_no_dict_dir() {
    let lines = render_config(OutputMode::Plain, "in.log", "out.txt", None);
    assert_eq!(lines[0], "Input file:  in.log");
    assert_eq!(lines[1], "Output file: out.txt");
    assert!(lines.iter().all(|l| !l.contains('\x1b')));
    assert!(!lines.iter().any(|l| l.contains("Dict")));
}

#[test]
fn config_plain_includes_dict_dir_when_given() {
    let lines = render_config(OutputMode::Plain, "in.log", "out.txt", Some("dicts/"));
    assert!(lines.iter().any(|l| l == "Dict output dir: dicts/"));
}

#[test]
fn config_fancy_uses_bullet_glyph() {
    let lines = render_config(OutputMode::Fancy, "in.log", "out.txt", None);
    assert!(lines[0].contains(glyph::BULLET));
    assert!(lines[0].contains("in.log"));
}

#[test]
fn nonce_loaded_plain_has_no_tag() {
    let line = render_nonce_loaded(OutputMode::Plain, 3, 0xDEADBEEF, "mfkey32");
    assert_eq!(line, "Loaded nonce 3: UID=0xDEADBEEF, attack=mfkey32");
}

#[test]
fn nonce_loaded_fancy_tags_static_encrypted_differently() {
    force_color();
    let normal = render_nonce_loaded(OutputMode::Fancy, 0, 1, "mfkey32");
    let static_enc = render_nonce_loaded(OutputMode::Fancy, 0, 1, "static_encrypted");
    assert!(normal.contains('\x1b'));
    assert!(static_enc.contains('\x1b'));
    assert_ne!(normal, static_enc);
}

#[test]
fn loading_complete_plain_matches_exact_text() {
    assert_eq!(
        render_loading_complete(OutputMode::Plain, 42),
        "Total nonces loaded: 42\n"
    );
}


#[test]
fn start_plain_is_single_line() {
    let lines = render_start(OutputMode::Plain);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("Ctrl+C to stop gracefully"));
}

#[test]
fn start_fancy_has_two_lines_with_rule() {
    let lines = render_start(OutputMode::Fancy);
    assert_eq!(lines.len(), 2);
    assert!(lines[1].chars().all(|c| c == '─'));
}

#[test]
fn summary_plain_matches_exact_counts() {
    let lines = render_summary(OutputMode::Plain, 10, 2, 5);
    assert!(lines.iter().any(|l| l == "Total nonces processed: 10"));
    assert!(lines.iter().any(|l| l == "Keys found: 2"));
    assert!(lines.iter().any(|l| l == "Candidate keys: 5"));
}

#[test]
fn summary_fancy_and_plain_have_same_line_count() {
    assert_eq!(
        render_summary(OutputMode::Fancy, 10, 2, 5).len(),
        render_summary(OutputMode::Plain, 10, 2, 5).len()
    );
}

#[test]
fn found_keys_list_empty_is_empty() {
    assert!(render_found_keys_list(OutputMode::Plain, &[]).is_empty());
    assert!(render_found_keys_list(OutputMode::Fancy, &[]).is_empty());
}

#[test]
fn found_keys_list_plain_shows_hex_no_ansi() {
    let keys = [key([0xFF; 6])];
    let lines = render_found_keys_list(OutputMode::Plain, &keys);
    assert_eq!(lines.len(), 2);
    assert!(lines[1].contains("FFFFFFFFFFFF"));
    assert!(!lines[1].contains('\x1b'));
}

#[test]
fn saved_files_always_has_header_even_when_empty() {
    let lines = render_saved_files(OutputMode::Plain, None, 0);
    assert_eq!(lines, vec!["\nFiles saved:".to_string()]);
}

#[test]
fn saved_files_adds_line_when_present_and_nonzero() {
    let lines = render_saved_files(OutputMode::Plain, Some("keys.txt"), 3);
    assert_eq!(lines.len(), 2);
    assert!(lines[1].contains("keys.txt"));
    assert!(lines[1].contains('3'));
}

#[test]
fn saved_files_skips_line_when_count_zero() {
    let lines = render_saved_files(OutputMode::Plain, Some("keys.txt"), 0);
    assert_eq!(lines.len(), 1);
}

#[test]
fn saved_dicts_empty_is_empty() {
    assert!(render_saved_dicts(OutputMode::Plain, &[]).is_empty());
}

#[test]
fn saved_dicts_uses_file_name_only() {
    let dicts = vec![("/some/path/dict.txt", 7usize)];
    let lines = render_saved_dicts(OutputMode::Plain, &dicts);
    assert_eq!(lines.len(), 2);
    assert!(lines[1].contains("dict.txt"));
    assert!(!lines[1].contains("/some/path"));
}

#[test]
fn no_keys_found_plain_has_four_lines() {
    let lines = render_no_keys_found(OutputMode::Plain);
    assert_eq!(lines.len(), 4);
    assert!(lines.iter().all(|l| !l.contains('\x1b')));
}

#[test]
fn no_keys_found_fancy_has_five_lines_with_ansi() {
    force_color();
    let lines = render_no_keys_found(OutputMode::Fancy);
    assert_eq!(lines.len(), 5);
    assert!(lines.iter().any(|l| l.contains('\x1b')));
}
