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

}}
