use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use sqlx::{Row, SqlitePool};

use crate::models::*;

pub async fn list_interfaces(pool: &SqlitePool) -> Result<Vec<InterfaceRow>> {
    Ok(sqlx::query_as::<_, InterfaceRow>(
        "SELECT id, name, mac, first_seen, last_seen FROM interfaces ORDER BY name",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn today_totals(pool: &SqlitePool) -> Result<Totals> {
    let (start, _) = day_bounds(Utc::now());
    sum_range(pool, start.timestamp(), Utc::now().timestamp(), None).await
}

pub async fn current_speeds(pool: &SqlitePool) -> Result<Totals> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(SUM(rx_rate), 0) AS rx, COALESCE(SUM(tx_rate), 0) AS tx
        FROM samples_raw
        WHERE ts = (SELECT MAX(ts) FROM samples_raw)
        "#,
    )
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row {
        Ok(Totals {
            download: 0,
            upload: 0,
            rx_rate: r.get::<i64, _>("rx").max(0) as u64,
            tx_rate: r.get::<i64, _>("tx").max(0) as u64,
        })
    } else {
        Ok(Totals::default())
    }
}

impl Default for Totals {
    fn default() -> Self {
        Self {
            download: 0,
            upload: 0,
            rx_rate: 0,
            tx_rate: 0,
        }
    }
}

pub async fn interface_stats_today(pool: &SqlitePool) -> Result<Vec<InterfaceStats>> {
    let (start, end) = day_bounds(Utc::now());
    let interfaces = list_interfaces(pool).await?;
    let mut stats = Vec::new();

    for iface in interfaces {
        let totals = sum_range(pool, start.timestamp(), end.timestamp(), Some(iface.id)).await?;
        let latest = latest_rates(pool, iface.id).await?;
        stats.push(InterfaceStats {
            id: iface.id,
            name: iface.name,
            mac: iface.mac,
            operstate: if iface.last_seen >= start.timestamp() {
                "UP".into()
            } else {
                "DOWN".into()
            },
            download: totals.download,
            upload: totals.upload,
            rx_rate: latest.0,
            tx_rate: latest.1,
        });
    }
    Ok(stats)
}

async fn latest_rates(pool: &SqlitePool, interface_id: i64) -> Result<(u64, u64)> {
    let row = sqlx::query(
        "SELECT rx_rate, tx_rate FROM samples_raw WHERE interface_id = ?1 ORDER BY ts DESC LIMIT 1",
    )
    .bind(interface_id)
    .fetch_optional(pool)
    .await?;

    Ok(row
        .map(|r| {
            (

}}
