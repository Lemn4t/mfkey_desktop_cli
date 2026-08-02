use super::*;
use std::path::Path;

#[test]
fn display_path_strips_verbatim_prefix() {
    assert_eq!(
        display_path(Path::new(r"\\?\C:\Users\x\file.nfc")),
        r"C:\Users\x\file.nfc"
    );
}

#[test]
fn display_path_rewrites_verbatim_unc_prefix() {
    assert_eq!(
        display_path(Path::new(r"\\?\UNC\server\share\file.nfc")),
        r"\\server\share\file.nfc"
    );
}

#[test]
fn display_path_leaves_plain_paths_unchanged() {
    assert_eq!(
        display_path(Path::new(r"C:\Users\x\file.nfc")),
        r"C:\Users\x\file.nfc"
    );
    assert_eq!(
        display_path(Path::new("relative/file.nfc")),
        "relative/file.nfc"
    );
}
