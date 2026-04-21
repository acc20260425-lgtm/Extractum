use grammers_client::peer::{Channel, Peer};
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Manager;

use crate::telegram::TelegramState;

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
    pub external_id: String,
    pub title: Option<String>,
    pub is_member: bool,
    pub is_active: bool,
    pub created_at: i64,
}

/// Returns all broadcast channels from the user's dialog list.
#[tauri::command]
pub async fn list_telegram_channels(
    state: tauri::State<'_, TelegramState>,
) -> Result<Vec<ChannelInfo>, String> {
    let client_lock = state.client.lock().await;
    let client = client_lock.as_ref().ok_or("Telegram client not initialized")?;

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

/// Adds a Telegram channel as a source by username or t.me link.
#[tauri::command]
pub async fn add_telegram_source(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    channel_ref: String,
) -> Result<SourceRecord, String> {
    let client_lock = state.client.lock().await;
    let client = client_lock.as_ref().ok_or("Telegram client not initialized")?;

    let username = parse_username(&channel_ref);
    let peer = client
        .resolve_username(&username)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Channel '{}' not found", channel_ref))?;

    let channel = match peer {
        Peer::Channel(c) => c,
        _ => return Err("The provided reference is not a broadcast channel".to_string()),
    };

    let external_id = channel.id().bare_id().to_string();
    let title = channel.title().to_string();
    let is_member = !channel.raw.left;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let pool = open_pool(&handle).await?;

    let row: SourceRecord = sqlx::query_as(
        r#"
        INSERT INTO sources (source_type, external_id, title, is_active, is_member, created_at)
        VALUES ('telegram_channel', ?, ?, 1, ?, ?)
        ON CONFLICT(source_type, external_id) DO UPDATE SET
            title = excluded.title,
            is_member = excluded.is_member
        RETURNING id, external_id, title, is_active, is_member, created_at
        "#,
    )
    .bind(&external_id)
    .bind(&title)
    .bind(is_member)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    pool.close().await;
    Ok(row)
}

/// Lists all sources from the database.
#[tauri::command]
pub async fn list_sources(handle: AppHandle) -> Result<Vec<SourceRecord>, String> {
    let pool = open_pool(&handle).await?;

    let rows: Vec<SourceRecord> = sqlx::query_as(
        "SELECT id, external_id, title, is_active, is_member, created_at FROM sources ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    pool.close().await;
    Ok(rows)
}

// --- helpers ---

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

async fn open_pool(handle: &AppHandle) -> Result<SqlitePool, String> {
    let app_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_path = app_dir.join("extractum.db");
    let url = format!("sqlite:{}", db_path.to_string_lossy());
    SqlitePool::connect(&url).await.map_err(|e| e.to_string())
}
