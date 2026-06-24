use std::path::Path;

use anyhow::{Context, Result};
use netwatch_stats::ComputedSample;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use tracing::info;

use crate::aggregation;
use crate::models::*;
use crate::queries;
use crate::retention;

const MIGRATIONS: &str = include_str!("migrations/001_init.sql");

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn open(path: &Path, create_dirs: bool) -> Result<Self> {
        if create_dirs {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create database directory {}", parent.display()))?;
            }
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .with_context(|| format!("open database {}", path.display()))?;

        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    pub async fn open_readonly(path: &Path) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .read_only(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);


}}
