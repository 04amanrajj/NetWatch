use std::collections::HashMap;

use chrono::{DateTime, Utc};
use netwatch_core::{AlertKind, InterfaceSnapshot, Units};

#[derive(Debug, Clone, Default)]
pub struct PreviousSample {
    pub ts: i64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct RateSample {
    pub rx_rate: u64,
    pub tx_rate: u64,
    pub anomaly: Option<AlertKind>,
}

#[derive(Debug, Clone)]
pub struct ComputedSample {
    pub interface: String,
    pub ts: i64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_rate: u64,
    pub tx_rate: u64,
    pub anomaly: Option<AlertKind>,
}

pub fn compute_rate(
    prev: Option<&PreviousSample>,
    snapshot: &InterfaceSnapshot,
) -> RateSample {
    let ts = snapshot.timestamp.timestamp();
    let Some(prev) = prev else {

}}
