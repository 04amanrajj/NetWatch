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
        return RateSample {
            rx_rate: 0,
            tx_rate: 0,
            anomaly: None,
        };
    };

    if ts < prev.ts {
        return RateSample {
            rx_rate: 0,
            tx_rate: 0,
            anomaly: Some(AlertKind::ClockJump),
        };
    }

    let dt = (ts - prev.ts).max(1) as u64;
    let (rx_delta, rx_anomaly) = byte_delta(prev.rx_bytes, snapshot.rx_bytes);
    let (tx_delta, tx_anomaly) = byte_delta(prev.tx_bytes, snapshot.tx_bytes);

    let anomaly = rx_anomaly.or(tx_anomaly);

    RateSample {
        rx_rate: rx_delta / dt,
        tx_rate: tx_delta / dt,
        anomaly,
    }
}

fn byte_delta(prev: u64, curr: u64) -> (u64, Option<AlertKind>) {
    if curr >= prev {
        (curr - prev, None)
    } else {
        // Counter reset or wrap — treat curr as new baseline
        (curr, Some(AlertKind::CounterOverflow))
    }
}

pub fn compute_samples(
    snapshots: &[InterfaceSnapshot],
    previous: &mut HashMap<String, PreviousSample>,
) -> Vec<ComputedSample> {
    snapshots
        .iter()
        .map(|snap| {
            let prev = previous.get(&snap.name);
            let rate = compute_rate(prev, snap);
            previous.insert(
                snap.name.clone(),
                PreviousSample {
                    ts: snap.timestamp.timestamp(),
                    rx_bytes: snap.rx_bytes,
                    tx_bytes: snap.tx_bytes,
                },
            );
            ComputedSample {
                interface: snap.name.clone(),
                ts: snap.timestamp.timestamp(),
                rx_bytes: snap.rx_bytes,
                tx_bytes: snap.tx_bytes,
                rx_rate: rate.rx_rate,
                tx_rate: rate.tx_rate,
                anomaly: rate.anomaly,
            }
        })
        .collect()
}

pub fn format_bytes(value: u64, units: Units) -> String {
    match units {
        Units::Bits => format_bits(value.saturating_mul(8)),
        Units::Bytes => format_size(value),
        Units::Auto => format_size(value),

}}
