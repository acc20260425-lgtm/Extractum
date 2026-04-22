use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager};
use tauri_plugin_sql::DbInstances;

use crate::error::{AppError, AppResult};

pub const DB_URL: &str = "sqlite:extractum.db";

pub async fn get_pool(handle: &AppHandle) -> AppResult<Pool<Sqlite>> {
    let instances = handle.state::<DbInstances>();
    let instances = instances.0.read().await;
    let db = instances.get(DB_URL).ok_or_else(|| {
        AppError::internal("Database not initialized. SQL preload may have failed.")
    })?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => Ok(pool.clone()),
        #[allow(unreachable_patterns)]
        _ => Err(AppError::internal("Expected SQLite pool")),
    }
}
