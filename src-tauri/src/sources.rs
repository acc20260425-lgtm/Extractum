use grammers_client::peer::Peer;
use grammers_session::types::PeerRef;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Pool;
use sqlx::Sqlite;
use std::io::Cursor;
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
    pub last_sync_state: Option<i64>,
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

#[derive(Serialize)]
pub struct SyncResult {
    pub inserted: i64,
    pub skipped: i64,
    pub last_message_id: Option<i64>,
}

#[derive(Serialize)]
pub struct ItemRecord {
    pub id: i64,
    pub source_id: i64,
    pub external_id: String,
    pub author: Option<String>,
    pub published_at: i64,
    pub content: String,
    pub has_raw_data: bool,
}

#[derive(sqlx::FromRow)]
struct SourceSyncTarget {
    id: i64,
    account_id: Option<i64>,
    external_id: String,
    title: Option<String>,
    metadata_zstd: Option<Vec<u8>>,
    last_sync_state: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct StoredItemRow {
    id: i64,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    content_zstd: Option<Vec<u8>>,
    raw_data_zstd: Option<Vec<u8>>,
}

#[derive(Default, Serialize, Deserialize)]
struct SourceMetadata {
    username: Option<String>,
}

/// Get the sqlx Pool from tauri-plugin-sql's managed state.
async fn get_pool(handle: &AppHandle) -> Result<Pool<Sqlite>, String> {
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
    let metadata_zstd = encode_source_metadata(&SourceMetadata {
        username: channel.username().map(|s| s.to_string()),
    })?;
    let now = now_secs();

    drop(accounts);

    let pool = get_pool(&handle).await?;
    sqlx::query_as(
        r#"
        INSERT INTO sources (source_type, external_id, title, metadata_zstd, is_active, is_member, account_id, created_at)
        VALUES ('telegram_channel', ?, ?, ?, 1, ?, ?, ?)
        ON CONFLICT(source_type, external_id) DO UPDATE SET
            title = excluded.title,
            metadata_zstd = excluded.metadata_zstd,
            is_member = excluded.is_member,
            account_id = excluded.account_id
        RETURNING id, account_id, external_id, title, last_sync_state, is_active, is_member, created_at
        "#,
    )
    .bind(&external_id)
    .bind(&title)
    .bind(metadata_zstd)
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
            "SELECT id, account_id, external_id, title, last_sync_state, is_active, is_member, created_at FROM sources WHERE account_id = ? ORDER BY created_at DESC",
        )
        .bind(aid)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())
    } else {
        sqlx::query_as(
            "SELECT id, account_id, external_id, title, last_sync_state, is_active, is_member, created_at FROM sources ORDER BY created_at DESC",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn sync_channel(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    source_id: i64,
) -> Result<SyncResult, String> {
    let pool = get_pool(&handle).await?;
    let source: SourceSyncTarget = sqlx::query_as(
        "SELECT id, account_id, external_id, title, metadata_zstd, last_sync_state FROM sources WHERE id = ?",
    )
    .bind(source_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Source {source_id} not found"))?;

    let account_id = source
        .account_id
        .ok_or_else(|| format!("Source {source_id} is not linked to an account"))?;

    let client = {
        let accounts = state.accounts.lock().await;
        crate::telegram::get_client(&accounts, account_id).await?.clone()
    };

    if !client.is_authorized().await.map_err(|e| e.to_string())? {
        return Err(format!("Account {account_id} is not authenticated"));
    }

    let peer = resolve_source_peer(&client, &source).await?;
    let mut messages = client.iter_messages(peer);
    let mut inserted = 0_i64;
    let mut skipped = 0_i64;
    let previous_last_sync = source.last_sync_state.unwrap_or(0);
    let mut max_message_id = previous_last_sync;

    while let Some(message) = messages.next().await.map_err(|e| e.to_string())? {
        let message_id = i64::from(message.id());
        if previous_last_sync > 0 && message_id <= previous_last_sync {
            break;
        }

        if message_id > max_message_id {
            max_message_id = message_id;
        }

        let content = message.text().trim();
        if content.is_empty() {
            skipped += 1;
            continue;
        }

        let author = message_author(&message);
        let published_at = message.date().timestamp();
        let content_zstd = compress_text(content)?;
        let raw_data_zstd = compress_json_bytes(&build_raw_payload(&message, &source.title, &author)?)?;

        let result = sqlx::query(
            r#"
            INSERT INTO items (source_id, external_id, author, published_at, ingested_at, content_zstd, raw_data_zstd)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(source_id, external_id) DO NOTHING
            "#,
        )
        .bind(source.id)
        .bind(message_id.to_string())
        .bind(&author)
        .bind(published_at)
        .bind(now_secs())
        .bind(content_zstd)
        .bind(raw_data_zstd)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if result.rows_affected() == 1 {
            inserted += 1;
        } else {
            skipped += 1;
        }
    }

    if max_message_id > previous_last_sync {
        sqlx::query("UPDATE sources SET last_sync_state = ? WHERE id = ?")
            .bind(max_message_id)
            .bind(source.id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(SyncResult {
        inserted,
        skipped,
        last_message_id: if max_message_id > 0 {
            Some(max_message_id)
        } else {
            source.last_sync_state
        },
    })
}

#[tauri::command]
pub async fn get_items(
    handle: AppHandle,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
) -> Result<Vec<ItemRecord>, String> {
    let pool = get_pool(&handle).await?;
    let limit = limit.clamp(1, 200);
    let rows: Vec<StoredItemRow> = if let Some(before) = before_published_at {
        sqlx::query_as(
            r#"
            SELECT id, source_id, external_id, author, published_at, content_zstd, raw_data_zstd
            FROM items
            WHERE source_id = ? AND published_at < ?
            ORDER BY published_at DESC
            LIMIT ?
            "#,
        )
        .bind(source_id)
        .bind(before)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            r#"
            SELECT id, source_id, external_id, author, published_at, content_zstd, raw_data_zstd
            FROM items
            WHERE source_id = ?
            ORDER BY published_at DESC
            LIMIT ?
            "#,
        )
        .bind(source_id)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    };

    rows.into_iter()
        .map(|row| {
            Ok(ItemRecord {
                id: row.id,
                source_id: row.source_id,
                external_id: row.external_id,
                author: row.author,
                published_at: row.published_at,
                content: decompress_text(
                    row.content_zstd
                        .as_deref()
                        .ok_or_else(|| format!("Item {} is missing content", row.id))?,
                )?,
                has_raw_data: row.raw_data_zstd.is_some(),
            })
        })
        .collect()
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

fn compress_text(input: &str) -> Result<Vec<u8>, String> {
    zstd::encode_all(Cursor::new(input.as_bytes()), 3).map_err(|e| e.to_string())
}

fn compress_json_bytes(bytes: &[u8]) -> Result<Vec<u8>, String> {
    zstd::encode_all(Cursor::new(bytes), 3).map_err(|e| e.to_string())
}

fn decompress_text(bytes: &[u8]) -> Result<String, String> {
    let decoded = zstd::decode_all(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    String::from_utf8(decoded).map_err(|e| e.to_string())
}

fn encode_source_metadata(metadata: &SourceMetadata) -> Result<Vec<u8>, String> {
    let json = serde_json::to_vec(metadata).map_err(|e| e.to_string())?;
    compress_json_bytes(&json)
}

fn decode_source_metadata(bytes: Option<&[u8]>) -> Result<SourceMetadata, String> {
    let Some(bytes) = bytes else {
        return Ok(SourceMetadata::default());
    };
    let decoded = zstd::decode_all(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    serde_json::from_slice(&decoded).map_err(|e| e.to_string())
}

async fn resolve_source_peer(
    client: &grammers_client::Client,
    source: &SourceSyncTarget,
) -> Result<PeerRef, String> {
    let channel_id = source
        .external_id
        .parse::<i64>()
        .map_err(|_| format!("Invalid external_id '{}' for source {}", source.external_id, source.id))?;

    let mut dialogs = client.iter_dialogs();
    while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
        if let Peer::Channel(channel) = dialog.peer() {
            if channel.id().bare_id() == channel_id {
                return Ok(channel.raw.clone().into());
            }
        }
    }

    let metadata = decode_source_metadata(source.metadata_zstd.as_deref())?;
    if let Some(username) = metadata.username {
        let peer = client
            .resolve_username(&username)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Channel '@{}' not found", username))?;

        return match peer {
            Peer::Channel(channel) => Ok(channel.raw.clone().into()),
            _ => Err(format!("Source {} does not resolve to a broadcast channel", source.id)),
        };
    }

    Err(format!(
        "Source {} could not be resolved from dialogs or stored username metadata",
        source.id
    ))
}

fn message_author(message: &grammers_client::message::Message) -> Option<String> {
    if let Some(post_author) = message.post_author() {
        return Some(post_author.to_string());
    }

    message.sender().and_then(|sender| {
        sender
            .name()
            .map(str::to_string)
            .or_else(|| sender.username().map(|username| format!("@{username}")))
    })
}

fn build_raw_payload(
    message: &grammers_client::message::Message,
    source_title: &Option<String>,
    author: &Option<String>,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&json!({
        "id": message.id(),
        "peer_id": message.peer_id().to_string(),
        "sender_id": message.sender_id().map(|id| id.to_string()),
        "published_at": message.date().timestamp(),
        "text": message.text(),
        "post_author": message.post_author(),
        "source_title": source_title,
        "author": author,
    }))
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{compress_text, decompress_text};

    #[test]
    fn text_roundtrip_through_zstd() {
        let original = "hello from extractum";
        let compressed = compress_text(original).expect("compress");
        let decompressed = decompress_text(&compressed).expect("decompress");
        assert_eq!(decompressed, original);
    }
}
