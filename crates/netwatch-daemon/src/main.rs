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

    run_daemon(config, db).await
}

async fn run_daemon(config: Config, db: Database) -> Result<()> {
    let sample_interval = Duration::from_secs(config.sample_interval);
    let batch_interval = Duration::from_secs(config.batch_write_interval);

    let mut sample_tick = interval(sample_interval);
    let mut batch_tick = interval(batch_interval);
    let mut agg_tick = interval(Duration::from_secs(60));
    let mut retention_tick = interval(Duration::from_secs(3600));

    let mut previous: HashMap<String, netwatch_stats::PreviousSample> = HashMap::new();
    let mut known_interfaces: HashSet<String> = HashSet::new();
    let mut pending_samples: Vec<netwatch_stats::ComputedSample> = Vec::new();
    let mut pending_macs: Vec<(String, Option<String>)> = Vec::new();

    loop {
        tokio::select! {
            _ = sample_tick.tick() => {
                let start = Instant::now();
                match collect_interfaces(&config) {
                    Ok(snapshots) => {
                        let current: HashSet<String> = snapshots.iter().map(|s| s.name.clone()).collect();
                        for removed in known_interfaces.difference(&current) {
                            let _ = db.mark_interface_removed(removed, chrono::Utc::now().timestamp()).await;
                        }
                        known_interfaces = current;

                        let computed = compute_samples(&snapshots, &mut previous);
                        for snap in &snapshots {
                            pending_macs.push((snap.name.clone(), snap.mac.clone()));
                        }
                        for sample in &computed {
                            if let Some(kind) = sample.anomaly {
                                let _ = db.insert_alert(
                                    sample.ts,
                                    kind.as_str(),
                                    &format!("Anomaly on {}: {:?}", sample.interface, kind),
                                ).await;
                            }
                        }
                        pending_samples.extend(computed);
                    }
                    Err(e) => warn!("collection failed: {e:#}"),
                }
                let elapsed = start.elapsed();
                if elapsed > Duration::from_millis(100) {
                    warn!("sample took {:?}", elapsed);
                }
            }

}}}
