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
    }

    pub async fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT value FROM daemon_meta WHERE key = ?1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.get::<String, _>("value")))
    }

    pub async fn upsert_interface(
        &self,
        name: &str,
        mac: Option<&str>,
        ts: i64,
    ) -> Result<i64> {
        if let Some(existing) = sqlx::query_as::<_, InterfaceRow>(
            "SELECT id, name, mac, first_seen, last_seen FROM interfaces WHERE name = ?1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?
        {
            sqlx::query("UPDATE interfaces SET last_seen = ?1, mac = COALESCE(?2, mac) WHERE id = ?3")
                .bind(ts)
                .bind(mac)
                .bind(existing.id)
                .execute(&self.pool)
                .await?;
            return Ok(existing.id);
        }

        let result = sqlx::query(
            "INSERT INTO interfaces (name, mac, first_seen, last_seen) VALUES (?1, ?2, ?3, ?3)",
        )
        .bind(name)
        .bind(mac)
        .bind(ts)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn ingest_samples(
        &self,
        snapshots: &[ComputedSample],
        macs: &[(String, Option<String>)],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for sample in snapshots {
            let mac = macs
                .iter()
                .find(|(n, _)| n == &sample.interface)
                .and_then(|(_, m)| m.as_deref());
            let iface_id = self
                .upsert_interface_in_tx(&mut tx, &sample.interface, mac, sample.ts)
                .await?;

            sqlx::query(
                "INSERT OR REPLACE INTO samples_raw (ts, interface_id, rx_bytes, tx_bytes, rx_rate, tx_rate)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(sample.ts)
            .bind(iface_id)
            .bind(sample.rx_bytes as i64)
            .bind(sample.tx_bytes as i64)
            .bind(sample.rx_rate as i64)
            .bind(sample.tx_rate as i64)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn upsert_interface_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        name: &str,
        mac: Option<&str>,
        ts: i64,
    ) -> Result<i64> {
        if let Some(existing) = sqlx::query_as::<_, InterfaceRow>(
            "SELECT id, name, mac, first_seen, last_seen FROM interfaces WHERE name = ?1",
        )
        .bind(name)
        .fetch_optional(&mut **tx)
        .await?
        {
            sqlx::query("UPDATE interfaces SET last_seen = ?1, mac = COALESCE(?2, mac) WHERE id = ?3")
                .bind(ts)
                .bind(mac)
                .bind(existing.id)
                .execute(&mut **tx)
                .await?;
            return Ok(existing.id);
        }

        let result = sqlx::query(
            "INSERT INTO interfaces (name, mac, first_seen, last_seen) VALUES (?1, ?2, ?3, ?3)",
        )

}}
