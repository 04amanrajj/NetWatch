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

}
