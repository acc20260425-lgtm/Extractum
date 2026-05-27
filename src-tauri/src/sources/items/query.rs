use crate::error::{AppError, AppResult};
use crate::sources::StoredItemRow;

use super::ForumTopicFilter;

pub(super) async fn load_item_rows_from_pool(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
    around_item_id: Option<i64>,
) -> AppResult<Vec<StoredItemRow>> {
    if crate::archive_read_model::source_archive_model_is_ready(pool, source_id).await? {
        return crate::archive_read_model::load_item_rows_from_archive(
            pool,
            source_id,
            limit,
            before_published_at,
            topic_filter,
            around_item_id,
        )
        .await;
    }

    load_item_rows_from_items_path(
        pool,
        source_id,
        limit,
        before_published_at,
        topic_filter,
        around_item_id,
    )
    .await
}

pub(crate) async fn load_item_rows_from_items_path(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
    around_item_id: Option<i64>,
) -> AppResult<Vec<StoredItemRow>> {
    let around_published_at = if let Some(item_id) = around_item_id {
        sqlx::query_scalar::<_, i64>(
            "SELECT items.published_at
             FROM items
             WHERE items.source_id = ?
               AND items.id = ?
               AND NOT EXISTS (
                 SELECT 1 FROM telegram_messages tm
                 WHERE tm.item_id = items.id
                   AND tm.is_migrated_history = 1
               )
             LIMIT 1",
        )
        .bind(source_id)
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?
    } else {
        None
    };

    let state = crate::topic_memberships::load_topic_resolution_state(pool, source_id).await?;
    let is_ready = crate::topic_memberships::is_ready_current_state(state.as_ref());
    if matches!(topic_filter.as_ref(), Some(ForumTopicFilter::Uncategorized)) && !is_ready {
        return Ok(Vec::new());
    }

    let mut sql = String::from(
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
            items.raw_data_zstd IS NOT NULL AS has_raw_data,
            items.reply_to_msg_id,
            items.reply_to_peer_kind,
            items.reply_to_peer_id,
            items.reply_to_top_id,
            items.reaction_count,
            forum_topics.topic_id AS forum_topic_id,
            forum_topics.title AS forum_topic_title,
            forum_topics.top_message_id AS forum_topic_top_message_id
        FROM items
        LEFT JOIN item_topic_memberships AS memberships
          ON memberships.item_id = items.id
        LEFT JOIN telegram_forum_topics AS forum_topics
          ON forum_topics.source_id = memberships.source_id
         AND forum_topics.topic_id = memberships.topic_id
        WHERE items.source_id = ?
          AND NOT EXISTS (
            SELECT 1 FROM telegram_messages tm
            WHERE tm.item_id = items.id
              AND tm.is_migrated_history = 1
          )
        "#,
    );

    if before_published_at.is_some() {
        sql.push_str(" AND items.published_at < ?");
    } else if around_published_at.is_some() {
        sql.push_str(" AND items.published_at <= ?");
    }

    match topic_filter.as_ref() {
        Some(ForumTopicFilter::Topic { .. }) => {
            sql.push_str(" AND memberships.topic_id = ?");
        }
        Some(ForumTopicFilter::Uncategorized) => {
            sql.push_str(" AND memberships.item_id IS NULL");
        }
        None => {}
    }

    sql.push_str(" ORDER BY items.published_at DESC LIMIT ?");

    let mut query = sqlx::query_as::<_, StoredItemRow>(&sql).bind(source_id);
    if let Some(before) = before_published_at {
        query = query.bind(before);
    } else if let Some(around) = around_published_at {
        query = query.bind(around);
    }
    if let Some(ForumTopicFilter::Topic { topic_id }) = topic_filter.as_ref() {
        query = query.bind(*topic_id);
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
    use crate::compression::compress_text;
    use crate::sources::items::ForumTopicFilter;
    use crate::sources::test_support::memory_pool_with_source_items_and_topics;

    async fn seed_direct_item(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        item_id: i64,
        external_id: &str,
        published_at: i64,
        content: &str,
    ) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (?, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .bind(source_id)
        .execute(pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_zstd, raw_data_zstd, content_kind, has_media
             ) VALUES (?, ?, ?, 'telegram_message', 'alice', ?, ?, ?, NULL, 'text_only', 0)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(external_id)
        .bind(published_at)
        .bind(published_at)
        .bind(crate::compression::compress_text(content).expect("compress"))
        .execute(pool)
        .await
        .expect("seed item");
    }

    #[tokio::test]
    async fn default_items_path_excludes_migrated_history_rows() {
        let pool = memory_pool_with_source_items_and_topics().await;
        seed_direct_item(&pool, 1, 10, "10", 1000, "current").await;
        seed_direct_item(&pool, 1, 11, "11", 900, "migrated").await;
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (10, 1, 'channel', 12345, 10, NULL, 0),
                      (11, 1, 'chat', 777, 10, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed telegram messages");

        let rows = load_item_rows_from_items_path(&pool, 1, 20, None, None, None)
            .await
            .expect("load rows");

        assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![10]);
    }

    #[tokio::test]
    async fn default_source_browsing_does_not_surface_migrated_rows_after_archive_ready() {
        let pool = memory_pool_with_source_items_and_topics().await;
        crate::sources::test_support::create_archive_read_model_tables(&pool).await;
        seed_direct_item(&pool, 1, 10, "10", 1000, "current").await;
        seed_direct_item(&pool, 1, 11, "11", 900, "migrated").await;
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (10, 1, 'channel', 12345, 10, NULL, 0),
                      (11, 1, 'chat', 777, 10, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed telegram rows");
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive");

        let rows = load_item_rows_from_pool(&pool, 1, 20, None, None, None)
            .await
            .expect("load rows");

        assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![10]);
    }

    #[tokio::test]
    async fn load_item_rows_attaches_topic_metadata_and_root_matches() {
        let pool = memory_pool_with_source_items_and_topics().await;
        sqlx::query(
            "INSERT OR IGNORE INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed source");
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
            (1_i64, "not-numeric-root", 500_i64, None, None, None),
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
        sqlx::query(
            "INSERT INTO telegram_messages (item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id)
             VALUES (1, 1, 'channel', 12345, 700)",
        )
        .execute(&pool)
        .await
        .expect("insert typed message identity");
        for (item_id, topic_id, match_kind) in [
            (1_i64, 200_i64, "typed_root_top_message_id"),
            (2_i64, 200_i64, "reply_to_top_id"),
            (3_i64, 200_i64, "reply_to_msg_id"),
            (4_i64, 1_i64, "general_fallback"),
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

        let rows = load_item_rows_from_items_path(&pool, 1, 20, None, None, None)
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

        let topic_rows = load_item_rows_from_items_path(
            &pool,
            1,
            20,
            None,
            Some(ForumTopicFilter::Topic { topic_id: 200 }),
            None,
        )
        .await
        .expect("load topic rows");
        assert_eq!(topic_rows.len(), 3);
        assert!(topic_rows.iter().all(|row| row.forum_topic_id == Some(200)));

        let general_rows = load_item_rows_from_items_path(
            &pool,
            1,
            20,
            None,
            Some(ForumTopicFilter::Topic { topic_id: 1 }),
            None,
        )
        .await
        .expect("load general rows");
        assert_eq!(general_rows.len(), 1);
        assert_eq!(general_rows[0].external_id, "999");

        let uncategorized_rows = load_item_rows_from_items_path(
            &pool,
            1,
            20,
            None,
            Some(ForumTopicFilter::Uncategorized),
            None,
        )
        .await
        .expect("load uncategorized rows");
        assert_eq!(uncategorized_rows.len(), 1);
        assert_eq!(uncategorized_rows[0].external_id, "1000");
    }

    #[tokio::test]
    async fn uncategorized_filter_returns_empty_when_topic_resolution_is_not_ready() {
        let pool = memory_pool_with_source_items_and_topics().await;
        sqlx::query(
            "INSERT OR IGNORE INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO telegram_topic_resolution_state (
                source_id, resolver_version, status, unresolved_count, pending_item_count
             ) VALUES (1, 1, 'dirty', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed dirty state");
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_kind, has_media
             ) VALUES (1, 1, '100', 'telegram_message', 'alice', 100, 100, 'text_only', 0)",
        )
        .execute(&pool)
        .await
        .expect("seed item");

        let rows = load_item_rows_from_items_path(
            &pool,
            1,
            20,
            None,
            Some(ForumTopicFilter::Uncategorized),
            None,
        )
        .await
        .expect("load uncategorized rows");

        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn load_item_rows_can_start_at_selected_item() {
        let pool = memory_pool_with_source_items_and_topics().await;
        for (id, external_id, published_at) in [
            (10_i64, "100", 500_i64),
            (11_i64, "101", 400_i64),
            (12_i64, "102", 300_i64),
        ] {
            sqlx::query(
                r#"
                INSERT INTO items (
                    id, source_id, external_id, item_kind, author, published_at, ingested_at,
                    content_zstd, raw_data_zstd, content_kind, has_media, media_kind,
                    media_metadata_zstd, reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id,
                    reply_to_top_id, reaction_count
                ) VALUES (?, 1, ?, 'telegram_message', 'alice', ?, ?, NULL, NULL, 'text_only', 0, NULL, NULL, NULL, NULL, NULL, NULL, NULL)
                "#,
            )
            .bind(id)
            .bind(external_id)
            .bind(published_at)
            .bind(published_at)
            .execute(&pool)
            .await
            .expect("insert item");
        }

        let rows = load_item_rows_from_items_path(&pool, 1, 2, None, None, Some(11))
            .await
            .expect("load around selected item");

        assert_eq!(
            rows.iter()
                .map(|row| row.external_id.as_str())
                .collect::<Vec<_>>(),
            vec!["101", "102"]
        );
    }

    #[tokio::test]
    async fn archive_reader_matches_items_path_for_source_browsing_rows() {
        let pool = memory_pool_with_source_items_and_topics().await;
        crate::sources::test_support::create_archive_read_model_tables(&pool).await;
        seed_browsing_parity_fixture(&pool).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive rows");

        let old_rows = load_item_rows_from_items_path(&pool, 1, 20, None, None, None)
            .await
            .expect("load old path");
        let new_rows =
            crate::archive_read_model::load_item_rows_from_archive(&pool, 1, 20, None, None, None)
                .await
                .expect("load archive path");

        assert_eq!(new_rows, old_rows);
    }

    #[tokio::test]
    async fn archive_reader_matches_topic_filter_and_around_item_semantics() {
        let pool = memory_pool_with_source_items_and_topics().await;
        crate::sources::test_support::create_archive_read_model_tables(&pool).await;
        seed_browsing_parity_fixture(&pool).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive rows");

        for filter in [
            Some(ForumTopicFilter::Topic { topic_id: 200 }),
            Some(ForumTopicFilter::Uncategorized),
            None,
        ] {
            let old_rows =
                load_item_rows_from_items_path(&pool, 1, 2, None, filter.clone(), Some(11))
                    .await
                    .expect("load old path");
            let new_rows = crate::archive_read_model::load_item_rows_from_archive(
                &pool,
                1,
                2,
                None,
                filter,
                Some(11),
            )
            .await
            .expect("load archive path");

            assert_eq!(new_rows, old_rows);
        }
    }

    #[tokio::test]
    async fn load_item_rows_uses_items_path_when_archive_model_is_not_ready() {
        let pool = memory_pool_with_source_items_and_topics().await;
        seed_source_browsing_fixture(&pool).await;

        let rows = load_item_rows_from_pool(&pool, 1, 20, None, None, None)
            .await
            .expect("load fallback rows");

        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].external_id, "not-numeric-root");
    }

    #[tokio::test]
    async fn load_item_rows_uses_archive_path_when_ready_and_current() {
        let pool = memory_pool_with_source_items_and_topics().await;
        crate::sources::test_support::create_archive_read_model_tables(&pool).await;
        seed_source_browsing_fixture(&pool).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive rows");

        sqlx::query(
            "UPDATE items
             SET external_id = 'canonical-mutated-after-archive-build'
             WHERE source_id = 1 AND external_id = 'not-numeric-root'",
        )
        .execute(&pool)
        .await
        .expect("mutate canonical row after archive build");

        let rows = load_item_rows_from_pool(&pool, 1, 20, None, None, None)
            .await
            .expect("load archive rows");

        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].external_id, "not-numeric-root");
    }

    #[tokio::test]
    async fn load_item_rows_uses_items_path_when_archive_model_is_stale() {
        let pool = memory_pool_with_source_items_and_topics().await;
        crate::sources::test_support::create_archive_read_model_tables(&pool).await;
        seed_source_browsing_fixture(&pool).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive rows");
        crate::archive_read_model::mark_source_stale(&pool, 1)
            .await
            .expect("mark stale");

        sqlx::query("DELETE FROM archive_read_items WHERE source_id = 1")
            .execute(&pool)
            .await
            .expect("delete archive rows");

        let rows = load_item_rows_from_pool(&pool, 1, 20, None, None, None)
            .await
            .expect("load fallback rows");

        assert_eq!(rows.len(), 5);
    }

    async fn seed_source_browsing_fixture(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("seed source");
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
            .execute(pool)
            .await
            .expect("insert forum topic");
        }

        for (id, external_id, published_at, reply_to_msg_id, reply_to_top_id, reaction_count) in [
            (1_i64, "not-numeric-root", 500_i64, None, None, None),
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
            .execute(pool)
            .await
            .expect("insert item");
        }
        sqlx::query(
            "INSERT INTO telegram_messages (item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id)
             VALUES (1, 1, 'channel', 12345, 700)",
        )
        .execute(pool)
        .await
        .expect("insert typed message identity");
        for (item_id, topic_id, match_kind) in [
            (1_i64, 200_i64, "typed_root_top_message_id"),
            (2_i64, 200_i64, "reply_to_top_id"),
            (3_i64, 200_i64, "reply_to_msg_id"),
            (4_i64, 1_i64, "general_fallback"),
        ] {
            sqlx::query(
                "INSERT INTO item_topic_memberships (
                    item_id, source_id, topic_id, match_kind, resolver_version
                 ) VALUES (?, 1, ?, ?, 1)",
            )
            .bind(item_id)
            .bind(topic_id)
            .bind(match_kind)
            .execute(pool)
            .await
            .expect("insert topic membership");
        }
        sqlx::query(
            "INSERT INTO telegram_topic_resolution_state (
                source_id, resolver_version, status, unresolved_count, pending_item_count
             ) VALUES (1, 1, 'ready', 1, 0)",
        )
        .execute(pool)
        .await
        .expect("insert ready topic state");
    }

    async fn seed_browsing_parity_fixture(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("seed source");

        sqlx::query(
            "INSERT INTO telegram_forum_topics (
                source_id, topic_id, top_message_id, title, last_seen_at, updated_at
             ) VALUES (1, 200, 700, 'Roadmap', 100, 100)",
        )
        .execute(pool)
        .await
        .expect("seed topic");

        sqlx::query(
            "INSERT INTO telegram_topic_resolution_state (
                source_id, resolver_version, status, unresolved_count, pending_item_count
             ) VALUES (1, 1, 'ready', 0, 0)",
        )
        .execute(pool)
        .await
        .expect("seed topic state");

        sqlx::query(
            r#"
            INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_zstd, raw_data_zstd, content_kind, has_media, media_kind,
                media_metadata_zstd, reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id,
                reply_to_top_id, reaction_count
            ) VALUES
                (10, 1, '700', 'telegram_message', 'Ada', 500, 500, ?, ?, 'text_only', 0, NULL, NULL, NULL, NULL, NULL, NULL, NULL),
                (11, 1, '701', 'telegram_message', 'Bob', 400, 400, ?, ?, 'text_with_media', 1, 'photo', ?, 700, 'channel', '12345', 200, 4),
                (12, 1, '702', 'telegram_message', NULL, 300, 300, NULL, NULL, 'media_only', 1, 'video', ?, NULL, NULL, NULL, NULL, NULL),
                (13, 1, '703', 'telegram_message', 'Cyd', 250, 250, ?, ?, 'text_only', 0, NULL, NULL, NULL, NULL, NULL, NULL, 1)
            "#,
        )
        .bind(compress_text("Topic root").expect("compress root"))
        .bind(vec![10_u8])
        .bind(compress_text("Topic reply with media").expect("compress reply"))
        .bind(vec![11_u8])
        .bind(vec![91_u8])
        .bind(vec![92_u8])
        .bind(compress_text("Later topic item").expect("compress later"))
        .bind(vec![13_u8])
        .execute(pool)
        .await
        .expect("seed items");

        for item_id in [10_i64, 11_i64, 13_i64] {
            sqlx::query(
                "INSERT INTO item_topic_memberships (
                    item_id, source_id, topic_id, match_kind, resolver_version
                 ) VALUES (?, 1, 200, 'reply_to_top_id', 1)",
            )
            .bind(item_id)
            .execute(pool)
            .await
            .expect("seed membership");
        }
    }
}
