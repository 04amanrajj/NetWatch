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
            COALESCE(AVG(rx_rate), 0),
            COALESCE(AVG(tx_rate), 0),
            COALESCE(MAX(rx_rate), 0),
            COALESCE(MAX(tx_rate), 0)
        FROM samples_raw
        WHERE ts >= (strftime('%s', 'now') - 86400)
        GROUP BY bucket, interface_id
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn aggregate_minute_to_hourly(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO samples_hourly (ts, interface_id, rx_bytes, tx_bytes, rx_rate_avg, tx_rate_avg, rx_rate_max, tx_rate_max)
        SELECT
            (ts / 3600) * 3600 AS bucket,
            interface_id,
            SUM(rx_bytes),
            SUM(tx_bytes),
            COALESCE(AVG(rx_rate_avg), 0),
            COALESCE(AVG(tx_rate_avg), 0),
            COALESCE(MAX(rx_rate_max), 0),
            COALESCE(MAX(tx_rate_max), 0)
        FROM samples_minute
        WHERE ts >= (strftime('%s', 'now') - 2592000)
        GROUP BY bucket, interface_id
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn aggregate_hourly_to_daily(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO samples_daily (ts, interface_id, rx_bytes, tx_bytes, rx_rate_avg, tx_rate_avg, rx_rate_max, tx_rate_max)
        SELECT
            (ts / 86400) * 86400 AS bucket,
            interface_id,
            SUM(rx_bytes),
            SUM(tx_bytes),
            COALESCE(AVG(rx_rate_avg), 0),
            COALESCE(AVG(tx_rate_avg), 0),
            COALESCE(MAX(rx_rate_max), 0),
            COALESCE(MAX(tx_rate_max), 0)
        FROM samples_hourly
        GROUP BY bucket, interface_id
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}
