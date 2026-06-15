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

}
