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
        let (name, rest) = line.split_once(':').ok_or_else(|| {
            netwatch_core::NetWatchError::Collection("invalid /proc/net/dev line".into())
        })?;
        let name = name.trim();
        if config.should_ignore(name)? {
            continue;
        }
        let fields: Vec<&str> = rest.split_whitespace().collect();
        if fields.len() < 16 {
            continue;
        }
        let rx_bytes: u64 = fields[0].parse().map_err(|_| {
            netwatch_core::NetWatchError::Collection(format!("invalid rx_bytes for {name}"))
        })?;
        let tx_bytes: u64 = fields[8].parse().map_err(|_| {
            netwatch_core::NetWatchError::Collection(format!("invalid tx_bytes for {name}"))
        })?;

        snapshots.push(InterfaceSnapshot {
            name: name.to_string(),
            mac: None,
            rx_bytes,
            tx_bytes,
            operstate: OperState::Unknown,
            timestamp: Utc::now(),
        });
    }

    snapshots.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(snapshots)
}
