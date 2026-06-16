use thiserror::Error;

pub type Result<T> = std::result::Result<T, NetWatchError>;

#[derive(Debug, Error)]
pub enum NetWatchError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("collection error: {0}")]
    Collection(String),

}
