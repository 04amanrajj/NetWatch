use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use netwatch_collector::collect_interfaces;
use netwatch_core::Config;
use netwatch_db::Database;
use netwatch_stats::compute_samples;
use tokio::time::{interval, Instant};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "netwatchd", about = "NetWatch background statistics daemon")]
struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("netwatchd=info".parse()?))
        .init();

    let args = Args::parse();
    let mut config = Config::load(args.config.as_deref())?;
    config.validate()?;

    let db_path = config.database_path();
    let db = Database::open(&db_path, true)
        .await
        .context("open database")?;

    if !db.integrity_check().await? {
        db.insert_alert(
            chrono::Utc::now().timestamp(),
            netwatch_core::AlertKind::DatabaseCorruption.as_str(),
            "Database integrity check failed",
        )
        .await?;
        anyhow::bail!("database integrity check failed");
    }

    let pid = std::process::id();
    db.set_meta("pid", &pid.to_string()).await?;
    db.set_meta("sample_interval", &config.sample_interval.to_string())
        .await?;

    info!(pid, "netwatchd starting");

}
