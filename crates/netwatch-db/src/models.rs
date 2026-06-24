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

