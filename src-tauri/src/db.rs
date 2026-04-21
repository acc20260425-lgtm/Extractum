use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager};
use tauri_plugin_sql::DbInstances;

pub const DB_URL: &str = "sqlite:extractum.db";

pub async fn get_pool(handle: &AppHandle) -> Result<Pool<Sqlite>, String> {
    let instances = handle.state::<DbInstances>();
    let instances = instances.0.read().await;
    let db = instances
        .get(DB_URL)
        .ok_or("Database not initialized. SQL preload may have failed.")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => Ok(pool.clone()),
        #[allow(unreachable_patterns)]
        _ => Err("Expected SQLite pool".to_string()),
    }
}
