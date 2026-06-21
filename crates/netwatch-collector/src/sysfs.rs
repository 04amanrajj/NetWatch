use std::fs;
use std::path::Path;

use chrono::Utc;
use netwatch_core::{Config, InterfaceSnapshot, OperState, Result};

pub struct SysfsCollector<'a> {
    config: &'a Config,
    root: &'a Path,
}

impl<'a> SysfsCollector<'a> {
    pub fn new(config: &'a Config, root: &'a Path) -> Self {
        Self { config, root }
    }

    pub fn collect(&self) -> Result<Vec<InterfaceSnapshot>> {
        let net_dir = self.root.join("sys/class/net");
        let mut snapshots = Vec::new();

        let entries = fs::read_dir(&net_dir).map_err(|e| {
            netwatch_core::NetWatchError::Collection(format!(
                "failed to read {}: {e}",
                net_dir.display()
            ))
        })?;

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if self.config.should_ignore(&name)? {
                continue;
            }

            let iface_dir = entry.path();
            let rx_bytes = read_u64_file(&iface_dir.join("statistics/rx_bytes"))?;
            let tx_bytes = read_u64_file(&iface_dir.join("statistics/tx_bytes"))?;
            let mac = fs::read_to_string(iface_dir.join("address"))
                .ok()

}}}
