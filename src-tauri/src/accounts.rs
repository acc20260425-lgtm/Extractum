use serde::Serialize;
use tauri::AppHandle;

use crate::db::get_pool;
use crate::telegram::{clear_account_runtime, TelegramState};

#[derive(Serialize, sqlx::FromRow)]
pub struct AccountRecord {
    pub id: i64,
    pub label: String,
    pub api_id: i64,
    pub api_hash: String,
    pub phone: Option<String>,
    pub created_at: i64,
}

#[tauri::command]
pub async fn list_accounts(handle: AppHandle) -> Result<Vec<AccountRecord>, String> {
    let pool = get_pool(&handle).await?;
    sqlx::query_as(
        "SELECT id, label, api_id, api_hash, phone, created_at FROM accounts ORDER BY created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_account(
    handle: AppHandle,
    account_id: i64,
) -> Result<Option<AccountRecord>, String> {
    let pool = get_pool(&handle).await?;
    sqlx::query_as(
        "SELECT id, label, api_id, api_hash, phone, created_at FROM accounts WHERE id = ?",
    )
    .bind(account_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_account(
    handle: AppHandle,
    label: String,
    api_id: i64,
    api_hash: String,
) -> Result<AccountRecord, String> {
    let pool = get_pool(&handle).await?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    sqlx::query_as(
        "INSERT INTO accounts (label, api_id, api_hash, created_at) VALUES (?, ?, ?, ?) RETURNING id, label, api_id, api_hash, phone, created_at",
    )
    .bind(&label)
    .bind(api_id)
    .bind(&api_hash)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_account_phone(
    handle: AppHandle,
    account_id: i64,
    phone: String,
) -> Result<(), String> {
    let pool = get_pool(&handle).await?;
    sqlx::query("UPDATE accounts SET phone = ? WHERE id = ?")
        .bind(&phone)
        .bind(account_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clear_account_phone(handle: AppHandle, account_id: i64) -> Result<(), String> {
    let pool = get_pool(&handle).await?;
    sqlx::query("UPDATE accounts SET phone = NULL WHERE id = ?")
        .bind(account_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_account(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
) -> Result<(), String> {
    let pool = get_pool(&handle).await?;
    sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(account_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    clear_account_runtime(&handle, &state, account_id, true).await;
    Ok(())
}
