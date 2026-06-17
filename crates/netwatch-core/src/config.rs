use std::fs;
use std::path::{Path, PathBuf};

use glob::Pattern;
use serde::{Deserialize, Serialize};

use crate::error::{NetWatchError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    Auto,
    Bytes,
    Bits,
}

impl Default for Units {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_sample_interval")]
    pub sample_interval: u64,

    #[serde(default = "default_database")]
    pub database: String,

    #[serde(default = "default_ignore")]
    pub ignore: Vec<String>,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default)]
    pub units: Units,

    #[serde(default = "default_history_days")]
    pub history_days: u32,

    #[serde(default = "default_batch_write_interval")]
    pub batch_write_interval: u64,

    #[serde(default = "default_skip_loopback")]
    pub skip_loopback: bool,
}

fn default_sample_interval() -> u64 {
    1
}

fn default_database() -> String {
    "~/.local/share/netwatch/netwatch.db".into()
}

fn default_ignore() -> Vec<String> {
    vec![
        "docker*".into(),
        "virbr*".into(),
        "veth*".into(),
    ]
}

fn default_theme() -> String {
    "default".into()
}

fn default_history_days() -> u32 {
    365
}

fn default_batch_write_interval() -> u64 {
    5
}

fn default_skip_loopback() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sample_interval: default_sample_interval(),
            database: default_database(),
            ignore: default_ignore(),
            theme: default_theme(),
            units: Units::default(),
            history_days: default_history_days(),
            batch_write_interval: default_batch_write_interval(),
            skip_loopback: default_skip_loopback(),
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let path = path
            .map(PathBuf::from)
            .unwrap_or_else(default_config_path);

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let mut config: Config = toml::from_str(&contents)
                .map_err(|e| NetWatchError::Config(e.to_string()))?;
            config.validate()?;
            Ok(config)
        } else {
            let mut config = Config::default();
            config.validate()?;
            Ok(config)

}}}
