use std::path::PathBuf;

#[cfg(unix)]
const CONFIG_DIR: &str = ".config";
const PROGRAM_DIR: &str = "mfkey";
const CONFIG_FILE: &str = "config.yaml";

#[cfg(unix)]
pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir")
        .join(CONFIG_DIR)
        .join(PROGRAM_DIR)
        .join(CONFIG_FILE)
}

#[cfg(windows)]
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .expect("no AppData/Roaming")
        .join(PROGRAM_DIR)
        .join(CONFIG_FILE)
}
