mod proc_net;
mod sysfs;

use std::path::Path;

use chrono::Utc;
use netwatch_core::{Config, InterfaceSnapshot, OperState, Result};

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
