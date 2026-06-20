mod proc_net;
mod sysfs;

use std::path::Path;

use chrono::Utc;
use netwatch_core::{Config, InterfaceSnapshot, OperState, Result};

pub use sysfs::SysfsCollector;

/// Collect network interface snapshots from the system or a custom root (for tests).
