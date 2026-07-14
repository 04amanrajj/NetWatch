mod proc_net;
mod sysfs;

use std::path::Path;

use netwatch_core::{Config, InterfaceSnapshot, Result};

pub use sysfs::SysfsCollector;

/// Collect network interface snapshots from the system or a custom root (for tests).
pub fn collect_interfaces(config: &Config) -> Result<Vec<InterfaceSnapshot>> {
    collect_from_root(config, Path::new("/"))
}

pub fn collect_from_root(config: &Config, root: &Path) -> Result<Vec<InterfaceSnapshot>> {
    let sysfs_root = root.join("sys/class/net");
    if sysfs_root.exists() {
        SysfsCollector::new(config, root).collect()
    } else {
        proc_net::collect_from_proc(config, &root.join("proc/net/dev"))
    }
}

pub fn collect_live(config: &Config) -> Result<Vec<InterfaceSnapshot>> {
    collect_interfaces(config)
}

#[cfg(test)]
pub fn snapshot_from_parts(
    name: &str,
    mac: Option<&str>,
    rx: u64,
    tx: u64,
    state: netwatch_core::OperState,
) -> InterfaceSnapshot {
    use chrono::Utc;
    InterfaceSnapshot {
        name: name.into(),
        mac: mac.map(String::from),
        rx_bytes: rx,
        tx_bytes: tx,
        operstate: state,
        timestamp: Utc::now(),
    }
}
