use grammers_client::tl;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;

use crate::compression::{compress_json_bytes, compress_text, decompress_text};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::media::{decode_media_metadata, encode_media_metadata, ExtractedItemPayload};
use crate::tx::{begin_immediate, finish_manual_transaction};

use super::identity_repair::{require_source_identity_ready, SourceIdentityRepairState};
use super::types::{
    now_secs, StoredItemRow, TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE,
    ITEM_KIND_YOUTUBE_COMMENT, ITEM_KIND_YOUTUBE_TRANSCRIPT,
};
use query::load_item_rows_from_pool;

mod query;

#[derive(Serialize)]
pub struct ItemRecord {
    pub id: i64,
    pub source_id: i64,
    pub external_id: String,
    pub item_kind: String,
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
    pub reply_to_msg_id: Option<i64>,
    pub reply_to_peer_kind: Option<String>,
    pub reply_to_peer_id: Option<String>,
    pub reply_to_top_id: Option<i64>,
    pub reaction_count: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ForumTopicFilter {
    Topic {
        #[serde(rename = "topicId")]
        topic_id: i64,
    },
    Uncategorized,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSourceItemsRequest {
    pub source_id: i64,
    pub limit: i64,
    pub before_published_at: Option<i64>,
    pub topic_filter: Option<ForumTopicFilter>,
    pub around_item_id: Option<i64>,
}

pub(crate) struct SourceItemInsert {
    #[allow(dead_code)]
    pub(crate) external_id: String,
    pub(crate) item_kind: String,
    pub(crate) author: Option<String>,
    pub(crate) published_at: i64,
    pub(crate) payload: ExtractedItemPayload,
    pub(crate) raw_data: Vec<u8>,
    pub(crate) telegram_context: TelegramItemContext,
    pub(crate) telegram_identity: Option<TelegramMessageIdentity>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelegramItemContext {
    pub(crate) reply_to_msg_id: Option<i64>,
    pub(crate) reply_to_peer_kind: Option<String>,
    pub(crate) reply_to_peer_id: Option<String>,
    pub(crate) reply_to_top_id: Option<i64>,
    pub(crate) reaction_count: Option<i64>,
}

struct PreparedSourceItem {
    content_zstd: Option<Vec<u8>>,
    raw_data_zstd: Vec<u8>,
    content_kind: String,
    has_media: bool,
    media_kind: Option<String>,
    media_metadata_zstd: Option<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TelegramItemInsertOutcome {
    Inserted { item_id: i64 },
    DuplicateObserved { item_id: i64 },
    Skipped { reason_code: &'static str },
}

impl TelegramItemInsertOutcome {
    pub(crate) fn is_inserted(self) -> bool {
        matches!(self, Self::Inserted { .. })
    }

    pub(crate) fn observation_parts(self) -> (&'static str, Option<i64>, Option<&'static str>) {
        match self {
            Self::Inserted { item_id } => ("inserted", Some(item_id), None),
            Self::DuplicateObserved { item_id } => ("duplicate_observed", Some(item_id), None),
            Self::Skipped { reason_code } => ("skipped", None, Some(reason_code)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArchiveReadMaintenanceMode {
    MaintainSingleWrite,
    MarkSourceStaleOnly,
}

fn prepare_source_item(item: &SourceItemInsert) -> AppResult<Option<PreparedSourceItem>> {
    let content_zstd = item
        .payload
        .content
        .as_deref()
        .map(compress_text)
        .transpose()
        .map_err(AppError::internal)?;
    let media_kind = item.payload.media.as_ref().map(|media| media.kind.clone());
    let media_metadata_zstd = item
        .payload
        .media
        .as_ref()
        .map(|media| encode_media_metadata(&media.metadata))
        .transpose()
        .map_err(AppError::internal)?;

    if content_zstd.is_none() && media_metadata_zstd.is_none() {
        return Ok(None);
    }

    let raw_data_zstd = compress_json_bytes(&item.raw_data).map_err(AppError::internal)?;
    Ok(Some(PreparedSourceItem {
        content_zstd,
        raw_data_zstd,
        content_kind: item.payload.content_kind.to_string(),
        has_media: item.payload.media.is_some(),
        media_kind,
        media_metadata_zstd,
    }))
}

#[allow(dead_code)]
pub(crate) async fn insert_source_item(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    item: SourceItemInsert,
) -> AppResult<bool> {
    let Some(prepared) = prepare_source_item(&item)? else {
        return Ok(false);
    };
    let result = sqlx::query(
        r#"
        INSERT INTO items (
            source_id,
            external_id,
            item_kind,
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
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(source_id, external_id) DO NOTHING
        "#,
    )
    .bind(source_id)
    .bind(&item.external_id)
    .bind(&item.item_kind)
    .bind(&item.author)
    .bind(item.published_at)
    .bind(now_secs())
    .bind(prepared.content_zstd)
    .bind(prepared.raw_data_zstd)
    .bind(prepared.content_kind)
    .bind(prepared.has_media)
    .bind(prepared.media_kind)
    .bind(prepared.media_metadata_zstd)
    .bind(item.telegram_context.reply_to_msg_id)
    .bind(&item.telegram_context.reply_to_peer_kind)
    .bind(&item.telegram_context.reply_to_peer_id)
    .bind(item.telegram_context.reply_to_top_id)
    .bind(item.telegram_context.reaction_count)
    .execute(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(result.rows_affected() == 1)
}

pub(crate) async fn insert_telegram_source_item(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
) -> AppResult<bool> {
    Ok(
        insert_telegram_source_item_outcome(pool, source_id, identity, item)
            .await?
            .is_inserted(),
    )
}

pub(crate) async fn insert_telegram_source_item_outcome(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
) -> AppResult<TelegramItemInsertOutcome> {
    let mut conn = begin_immediate(pool).await?;

    let result = insert_telegram_source_item_on_connection(
        &mut conn,
        source_id,
        identity,
        item,
        ArchiveReadMaintenanceMode::MaintainSingleWrite,
    )
    .await;

    match result {
        Ok(outcome) => finish_manual_transaction(&mut conn, Ok(outcome)).await,
        Err(error) => {
            let is_skippable_conflict = error.kind == crate::error::AppErrorKind::Conflict
                || error.message.contains("telegram_messages");
            let result = finish_manual_transaction(&mut conn, Err(error)).await;
            if is_skippable_conflict {
                return Ok(TelegramItemInsertOutcome::Skipped {
                    reason_code: "conflict_without_item_id",
                });
            }
            result
        }
    }
}

pub(crate) async fn insert_telegram_source_item_with_observation(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
) -> AppResult<TelegramItemInsertOutcome> {
    let provider_identity = crate::ingest_provenance::telegram_provider_identity(&identity);
    let mut conn = begin_immediate(pool).await?;

    let result: AppResult<TelegramItemInsertOutcome> = async {
        let outcome = insert_telegram_source_item_on_connection(
            &mut conn,
            source_id,
            identity,
            item,
            ArchiveReadMaintenanceMode::MarkSourceStaleOnly,
        )
        .await?;
        let (outcome_name, item_id, reason_code) = outcome.observation_parts();
        crate::ingest_provenance::record_ingest_observation_on_connection(
            &mut conn,
            crate::ingest_provenance::IngestObservation {
                batch_id,
                source_id,
                item_id,
                provider_item_kind: ITEM_KIND_TELEGRAM_MESSAGE,
                provider_identity_kind: "telegram_message",
                provider_identity,
                outcome: outcome_name,
                reason_code,
            },
        )
        .await?;
        Ok(outcome)
    }
    .await;

    finish_manual_transaction(&mut conn, result).await
}

async fn insert_telegram_source_item_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
    archive_maintenance: ArchiveReadMaintenanceMode,
) -> AppResult<TelegramItemInsertOutcome> {
    identity.validate()?;
    if item.item_kind != ITEM_KIND_TELEGRAM_MESSAGE {
        return Err(AppError::validation(format!(
            "insert_telegram_source_item requires item_kind '{ITEM_KIND_TELEGRAM_MESSAGE}'"
        )));
    }
    let Some(prepared) = prepare_source_item(&item)? else {
        return Ok(TelegramItemInsertOutcome::Skipped {
            reason_code: "empty_payload",
        });
    };

    let existing: Option<i64> = sqlx::query_scalar(
        r#"
            SELECT item_id
            FROM telegram_messages
            WHERE source_id = ?
              AND history_peer_kind = ?
              AND history_peer_id = ?
              AND telegram_message_id = ?
            "#,
    )
    .bind(source_id)
    .bind(&identity.history_peer_kind)
    .bind(identity.history_peer_id)
    .bind(identity.telegram_message_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if let Some(item_id) = existing {
        return Ok(TelegramItemInsertOutcome::DuplicateObserved { item_id });
    }

    let item_id: i64 = sqlx::query_scalar(
        r#"
            INSERT INTO items (
                source_id,
                external_id,
                item_kind,
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
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
    )
    .bind(source_id)
    .bind(identity.telegram_message_id.to_string())
    .bind(&item.item_kind)
    .bind(&item.author)
    .bind(item.published_at)
    .bind(now_secs())
    .bind(prepared.content_zstd)
    .bind(prepared.raw_data_zstd)
    .bind(prepared.content_kind)
    .bind(prepared.has_media)
    .bind(prepared.media_kind)
    .bind(prepared.media_metadata_zstd)
    .bind(item.telegram_context.reply_to_msg_id)
    .bind(&item.telegram_context.reply_to_peer_kind)
    .bind(&item.telegram_context.reply_to_peer_id)
    .bind(item.telegram_context.reply_to_top_id)
    .bind(item.telegram_context.reaction_count)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;

    sqlx::query(
        r#"
            INSERT INTO telegram_messages (
                item_id,
                source_id,
                history_peer_kind,
                history_peer_id,
                telegram_message_id,
                migration_domain,
                is_migrated_history,
                reply_to_msg_id,
                reply_to_peer_kind,
                reply_to_peer_id,
                reply_to_top_id,
                reaction_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
    )
    .bind(item_id)
    .bind(source_id)
    .bind(&identity.history_peer_kind)
    .bind(identity.history_peer_id)
    .bind(identity.telegram_message_id)
    .bind(&identity.migration_domain)
    .bind(i64::from(identity.is_migrated_history))
    .bind(item.telegram_context.reply_to_msg_id)
    .bind(&item.telegram_context.reply_to_peer_kind)
    .bind(
        item.telegram_context
            .reply_to_peer_id
            .as_deref()
            .and_then(|value| value.parse::<i64>().ok()),
    )
    .bind(item.telegram_context.reply_to_top_id)
    .bind(item.telegram_context.reaction_count)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;

    crate::topic_memberships::resolve_scoped_topic_memberships_on_connection(
        conn,
        source_id,
        &[item_id],
        now_secs(),
    )
    .await?;

    crate::analysis_documents::upsert_item_backed_document_on_connection(conn, item_id).await?;
    match archive_maintenance {
        ArchiveReadMaintenanceMode::MaintainSingleWrite => {
            crate::archive_read_model::upsert_item_on_connection(conn, item_id).await?;
        }
        ArchiveReadMaintenanceMode::MarkSourceStaleOnly => {
            crate::archive_read_model::mark_source_stale_on_connection(conn, source_id).await?;
        }
    }

    Ok(TelegramItemInsertOutcome::Inserted { item_id })
}

pub(crate) async fn upsert_youtube_transcript_item(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    source_id: i64,
    external_id: &str,
    author: Option<&str>,
    published_at: i64,
    content: &str,
    raw_data: &impl Serialize,
) -> AppResult<i64> {
    let content_zstd = compress_text(content).map_err(AppError::internal)?;
    let raw_data_json =
        serde_json::to_vec(raw_data).map_err(|e| AppError::internal(e.to_string()))?;
    let raw_data_zstd = compress_json_bytes(&raw_data_json).map_err(AppError::internal)?;

    sqlx::query_scalar(
        r#"
        INSERT INTO items (
            source_id,
            external_id,
            item_kind,
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
        VALUES (?, ?, ?, ?, ?, strftime('%s','now'), ?, ?, 'text_only', 0, NULL, NULL)
        ON CONFLICT(source_id, external_id)
        WHERE item_kind <> 'telegram_message'
        DO UPDATE SET
            item_kind = excluded.item_kind,
            author = excluded.author,
            published_at = excluded.published_at,
            ingested_at = excluded.ingested_at,
            content_zstd = excluded.content_zstd,
            raw_data_zstd = excluded.raw_data_zstd,
            content_kind = excluded.content_kind,
            has_media = excluded.has_media,
            media_kind = excluded.media_kind,
            media_metadata_zstd = excluded.media_metadata_zstd
        RETURNING id
        "#,
    )
    .bind(source_id)
    .bind(external_id)
    .bind(ITEM_KIND_YOUTUBE_TRANSCRIPT)
    .bind(author)
    .bind(published_at)
    .bind(content_zstd)
    .bind(raw_data_zstd)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn upsert_youtube_comment_item(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    source_id: i64,
    comment: &crate::youtube::dto::YoutubeComment,
) -> AppResult<i64> {
    let content_zstd = compress_text(&comment.text).map_err(AppError::internal)?;
    let raw_data_json =
        serde_json::to_vec(comment).map_err(|e| AppError::internal(e.to_string()))?;
    let raw_data_zstd = compress_json_bytes(&raw_data_json).map_err(AppError::internal)?;
    let external_id = format!("comment:{}", comment.comment_id);

    let item_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO items (
            source_id,
            external_id,
            item_kind,
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
        VALUES (?, ?, ?, ?, ?, strftime('%s','now'), ?, ?, 'text_only', 0, NULL, NULL, NULL, NULL, NULL, NULL, ?)
        ON CONFLICT(source_id, external_id)
        WHERE item_kind <> 'telegram_message'
        DO UPDATE SET
            item_kind = excluded.item_kind,
            author = excluded.author,
            published_at = excluded.published_at,
            ingested_at = excluded.ingested_at,
            content_zstd = excluded.content_zstd,
            raw_data_zstd = excluded.raw_data_zstd,
            content_kind = excluded.content_kind,
            has_media = excluded.has_media,
            media_kind = excluded.media_kind,
            media_metadata_zstd = excluded.media_metadata_zstd,
            reaction_count = excluded.reaction_count
        RETURNING id
        "#,
    )
    .bind(source_id)
    .bind(external_id)
    .bind(ITEM_KIND_YOUTUBE_COMMENT)
    .bind(comment.author.as_deref())
    .bind(comment.published_at)
    .bind(content_zstd)
    .bind(raw_data_zstd)
    .bind(comment.like_count)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    crate::analysis_documents::upsert_item_backed_document_on_connection(&mut **tx, item_id)
        .await?;
    Ok(item_id)
}

#[tauri::command]
pub async fn list_source_items(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    request: ListSourceItemsRequest,
) -> AppResult<Vec<ItemRecord>> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    let limit = request.limit.clamp(1, 200);
    let rows = load_item_rows_from_pool(
        &pool,
        request.source_id,
        limit,
        request.before_published_at,
        request.topic_filter,
        request.around_item_id,
    )
    .await?;

    rows.into_iter().map(item_record_from_row).collect()
}

fn item_record_from_row(row: StoredItemRow) -> AppResult<ItemRecord> {
    let media_metadata = decode_media_metadata(row.media_metadata_zstd.as_deref())?;
    Ok(ItemRecord {
        id: row.id,
        source_id: row.source_id,
        external_id: row.external_id,
        item_kind: row.item_kind,
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
        has_raw_data: row.has_raw_data,
        forum_topic_id: row.forum_topic_id,
        forum_topic_title: row.forum_topic_title,
        forum_topic_top_message_id: row.forum_topic_top_message_id,
        reply_to_msg_id: row.reply_to_msg_id,
        reply_to_peer_kind: row.reply_to_peer_kind,
        reply_to_peer_id: row.reply_to_peer_id,
        reply_to_top_id: row.reply_to_top_id,
        reaction_count: row.reaction_count,
    })
}

pub(super) fn message_author(message: &grammers_client::message::Message) -> Option<String> {
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

pub(super) fn extract_telegram_context(
    message: &grammers_client::message::Message,
) -> TelegramItemContext {
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

pub(super) fn build_raw_payload(
    message: &grammers_client::message::Message,
    source_title: &Option<String>,
    author: &Option<String>,
    item_payload: &ExtractedItemPayload,
) -> AppResult<Vec<u8>> {
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
    .map_err(|e| AppError::internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        decode_media_metadata, encode_media_metadata, insert_source_item,
        insert_telegram_source_item, insert_telegram_source_item_outcome,
        insert_telegram_source_item_with_observation, reply_peer_context, tl,
        upsert_youtube_comment_item, upsert_youtube_transcript_item, ForumTopicFilter,
        SourceItemInsert, StoredItemRow, TelegramItemContext, TelegramItemInsertOutcome,
    };
    use crate::compression::{compress_text, decompress_bytes, decompress_text};
    use crate::media::{
        ExtractedItemPayload, ExtractedMediaPayload, ItemMediaMetadata, CONTENT_KIND_TEXT_ONLY,
        CONTENT_KIND_TEXT_WITH_MEDIA,
    };
    use crate::sources::test_support::{
        create_analysis_documents_table, create_ingest_provenance_tables,
        create_item_identity_indexes, memory_pool_with_source_items_and_topics,
    };
    use crate::sources::types::{TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE};

    #[test]
    fn forum_topic_filter_deserializes_camel_case_topic_id() {
        let filter: ForumTopicFilter =
            serde_json::from_str(r#"{"kind":"topic","topicId":200}"#).expect("deserialize");
        assert_eq!(filter, ForumTopicFilter::Topic { topic_id: 200 });
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
        create_legacy_item_external_unique_index(&pool).await;
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
                item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
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
                telegram_identity: None,
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
                item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
                author: None,
                published_at: 9999,
                payload: ExtractedItemPayload {
                    content: Some("duplicate".to_string()),
                    content_kind: CONTENT_KIND_TEXT_ONLY,
                    media: None,
                },
                raw_data: br#"{"id":42,"duplicate":true}"#.to_vec(),
                telegram_context: TelegramItemContext::default(),
                telegram_identity: None,
            },
        )
        .await
        .expect("skip duplicate");
        assert!(!duplicate);

        let row: StoredItemRow = sqlx::query_as(
            r#"
            SELECT
                id, source_id, external_id, item_kind, author, published_at, content_kind, has_media,
                media_kind, content_zstd, media_metadata_zstd,
                raw_data_zstd IS NOT NULL AS has_raw_data,
                reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id,
                reaction_count,
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
        assert_eq!(row.item_kind, ITEM_KIND_TELEGRAM_MESSAGE);
        assert_eq!(row.author.as_deref(), Some("alice"));
        assert_eq!(row.published_at, 1234);
        assert_eq!(row.content_kind, CONTENT_KIND_TEXT_WITH_MEDIA);
        assert!(row.has_media);
        assert_eq!(row.media_kind.as_deref(), Some("photo"));
        assert_eq!(row.reply_to_msg_id, Some(7));
        assert_eq!(row.reply_to_peer_kind.as_deref(), Some("channel"));
        assert_eq!(row.reply_to_peer_id.as_deref(), Some("99"));
        assert_eq!(row.reply_to_top_id, Some(5));
        assert_eq!(row.reaction_count, Some(3));
        assert_eq!(
            decompress_text(&row.content_zstd.expect("content")).expect("decode content"),
            "hello"
        );
        assert_eq!(
            decode_media_metadata(row.media_metadata_zstd.as_deref()).expect("decode media"),
            media_metadata
        );
        let raw_data_zstd: Vec<u8> = sqlx::query_scalar(
            "SELECT raw_data_zstd FROM items WHERE source_id = ? AND external_id = ?",
        )
        .bind(1_i64)
        .bind("42")
        .fetch_one(&pool)
        .await
        .expect("load raw payload");
        assert_eq!(
            decompress_bytes(&raw_data_zstd).expect("decode raw"),
            br#"{"id":42}"#.to_vec()
        );
    }

    #[tokio::test]
    async fn insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload()
    {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        seed_item_source(&pool, 1).await;
        let identity = TelegramMessageIdentity {
            history_peer_kind: "channel".to_string(),
            history_peer_id: 12345,
            telegram_message_id: 42,
            migration_domain: None,
            is_migrated_history: false,
        };

        let inserted = insert_telegram_source_item(
            &pool,
            1,
            identity.clone(),
            telegram_insert("42", "first payload"),
        )
        .await
        .expect("insert first");
        assert!(inserted);

        let duplicate = insert_telegram_source_item(
            &pool,
            1,
            identity,
            telegram_insert("42", "second payload"),
        )
        .await
        .expect("skip duplicate");
        assert!(!duplicate);

        let content: Vec<u8> = sqlx::query_scalar(
            "SELECT content_zstd FROM items WHERE source_id = 1 AND external_id = '42'",
        )
        .fetch_one(&pool)
        .await
        .expect("load content");
        assert_eq!(
            decompress_text(&content).expect("decode content"),
            "first payload"
        );

        let child_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_messages")
            .fetch_one(&pool)
            .await
            .expect("count child rows");
        assert_eq!(child_count, 1);
    }

    #[tokio::test]
    async fn telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        seed_item_source(&pool, 1).await;
        let identity = TelegramMessageIdentity {
            history_peer_kind: "channel".to_string(),
            history_peer_id: 12345,
            telegram_message_id: 42,
            migration_domain: None,
            is_migrated_history: false,
        };

        let inserted = insert_telegram_source_item_outcome(
            &pool,
            1,
            identity.clone(),
            telegram_insert("42", "first payload"),
        )
        .await
        .expect("insert first");
        let first_id = match inserted {
            TelegramItemInsertOutcome::Inserted { item_id } => item_id,
            other => panic!("expected inserted outcome, got {other:?}"),
        };

        let duplicate = insert_telegram_source_item_outcome(
            &pool,
            1,
            identity,
            telegram_insert("42", "second payload"),
        )
        .await
        .expect("observe duplicate");
        assert_eq!(
            duplicate,
            TelegramItemInsertOutcome::DuplicateObserved { item_id: first_id }
        );
    }

    #[tokio::test]
    async fn telegram_insert_writes_analysis_document_in_same_writer_transaction() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        seed_item_source(&pool, 1).await;

        let outcome = insert_telegram_source_item_outcome(
            &pool,
            1,
            telegram_identity(42),
            telegram_insert("42", "Document text"),
        )
        .await
        .expect("insert telegram item");

        let TelegramItemInsertOutcome::Inserted { item_id } = outcome else {
            panic!("expected insert");
        };

        let row: (String, String, i64, String, String) = sqlx::query_as(
            "SELECT document_kind, ref, document_order, source_type, source_subtype
             FROM analysis_documents WHERE item_id = ?",
        )
        .bind(item_id)
        .fetch_one(&pool)
        .await
        .expect("load document");
        assert_eq!(
            row,
            (
                "telegram_message".to_string(),
                format!("s1-i{item_id}"),
                item_id,
                "telegram".to_string(),
                "supergroup".to_string(),
            )
        );
    }

    #[tokio::test]
    async fn single_telegram_insert_maintains_ready_archive_model() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        crate::sources::test_support::create_archive_read_model_tables(&pool).await;
        seed_item_source(&pool, 1).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("initial rebuild");

        assert!(insert_telegram_source_item(
            &pool,
            1,
            telegram_identity(42),
            telegram_insert("42", "new ready archive row"),
        )
        .await
        .expect("insert item"));

        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM archive_read_items WHERE source_id = 1 AND ref = 's1-i1'",
        )
        .fetch_one(&pool)
        .await
        .expect("count archive row");
        assert_eq!(exists, 1);

        assert!(
            crate::archive_read_model::source_archive_model_is_ready(&pool, 1)
                .await
                .expect("ready check")
        );
    }

    #[tokio::test]
    async fn telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        create_analysis_documents_table(&pool).await;
        seed_item_source(&pool, 1).await;
        let batch_id = crate::ingest_provenance::create_telegram_takeout_batch(
            &pool,
            crate::ingest_provenance::CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create batch");
        let identity = TelegramMessageIdentity {
            history_peer_kind: "channel".to_string(),
            history_peer_id: 12345,
            telegram_message_id: 42,
            migration_domain: None,
            is_migrated_history: false,
        };

        let inserted = insert_telegram_source_item_with_observation(
            &pool,
            batch_id,
            1,
            identity.clone(),
            telegram_insert("42", "first payload"),
        )
        .await
        .expect("insert with observation");
        let item_id = match inserted {
            TelegramItemInsertOutcome::Inserted { item_id } => item_id,
            other => panic!("expected insert, got {other:?}"),
        };

        let duplicate = insert_telegram_source_item_with_observation(
            &pool,
            batch_id,
            1,
            identity.clone(),
            telegram_insert("42", "duplicate payload"),
        )
        .await
        .expect("duplicate with observation");
        assert_eq!(
            duplicate,
            TelegramItemInsertOutcome::DuplicateObserved { item_id }
        );

        let empty_item = SourceItemInsert {
            payload: ExtractedItemPayload {
                content: None,
                content_kind: CONTENT_KIND_TEXT_ONLY,
                media: None,
            },
            ..telegram_insert("43", "")
        };
        let skipped_identity = TelegramMessageIdentity {
            telegram_message_id: 43,
            ..identity
        };
        let skipped = insert_telegram_source_item_with_observation(
            &pool,
            batch_id,
            1,
            skipped_identity,
            empty_item,
        )
        .await
        .expect("skipped with observation");
        assert_eq!(
            skipped,
            TelegramItemInsertOutcome::Skipped {
                reason_code: "empty_payload"
            }
        );

        let rows: Vec<(String, Option<i64>, String, Option<String>)> = sqlx::query_as(
            "SELECT outcome, item_id, provider_identity, reason_code
             FROM ingest_item_observations
             WHERE batch_id = ?
             ORDER BY id",
        )
        .bind(batch_id)
        .fetch_all(&pool)
        .await
        .expect("load observations");

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0, "inserted");
        assert_eq!(rows[0].1, Some(item_id));
        assert_eq!(rows[0].2, "telegram:history_peer:channel:12345:message:42");
        assert_eq!(rows[1].0, "duplicate_observed");
        assert_eq!(rows[1].1, Some(item_id));
        assert_eq!(rows[2].0, "skipped");
        assert_eq!(rows[2].1, None);
        assert_eq!(rows[2].3.as_deref(), Some("empty_payload"));
    }

    #[tokio::test]
    async fn takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        create_ingest_provenance_tables(&pool).await;
        crate::sources::test_support::create_archive_read_model_tables(&pool).await;
        seed_item_source(&pool, 1).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("initial rebuild");
        let batch_id = crate::ingest_provenance::create_telegram_takeout_batch(
            &pool,
            crate::ingest_provenance::CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create batch");

        let outcome = insert_telegram_source_item_with_observation(
            &pool,
            batch_id,
            1,
            telegram_identity(77),
            telegram_insert("77", "bulk row"),
        )
        .await
        .expect("bulk insert");

        assert!(outcome.is_inserted());
        let state = crate::archive_read_model::load_source_state(&pool, 1)
            .await
            .expect("load state")
            .expect("state exists");
        assert_eq!(state.status, crate::archive_read_model::STATUS_STALE);
    }

    #[tokio::test]
    async fn insert_telegram_source_item_resolves_topic_membership_only_for_new_item() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        seed_item_source(&pool, 1).await;
        sqlx::query(
            "INSERT INTO telegram_forum_topics (
                source_id, topic_id, top_message_id, title, last_seen_at, updated_at
             ) VALUES (1, 200, 700, 'Roadmap', 100, 100)",
        )
        .execute(&pool)
        .await
        .expect("seed topic");
        sqlx::query(
            "INSERT INTO telegram_topic_resolution_state (
                source_id, resolver_version, status, unresolved_count, pending_item_count
             ) VALUES (1, 1, 'ready', 0, 0)",
        )
        .execute(&pool)
        .await
        .expect("seed ready state");

        let identity = TelegramMessageIdentity {
            history_peer_kind: "channel".to_string(),
            history_peer_id: 12345,
            telegram_message_id: 701,
            migration_domain: None,
            is_migrated_history: false,
        };
        let mut item = telegram_insert("701", "topic reply");
        item.telegram_context.reply_to_top_id = Some(200);

        assert!(
            insert_telegram_source_item(&pool, 1, identity.clone(), item)
                .await
                .expect("insert")
        );
        assert!(!insert_telegram_source_item(
            &pool,
            1,
            identity,
            telegram_insert("701", "duplicate")
        )
        .await
        .expect("duplicate"));

        let membership_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM item_topic_memberships WHERE source_id = 1")
                .fetch_one(&pool)
                .await
                .expect("count memberships");
        assert_eq!(membership_count, 1);

        let state: (String, i64, i64) = sqlx::query_as(
            "SELECT status, unresolved_count, pending_item_count FROM telegram_topic_resolution_state WHERE source_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("load state");
        assert_eq!(state, ("ready".to_string(), 0, 0));
    }

    #[tokio::test]
    async fn scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        seed_item_source(&pool, 1).await;
        sqlx::query(
            "INSERT INTO telegram_topic_resolution_state (
                source_id, resolver_version, status, unresolved_count, pending_item_count
             ) VALUES (1, 1, 'ready', 2, 0)",
        )
        .execute(&pool)
        .await
        .expect("seed ready state");

        let identity = TelegramMessageIdentity {
            history_peer_kind: "channel".to_string(),
            history_peer_id: 12345,
            telegram_message_id: 900,
            migration_domain: None,
            is_migrated_history: false,
        };

        assert!(insert_telegram_source_item(
            &pool,
            1,
            identity,
            telegram_insert("900", "unmatched")
        )
        .await
        .expect("insert unmatched"));

        let state: (String, i64, i64) = sqlx::query_as(
            "SELECT status, unresolved_count, pending_item_count FROM telegram_topic_resolution_state WHERE source_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("load state");
        assert_eq!(state, ("ready".to_string(), 3, 0));
    }

    #[tokio::test]
    async fn insert_telegram_source_item_allows_same_message_id_in_different_history_domains() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        seed_item_source(&pool, 1).await;

        let first = TelegramMessageIdentity {
            history_peer_kind: "channel".to_string(),
            history_peer_id: 12345,
            telegram_message_id: 42,
            migration_domain: None,
            is_migrated_history: false,
        };
        let second = TelegramMessageIdentity {
            history_peer_kind: "chat".to_string(),
            history_peer_id: 777,
            telegram_message_id: 42,
            migration_domain: Some("migrated_from_chat".to_string()),
            is_migrated_history: true,
        };

        assert!(
            insert_telegram_source_item(&pool, 1, first, telegram_insert("42", "current"))
                .await
                .expect("insert current")
        );
        assert!(
            insert_telegram_source_item(&pool, 1, second, telegram_insert("42", "migrated"))
                .await
                .expect("insert migrated")
        );

        let item_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM items WHERE source_id = 1 AND external_id = '42'",
        )
        .fetch_one(&pool)
        .await
        .expect("count items");
        assert_eq!(item_count, 2);
    }

    #[tokio::test]
    async fn youtube_transcript_upsert_targets_non_telegram_partial_unique_index() {
        let pool = memory_pool_with_source_items_and_topics().await;
        sqlx::query("DROP INDEX IF EXISTS idx_items_ext")
            .execute(&pool)
            .await
            .expect("drop legacy index fixture");
        create_item_identity_indexes(&pool).await;

        let mut tx = pool.begin().await.expect("begin transaction");
        let first_id = upsert_youtube_transcript_item(
            &mut tx,
            1,
            "transcript:video01:en:manual",
            Some("Demo Channel"),
            1_700_000_000,
            "old transcript",
            &serde_json::json!({ "version": 1 }),
        )
        .await
        .expect("insert transcript");
        let second_id = upsert_youtube_transcript_item(
            &mut tx,
            1,
            "transcript:video01:en:manual",
            Some("Demo Channel"),
            1_700_000_001,
            "new transcript",
            &serde_json::json!({ "version": 2 }),
        )
        .await
        .expect("update transcript");
        tx.commit().await.expect("commit");

        assert_eq!(first_id, second_id);
    }

    #[tokio::test]
    async fn upsert_youtube_transcript_item_updates_existing_text_and_returns_id() {
        let pool = memory_pool_with_source_items_and_topics().await;
        let mut tx = pool.begin().await.expect("begin transaction");

        let first_id = upsert_youtube_transcript_item(
            &mut tx,
            1,
            "transcript:video01:en:manual",
            Some("Demo Channel"),
            1_700_000_000,
            "old transcript",
            &serde_json::json!({ "version": 1 }),
        )
        .await
        .expect("insert transcript");
        let second_id = upsert_youtube_transcript_item(
            &mut tx,
            1,
            "transcript:video01:en:manual",
            Some("Demo Channel"),
            1_700_000_001,
            "new transcript",
            &serde_json::json!({ "version": 2 }),
        )
        .await
        .expect("update transcript");
        tx.commit().await.expect("commit");

        assert_eq!(first_id, second_id);

        let row: StoredItemRow = sqlx::query_as(
            r#"
            SELECT
                id, source_id, external_id, item_kind, author, published_at, content_kind, has_media,
                media_kind, content_zstd, media_metadata_zstd,
                raw_data_zstd IS NOT NULL AS has_raw_data,
                reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id,
                reaction_count,
                NULL AS forum_topic_id, NULL AS forum_topic_title, NULL AS forum_topic_top_message_id
            FROM items
            WHERE id = ?
            "#,
        )
        .bind(first_id)
        .fetch_one(&pool)
        .await
        .expect("load transcript item");

        assert_eq!(row.external_id, "transcript:video01:en:manual");
        assert_eq!(row.item_kind, "youtube_transcript");
        assert_eq!(row.author.as_deref(), Some("Demo Channel"));
        assert_eq!(row.published_at, 1_700_000_001);
        assert_eq!(row.content_kind, CONTENT_KIND_TEXT_ONLY);
        assert!(!row.has_media);
        assert_eq!(
            decompress_text(&row.content_zstd.expect("content")).expect("decode content"),
            "new transcript"
        );
        let raw_data_zstd: Vec<u8> =
            sqlx::query_scalar("SELECT raw_data_zstd FROM items WHERE id = ?")
                .bind(first_id)
                .fetch_one(&pool)
                .await
                .expect("load raw transcript payload");
        assert_eq!(
            decompress_bytes(&raw_data_zstd).expect("decode raw"),
            serde_json::to_vec(&serde_json::json!({ "version": 2 })).expect("json")
        );
    }

    #[tokio::test]
    async fn youtube_comment_upsert_targets_non_telegram_partial_unique_index() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        sqlx::query("DROP INDEX IF EXISTS idx_items_ext")
            .execute(&pool)
            .await
            .expect("drop legacy index fixture");
        create_item_identity_indexes(&pool).await;

        let mut tx = pool.begin().await.expect("begin transaction");
        let mut comment = crate::youtube::dto::YoutubeComment {
            comment_id: "UgPartial".to_string(),
            parent_comment_id: None,
            is_reply: false,
            author: Some("Alice".to_string()),
            author_channel_id: None,
            author_channel_url: None,
            published_at: 1_700_000_000,
            text: "old comment".to_string(),
            like_count: Some(1),
            is_pinned: None,
            is_hearted: None,
            raw_payload: serde_json::json!({ "id": "UgPartial" }),
        };
        let first_id = upsert_youtube_comment_item(&mut tx, 1, &comment)
            .await
            .expect("insert comment");
        comment.text = "new comment".to_string();
        comment.like_count = Some(5);
        let second_id = upsert_youtube_comment_item(&mut tx, 1, &comment)
            .await
            .expect("update comment");
        tx.commit().await.expect("commit");

        assert_eq!(first_id, second_id);
    }

    #[tokio::test]
    async fn youtube_comment_upsert_writes_analysis_document_and_updates_content() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        seed_youtube_video_source(&pool, 2).await;

        let mut tx = pool.begin().await.expect("begin tx");
        let first = upsert_youtube_comment_item(&mut tx, 2, &youtube_comment("c1", "First"))
            .await
            .expect("first comment");
        tx.commit().await.expect("commit first");

        let content: Vec<u8> =
            sqlx::query_scalar("SELECT content_zstd FROM analysis_documents WHERE item_id = ?")
                .bind(first)
                .fetch_one(&pool)
                .await
                .expect("load first document");
        assert_eq!(
            decompress_text(&content).expect("decompress first"),
            "First"
        );

        let mut tx = pool.begin().await.expect("begin tx");
        let second = upsert_youtube_comment_item(&mut tx, 2, &youtube_comment("c1", "Second"))
            .await
            .expect("second comment");
        tx.commit().await.expect("commit second");
        assert_eq!(first, second);

        let content: Vec<u8> =
            sqlx::query_scalar("SELECT content_zstd FROM analysis_documents WHERE item_id = ?")
                .bind(first)
                .fetch_one(&pool)
                .await
                .expect("load updated document");
        assert_eq!(
            decompress_text(&content).expect("decompress second"),
            "Second"
        );
    }

    #[tokio::test]
    async fn upsert_youtube_comment_item_updates_existing_text_and_reaction_count() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        let mut tx = pool.begin().await.expect("begin transaction");

        let mut comment = crate::youtube::dto::YoutubeComment {
            comment_id: "Ugabc".to_string(),
            parent_comment_id: None,
            is_reply: false,
            author: Some("Alice".to_string()),
            author_channel_id: Some("UCalice".to_string()),
            author_channel_url: Some("https://www.youtube.com/@alice".to_string()),
            published_at: 1_700_000_000,
            text: "old comment".to_string(),
            like_count: Some(3),
            is_pinned: Some(false),
            is_hearted: Some(false),
            raw_payload: serde_json::json!({ "id": "Ugabc", "text": "old comment" }),
        };
        let first_id = upsert_youtube_comment_item(&mut tx, 1, &comment)
            .await
            .expect("insert comment");

        comment.text = "new comment".to_string();
        comment.like_count = Some(9);
        comment.raw_payload = serde_json::json!({ "id": "Ugabc", "text": "new comment" });
        let second_id = upsert_youtube_comment_item(&mut tx, 1, &comment)
            .await
            .expect("update comment");
        tx.commit().await.expect("commit");

        assert_eq!(first_id, second_id);

        let row: StoredItemRow = sqlx::query_as(
            r#"
            SELECT
                id, source_id, external_id, item_kind, author, published_at, content_kind, has_media,
                media_kind, content_zstd, media_metadata_zstd,
                raw_data_zstd IS NOT NULL AS has_raw_data,
                reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id,
                reaction_count,
                NULL AS forum_topic_id, NULL AS forum_topic_title, NULL AS forum_topic_top_message_id
            FROM items
            WHERE id = ?
            "#,
        )
        .bind(first_id)
        .fetch_one(&pool)
        .await
        .expect("load comment item");

        let reaction_count: Option<i64> =
            sqlx::query_scalar("SELECT reaction_count FROM items WHERE id = ?")
                .bind(first_id)
                .fetch_one(&pool)
                .await
                .expect("load reaction count");

        assert_eq!(row.external_id, "comment:Ugabc");
        assert_eq!(row.item_kind, "youtube_comment");
        assert_eq!(row.author.as_deref(), Some("Alice"));
        assert_eq!(row.published_at, 1_700_000_000);
        assert_eq!(row.content_kind, CONTENT_KIND_TEXT_ONLY);
        assert!(!row.has_media);
        assert_eq!(reaction_count, Some(9));
        assert_eq!(
            decompress_text(&row.content_zstd.expect("content")).expect("decode content"),
            "new comment"
        );

        let raw_data_zstd: Vec<u8> =
            sqlx::query_scalar("SELECT raw_data_zstd FROM items WHERE id = ?")
                .bind(first_id)
                .fetch_one(&pool)
                .await
                .expect("load raw comment payload");
        let raw: serde_json::Value =
            serde_json::from_slice(&decompress_bytes(&raw_data_zstd).expect("decode raw"))
                .expect("decode raw json");
        assert_eq!(raw["comment_id"], "Ugabc");
        assert_eq!(raw["raw_payload"]["text"], "new comment");
    }

    #[test]
    fn reply_peer_context_uses_telegram_peer_kinds() {
        assert_eq!(
            reply_peer_context(Some(&tl::enums::Peer::User(tl::types::PeerUser {
                user_id: 11
            }))),
            Some(("user", "11".to_string()))
        );
        assert_eq!(
            reply_peer_context(Some(&tl::enums::Peer::Chat(tl::types::PeerChat {
                chat_id: 22
            }))),
            Some(("chat", "22".to_string()))
        );
        assert_eq!(
            reply_peer_context(Some(&tl::enums::Peer::Channel(tl::types::PeerChannel {
                channel_id: 33
            }))),
            Some(("channel", "33".to_string()))
        );
        assert_eq!(reply_peer_context(None), None);
    }

    async fn create_legacy_item_external_unique_index(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_items_ext ON items(source_id, external_id)",
        )
        .execute(pool)
        .await
        .expect("create legacy items unique index");
    }

    async fn seed_item_source(pool: &sqlx::SqlitePool, source_id: i64) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
             VALUES (?, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .bind(source_id)
        .execute(pool)
        .await
        .expect("seed source");
    }

    async fn seed_youtube_video_source(pool: &sqlx::SqlitePool, source_id: i64) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (?, 'youtube', 'video', ?, 'Video', 1, 1, 1)",
        )
        .bind(source_id)
        .bind(format!("video{source_id}"))
        .execute(pool)
        .await
        .expect("seed youtube source");
    }

    fn telegram_identity(message_id: i64) -> TelegramMessageIdentity {
        TelegramMessageIdentity {
            history_peer_kind: "channel".to_string(),
            history_peer_id: 12345,
            telegram_message_id: message_id,
            migration_domain: None,
            is_migrated_history: false,
        }
    }

    fn telegram_insert(external_id: &str, content: &str) -> SourceItemInsert {
        SourceItemInsert {
            external_id: external_id.to_string(),
            item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
            author: Some("alice".to_string()),
            published_at: 1234,
            payload: ExtractedItemPayload {
                content: Some(content.to_string()),
                content_kind: CONTENT_KIND_TEXT_ONLY,
                media: None,
            },
            raw_data: serde_json::to_vec(&serde_json::json!({ "id": external_id }))
                .expect("raw json"),
            telegram_context: TelegramItemContext::default(),
            telegram_identity: None,
        }
    }

    fn youtube_comment(comment_id: &str, text: &str) -> crate::youtube::dto::YoutubeComment {
        crate::youtube::dto::YoutubeComment {
            comment_id: comment_id.to_string(),
            parent_comment_id: None,
            is_reply: false,
            author: Some("Alice".to_string()),
            author_channel_id: None,
            author_channel_url: None,
            published_at: 1_700_000_000,
            text: text.to_string(),
            like_count: Some(1),
            is_pinned: None,
            is_hearted: None,
            raw_payload: serde_json::json!({ "id": comment_id, "text": text }),
        }
    }
}
