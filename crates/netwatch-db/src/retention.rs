use anyhow::Result;
use sqlx::SqlitePool;

const RAW_RETENTION_SECS: i64 = 86400; // 24 hours
const MINUTE_RETENTION_SECS: i64 = 2592000; // 30 days

pub async fn apply_retention(pool: &SqlitePool) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    let raw_cutoff = now - RAW_RETENTION_SECS;
    let minute_cutoff = now - MINUTE_RETENTION_SECS;

    sqlx::query("DELETE FROM samples_raw WHERE ts < ?1")
        .bind(raw_cutoff)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM samples_minute WHERE ts < ?1")
        .bind(minute_cutoff)
        .execute(pool)
        .await?;

    Ok(())
}
