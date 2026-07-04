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
            entry.peak_download,
            entry.peak_upload,
        ));
    }
    let _ = units;
    Ok(out)
}

fn export_json(history: &[netwatch_db::HistoryEntry]) -> Result<String> {
    Ok(serde_json::to_string_pretty(history)?)
}

fn export_markdown(
    history: &[netwatch_db::HistoryEntry],
    units: Units,
    range: &TimeRange,
) -> Result<String> {
    let mut out = String::new();
    writeln!(out, "# NetWatch Export")?;
    writeln!(out, "\nRange: `{range:?}`\n")?;
    writeln!(
        out,
        "| Date | Download | Upload | Total | Peak DL | Peak UL |"
    )?;
    writeln!(out, "|------|----------|--------|-------|---------|---------|")?;
    for entry in history {
        writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            entry.date,
            format_bytes(entry.download, units),
            format_bytes(entry.upload, units),
            format_bytes(entry.total, units),
            format_bytes(entry.peak_download, units),
            format_bytes(entry.peak_upload, units),
        )?;
    }
    Ok(out)
}

pub fn parse_range_flag(today: bool, month: bool, range: Option<&str>) -> Result<TimeRange> {
    if today {
        return Ok(TimeRange::Today);
    }
    if month {
        return Ok(TimeRange::CurrentMonth);
    }
    if let Some(r) = range {
        let parts: Vec<&str> = r.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("range must be YYYY-MM-DD:YYYY-MM-DD");
        }
        let start = chrono::NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")?
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let end = chrono::NaiveDate::parse_from_str(parts[1], "%Y-%m-%d")?
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc()
            .timestamp();
        return Ok(TimeRange::Custom { start, end });
    }
    Ok(TimeRange::Last30Days)
}
