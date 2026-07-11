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
        #[arg(long)]
        range: Option<String>,
        #[arg(short, long, value_enum, default_value_t = ExportFormatArg::Csv)]
        format: ExportFormatArg,
    },
    /// Diagnose installation and daemon health
    Doctor,
}

#[derive(Debug, Subcommand)]
enum DaemonAction {
    Status,
    Start,
    Stop,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExportFormatArg {
    Csv,
    Json,
    Markdown,
}

impl From<ExportFormatArg> for ExportFormat {
    fn from(v: ExportFormatArg) -> Self {
        match v {
            ExportFormatArg::Csv => ExportFormat::Csv,
            ExportFormatArg::Json => ExportFormat::Json,
            ExportFormatArg::Markdown => ExportFormat::Markdown,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;
    let db_path = config.database_path();

    match cli.command {
        None => run_tui(&config, &db_path, Page::Home).await?,
        Some(Commands::Live) => run_tui(&config, &db_path, Page::Live).await?,

}}
