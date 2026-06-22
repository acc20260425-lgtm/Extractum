use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager};
use tauri_plugin_sql::DbInstances;

use crate::error::{AppError, AppResult};

pub const APP_IDENTIFIER: &str = "org.ai.extractum";
pub const DB_FILENAME: &str = "extractum.db";
pub const DB_URL: &str = "sqlite:extractum.db";

pub(crate) fn db_path_from_config_dir(config_dir: &std::path::Path) -> std::path::PathBuf {
    config_dir.join(APP_IDENTIFIER).join(DB_FILENAME)
}

pub(crate) fn app_config_db_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|dir| db_path_from_config_dir(&dir))
}

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
