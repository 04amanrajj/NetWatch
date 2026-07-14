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
        Some(Commands::History) => run_tui(&config, &db_path, Page::History).await?,
        Some(Commands::Today) => print_summary(&config, &db_path, TimeRange::Today).await?,
        Some(Commands::Yesterday) => {
            print_summary(&config, &db_path, TimeRange::Yesterday).await?
        }
        Some(Commands::Interfaces) => print_interfaces(&config, &db_path).await?,
        Some(Commands::Daemon { action }) => daemon_action(action)?,
        Some(Commands::Export {
            today,
            month,
            range,
            format,
        }) => {
            export_data(&config, &db_path, today, month, range.as_deref(), format.into()).await?
        }
        Some(Commands::Doctor) => doctor(&config, &db_path).await?,
    }

    Ok(())
}

async fn open_db(path: &std::path::Path) -> Result<Database> {
    if path.exists() {
        match Database::open_readonly(path).await {
            Ok(db) => Ok(db),
            Err(_) => Database::open(path, false)
                .await
                .context("open database"),
        }
    } else {
        Database::open(path, true)
            .await
            .context("create database")
    }
}

async fn run_tui(config: &Config, db_path: &std::path::Path, page: Page) -> Result<()> {
    let db = open_db(db_path).await?;
    netwatch_tui::run(
        config,
        &db,
        RunOptions { initial_page: page },
    )
    .await
}

async fn print_summary(config: &Config, db_path: &std::path::Path, range: TimeRange) -> Result<()> {
    let db = open_db(db_path).await?;
    let now = chrono::Utc::now();
    let (start, end) = range.bounds(now);
    let totals = db
        .history_table(start.timestamp(), end.timestamp())
        .await?
        .into_iter()
        .fold((0u64, 0u64), |(dl, ul), e| (dl + e.download, ul + e.upload));

    let units = config.units;
    println!("Download: {}", format_bytes(totals.0, units));
    println!("Upload:   {}", format_bytes(totals.1, units));
    Ok(())
}

async fn print_interfaces(config: &Config, db_path: &std::path::Path) -> Result<()> {
    let db = open_db(db_path).await?;
    let stats = db.interface_stats_today().await?;
    let units = config.units;
    println!("{:<16} {:>12} {:>12} {:>8}", "Interface", "Download", "Upload", "Status");
    for iface in stats {
        println!(
            "{:<16} {:>12} {:>12} {:>8}",
            iface.name,
            format_bytes(iface.download, units),
            format_bytes(iface.upload, units),
            iface.operstate,
        );
    }
    Ok(())
}

fn daemon_action(action: DaemonAction) -> Result<()> {
    let systemctl = |args: &[&str]| {
        let status = Command::new("systemctl")
            .args(["--user"])
            .args(args)
            .status()
            .context("run systemctl")?;
        if !status.success() {
            anyhow::bail!("systemctl {} failed", args.join(" "));
        }
        Ok(())
    };

}
