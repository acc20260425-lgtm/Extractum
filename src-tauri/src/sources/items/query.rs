use crate::error::{AppError, AppResult};
use crate::sources::types::{
    SourceItemsCursor, StoredItemRow, TelegramHistoryScope, TELEGRAM_HISTORY_SCOPE_CURRENT,
    TELEGRAM_HISTORY_SCOPE_LABEL_CURRENT, TELEGRAM_SOURCE_TYPE,
};

use super::ForumTopicFilter;

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub(super) struct BrowsableItemRow {
    pub(super) id: i64,
    pub(super) source_id: i64,
    pub(super) external_id: String,
    pub(super) item_kind: String,
    pub(super) author: Option<String>,
    pub(super) published_at: i64,
    pub(super) content_kind: String,
    pub(super) has_media: bool,
    pub(super) media_kind: Option<String>,
    pub(super) content_zstd: Option<Vec<u8>>,
    pub(super) media_metadata_zstd: Option<Vec<u8>>,
    pub(super) has_raw_data: bool,
    pub(super) forum_topic_id: Option<i64>,
    pub(super) forum_topic_title: Option<String>,
    pub(super) forum_topic_top_message_id: Option<i64>,
    pub(super) reply_to_msg_id: Option<i64>,
    pub(super) reply_to_peer_kind: Option<String>,
    pub(super) reply_to_peer_id: Option<String>,
    pub(super) reply_to_top_id: Option<i64>,
    pub(super) reaction_count: Option<i64>,
    pub(super) history_scope: String,
    pub(super) is_migrated_history: bool,
    pub(super) migration_domain: Option<String>,
    pub(super) history_scope_label: String,
    pub(super) history_scope_order: i64,
    pub(super) history_peer_kind: String,
    pub(super) history_peer_id: i64,
    pub(super) telegram_message_id: i64,
}

impl BrowsableItemRow {
    pub(super) fn cursor(&self) -> SourceItemsCursor {
        SourceItemsCursor {
            published_at: self.published_at,
            history_scope_order: self.history_scope_order,
            history_peer_kind: self.history_peer_kind.clone(),
            history_peer_id: self.history_peer_id,
            telegram_message_id: self.telegram_message_id,
            item_id: self.id,
        }
    }
}

pub(super) async fn load_item_rows_from_pool(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    source_type: &str,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
    around_item_id: Option<i64>,
    history_scope: Option<TelegramHistoryScope>,
    before_cursor: Option<SourceItemsCursor>,
) -> AppResult<Vec<BrowsableItemRow>> {
    let scope = TelegramHistoryScope::from_optional(history_scope);

    if source_type == TELEGRAM_SOURCE_TYPE {
        return load_scoped_telegram_item_rows(
            pool,
            source_id,
            limit,
            topic_filter,
            around_item_id,
            scope,
            before_cursor,
        )
        .await;
    }

    if source_type != TELEGRAM_SOURCE_TYPE
        && scope == TelegramHistoryScope::Current
        && before_cursor.is_none()
        && crate::archive_read_model::source_archive_model_is_ready(pool, source_id).await?
    {
        return crate::archive_read_model::load_item_rows_from_archive(
            pool,
            source_id,
            limit,
            before_published_at,
            topic_filter,
            around_item_id,
        )
        .await
        .map(|rows| {
            rows.into_iter()
                .map(non_telegram_item_row_from_archive)
                .collect()
        });
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
) -> AppResult<Vec<BrowsableItemRow>> {
    load_scoped_item_rows(
        pool,
        source_id,
        limit,
        before_published_at,
        topic_filter,
        around_item_id,
        TelegramHistoryScope::Current,
        None,
    )
    .await
}

async fn load_scoped_telegram_item_rows(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    limit: i64,
    topic_filter: Option<ForumTopicFilter>,
    around_item_id: Option<i64>,
    scope: TelegramHistoryScope,
    before_cursor: Option<SourceItemsCursor>,
) -> AppResult<Vec<BrowsableItemRow>> {
    if scope != TelegramHistoryScope::Current && topic_filter.is_some() {
        return Err(AppError::validation(
            "Telegram forum topic filters apply only to current supergroup history",
        ));
    }

    load_scoped_item_rows(
        pool,
        source_id,
        limit,
        None,
        topic_filter,
        around_item_id,
        scope,
        before_cursor,
    )
    .await
}

async fn load_scoped_item_rows(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
    around_item_id: Option<i64>,
    scope: TelegramHistoryScope,
    before_cursor: Option<SourceItemsCursor>,
) -> AppResult<Vec<BrowsableItemRow>> {
    let state = crate::topic_memberships::load_topic_resolution_state(pool, source_id).await?;
    let is_ready = crate::topic_memberships::is_ready_current_state(state.as_ref());
    if matches!(topic_filter.as_ref(), Some(ForumTopicFilter::Uncategorized)) && !is_ready {
        return Ok(Vec::new());
    }

    let around_cursor = match around_item_id {
        Some(item_id) => {
            load_item_cursor(pool, source_id, item_id, topic_filter.as_ref(), scope).await?
        }
        None => None,
    };

    let mut sql = scoped_items_base_sql();

    match scope {
        TelegramHistoryScope::Current => {
            sql.push_str(" AND is_migrated_history = 0");
        }
        TelegramHistoryScope::Migrated => {
            sql.push_str(
                " AND is_migrated_history = 1
                  AND migration_domain = 'migrated_from_chat'",
            );
        }
        TelegramHistoryScope::Merged => {}
    }

    let page_cursor = around_cursor.as_ref().or(before_cursor.as_ref());
    if let Some(cursor) = page_cursor {
        push_after_cursor_predicate(&mut sql, cursor, around_cursor.is_some());
    } else if before_published_at.is_some() {
        sql.push_str(" AND published_at < ?");
    }

    match topic_filter.as_ref() {
        Some(ForumTopicFilter::Topic { .. }) => {
            sql.push_str(" AND forum_topic_id = ?");
        }
        Some(ForumTopicFilter::Uncategorized) => {
            sql.push_str(" AND forum_topic_id IS NULL");
        }
        None => {}
    }

    sql.push_str(
        " ORDER BY
          published_at DESC,
          history_scope_order ASC,
          history_peer_kind ASC,
          history_peer_id ASC,
          telegram_message_id ASC,
          id ASC
          LIMIT ?",
    );

    let mut query = sqlx::query_as::<_, BrowsableItemRow>(&sql).bind(source_id);
    if let Some(cursor) = page_cursor {
        query = bind_after_cursor(query, cursor);
    } else if let Some(before) = before_published_at {
        query = query.bind(before);
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

fn scoped_items_base_sql() -> String {
    String::from(
        r#"
        WITH scoped_items AS (
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
            forum_topics.top_message_id AS forum_topic_top_message_id,
            CASE WHEN COALESCE(tm.is_migrated_history, 0) = 1 THEN 'migrated' ELSE 'current' END AS history_scope,
            COALESCE(tm.is_migrated_history, 0) AS is_migrated_history,
            tm.migration_domain AS migration_domain,
            CASE
              WHEN COALESCE(tm.is_migrated_history, 0) = 1 THEN 'Migrated small-group history'
              ELSE 'Current supergroup history'
            END AS history_scope_label,
            CASE WHEN COALESCE(tm.is_migrated_history, 0) = 1 THEN 1 ELSE 0 END AS history_scope_order,
            COALESCE(tm.history_peer_kind, '') AS history_peer_kind,
            COALESCE(tm.history_peer_id, 0) AS history_peer_id,
            COALESCE(tm.telegram_message_id, 0) AS telegram_message_id
        FROM items
        LEFT JOIN telegram_messages tm ON tm.item_id = items.id
        LEFT JOIN item_topic_memberships AS memberships
          ON memberships.item_id = items.id
        LEFT JOIN telegram_forum_topics AS forum_topics
          ON forum_topics.source_id = memberships.source_id
         AND forum_topics.topic_id = memberships.topic_id
        )
        SELECT *
        FROM scoped_items
        WHERE source_id = ?
        "#,
    )
}

async fn load_item_cursor(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    item_id: i64,
    topic_filter: Option<&ForumTopicFilter>,
    scope: TelegramHistoryScope,
) -> AppResult<Option<SourceItemsCursor>> {
    let mut sql = scoped_items_base_sql();
    sql.push_str(" AND id = ?");
    match scope {
        TelegramHistoryScope::Current => {
            sql.push_str(" AND is_migrated_history = 0");
        }
        TelegramHistoryScope::Migrated => {
            sql.push_str(
                " AND is_migrated_history = 1
                  AND migration_domain = 'migrated_from_chat'",
            );
        }
        TelegramHistoryScope::Merged => {}
    }
    match topic_filter {
        Some(ForumTopicFilter::Topic { .. }) => {
            sql.push_str(" AND forum_topic_id = ?");
        }
        Some(ForumTopicFilter::Uncategorized) => {
            sql.push_str(" AND forum_topic_id IS NULL");
        }
        None => {}
    }
    sql.push_str(" LIMIT 1");

    let mut query = sqlx::query_as::<_, BrowsableItemRow>(&sql)
        .bind(source_id)
        .bind(item_id);
    if let Some(ForumTopicFilter::Topic { topic_id }) = topic_filter {
        query = query.bind(*topic_id);
    }

    query
        .fetch_optional(pool)
        .await
        .map(|row| row.map(|row| row.cursor()))
        .map_err(|e| AppError::internal(e.to_string()))
}

fn push_after_cursor_predicate(sql: &mut String, cursor: &SourceItemsCursor, inclusive: bool) {
    let item_operator = if inclusive { ">=" } else { ">" };
    let _ = cursor;
    sql.push_str(&format!(
        " AND (
            published_at < ?
            OR (
                published_at = ?
                AND (
                    history_scope_order > ?
                    OR (history_scope_order = ? AND history_peer_kind > ?)
                    OR (history_scope_order = ? AND history_peer_kind = ? AND history_peer_id > ?)
                    OR (history_scope_order = ? AND history_peer_kind = ? AND history_peer_id = ? AND telegram_message_id > ?)
                    OR (history_scope_order = ? AND history_peer_kind = ? AND history_peer_id = ? AND telegram_message_id = ? AND id {item_operator} ?)
                )
            )
        )"
    ));
}

fn bind_after_cursor<'q>(
    mut query: sqlx::query::QueryAs<
        'q,
        sqlx::Sqlite,
        BrowsableItemRow,
        <sqlx::Sqlite as sqlx::Database>::Arguments<'q>,
    >,
    cursor: &'q SourceItemsCursor,
) -> sqlx::query::QueryAs<
    'q,
    sqlx::Sqlite,
    BrowsableItemRow,
    <sqlx::Sqlite as sqlx::Database>::Arguments<'q>,
> {
    query = query
        .bind(cursor.published_at)
        .bind(cursor.published_at)
        .bind(cursor.history_scope_order)
        .bind(cursor.history_scope_order)
        .bind(&cursor.history_peer_kind)
        .bind(cursor.history_scope_order)
        .bind(&cursor.history_peer_kind)
        .bind(cursor.history_peer_id)
        .bind(cursor.history_scope_order)
        .bind(&cursor.history_peer_kind)
        .bind(cursor.history_peer_id)
        .bind(cursor.telegram_message_id)
        .bind(cursor.history_scope_order)
        .bind(&cursor.history_peer_kind)
        .bind(cursor.history_peer_id)
        .bind(cursor.telegram_message_id)
        .bind(cursor.item_id);
    query
}

fn non_telegram_item_row_from_archive(row: StoredItemRow) -> BrowsableItemRow {
    BrowsableItemRow {
        id: row.id,
        source_id: row.source_id,
        external_id: row.external_id,
        item_kind: row.item_kind,
        author: row.author,
        published_at: row.published_at,
        content_kind: row.content_kind,
        has_media: row.has_media,
        media_kind: row.media_kind,
        content_zstd: row.content_zstd,
        media_metadata_zstd: row.media_metadata_zstd,
        has_raw_data: row.has_raw_data,
        forum_topic_id: row.forum_topic_id,
        forum_topic_title: row.forum_topic_title,
        forum_topic_top_message_id: row.forum_topic_top_message_id,
        reply_to_msg_id: row.reply_to_msg_id,
        reply_to_peer_kind: row.reply_to_peer_kind,
        reply_to_peer_id: row.reply_to_peer_id,
        reply_to_top_id: row.reply_to_top_id,
        reaction_count: row.reaction_count,
        history_scope: TELEGRAM_HISTORY_SCOPE_CURRENT.to_string(),
        is_migrated_history: false,
        migration_domain: None,
        history_scope_label: TELEGRAM_HISTORY_SCOPE_LABEL_CURRENT.to_string(),
        history_scope_order: 0,
        history_peer_kind: String::new(),
        history_peer_id: 0,
        telegram_message_id: row.id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_text;
    use crate::sources::items::ForumTopicFilter;
    use crate::sources::test_support::memory_pool_with_source_items_and_topics;
    use crate::sources::types::{SourceItemsCursor, TelegramHistoryScope, TELEGRAM_SOURCE_TYPE};

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

    async fn seed_telegram_identity(
        pool: &sqlx::SqlitePool,
        item_id: i64,
        history_peer_kind: &str,
        history_peer_id: i64,
        telegram_message_id: i64,
        migration_domain: Option<&str>,
        is_migrated_history: bool,
    ) {
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (?, 1, ?, ?, ?, ?, ?)",
        )
        .bind(item_id)
        .bind(history_peer_kind)
        .bind(history_peer_id)
        .bind(telegram_message_id)
        .bind(migration_domain)
        .bind(i64::from(is_migrated_history))
        .execute(pool)
        .await
        .expect("seed telegram identity");
    }

    #[tokio::test]
    async fn scoped_browsing_defaults_to_current_rows() {
        let pool = memory_pool_with_source_items_and_topics().await;
        seed_direct_item(&pool, 1, 10, "10", 1000, "current").await;
        seed_direct_item(&pool, 1, 11, "11", 900, "migrated").await;
        seed_telegram_identity(&pool, 10, "channel", 12345, 10, None, false).await;
        seed_telegram_identity(&pool, 11, "chat", 777, 10, Some("migrated_from_chat"), true).await;

        let rows = load_item_rows_from_pool(
            &pool,
            1,
            TELEGRAM_SOURCE_TYPE,
            20,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("load rows");

        assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![10]);
        assert_eq!(rows[0].history_scope, "current");
    }

    #[tokio::test]
    async fn scoped_browsing_can_load_only_migrated_rows_with_labels() {
        let pool = memory_pool_with_source_items_and_topics().await;
        seed_direct_item(&pool, 1, 10, "10", 1000, "current").await;
        seed_direct_item(&pool, 1, 11, "11", 900, "migrated").await;
        seed_telegram_identity(&pool, 10, "channel", 12345, 10, None, false).await;
        seed_telegram_identity(&pool, 11, "chat", 777, 10, Some("migrated_from_chat"), true).await;

        let rows = load_item_rows_from_pool(
            &pool,
            1,
            TELEGRAM_SOURCE_TYPE,
            20,
            None,
            None,
            None,
            Some(TelegramHistoryScope::Migrated),
            None,
        )
        .await
        .expect("load rows");

        assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![11]);
        assert_eq!(rows[0].history_scope, "migrated");
        assert_eq!(rows[0].history_scope_label, "Migrated small-group history");
        assert_eq!(
            rows[0].migration_domain.as_deref(),
            Some("migrated_from_chat")
        );
    }

    #[tokio::test]
    async fn merged_browsing_uses_full_cursor_tuple_for_equal_timestamps() {
        let pool = memory_pool_with_source_items_and_topics().await;
        for (item_id, external_id, content) in [
            (10_i64, "40", "current low"),
            (11_i64, "41", "current high"),
            (12_i64, "40", "migrated old"),
        ] {
            seed_direct_item(&pool, 1, item_id, external_id, 1000, content).await;
        }
        seed_telegram_identity(&pool, 10, "channel", 12345, 40, None, false).await;
        seed_telegram_identity(&pool, 11, "channel", 12345, 41, None, false).await;
        seed_telegram_identity(&pool, 12, "chat", 777, 40, Some("migrated_from_chat"), true).await;

        let first_page = load_item_rows_from_pool(
            &pool,
            1,
            TELEGRAM_SOURCE_TYPE,
            2,
            None,
            None,
            None,
            Some(TelegramHistoryScope::Merged),
            None,
        )
        .await
        .expect("first page");

        assert_eq!(
            first_page.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![10, 11]
        );

        let cursor = first_page[1].cursor();
        let encoded_cursor = cursor.encode_opaque().expect("encode opaque cursor");
        assert_ne!(
            encoded_cursor,
            serde_json::to_string(&cursor).expect("serialize cursor")
        );
        assert_eq!(
            SourceItemsCursor::decode_opaque(&encoded_cursor).expect("decode opaque cursor"),
            cursor
        );
        let second_page = load_item_rows_from_pool(
            &pool,
            1,
            TELEGRAM_SOURCE_TYPE,
            2,
            None,
            None,
            None,
            Some(TelegramHistoryScope::Merged),
            Some(cursor),
        )
        .await
        .expect("second page");

        assert_eq!(
            second_page.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![12]
        );
    }

    #[tokio::test]
    async fn topic_filters_are_rejected_for_non_current_history_scope() {
        let pool = memory_pool_with_source_items_and_topics().await;
        seed_direct_item(&pool, 1, 10, "10", 1000, "current").await;

        let error = load_item_rows_from_pool(
            &pool,
            1,
            TELEGRAM_SOURCE_TYPE,
            20,
            None,
            Some(ForumTopicFilter::Topic { topic_id: 200 }),
            None,
            Some(TelegramHistoryScope::Merged),
            None,
        )
        .await
        .expect_err("reject topic filter");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
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

        let rows = load_item_rows_from_pool(
            &pool,
            1,
            TELEGRAM_SOURCE_TYPE,
            20,
            None,
            None,
            None,
            None,
            None,
        )
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

        assert_eq!(
            new_rows,
            old_rows.iter().map(stored_projection).collect::<Vec<_>>()
        );
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

            assert_eq!(
                new_rows,
                old_rows.iter().map(stored_projection).collect::<Vec<_>>()
            );
        }
    }

    #[tokio::test]
    async fn load_item_rows_uses_items_path_when_archive_model_is_not_ready() {
        let pool = memory_pool_with_source_items_and_topics().await;
        seed_source_browsing_fixture(&pool).await;

        let rows = load_item_rows_from_pool(
            &pool,
            1,
            TELEGRAM_SOURCE_TYPE,
            20,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("load fallback rows");

        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].external_id, "not-numeric-root");
    }

    #[tokio::test]
    async fn telegram_load_item_rows_uses_items_path_when_archive_model_is_ready() {
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

        let rows = load_item_rows_from_pool(
            &pool,
            1,
            TELEGRAM_SOURCE_TYPE,
            20,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("load direct rows");

        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].external_id, "canonical-mutated-after-archive-build");
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

        let rows = load_item_rows_from_pool(
            &pool,
            1,
            TELEGRAM_SOURCE_TYPE,
            20,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("load fallback rows");

        assert_eq!(rows.len(), 5);
    }

    fn stored_projection(row: &BrowsableItemRow) -> StoredItemRow {
        StoredItemRow {
            id: row.id,
            source_id: row.source_id,
            external_id: row.external_id.clone(),
            item_kind: row.item_kind.clone(),
            author: row.author.clone(),
            published_at: row.published_at,
            content_kind: row.content_kind.clone(),
            has_media: row.has_media,
            media_kind: row.media_kind.clone(),
            content_zstd: row.content_zstd.clone(),
            media_metadata_zstd: row.media_metadata_zstd.clone(),
            has_raw_data: row.has_raw_data,
            forum_topic_id: row.forum_topic_id,
            forum_topic_title: row.forum_topic_title.clone(),
            forum_topic_top_message_id: row.forum_topic_top_message_id,
            reply_to_msg_id: row.reply_to_msg_id,
            reply_to_peer_kind: row.reply_to_peer_kind.clone(),
            reply_to_peer_id: row.reply_to_peer_id.clone(),
            reply_to_top_id: row.reply_to_top_id,
            reaction_count: row.reaction_count,
        }
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
