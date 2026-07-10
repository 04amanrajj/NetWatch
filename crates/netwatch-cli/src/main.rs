use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use netwatch_core::{Config, TimeRange};
use netwatch_db::Database;
use netwatch_export::{parse_range_flag, ExportFormat, ExportOptions};
use netwatch_stats::format_bytes;
use netwatch_tui::{Page, RunOptions};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "netwatch", about = "NetWatch network usage monitor", version)]
struct Cli {
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Launch live monitor TUI
    Live,
    /// Print today's summary
    Today,
    /// Print yesterday's summary
    Yesterday,
    /// List interfaces
    Interfaces,
    /// Open history in TUI
    History,
    /// Daemon control
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Export statistics
    Export {
        #[arg(long)]
        today: bool,
        #[arg(long)]
        month: bool,

}}
