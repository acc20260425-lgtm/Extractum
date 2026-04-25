use base64::{engine::general_purpose, Engine as _};
use grammers_client::{media::Media, peer::Peer, tl};
use grammers_session::types::PeerRef;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Cursor;
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::telegram::TelegramState;

const DEFAULT_INITIAL_SYNC_MESSAGE_LIMIT: i64 = 500;
const MIN_INITIAL_SYNC_MESSAGE_LIMIT: i64 = 50;
const MAX_INITIAL_SYNC_MESSAGE_LIMIT: i64 = 5_000;
const DEFAULT_INITIAL_SYNC_DAY_LIMIT: i64 = 30;
const MIN_INITIAL_SYNC_DAY_LIMIT: i64 = 1;
const MAX_INITIAL_SYNC_DAY_LIMIT: i64 = 365;
const INITIAL_SYNC_MODE_SETTING_KEY: &str = "sync.initial.mode";
const INITIAL_SYNC_VALUE_SETTING_KEY: &str = "sync.initial.value";
const CONTENT_KIND_TEXT_ONLY: &str = "text_only";
const CONTENT_KIND_TEXT_WITH_MEDIA: &str = "text_with_media";
const CONTENT_KIND_MEDIA_ONLY: &str = "media_only";
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InitialSyncMode {
    RecentMessages,
    RecentDays,
}

impl InitialSyncMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "recent_messages" => Ok(Self::RecentMessages),
            "recent_days" => Ok(Self::RecentDays),
            other => Err(format!("Unsupported initial sync mode '{other}'")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::RecentMessages => "recent_messages",
            Self::RecentDays => "recent_days",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SyncSettingsRecord {
    pub initial_sync_mode: InitialSyncMode,
    pub initial_sync_value: i64,
}

#[derive(Serialize)]
pub struct ChannelInfo {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub is_member: bool,
    pub photo_data_url: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SourceRecord {
    pub id: i64,
    pub account_id: Option<i64>,
    pub external_id: String,
    pub title: Option<String>,
    pub last_sync_state: Option<i64>,
    pub last_synced_at: Option<i64>,
    pub is_member: bool,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct SyncResult {
    pub inserted: i64,
    pub skipped: i64,
    pub last_message_id: Option<i64>,
    pub initial_sync_policy_applied: Option<String>,
}

#[derive(Serialize)]
pub struct ItemRecord {
    pub id: i64,
    pub source_id: i64,
    pub external_id: String,
    pub author: Option<String>,
    pub published_at: i64,
    pub content: Option<String>,
    pub content_kind: String,
    pub has_media: bool,
    pub media_kind: Option<String>,
    pub media_summary: Option<String>,
    pub media_file_name: Option<String>,
    pub media_mime_type: Option<String>,
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
    content_kind: String,
    has_media: bool,
    media_kind: Option<String>,
    content_zstd: Option<Vec<u8>>,
    media_metadata_zstd: Option<Vec<u8>>,
    raw_data_zstd: Option<Vec<u8>>,
}

#[derive(Default, Serialize, Deserialize)]
struct SourceMetadata {
    username: Option<String>,
}

struct ResolvedChannelSource {
    external_id: String,
    title: String,
    is_member: bool,
    username: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
struct ItemMediaMetadata {
    summary: Option<String>,
    file_name: Option<String>,
    mime_type: Option<String>,
    size_bytes: Option<i64>,
    width: Option<i32>,
    height: Option<i32>,
    duration_seconds: Option<f64>,
}

struct ExtractedMediaPayload {
    kind: String,
    metadata: ItemMediaMetadata,
}

struct ExtractedItemPayload {
    content: Option<String>,
    content_kind: &'static str,
    media: Option<ExtractedMediaPayload>,
}

#[derive(Default)]
struct DocumentSignals {
    mime_type: Option<String>,
    has_video: bool,
    has_audio: bool,
    is_voice: bool,
    is_animated: bool,
}

fn default_sync_settings() -> SyncSettingsRecord {
    SyncSettingsRecord {
        initial_sync_mode: InitialSyncMode::RecentMessages,
        initial_sync_value: DEFAULT_INITIAL_SYNC_MESSAGE_LIMIT,
    }
}

fn validate_sync_settings(
    initial_sync_mode: InitialSyncMode,
    initial_sync_value: i64,
) -> AppResult<SyncSettingsRecord> {
    let allowed_range = match initial_sync_mode {
        InitialSyncMode::RecentMessages => {
            MIN_INITIAL_SYNC_MESSAGE_LIMIT..=MAX_INITIAL_SYNC_MESSAGE_LIMIT
        }
        InitialSyncMode::RecentDays => MIN_INITIAL_SYNC_DAY_LIMIT..=MAX_INITIAL_SYNC_DAY_LIMIT,
    };

    if !allowed_range.contains(&initial_sync_value) {
        let (unit_label, min_value, max_value) = match initial_sync_mode {
            InitialSyncMode::RecentMessages => (
                "messages",
                MIN_INITIAL_SYNC_MESSAGE_LIMIT,
                MAX_INITIAL_SYNC_MESSAGE_LIMIT,
            ),
            InitialSyncMode::RecentDays => (
                "days",
                MIN_INITIAL_SYNC_DAY_LIMIT,
                MAX_INITIAL_SYNC_DAY_LIMIT,
            ),
        };
        return Err(AppError::validation(format!(
            "Initial sync value for {} must be between {} and {} {}",
            initial_sync_mode.as_str(),
            min_value,
            max_value,
            unit_label
        )));
    }

    Ok(SyncSettingsRecord {
        initial_sync_mode,
        initial_sync_value,
    })
}

fn initial_sync_policy_label(settings: &SyncSettingsRecord) -> String {
    match settings.initial_sync_mode {
        InitialSyncMode::RecentMessages => {
            let unit = if settings.initial_sync_value == 1 {
                "message"
            } else {
                "messages"
            };
            format!("last {} {}", settings.initial_sync_value, unit)
        }
        InitialSyncMode::RecentDays => {
            let unit = if settings.initial_sync_value == 1 {
                "day"
            } else {
                "days"
            };
            format!("last {} {}", settings.initial_sync_value, unit)
        }
    }
}

async fn read_setting(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    key: &str,
) -> Result<Option<String>, String> {
    sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())
}

async fn write_setting(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    key: &str,
    value: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO app_settings (key, value)
        VALUES (?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value
        "#,
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

async fn load_sync_settings_from_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> AppResult<SyncSettingsRecord> {
    let default_settings = default_sync_settings();
    let mode = read_setting(pool, INITIAL_SYNC_MODE_SETTING_KEY)
        .await?
        .as_deref()
        .map(InitialSyncMode::parse)
        .transpose()?
        .unwrap_or(default_settings.initial_sync_mode);
    let value = read_setting(pool, INITIAL_SYNC_VALUE_SETTING_KEY)
        .await?
        .as_deref()
        .and_then(|stored| stored.trim().parse::<i64>().ok())
        .unwrap_or(match mode {
            InitialSyncMode::RecentMessages => DEFAULT_INITIAL_SYNC_MESSAGE_LIMIT,
            InitialSyncMode::RecentDays => DEFAULT_INITIAL_SYNC_DAY_LIMIT,
        });

    validate_sync_settings(mode, value)
}

async fn save_sync_settings_to_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    settings: &SyncSettingsRecord,
) -> AppResult<()> {
    write_setting(
        pool,
        INITIAL_SYNC_MODE_SETTING_KEY,
        settings.initial_sync_mode.as_str(),
    )
    .await?;
    write_setting(
        pool,
        INITIAL_SYNC_VALUE_SETTING_KEY,
        &settings.initial_sync_value.to_string(),
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_sync_settings(handle: AppHandle) -> AppResult<SyncSettingsRecord> {
    let pool = get_pool(&handle).await?;
    load_sync_settings_from_pool(&pool).await
}

#[tauri::command]
pub async fn save_sync_settings(
    handle: AppHandle,
    initial_sync_mode: String,
    initial_sync_value: i64,
) -> AppResult<SyncSettingsRecord> {
    let pool = get_pool(&handle).await?;
    let mode = InitialSyncMode::parse(&initial_sync_mode)?;
    let settings = validate_sync_settings(mode, initial_sync_value)?;
    save_sync_settings_to_pool(&pool, &settings).await?;
    Ok(settings)
}

#[tauri::command]
pub async fn delete_source(handle: AppHandle, source_id: i64) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    let result = sqlx::query("DELETE FROM sources WHERE id = ?")
        .bind(source_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!("Source {source_id} not found")));
    }

    Ok(())
}

#[tauri::command]
pub async fn list_telegram_channels(
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
) -> AppResult<Vec<ChannelInfo>> {
    let client = {
        let accounts = state.accounts.lock().await;
        crate::telegram::get_client(&accounts, account_id)
            .await?
            .clone()
    };

    let mut channels = Vec::new();
    let mut dialogs = client.iter_dialogs();
    while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
        if let Peer::Channel(channel) = dialog.peer() {
            let photo_data_url = channel_photo_data_url(&client, dialog.peer())
                .await
                .unwrap_or(None);
            channels.push(ChannelInfo {
                id: channel.id().bare_id(),
                title: channel.title().to_string(),
                username: channel.username().map(|s| s.to_string()),
                is_member: !channel.raw.left,
                photo_data_url,
            });
        }
    }
    Ok(channels)
}

async fn channel_photo_data_url(
    client: &grammers_client::Client,
    peer: &Peer,
) -> Result<Option<String>, String> {
    let Some(photo) = peer.photo(false).await else {
        return Ok(None);
    };

    let mut bytes = Vec::new();
    let mut download = client.iter_download(&photo).chunk_size(4 * 1024);
    while let Some(chunk) = download.next().await.map_err(|e| e.to_string())? {
        bytes.extend(chunk);
    }

    if bytes.is_empty() {
        return Ok(None);
    }

    Ok(Some(format!(
        "data:image/jpeg;base64,{}",
        general_purpose::STANDARD.encode(bytes)
    )))
}

#[tauri::command]
pub async fn add_telegram_source(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
    channel_ref: String,
) -> AppResult<SourceRecord> {
    let accounts = state.accounts.lock().await;
    let client = crate::telegram::get_client(&accounts, account_id).await?;

    let resolved = resolve_channel_source(&client, &channel_ref).await?;
    let metadata_zstd = encode_source_metadata(&SourceMetadata {
        username: resolved.username,
    })?;
    let now = now_secs();

    drop(accounts);

    let pool = get_pool(&handle).await?;
    Ok(sqlx::query_as(
        r#"
        INSERT INTO sources (source_type, external_id, title, metadata_zstd, is_active, is_member, account_id, created_at)
        VALUES ('telegram_channel', ?, ?, ?, 1, ?, ?, ?)
        ON CONFLICT(source_type, external_id) DO UPDATE SET
            title = excluded.title,
            metadata_zstd = excluded.metadata_zstd,
            is_member = excluded.is_member,
            account_id = excluded.account_id
        RETURNING id, account_id, external_id, title, last_sync_state, last_synced_at, is_active, is_member, created_at
        "#,
    )
    .bind(&resolved.external_id)
    .bind(&resolved.title)
    .bind(metadata_zstd)
    .bind(resolved.is_member)
    .bind(account_id)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn list_sources(
    handle: AppHandle,
    account_id: Option<i64>,
) -> AppResult<Vec<SourceRecord>> {
    let pool = get_pool(&handle).await?;
    if let Some(aid) = account_id {
        Ok(sqlx::query_as(
            "SELECT id, account_id, external_id, title, last_sync_state, last_synced_at, is_active, is_member, created_at FROM sources WHERE account_id = ? ORDER BY created_at DESC",
        )
        .bind(aid)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?)
    } else {
        Ok(sqlx::query_as(
            "SELECT id, account_id, external_id, title, last_sync_state, last_synced_at, is_active, is_member, created_at FROM sources ORDER BY created_at DESC",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?)
    }
}

#[tauri::command]
pub async fn sync_channel(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    source_id: i64,
) -> AppResult<SyncResult> {
    let pool = get_pool(&handle).await?;
    let source: SourceSyncTarget = sqlx::query_as(
        "SELECT id, account_id, external_id, title, metadata_zstd, last_sync_state FROM sources WHERE id = ?",
    )
    .bind(source_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| AppError::not_found(format!("Source {source_id} not found")))?;

    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;

    let client = {
        let accounts = state.accounts.lock().await;
        crate::telegram::get_client(&accounts, account_id)
            .await?
            .clone()
    };

    if !client.is_authorized().await.map_err(|e| e.to_string())? {
        return Err(AppError::auth(format!(
            "Account {account_id} is not authenticated"
        )));
    }

    let peer = resolve_source_peer(&client, &source).await?;
    let mut inserted = 0_i64;
    let mut skipped = 0_i64;
    let previous_last_sync = source.last_sync_state.unwrap_or(0);
    let initial_sync_settings = if previous_last_sync == 0 {
        Some(load_sync_settings_from_pool(&pool).await?)
    } else {
        None
    };
    let initial_sync_policy_applied = initial_sync_settings
        .as_ref()
        .map(initial_sync_policy_label);
    let initial_sync_cutoff =
        initial_sync_settings
            .as_ref()
            .and_then(|settings| match settings.initial_sync_mode {
                InitialSyncMode::RecentDays => {
                    Some(now_secs() - settings.initial_sync_value * SECONDS_PER_DAY)
                }
                InitialSyncMode::RecentMessages => None,
            });
    let mut max_message_id = previous_last_sync;
    let mut messages = if let Some(settings) = initial_sync_settings.as_ref() {
        match settings.initial_sync_mode {
            InitialSyncMode::RecentMessages => client
                .iter_messages(peer)
                .limit(settings.initial_sync_value as usize),
            InitialSyncMode::RecentDays => client.iter_messages(peer),
        }
    } else {
        client.iter_messages(peer)
    };

    while let Some(message) = messages.next().await.map_err(|e| e.to_string())? {
        let message_id = i64::from(message.id());
        if previous_last_sync > 0 && message_id <= previous_last_sync {
            break;
        }
        let published_at = message.date().timestamp();
        if let Some(cutoff) = initial_sync_cutoff {
            if published_at < cutoff {
                break;
            }
        }

        if message_id > max_message_id {
            max_message_id = message_id;
        }

        let item_payload = match extract_item_payload(&message) {
            Some(payload) => payload,
            None => {
                skipped += 1;
                continue;
            }
        };

        let content_zstd = item_payload
            .content
            .as_deref()
            .map(compress_text)
            .transpose()?;
        let media_kind = item_payload.media.as_ref().map(|media| media.kind.clone());
        let media_metadata_zstd = item_payload
            .media
            .as_ref()
            .map(|media| encode_media_metadata(&media.metadata))
            .transpose()?;

        if content_zstd.is_none() && media_metadata_zstd.is_none() {
            skipped += 1;
            continue;
        }

        let author = message_author(&message);
        let raw_data_zstd = compress_json_bytes(&build_raw_payload(
            &message,
            &source.title,
            &author,
            &item_payload,
        )?)?;

        let result = sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                ingested_at,
                content_zstd,
                raw_data_zstd,
                content_kind,
                has_media,
                media_kind,
                media_metadata_zstd
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(item_payload.content_kind)
        .bind(item_payload.media.is_some())
        .bind(&media_kind)
        .bind(media_metadata_zstd)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if result.rows_affected() == 1 {
            inserted += 1;
        } else {
            skipped += 1;
        }
    }

    let sync_completed_at = now_secs();
    let last_sync_state = if max_message_id > previous_last_sync {
        Some(max_message_id)
    } else {
        source.last_sync_state
    };

    sqlx::query("UPDATE sources SET last_sync_state = ?, last_synced_at = ? WHERE id = ?")
        .bind(last_sync_state)
        .bind(sync_completed_at)
        .bind(source.id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SyncResult {
        inserted,
        skipped,
        last_message_id: last_sync_state,
        initial_sync_policy_applied,
    })
}

#[tauri::command]
pub async fn get_items(
    handle: AppHandle,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
) -> AppResult<Vec<ItemRecord>> {
    let pool = get_pool(&handle).await?;
    let limit = limit.clamp(1, 200);
    let rows: Vec<StoredItemRow> = if let Some(before) = before_published_at {
        sqlx::query_as(
            r#"
            SELECT
                id,
                source_id,
                external_id,
                author,
                published_at,
                content_kind,
                has_media,
                media_kind,
                content_zstd,
                media_metadata_zstd,
                raw_data_zstd
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
            SELECT
                id,
                source_id,
                external_id,
                author,
                published_at,
                content_kind,
                has_media,
                media_kind,
                content_zstd,
                media_metadata_zstd,
                raw_data_zstd
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

    Ok(rows
        .into_iter()
        .map(|row| {
            let media_metadata = decode_media_metadata(row.media_metadata_zstd.as_deref())?;
            Ok(ItemRecord {
                id: row.id,
                source_id: row.source_id,
                external_id: row.external_id,
                author: row.author,
                published_at: row.published_at,
                content: row
                    .content_zstd
                    .as_deref()
                    .map(decompress_text)
                    .transpose()?,
                content_kind: row.content_kind,
                has_media: row.has_media,
                media_kind: row.media_kind,
                media_summary: media_metadata.summary,
                media_file_name: media_metadata.file_name,
                media_mime_type: media_metadata.mime_type,
                has_raw_data: row.raw_data_zstd.is_some(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?)
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

async fn resolve_channel_source(
    client: &grammers_client::Client,
    channel_ref: &str,
) -> Result<ResolvedChannelSource, String> {
    let trimmed = channel_ref.trim();
    let username = parse_username(trimmed);

    if !username.is_empty() && !username.chars().all(|char| char.is_ascii_digit()) {
        let peer = client
            .resolve_username(&username)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Channel '{}' not found", channel_ref))?;

        return match peer {
            Peer::Channel(channel) => Ok(ResolvedChannelSource {
                external_id: channel.id().bare_id().to_string(),
                title: channel.title().to_string(),
                is_member: !channel.raw.left,
                username: channel.username().map(|value| value.to_string()),
            }),
            _ => Err("Not a broadcast channel".to_string()),
        };
    }

    let Ok(channel_id) = trimmed.parse::<i64>() else {
        return Err(format!("Channel '{}' not found", channel_ref));
    };

    let mut dialogs = client.iter_dialogs();
    while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
        if let Peer::Channel(channel) = dialog.peer() {
            if channel.id().bare_id() == channel_id {
                return Ok(ResolvedChannelSource {
                    external_id: channel.id().bare_id().to_string(),
                    title: channel.title().to_string(),
                    is_member: !channel.raw.left,
                    username: channel.username().map(|value| value.to_string()),
                });
            }
        }
    }

    Err(format!(
        "Channel '{}' could not be found in this account's dialogs",
        channel_ref
    ))
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

fn encode_media_metadata(metadata: &ItemMediaMetadata) -> Result<Vec<u8>, String> {
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

fn decode_media_metadata(bytes: Option<&[u8]>) -> Result<ItemMediaMetadata, String> {
    let Some(bytes) = bytes else {
        return Ok(ItemMediaMetadata::default());
    };
    let decoded = zstd::decode_all(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    serde_json::from_slice(&decoded).map_err(|e| e.to_string())
}

async fn resolve_source_peer(
    client: &grammers_client::Client,
    source: &SourceSyncTarget,
) -> Result<PeerRef, String> {
    let channel_id = source.external_id.parse::<i64>().map_err(|_| {
        format!(
            "Invalid external_id '{}' for source {}",
            source.external_id, source.id
        )
    })?;

    let metadata = decode_source_metadata(source.metadata_zstd.as_deref())?;
    if let Some(username) = metadata.username {
        if let Some(peer) = client
            .resolve_username(&username)
            .await
            .map_err(|e| e.to_string())?
        {
            return match peer {
                Peer::Channel(channel) => Ok(channel.raw.clone().into()),
                _ => Err(format!(
                    "Source {} does not resolve to a broadcast channel",
                    source.id
                )),
            };
        }
    }

    let mut dialogs = client.iter_dialogs();
    while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
        if let Peer::Channel(channel) = dialog.peer() {
            if channel.id().bare_id() == channel_id {
                return Ok(channel.raw.clone().into());
            }
        }
    }

    Err(format!(
        "Source {} could not be resolved from stored username metadata or dialogs",
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

fn trimmed_non_empty(input: &str) -> Option<String> {
    let trimmed = input.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn media_label(kind: &str) -> &'static str {
    match kind {
        "photo" => "Photo",
        "video" => "Video",
        "audio" => "Audio",
        "voice" => "Voice message",
        "image" => "Image",
        "animation" => "Animation",
        "sticker" => "Sticker",
        "contact" => "Contact card",
        "poll" => "Poll",
        "location" => "Location",
        "live_location" => "Live location",
        "venue" => "Venue",
        "webpage" => "Web page preview",
        "dice" => "Dice",
        _ => "Document",
    }
}

fn derive_content_kind(has_content: bool, has_media: bool) -> &'static str {
    match (has_content, has_media) {
        (true, true) => CONTENT_KIND_TEXT_WITH_MEDIA,
        (false, true) => CONTENT_KIND_MEDIA_ONLY,
        _ => CONTENT_KIND_TEXT_ONLY,
    }
}

fn collect_document_signals(document: &grammers_client::media::Document) -> DocumentSignals {
    let mut signals = DocumentSignals {
        mime_type: document.mime_type().map(str::to_string),
        is_animated: document.is_animated(),
        ..DocumentSignals::default()
    };

    if let Some(tl::enums::Document::Document(raw_document)) = document.raw.document.as_ref() {
        for attribute in &raw_document.attributes {
            match attribute {
                tl::enums::DocumentAttribute::Video(_) => signals.has_video = true,
                tl::enums::DocumentAttribute::Audio(audio) => {
                    signals.has_audio = true;
                    signals.is_voice = audio.voice;
                }
                _ => {}
            }
        }
    }

    signals
}

fn derive_document_media_kind(signals: &DocumentSignals) -> &'static str {
    let mime_type = signals.mime_type.as_deref().unwrap_or("");

    if signals.has_video || mime_type.starts_with("video/") {
        return "video";
    }
    if signals.is_voice {
        return "voice";
    }
    if signals.has_audio || mime_type.starts_with("audio/") {
        return "audio";
    }
    if signals.is_animated {
        return "animation";
    }
    if mime_type.starts_with("image/") {
        return "image";
    }
    "document"
}

fn contact_summary(contact: &grammers_client::media::Contact) -> String {
    let display_name = [contact.first_name(), contact.last_name()]
        .into_iter()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if !display_name.is_empty() {
        return format!("Contact: {display_name}");
    }

    if !contact.phone_number().trim().is_empty() {
        return format!("Contact: {}", contact.phone_number().trim());
    }

    "Contact card".to_string()
}

fn extract_document_media_payload(
    document: &grammers_client::media::Document,
) -> ExtractedMediaPayload {
    let signals = collect_document_signals(document);
    let kind = derive_document_media_kind(&signals).to_string();
    let resolution = document.resolution();

    ExtractedMediaPayload {
        kind: kind.clone(),
        metadata: ItemMediaMetadata {
            summary: Some(media_label(&kind).to_string()),
            file_name: document.name().and_then(|name| trimmed_non_empty(name)),
            mime_type: document.mime_type().map(str::to_string),
            size_bytes: document.size().and_then(|size| i64::try_from(size).ok()),
            width: resolution.map(|(width, _)| width),
            height: resolution.map(|(_, height)| height),
            duration_seconds: document.duration(),
        },
    }
}

fn extract_media_payload(media: Media) -> ExtractedMediaPayload {
    match media {
        Media::Photo(photo) => ExtractedMediaPayload {
            kind: "photo".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Photo".to_string()),
                size_bytes: photo.size().and_then(|size| i64::try_from(size).ok()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Document(document) => extract_document_media_payload(&document),
        Media::Sticker(sticker) => ExtractedMediaPayload {
            kind: "sticker".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some(if sticker.emoji().trim().is_empty() {
                    "Sticker".to_string()
                } else {
                    format!("Sticker {}", sticker.emoji().trim())
                }),
                file_name: sticker.document.name().and_then(trimmed_non_empty),
                mime_type: sticker.document.mime_type().map(str::to_string),
                size_bytes: sticker
                    .document
                    .size()
                    .and_then(|size| i64::try_from(size).ok()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Contact(contact) => ExtractedMediaPayload {
            kind: "contact".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some(contact_summary(&contact)),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Poll(_) => ExtractedMediaPayload {
            kind: "poll".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Poll".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Geo(_) => ExtractedMediaPayload {
            kind: "location".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Location".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Dice(_) => ExtractedMediaPayload {
            kind: "dice".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Dice".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::Venue(venue) => ExtractedMediaPayload {
            kind: "venue".to_string(),
            metadata: ItemMediaMetadata {
                summary: trimmed_non_empty(&venue.raw_venue.title)
                    .or_else(|| Some("Venue".to_string())),
                ..ItemMediaMetadata::default()
            },
        },
        Media::GeoLive(_) => ExtractedMediaPayload {
            kind: "live_location".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Live location".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        Media::WebPage(_) => ExtractedMediaPayload {
            kind: "webpage".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Web page preview".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
        _ => ExtractedMediaPayload {
            kind: "document".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Media".to_string()),
                ..ItemMediaMetadata::default()
            },
        },
    }
}

fn extract_item_payload(
    message: &grammers_client::message::Message,
) -> Option<ExtractedItemPayload> {
    let content = trimmed_non_empty(message.text());
    let media = message.media().map(extract_media_payload);
    let has_content = content.is_some();
    let has_media = media.is_some();

    if !has_content && !has_media {
        return None;
    }

    Some(ExtractedItemPayload {
        content,
        content_kind: derive_content_kind(has_content, has_media),
        media,
    })
}

fn build_raw_payload(
    message: &grammers_client::message::Message,
    source_title: &Option<String>,
    author: &Option<String>,
    item_payload: &ExtractedItemPayload,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&json!({
        "id": message.id(),
        "peer_id": message.peer_id().to_string(),
        "sender_id": message.sender_id().map(|id| id.to_string()),
        "published_at": message.date().timestamp(),
        "text": item_payload.content.as_deref(),
        "content_kind": item_payload.content_kind,
        "has_media": item_payload.media.is_some(),
        "media_kind": item_payload.media.as_ref().map(|media| &media.kind),
        "media_metadata": item_payload.media.as_ref().map(|media| &media.metadata),
        "post_author": message.post_author(),
        "source_title": source_title,
        "author": author,
    }))
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        compress_text, decode_media_metadata, decompress_text, default_sync_settings,
        derive_content_kind, derive_document_media_kind, encode_media_metadata,
        initial_sync_policy_label, load_sync_settings_from_pool, save_sync_settings_to_pool,
        validate_sync_settings, DocumentSignals, InitialSyncMode, ItemMediaMetadata,
        SyncSettingsRecord, CONTENT_KIND_MEDIA_ONLY, CONTENT_KIND_TEXT_ONLY,
        CONTENT_KIND_TEXT_WITH_MEDIA,
    };

    async fn memory_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query("CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT)")
            .execute(&pool)
            .await
            .expect("create app_settings");
        pool
    }

    #[test]
    fn text_roundtrip_through_zstd() {
        let original = "hello from extractum";
        let compressed = compress_text(original).expect("compress");
        let decompressed = decompress_text(&compressed).expect("decompress");
        assert_eq!(decompressed, original);
    }

    #[test]
    fn media_metadata_roundtrip_through_zstd() {
        let original = ItemMediaMetadata {
            summary: Some("Video".to_string()),
            file_name: Some("clip.mp4".to_string()),
            mime_type: Some("video/mp4".to_string()),
            size_bytes: Some(42),
            width: Some(1920),
            height: Some(1080),
            duration_seconds: Some(12.5),
        };

        let encoded = encode_media_metadata(&original).expect("encode");
        let decoded = decode_media_metadata(Some(&encoded)).expect("decode");
        assert_eq!(decoded, original);
    }

    #[test]
    fn derive_content_kind_tracks_text_and_media_presence() {
        assert_eq!(derive_content_kind(true, false), CONTENT_KIND_TEXT_ONLY);
        assert_eq!(
            derive_content_kind(true, true),
            CONTENT_KIND_TEXT_WITH_MEDIA
        );
        assert_eq!(derive_content_kind(false, true), CONTENT_KIND_MEDIA_ONLY);
    }

    #[test]
    fn derive_document_media_kind_prefers_specific_signals() {
        let voice = DocumentSignals {
            mime_type: Some("audio/ogg".to_string()),
            has_audio: true,
            is_voice: true,
            ..DocumentSignals::default()
        };
        assert_eq!(derive_document_media_kind(&voice), "voice");

        let video = DocumentSignals {
            mime_type: Some("application/octet-stream".to_string()),
            has_video: true,
            ..DocumentSignals::default()
        };
        assert_eq!(derive_document_media_kind(&video), "video");

        let image = DocumentSignals {
            mime_type: Some("image/png".to_string()),
            ..DocumentSignals::default()
        };
        assert_eq!(derive_document_media_kind(&image), "image");
    }

    #[test]
    fn initial_sync_policy_label_formats_messages_and_days() {
        assert_eq!(
            initial_sync_policy_label(&SyncSettingsRecord {
                initial_sync_mode: InitialSyncMode::RecentMessages,
                initial_sync_value: 500,
            }),
            "last 500 messages"
        );
        assert_eq!(
            initial_sync_policy_label(&SyncSettingsRecord {
                initial_sync_mode: InitialSyncMode::RecentDays,
                initial_sync_value: 1,
            }),
            "last 1 day"
        );
    }

    #[test]
    fn validate_sync_settings_rejects_out_of_range_values() {
        let result = validate_sync_settings(InitialSyncMode::RecentDays, 0);
        assert!(result.is_err());

        let result = validate_sync_settings(InitialSyncMode::RecentMessages, 10_000);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_settings_default_when_app_settings_are_missing() {
        let pool = memory_pool().await;
        let loaded = load_sync_settings_from_pool(&pool)
            .await
            .expect("load default sync settings");

        assert_eq!(loaded, default_sync_settings());
    }

    #[tokio::test]
    async fn sync_settings_roundtrip_through_app_settings() {
        let pool = memory_pool().await;
        let expected = SyncSettingsRecord {
            initial_sync_mode: InitialSyncMode::RecentDays,
            initial_sync_value: 14,
        };

        save_sync_settings_to_pool(&pool, &expected)
            .await
            .expect("save sync settings");
        let loaded = load_sync_settings_from_pool(&pool)
            .await
            .expect("load sync settings");

        assert_eq!(loaded, expected);
    }
}
