use sqlx::{Sqlite, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(crate) const CURRENT_TOPIC_RESOLVER_VERSION: i64 = 1;
pub(crate) const TOPIC_STATE_NEVER_RUN: &str = "never_run";
pub(crate) const TOPIC_STATE_READY: &str = "ready";
pub(crate) const TOPIC_STATE_DIRTY: &str = "dirty";
pub(crate) const TOPIC_STATE_REBUILDING: &str = "rebuilding";
pub(crate) const TOPIC_STATE_FAILED: &str = "failed";
pub(crate) const TOPIC_LAST_ERROR_MAX_CHARS: usize = 1000;

const RESOLVED_MEMBERSHIP_SELECT_SQL: &str = r#"
WITH eligible AS (
    SELECT
        items.id AS item_id,
        items.source_id,
        items.external_id,
        items.reply_to_top_id,
        items.reply_to_msg_id,
        telegram_messages.item_id AS typed_item_id,
        telegram_messages.telegram_message_id
    FROM items
    JOIN sources ON sources.id = items.source_id
    LEFT JOIN telegram_messages ON telegram_messages.item_id = items.id
    WHERE items.source_id = ?
      AND sources.source_type = 'telegram'
      AND sources.source_subtype = 'supergroup'
      AND items.item_kind = 'telegram_message'
),
candidates AS (
    SELECT e.item_id, e.source_id, t.topic_id, 'reply_to_top_id' AS match_kind, 1 AS priority
    FROM eligible e
    JOIN telegram_forum_topics t
      ON t.source_id = e.source_id
     AND e.reply_to_top_id = t.topic_id

    UNION ALL

    SELECT e.item_id, e.source_id, t.topic_id, 'typed_root_top_message_id' AS match_kind, 2 AS priority
    FROM eligible e
    JOIN telegram_forum_topics t
      ON t.source_id = e.source_id
     AND e.reply_to_top_id IS NULL
     AND e.telegram_message_id = t.top_message_id

    UNION ALL

    SELECT e.item_id, e.source_id, t.topic_id, 'legacy_root_external_id' AS match_kind, 3 AS priority
    FROM eligible e
    JOIN telegram_forum_topics t
      ON t.source_id = e.source_id
     AND e.reply_to_top_id IS NULL
     AND e.typed_item_id IS NULL
     AND e.external_id <> ''
     AND e.external_id NOT GLOB '*[^0-9]*'
     AND CAST(e.external_id AS INTEGER) = t.top_message_id

    UNION ALL

    SELECT e.item_id, e.source_id, t.topic_id, 'reply_to_msg_id' AS match_kind, 4 AS priority
    FROM eligible e
    JOIN telegram_forum_topics t
      ON t.source_id = e.source_id
     AND e.reply_to_top_id IS NULL
     AND e.reply_to_msg_id = t.topic_id

    UNION ALL

    SELECT e.item_id, e.source_id, t.topic_id, 'general_fallback' AS match_kind, 5 AS priority
    FROM eligible e
    JOIN telegram_forum_topics t
      ON t.source_id = e.source_id
     AND e.reply_to_top_id IS NULL
     AND t.topic_id = 1
),
ranked AS (
    SELECT
        item_id,
        source_id,
        topic_id,
        match_kind,
        ROW_NUMBER() OVER (PARTITION BY item_id ORDER BY priority ASC, topic_id ASC) AS rn
    FROM candidates
)
SELECT item_id, source_id, topic_id, match_kind
FROM ranked
WHERE rn = 1
"#;

#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct TopicResolutionStateRow {
    pub(crate) source_id: i64,
    pub(crate) resolver_version: i64,
    pub(crate) catalog_refreshed_at: Option<i64>,
    pub(crate) memberships_refreshed_at: Option<i64>,
    pub(crate) status: String,
    pub(crate) unresolved_count: i64,
    pub(crate) pending_item_count: i64,
    pub(crate) last_error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TopicRebuildStats {
    pub(crate) eligible_items: i64,
    pub(crate) inserted_memberships: i64,
    pub(crate) unresolved_count: i64,
}

pub(crate) fn is_ready_current_state(state: Option<&TopicResolutionStateRow>) -> bool {
    matches!(
        state,
        Some(row)
            if row.status == TOPIC_STATE_READY
                && row.resolver_version == CURRENT_TOPIC_RESOLVER_VERSION
    )
}

pub(crate) fn truncate_topic_resolution_error(error: impl AsRef<str>) -> String {
    error
        .as_ref()
        .chars()
        .take(TOPIC_LAST_ERROR_MAX_CHARS)
        .collect()
}

pub(crate) async fn create_topic_membership_schema(conn: &mut SqliteConnection) -> AppResult<()> {
    sqlx::raw_sql(TOPIC_MEMBERSHIP_SCHEMA_SQL)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn rebuild_topic_memberships_for_source_on_connection(
    conn: &mut SqliteConnection,
    source_id: i64,
    refreshed_at: i64,
    visible_rebuilding: bool,
) -> AppResult<TopicRebuildStats> {
    if visible_rebuilding {
        upsert_resolution_state(
            conn,
            source_id,
            TOPIC_STATE_REBUILDING,
            None,
            None,
            0,
            0,
            None,
            refreshed_at,
        )
        .await?;
    }

    sqlx::query("DELETE FROM item_topic_memberships WHERE source_id = ?")
        .bind(source_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let insert_sql = format!(
        "INSERT INTO item_topic_memberships (
             item_id, source_id, topic_id, match_kind, resolver_version, created_at, updated_at
         )
         SELECT item_id, source_id, topic_id, match_kind, ?, ?, ?
         FROM ({RESOLVED_MEMBERSHIP_SELECT_SQL})"
    );
    sqlx::query(&insert_sql)
        .bind(CURRENT_TOPIC_RESOLVER_VERSION)
        .bind(refreshed_at)
        .bind(refreshed_at)
        .bind(source_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let eligible = eligible_item_count(conn, source_id).await?;
    let inserted = inserted_membership_count(conn, source_id).await?;
    let unresolved = (eligible - inserted).max(0);

    let catalog_refreshed_at = source_catalog_refreshed_at(conn, source_id).await?;
    assert_ready_source_invariants(conn, source_id, eligible, inserted, unresolved).await?;
    upsert_resolution_state(
        conn,
        source_id,
        TOPIC_STATE_READY,
        catalog_refreshed_at,
        Some(refreshed_at),
        unresolved,
        0,
        None,
        refreshed_at,
    )
    .await?;

    Ok(TopicRebuildStats {
        eligible_items: eligible,
        inserted_memberships: inserted,
        unresolved_count: unresolved,
    })
}

pub(crate) async fn resolve_scoped_topic_memberships_on_connection(
    conn: &mut SqliteConnection,
    source_id: i64,
    inserted_item_ids: &[i64],
    resolved_at: i64,
) -> AppResult<()> {
    let _ = (conn, source_id, inserted_item_ids, resolved_at);
    Ok(())
}

pub(crate) async fn load_topic_resolution_state(
    pool: &sqlx::Pool<Sqlite>,
    source_id: i64,
) -> AppResult<Option<TopicResolutionStateRow>> {
    sqlx::query_as(
        r#"
        SELECT
            source_id,
            resolver_version,
            catalog_refreshed_at,
            memberships_refreshed_at,
            status,
            unresolved_count,
            pending_item_count,
            last_error
        FROM telegram_topic_resolution_state
        WHERE source_id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

async fn eligible_item_count(conn: &mut SqliteConnection, source_id: i64) -> AppResult<i64> {
    sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM items
         JOIN sources ON sources.id = items.source_id
         WHERE items.source_id = ?
           AND sources.source_type = 'telegram'
           AND sources.source_subtype = 'supergroup'
           AND items.item_kind = 'telegram_message'",
    )
    .bind(source_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)
}

async fn inserted_membership_count(conn: &mut SqliteConnection, source_id: i64) -> AppResult<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM item_topic_memberships WHERE source_id = ?")
        .bind(source_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)
}

async fn upsert_resolution_state(
    conn: &mut SqliteConnection,
    source_id: i64,
    status: &str,
    catalog_refreshed_at: Option<i64>,
    memberships_refreshed_at: Option<i64>,
    unresolved_count: i64,
    pending_item_count: i64,
    last_error: Option<&str>,
    updated_at: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO telegram_topic_resolution_state (
            source_id, resolver_version, catalog_refreshed_at, memberships_refreshed_at,
            status, unresolved_count, pending_item_count, last_error, updated_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(source_id) DO UPDATE SET
            resolver_version = excluded.resolver_version,
            catalog_refreshed_at = excluded.catalog_refreshed_at,
            memberships_refreshed_at = excluded.memberships_refreshed_at,
            status = excluded.status,
            unresolved_count = excluded.unresolved_count,
            pending_item_count = excluded.pending_item_count,
            last_error = excluded.last_error,
            updated_at = excluded.updated_at",
    )
    .bind(source_id)
    .bind(CURRENT_TOPIC_RESOLVER_VERSION)
    .bind(catalog_refreshed_at)
    .bind(memberships_refreshed_at)
    .bind(status)
    .bind(unresolved_count)
    .bind(pending_item_count)
    .bind(last_error.map(truncate_topic_resolution_error))
    .bind(updated_at)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn source_catalog_refreshed_at(
    conn: &mut SqliteConnection,
    source_id: i64,
) -> AppResult<Option<i64>> {
    sqlx::query_scalar(
        "SELECT MAX(COALESCE(updated_at, last_seen_at))
         FROM telegram_forum_topics
         WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)
}

async fn assert_ready_source_invariants(
    conn: &mut SqliteConnection,
    source_id: i64,
    eligible: i64,
    inserted: i64,
    unresolved: i64,
) -> AppResult<()> {
    if inserted + unresolved != eligible {
        return Err(AppError::validation(format!(
            "Topic membership rebuild for source {source_id} has inconsistent counts: inserted {inserted}, unresolved {unresolved}, eligible {eligible}"
        )));
    }

    let source_mismatch: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM item_topic_memberships m
         JOIN items i ON i.id = m.item_id
         WHERE m.source_id = ? AND m.source_id <> i.source_id",
    )
    .bind(source_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if source_mismatch != 0 {
        return Err(AppError::validation(format!(
            "Topic membership rebuild for source {source_id} produced {source_mismatch} source mismatches"
        )));
    }

    let version_mismatch: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM item_topic_memberships
         WHERE source_id = ? AND resolver_version <> ?",
    )
    .bind(source_id)
    .bind(CURRENT_TOPIC_RESOLVER_VERSION)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if version_mismatch != 0 {
        return Err(AppError::validation(format!(
            "Topic membership rebuild for source {source_id} produced {version_mismatch} stale resolver versions"
        )));
    }

    Ok(())
}

pub(crate) const TOPIC_MEMBERSHIP_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS item_topic_memberships (
    item_id INTEGER PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL,
    match_kind TEXT NOT NULL,
    resolver_version INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY (source_id, topic_id)
        REFERENCES telegram_forum_topics(source_id, topic_id)
        ON DELETE CASCADE,
    CHECK (match_kind IN (
        'reply_to_top_id',
        'typed_root_top_message_id',
        'legacy_root_external_id',
        'reply_to_msg_id',
        'general_fallback'
    )),
    CHECK (resolver_version > 0)
);

CREATE INDEX IF NOT EXISTS idx_item_topic_memberships_source_topic
    ON item_topic_memberships(source_id, topic_id);

CREATE INDEX IF NOT EXISTS idx_item_topic_memberships_source_item
    ON item_topic_memberships(source_id, item_id);

CREATE TABLE IF NOT EXISTS telegram_topic_resolution_state (
    source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    resolver_version INTEGER NOT NULL,
    catalog_refreshed_at INTEGER,
    memberships_refreshed_at INTEGER,
    status TEXT NOT NULL,
    unresolved_count INTEGER NOT NULL DEFAULT 0,
    pending_item_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (resolver_version > 0),
    CHECK (status IN ('never_run', 'ready', 'dirty', 'rebuilding', 'failed')),
    CHECK (unresolved_count >= 0),
    CHECK (pending_item_count >= 0)
);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rebuild_prioritizes_specific_topic_matches_before_general_fallback() {
        let pool = resolver_pool().await;
        seed_supergroup_source(&pool, 1).await;
        seed_topic(&pool, 1, 10, 1000, "Specific", false, false).await;
        seed_topic(&pool, 1, 1, 1, "General", false, false).await;
        seed_item(&pool, 101, 1, "999", Some(10), None).await;
        seed_item(&pool, 102, 1, "1000", None, None).await;
        seed_typed_message(&pool, 102, 1, 1000).await;
        seed_item(&pool, 103, 1, "1001", None, Some(10)).await;
        seed_item(&pool, 104, 1, "1002", None, None).await;

        let mut conn = pool.acquire().await.expect("acquire");
        let stats = rebuild_topic_memberships_for_source_on_connection(&mut conn, 1, 1234, false)
            .await
            .expect("rebuild");

        assert_eq!(stats.eligible_items, 4);
        assert_eq!(stats.inserted_memberships, 4);
        assert_eq!(stats.unresolved_count, 0);

        let rows: Vec<(i64, i64, String)> = sqlx::query_as(
            "SELECT item_id, topic_id, match_kind FROM item_topic_memberships ORDER BY item_id",
        )
        .fetch_all(&pool)
        .await
        .expect("load memberships");
        assert_eq!(
            rows,
            vec![
                (101, 10, "reply_to_top_id".to_string()),
                (102, 10, "typed_root_top_message_id".to_string()),
                (103, 10, "reply_to_msg_id".to_string()),
                (104, 1, "general_fallback".to_string()),
            ]
        );
    }

    #[tokio::test]
    async fn rebuild_uses_legacy_root_only_without_typed_child() {
        let pool = resolver_pool().await;
        seed_supergroup_source(&pool, 1).await;
        seed_topic(&pool, 1, 20, 700, "Root", false, false).await;
        seed_item(&pool, 201, 1, "700", None, None).await;
        seed_item(&pool, 202, 1, "700", None, None).await;
        seed_typed_message(&pool, 202, 1, 701).await;

        let mut conn = pool.acquire().await.expect("acquire");
        let stats = rebuild_topic_memberships_for_source_on_connection(&mut conn, 1, 1234, false)
            .await
            .expect("rebuild");

        assert_eq!(stats.eligible_items, 2);
        assert_eq!(stats.inserted_memberships, 1);
        assert_eq!(stats.unresolved_count, 1);

        let rows: Vec<(i64, i64, String)> = sqlx::query_as(
            "SELECT item_id, topic_id, match_kind FROM item_topic_memberships ORDER BY item_id",
        )
        .fetch_all(&pool)
        .await
        .expect("load memberships");
        assert_eq!(rows, vec![(201, 20, "legacy_root_external_id".to_string())]);
    }

    #[tokio::test]
    async fn rebuild_matches_retained_hidden_and_deleted_topics() {
        let pool = resolver_pool().await;
        seed_supergroup_source(&pool, 1).await;
        seed_topic(&pool, 1, 30, 300, "Hidden", true, false).await;
        seed_topic(&pool, 1, 40, 400, "Deleted", false, true).await;
        seed_item(&pool, 301, 1, "301", Some(30), None).await;
        seed_item(&pool, 401, 1, "401", Some(40), None).await;

        let mut conn = pool.acquire().await.expect("acquire");
        rebuild_topic_memberships_for_source_on_connection(&mut conn, 1, 1234, false)
            .await
            .expect("rebuild");

        let topics: Vec<i64> =
            sqlx::query_scalar("SELECT topic_id FROM item_topic_memberships ORDER BY item_id")
                .fetch_all(&pool)
                .await
                .expect("load topics");
        assert_eq!(topics, vec![30, 40]);
    }

    #[tokio::test]
    async fn rebuild_replaces_stale_memberships_and_versions() {
        let pool = resolver_pool().await;
        seed_supergroup_source(&pool, 1).await;
        seed_topic(&pool, 1, 50, 500, "Fresh", false, false).await;
        seed_item(&pool, 501, 1, "501", Some(50), None).await;
        sqlx::query(
            "INSERT INTO item_topic_memberships (item_id, source_id, topic_id, match_kind, resolver_version)
             VALUES (501, 1, 50, 'reply_to_top_id', 999)",
        )
        .execute(&pool)
        .await
        .expect("insert stale membership");

        let mut conn = pool.acquire().await.expect("acquire");
        rebuild_topic_memberships_for_source_on_connection(&mut conn, 1, 1234, false)
            .await
            .expect("rebuild");

        let version: i64 = sqlx::query_scalar(
            "SELECT resolver_version FROM item_topic_memberships WHERE item_id = 501",
        )
        .fetch_one(&pool)
        .await
        .expect("load resolver version");
        assert_eq!(version, CURRENT_TOPIC_RESOLVER_VERSION);
    }

    async fn resolver_pool() -> sqlx::SqlitePool {
        crate::sources::test_support::memory_pool_with_source_items_and_topics().await
    }

    async fn seed_supergroup_source(pool: &sqlx::SqlitePool, source_id: i64) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
             VALUES (?, 'telegram', 'supergroup', ?, 'Forum', 1, 1, 1)",
        )
        .bind(source_id)
        .bind(source_id.to_string())
        .execute(pool)
        .await
        .expect("seed supergroup source");
    }

    async fn seed_topic(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        topic_id: i64,
        top_message_id: i64,
        title: &str,
        hidden: bool,
        deleted: bool,
    ) {
        sqlx::query(
            "INSERT INTO telegram_forum_topics (
                source_id, topic_id, top_message_id, title, is_closed, is_pinned,
                is_hidden, is_deleted, sort_order, last_seen_at, updated_at
             ) VALUES (?, ?, ?, ?, 0, 0, ?, ?, NULL, 100, 100)",
        )
        .bind(source_id)
        .bind(topic_id)
        .bind(top_message_id)
        .bind(title)
        .bind(if hidden { 1_i64 } else { 0_i64 })
        .bind(if deleted { 1_i64 } else { 0_i64 })
        .execute(pool)
        .await
        .expect("seed topic");
    }

    async fn seed_item(
        pool: &sqlx::SqlitePool,
        item_id: i64,
        source_id: i64,
        external_id: &str,
        reply_to_top_id: Option<i64>,
        reply_to_msg_id: Option<i64>,
    ) {
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at,
                ingested_at, content_kind, has_media, reply_to_top_id, reply_to_msg_id
             ) VALUES (?, ?, ?, 'telegram_message', 'alice', ?, ?, 'text_only', 0, ?, ?)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(external_id)
        .bind(item_id)
        .bind(item_id)
        .bind(reply_to_top_id)
        .bind(reply_to_msg_id)
        .execute(pool)
        .await
        .expect("seed item");
    }

    async fn seed_typed_message(
        pool: &sqlx::SqlitePool,
        item_id: i64,
        source_id: i64,
        telegram_message_id: i64,
    ) {
        sqlx::query(
            "INSERT INTO telegram_messages (item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id)
             VALUES (?, ?, 'channel', 12345, ?)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(telegram_message_id)
        .execute(pool)
        .await
        .expect("seed typed message");
    }
}
