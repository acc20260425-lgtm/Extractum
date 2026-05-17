use sqlx::{Sqlite, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(crate) const CURRENT_TOPIC_RESOLVER_VERSION: i64 = 1;
pub(crate) const TOPIC_STATE_NEVER_RUN: &str = "never_run";
pub(crate) const TOPIC_STATE_READY: &str = "ready";
pub(crate) const TOPIC_STATE_DIRTY: &str = "dirty";
pub(crate) const TOPIC_STATE_REBUILDING: &str = "rebuilding";
pub(crate) const TOPIC_STATE_FAILED: &str = "failed";
pub(crate) const TOPIC_LAST_ERROR_MAX_CHARS: usize = 1000;

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
    let _ = (conn, source_id, refreshed_at, visible_rebuilding);
    Err(AppError::internal(
        "topic membership rebuild is not implemented",
    ))
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
