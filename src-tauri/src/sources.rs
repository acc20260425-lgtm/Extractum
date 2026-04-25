use base64::{engine::general_purpose, Engine as _};
use grammers_client::{media::Media, peer::Peer, tl};
use grammers_session::types::{PeerAuth, PeerId, PeerRef};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, io::Cursor, path::PathBuf};
use tauri::{AppHandle, Manager};
use tokio::time::{timeout, Duration, Instant};

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
const TELEGRAM_SOURCE_TYPE: &str = "telegram";
const TELEGRAM_KIND_CHANNEL: &str = "channel";
const TELEGRAM_KIND_SUPERGROUP: &str = "supergroup";
const TELEGRAM_KIND_GROUP: &str = "group";
const TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS: u64 = 750;
const TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS: u64 = 4_000;
const TELEGRAM_SOURCE_AVATAR_CACHE_DIR: &str = "source_avatars";

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
pub struct TelegramSourceInfo {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub telegram_source_kind: String,
    pub is_member: bool,
    pub photo_data_url: Option<String>,
}

#[derive(Serialize)]
pub struct SourceRecord {
    pub id: i64,
    pub source_type: String,
    pub telegram_source_kind: String,
    pub account_id: Option<i64>,
    pub external_id: String,
    pub title: Option<String>,
    pub last_sync_state: Option<i64>,
    pub last_synced_at: Option<i64>,
    pub is_member: bool,
    pub is_active: bool,
    pub created_at: i64,
    pub avatar_data_url: Option<String>,
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
    source_type: String,
    telegram_source_kind: String,
    account_id: Option<i64>,
    external_id: String,
    title: Option<String>,
    metadata_zstd: Option<Vec<u8>>,
    last_sync_state: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct SourceRecordRow {
    id: i64,
    source_type: String,
    telegram_source_kind: String,
    account_id: Option<i64>,
    external_id: String,
    title: Option<String>,
    metadata_zstd: Option<Vec<u8>>,
    last_sync_state: Option<i64>,
    last_synced_at: Option<i64>,
    is_active: bool,
    is_member: bool,
    created_at: i64,
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
    added_from: Option<String>,
    access_hash: Option<i64>,
    avatar_cache_key: Option<String>,
}

struct ResolvedTelegramSource {
    external_id: String,
    title: String,
    telegram_source_kind: String,
    is_member: bool,
    username: Option<String>,
    access_hash: Option<i64>,
    avatar_bytes: Option<Vec<u8>>,
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
pub async fn list_telegram_sources(
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
) -> AppResult<Vec<TelegramSourceInfo>> {
    let client = {
        let accounts = state.accounts.lock().await;
        crate::telegram::get_client(&accounts, account_id)
            .await?
            .clone()
    };

    let mut sources = Vec::new();
    let mut dialogs = client.iter_dialogs();
    let photo_budget_started_at = Instant::now();
    let photo_budget = Duration::from_millis(TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS);
    while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
        if let Some(mut source) = telegram_source_info_from_peer(dialog.peer()) {
            if photo_budget_started_at.elapsed() < photo_budget {
                source.photo_data_url =
                    peer_photo_data_url_with_timeout(&client, dialog.peer()).await;
            }
            sources.push(source);
        }
    }
    Ok(sources)
}

async fn peer_photo_data_url_with_timeout(
    client: &grammers_client::Client,
    peer: &Peer,
) -> Option<String> {
    peer_photo_bytes_with_timeout(client, peer)
        .await
        .map(photo_bytes_data_url)
}

async fn peer_photo_bytes_with_timeout(
    client: &grammers_client::Client,
    peer: &Peer,
) -> Option<Vec<u8>> {
    timeout(
        Duration::from_millis(TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS),
        peer_photo_bytes(client, peer),
    )
    .await
    .ok()
    .and_then(Result::ok)
    .flatten()
}

async fn peer_photo_bytes(
    client: &grammers_client::Client,
    peer: &Peer,
) -> Result<Option<Vec<u8>>, String> {
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

    Ok(Some(bytes))
}

fn photo_bytes_data_url(bytes: Vec<u8>) -> String {
    format!(
        "data:image/jpeg;base64,{}",
        general_purpose::STANDARD.encode(bytes)
    )
}

fn telegram_source_info_from_peer(peer: &Peer) -> Option<TelegramSourceInfo> {
    match peer {
        Peer::Channel(channel) => Some(TelegramSourceInfo {
            id: channel.id().bare_id(),
            title: channel.title().to_string(),
            username: channel.username().map(|value| value.to_string()),
            telegram_source_kind: TELEGRAM_KIND_CHANNEL.to_string(),
            is_member: !channel.raw.left,
            photo_data_url: None,
        }),
        Peer::Group(group) => Some(TelegramSourceInfo {
            id: group.id().bare_id(),
            title: group.title().unwrap_or("Untitled group").to_string(),
            username: group.username().map(|value| value.to_string()),
            telegram_source_kind: telegram_group_kind(group).to_string(),
            is_member: telegram_group_is_member(group),
            photo_data_url: None,
        }),
        Peer::User(_) => None,
    }
}

fn telegram_group_kind(group: &grammers_client::peer::Group) -> &'static str {
    if group.is_megagroup() {
        TELEGRAM_KIND_SUPERGROUP
    } else {
        TELEGRAM_KIND_GROUP
    }
}

fn telegram_group_is_member(group: &grammers_client::peer::Group) -> bool {
    match &group.raw {
        tl::enums::Chat::Chat(chat) => !chat.left && !chat.deactivated,
        tl::enums::Chat::Channel(channel) => !channel.left,
        tl::enums::Chat::Empty(_)
        | tl::enums::Chat::Forbidden(_)
        | tl::enums::Chat::ChannelForbidden(_) => false,
    }
}

fn source_avatar_cache_key(
    account_id: i64,
    telegram_source_kind: &str,
    external_id: &str,
) -> String {
    format!("{account_id}_{telegram_source_kind}_{external_id}.jpg")
}

fn source_avatar_cache_dir(handle: &AppHandle) -> Result<PathBuf, String> {
    Ok(handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(TELEGRAM_SOURCE_AVATAR_CACHE_DIR))
}

fn cache_source_avatar(
    handle: &AppHandle,
    account_id: i64,
    telegram_source_kind: &str,
    external_id: &str,
    bytes: &[u8],
) -> Result<Option<String>, String> {
    if bytes.is_empty() {
        return Ok(None);
    }

    let cache_key = source_avatar_cache_key(account_id, telegram_source_kind, external_id);
    let cache_dir = source_avatar_cache_dir(handle)?;
    fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    fs::write(cache_dir.join(&cache_key), bytes).map_err(|e| e.to_string())?;
    Ok(Some(cache_key))
}

fn read_source_avatar_data_url(handle: &AppHandle, cache_key: &str) -> Option<String> {
    if cache_key.contains(['/', '\\']) {
        return None;
    }

    let path = source_avatar_cache_dir(handle).ok()?.join(cache_key);
    let bytes = fs::read(path).ok()?;
    if bytes.is_empty() {
        return None;
    }

    Some(photo_bytes_data_url(bytes))
}

#[tauri::command]
pub async fn add_telegram_source(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
    source_ref: String,
    telegram_source_kind: Option<String>,
) -> AppResult<SourceRecord> {
    let client = {
        let accounts = state.accounts.lock().await;
        crate::telegram::get_client(&accounts, account_id)
            .await?
            .clone()
    };

    let resolved =
        resolve_telegram_source(&client, &source_ref, telegram_source_kind.as_deref()).await?;
    let avatar_cache_key = if let Some(bytes) = resolved.avatar_bytes.as_deref() {
        cache_source_avatar(
            &handle,
            account_id,
            &resolved.telegram_source_kind,
            &resolved.external_id,
            bytes,
        )?
    } else {
        None
    };
    let metadata_zstd = encode_source_metadata(&SourceMetadata {
        username: resolved.username.clone(),
        added_from: Some(
            if telegram_source_kind.is_some() {
                "dialog"
            } else {
                "username"
            }
            .to_string(),
        ),
        access_hash: resolved.access_hash,
        avatar_cache_key,
    })?;
    let now = now_secs();

    let pool = get_pool(&handle).await?;
    let row: SourceRecordRow = sqlx::query_as(
        r#"
        INSERT INTO sources (
            source_type,
            telegram_source_kind,
            external_id,
            title,
            metadata_zstd,
            is_active,
            is_member,
            account_id,
            created_at
        )
        VALUES ('telegram', ?, ?, ?, ?, 1, ?, ?, ?)
        ON CONFLICT(account_id, source_type, telegram_source_kind, external_id) DO UPDATE SET
            title = excluded.title,
            metadata_zstd = excluded.metadata_zstd,
            is_member = excluded.is_member,
            account_id = excluded.account_id
        RETURNING
            id,
            source_type,
            telegram_source_kind,
            account_id,
            external_id,
            title,
            metadata_zstd,
            last_sync_state,
            last_synced_at,
            is_active,
            is_member,
            created_at
        "#,
    )
    .bind(&resolved.telegram_source_kind)
    .bind(&resolved.external_id)
    .bind(&resolved.title)
    .bind(metadata_zstd)
    .bind(resolved.is_member)
    .bind(account_id)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;
    source_record_from_row(&handle, row)
}

#[tauri::command]
pub async fn list_sources(
    handle: AppHandle,
    account_id: Option<i64>,
) -> AppResult<Vec<SourceRecord>> {
    let pool = get_pool(&handle).await?;
    let rows: Vec<SourceRecordRow> = if let Some(aid) = account_id {
        sqlx::query_as(
            "SELECT id, source_type, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at FROM sources WHERE account_id = ? ORDER BY created_at DESC",
        )
        .bind(aid)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            "SELECT id, source_type, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at FROM sources ORDER BY created_at DESC",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    };

    rows.into_iter()
        .map(|row| source_record_from_row(&handle, row))
        .collect()
}

fn source_record_from_row(handle: &AppHandle, row: SourceRecordRow) -> AppResult<SourceRecord> {
    let metadata = decode_source_metadata(row.metadata_zstd.as_deref())?;
    let avatar_data_url = metadata
        .avatar_cache_key
        .as_deref()
        .and_then(|cache_key| read_source_avatar_data_url(handle, cache_key));

    Ok(SourceRecord {
        id: row.id,
        source_type: row.source_type,
        telegram_source_kind: row.telegram_source_kind,
        account_id: row.account_id,
        external_id: row.external_id,
        title: row.title,
        last_sync_state: row.last_sync_state,
        last_synced_at: row.last_synced_at,
        is_member: row.is_member,
        is_active: row.is_active,
        created_at: row.created_at,
        avatar_data_url,
    })
}

#[tauri::command]
pub async fn sync_source(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    source_id: i64,
) -> AppResult<SyncResult> {
    let pool = get_pool(&handle).await?;
    let source: SourceSyncTarget = sqlx::query_as(
        "SELECT id, source_type, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state FROM sources WHERE id = ?",
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
    let refreshed_metadata_zstd =
        refresh_source_avatar_cache(&handle, &client, &source, account_id, peer).await;
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

    if let Some(metadata_zstd) = refreshed_metadata_zstd {
        sqlx::query(
            "UPDATE sources SET last_sync_state = ?, last_synced_at = ?, metadata_zstd = ? WHERE id = ?",
        )
        .bind(last_sync_state)
        .bind(sync_completed_at)
        .bind(metadata_zstd)
        .bind(source.id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        sqlx::query("UPDATE sources SET last_sync_state = ?, last_synced_at = ? WHERE id = ?")
            .bind(last_sync_state)
            .bind(sync_completed_at)
            .bind(source.id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(SyncResult {
        inserted,
        skipped,
        last_message_id: last_sync_state,
        initial_sync_policy_applied,
    })
}

async fn refresh_source_avatar_cache(
    handle: &AppHandle,
    client: &grammers_client::Client,
    source: &SourceSyncTarget,
    account_id: i64,
    peer_ref: PeerRef,
) -> Option<Vec<u8>> {
    let peer = client.resolve_peer(peer_ref).await.ok()?;
    let bytes = peer_photo_bytes_with_timeout(client, &peer).await?;
    let cache_key = cache_source_avatar(
        handle,
        account_id,
        &source.telegram_source_kind,
        &source.external_id,
        &bytes,
    )
    .ok()
    .flatten()?;

    let mut metadata = decode_source_metadata(source.metadata_zstd.as_deref()).ok()?;
    metadata.avatar_cache_key = Some(cache_key);
    encode_source_metadata(&metadata).ok()
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

async fn resolve_telegram_source(
    client: &grammers_client::Client,
    source_ref: &str,
    expected_kind: Option<&str>,
) -> Result<ResolvedTelegramSource, String> {
    let trimmed = source_ref.trim();
    let username = parse_username(trimmed);

    if !username.is_empty() && !username.chars().all(|char| char.is_ascii_digit()) {
        let peer = client
            .resolve_username(&username)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Telegram source '{}' not found", source_ref))?;

        let mut source = resolved_telegram_source_from_peer(&peer)
            .ok_or_else(|| "Not a Telegram channel, group, or supergroup".to_string())?;
        validate_expected_telegram_source_kind(&source, expected_kind)?;
        source.avatar_bytes = peer_photo_bytes_with_timeout(client, &peer).await;
        return Ok(source);
    }

    let Ok(source_id) = trimmed.parse::<i64>() else {
        return Err(format!("Telegram source '{}' not found", source_ref));
    };

    let mut dialogs = client.iter_dialogs();
    let mut found_wrong_kind = false;
    while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
        if dialog.peer().id().bare_id() == source_id {
            if let Some(source) = resolved_telegram_source_from_peer(dialog.peer()) {
                if telegram_source_kind_matches(&source, expected_kind)? {
                    let mut source = source;
                    source.avatar_bytes =
                        peer_photo_bytes_with_timeout(client, dialog.peer()).await;
                    return Ok(source);
                }
                found_wrong_kind = true;
            }
        }
    }

    if found_wrong_kind {
        return Err(format!(
            "Telegram source '{}' was found, but not as the requested source kind",
            source_ref
        ));
    }

    Err(format!(
        "Telegram source '{}' could not be found in this account's dialogs",
        source_ref
    ))
}

fn telegram_source_kind_matches(
    source: &ResolvedTelegramSource,
    expected_kind: Option<&str>,
) -> Result<bool, String> {
    let Some(expected_kind) = expected_kind else {
        return Ok(true);
    };

    ensure_supported_telegram_source_kind(expected_kind)?;
    Ok(source.telegram_source_kind == expected_kind)
}

fn validate_expected_telegram_source_kind(
    source: &ResolvedTelegramSource,
    expected_kind: Option<&str>,
) -> Result<(), String> {
    if telegram_source_kind_matches(source, expected_kind)? {
        Ok(())
    } else {
        Err("Resolved Telegram source has a different source kind".to_string())
    }
}

fn ensure_supported_telegram_source_kind(kind: &str) -> Result<(), String> {
    match kind {
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP | TELEGRAM_KIND_GROUP => Ok(()),
        other => Err(format!("Unsupported telegram_source_kind '{other}'")),
    }
}

fn resolved_telegram_source_from_peer(peer: &Peer) -> Option<ResolvedTelegramSource> {
    telegram_source_info_from_peer(peer).map(|source| ResolvedTelegramSource {
        external_id: source.id.to_string(),
        title: source.title,
        telegram_source_kind: source.telegram_source_kind,
        is_member: source.is_member,
        username: source.username,
        access_hash: peer_access_hash(peer),
        avatar_bytes: None,
    })
}

fn peer_access_hash(peer: &Peer) -> Option<i64> {
    match peer {
        Peer::Channel(channel) => channel.raw.access_hash,
        Peer::Group(group) => match &group.raw {
            tl::enums::Chat::Channel(channel) => channel.access_hash,
            tl::enums::Chat::ChannelForbidden(channel) => Some(channel.access_hash),
            tl::enums::Chat::Empty(_)
            | tl::enums::Chat::Chat(_)
            | tl::enums::Chat::Forbidden(_) => None,
        },
        Peer::User(_) => None,
    }
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
    if source.source_type != TELEGRAM_SOURCE_TYPE {
        return Err(format!(
            "Source {} has unsupported source_type '{}'",
            source.id, source.source_type
        ));
    }

    let telegram_source_id = source.external_id.parse::<i64>().map_err(|_| {
        format!(
            "Invalid external_id '{}' for source {}",
            source.external_id, source.id
        )
    })?;

    let metadata = decode_source_metadata(source.metadata_zstd.as_deref())?;
    if let Some(username) = metadata.username.as_deref() {
        if let Some(peer) = client
            .resolve_username(username)
            .await
            .map_err(|e| e.to_string())?
        {
            return peer_ref_for_source_kind(&peer, &source.telegram_source_kind, source.id);
        }
    }

    if let Some(peer_ref) = source_peer_ref_from_metadata(&source, telegram_source_id, &metadata)? {
        return Ok(peer_ref);
    }

    let mut dialogs = client.iter_dialogs();
    while let Some(dialog) = dialogs.next().await.map_err(|e| e.to_string())? {
        if dialog.peer().id().bare_id() == telegram_source_id {
            return peer_ref_for_source_kind(
                dialog.peer(),
                &source.telegram_source_kind,
                source.id,
            );
        }
    }

    Err(format!(
        "Source {} could not be resolved from stored username, peer identity metadata, or dialogs. If this is a private Telegram source, re-add it from the account's dialogs.",
        source.id
    ))
}

fn source_peer_ref_from_metadata(
    source: &SourceSyncTarget,
    telegram_source_id: i64,
    metadata: &SourceMetadata,
) -> Result<Option<PeerRef>, String> {
    let Some(access_hash) = metadata.access_hash else {
        return Ok(None);
    };

    match source.telegram_source_kind.as_str() {
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP => Ok(Some(PeerRef {
            id: PeerId::channel(telegram_source_id),
            auth: PeerAuth::from_hash(access_hash),
        })),
        TELEGRAM_KIND_GROUP => Ok(None),
        other => Err(format!(
            "Source {} has unsupported telegram_source_kind '{}'",
            source.id, other
        )),
    }
}

fn peer_ref_for_source_kind(
    peer: &Peer,
    telegram_source_kind: &str,
    source_id: i64,
) -> Result<PeerRef, String> {
    match (telegram_source_kind, peer) {
        (TELEGRAM_KIND_CHANNEL, Peer::Channel(channel)) => Ok(channel.raw.clone().into()),
        (TELEGRAM_KIND_SUPERGROUP, Peer::Group(group)) if group.is_megagroup() => {
            Ok(group.raw.clone().into())
        }
        (TELEGRAM_KIND_GROUP, Peer::Group(group)) if !group.is_megagroup() => {
            Ok(group.raw.clone().into())
        }
        (TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP | TELEGRAM_KIND_GROUP, _) => {
            Err(format!(
                "Source {} resolved to a different Telegram source kind",
                source_id
            ))
        }
        (other, _) => Err(format!(
            "Source {} has unsupported telegram_source_kind '{}'",
            source_id, other
        )),
    }
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
        compress_text, decode_media_metadata, decode_source_metadata, decompress_text,
        default_sync_settings, derive_content_kind, derive_document_media_kind,
        encode_media_metadata, encode_source_metadata, initial_sync_policy_label,
        load_sync_settings_from_pool, save_sync_settings_to_pool, source_peer_ref_from_metadata,
        validate_sync_settings, DocumentSignals, InitialSyncMode, ItemMediaMetadata,
        SourceMetadata, SourceSyncTarget, SyncSettingsRecord, CONTENT_KIND_MEDIA_ONLY,
        CONTENT_KIND_TEXT_ONLY, CONTENT_KIND_TEXT_WITH_MEDIA, TELEGRAM_KIND_CHANNEL,
        TELEGRAM_KIND_GROUP, TELEGRAM_SOURCE_TYPE,
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
    fn source_metadata_decodes_old_username_only_payloads() {
        let encoded = super::compress_json_bytes(br#"{"username":"example"}"#).expect("encode");
        let decoded = decode_source_metadata(Some(&encoded)).expect("decode");

        assert_eq!(decoded.username.as_deref(), Some("example"));
        assert_eq!(decoded.added_from, None);
        assert_eq!(decoded.access_hash, None);
        assert_eq!(decoded.avatar_cache_key, None);
    }

    #[test]
    fn source_metadata_roundtrip_preserves_peer_identity() {
        let original = SourceMetadata {
            username: Some("example".to_string()),
            added_from: Some("dialog".to_string()),
            access_hash: Some(42),
            avatar_cache_key: Some("1_channel_42.jpg".to_string()),
        };

        let encoded = encode_source_metadata(&original).expect("encode");
        let decoded = decode_source_metadata(Some(&encoded)).expect("decode");

        assert_eq!(decoded.username, original.username);
        assert_eq!(decoded.added_from, original.added_from);
        assert_eq!(decoded.access_hash, original.access_hash);
        assert_eq!(decoded.avatar_cache_key, original.avatar_cache_key);
    }

    #[test]
    fn peer_ref_from_metadata_uses_channel_access_hash() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            telegram_source_kind: TELEGRAM_KIND_CHANNEL.to_string(),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            metadata_zstd: None,
            last_sync_state: None,
        };
        let metadata = SourceMetadata {
            access_hash: Some(67890),
            ..SourceMetadata::default()
        };

        let peer_ref = source_peer_ref_from_metadata(&source, 12345, &metadata)
            .expect("metadata peer ref")
            .expect("peer ref");

        assert_eq!(peer_ref.id.bare_id(), 12345);
        assert_eq!(peer_ref.auth.hash(), 67890);
    }

    #[test]
    fn peer_ref_from_metadata_ignores_small_groups_without_access_hash_identity() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            telegram_source_kind: TELEGRAM_KIND_GROUP.to_string(),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            metadata_zstd: None,
            last_sync_state: None,
        };
        let metadata = SourceMetadata {
            access_hash: Some(67890),
            ..SourceMetadata::default()
        };

        let peer_ref =
            source_peer_ref_from_metadata(&source, 12345, &metadata).expect("metadata peer ref");

        assert!(peer_ref.is_none());
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
