use grammers_client::{tl, Client};
use grammers_session::types::PeerRef;
use serde::Serialize;
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::forum_topics::{FORUM_TOPIC_UNCATEGORIZED_KEY, FORUM_TOPIC_UNCATEGORIZED_TITLE};

use super::identity_repair::{require_source_identity_ready, SourceIdentityRepairState};
use super::types::{now_secs, SourceForumTopicRow, SourceSyncTarget, TelegramSourceKind};

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

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TopicResolutionStateSummary {
    pub status: String,
    pub resolver_version: i64,
    pub unresolved_count: i64,
    pub pending_item_count: i64,
    pub memberships_refreshed_at: Option<i64>,
}

#[derive(Serialize)]
pub struct SourceForumTopicsResponse {
    pub topics: Vec<SourceForumTopicRecord>,
    pub topic_resolution_state: TopicResolutionStateSummary,
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

pub(super) async fn refresh_forum_topics(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    client: &Client,
    peer: PeerRef,
    source: &SourceSyncTarget,
) -> Vec<String> {
    let supports_forum_topics = match source_supports_forum_topics(pool, source.id).await {
        Ok(supports_forum_topics) => supports_forum_topics,
        Err(error) => {
            return vec![format!(
                "Forum topic refresh failed for source {}: {error}",
                source.id
            )];
        }
    };
    if !supports_forum_topics {
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
        Err(error) if is_non_forum_topic_refresh_error(&error.message) => Vec::new(),
        Err(error) => vec![format!(
            "Forum topic refresh failed for source {}: {error}",
            source.id
        )],
    }
}

async fn source_supports_forum_topics(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<bool> {
    let identity = crate::sources::identity::load_telegram_source_identity(pool, source_id).await?;
    Ok(identity.source_subtype == TelegramSourceKind::Supergroup)
}

async fn fetch_all_forum_topics(
    client: &Client,
    peer: PeerRef,
) -> AppResult<(Vec<ForumTopicSnapshot>, Vec<i64>)> {
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
            .map_err(|e| AppError::network(e.to_string()))?;

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
) -> AppResult<()> {
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
        .map_err(|e| AppError::internal(e.to_string()))?;
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
        .map_err(|e| AppError::internal(e.to_string()))?;
    }

    crate::topic_memberships::rebuild_topic_memberships_for_source(pool, source_id, refreshed_at)
        .await?;

    Ok(())
}

fn is_non_forum_topic_refresh_error(error: &str) -> bool {
    error.contains("CHANNEL_FORUM_MISSING") || error.contains("CHANNEL_MONOFORUM_UNSUPPORTED")
}

#[tauri::command]
pub async fn list_source_forum_topics(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    source_id: i64,
) -> AppResult<SourceForumTopicsResponse> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    list_source_forum_topics_from_pool(&pool, source_id).await
}

async fn list_source_forum_topics_from_pool(
    pool: &sqlx::SqlitePool,
    source_id: i64,
) -> AppResult<SourceForumTopicsResponse> {
    let state = crate::topic_memberships::load_topic_resolution_state(pool, source_id).await?;
    let is_ready = crate::topic_memberships::is_ready_current_state(state.as_ref());
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
            COUNT(memberships.item_id) AS message_count
        FROM telegram_forum_topics AS topics
        LEFT JOIN item_topic_memberships AS memberships
          ON memberships.source_id = topics.source_id
         AND memberships.topic_id = topics.topic_id
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
    .map_err(|e| AppError::internal(e.to_string()))?;

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

    let unresolved_count = state.as_ref().map(|row| row.unresolved_count).unwrap_or(0);
    if is_ready && unresolved_count > 0 {
        records.push(SourceForumTopicRecord {
            kind: "uncategorized".to_string(),
            key: FORUM_TOPIC_UNCATEGORIZED_KEY.to_string(),
            title: FORUM_TOPIC_UNCATEGORIZED_TITLE.to_string(),
            message_count: unresolved_count,
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

    Ok(SourceForumTopicsResponse {
        topics: records,
        topic_resolution_state: state_summary_from_row(state.as_ref()),
    })
}

fn state_summary_from_row(
    row: Option<&crate::topic_memberships::TopicResolutionStateRow>,
) -> TopicResolutionStateSummary {
    match row {
        Some(row) => TopicResolutionStateSummary {
            status: row.status.clone(),
            resolver_version: row.resolver_version,
            unresolved_count: row.unresolved_count,
            pending_item_count: row.pending_item_count,
            memberships_refreshed_at: row.memberships_refreshed_at,
        },
        None => TopicResolutionStateSummary {
            status: crate::topic_memberships::TOPIC_STATE_NEVER_RUN.to_string(),
            resolver_version: crate::topic_memberships::CURRENT_TOPIC_RESOLVER_VERSION,
            unresolved_count: 0,
            pending_item_count: 0,
            memberships_refreshed_at: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_non_forum_topic_refresh_error, list_source_forum_topics_from_pool,
        source_supports_forum_topics, upsert_forum_topics_from_refresh, ForumTopicSnapshot,
    };
    use crate::sources::test_support::memory_pool_with_source_items_and_topics;

    #[tokio::test]
    async fn forum_topic_refresh_gate_uses_typed_identity_not_legacy_kind() {
        let pool = memory_pool_with_source_items_and_topics().await;
        for (source_id, source_subtype) in [(10_i64, "channel"), (11_i64, "supergroup")] {
            sqlx::query(
                r#"
                INSERT INTO sources (
                    id, source_type, source_subtype, account_id, external_id,
                    title, metadata_zstd, last_sync_state, is_active, is_member,
                    created_at
                )
                VALUES (?, 'telegram', ?, ?, ?, ?, NULL, NULL, 1, 1, ?)
                "#,
            )
            .bind(source_id)
            .bind(source_subtype)
            .bind(42_i64)
            .bind(source_id.to_string())
            .bind(format!("source {source_id}"))
            .bind(1_i64)
            .execute(&pool)
            .await
            .expect("insert source");
            sqlx::query(
                r#"
                INSERT INTO telegram_sources (
                    source_id, account_id, source_subtype, peer_kind, peer_id,
                    resolution_strategy, username, access_hash, avatar_cache_key,
                    identity_refreshed_at, created_at, updated_at
                )
                VALUES (?, ?, ?, 'channel', ?, 'legacy_metadata', NULL, ?, NULL, ?, ?, ?)
                "#,
            )
            .bind(source_id)
            .bind(42_i64)
            .bind(source_subtype)
            .bind(source_id)
            .bind(1000_i64 + source_id)
            .bind(1_i64)
            .bind(1_i64)
            .bind(1_i64)
            .execute(&pool)
            .await
            .expect("insert typed identity");
        }

        assert!(!source_supports_forum_topics(&pool, 10)
            .await
            .expect("load channel identity"));
        assert!(source_supports_forum_topics(&pool, 11)
            .await
            .expect("load supergroup identity"));
    }

    #[tokio::test]
    async fn forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists() {
        let pool = memory_pool_with_source_items_and_topics().await;
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id,
                title, metadata_zstd, last_sync_state, is_active, is_member,
                created_at
            )
            VALUES (11, 'telegram', 'supergroup', 42, '11', 'source 11', x'00', NULL, 1, 1, 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy, username, access_hash, avatar_cache_key,
                identity_refreshed_at, created_at, updated_at
            )
            VALUES (11, 42, 'supergroup', 'channel', 11, 'legacy_metadata', NULL, 1011, NULL, 1, 1, 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert typed identity");

        assert!(source_supports_forum_topics(&pool, 11)
            .await
            .expect("load typed identity"));
    }

    #[tokio::test]
    async fn list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket() {
        let pool = memory_pool_with_source_items_and_topics().await;
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed source");

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
        for (item_id, topic_id, match_kind) in [
            (1_i64, 11_i64, "legacy_root_external_id"),
            (2_i64, 11_i64, "reply_to_top_id"),
            (3_i64, 1_i64, "general_fallback"),
            (4_i64, 22_i64, "reply_to_top_id"),
            (5_i64, 22_i64, "reply_to_msg_id"),
        ] {
            sqlx::query(
                "INSERT INTO item_topic_memberships (
                    item_id, source_id, topic_id, match_kind, resolver_version
                 ) VALUES (?, 1, ?, ?, 1)",
            )
            .bind(item_id)
            .bind(topic_id)
            .bind(match_kind)
            .execute(&pool)
            .await
            .expect("insert topic membership");
        }
        sqlx::query(
            "INSERT INTO telegram_topic_resolution_state (
                source_id, resolver_version, status, unresolved_count, pending_item_count
             ) VALUES (1, 1, 'ready', 1, 0)",
        )
        .execute(&pool)
        .await
        .expect("insert ready topic state");

        let response = list_source_forum_topics_from_pool(&pool, 1)
            .await
            .expect("list source forum topics");
        let records = response.topics;

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
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', '1', 'Forum', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed source");

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

    #[tokio::test]
    async fn topic_refresh_rebuilds_materialized_memberships() {
        let pool = memory_pool_with_source_items_and_topics().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, account_id, external_id, title, is_active, is_member, created_at)
             VALUES (1, 'telegram', 'supergroup', 42, '1', 'Forum', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at,
                ingested_at, content_kind, has_media, reply_to_top_id
             ) VALUES (10, 1, '10', 'telegram_message', 'alice', 10, 10, 'text_only', 0, 55)",
        )
        .execute(&pool)
        .await
        .expect("seed item");

        upsert_forum_topics_from_refresh(
            &pool,
            1,
            &[ForumTopicSnapshot {
                topic_id: 55,
                top_message_id: 500,
                title: "Fresh".to_string(),
                icon_color: 1,
                icon_emoji_id: None,
                is_closed: false,
                is_pinned: false,
                is_hidden: false,
                sort_order: 0,
            }],
            &[],
            2000,
        )
        .await
        .expect("refresh topics and rebuild");

        let topic_id: i64 =
            sqlx::query_scalar("SELECT topic_id FROM item_topic_memberships WHERE item_id = 10")
                .fetch_one(&pool)
                .await
                .expect("load membership");
        assert_eq!(topic_id, 55);

        let state: (String, i64) = sqlx::query_as(
            "SELECT status, unresolved_count FROM telegram_topic_resolution_state WHERE source_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("load state");
        assert_eq!(state, ("ready".to_string(), 0));
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
