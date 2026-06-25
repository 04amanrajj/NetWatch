use anyhow::Result;
use sqlx::SqlitePool;

const RAW_RETENTION_SECS: i64 = 86400; // 24 hours
const MINUTE_RETENTION_SECS: i64 = 2592000; // 30 days

pub async fn apply_retention(pool: &SqlitePool) -> Result<()> {

}
