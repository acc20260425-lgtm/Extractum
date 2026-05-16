use grammers_client::tl;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;

use crate::compression::{compress_json_bytes, compress_text, decompress_text};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::media::{decode_media_metadata, encode_media_metadata, ExtractedItemPayload};

use super::identity_repair::{require_source_identity_ready, SourceIdentityRepairState};
use super::types::{
    now_secs, StoredItemRow, ITEM_KIND_YOUTUBE_COMMENT, ITEM_KIND_YOUTUBE_TRANSCRIPT,
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
    pub(crate) external_id: String,
    pub(crate) item_kind: String,
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

pub(crate) async fn insert_source_item(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    item: SourceItemInsert,
) -> AppResult<bool> {
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
        return Ok(false);
    }

    let raw_data_zstd = compress_json_bytes(&item.raw_data).map_err(AppError::internal)?;
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
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(result.rows_affected() == 1)
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
        ON CONFLICT(source_id, external_id) DO UPDATE SET
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
            media_metadata_zstd,
            reply_to_msg_id,
            reply_to_peer_kind,
            reply_to_peer_id,
            reply_to_top_id,
            reaction_count
        )
        VALUES (?, ?, ?, ?, ?, strftime('%s','now'), ?, ?, 'text_only', 0, NULL, NULL, NULL, NULL, NULL, NULL, ?)
        ON CONFLICT(source_id, external_id) DO UPDATE SET
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
    .map_err(AppError::database)
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
        has_raw_data: row.raw_data_zstd.is_some(),
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
        decode_media_metadata, encode_media_metadata, insert_source_item, reply_peer_context, tl,
        upsert_youtube_comment_item, upsert_youtube_transcript_item, ForumTopicFilter,
        SourceItemInsert, StoredItemRow, TelegramItemContext,
    };
    use crate::compression::{compress_text, decompress_bytes, decompress_text};
    use crate::media::{
        ExtractedItemPayload, ExtractedMediaPayload, ItemMediaMetadata, CONTENT_KIND_TEXT_ONLY,
        CONTENT_KIND_TEXT_WITH_MEDIA,
    };
    use crate::sources::test_support::memory_pool_with_source_items_and_topics;
    use crate::sources::types::ITEM_KIND_TELEGRAM_MESSAGE;

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
            },
        )
        .await
        .expect("skip duplicate");
        assert!(!duplicate);

        let row: StoredItemRow = sqlx::query_as(
            r#"
            SELECT
                id, source_id, external_id, item_kind, author, published_at, content_kind, has_media,
                media_kind, content_zstd, media_metadata_zstd, raw_data_zstd,
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
        assert_eq!(
            decompress_bytes(&row.raw_data_zstd.expect("raw")).expect("decode raw"),
            br#"{"id":42}"#.to_vec()
        );
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
                media_kind, content_zstd, media_metadata_zstd, raw_data_zstd,
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
        assert_eq!(
            decompress_bytes(&row.raw_data_zstd.expect("raw")).expect("decode raw"),
            serde_json::to_vec(&serde_json::json!({ "version": 2 })).expect("json")
        );
    }

    #[tokio::test]
    async fn upsert_youtube_comment_item_updates_existing_text_and_reaction_count() {
        let pool = memory_pool_with_source_items_and_topics().await;
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
                media_kind, content_zstd, media_metadata_zstd, raw_data_zstd,
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

        let raw: serde_json::Value = serde_json::from_slice(
            &decompress_bytes(&row.raw_data_zstd.expect("raw")).expect("decode raw"),
        )
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
}
