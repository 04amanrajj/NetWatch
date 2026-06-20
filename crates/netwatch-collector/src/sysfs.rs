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

}}
