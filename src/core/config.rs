use crate::core::paths::config_path;
use crate::ext::option::OptionExt;
use crate::ext::result::{ResultExt, Rslt};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub disclaimer_accepted: bool,
}

impl Config {
    pub fn read() -> Config {
        config_path()
            .and_then(|p| File::open(p).ok())
            .and_then(|file| yaml_serde::from_reader(file).ok())
            .unwrap_or_default()
    }

    fn store(&self) -> Rslt<()> {
        let config_path = config_path().or_err("no config directory available")?;
        if !config_path.exists() {
            let parent = config_path.parent().or_err("No parent directory")?;
            fs::create_dir_all(parent)?;
        }
        let file = File::create(&config_path)?;
        yaml_serde::to_writer(file, self).boxed()
    }

    pub fn accept_disclaimer_and_store(&mut self) -> Rslt<()> {
        self.disclaimer_accepted = true;
        self.store()
    }
}
