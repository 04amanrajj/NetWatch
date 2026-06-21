use std::fs;
use std::path::Path;

use chrono::Utc;
use netwatch_core::{Config, InterfaceSnapshot, OperState, Result};

pub fn collect_from_proc(config: &Config, proc_path: &Path) -> Result<Vec<InterfaceSnapshot>> {
    let contents = fs::read_to_string(proc_path).map_err(|e| {
        netwatch_core::NetWatchError::Collection(format!(
            "failed to read {}: {e}",
            proc_path.display()
        ))
    })?;

    let mut snapshots = Vec::new();
    for line in contents.lines().skip(2) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (name, rest) = line
            .split_once(':')
            .ok_or_else(|| netwatch_core::NetWatchError::Collection("invalid /proc/net/dev line".into()))?;
        let name = name.trim();
        if config.should_ignore(name)? {

}}}
