use std::fmt::Write;

use anyhow::Result;
use chrono::Utc;
use netwatch_core::{TimeRange, Units};
use netwatch_db::Database;
use netwatch_stats::format_bytes;
use serde_json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Json,
    Markdown,
}

pub struct ExportOptions {
    pub format: ExportFormat,
    pub range: TimeRange,
    pub interface_id: Option<i64>,
}
