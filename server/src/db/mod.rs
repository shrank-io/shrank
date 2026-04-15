use std::path::Path;

use deadpool_sqlite::{Config as PoolConfig, Pool, Runtime};

use crate::AppError;

pub mod documents;
pub mod entities;
pub mod graph;
pub mod migrations;
pub mod search;
pub mod sync;

pub struct Db {
    writer: Pool,
    reader: Pool,
}

impl Db {
    pub async fn new(data_dir: &Path) -> Result<Self, AppError> {
        let db_path = data_dir.join("shrank.db");
        let db_path_str = db_path.to_string_lossy().to_string();

        // Writer pool: single connection for serialized writes
        let writer_cfg = PoolConfig::new(&db_path_str);
        let writer = writer_cfg.create_pool(Runtime::Tokio1).map_err(|e| {
            AppError::Pool(format!("failed to create writer pool: {e}"))
        })?;

        // Reader pool: multiple connections for concurrent reads
        let mut reader_cfg = PoolConfig::new(&db_path_str);
        reader_cfg.pool = Some(deadpool_sqlite::PoolConfig {
            max_size: 4,
            ..Default::default()
        });
        let reader = reader_cfg.create_pool(Runtime::Tokio1).map_err(|e| {
            AppError::Pool(format!("failed to create reader pool: {e}"))
        })?;

        // Configure writer connection with PRAGMAs and run migrations
        {
            let conn = writer.get().await?;
            conn.interact(|conn| {
                conn.execute_batch(
                    "PRAGMA journal_mode = WAL;
                     PRAGMA foreign_keys = ON;
                     PRAGMA synchronous = NORMAL;
                     PRAGMA busy_timeout = 5000;",
                )?;
                migrations::run(conn)?;
                Ok::<(), rusqlite::Error>(())
            })
            .await??;
        }

        // Configure reader connections
        {
            let conn = reader.get().await?;
            conn.interact(|conn| {
                conn.execute_batch(
                    "PRAGMA journal_mode = WAL;
                     PRAGMA foreign_keys = ON;
                     PRAGMA synchronous = NORMAL;
                     PRAGMA busy_timeout = 5000;",
                )?;
                Ok::<(), rusqlite::Error>(())
            })
            .await??;
        }

        tracing::info!(path = %db_path.display(), "database initialized");
        Ok(Self { writer, reader })
    }

    pub async fn write(&self) -> Result<deadpool_sqlite::Object, AppError> {
        Ok(self.writer.get().await?)
    }

    pub async fn read(&self) -> Result<deadpool_sqlite::Object, AppError> {
        Ok(self.reader.get().await?)
    }
}
