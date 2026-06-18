use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperState {
    Up,
    Down,
    Unknown,
}

impl OperState {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "up" => Self::Up,
            "down" => Self::Down,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "UP",
            Self::Down => "DOWN",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSnapshot {
    pub name: String,
    pub mac: Option<String>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub operstate: OperState,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertKind {
    DatabaseCorruption,
    InterfaceRemoved,
    CounterOverflow,
    ClockJump,
}

impl AlertKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DatabaseCorruption => "database_corruption",
            Self::InterfaceRemoved => "interface_removed",
            Self::CounterOverflow => "counter_overflow",
            Self::ClockJump => "clock_jump",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    Today,
    Yesterday,
    Last7Days,
    Last30Days,
    CurrentMonth,
    PreviousMonth,
    ThisYear,
    Custom { start: i64, end: i64 },
}

impl TimeRange {
    pub fn bounds(&self, now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
        use chrono::{Datelike, Duration, TimeZone, Timelike};

        let start_of_day = |dt: DateTime<Utc>| {
            Utc.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
                .single()
                .unwrap()
        };

        match self {
            Self::Today => {
                let start = start_of_day(now);
                (start, now)
            }
            Self::Yesterday => {
                let y = now - Duration::days(1);
                let start = start_of_day(y);
                let end = start_of_day(now);
                (start, end)
            }
            Self::Last7Days => (now - Duration::days(7), now),
            Self::Last30Days => (now - Duration::days(30), now),
            Self::CurrentMonth => {
                let start = Utc
                    .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
                    .single()
                    .unwrap();
                (start, now)
            }
            Self::PreviousMonth => {
                let first_this = Utc
                    .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
                    .single()
                    .unwrap();
                let end = first_this;
                let prev = end - Duration::days(1);
                let start = Utc
                    .with_ymd_and_hms(prev.year(), prev.month(), 1, 0, 0, 0)
                    .single()
                    .unwrap();
                (start, end)
            }
            Self::ThisYear => {
                let start = Utc
                    .with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0)
                    .single()
                    .unwrap();
                (start, now)
            }
            Self::Custom { start, end } => {
                let s = DateTime::from_timestamp(*start, 0).unwrap_or(now);
                let e = DateTime::from_timestamp(*end, 0).unwrap_or(now);
                (s, e)
            }
        }
    }
}
