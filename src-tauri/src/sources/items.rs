use grammers_client::tl;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;

use crate::compression::{compress_json_bytes, compress_text, decompress_text};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::forum_topics::{resolved_topic_join, ResolvedTopicAliases};
use crate::media::{decode_media_metadata, encode_media_metadata, ExtractedItemPayload};

use super::types::{now_secs, StoredItemRow};

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
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(result.rows_affected() == 1)
}

#[tauri::command]
pub async fn list_source_items(
    handle: AppHandle,
    request: ListSourceItemsRequest,
) -> AppResult<Vec<ItemRecord>> {
    let pool = get_pool(&handle).await?;
    let limit = request.limit.clamp(1, 200);
    let rows = load_item_rows_from_pool(
        &pool,
        request.source_id,
        limit,
        request.before_published_at,
        request.topic_filter,
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
        .map_err(|e| AppError::internal(e.to_string()))
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
        decode_media_metadata, encode_media_metadata, insert_source_item, load_item_rows_from_pool,
        reply_peer_context, tl, ForumTopicFilter, SourceItemInsert, StoredItemRow,
        TelegramItemContext,
    };
    use crate::compression::{compress_text, decompress_bytes, decompress_text};
    use crate::media::{
        ExtractedItemPayload, ExtractedMediaPayload, ItemMediaMetadata, CONTENT_KIND_TEXT_ONLY,
        CONTENT_KIND_TEXT_WITH_MEDIA,
    };
    use crate::sources::test_support::memory_pool_with_source_items_and_topics;

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
}
