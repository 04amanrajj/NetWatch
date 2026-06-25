use anyhow::Result;
use sqlx::SqlitePool;

pub async fn aggregate_all(pool: &SqlitePool) -> Result<()> {
    aggregate_raw_to_minute(pool).await?;
    aggregate_minute_to_hourly(pool).await?;
    aggregate_hourly_to_daily(pool).await?;
    Ok(())
}

async fn aggregate_raw_to_minute(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO samples_minute (ts, interface_id, rx_bytes, tx_bytes, rx_rate_avg, tx_rate_avg, rx_rate_max, tx_rate_max)
        SELECT
            (ts / 60) * 60 AS bucket,
            interface_id,
            MAX(rx_bytes) - MIN(rx_bytes) AS rx_delta,
            MAX(tx_bytes) - MIN(tx_bytes) AS tx_delta,

}
