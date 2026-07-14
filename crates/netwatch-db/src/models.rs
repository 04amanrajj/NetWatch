use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct InterfaceRow {
    pub id: i64,
    pub name: String,
    pub mac: Option<String>,
    pub first_seen: i64,
    pub last_seen: i64,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct SampleRow {
    pub ts: i64,
    pub interface_id: i64,
    pub rx_bytes: i64,
    pub tx_bytes: i64,
    pub rx_rate: Option<i64>,
    pub tx_rate: Option<i64>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct AggregatedRow {
    pub ts: i64,
    pub interface_id: i64,
    pub rx_bytes: i64,
    pub tx_bytes: i64,
    pub rx_rate_avg: i64,
    pub tx_rate_avg: i64,
    pub rx_rate_max: i64,
    pub tx_rate_max: i64,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct AlertRow {
    pub id: i64,
    pub ts: i64,
    pub kind: String,
    pub message: String,
    pub acknowledged: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceStats {
    pub id: i64,
    pub name: String,
    pub mac: Option<String>,
    pub operstate: String,
    pub download: u64,
    pub upload: u64,
    pub rx_rate: u64,
    pub tx_rate: u64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Totals {
    pub download: u64,
    pub upload: u64,
    pub rx_rate: u64,
    pub tx_rate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub pid: Option<i64>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub sample_interval: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub date: String,
    pub download: u64,
    pub upload: u64,
    pub total: u64,
    pub peak_download: u64,
    pub peak_upload: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPoint {
    pub ts: i64,
    pub rx_rate: u64,
    pub tx_rate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDetail {
    pub name: String,
    pub current_rx_rate: u64,
    pub current_tx_rate: u64,
    pub peak_rx_rate: u64,
    pub peak_tx_rate: u64,
    pub avg_rx_rate: u64,
    pub avg_tx_rate: u64,
    pub today_download: u64,
    pub today_upload: u64,
    pub yesterday_download: u64,
    pub yesterday_upload: u64,
    pub this_week_download: u64,
    pub this_week_upload: u64,
    pub last_week_download: u64,
    pub last_week_upload: u64,
    pub this_month_download: u64,
    pub this_month_upload: u64,
    pub last_month_download: u64,
    pub last_month_upload: u64,
    pub this_year_download: u64,
    pub this_year_upload: u64,
    pub total_download: u64,
    pub total_upload: u64,
}
