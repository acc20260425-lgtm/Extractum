use grammers_client::tl;
use grammers_session::types::PeerRef;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;
use tokio::time::{Duration, Instant};

use crate::compression::{compress_json_bytes, compress_text, decompress_bytes, decompress_text};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::forum_topics::{
    resolved_topic_join, resolved_topic_predicate, ResolvedTopicAliases,
    FORUM_TOPIC_UNCATEGORIZED_KEY, FORUM_TOPIC_UNCATEGORIZED_TITLE,
};
use crate::media::{
    decode_media_metadata, encode_media_metadata, extract_item_payload, ExtractedItemPayload,
};
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::telegram::TelegramState;

mod avatar;
mod peer_resolution;
mod settings;
mod types;

pub use self::settings::{
    get_sync_settings, save_sync_settings, InitialSyncMode, SyncSettingsRecord,
};
pub use self::types::{SourceRecord, TelegramSourceInfo};

pub(crate) use self::peer_resolution::{resolve_and_refresh_peer, ResolvedSyncPeer};
pub(crate) use self::types::SourceSyncTarget;

use self::avatar::{
    cache_source_avatar, peer_photo_data_url_with_timeout, read_source_avatar_data_url,
    TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS,
};
use self::peer_resolution::{
    decode_source_metadata, encode_source_metadata, resolve_telegram_source,
    source_metadata_for_added_source, telegram_source_info_from_peer,
};
use self::settings::{initial_sync_policy_label, load_sync_settings_from_pool, SECONDS_PER_DAY};
use self::types::{
    SourceForumTopicRow, SourceRecordRow, StoredItemRow, TELEGRAM_KIND_CHANNEL,
    TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP, TELEGRAM_SOURCE_TYPE,
};

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

pub(crate) struct SourceItemInsert {
    pub(crate) external_id: String,
    pub(crate) author: Option<String>,
    pub(crate) published_at: i64,
    pub(crate) payload: ExtractedItemPayload,
    pub(crate) raw_data: Vec<u8>,
    pub(crate) telegram_context: TelegramItemContext,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelegramItemContext {
    pub(crate) reply_to_msg_id: Option<i64>,
    pub(crate) reply_to_peer_kind: Option<String>,
    pub(crate) reply_to_peer_id: Option<String>,
    pub(crate) reply_to_top_id: Option<i64>,
    pub(crate) reaction_count: Option<i64>,
}

#[tauri::command]
pub async fn delete_source(
    handle: AppHandle,
    ingest_locks: tauri::State<'_, SourceIngestLocks>,
    source_id: i64,
) -> AppResult<()> {
    let _ingest_guard = ingest_locks
        .try_acquire(source_id, SourceIngestKind::Delete)
        .await?;
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

pub(crate) async fn load_source(
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

        let author = message_author(&message);
        let telegram_context = extract_telegram_context(&message);
        let raw_data = build_raw_payload(&message, &source.title, &author, &item_payload)?;

        let inserted_item = insert_source_item(
            pool,
            source.id,
            SourceItemInsert {
                external_id: message_id.to_string(),
                author,
                published_at,
                payload: item_payload,
                raw_data,
                telegram_context,
            },
        )
        .await?;

        if inserted_item {
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

pub(crate) async fn finalize_sync(
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

pub(crate) async fn insert_source_item(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    item: SourceItemInsert,
) -> Result<bool, String> {
    let content_zstd = item
        .payload
        .content
        .as_deref()
        .map(compress_text)
        .transpose()?;
    let media_kind = item.payload.media.as_ref().map(|media| media.kind.clone());
    let media_metadata_zstd = item
        .payload
        .media
        .as_ref()
        .map(|media| encode_media_metadata(&media.metadata))
        .transpose()?;

    if content_zstd.is_none() && media_metadata_zstd.is_none() {
        return Ok(false);
    }

    let raw_data_zstd = compress_json_bytes(&item.raw_data)?;
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
    .bind(source_id)
    .bind(&item.external_id)
    .bind(&item.author)
    .bind(item.published_at)
    .bind(now_secs())
    .bind(content_zstd)
    .bind(raw_data_zstd)
    .bind(item.payload.content_kind)
    .bind(item.payload.media.is_some())
    .bind(&media_kind)
    .bind(media_metadata_zstd)
    .bind(item.telegram_context.reply_to_msg_id)
    .bind(&item.telegram_context.reply_to_peer_kind)
    .bind(&item.telegram_context.reply_to_peer_id)
    .bind(item.telegram_context.reply_to_top_id)
    .bind(item.telegram_context.reaction_count)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() == 1)
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
    ingest_locks: tauri::State<'_, SourceIngestLocks>,
    source_id: i64,
) -> AppResult<SyncResult> {
    let _ingest_guard = ingest_locks
        .try_acquire(source_id, SourceIngestKind::Sync)
        .await?;
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;

    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;

    let runtime = crate::telegram::get_authorized_runtime(&state, account_id).await?;
    let client = runtime.client;
    let resolved_peer = resolve_and_refresh_peer(&handle, &client, &source, account_id).await?;
    let forum_topic_warnings =
        refresh_forum_topics(&pool, &client, resolved_peer.peer, &source).await;
    let sync_policy = determine_sync_policy(&pool, &source).await?;
    let ingest = persist_items(&pool, &client, resolved_peer.peer, &source, &sync_policy).await?;
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

        let tl::enums::messages::ForumTopics::Topics(forum_topics) = response;

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
    let topic_join = resolved_topic_join(&ResolvedTopicAliases {
        item: "items",
        topic: "forum_topics",
        matched_topic: "matched_topics",
    });
    let mut sql = format!(
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
        {topic_join}
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
    let topic_match = resolved_topic_predicate(&ResolvedTopicAliases {
        item: "items",
        topic: "topics",
        matched_topic: "matched_topics",
    });
    let rows_sql = format!(
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
          ON {topic_match}
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
    );
    let rows: Vec<SourceForumTopicRow> = sqlx::query_as(&rows_sql)
        .bind(source_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::from(e.to_string()))?;

    let topic_join = resolved_topic_join(&ResolvedTopicAliases {
        item: "items",
        topic: "forum_topics",
        matched_topic: "matched_topics",
    });
    let uncategorized_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM items
        {topic_join}
        WHERE items.source_id = ?
          AND forum_topics.topic_id IS NULL
        "#,
    );
    let uncategorized_count: i64 = sqlx::query_scalar(&uncategorized_sql)
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

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
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
        context.reply_to_msg_id = header
            .reply_to_msg_id
            .map(i64::from)
            .or(context.reply_to_msg_id);
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
        decode_media_metadata, decode_source_metadata, determine_sync_policy,
        encode_media_metadata, finalize_sync, insert_source_item,
        is_non_forum_topic_refresh_error, list_source_forum_topics_from_pool,
        load_item_rows_from_pool, load_source, reply_peer_context,
        upsert_forum_topics_from_refresh, ForumTopicFilter, ForumTopicSnapshot, SourceItemInsert,
        SourceRecordRow, SourceSyncTarget, StoredItemRow, TelegramItemContext,
        TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP,
        TELEGRAM_SOURCE_TYPE,
    };
    use crate::compression::{
        compress_json_bytes, compress_text, decompress_bytes, decompress_text,
    };
    use crate::error::AppErrorKind;
    use crate::media::{
        ExtractedItemPayload, ExtractedMediaPayload, ItemMediaMetadata, CONTENT_KIND_TEXT_ONLY,
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
            CREATE UNIQUE INDEX idx_items_source_external
            ON items(source_id, external_id)
            "#,
        )
        .execute(&pool)
        .await
        .expect("create items unique index");
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

    #[tokio::test]
    async fn insert_source_item_writes_payload_and_skips_duplicates() {
        let pool = memory_pool_with_source_items_and_topics().await;
        let media_metadata = ItemMediaMetadata {
            summary: Some("Photo".to_string()),
            file_name: Some("photo.jpg".to_string()),
            mime_type: Some("image/jpeg".to_string()),
            width: Some(640),
            height: Some(480),
            ..ItemMediaMetadata::default()
        };

        let inserted = insert_source_item(
            &pool,
            1,
            SourceItemInsert {
                external_id: "42".to_string(),
                author: Some("alice".to_string()),
                published_at: 1234,
                payload: ExtractedItemPayload {
                    content: Some("hello".to_string()),
                    content_kind: CONTENT_KIND_TEXT_WITH_MEDIA,
                    media: Some(ExtractedMediaPayload {
                        kind: "photo".to_string(),
                        metadata: media_metadata.clone(),
                    }),
                },
                raw_data: br#"{"id":42}"#.to_vec(),
                telegram_context: TelegramItemContext {
                    reply_to_msg_id: Some(7),
                    reply_to_peer_kind: Some("channel".to_string()),
                    reply_to_peer_id: Some("99".to_string()),
                    reply_to_top_id: Some(5),
                    reaction_count: Some(3),
                },
            },
        )
        .await
        .expect("insert item");
        assert!(inserted);

        let duplicate = insert_source_item(
            &pool,
            1,
            SourceItemInsert {
                external_id: "42".to_string(),
                author: None,
                published_at: 9999,
                payload: ExtractedItemPayload {
                    content: Some("duplicate".to_string()),
                    content_kind: CONTENT_KIND_TEXT_ONLY,
                    media: None,
                },
                raw_data: br#"{"id":42,"duplicate":true}"#.to_vec(),
                telegram_context: TelegramItemContext::default(),
            },
        )
        .await
        .expect("skip duplicate");
        assert!(!duplicate);

        let row: StoredItemRow = sqlx::query_as(
            r#"
            SELECT
                id, source_id, external_id, author, published_at, content_kind, has_media,
                media_kind, content_zstd, media_metadata_zstd, raw_data_zstd,
                NULL AS forum_topic_id, NULL AS forum_topic_title, NULL AS forum_topic_top_message_id
            FROM items
            WHERE source_id = ? AND external_id = ?
            "#,
        )
        .bind(1_i64)
        .bind("42")
        .fetch_one(&pool)
        .await
        .expect("load inserted item");

        assert_eq!(row.source_id, 1);
        assert_eq!(row.author.as_deref(), Some("alice"));
        assert_eq!(row.published_at, 1234);
        assert_eq!(row.content_kind, CONTENT_KIND_TEXT_WITH_MEDIA);
        assert!(row.has_media);
        assert_eq!(row.media_kind.as_deref(), Some("photo"));
        assert_eq!(
            decompress_text(&row.content_zstd.expect("content")).expect("decode content"),
            "hello"
        );
        assert_eq!(
            decode_media_metadata(row.media_metadata_zstd.as_deref()).expect("decode media"),
            media_metadata
        );
        assert_eq!(
            decompress_bytes(&row.raw_data_zstd.expect("raw")).expect("decode raw"),
            br#"{"id":42}"#.to_vec()
        );
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
        let metadata_zstd = compress_json_bytes(
            br#"{"peer_identity":{"strategy":"username","username":"before"}}"#,
        )
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
        let updated_metadata_zstd = compress_json_bytes(
            br#"{"peer_identity":{"strategy":"username","username":"after"},"avatar_cache_key":"1_channel_12345.jpg"}"#,
        )
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
        let metadata_bytes = row.metadata_zstd.as_deref().expect("updated metadata");
        let decoded_metadata =
            decode_source_metadata(Some(metadata_bytes)).expect("decode metadata");
        let raw_metadata: serde_json::Value = serde_json::from_slice(
            &decompress_bytes(metadata_bytes).expect("decompress metadata"),
        )
        .expect("parse metadata");
        assert_eq!(
            raw_metadata
                .pointer("/peer_identity/username")
                .and_then(serde_json::Value::as_str),
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
