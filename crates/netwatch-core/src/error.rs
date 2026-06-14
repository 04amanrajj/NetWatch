use thiserror::Error;

pub type Result<T> = std::result::Result<T, NetWatchError>;

#[derive(Debug, Error)]
pub enum NetWatchError {
    #[error("configuration error: {0}")]

}
