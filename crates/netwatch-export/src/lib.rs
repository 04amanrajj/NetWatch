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

pub async fn export(db: &Database, options: ExportOptions, units: Units) -> Result<String> {
    let now = Utc::now();
    let (start, end) = options.range.bounds(now);
    let history = db.history_table(start.timestamp(), end.timestamp()).await?;

    match options.format {
        ExportFormat::Csv => export_csv(&history, units),
        ExportFormat::Json => export_json(&history),
        ExportFormat::Markdown => export_markdown(&history, units, &options.range),
    }
}

fn export_csv(history: &[netwatch_db::HistoryEntry], units: Units) -> Result<String> {
    let mut out = String::from("date,download,upload,total,peak_download,peak_upload\n");
    for entry in history {
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            entry.date,
            entry.download,
            entry.upload,
            entry.total,

}}
