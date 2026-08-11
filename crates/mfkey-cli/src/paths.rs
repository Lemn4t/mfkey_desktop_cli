use std::path::PathBuf;

#[cfg(unix)]
const CONFIG_DIR: &str = ".config";
const PROGRAM_DIR: &str = "mfkey";
const CONFIG_FILE: &str = "config.yaml";

#[cfg(unix)]
pub fn config_path() -> Option<PathBuf> {
    Some(
        dirs::home_dir()?
            .join(CONFIG_DIR)
            .join(PROGRAM_DIR)
            .join(CONFIG_FILE),
    )
}

#[cfg(windows)]
pub fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join(PROGRAM_DIR).join(CONFIG_FILE))
}

pub fn display_path(p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = s.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        s.into_owned()
    }
}

#[cfg(test)]
#[path = "tests/core_paths.rs"]
mod tests;
