use std::collections::{HashMap, HashSet};

use sqlx::FromRow;

use crate::error::{AppError, AppResult};
use crate::notebooklm_export::message_mapping::{
    map_export_rows, reply_snippet, ExportMessageRow, ReplyContext, ReplyLookupRow,
};
use crate::notebooklm_export::model::{NotebookLmExportMessage, NotebookLmExportSource};
use crate::readiness::{is_ready_current, ReadinessStatus};

#[derive(FromRow)]
struct SourceRow {
    id: i64,
    source_type: String,
    source_subtype: Option<String>,
    external_id: String,
    title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotebookLmExportSourceGroup {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) source_type: String,
    pub(crate) members: Vec<NotebookLmExportSourceGroupMember>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotebookLmExportSourceGroupMember {
    pub(crate) source_id: i64,
    pub(crate) source_title: Option<String>,
    pub(crate) source_type: String,
}

#[derive(FromRow)]
struct SourceGroupRow {
    id: i64,
    name: String,
    source_type: String,
}

#[derive(FromRow)]
struct SourceGroupMemberRow {
    source_id: i64,
    source_title: Option<String>,
    source_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ExportLoaderSelection {
    ArchiveReadModel {
        model_version: i64,
    },
    ItemsPath {
        reason: ArchiveReadinessFallbackReason,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ArchiveReadinessFallbackReason {
    MissingState,
    NeverBuilt,
    Building,
    Stale,
    Failed,
    OldModelVersion { found: i64, current: i64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportHistoryScope {
    Current,
    Migrated,
}

pub(crate) async fn load_export_source(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<NotebookLmExportSource> {
    let source: SourceRow = sqlx::query_as(
        r#"
        SELECT id, source_type, source_subtype, external_id, title
        FROM sources
        WHERE id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found(format!("Source {source_id} not found")))?;

    if source.source_type != "telegram" {
        return Err(AppError::validation(format!(
            "Source {source_id} is not a Telegram source"
        )));
    }
    let source_subtype = source
        .source_subtype
        .ok_or_else(|| AppError::validation(format!("Source {source_id} has no source_subtype")))?;
    let source_subtype = crate::sources::TelegramSourceKind::from_source_subtype(&source_subtype)?;

    Ok(NotebookLmExportSource {
        id: source.id,
        source_type: source.source_type,
        source_subtype: source_subtype.as_str().to_string(),
        external_id: source.external_id,
        title: source.title,
    })
}

pub(crate) async fn load_export_source_group(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_group_id: i64,
) -> AppResult<NotebookLmExportSourceGroup> {
    let group = sqlx::query_as::<_, SourceGroupRow>(
        r#"
        SELECT id, name, source_type
        FROM analysis_source_groups
        WHERE id = ?
        "#,
    )
    .bind(source_group_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found(format!("Source group {source_group_id} not found")))?;

    let members = sqlx::query_as::<_, SourceGroupMemberRow>(
        r#"
        SELECT
            sources.id AS source_id,
            sources.title AS source_title,
            sources.source_type AS source_type
        FROM analysis_source_group_members members
        JOIN sources ON sources.id = members.source_id
        WHERE members.group_id = ?
        ORDER BY COALESCE(sources.title, ''), sources.id
        "#,
    )
    .bind(source_group_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?
    .into_iter()
    .map(|row| NotebookLmExportSourceGroupMember {
        source_id: row.source_id,
        source_title: row.source_title,
        source_type: row.source_type,
    })
    .collect();

    Ok(NotebookLmExportSourceGroup {
        id: group.id,
        name: group.name,
        source_type: group.source_type,
        members,
    })
}

pub(crate) async fn select_notebooklm_export_loader(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<ExportLoaderSelection> {
    let Some(state) = crate::archive_read_model::load_source_state(pool, source_id).await? else {
        return Ok(ExportLoaderSelection::ItemsPath {
            reason: ArchiveReadinessFallbackReason::MissingState,
        });
    };

    let status = state.readiness_status();
    if status.is_some_and(|status| {
        is_ready_current(
            status,
            state.model_version,
            crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
        )
    }) {
        return Ok(ExportLoaderSelection::ArchiveReadModel {
            model_version: state.model_version,
        });
    }

    let reason = if status == Some(ReadinessStatus::Ready) {
        ArchiveReadinessFallbackReason::OldModelVersion {
            found: state.model_version,
            current: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
        }
    } else {
        match status {
            Some(ReadinessStatus::NeverBuilt) => ArchiveReadinessFallbackReason::NeverBuilt,
            Some(ReadinessStatus::Building) => ArchiveReadinessFallbackReason::Building,
            Some(ReadinessStatus::Stale) => ArchiveReadinessFallbackReason::Stale,
            Some(ReadinessStatus::Failed) | None => ArchiveReadinessFallbackReason::Failed,
            Some(ReadinessStatus::Ready) => unreachable!("ready status handled above"),
        }
    };

    Ok(ExportLoaderSelection::ItemsPath { reason })
}

pub(crate) async fn load_export_messages_from_items_path(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
    scope: ExportHistoryScope,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    let rows: Vec<ExportMessageRow> = match (period_from, period_to) {
        (Some(from), Some(to)) => {
            let sql = base_query(
                "items.source_id = ? AND items.published_at >= ? AND items.published_at <= ?",
                scope,
            );
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(from)
                .bind(to)
                .fetch_all(pool)
                .await
        }
        (Some(from), None) => {
            let sql = base_query("items.source_id = ? AND items.published_at >= ?", scope);
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(from)
                .fetch_all(pool)
                .await
        }
        (None, Some(to)) => {
            let sql = base_query("items.source_id = ? AND items.published_at <= ?", scope);
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(to)
                .fetch_all(pool)
                .await
        }
        (None, None) => {
            let sql = base_query("items.source_id = ?", scope);
            sqlx::query_as(&sql).bind(source_id).fetch_all(pool).await
        }
    }
    .map_err(AppError::database)?;

    let reply_contexts = load_reply_contexts_from_items_path(pool, source_id, &rows).await?;
    map_export_rows(rows, reply_contexts)
}

pub(crate) async fn load_export_messages_from_archive(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    let rows: Vec<ExportMessageRow> = match (period_from, period_to) {
        (Some(from), Some(to)) => {
            let sql = archive_base_query(
                "source_id = ? AND model_version = ? AND item_kind = 'telegram_message'
                 AND published_at >= ? AND published_at <= ?",
            );
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
                .bind(from)
                .bind(to)
                .fetch_all(pool)
                .await
        }
        (Some(from), None) => {
            let sql = archive_base_query(
                "source_id = ? AND model_version = ? AND item_kind = 'telegram_message'
                 AND published_at >= ?",
            );
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
                .bind(from)
                .fetch_all(pool)
                .await
        }
        (None, Some(to)) => {
            let sql = archive_base_query(
                "source_id = ? AND model_version = ? AND item_kind = 'telegram_message'
                 AND published_at <= ?",
            );
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
                .bind(to)
                .fetch_all(pool)
                .await
        }
        (None, None) => {
            let sql = archive_base_query(
                "source_id = ? AND model_version = ? AND item_kind = 'telegram_message'",
            );
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
                .fetch_all(pool)
                .await
        }
    }
    .map_err(AppError::database)?;

    let reply_contexts = load_reply_contexts_from_archive(pool, source_id, &rows).await?;
    map_export_rows(rows, reply_contexts)
}

pub(crate) async fn load_export_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
    scope: ExportHistoryScope,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    if scope == ExportHistoryScope::Migrated {
        return load_export_messages_from_items_path(
            pool,
            source_id,
            period_from,
            period_to,
            scope,
        )
        .await;
    }

    match select_notebooklm_export_loader(pool, source_id).await? {
        ExportLoaderSelection::ArchiveReadModel { .. } => {
            load_export_messages_from_archive(pool, source_id, period_from, period_to).await
        }
        ExportLoaderSelection::ItemsPath { .. } => {
            load_export_messages_from_items_path(pool, source_id, period_from, period_to, scope)
                .await
        }
    }
}

async fn load_reply_contexts_from_items_path(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    rows: &[ExportMessageRow],
) -> AppResult<HashMap<i64, ReplyContext>> {
    let mut contexts = HashMap::new();

    for row in rows.iter().filter(|row| row.reply_to_msg_id.is_some()) {
        if let Some(context) = load_domain_reply_context_from_items_path(pool, row).await? {
            contexts.insert(row.id, context);
            continue;
        }

        if let Some(reply_to_msg_id) = row.reply_to_msg_id {
            if let Some(context) =
                load_legacy_reply_context_from_items_path(pool, source_id, reply_to_msg_id).await?
            {
                contexts.insert(row.id, context);
            }
        }
    }

    Ok(contexts)
}

async fn load_domain_reply_context_from_items_path(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    row: &ExportMessageRow,
) -> AppResult<Option<ReplyContext>> {
    let Some(_) = row.reply_to_msg_id else {
        return Ok(None);
    };

    let reply = sqlx::query_as::<_, ReplyLookupRow>(
        r#"
        SELECT
          target_items.external_id,
          target_items.author,
          target_items.content_zstd,
          target_items.has_media,
          target_items.media_kind
        FROM telegram_messages reply_tm
        JOIN telegram_messages target_tm
          ON target_tm.source_id = reply_tm.source_id
         AND target_tm.history_peer_kind = reply_tm.history_peer_kind
         AND target_tm.history_peer_id = reply_tm.history_peer_id
         AND target_tm.telegram_message_id = reply_tm.reply_to_msg_id
        JOIN items target_items
          ON target_items.id = target_tm.item_id
        WHERE reply_tm.item_id = ?
        "#,
    )
    .bind(row.id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    reply
        .map(|reply| {
            reply_snippet(&reply).map(|snippet| ReplyContext {
                author: reply.author,
                snippet,
            })
        })
        .transpose()
}

async fn load_legacy_reply_context_from_items_path(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    reply_to_msg_id: i64,
) -> AppResult<Option<ReplyContext>> {
    let reply = sqlx::query_as::<_, ReplyLookupRow>(
        r#"
        SELECT external_id, author, content_zstd, has_media, media_kind
        FROM items
        WHERE source_id = ? AND external_id = ?
        ORDER BY id ASC
        LIMIT 1
        "#,
    )
    .bind(source_id)
    .bind(reply_to_msg_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    reply
        .map(|reply| {
            reply_snippet(&reply).map(|snippet| ReplyContext {
                author: reply.author,
                snippet,
            })
        })
        .transpose()
}

async fn load_reply_contexts_from_archive(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    rows: &[ExportMessageRow],
) -> AppResult<HashMap<i64, ReplyContext>> {
    let mut reply_ids = rows
        .iter()
        .filter_map(|row| row.reply_to_msg_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    reply_ids.sort_unstable();

    let mut contexts = HashMap::new();
    let mut lookup_by_reply_id = HashMap::new();
    for chunk in reply_ids.chunks(500) {
        if chunk.is_empty() {
            continue;
        }

        let placeholders = std::iter::repeat_n("?", chunk.len())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            r#"
            SELECT external_id, author, content_zstd, has_media, media_kind
            FROM archive_read_items
            WHERE source_id = ?
              AND model_version = ?
              AND item_kind = 'telegram_message'
              AND external_id IN ({placeholders})
            "#
        );

        let mut query = sqlx::query_as::<_, ReplyLookupRow>(&sql)
            .bind(source_id)
            .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION);
        for reply_id in chunk {
            query = query.bind(reply_id.to_string());
        }

        let lookup_rows = query.fetch_all(pool).await.map_err(AppError::database)?;
        for row in lookup_rows {
            let Ok(reply_id) = row.external_id.parse::<i64>() else {
                continue;
            };
            let snippet = reply_snippet(&row)?;
            lookup_by_reply_id.insert(
                reply_id,
                ReplyContext {
                    author: row.author,
                    snippet,
                },
            );
        }
    }

    for row in rows {
        if let Some(context) = row
            .reply_to_msg_id
            .and_then(|reply_id| lookup_by_reply_id.get(&reply_id))
        {
            contexts.insert(row.id, context.clone());
        }
    }

    Ok(contexts)
}

fn base_query(where_clause: &str, scope: ExportHistoryScope) -> String {
    let (history_scope, migration_domain, telegram_join, history_filter) = match scope {
        ExportHistoryScope::Current => (
            crate::sources::NOTEBOOKLM_HISTORY_SCOPE_CURRENT_SUPERGROUP,
            "NULL",
            "",
            r#"
      AND NOT EXISTS (
        SELECT 1 FROM telegram_messages tm
        WHERE tm.item_id = items.id
          AND tm.is_migrated_history = 1
      )"#,
        ),
        ExportHistoryScope::Migrated => (
            crate::sources::NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP,
            "tm.migration_domain",
            "JOIN telegram_messages tm ON tm.item_id = items.id",
            r#"
      AND tm.is_migrated_history = 1
      AND tm.migration_domain = 'migrated_from_chat'"#,
        ),
    };

    format!(
        r#"
    SELECT
        items.id,
        items.source_id,
        items.external_id,
        items.author,
        items.published_at,
        items.content_zstd,
        items.content_kind,
        items.has_media,
        items.media_kind,
        items.media_metadata_zstd,
        items.reply_to_msg_id,
        items.reply_to_peer_kind,
        items.reply_to_peer_id,
        items.reply_to_top_id,
        items.reaction_count,
        forum_topics.topic_id AS forum_topic_id,
        forum_topics.title AS forum_topic_title,
        forum_topics.top_message_id AS forum_topic_top_message_id,
        '{history_scope}' AS history_scope,
        {migration_domain} AS migration_domain
    FROM items
    {telegram_join}
    LEFT JOIN item_topic_memberships AS memberships
      ON memberships.item_id = items.id
    LEFT JOIN telegram_forum_topics AS forum_topics
      ON forum_topics.source_id = memberships.source_id
     AND forum_topics.topic_id = memberships.topic_id
    WHERE {where_clause}
      {history_filter}
    ORDER BY items.published_at ASC, items.id ASC
"#
    )
}

fn archive_base_query(where_clause: &str) -> String {
    format!(
        r#"
    SELECT
        item_id AS id,
        source_id,
        external_id,
        author,
        published_at,
        content_zstd,
        content_kind,
        has_media,
        media_kind,
        media_metadata_zstd,
        reply_to_msg_id,
        reply_to_peer_kind,
        reply_to_peer_id,
        reply_to_top_id,
        reaction_count,
        forum_topic_id,
        forum_topic_title,
        forum_topic_top_message_id,
        '{current_scope}' AS history_scope,
        NULL AS migration_domain
    FROM archive_read_items
    WHERE {where_clause}
      AND NOT EXISTS (
        SELECT 1 FROM telegram_messages tm
        WHERE tm.item_id = archive_read_items.item_id
          AND tm.is_migrated_history = 1
      )
    ORDER BY published_at ASC, item_id ASC
"#,
        current_scope = crate::sources::NOTEBOOKLM_HISTORY_SCOPE_CURRENT_SUPERGROUP,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        load_export_messages, load_export_messages_from_archive,
        load_export_messages_from_items_path, load_export_source, load_export_source_group,
        select_notebooklm_export_loader, ArchiveReadinessFallbackReason, ExportHistoryScope,
        ExportLoaderSelection,
    };
    use crate::compression::compress_text;
    use crate::error::AppErrorKind;
    use crate::media::{encode_media_metadata, ItemMediaMetadata};
    use crate::readiness::ReadinessStatus;

    #[test]
    fn notebooklm_export_query_file_has_no_export_row_mapping() {
        let source =
            std::fs::read_to_string("src/notebooklm_export/query.rs").expect("read query.rs");
        let mapper_function = ["fn ", "map_export_rows"].join("");

        assert!(
            !source.contains(&mapper_function),
            "NotebookLM export row mapping should live outside src/notebooklm_export/query.rs"
        );
    }

    async fn export_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_type TEXT NOT NULL,
                source_subtype TEXT,
                external_id TEXT NOT NULL,
                title TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create sources");
        sqlx::query(
            r#"
            CREATE TABLE analysis_source_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL DEFAULT 'telegram',
                created_at INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create analysis_source_groups");
        sqlx::query(
            r#"
            CREATE TABLE analysis_source_group_members (
                group_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (group_id, source_id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create analysis_source_group_members");
        sqlx::query(
            r#"
            CREATE TABLE items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id INTEGER NOT NULL,
                external_id TEXT NOT NULL,
                item_kind TEXT NOT NULL DEFAULT 'telegram_message',
                author TEXT,
                published_at INTEGER NOT NULL,
                ingested_at INTEGER NOT NULL DEFAULT 0,
                content_zstd BLOB,
                raw_data_zstd BLOB,
                content_kind TEXT NOT NULL,
                has_media INTEGER NOT NULL,
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
            CREATE TABLE telegram_forum_topics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                top_message_id INTEGER NOT NULL,
                title TEXT,
                is_deleted INTEGER NOT NULL DEFAULT 0
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
        .expect("create telegram_forum_topics source/topic index");
        crate::sources::test_support::create_telegram_messages_table(&pool).await;
        seed_materialized_topic_schema(&pool).await;
        crate::sources::test_support::create_archive_read_model_tables(&pool).await;
        pool
    }

    async fn seed_materialized_topic_schema(pool: &sqlx::SqlitePool) {
        crate::sources::test_support::create_topic_membership_tables(pool).await;
    }

    async fn seed_export_source(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
        )
        .execute(pool)
        .await
        .expect("seed export source");
    }

    async fn seed_archive_state(pool: &sqlx::SqlitePool, status: &str, model_version: i64) {
        sqlx::query(
            "INSERT INTO archive_read_model_state (
                source_id, model_version, status, built_at, item_count, row_count
             ) VALUES (1, ?, ?, 100, 0, 0)",
        )
        .bind(model_version)
        .bind(status)
        .execute(pool)
        .await
        .expect("seed archive state");
    }

    async fn seed_notebooklm_export_parity_fixture(pool: &sqlx::SqlitePool) {
        seed_export_source(pool).await;

        sqlx::query(
            "INSERT INTO telegram_forum_topics (
                source_id, topic_id, top_message_id, title, is_deleted
             ) VALUES (1, 200, 700, 'Roadmap', 0)",
        )
        .execute(pool)
        .await
        .expect("seed forum topic");

        let photo_metadata = encode_media_metadata(&ItemMediaMetadata {
            summary: Some("Photo".to_string()),
            file_name: Some("roadmap.png".to_string()),
            mime_type: Some("image/png".to_string()),
            size_bytes: Some(42),
            width: Some(640),
            height: Some(480),
            duration_seconds: None,
        })
        .expect("encode photo metadata");
        let document_metadata = encode_media_metadata(&ItemMediaMetadata {
            summary: Some("Document".to_string()),
            file_name: Some("notes.pdf".to_string()),
            mime_type: Some("application/pdf".to_string()),
            size_bytes: Some(128),
            width: None,
            height: None,
            duration_seconds: None,
        })
        .expect("encode document metadata");

        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_zstd, content_kind, has_media, media_kind, media_metadata_zstd
             ) VALUES
                (1, 1, '10', 'telegram_message', 'Bob', 10, 10, ?, 'text_only', 0, NULL, NULL),
                (2, 1, '20', 'telegram_message', 'Ada', 100, 100, ?, 'text_with_media', 1, 'photo', ?),
                (3, 1, '30', 'telegram_message', 'Cy', 110, 110, NULL, 'media_only', 1, 'document', ?),
                (4, 1, '40', 'telegram_message', 'Dana', 120, 120, ?, 'text_only', 0, NULL, NULL),
                (5, 1, '700a', 'telegram_message', 'Eve', 130, 130, ?, 'text_only', 0, NULL, NULL)",
        )
        .bind(compress_text("Original reply target").expect("compress original"))
        .bind(compress_text("Reply with link https://example.test").expect("compress reply"))
        .bind(photo_metadata)
        .bind(document_metadata)
        .bind(compress_text("Missing reply target").expect("compress missing reply"))
        .bind(compress_text("Looks numeric but is not").expect("compress nonnumeric"))
        .execute(pool)
        .await
        .expect("seed parity items");

        sqlx::query(
            "UPDATE items
             SET reply_to_msg_id = 10,
                 reply_to_peer_kind = 'channel',
                 reply_to_peer_id = '42',
                 reply_to_top_id = 200,
                 reaction_count = 3
             WHERE id = 2",
        )
        .execute(pool)
        .await
        .expect("update reply metadata");

        sqlx::query(
            "UPDATE items
             SET reply_to_msg_id = 999,
                 reply_to_peer_kind = 'channel',
                 reply_to_peer_id = '42',
                 reaction_count = 1
             WHERE id = 4",
        )
        .execute(pool)
        .await
        .expect("update missing reply metadata");

        for item_id in [2_i64, 3_i64] {
            sqlx::query(
                "INSERT INTO item_topic_memberships (
                    item_id, source_id, topic_id, match_kind, resolver_version
                 ) VALUES (?, 1, 200, 'reply_to_top_id', 1)",
            )
            .bind(item_id)
            .execute(pool)
            .await
            .expect("seed topic membership");
        }
    }

    #[tokio::test]
    async fn notebooklm_export_loader_selection_reports_all_fallback_reasons() {
        let cases = [
            (
                "never_built",
                crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                ArchiveReadinessFallbackReason::NeverBuilt,
            ),
            (
                "building",
                crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                ArchiveReadinessFallbackReason::Building,
            ),
            (
                "stale",
                crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                ArchiveReadinessFallbackReason::Stale,
            ),
            (
                "failed",
                crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                ArchiveReadinessFallbackReason::Failed,
            ),
        ];

        for (status, version, expected_reason) in cases {
            let pool = export_pool().await;
            seed_export_source(&pool).await;
            seed_archive_state(&pool, status, version).await;

            let selection = select_notebooklm_export_loader(&pool, 1)
                .await
                .expect("select loader");

            assert_eq!(
                selection,
                ExportLoaderSelection::ItemsPath {
                    reason: expected_reason
                },
                "unexpected selection for {status}"
            );
        }
    }

    #[tokio::test]
    async fn notebooklm_export_loader_selection_reports_missing_and_old_version() {
        let pool = export_pool().await;
        seed_export_source(&pool).await;

        assert_eq!(
            select_notebooklm_export_loader(&pool, 1)
                .await
                .expect("select missing state"),
            ExportLoaderSelection::ItemsPath {
                reason: ArchiveReadinessFallbackReason::MissingState
            }
        );

        seed_archive_state(
            &pool,
            ReadinessStatus::Ready.as_str(),
            crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION - 1,
        )
        .await;

        assert_eq!(
            select_notebooklm_export_loader(&pool, 1)
                .await
                .expect("select old state"),
            ExportLoaderSelection::ItemsPath {
                reason: ArchiveReadinessFallbackReason::OldModelVersion {
                    found: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION - 1,
                    current: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                }
            }
        );
    }

    #[tokio::test]
    async fn notebooklm_export_loader_selection_uses_archive_for_ready_current_state() {
        let pool = export_pool().await;
        seed_export_source(&pool).await;
        seed_archive_state(
            &pool,
            ReadinessStatus::Ready.as_str(),
            crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
        )
        .await;

        assert_eq!(
            select_notebooklm_export_loader(&pool, 1)
                .await
                .expect("select ready state"),
            ExportLoaderSelection::ArchiveReadModel {
                model_version: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
            }
        );
    }

    #[tokio::test]
    async fn load_export_source_rejects_non_telegram_before_message_loader_selection() {
        let pool = export_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (2, 'youtube', 'video', 'video-id', 'Video')",
        )
        .execute(&pool)
        .await
        .expect("seed youtube source");
        sqlx::query(
            "INSERT INTO archive_read_model_state (
                source_id, model_version, status, built_at, item_count, row_count
             ) VALUES (2, ?, 'ready', 100, 0, 0)",
        )
        .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
        .execute(&pool)
        .await
        .expect("seed ready youtube archive state");

        let error = load_export_source(&pool, 2)
            .await
            .expect_err("youtube source is rejected before message loading");

        assert!(error.to_string().contains("is not a Telegram source"));
    }

    #[tokio::test]
    async fn archive_export_loader_matches_items_path_for_notebooklm_messages() {
        let pool = export_pool().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive model");

        let old_rows =
            load_export_messages_from_items_path(&pool, 1, None, None, ExportHistoryScope::Current)
                .await
                .expect("load old path");
        let archive_rows = load_export_messages_from_archive(&pool, 1, None, None)
            .await
            .expect("load archive path");

        assert_eq!(archive_rows, old_rows);
        assert_eq!(archive_rows.len(), 5);
        assert_eq!(
            archive_rows[1].reply_to_snippet.as_deref(),
            Some("Original reply target")
        );
        assert_eq!(
            archive_rows[1].reply_to_peer_kind.as_deref(),
            Some("channel")
        );
        assert_eq!(archive_rows[1].reply_to_peer_id.as_deref(), Some("42"));
        assert_eq!(archive_rows[1].reply_to_top_id, Some(200));
        assert_eq!(archive_rows[1].reaction_count, Some(3));
        assert_eq!(
            archive_rows[1].forum_topic_title.as_deref(),
            Some("Roadmap")
        );
        assert!(!archive_rows[1].media_placeholders.is_empty());
        assert!(!archive_rows[2].media_placeholders.is_empty());
        assert_eq!(archive_rows[4].forum_topic_id, None);
    }

    #[tokio::test]
    async fn notebooklm_default_export_excludes_migrated_history_rows() {
        let pool = export_pool().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (1, 1, 'channel', 12345, 1, NULL, 0),
                      (2, 1, 'chat', 777, 1, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed telegram rows");

        let messages =
            load_export_messages_from_items_path(&pool, 1, None, None, ExportHistoryScope::Current)
                .await
                .expect("load export messages");

        assert!(messages.iter().all(|message| message.item_id != 2));
    }

    #[tokio::test]
    async fn opted_in_export_loads_migrated_rows_separately_with_markers() {
        let pool = export_pool().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_zstd, raw_data_zstd, content_kind, has_media
             ) VALUES (30, 1, '42', 'telegram_message', 'Old', 130, 130, ?, NULL, 'text_only', 0)",
        )
        .bind(crate::compression::compress_text("old history").expect("compress"))
        .execute(&pool)
        .await
        .expect("seed migrated item");
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (30, 1, 'chat', 777, 42, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed migrated identity");

        let current = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Current)
            .await
            .expect("current messages");
        let migrated = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Migrated)
            .await
            .expect("migrated messages");

        assert!(current.iter().all(|message| {
            message.history_scope == crate::sources::NOTEBOOKLM_HISTORY_SCOPE_CURRENT_SUPERGROUP
        }));
        assert_eq!(
            migrated
                .iter()
                .map(|message| message.item_id)
                .collect::<Vec<_>>(),
            vec![30]
        );
        assert_eq!(
            migrated[0].history_scope,
            crate::sources::NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP
        );
        assert_eq!(
            migrated[0].migration_domain.as_deref(),
            Some("migrated_from_chat")
        );
    }

    #[tokio::test]
    async fn current_export_archive_loader_sets_scope_markers() {
        let pool = export_pool().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive");

        let messages = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Current)
            .await
            .expect("current archive messages");

        assert!(!messages.is_empty());
        assert!(messages.iter().all(|message| {
            message.history_scope == crate::sources::NOTEBOOKLM_HISTORY_SCOPE_CURRENT_SUPERGROUP
        }));
        assert!(messages
            .iter()
            .all(|message| message.migration_domain.is_none()));
    }

    #[tokio::test]
    async fn migrated_export_reply_lookup_stays_inside_old_history_domain() {
        let pool = export_pool().await;
        seed_export_source(&pool).await;
        for (id, external_id, text, reply_to) in [
            (20_i64, "7", "current seven", None),
            (30_i64, "7", "old seven", None),
            (31_i64, "8", "old reply", Some(7_i64)),
        ] {
            sqlx::query(
                "INSERT INTO items (
                    id, source_id, external_id, item_kind, author, published_at, ingested_at,
                    content_zstd, raw_data_zstd, content_kind, has_media, reply_to_msg_id
                 ) VALUES (?, 1, ?, 'telegram_message', 'A', ?, ?, ?, NULL, 'text_only', 0, ?)",
            )
            .bind(id)
            .bind(external_id)
            .bind(id)
            .bind(id)
            .bind(crate::compression::compress_text(text).expect("compress"))
            .bind(reply_to)
            .execute(&pool)
            .await
            .expect("seed item");
        }
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history, reply_to_msg_id
             ) VALUES
                (20, 1, 'channel', 12345, 7, NULL, 0, NULL),
                (30, 1, 'chat', 777, 7, 'migrated_from_chat', 1, NULL),
                (31, 1, 'chat', 777, 8, 'migrated_from_chat', 1, 7)",
        )
        .execute(&pool)
        .await
        .expect("seed identities");

        let migrated = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Migrated)
            .await
            .expect("migrated messages");
        let reply = migrated
            .iter()
            .find(|message| message.item_id == 31)
            .expect("reply");

        assert_eq!(reply.reply_to_snippet.as_deref(), Some("old seven"));
    }

    #[tokio::test]
    async fn notebooklm_archive_export_excludes_migrated_history_rows_even_if_materialized() {
        let pool = export_pool().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (1, 1, 'channel', 12345, 1, NULL, 0),
                      (2, 1, 'chat', 777, 1, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed telegram rows");
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive");
        sqlx::query(
            "INSERT INTO archive_read_items (
                source_id, item_id, ref, external_id, item_kind, author, published_at,
                content_kind, has_media, has_raw_data, model_version, built_at
             ) VALUES (
                1, 2, 'manual-migrated', '20', 'telegram_message', 'Ada', 100,
                'text_only', 0, 0, ?, 100
             )
             ON CONFLICT(source_id, item_id) DO UPDATE SET
                ref = excluded.ref,
                model_version = excluded.model_version",
        )
        .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
        .execute(&pool)
        .await
        .expect("force archive row");

        let messages = load_export_messages_from_archive(&pool, 1, None, None)
            .await
            .expect("load archive messages");

        assert!(messages.iter().all(|message| message.item_id != 2));
    }

    #[tokio::test]
    async fn archive_export_loader_matches_items_path_for_bounded_periods() {
        let pool = export_pool().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive model");

        let old_rows = load_export_messages_from_items_path(
            &pool,
            1,
            Some(50),
            Some(115),
            ExportHistoryScope::Current,
        )
        .await
        .expect("load old bounded path");
        let archive_rows = load_export_messages_from_archive(&pool, 1, Some(50), Some(115))
            .await
            .expect("load archive bounded path");

        assert_eq!(archive_rows, old_rows);
        assert_eq!(
            archive_rows
                .iter()
                .map(|row| row.external_id.as_str())
                .collect::<Vec<_>>(),
            vec!["20", "30"]
        );
        assert_eq!(
            archive_rows[0].reply_to_snippet.as_deref(),
            Some("Original reply target")
        );
    }

    #[tokio::test]
    async fn export_fixture_rejects_null_published_at_before_loader_parity() {
        let pool = export_pool().await;
        seed_export_source(&pool).await;

        let result = sqlx::query(
            "INSERT INTO items (
                source_id, external_id, author, published_at, content_zstd, content_kind, has_media
             ) VALUES (1, 'null-date', 'Ada', NULL, ?, 'text_only', 0)",
        )
        .bind(compress_text("Null date").expect("compress null date"))
        .execute(&pool)
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn notebooklm_export_wrapper_matches_items_path_for_missing_stale_and_failed_states() {
        for status in [None, Some("stale"), Some("failed")] {
            let pool = export_pool().await;
            seed_notebooklm_export_parity_fixture(&pool).await;
            if let Some(status) = status {
                seed_archive_state(
                    &pool,
                    status,
                    crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                )
                .await;
            }

            let direct = load_export_messages_from_items_path(
                &pool,
                1,
                Some(50),
                Some(125),
                ExportHistoryScope::Current,
            )
            .await
            .expect("load direct items path");
            let wrapped =
                load_export_messages(&pool, 1, Some(50), Some(125), ExportHistoryScope::Current)
                    .await
                    .expect("load wrapped fallback");

            assert_eq!(wrapped, direct, "unexpected fallback result for {status:?}");
        }
    }

    #[tokio::test]
    async fn notebooklm_export_wrapper_uses_archive_reply_context_after_ready_selection() {
        let pool = export_pool().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive model");

        sqlx::query("UPDATE items SET content_zstd = ? WHERE id = 1")
            .bind(compress_text("Canonical reply target should not be used").expect("compress old"))
            .execute(&pool)
            .await
            .expect("mutate canonical reply target");
        sqlx::query(
            "UPDATE archive_read_items SET content_zstd = ? WHERE source_id = 1 AND item_id = 1",
        )
        .bind(compress_text("Archive reply target wins").expect("compress archive"))
        .execute(&pool)
        .await
        .expect("mutate archive reply target");

        let messages =
            load_export_messages(&pool, 1, Some(50), Some(115), ExportHistoryScope::Current)
                .await
                .expect("load wrapped archive path");

        assert_eq!(
            messages[0].reply_to_snippet.as_deref(),
            Some("Archive reply target wins")
        );
    }

    #[tokio::test]
    async fn notebooklm_export_wrapper_does_not_fallback_after_archive_selection_fails() {
        let pool = export_pool().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive model");

        let direct = load_export_messages_from_items_path(
            &pool,
            1,
            Some(50),
            Some(115),
            ExportHistoryScope::Current,
        )
        .await
        .expect("items path remains valid");
        assert!(!direct.is_empty());

        sqlx::query(
            "UPDATE archive_read_items
             SET content_zstd = X'00'
             WHERE source_id = 1 AND item_id = 2",
        )
        .execute(&pool)
        .await
        .expect("corrupt archive row");

        let error =
            load_export_messages(&pool, 1, Some(50), Some(115), ExportHistoryScope::Current)
                .await
                .expect_err("archive decode failure is returned");

        assert_eq!(error.kind, AppErrorKind::Internal);
        assert!(!error.message.is_empty());
    }

    #[tokio::test]
    async fn corrupt_archive_reply_target_outside_period_fails_archive_loader() {
        let pool = export_pool().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive model");

        sqlx::query(
            "UPDATE archive_read_items
             SET content_zstd = X'00'
             WHERE source_id = 1 AND item_id = 1",
        )
        .execute(&pool)
        .await
        .expect("corrupt archive reply target");

        let error =
            load_export_messages(&pool, 1, Some(50), Some(115), ExportHistoryScope::Current)
                .await
                .expect_err("corrupt reply target fails archive loader");

        assert_eq!(error.kind, AppErrorKind::Internal);
        assert!(!error.message.is_empty());
    }

    #[tokio::test]
    async fn load_export_source_uses_canonical_subtype_not_legacy_kind() {
        let pool = export_pool().await;
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, external_id, title
            )
            VALUES (?, 'telegram', 'supergroup', ?, ?)
            "#,
        )
        .bind(7_i64)
        .bind("12345")
        .bind("Forum source")
        .execute(&pool)
        .await
        .expect("insert source");

        let source = load_export_source(&pool, 7)
            .await
            .expect("load export source");

        assert_eq!(source.source_subtype, "supergroup");
    }

    #[tokio::test]
    async fn load_export_source_group_orders_members_by_title_then_id() {
        let pool = export_pool().await;
        for (id, source_type, title) in [
            (30_i64, "telegram", Some("Beta")),
            (10_i64, "telegram", Some("Alpha")),
            (20_i64, "telegram", Some("Alpha")),
            (40_i64, "telegram", None),
        ] {
            sqlx::query(
                "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
                 VALUES (?, ?, 'channel', ?, ?)",
            )
            .bind(id)
            .bind(source_type)
            .bind(format!("ext-{id}"))
            .bind(title)
            .execute(&pool)
            .await
            .expect("insert source");
        }
        sqlx::query(
            "INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
             VALUES (9, 'Notebook Group', 'telegram', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert group");
        for source_id in [30_i64, 10, 20, 40] {
            sqlx::query(
                "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
                 VALUES (9, ?, 1)",
            )
            .bind(source_id)
            .execute(&pool)
            .await
            .expect("insert member");
        }

        let group = load_export_source_group(&pool, 9)
            .await
            .expect("load group");

        assert_eq!(group.id, 9);
        assert_eq!(group.name, "Notebook Group");
        assert_eq!(group.source_type, "telegram");
        assert_eq!(
            group
                .members
                .iter()
                .map(|member| member.source_id)
                .collect::<Vec<_>>(),
            vec![40, 10, 20, 30]
        );
    }

    #[tokio::test]
    async fn load_export_source_group_keeps_dirty_member_source_type_for_skip_logic() {
        let pool = export_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (1, 'telegram', 'channel', 'telegram-1', 'Telegram'),
                    (2, 'youtube', 'video', 'youtube-1', 'YouTube')",
        )
        .execute(&pool)
        .await
        .expect("insert sources");
        sqlx::query(
            "INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
             VALUES (9, 'Dirty Group', 'telegram', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert group");
        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
             VALUES (9, 1, 1), (9, 2, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert members");

        let group = load_export_source_group(&pool, 9)
            .await
            .expect("load group");

        assert_eq!(
            group
                .members
                .iter()
                .map(|member| (member.source_id, member.source_type.as_str()))
                .collect::<Vec<_>>(),
            vec![(1, "telegram"), (2, "youtube")]
        );
    }

    #[tokio::test]
    async fn load_export_source_group_exposes_youtube_group_for_hard_validation() {
        let pool = export_pool().await;
        sqlx::query(
            "INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
             VALUES (9, 'YouTube Group', 'youtube', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert group");

        let group = load_export_source_group(&pool, 9)
            .await
            .expect("load group");

        assert_eq!(group.source_type, "youtube");
        assert!(group.members.is_empty());
    }

    #[tokio::test]
    async fn load_export_messages_adds_local_reply_context_outside_period() {
        let pool = export_pool().await;
        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("10")
        .bind("Bob")
        .bind(10_i64)
        .bind(compress_text("Original reply target").expect("compress original"))
        .bind("text_only")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert original");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media,
                reply_to_msg_id,
                reply_to_peer_kind,
                reply_to_peer_id,
                reply_to_top_id,
                reaction_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("20")
        .bind("Ada")
        .bind(100_i64)
        .bind(compress_text("Reply message").expect("compress reply"))
        .bind("text_only")
        .bind(0_i64)
        .bind(10_i64)
        .bind("channel")
        .bind("42")
        .bind(10_i64)
        .bind(3_i64)
        .execute(&pool)
        .await
        .expect("insert reply");

        let messages = load_export_messages(&pool, 1, Some(50), None, ExportHistoryScope::Current)
            .await
            .expect("load export messages");

        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        assert_eq!(message.external_id, "20");
        assert_eq!(message.reply_to_msg_id, Some(10));
        assert_eq!(message.reply_to_author.as_deref(), Some("Bob"));
        assert_eq!(
            message.reply_to_snippet.as_deref(),
            Some("Original reply target")
        );
        assert_eq!(message.reply_to_peer_kind.as_deref(), Some("channel"));
        assert_eq!(message.reply_to_peer_id.as_deref(), Some("42"));
        assert_eq!(message.reply_to_top_id, Some(10));
        assert_eq!(message.reaction_count, Some(3));
    }

    #[tokio::test]
    async fn load_export_messages_attaches_topic_metadata_for_reply_and_root_messages() {
        let pool = export_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
        )
        .execute(&pool)
        .await
        .expect("seed source");

        sqlx::query(
            r#"
            INSERT INTO telegram_forum_topics (
                source_id,
                topic_id,
                top_message_id,
                title,
                is_deleted
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind(200_i64)
        .bind(700_i64)
        .bind("Roadmap")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert forum topic");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media,
                reply_to_top_id
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("701")
        .bind("Ada")
        .bind(100_i64)
        .bind(compress_text("Reply in topic").expect("compress reply"))
        .bind("text_only")
        .bind(0_i64)
        .bind(200_i64)
        .execute(&pool)
        .await
        .expect("insert topic reply");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media,
                reply_to_msg_id
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("702")
        .bind("Edsger")
        .bind(101_i64)
        .bind(compress_text("Top-level topic message").expect("compress topic fallback"))
        .bind("text_only")
        .bind(0_i64)
        .bind(200_i64)
        .execute(&pool)
        .await
        .expect("insert topic reply header fallback");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("not-numeric-root")
        .bind("Bob")
        .bind(102_i64)
        .bind(compress_text("Root topic message").expect("compress root"))
        .bind("text_only")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert topic root");
        let root_item_id: i64 =
            sqlx::query_scalar("SELECT id FROM items WHERE external_id = 'not-numeric-root'")
                .fetch_one(&pool)
                .await
                .expect("load root item id");
        sqlx::query(
            "INSERT INTO telegram_messages (item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id)
             VALUES (?, 1, 'channel', 12345, 700)",
        )
        .bind(root_item_id)
        .execute(&pool)
        .await
        .expect("insert typed message identity");
        for external_id in ["701", "702", "not-numeric-root"] {
            let item_id: i64 = sqlx::query_scalar("SELECT id FROM items WHERE external_id = ?")
                .bind(external_id)
                .fetch_one(&pool)
                .await
                .expect("load item id for membership");
            sqlx::query(
                "INSERT INTO item_topic_memberships (
                    item_id, source_id, topic_id, match_kind, resolver_version
                 ) VALUES (?, 1, 200, 'reply_to_top_id', 1)",
            )
            .bind(item_id)
            .execute(&pool)
            .await
            .expect("insert topic membership");
        }

        let messages = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Current)
            .await
            .expect("load export messages");

        assert_eq!(messages.len(), 3);
        assert!(messages
            .iter()
            .all(|message| message.forum_topic_id == Some(200)));
        assert!(messages
            .iter()
            .all(|message| message.forum_topic_title.as_deref() == Some("Roadmap")));
        assert!(messages
            .iter()
            .all(|message| message.forum_topic_top_message_id == Some(700)));
    }

    #[tokio::test]
    async fn load_export_messages_attaches_general_topic_when_topic_header_is_missing() {
        let pool = export_pool().await;

        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
        )
        .execute(&pool)
        .await
        .expect("seed source");

        sqlx::query(
            r#"
            INSERT INTO telegram_forum_topics (
                source_id,
                topic_id,
                top_message_id,
                title,
                is_deleted
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind(1_i64)
        .bind(1_i64)
        .bind("General")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert general topic");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("999")
        .bind("Ada")
        .bind(100_i64)
        .bind(compress_text("General message").expect("compress message"))
        .bind("text_only")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert general message");
        let item_id: i64 = sqlx::query_scalar("SELECT id FROM items WHERE external_id = '999'")
            .fetch_one(&pool)
            .await
            .expect("load general item id");
        sqlx::query(
            "INSERT INTO item_topic_memberships (
                item_id, source_id, topic_id, match_kind, resolver_version
             ) VALUES (?, 1, 1, 'general_fallback', 1)",
        )
        .bind(item_id)
        .execute(&pool)
        .await
        .expect("insert general membership");

        let messages = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Current)
            .await
            .expect("load export messages");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].forum_topic_id, Some(1));
        assert_eq!(messages[0].forum_topic_title.as_deref(), Some("General"));
        assert_eq!(messages[0].forum_topic_top_message_id, Some(1));
    }

    #[tokio::test]
    async fn load_export_messages_does_not_root_match_non_numeric_external_ids() {
        let pool = export_pool().await;

        sqlx::query(
            r#"
            INSERT INTO telegram_forum_topics (
                source_id,
                topic_id,
                top_message_id,
                title,
                is_deleted
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind(200_i64)
        .bind(700_i64)
        .bind("Roadmap")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert forum topic");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("700a")
        .bind("Ada")
        .bind(100_i64)
        .bind(compress_text("Looks numeric but is not").expect("compress message"))
        .bind("text_only")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert message");

        let messages = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Current)
            .await
            .expect("load export messages");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].forum_topic_id, None);
        assert_eq!(messages[0].forum_topic_title, None);
        assert_eq!(messages[0].forum_topic_top_message_id, None);
    }

    #[tokio::test]
    async fn load_export_messages_reads_materialized_topic_memberships() {
        let pool = export_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
        )
        .execute(&pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO telegram_forum_topics (
                source_id, topic_id, top_message_id, title, is_deleted
             ) VALUES (1, 200, 700, 'Roadmap', 0)",
        )
        .execute(&pool)
        .await
        .expect("seed topic");
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, author, published_at, content_zstd,
                content_kind, has_media
             ) VALUES (1, 1, '701', 'Ada', 100, ?, 'text_only', 0)",
        )
        .bind(compress_text("Reply in topic").expect("compress"))
        .execute(&pool)
        .await
        .expect("seed item");
        sqlx::query(
            "INSERT INTO item_topic_memberships (
                item_id, source_id, topic_id, match_kind, resolver_version
             ) VALUES (1, 1, 200, 'reply_to_top_id', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed membership");

        let messages = load_export_messages(&pool, 1, None, None, ExportHistoryScope::Current)
            .await
            .expect("load export messages");

        assert_eq!(messages[0].forum_topic_id, Some(200));
        assert_eq!(messages[0].forum_topic_title.as_deref(), Some("Roadmap"));
    }
}
