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

        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await
            .with_context(|| format!("open database readonly {}", path.display()))?;

        let db = Self { pool };
        Ok(db)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    async fn migrate(&self) -> Result<()> {
        for statement in MIGRATIONS.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement.trim())
                .execute(&self.pool)
                .await
                .context("run migration")?;
        }
        Ok(())
    }

    pub async fn integrity_check(&self) -> Result<bool> {
        let row: (String,) = sqlx::query_as("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0 == "ok")
    }

    pub async fn database_size_bytes(&self) -> Result<u64> {
        let page_count: (i64,) = sqlx::query_as("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await?;
        let page_size: (i64,) = sqlx::query_as("PRAGMA page_size")
            .fetch_one(&self.pool)
            .await?;
        Ok((page_count.0 * page_size.0) as u64)
    }

    pub async fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO daemon_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())

}}
