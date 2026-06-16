pub mod config;
pub mod error;
pub mod types;

pub use config::{default_config_path, Config, Units};
pub use error::{NetWatchError, Result};
pub use types::{AlertKind, InterfaceSnapshot, OperState, TimeRange};
