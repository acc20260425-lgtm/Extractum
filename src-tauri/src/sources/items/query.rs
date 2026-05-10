use crate::error::{AppError, AppResult};
use crate::forum_topics::{resolved_topic_join, ResolvedTopicAliases};
use crate::sources::types::StoredItemRow;

use super::ForumTopicFilter;

pub(super) async fn load_item_rows_from_pool(
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
            items.item_kind,
            items.author,
            items.published_at,
            items.content_kind,
            items.has_media,
            items.media_kind,
            items.content_zstd,
            items.media_metadata_zstd,
            items.raw_data_zstd,
            items.reply_to_msg_id,
            items.reply_to_peer_kind,
            items.reply_to_peer_id,
            items.reply_to_top_id,
            items.reaction_count,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::items::ForumTopicFilter;
    use crate::sources::test_support::memory_pool_with_source_items_and_topics;

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

        for (id, external_id, published_at, reply_to_msg_id, reply_to_top_id, reaction_count) in [
            (1_i64, "700", 500_i64, None, None, None),
            (2_i64, "701", 400_i64, None, Some(200_i64), Some(2_i64)),
            (3_i64, "702", 300_i64, Some(200_i64), None, Some(3_i64)),
            (4_i64, "999", 200_i64, None, None, None),
            (
                5_i64,
                "1000",
                100_i64,
                Some(123_i64),
                Some(404_i64),
                Some(5_i64),
            ),
        ] {
            sqlx::query(
                r#"
                INSERT INTO items (
                    id, source_id, external_id, item_kind, author, published_at, ingested_at, content_zstd,
                    raw_data_zstd, content_kind, has_media, media_kind, media_metadata_zstd,
                    reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id,
                    reaction_count
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(id)
            .bind(1_i64)
            .bind(external_id)
            .bind("telegram_message")
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
            .bind(reaction_count)
            .execute(&pool)
            .await
            .expect("insert item");
        }

        let rows = load_item_rows_from_pool(&pool, 1, 20, None, None)
            .await
            .expect("load all rows");
        assert_eq!(rows.len(), 5);
        assert!(rows.iter().all(|row| row.item_kind == "telegram_message"));
        assert_eq!(rows[0].forum_topic_id, Some(200));
        assert_eq!(rows[0].forum_topic_top_message_id, Some(700));
        assert_eq!(rows[1].forum_topic_id, Some(200));
        assert_eq!(rows[2].forum_topic_id, Some(200));
        assert_eq!(rows[2].reply_to_msg_id, Some(200));
        assert_eq!(rows[2].reaction_count, Some(3));
        assert_eq!(rows[3].forum_topic_id, Some(1));
        assert_eq!(rows[4].forum_topic_id, None);
        assert_eq!(rows[4].reply_to_top_id, Some(404));
        assert_eq!(rows[4].reaction_count, Some(5));

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
