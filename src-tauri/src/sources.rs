use grammers_client::peer::Peer;
use serde::Serialize;
use sqlx::Pool;
use sqlx::Sqlite;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_sql::DbInstances;

use crate::telegram::TelegramState;

const DB_URL: &str = "sqlite:extractum.db";

#[derive(Serialize)]
pub struct ChannelInfo {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub is_member: bool,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SourceRecord {
    pub id: i64,
    pub account_id: Option<i64>,
    pub external_id: String,
    pub title: Option<String>,
    pub is_member: bool,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AccountRecord {
    pub id: i64,
    pub label: String,
    pub api_id: i64,
    pub api_hash: String,
    pub phone: Option<String>,
    pub created_at: i64,
}

/// Get the sqlx Pool from tauri-plugin-sql's managed state.
async fn get_pool(handle: &AppHandle) -> Result<Pool<Sqlite>, String> {
    let instances = handle.state::<DbInstances>();
    let instances = instances.0.read().await;
    let db = instances
        .get(DB_URL)
        .ok_or("Database not initialized. Call Database.load() first.")?;
    match db {
        tauri_plugin_sql::DbPool::Sqlite(pool) => Ok(pool.clone()),
        #[allow(unreachable_patterns)]
        _ => Err("Expected SQLite pool".to_string()),
    }
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
pub async fn create_account(
    handle: AppHandle,
    label: String,
    api_id: i64,
    api_hash: String,
) -> Result<AccountRecord, String> {
    let pool = get_pool(&handle).await?;
    let now = now_secs();
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
pub async fn delete_account(handle: AppHandle, account_id: i64) -> Result<(), String> {
    let pool = get_pool(&handle).await?;
    sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(account_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn list_telegram_channels(
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
) -> Result<Vec<ChannelInfo>, String> {
    let accounts = state.accounts.lock().await;
    let client = crate::telegram::get_client(&accounts, account_id).await?;

    let mut channels = Vec::new();
    let mut dialogs = client.iter_dialogs();
    while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
        if let Peer::Channel(channel) = dialog.peer() {
            channels.push(ChannelInfo {
                id: channel.id().bare_id(),
                title: channel.title().to_string(),
                username: channel.username().map(|s| s.to_string()),
                is_member: !channel.raw.left,
            });
        }
    }
    Ok(channels)
}

#[tauri::command]
pub async fn add_telegram_source(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
    channel_ref: String,
) -> Result<SourceRecord, String> {
    let accounts = state.accounts.lock().await;
    let client = crate::telegram::get_client(&accounts, account_id).await?;

    let username = parse_username(&channel_ref);
    let peer = client
        .resolve_username(&username)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Channel '{}' not found", channel_ref))?;

    let channel = match peer {
        Peer::Channel(c) => c,
        _ => return Err("Not a broadcast channel".to_string()),
    };

    let external_id = channel.id().bare_id().to_string();
    let title = channel.title().to_string();
    let is_member = !channel.raw.left;
    let now = now_secs();

    drop(accounts);

    let pool = get_pool(&handle).await?;
    sqlx::query_as(
        r#"
        INSERT INTO sources (source_type, external_id, title, is_active, is_member, account_id, created_at)
        VALUES ('telegram_channel', ?, ?, 1, ?, ?, ?)
        ON CONFLICT(source_type, external_id) DO UPDATE SET
            title = excluded.title,
            is_member = excluded.is_member,
            account_id = excluded.account_id
        RETURNING id, account_id, external_id, title, is_active, is_member, created_at
        "#,
    )
    .bind(&external_id)
    .bind(&title)
    .bind(is_member)
    .bind(account_id)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_sources(
    handle: AppHandle,
    account_id: Option<i64>,
) -> Result<Vec<SourceRecord>, String> {
    let pool = get_pool(&handle).await?;
    if let Some(aid) = account_id {
        sqlx::query_as(
            "SELECT id, account_id, external_id, title, is_active, is_member, created_at FROM sources WHERE account_id = ? ORDER BY created_at DESC",
        )
        .bind(aid)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())
    } else {
        sqlx::query_as(
            "SELECT id, account_id, external_id, title, is_active, is_member, created_at FROM sources ORDER BY created_at DESC",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())
    }
}

fn parse_username(input: &str) -> String {
    let s = input.trim();
    if let Some(rest) = s.strip_prefix("https://t.me/") {
        return rest.split('/').next().unwrap_or(rest).to_string();
    }
    if let Some(rest) = s.strip_prefix("t.me/") {
        return rest.split('/').next().unwrap_or(rest).to_string();
    }
    s.trim_start_matches('@').to_string()
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
