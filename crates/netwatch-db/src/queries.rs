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
                r.get::<Option<i64>, _>("rx_rate").unwrap_or(0).max(0) as u64,
                r.get::<Option<i64>, _>("tx_rate").unwrap_or(0).max(0) as u64,
            )
        })
        .unwrap_or((0, 0)))
}

async fn sum_range(
    pool: &SqlitePool,
    start: i64,
    end: i64,
    interface_id: Option<i64>,
) -> Result<Totals> {
    let mut download: u64 = 0;
    let mut upload: u64 = 0;

    if end - start <= 86400 {
        let (dl, ul) = delta_from_raw(pool, start, end, interface_id).await?;
        download += dl;
        upload += ul;
    }

    if download == 0 && upload == 0 {
        let (dl, ul) = sum_aggregated(pool, "samples_minute", start, end, interface_id).await?;
        download += dl;
        upload += ul;
    }
    if download == 0 && upload == 0 {
        let (dl, ul) = sum_aggregated(pool, "samples_hourly", start, end, interface_id).await?;
        download += dl;
        upload += ul;
    }
    if download == 0 && upload == 0 {
        let (dl, ul) = sum_aggregated(pool, "samples_daily", start, end, interface_id).await?;
        download += dl;
        upload += ul;
    }

    let speeds = if interface_id.is_none() {
        current_speeds(pool).await?
    } else if let Some(id) = interface_id {
        let (rx, tx) = latest_rates(pool, id).await?;
        Totals {
            download: 0,
            upload: 0,
            rx_rate: rx,
            tx_rate: tx,
        }
    } else {
        Totals::default()
    };

    Ok(Totals {
        download,
        upload,
        rx_rate: speeds.rx_rate,
        tx_rate: speeds.tx_rate,
    })
}

async fn delta_from_raw(
    pool: &SqlitePool,
    start: i64,
    end: i64,
    interface_id: Option<i64>,
) -> Result<(u64, u64)> {
    let query = if let Some(id) = interface_id {
        sqlx::query(
            r#"
            SELECT
                COALESCE(MAX(rx_bytes) - MIN(rx_bytes), 0) AS dl,
                COALESCE(MAX(tx_bytes) - MIN(tx_bytes), 0) AS ul
            FROM samples_raw
            WHERE interface_id = ?1 AND ts BETWEEN ?2 AND ?3
            "#,
        )
        .bind(id)
        .bind(start)
        .bind(end)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(dl), 0) AS dl,
                COALESCE(SUM(ul), 0) AS ul
            FROM (
                SELECT
                    MAX(rx_bytes) - MIN(rx_bytes) AS dl,
                    MAX(tx_bytes) - MIN(tx_bytes) AS ul
                FROM samples_raw
                WHERE ts BETWEEN ?1 AND ?2
                GROUP BY interface_id
            )
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_one(pool)
        .await?
    };

    Ok((
        query.get::<i64, _>("dl").max(0) as u64,
        query.get::<i64, _>("ul").max(0) as u64,
    ))
}

async fn sum_aggregated(
    pool: &SqlitePool,
    table: &str,
    start: i64,
    end: i64,
    interface_id: Option<i64>,
) -> Result<(u64, u64)> {
    let sql = if interface_id.is_some() {
        format!(
            "SELECT COALESCE(SUM(rx_bytes), 0) AS dl, COALESCE(SUM(tx_bytes), 0) AS ul
             FROM {table} WHERE interface_id = ?1 AND ts BETWEEN ?2 AND ?3"
        )
    } else {
        format!(
            "SELECT COALESCE(SUM(rx_bytes), 0) AS dl, COALESCE(SUM(tx_bytes), 0) AS ul
             FROM {table} WHERE ts BETWEEN ?1 AND ?2"
        )
    };

    let row = if let Some(id) = interface_id {
        sqlx::query(&sql)
            .bind(id)
            .bind(start)
            .bind(end)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query(&sql).bind(start).bind(end).fetch_one(pool).await?
    };

    Ok((
        row.get::<i64, _>("dl").max(0) as u64,
        row.get::<i64, _>("ul").max(0) as u64,
    ))
}

pub async fn interface_detail(pool: &SqlitePool, interface_id: i64) -> Result<InterfaceDetail> {
    let iface = sqlx::query_as::<_, InterfaceRow>(
        "SELECT id, name, mac, first_seen, last_seen FROM interfaces WHERE id = ?1",
    )
    .bind(interface_id)
    .fetch_one(pool)
    .await?;

    let now = Utc::now();
    let (today_start, _) = day_bounds(now);
    let yesterday_start = today_start - chrono::Duration::days(1);
    let week_start = now - chrono::Duration::days(7);
    let last_week_start = now - chrono::Duration::days(14);
    let month_start = Utc
        .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
        .single()
        .unwrap();
    let prev_month_end = month_start;
    let prev_month_start = prev_month_end - chrono::Duration::days(1);
    let prev_month_start = Utc
        .with_ymd_and_hms(
            prev_month_start.year(),
            prev_month_start.month(),
            1,
            0,
            0,
            0,
        )
        .single()
        .unwrap();
    let year_start = Utc
        .with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0)
        .single()
        .unwrap();

    let today = sum_range(pool, today_start.timestamp(), now.timestamp(), Some(interface_id)).await?;
    let yesterday = sum_range(
        pool,
        yesterday_start.timestamp(),
        today_start.timestamp(),
        Some(interface_id),
    )
    .await?;
    let this_week = sum_range(pool, week_start.timestamp(), now.timestamp(), Some(interface_id)).await?;
    let last_week = sum_range(
        pool,
        last_week_start.timestamp(),
        week_start.timestamp(),
        Some(interface_id),
    )
    .await?;
    let this_month = sum_range(pool, month_start.timestamp(), now.timestamp(), Some(interface_id)).await?;
    let last_month = sum_range(
        pool,
        prev_month_start.timestamp(),
        prev_month_end.timestamp(),
        Some(interface_id),
    )
    .await?;
    let this_year = sum_range(pool, year_start.timestamp(), now.timestamp(), Some(interface_id)).await?;
    let total = sum_range(pool, iface.first_seen, now.timestamp(), Some(interface_id)).await?;
    let (cur_rx, cur_tx) = latest_rates(pool, interface_id).await?;

    let peak = sqlx::query(
        "SELECT COALESCE(MAX(rx_rate), 0) AS prx, COALESCE(MAX(tx_rate), 0) AS ptx FROM samples_raw WHERE interface_id = ?1",
    )
    .bind(interface_id)
    .fetch_one(pool)
    .await?;

    let avg = sqlx::query(
        "SELECT COALESCE(AVG(rx_rate), 0) AS arx, COALESCE(AVG(tx_rate), 0) AS atx FROM samples_raw WHERE interface_id = ?1 AND ts >= ?2",
    )
    .bind(interface_id)
    .bind(today_start.timestamp())
    .fetch_one(pool)
    .await?;

    Ok(InterfaceDetail {
        name: iface.name,
        current_rx_rate: cur_rx,
        current_tx_rate: cur_tx,
        peak_rx_rate: peak.get::<i64, _>("prx").max(0) as u64,
        peak_tx_rate: peak.get::<i64, _>("ptx").max(0) as u64,
        avg_rx_rate: avg.get::<i64, _>("arx").max(0) as u64,
        avg_tx_rate: avg.get::<i64, _>("atx").max(0) as u64,
        today_download: today.download,
        today_upload: today.upload,
        yesterday_download: yesterday.download,
        yesterday_upload: yesterday.upload,
        this_week_download: this_week.download,
        this_week_upload: this_week.upload,
        last_week_download: last_week.download,
        last_week_upload: last_week.upload,
        this_month_download: this_month.download,
        this_month_upload: this_month.upload,
        last_month_download: last_month.download,
        last_month_upload: last_month.upload,
        this_year_download: this_year.download,
        this_year_upload: this_year.upload,
        total_download: total.download,
        total_upload: total.upload,
    })
}

pub async fn history_table(pool: &SqlitePool, start_ts: i64, end_ts: i64) -> Result<Vec<HistoryEntry>> {
    let rows = sqlx::query(
        r#"
        SELECT
            ts,
            SUM(rx_bytes) AS dl,
            SUM(tx_bytes) AS ul,
            MAX(rx_rate_max) AS pdl,
            MAX(tx_rate_max) AS pul
        FROM samples_daily
        WHERE ts BETWEEN ?1 AND ?2
        GROUP BY ts
        ORDER BY ts
        "#,
    )
    .bind(start_ts)
    .bind(end_ts)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {

}}
