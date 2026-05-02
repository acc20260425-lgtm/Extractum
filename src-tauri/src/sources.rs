use base64::{engine::general_purpose, Engine as _};
use grammers_client::{peer::Peer, tl};
use grammers_session::types::{PeerAuth, PeerId, PeerRef};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};
use tokio::time::{timeout, Duration, Instant};

use crate::compression::{compress_json_bytes, compress_text, decompress_bytes, decompress_text};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::media::{
    decode_media_metadata, encode_media_metadata, extract_item_payload, ExtractedItemPayload,
};
use crate::telegram::TelegramState;

const DEFAULT_INITIAL_SYNC_MESSAGE_LIMIT: i64 = 500;
const MIN_INITIAL_SYNC_MESSAGE_LIMIT: i64 = 50;
const MAX_INITIAL_SYNC_MESSAGE_LIMIT: i64 = 5_000;
const DEFAULT_INITIAL_SYNC_DAY_LIMIT: i64 = 30;
const MIN_INITIAL_SYNC_DAY_LIMIT: i64 = 1;
const MAX_INITIAL_SYNC_DAY_LIMIT: i64 = 365;
const INITIAL_SYNC_MODE_SETTING_KEY: &str = "sync.initial.mode";
const INITIAL_SYNC_VALUE_SETTING_KEY: &str = "sync.initial.value";
const SECONDS_PER_DAY: i64 = 86_400;
const TELEGRAM_SOURCE_TYPE: &str = "telegram";
const TELEGRAM_KIND_CHANNEL: &str = "channel";
const TELEGRAM_KIND_SUPERGROUP: &str = "supergroup";
const TELEGRAM_KIND_GROUP: &str = "group";
const TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS: u64 = 750;
const TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS: u64 = 4_000;
const TELEGRAM_SOURCE_AVATAR_CACHE_DIR: &str = "source_avatars";
const FORUM_TOPIC_UNCATEGORIZED_KEY: &str = "unrecognized_topic";
const FORUM_TOPIC_UNCATEGORIZED_TITLE: &str = "Unrecognized topic";

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
    pub warnings: Vec<String>,
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
    pub forum_topic_id: Option<i64>,
    pub forum_topic_title: Option<String>,
    pub forum_topic_top_message_id: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ForumTopicFilter {
    Topic { topic_id: i64 },
    Uncategorized,
}

#[derive(Serialize)]
pub struct SourceForumTopicRecord {
    pub kind: String,
    pub key: String,
    pub title: String,
    pub message_count: i64,
    pub topic_id: Option<i64>,
    pub top_message_id: Option<i64>,
    pub icon_color: Option<i64>,
    pub icon_emoji_id: Option<i64>,
    pub is_closed: bool,
    pub is_pinned: bool,
    pub is_hidden: bool,
    pub is_deleted: bool,
    pub sort_order: Option<i64>,
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
    forum_topic_id: Option<i64>,
    forum_topic_title: Option<String>,
    forum_topic_top_message_id: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct SourceForumTopicRow {
    topic_id: i64,
    top_message_id: i64,
    title: String,
    icon_color: Option<i64>,
    icon_emoji_id: Option<i64>,
    is_closed: bool,
    is_pinned: bool,
    is_hidden: bool,
    is_deleted: bool,
    sort_order: Option<i64>,
    message_count: i64,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SourcePeerResolutionStrategy {
    Username,
    Dialog,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
struct SourcePeerIdentity {
    strategy: SourcePeerResolutionStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    access_hash: Option<i64>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
struct SourceMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    peer_identity: Option<SourcePeerIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    avatar_cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    added_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    access_hash: Option<i64>,
}

impl SourcePeerIdentity {
    fn has_username(&self) -> bool {
        self.username
            .as_deref()
            .is_some_and(|username| !username.trim().is_empty())
    }
}

impl SourceMetadata {
    fn normalized(&self) -> Self {
        let mut normalized = self.clone();

        if normalized.peer_identity.is_none() {
            normalized.peer_identity = legacy_peer_identity(
                normalized.username.clone(),
                normalized.added_from.clone(),
                normalized.access_hash,
            );
        }

        normalized.username = None;
        normalized.added_from = None;
        normalized.access_hash = None;
        normalized
    }
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

struct ResolvedSyncPeer {
    peer: PeerRef,
    refreshed_metadata_zstd: Option<Vec<u8>>,
}

struct SyncPolicy {
    previous_last_sync: i64,
    initial_sync_settings: Option<SyncSettingsRecord>,
    initial_sync_policy_applied: Option<String>,
    initial_sync_cutoff: Option<i64>,
}

struct IngestOutcome {
    inserted: i64,
    skipped: i64,
    max_message_id: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ForumTopicSnapshot {
    topic_id: i64,
    top_message_id: i64,
    title: String,
    icon_color: i64,
    icon_emoji_id: Option<i64>,
    is_closed: bool,
    is_pinned: bool,
    is_hidden: bool,
    sort_order: i64,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct TelegramItemContext {
    reply_to_msg_id: Option<i64>,
    reply_to_peer_kind: Option<String>,
    reply_to_peer_id: Option<String>,
    reply_to_top_id: Option<i64>,
    reaction_count: Option<i64>,
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

async fn load_source(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<SourceSyncTarget> {
    sqlx::query_as(
        "SELECT id, source_type, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state FROM sources WHERE id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| AppError::not_found(format!("Source {source_id} not found")))
}

async fn get_authorized_client(
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
) -> AppResult<grammers_client::Client> {
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

    Ok(client)
}

async fn resolve_and_refresh_peer(
    handle: &AppHandle,
    client: &grammers_client::Client,
    source: &SourceSyncTarget,
    account_id: i64,
) -> Result<ResolvedSyncPeer, String> {
    let peer = resolve_source_peer(client, source).await?;
    let refreshed_metadata_zstd =
        refresh_source_avatar_cache(handle, client, source, account_id, peer).await;

    Ok(ResolvedSyncPeer {
        peer,
        refreshed_metadata_zstd,
    })
}

async fn determine_sync_policy(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &SourceSyncTarget,
) -> AppResult<SyncPolicy> {
    let previous_last_sync = source.last_sync_state.unwrap_or(0);
    let initial_sync_settings = if previous_last_sync == 0 {
        Some(load_sync_settings_from_pool(pool).await?)
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

    Ok(SyncPolicy {
        previous_last_sync,
        initial_sync_settings,
        initial_sync_policy_applied,
        initial_sync_cutoff,
    })
}

async fn persist_items(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &grammers_client::Client,
    peer: PeerRef,
    source: &SourceSyncTarget,
    sync_policy: &SyncPolicy,
) -> Result<IngestOutcome, String> {
    let mut inserted = 0_i64;
    let mut skipped = 0_i64;
    let mut max_message_id = sync_policy.previous_last_sync;
    let mut messages = if let Some(settings) = sync_policy.initial_sync_settings.as_ref() {
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
        if sync_policy.previous_last_sync > 0 && message_id <= sync_policy.previous_last_sync {
            break;
        }
        let published_at = message.date().timestamp();
        if let Some(cutoff) = sync_policy.initial_sync_cutoff {
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
        let telegram_context = extract_telegram_context(&message);
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
                media_metadata_zstd,
                reply_to_msg_id,
                reply_to_peer_kind,
                reply_to_peer_id,
                reply_to_top_id,
                reaction_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(telegram_context.reply_to_msg_id)
        .bind(&telegram_context.reply_to_peer_kind)
        .bind(&telegram_context.reply_to_peer_id)
        .bind(telegram_context.reply_to_top_id)
        .bind(telegram_context.reaction_count)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        if result.rows_affected() == 1 {
            inserted += 1;
        } else {
            skipped += 1;
        }
    }

    Ok(IngestOutcome {
        inserted,
        skipped,
        max_message_id,
    })
}

async fn finalize_sync(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &SourceSyncTarget,
    previous_last_sync: i64,
    max_message_id: i64,
    refreshed_metadata_zstd: Option<Vec<u8>>,
) -> Result<Option<i64>, String> {
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
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        sqlx::query("UPDATE sources SET last_sync_state = ?, last_synced_at = ? WHERE id = ?")
            .bind(last_sync_state)
            .bind(sync_completed_at)
            .bind(source.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(last_sync_state)
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
    let metadata_zstd = encode_source_metadata(&source_metadata_for_added_source(
        &source_ref,
        telegram_source_kind.as_deref(),
        &resolved,
        avatar_cache_key,
    ))?;
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
    let source = load_source(&pool, source_id).await?;

    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;

    let client = get_authorized_client(state, account_id).await?;
    let resolved_peer = resolve_and_refresh_peer(&handle, &client, &source, account_id).await?;
    let forum_topic_warnings =
        refresh_forum_topics(&pool, &client, resolved_peer.peer.clone(), &source).await;
    let sync_policy = determine_sync_policy(&pool, &source).await?;
    let ingest = persist_items(
        &pool,
        &client,
        resolved_peer.peer.clone(),
        &source,
        &sync_policy,
    )
    .await?;
    let last_sync_state = finalize_sync(
        &pool,
        &source,
        sync_policy.previous_last_sync,
        ingest.max_message_id,
        resolved_peer.refreshed_metadata_zstd,
    )
    .await?;

    Ok(SyncResult {
        inserted: ingest.inserted,
        skipped: ingest.skipped,
        last_message_id: last_sync_state,
        initial_sync_policy_applied: sync_policy.initial_sync_policy_applied,
        warnings: forum_topic_warnings,
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

async fn refresh_forum_topics(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &grammers_client::Client,
    peer: PeerRef,
    source: &SourceSyncTarget,
) -> Vec<String> {
    if source.telegram_source_kind != TELEGRAM_KIND_SUPERGROUP {
        return Vec::new();
    }

    match fetch_all_forum_topics(client, peer).await {
        Ok((topics, deleted_topic_ids)) => {
            if let Err(error) = upsert_forum_topics_from_refresh(
                pool,
                source.id,
                &topics,
                &deleted_topic_ids,
                now_secs(),
            )
            .await
            {
                vec![format!(
                    "Forum topic refresh failed for source {}: {error}",
                    source.id
                )]
            } else {
                Vec::new()
            }
        }
        Err(error) if is_non_forum_topic_refresh_error(&error) => Vec::new(),
        Err(error) => vec![format!(
            "Forum topic refresh failed for source {}: {error}",
            source.id
        )],
    }
}

async fn fetch_all_forum_topics(
    client: &grammers_client::Client,
    peer: PeerRef,
) -> Result<(Vec<ForumTopicSnapshot>, Vec<i64>), String> {
    let mut topics = Vec::new();
    let mut deleted_topic_ids = Vec::new();
    let mut offset_date = 0_i32;
    let mut offset_id = 0_i32;
    let mut offset_topic = 0_i32;
    let mut sort_order = 0_i64;

    loop {
        let response = client
            .invoke(&tl::functions::messages::GetForumTopics {
                peer: peer.into(),
                q: None,
                offset_date,
                offset_id,
                offset_topic,
                limit: 100,
            })
            .await
            .map_err(|e| e.to_string())?;

        let forum_topics = match response {
            tl::enums::messages::ForumTopics::Topics(topics) => topics,
        };

        if forum_topics.topics.is_empty() {
            break;
        }

        let last_cursor = forum_topic_page_cursor(&forum_topics);
        let page_topics = forum_topics.topics;
        for topic in page_topics {
            match topic {
                tl::enums::ForumTopic::Topic(topic) => {
                    topics.push(ForumTopicSnapshot {
                        topic_id: i64::from(topic.id),
                        top_message_id: i64::from(topic.top_message),
                        title: topic.title,
                        icon_color: i64::from(topic.icon_color),
                        icon_emoji_id: topic.icon_emoji_id,
                        is_closed: topic.closed,
                        is_pinned: topic.pinned,
                        is_hidden: topic.hidden,
                        sort_order,
                    });
                    sort_order += 1;
                }
                tl::enums::ForumTopic::Deleted(topic) => {
                    deleted_topic_ids.push(i64::from(topic.id));
                }
            }
        }

        let Some((next_offset_date, next_offset_id, next_offset_topic)) = last_cursor else {
            break;
        };
        if next_offset_date == offset_date
            && next_offset_id == offset_id
            && next_offset_topic == offset_topic
        {
            break;
        }

        offset_date = next_offset_date;
        offset_id = next_offset_id;
        offset_topic = next_offset_topic;
    }

    Ok((topics, deleted_topic_ids))
}

fn forum_topic_page_cursor(
    forum_topics: &tl::types::messages::ForumTopics,
) -> Option<(i32, i32, i32)> {
    let last_topic = forum_topics
        .topics
        .iter()
        .rev()
        .find_map(|topic| match topic {
            tl::enums::ForumTopic::Topic(topic) => Some(topic),
            tl::enums::ForumTopic::Deleted(_) => None,
        })?;
    let offset_date = forum_topics
        .messages
        .iter()
        .find(|message| message.id() == last_topic.top_message)
        .and_then(forum_topic_message_date)
        .unwrap_or(last_topic.date);

    Some((offset_date, last_topic.top_message, last_topic.id))
}

fn forum_topic_message_date(message: &tl::enums::Message) -> Option<i32> {
    match message {
        tl::enums::Message::Empty(_) => None,
        tl::enums::Message::Message(message) => Some(message.date),
        tl::enums::Message::Service(message) => Some(message.date),
    }
}

async fn upsert_forum_topics_from_refresh(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    topics: &[ForumTopicSnapshot],
    deleted_topic_ids: &[i64],
    refreshed_at: i64,
) -> Result<(), String> {
    for topic in topics {
        sqlx::query(
            r#"
            INSERT INTO telegram_forum_topics (
                source_id,
                topic_id,
                top_message_id,
                title,
                icon_color,
                icon_emoji_id,
                is_closed,
                is_pinned,
                is_hidden,
                is_deleted,
                sort_order,
                last_seen_at,
                updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?)
            ON CONFLICT(source_id, topic_id) DO UPDATE SET
                top_message_id = excluded.top_message_id,
                title = excluded.title,
                icon_color = excluded.icon_color,
                icon_emoji_id = excluded.icon_emoji_id,
                is_closed = excluded.is_closed,
                is_pinned = excluded.is_pinned,
                is_hidden = excluded.is_hidden,
                is_deleted = 0,
                sort_order = excluded.sort_order,
                last_seen_at = excluded.last_seen_at,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(source_id)
        .bind(topic.topic_id)
        .bind(topic.top_message_id)
        .bind(&topic.title)
        .bind(topic.icon_color)
        .bind(topic.icon_emoji_id)
        .bind(topic.is_closed)
        .bind(topic.is_pinned)
        .bind(topic.is_hidden)
        .bind(topic.sort_order)
        .bind(refreshed_at)
        .bind(refreshed_at)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    for topic_id in deleted_topic_ids {
        sqlx::query(
            r#"
            UPDATE telegram_forum_topics
            SET is_deleted = 1, last_seen_at = ?, updated_at = ?
            WHERE source_id = ? AND topic_id = ?
            "#,
        )
        .bind(refreshed_at)
        .bind(refreshed_at)
        .bind(source_id)
        .bind(topic_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn is_non_forum_topic_refresh_error(error: &str) -> bool {
    error.contains("CHANNEL_FORUM_MISSING") || error.contains("CHANNEL_MONOFORUM_UNSUPPORTED")
}

#[tauri::command]
pub async fn get_items(
    handle: AppHandle,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
) -> AppResult<Vec<ItemRecord>> {
    let pool = get_pool(&handle).await?;
    let limit = limit.clamp(1, 200);
    let rows = load_item_rows_from_pool(&pool, source_id, limit, before_published_at, topic_filter)
        .await?;

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
                forum_topic_id: row.forum_topic_id,
                forum_topic_title: row.forum_topic_title,
                forum_topic_top_message_id: row.forum_topic_top_message_id,
            })
        })
        .collect::<Result<Vec<_>, String>>()?)
}

#[tauri::command]
pub async fn list_source_forum_topics(
    handle: AppHandle,
    source_id: i64,
) -> AppResult<Vec<SourceForumTopicRecord>> {
    let pool = get_pool(&handle).await?;
    list_source_forum_topics_from_pool(&pool, source_id).await
}

async fn load_item_rows_from_pool(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
) -> AppResult<Vec<StoredItemRow>> {
    let mut sql = String::from(
        r#"
        SELECT
            items.id,
            items.source_id,
            items.external_id,
            items.author,
            items.published_at,
            items.content_kind,
            items.has_media,
            items.media_kind,
            items.content_zstd,
            items.media_metadata_zstd,
            items.raw_data_zstd,
            forum_topics.topic_id AS forum_topic_id,
            forum_topics.title AS forum_topic_title,
            forum_topics.top_message_id AS forum_topic_top_message_id
        FROM items
        LEFT JOIN telegram_forum_topics AS forum_topics
          ON forum_topics.source_id = items.source_id
         AND (
                items.reply_to_top_id = forum_topics.topic_id
                OR (
                    items.reply_to_top_id IS NULL
                    AND items.external_id <> ''
                    AND items.external_id NOT GLOB '*[^0-9]*'
                    AND CAST(items.external_id AS INTEGER) = forum_topics.top_message_id
                )
                OR (
                    items.reply_to_top_id IS NULL
                    AND items.reply_to_msg_id = forum_topics.topic_id
                )
                OR (
                    items.reply_to_top_id IS NULL
                    AND forum_topics.topic_id = 1
                    AND NOT EXISTS (
                        SELECT 1
                        FROM telegram_forum_topics AS matched_topics
                        WHERE matched_topics.source_id = items.source_id
                          AND (
                                (
                                    items.external_id <> ''
                                    AND items.external_id NOT GLOB '*[^0-9]*'
                                    AND CAST(items.external_id AS INTEGER) = matched_topics.top_message_id
                                )
                                OR items.reply_to_msg_id = matched_topics.topic_id
                          )
                    )
                )
            )
        WHERE items.source_id = ?
        "#,
    );

    if before_published_at.is_some() {
        sql.push_str(" AND items.published_at < ?");
    }

    match topic_filter {
        Some(ForumTopicFilter::Topic { .. }) => {
            sql.push_str(" AND forum_topics.topic_id = ?");
        }
        Some(ForumTopicFilter::Uncategorized) => {
            sql.push_str(" AND forum_topics.topic_id IS NULL");
        }
        None => {}
    }

    sql.push_str(" ORDER BY items.published_at DESC LIMIT ?");

    let mut query = sqlx::query_as::<_, StoredItemRow>(&sql).bind(source_id);
    if let Some(before) = before_published_at {
        query = query.bind(before);
    }
    if let Some(ForumTopicFilter::Topic { topic_id }) = topic_filter {
        query = query.bind(topic_id);
    }

    query
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::from(e.to_string()))
}

async fn list_source_forum_topics_from_pool(
    pool: &sqlx::SqlitePool,
    source_id: i64,
) -> AppResult<Vec<SourceForumTopicRecord>> {
    let rows: Vec<SourceForumTopicRow> = sqlx::query_as(
        r#"
        SELECT
            topics.topic_id,
            topics.top_message_id,
            topics.title,
            topics.icon_color,
            topics.icon_emoji_id,
            topics.is_closed,
            topics.is_pinned,
            topics.is_hidden,
            topics.is_deleted,
            topics.sort_order,
            COUNT(items.id) AS message_count
        FROM telegram_forum_topics AS topics
        LEFT JOIN items
          ON items.source_id = topics.source_id
         AND (
                items.reply_to_top_id = topics.topic_id
                OR (
                    items.reply_to_top_id IS NULL
                    AND items.external_id <> ''
                    AND items.external_id NOT GLOB '*[^0-9]*'
                    AND CAST(items.external_id AS INTEGER) = topics.top_message_id
                )
                OR (
                    items.reply_to_top_id IS NULL
                    AND items.reply_to_msg_id = topics.topic_id
                )
                OR (
                    items.reply_to_top_id IS NULL
                    AND topics.topic_id = 1
                    AND NOT EXISTS (
                        SELECT 1
                        FROM telegram_forum_topics AS matched_topics
                        WHERE matched_topics.source_id = items.source_id
                          AND (
                                (
                                    items.external_id <> ''
                                    AND items.external_id NOT GLOB '*[^0-9]*'
                                    AND CAST(items.external_id AS INTEGER) = matched_topics.top_message_id
                                )
                                OR items.reply_to_msg_id = matched_topics.topic_id
                          )
                    )
                )
            )
        WHERE topics.source_id = ?
        GROUP BY
            topics.topic_id,
            topics.top_message_id,
            topics.title,
            topics.icon_color,
            topics.icon_emoji_id,
            topics.is_closed,
            topics.is_pinned,
            topics.is_hidden,
            topics.is_deleted,
            topics.sort_order
        ORDER BY
            topics.is_pinned DESC,
            topics.sort_order ASC NULLS LAST,
            topics.title COLLATE NOCASE ASC,
            topics.topic_id ASC
        "#,
    )
    .bind(source_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::from(e.to_string()))?;

    let uncategorized_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM items
        LEFT JOIN telegram_forum_topics AS forum_topics
          ON forum_topics.source_id = items.source_id
         AND (
                items.reply_to_top_id = forum_topics.topic_id
                OR (
                    items.reply_to_top_id IS NULL
                    AND items.external_id <> ''
                    AND items.external_id NOT GLOB '*[^0-9]*'
                    AND CAST(items.external_id AS INTEGER) = forum_topics.top_message_id
                )
                OR (
                    items.reply_to_top_id IS NULL
                    AND items.reply_to_msg_id = forum_topics.topic_id
                )
                OR (
                    items.reply_to_top_id IS NULL
                    AND forum_topics.topic_id = 1
                    AND NOT EXISTS (
                        SELECT 1
                        FROM telegram_forum_topics AS matched_topics
                        WHERE matched_topics.source_id = items.source_id
                          AND (
                                (
                                    items.external_id <> ''
                                    AND items.external_id NOT GLOB '*[^0-9]*'
                                    AND CAST(items.external_id AS INTEGER) = matched_topics.top_message_id
                                )
                                OR items.reply_to_msg_id = matched_topics.topic_id
                          )
                    )
                )
            )
        WHERE items.source_id = ?
          AND forum_topics.topic_id IS NULL
        "#,
    )
    .bind(source_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::from(e.to_string()))?;

    let mut records = rows
        .into_iter()
        .map(|row| SourceForumTopicRecord {
            kind: "topic".to_string(),
            key: format!("topic:{}", row.topic_id),
            title: row.title,
            message_count: row.message_count,
            topic_id: Some(row.topic_id),
            top_message_id: Some(row.top_message_id),
            icon_color: row.icon_color,
            icon_emoji_id: row.icon_emoji_id,
            is_closed: row.is_closed,
            is_pinned: row.is_pinned,
            is_hidden: row.is_hidden,
            is_deleted: row.is_deleted,
            sort_order: row.sort_order,
        })
        .collect::<Vec<_>>();

    if uncategorized_count > 0 {
        records.push(SourceForumTopicRecord {
            kind: "uncategorized".to_string(),
            key: FORUM_TOPIC_UNCATEGORIZED_KEY.to_string(),
            title: FORUM_TOPIC_UNCATEGORIZED_TITLE.to_string(),
            message_count: uncategorized_count,
            topic_id: None,
            top_message_id: None,
            icon_color: None,
            icon_emoji_id: None,
            is_closed: false,
            is_pinned: false,
            is_hidden: false,
            is_deleted: false,
            sort_order: None,
        });
    }

    Ok(records)
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum ManualTelegramSourceRef {
    Username(String),
    NumericId(i64),
}

fn unsupported_manual_source_ref_message(source_ref: &str) -> String {
    format!(
        "Unsupported manual Telegram source reference '{}'. Use @username or t.me/name for public sources. For private Telegram sources, add them from the account's dialogs.",
        source_ref
    )
}

fn unsupported_private_manual_source_ref_message(source_ref: &str) -> String {
    format!(
        "Unsupported private Telegram source reference '{}'. Private invite links and internal t.me/c links are not supported for manual add. Add this source from the account's dialogs instead.",
        source_ref
    )
}

fn parse_supported_manual_telegram_source_ref(
    source_ref: &str,
) -> Result<ManualTelegramSourceRef, String> {
    let trimmed = source_ref.trim();
    if trimmed.is_empty() {
        return Err("Telegram source reference cannot be empty".to_string());
    }

    if let Ok(source_id) = trimmed.parse::<i64>() {
        return Ok(ManualTelegramSourceRef::NumericId(source_id));
    }

    if let Some(rest) = trimmed.strip_prefix('@') {
        let username = rest.trim();
        if username.is_empty() || username.contains('/') || username.starts_with('+') {
            return Err(unsupported_manual_source_ref_message(source_ref));
        }
        return Ok(ManualTelegramSourceRef::Username(username.to_string()));
    }

    if let Some(rest) = trimmed
        .strip_prefix("https://t.me/")
        .or_else(|| trimmed.strip_prefix("http://t.me/"))
        .or_else(|| trimmed.strip_prefix("t.me/"))
    {
        let path = rest.trim_matches('/');
        let first_segment = path.split('/').next().unwrap_or(path).trim();
        if first_segment.is_empty() {
            return Err(unsupported_manual_source_ref_message(source_ref));
        }
        if first_segment.eq_ignore_ascii_case("joinchat")
            || first_segment.eq_ignore_ascii_case("c")
            || first_segment.starts_with('+')
        {
            return Err(unsupported_private_manual_source_ref_message(source_ref));
        }
        return Ok(ManualTelegramSourceRef::Username(first_segment.to_string()));
    }

    let username = parse_username(trimmed);
    if !username.is_empty()
        && !username.contains('/')
        && !username.starts_with('+')
        && !username.chars().all(|char| char.is_ascii_digit())
    {
        return Ok(ManualTelegramSourceRef::Username(username));
    }

    Err(unsupported_manual_source_ref_message(source_ref))
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourcePeerResolutionStep {
    Username,
    StoredPeerIdentity,
    DialogScan,
}

fn legacy_peer_identity(
    username: Option<String>,
    added_from: Option<String>,
    access_hash: Option<i64>,
) -> Option<SourcePeerIdentity> {
    if username.is_none() && access_hash.is_none() {
        return None;
    }

    let strategy = match added_from
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("dialog") => SourcePeerResolutionStrategy::Dialog,
        Some("username") => SourcePeerResolutionStrategy::Username,
        _ if username.is_some() => SourcePeerResolutionStrategy::Username,
        _ => SourcePeerResolutionStrategy::Dialog,
    };

    Some(SourcePeerIdentity {
        strategy,
        username,
        access_hash,
    })
}

fn add_source_resolution_strategy(
    source_ref: &str,
    telegram_source_kind: Option<&str>,
) -> SourcePeerResolutionStrategy {
    if telegram_source_kind.is_some() {
        return SourcePeerResolutionStrategy::Dialog;
    }

    let username = parse_username(source_ref);
    if username.is_empty() || username.chars().all(|char| char.is_ascii_digit()) {
        SourcePeerResolutionStrategy::Dialog
    } else {
        SourcePeerResolutionStrategy::Username
    }
}

fn source_metadata_for_added_source(
    source_ref: &str,
    telegram_source_kind: Option<&str>,
    resolved: &ResolvedTelegramSource,
    avatar_cache_key: Option<String>,
) -> SourceMetadata {
    SourceMetadata {
        peer_identity: Some(SourcePeerIdentity {
            strategy: add_source_resolution_strategy(source_ref, telegram_source_kind),
            username: resolved.username.clone(),
            access_hash: resolved.access_hash,
        }),
        avatar_cache_key,
        ..SourceMetadata::default()
    }
}

fn source_peer_resolution_plan(metadata: &SourceMetadata) -> Vec<SourcePeerResolutionStep> {
    let Some(identity) = metadata.peer_identity.as_ref() else {
        return vec![SourcePeerResolutionStep::DialogScan];
    };

    let mut plan = Vec::new();
    match identity.strategy {
        SourcePeerResolutionStrategy::Username => {
            if identity.has_username() {
                plan.push(SourcePeerResolutionStep::Username);
            }
        }
        SourcePeerResolutionStrategy::Dialog => {
            if identity.access_hash.is_some() {
                plan.push(SourcePeerResolutionStep::StoredPeerIdentity);
            }
            if identity.has_username() {
                plan.push(SourcePeerResolutionStep::Username);
            }
        }
    }

    plan.push(SourcePeerResolutionStep::DialogScan);
    plan
}

fn source_peer_resolution_failure(source: &SourceSyncTarget, metadata: &SourceMetadata) -> String {
    match metadata
        .peer_identity
        .as_ref()
        .map(|identity| identity.strategy)
    {
        Some(SourcePeerResolutionStrategy::Username) => {
            let username = metadata
                .peer_identity
                .as_ref()
                .and_then(|identity| identity.username.as_deref())
                .unwrap_or("unknown");
            format!(
                "Source {} could not be resolved from stored username '{}' or compatibility dialog scanning. If the public username changed or the source became private, re-add it from the account's dialogs.",
                source.id, username
            )
        }
        Some(SourcePeerResolutionStrategy::Dialog)
            if source.telegram_source_kind == TELEGRAM_KIND_GROUP =>
        {
            format!(
                "Source {} could not be resolved from dialogs. Small Telegram groups still depend on dialog availability; if this group disappeared from the account's dialogs, re-add it from that account.",
                source.id
            )
        }
        Some(SourcePeerResolutionStrategy::Dialog) => format!(
            "Source {} could not be resolved from stored peer identity or dialogs. If this private Telegram source disappeared from the account's dialogs, re-add it from that account.",
            source.id
        ),
        None => format!(
            "Source {} could not be resolved from compatibility dialog scanning. If this is a private Telegram source, re-add it from the account's dialogs.",
            source.id
        ),
    }
}

async fn resolve_telegram_source_by_username(
    client: &grammers_client::Client,
    username: &str,
    source_ref: &str,
    expected_kind: Option<&str>,
) -> Result<ResolvedTelegramSource, String> {
    let peer = client
        .resolve_username(username)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Telegram source '{}' not found", source_ref))?;

    let mut source = resolved_telegram_source_from_peer(&peer)
        .ok_or_else(|| "Not a Telegram channel, group, or supergroup".to_string())?;
    validate_expected_telegram_source_kind(&source, expected_kind)?;
    source.avatar_bytes = peer_photo_bytes_with_timeout(client, &peer).await;
    Ok(source)
}

fn dialog_lookup_not_found_message(source_ref: &str, expected_kind: Option<&str>) -> String {
    if expected_kind.is_some() {
        format!(
            "Telegram source '{}' was not found in this account's dialogs",
            source_ref
        )
    } else {
        format!(
            "Telegram source '{}' was not found in this account's dialogs. Numeric manual adds only work for sources that are still visible in that account's dialogs. For private Telegram sources, add them from the account's dialogs instead.",
            source_ref
        )
    }
}

async fn resolve_telegram_source_from_dialogs(
    client: &grammers_client::Client,
    source_id: i64,
    source_ref: &str,
    expected_kind: Option<&str>,
) -> Result<ResolvedTelegramSource, String> {
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
            "Telegram source '{}' was found, but it has a different Telegram source kind than the requested source kind",
            source_ref
        ));
    }

    Err(dialog_lookup_not_found_message(source_ref, expected_kind))
}

async fn resolve_telegram_source(
    client: &grammers_client::Client,
    source_ref: &str,
    expected_kind: Option<&str>,
) -> Result<ResolvedTelegramSource, String> {
    let trimmed = source_ref.trim();
    if expected_kind.is_none() {
        match parse_supported_manual_telegram_source_ref(trimmed)? {
            ManualTelegramSourceRef::Username(username) => {
                return resolve_telegram_source_by_username(
                    client,
                    &username,
                    source_ref,
                    expected_kind,
                )
                .await
            }
            ManualTelegramSourceRef::NumericId(source_id) => {
                return resolve_telegram_source_from_dialogs(
                    client,
                    source_id,
                    source_ref,
                    expected_kind,
                )
                .await
            }
        }
    }

    let username = parse_username(trimmed);
    if !username.is_empty() && !username.chars().all(|char| char.is_ascii_digit()) {
        return resolve_telegram_source_by_username(client, &username, source_ref, expected_kind)
            .await;
    }

    let Ok(source_id) = trimmed.parse::<i64>() else {
        return Err(format!("Telegram source '{}' not found", source_ref));
    };

    resolve_telegram_source_from_dialogs(client, source_id, source_ref, expected_kind).await
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
        Err(format!(
            "Resolved Telegram source has a different Telegram source kind than the requested source kind: expected '{}', got '{}'",
            expected_kind.unwrap_or("unknown"),
            source.telegram_source_kind
        ))
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

fn encode_source_metadata(metadata: &SourceMetadata) -> Result<Vec<u8>, String> {
    let json = serde_json::to_vec(&metadata.normalized()).map_err(|e| e.to_string())?;
    compress_json_bytes(&json)
}

fn decode_source_metadata(bytes: Option<&[u8]>) -> Result<SourceMetadata, String> {
    let Some(bytes) = bytes else {
        return Ok(SourceMetadata::default());
    };
    let decoded = decompress_bytes(bytes)?;
    serde_json::from_slice::<SourceMetadata>(&decoded)
        .map(|metadata| metadata.normalized())
        .map_err(|e| e.to_string())
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
    for step in source_peer_resolution_plan(&metadata) {
        match step {
            SourcePeerResolutionStep::Username => {
                let Some(username) = metadata
                    .peer_identity
                    .as_ref()
                    .and_then(|identity| identity.username.as_deref())
                else {
                    continue;
                };

                if let Some(peer) = client
                    .resolve_username(username)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    return peer_ref_for_source_kind(
                        &peer,
                        &source.telegram_source_kind,
                        source.id,
                    );
                }
            }
            SourcePeerResolutionStep::StoredPeerIdentity => {
                if let Some(peer_ref) =
                    source_peer_ref_from_identity(source, telegram_source_id, &metadata)?
                {
                    return Ok(peer_ref);
                }
            }
            SourcePeerResolutionStep::DialogScan => {
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
            }
        }
    }

    Err(source_peer_resolution_failure(source, &metadata))
}

fn source_peer_ref_from_identity(
    source: &SourceSyncTarget,
    telegram_source_id: i64,
    metadata: &SourceMetadata,
) -> Result<Option<PeerRef>, String> {
    let Some(access_hash) = metadata
        .peer_identity
        .as_ref()
        .and_then(|identity| identity.access_hash)
    else {
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
        (TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP | TELEGRAM_KIND_GROUP, _) => Err(
            format!(
                "Source {} resolved to a different Telegram source kind than the requested source kind",
                source_id
            ),
        ),
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

fn extract_telegram_context(message: &grammers_client::message::Message) -> TelegramItemContext {
    let mut context = TelegramItemContext {
        reply_to_msg_id: message.reply_to_message_id().map(i64::from),
        reaction_count: message.reaction_count().map(i64::from),
        ..TelegramItemContext::default()
    };

    if let Some(tl::enums::MessageReplyHeader::Header(header)) = message.reply_header() {
        context.reply_to_msg_id = header.reply_to_msg_id.map(i64::from);
        context.reply_to_top_id = header.reply_to_top_id.map(i64::from);
        if let Some((kind, id)) = reply_peer_context(header.reply_to_peer_id.as_ref()) {
            context.reply_to_peer_kind = Some(kind.to_string());
            context.reply_to_peer_id = Some(id);
        }
    }

    context
}

fn reply_peer_context(peer: Option<&tl::enums::Peer>) -> Option<(&'static str, String)> {
    match peer? {
        tl::enums::Peer::User(peer) => Some(("user", peer.user_id.to_string())),
        tl::enums::Peer::Chat(peer) => Some(("chat", peer.chat_id.to_string())),
        tl::enums::Peer::Channel(peer) => Some(("channel", peer.channel_id.to_string())),
    }
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
        add_source_resolution_strategy, decode_media_metadata, decode_source_metadata,
        default_sync_settings, determine_sync_policy, dialog_lookup_not_found_message,
        encode_media_metadata, encode_source_metadata, finalize_sync, initial_sync_policy_label,
        is_non_forum_topic_refresh_error, list_source_forum_topics_from_pool,
        load_item_rows_from_pool, load_source, load_sync_settings_from_pool,
        parse_supported_manual_telegram_source_ref, parse_username, reply_peer_context,
        save_sync_settings_to_pool, source_peer_ref_from_identity, source_peer_resolution_failure,
        source_peer_resolution_plan, upsert_forum_topics_from_refresh,
        validate_expected_telegram_source_kind, validate_sync_settings, ForumTopicFilter,
        ForumTopicSnapshot, InitialSyncMode, ManualTelegramSourceRef, ResolvedTelegramSource,
        SourceMetadata, SourcePeerIdentity, SourcePeerResolutionStep, SourcePeerResolutionStrategy,
        SourceRecordRow, SourceSyncTarget, SyncSettingsRecord, TELEGRAM_KIND_CHANNEL,
        TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP, TELEGRAM_SOURCE_TYPE,
    };
    use crate::compression::{compress_json_bytes, compress_text, decompress_text};
    use crate::error::AppErrorKind;
    use crate::media::ItemMediaMetadata;

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

    async fn memory_pool_with_sources() -> sqlx::SqlitePool {
        let pool = memory_pool().await;
        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY,
                source_type TEXT NOT NULL,
                telegram_source_kind TEXT NOT NULL,
                account_id INTEGER,
                external_id TEXT NOT NULL,
                title TEXT,
                metadata_zstd BLOB,
                last_sync_state INTEGER,
                last_synced_at INTEGER,
                is_active INTEGER NOT NULL DEFAULT 1,
                is_member INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create sources");
        pool
    }

    async fn memory_pool_with_source_items_and_topics() -> sqlx::SqlitePool {
        let pool = memory_pool_with_sources().await;
        sqlx::query(
            r#"
            CREATE TABLE items (
                id INTEGER PRIMARY KEY,
                source_id INTEGER NOT NULL,
                external_id TEXT NOT NULL,
                author TEXT,
                published_at INTEGER NOT NULL,
                ingested_at INTEGER NOT NULL,
                content_zstd BLOB,
                raw_data_zstd BLOB,
                content_kind TEXT NOT NULL,
                has_media INTEGER NOT NULL DEFAULT 0,
                media_kind TEXT,
                media_metadata_zstd BLOB,
                reply_to_msg_id INTEGER,
                reply_to_peer_kind TEXT,
                reply_to_peer_id TEXT,
                reply_to_top_id INTEGER,
                reaction_count INTEGER
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create items");
        sqlx::query(
            r#"
            CREATE TABLE telegram_forum_topics (
                id INTEGER PRIMARY KEY,
                source_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                top_message_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                icon_color INTEGER,
                icon_emoji_id INTEGER,
                is_closed INTEGER NOT NULL DEFAULT 0,
                is_pinned INTEGER NOT NULL DEFAULT 0,
                is_hidden INTEGER NOT NULL DEFAULT 0,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER,
                last_seen_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create telegram_forum_topics");
        sqlx::query(
            r#"
            CREATE UNIQUE INDEX idx_telegram_forum_topics_source_topic
            ON telegram_forum_topics(source_id, topic_id)
            "#,
        )
        .execute(&pool)
        .await
        .expect("create telegram_forum_topics unique index");
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
        let encoded = compress_json_bytes(br#"{"username":"example"}"#).expect("encode");
        let decoded = decode_source_metadata(Some(&encoded)).expect("decode");

        assert_eq!(
            decoded.peer_identity,
            Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Username,
                username: Some("example".to_string()),
                access_hash: None,
            })
        );
        assert_eq!(decoded.username, None);
        assert_eq!(decoded.added_from, None);
        assert_eq!(decoded.access_hash, None);
        assert_eq!(decoded.avatar_cache_key, None);
    }

    #[test]
    fn source_metadata_decodes_old_dialog_payloads_into_peer_identity() {
        let encoded = compress_json_bytes(
            br#"{"username":"example","added_from":"dialog","access_hash":42,"avatar_cache_key":"1_channel_42.jpg"}"#,
        )
        .expect("encode");
        let decoded = decode_source_metadata(Some(&encoded)).expect("decode");

        assert_eq!(
            decoded.peer_identity,
            Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: Some("example".to_string()),
                access_hash: Some(42),
            })
        );
        assert_eq!(decoded.username, None);
        assert_eq!(decoded.added_from, None);
        assert_eq!(decoded.access_hash, None);
        assert_eq!(
            decoded.avatar_cache_key.as_deref(),
            Some("1_channel_42.jpg")
        );
    }

    #[test]
    fn source_metadata_roundtrip_preserves_peer_identity() {
        let original = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: Some("example".to_string()),
                access_hash: Some(42),
            }),
            avatar_cache_key: Some("1_channel_42.jpg".to_string()),
            ..SourceMetadata::default()
        };

        let encoded = encode_source_metadata(&original).expect("encode");
        let decoded = decode_source_metadata(Some(&encoded)).expect("decode");

        assert_eq!(decoded, original);
    }

    #[test]
    fn parse_username_accepts_username_and_t_me_links() {
        assert_eq!(parse_username("@example"), "example");
        assert_eq!(parse_username("t.me/example"), "example");
        assert_eq!(parse_username("https://t.me/example/42"), "example");
    }

    #[test]
    fn parse_supported_manual_telegram_source_ref_accepts_public_refs_and_numeric_ids() {
        assert_eq!(
            parse_supported_manual_telegram_source_ref("@example"),
            Ok(ManualTelegramSourceRef::Username("example".to_string()))
        );
        assert_eq!(
            parse_supported_manual_telegram_source_ref("t.me/example"),
            Ok(ManualTelegramSourceRef::Username("example".to_string()))
        );
        assert_eq!(
            parse_supported_manual_telegram_source_ref("https://t.me/example/42"),
            Ok(ManualTelegramSourceRef::Username("example".to_string()))
        );
        assert_eq!(
            parse_supported_manual_telegram_source_ref("12345"),
            Ok(ManualTelegramSourceRef::NumericId(12345))
        );
    }

    #[test]
    fn parse_supported_manual_telegram_source_ref_rejects_private_links() {
        for source_ref in [
            "https://t.me/+AAAAAE-example",
            "t.me/joinchat/AAAAAE-example",
            "https://t.me/c/12345/67",
        ] {
            let error = parse_supported_manual_telegram_source_ref(source_ref)
                .expect_err("private/manual ref should be rejected");
            assert!(error.contains("not supported for manual add"));
            assert!(error.contains("dialogs"));
        }
    }

    #[test]
    fn dialog_lookup_not_found_message_explains_numeric_manual_limit() {
        let message = dialog_lookup_not_found_message("12345", None);
        assert!(message.contains("not found in this account's dialogs"));
        assert!(message.contains("Numeric manual adds only work"));
        assert!(message.contains("private Telegram sources"));
    }

    #[test]
    fn add_source_resolution_strategy_distinguishes_username_and_dialog_flows() {
        assert_eq!(
            add_source_resolution_strategy("@example", None),
            SourcePeerResolutionStrategy::Username
        );
        assert_eq!(
            add_source_resolution_strategy("t.me/example", None),
            SourcePeerResolutionStrategy::Username
        );
        assert_eq!(
            add_source_resolution_strategy("12345", None),
            SourcePeerResolutionStrategy::Dialog
        );
        assert_eq!(
            add_source_resolution_strategy("@example", Some(TELEGRAM_KIND_CHANNEL)),
            SourcePeerResolutionStrategy::Dialog
        );
    }

    #[test]
    fn source_peer_resolution_plan_prefers_explicit_strategy_order() {
        let dialog_metadata = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: Some("example".to_string()),
                access_hash: Some(42),
            }),
            ..SourceMetadata::default()
        };
        assert_eq!(
            source_peer_resolution_plan(&dialog_metadata),
            vec![
                SourcePeerResolutionStep::StoredPeerIdentity,
                SourcePeerResolutionStep::Username,
                SourcePeerResolutionStep::DialogScan,
            ]
        );

        let username_metadata = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Username,
                username: Some("example".to_string()),
                access_hash: Some(42),
            }),
            ..SourceMetadata::default()
        };
        assert_eq!(
            source_peer_resolution_plan(&username_metadata),
            vec![
                SourcePeerResolutionStep::Username,
                SourcePeerResolutionStep::DialogScan,
            ]
        );
    }

    #[test]
    fn validate_expected_telegram_source_kind_reports_requested_and_actual_kind() {
        let source = ResolvedTelegramSource {
            external_id: "123".to_string(),
            title: "Example".to_string(),
            telegram_source_kind: TELEGRAM_KIND_SUPERGROUP.to_string(),
            is_member: true,
            username: Some("example".to_string()),
            access_hash: Some(42),
            avatar_bytes: None,
        };

        let error = validate_expected_telegram_source_kind(&source, Some(TELEGRAM_KIND_CHANNEL))
            .expect_err("expected kind mismatch");

        assert!(error.contains("requested source kind"));
        assert!(error.contains(TELEGRAM_KIND_CHANNEL));
        assert!(error.contains(TELEGRAM_KIND_SUPERGROUP));
    }

    #[test]
    fn reply_peer_context_uses_telegram_peer_kinds() {
        assert_eq!(
            reply_peer_context(Some(&super::tl::enums::Peer::User(
                super::tl::types::PeerUser { user_id: 11 }
            ))),
            Some(("user", "11".to_string()))
        );
        assert_eq!(
            reply_peer_context(Some(&super::tl::enums::Peer::Chat(
                super::tl::types::PeerChat { chat_id: 22 }
            ))),
            Some(("chat", "22".to_string()))
        );
        assert_eq!(
            reply_peer_context(Some(&super::tl::enums::Peer::Channel(
                super::tl::types::PeerChannel { channel_id: 33 }
            ))),
            Some(("channel", "33".to_string()))
        );
        assert_eq!(reply_peer_context(None), None);
    }

    #[test]
    fn peer_ref_from_identity_uses_channel_access_hash() {
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
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: None,
                access_hash: Some(67890),
            }),
            ..SourceMetadata::default()
        };

        let peer_ref = source_peer_ref_from_identity(&source, 12345, &metadata)
            .expect("metadata peer ref")
            .expect("peer ref");

        assert_eq!(peer_ref.id.bare_id(), 12345);
        assert_eq!(peer_ref.auth.hash(), 67890);
    }

    #[test]
    fn peer_ref_from_identity_uses_supergroup_access_hash() {
        let source = SourceSyncTarget {
            id: 7,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            telegram_source_kind: TELEGRAM_KIND_SUPERGROUP.to_string(),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            metadata_zstd: None,
            last_sync_state: None,
        };
        let metadata = SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: None,
                access_hash: Some(67890),
            }),
            ..SourceMetadata::default()
        };

        let peer_ref = source_peer_ref_from_identity(&source, 12345, &metadata)
            .expect("metadata peer ref")
            .expect("peer ref");

        assert_eq!(peer_ref.id.bare_id(), 12345);
        assert_eq!(peer_ref.auth.hash(), 67890);
    }

    #[test]
    fn peer_ref_from_identity_ignores_small_groups_without_supported_identity() {
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
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: None,
                access_hash: Some(67890),
            }),
            ..SourceMetadata::default()
        };

        let peer_ref =
            source_peer_ref_from_identity(&source, 12345, &metadata).expect("metadata peer ref");

        assert!(peer_ref.is_none());
    }

    #[test]
    fn source_peer_resolution_failure_explains_small_group_dialog_dependency() {
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
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Dialog,
                username: None,
                access_hash: None,
            }),
            ..SourceMetadata::default()
        };

        let message = source_peer_resolution_failure(&source, &metadata);
        assert!(message.contains("Small Telegram groups"));
        assert!(message.contains("dialogs"));
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

    #[tokio::test]
    async fn load_source_returns_not_found_for_missing_source() {
        let pool = memory_pool_with_sources().await;
        let error = match load_source(&pool, 999).await {
            Ok(_) => panic!("expected missing source error"),
            Err(error) => error,
        };

        assert_eq!(error.kind, AppErrorKind::NotFound);
    }

    #[tokio::test]
    async fn determine_sync_policy_only_applies_initial_settings_on_first_sync() {
        let pool = memory_pool_with_sources().await;
        let source = SourceSyncTarget {
            id: 1,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            telegram_source_kind: TELEGRAM_KIND_CHANNEL.to_string(),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("Example".to_string()),
            metadata_zstd: None,
            last_sync_state: None,
        };

        let initial = determine_sync_policy(&pool, &source)
            .await
            .expect("determine initial policy");
        assert_eq!(initial.previous_last_sync, 0);
        assert_eq!(
            initial.initial_sync_policy_applied.as_deref(),
            Some("last 500 messages")
        );
        assert!(initial.initial_sync_settings.is_some());
        assert_eq!(initial.initial_sync_cutoff, None);

        let incremental = determine_sync_policy(
            &pool,
            &SourceSyncTarget {
                last_sync_state: Some(77),
                ..source
            },
        )
        .await
        .expect("determine incremental policy");
        assert_eq!(incremental.previous_last_sync, 77);
        assert!(incremental.initial_sync_settings.is_none());
        assert!(incremental.initial_sync_policy_applied.is_none());
        assert_eq!(incremental.initial_sync_cutoff, None);
    }

    #[tokio::test]
    async fn finalize_sync_updates_source_state_and_metadata() {
        let pool = memory_pool_with_sources().await;
        let metadata_zstd = encode_source_metadata(&SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Username,
                username: Some("before".to_string()),
                access_hash: None,
            }),
            ..SourceMetadata::default()
        })
        .expect("encode initial metadata");
        sqlx::query(
            r#"
            INSERT INTO sources (
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
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind(TELEGRAM_SOURCE_TYPE)
        .bind(TELEGRAM_KIND_CHANNEL)
        .bind(1_i64)
        .bind("12345")
        .bind("Example")
        .bind(metadata_zstd)
        .bind(5_i64)
        .bind(10_i64)
        .bind(1_i64)
        .bind(1_i64)
        .bind(20_i64)
        .execute(&pool)
        .await
        .expect("insert source");

        let source = load_source(&pool, 1).await.expect("load source");
        let updated_metadata_zstd = encode_source_metadata(&SourceMetadata {
            peer_identity: Some(SourcePeerIdentity {
                strategy: SourcePeerResolutionStrategy::Username,
                username: Some("after".to_string()),
                access_hash: None,
            }),
            avatar_cache_key: Some("1_channel_12345.jpg".to_string()),
            ..SourceMetadata::default()
        })
        .expect("encode updated metadata");

        let last_sync_state = finalize_sync(&pool, &source, 5, 9, Some(updated_metadata_zstd))
            .await
            .expect("finalize sync");
        assert_eq!(last_sync_state, Some(9));

        let row: SourceRecordRow = sqlx::query_as(
            "SELECT id, source_type, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at FROM sources WHERE id = ?",
        )
        .bind(1_i64)
        .fetch_one(&pool)
        .await
        .expect("reload updated source");

        assert_eq!(row.last_sync_state, Some(9));
        assert!(row.last_synced_at.is_some());
        let decoded_metadata =
            decode_source_metadata(row.metadata_zstd.as_deref()).expect("decode metadata");
        assert_eq!(
            decoded_metadata
                .peer_identity
                .as_ref()
                .and_then(|identity| identity.username.as_deref()),
            Some("after")
        );
        assert_eq!(
            decoded_metadata.avatar_cache_key.as_deref(),
            Some("1_channel_12345.jpg")
        );
    }

    #[tokio::test]
    async fn load_item_rows_attaches_topic_metadata_and_root_matches() {
        let pool = memory_pool_with_source_items_and_topics().await;
        for (id, topic_id, top_message_id, title, sort_order) in [
            (1_i64, 200_i64, 700_i64, "Announcements", 1_i64),
            (2_i64, 1_i64, 1_i64, "General", 2_i64),
        ] {
            sqlx::query(
                r#"
                INSERT INTO telegram_forum_topics (
                    id, source_id, topic_id, top_message_id, title, is_closed, is_pinned, is_hidden,
                    is_deleted, sort_order, last_seen_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(id)
            .bind(1_i64)
            .bind(topic_id)
            .bind(top_message_id)
            .bind(title)
            .bind(0_i64)
            .bind(1_i64)
            .bind(0_i64)
            .bind(0_i64)
            .bind(sort_order)
            .bind(100_i64)
            .bind(100_i64)
            .execute(&pool)
            .await
            .expect("insert forum topic");
        }

        for (id, external_id, published_at, reply_to_msg_id, reply_to_top_id) in [
            (1_i64, "700", 500_i64, None, None),
            (2_i64, "701", 400_i64, None, Some(200_i64)),
            (3_i64, "702", 300_i64, Some(200_i64), None),
            (4_i64, "999", 200_i64, None, None),
            (5_i64, "1000", 100_i64, Some(123_i64), Some(404_i64)),
        ] {
            sqlx::query(
                r#"
                INSERT INTO items (
                    id, source_id, external_id, author, published_at, ingested_at, content_zstd,
                    raw_data_zstd, content_kind, has_media, media_kind, media_metadata_zstd,
                    reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id,
                    reaction_count
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(id)
            .bind(1_i64)
            .bind(external_id)
            .bind("alice")
            .bind(published_at)
            .bind(published_at)
            .bind(None::<Vec<u8>>)
            .bind(None::<Vec<u8>>)
            .bind("text_only")
            .bind(0_i64)
            .bind(None::<String>)
            .bind(None::<Vec<u8>>)
            .bind(reply_to_msg_id)
            .bind(None::<String>)
            .bind(None::<String>)
            .bind(reply_to_top_id)
            .bind(None::<i64>)
            .execute(&pool)
            .await
            .expect("insert item");
        }

        let rows = load_item_rows_from_pool(&pool, 1, 20, None, None)
            .await
            .expect("load all rows");
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].forum_topic_id, Some(200));
        assert_eq!(rows[0].forum_topic_top_message_id, Some(700));
        assert_eq!(rows[1].forum_topic_id, Some(200));
        assert_eq!(rows[2].forum_topic_id, Some(200));
        assert_eq!(rows[3].forum_topic_id, Some(1));
        assert_eq!(rows[4].forum_topic_id, None);

        let topic_rows = load_item_rows_from_pool(
            &pool,
            1,
            20,
            None,
            Some(ForumTopicFilter::Topic { topic_id: 200 }),
        )
        .await
        .expect("load topic rows");
        assert_eq!(topic_rows.len(), 3);
        assert!(topic_rows.iter().all(|row| row.forum_topic_id == Some(200)));

        let general_rows = load_item_rows_from_pool(
            &pool,
            1,
            20,
            None,
            Some(ForumTopicFilter::Topic { topic_id: 1 }),
        )
        .await
        .expect("load general rows");
        assert_eq!(general_rows.len(), 1);
        assert_eq!(general_rows[0].external_id, "999");

        let uncategorized_rows =
            load_item_rows_from_pool(&pool, 1, 20, None, Some(ForumTopicFilter::Uncategorized))
                .await
                .expect("load uncategorized rows");
        assert_eq!(uncategorized_rows.len(), 1);
        assert_eq!(uncategorized_rows[0].external_id, "1000");
    }

    #[tokio::test]
    async fn list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket() {
        let pool = memory_pool_with_source_items_and_topics().await;

        for (id, topic_id, top_message_id, title, is_pinned, sort_order) in [
            (1_i64, 22_i64, 900_i64, "beta", 0_i64, 2_i64),
            (2_i64, 11_i64, 800_i64, "Alpha", 1_i64, 5_i64),
            (3_i64, 1_i64, 1_i64, "General", 0_i64, 3_i64),
        ] {
            sqlx::query(
                r#"
                INSERT INTO telegram_forum_topics (
                    id, source_id, topic_id, top_message_id, title, is_closed, is_pinned, is_hidden,
                    is_deleted, sort_order, last_seen_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(id)
            .bind(1_i64)
            .bind(topic_id)
            .bind(top_message_id)
            .bind(title)
            .bind(0_i64)
            .bind(is_pinned)
            .bind(0_i64)
            .bind(0_i64)
            .bind(sort_order)
            .bind(100_i64)
            .bind(100_i64)
            .execute(&pool)
            .await
            .expect("insert topic");
        }

        for (id, external_id, published_at, reply_to_msg_id, reply_to_top_id) in [
            (1_i64, "800", 400_i64, None, None),
            (2_i64, "801", 300_i64, None, Some(11_i64)),
            (3_i64, "950", 200_i64, None, None),
            (4_i64, "901", 100_i64, None, Some(22_i64)),
            (5_i64, "902", 50_i64, Some(22_i64), None),
            (6_i64, "951", 25_i64, None, Some(404_i64)),
        ] {
            sqlx::query(
                r#"
                INSERT INTO items (
                    id, source_id, external_id, author, published_at, ingested_at, content_zstd,
                    raw_data_zstd, content_kind, has_media, media_kind, media_metadata_zstd,
                    reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id,
                    reaction_count
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(id)
            .bind(1_i64)
            .bind(external_id)
            .bind("bob")
            .bind(published_at)
            .bind(published_at)
            .bind(None::<Vec<u8>>)
            .bind(None::<Vec<u8>>)
            .bind("text_only")
            .bind(0_i64)
            .bind(None::<String>)
            .bind(None::<Vec<u8>>)
            .bind(reply_to_msg_id)
            .bind(None::<String>)
            .bind(None::<String>)
            .bind(reply_to_top_id)
            .bind(None::<i64>)
            .execute(&pool)
            .await
            .expect("insert item");
        }

        let records = list_source_forum_topics_from_pool(&pool, 1)
            .await
            .expect("list source forum topics");

        assert_eq!(records.len(), 4);
        assert_eq!(records[0].kind, "topic");
        assert_eq!(records[0].topic_id, Some(11));
        assert_eq!(records[0].message_count, 2);
        assert_eq!(records[1].topic_id, Some(22));
        assert_eq!(records[1].message_count, 2);
        assert_eq!(records[2].topic_id, Some(1));
        assert_eq!(records[2].message_count, 1);
        assert_eq!(records[3].kind, "uncategorized");
        assert_eq!(records[3].key, "unrecognized_topic");
        assert_eq!(records[3].message_count, 1);
    }

    #[tokio::test]
    async fn upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted() {
        let pool = memory_pool_with_source_items_and_topics().await;

        sqlx::query(
            r#"
            INSERT INTO telegram_forum_topics (
                id, source_id, topic_id, top_message_id, title, icon_color, icon_emoji_id,
                is_closed, is_pinned, is_hidden, is_deleted, sort_order, last_seen_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind(1_i64)
        .bind(10_i64)
        .bind(500_i64)
        .bind("Keep me")
        .bind(1_i64)
        .bind(None::<i64>)
        .bind(0_i64)
        .bind(0_i64)
        .bind(0_i64)
        .bind(0_i64)
        .bind(0_i64)
        .bind(10_i64)
        .bind(10_i64)
        .execute(&pool)
        .await
        .expect("insert preserved topic");

        sqlx::query(
            r#"
            INSERT INTO telegram_forum_topics (
                id, source_id, topic_id, top_message_id, title, icon_color, icon_emoji_id,
                is_closed, is_pinned, is_hidden, is_deleted, sort_order, last_seen_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(2_i64)
        .bind(1_i64)
        .bind(20_i64)
        .bind(600_i64)
        .bind("Delete me")
        .bind(1_i64)
        .bind(None::<i64>)
        .bind(0_i64)
        .bind(0_i64)
        .bind(0_i64)
        .bind(0_i64)
        .bind(1_i64)
        .bind(10_i64)
        .bind(10_i64)
        .execute(&pool)
        .await
        .expect("insert deleted topic");

        upsert_forum_topics_from_refresh(
            &pool,
            1,
            &[ForumTopicSnapshot {
                topic_id: 30,
                top_message_id: 700,
                title: "Fresh".to_string(),
                icon_color: 7,
                icon_emoji_id: Some(999),
                is_closed: true,
                is_pinned: true,
                is_hidden: false,
                sort_order: 2,
            }],
            &[20],
            1234,
        )
        .await
        .expect("upsert forum topics");

        let rows: Vec<(i64, String, i64, i64)> = sqlx::query_as(
            r#"
            SELECT topic_id, title, is_deleted, last_seen_at
            FROM telegram_forum_topics
            WHERE source_id = ?
            ORDER BY topic_id ASC
            "#,
        )
        .bind(1_i64)
        .fetch_all(&pool)
        .await
        .expect("reload topics");

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], (10, "Keep me".to_string(), 0, 10));
        assert_eq!(rows[1], (20, "Delete me".to_string(), 1, 1234));
        assert_eq!(rows[2], (30, "Fresh".to_string(), 0, 1234));
    }

    #[test]
    fn non_forum_topic_refresh_errors_are_detected() {
        assert!(is_non_forum_topic_refresh_error(
            "Rpc error 400: CHANNEL_FORUM_MISSING"
        ));
        assert!(is_non_forum_topic_refresh_error(
            "Rpc error 400: CHANNEL_MONOFORUM_UNSUPPORTED"
        ));
        assert!(!is_non_forum_topic_refresh_error(
            "Rpc error 400: CHANNEL_PRIVATE"
        ));
    }
}
