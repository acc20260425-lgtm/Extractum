#![allow(dead_code)]

use sqlx::{Executor, Sqlite};

use crate::error::{AppError, AppResult};

pub(crate) const ANALYSIS_DOCUMENTS_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS analysis_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    item_id INTEGER REFERENCES items(id) ON DELETE CASCADE,

    document_key TEXT NOT NULL,
    document_kind TEXT NOT NULL,

    source_type TEXT NOT NULL,
    source_subtype TEXT,
    external_id TEXT NOT NULL,

    author TEXT,
    published_at INTEGER NOT NULL,
    document_order INTEGER NOT NULL DEFAULT 0,

    ref TEXT NOT NULL,
    content_zstd BLOB NOT NULL,
    metadata_zstd BLOB,

    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,

    CHECK (document_kind IN (
        'telegram_message',
        'youtube_transcript',
        'youtube_comment',
        'youtube_description'
    )),
    CHECK (source_type IN ('telegram', 'youtube')),
    CHECK (
        (document_kind = 'telegram_message' AND source_type = 'telegram')
        OR
        (document_kind IN (
            'youtube_transcript',
            'youtube_comment',
            'youtube_description'
        ) AND source_type = 'youtube')
    ),
    CHECK (
        (source_type = 'telegram'
            AND COALESCE(source_subtype, '')
                IN ('channel', 'supergroup', 'group'))
        OR
        (source_type = 'youtube' AND COALESCE(source_subtype, '') = 'video')
    ),
    CHECK (
        (document_kind IN (
            'telegram_message',
            'youtube_transcript',
            'youtube_comment'
        ) AND item_id IS NOT NULL)
        OR
        (document_kind = 'youtube_description' AND item_id IS NULL)
    ),
    CHECK (
        (document_kind IN (
            'telegram_message',
            'youtube_transcript',
            'youtube_comment'
        ) AND document_key LIKE 'item:%')
        OR
        (document_kind = 'youtube_description'
            AND document_key = 'youtube:description')
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_analysis_documents_source_key
ON analysis_documents(source_id, document_key);

CREATE INDEX IF NOT EXISTS idx_analysis_documents_source_published
ON analysis_documents(source_id, published_at, document_order, id);

CREATE INDEX IF NOT EXISTS idx_analysis_documents_kind_source_published
ON analysis_documents(document_kind, source_id, published_at, document_order, id);

CREATE INDEX IF NOT EXISTS idx_analysis_documents_ref
ON analysis_documents(ref);
"#;

pub(crate) async fn create_analysis_documents_schema<'e, E>(executor: E) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::raw_sql(ANALYSIS_DOCUMENTS_SCHEMA_SQL)
        .execute(executor)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::test_support::memory_pool_with_source_items_and_topics;

    #[tokio::test]
    async fn schema_creates_analysis_documents_constraints_and_indexes() {
        let pool = memory_pool_with_source_items_and_topics().await;

        create_analysis_documents_schema(&pool)
            .await
            .expect("create analysis document schema");

        let table_exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'analysis_documents'",
        )
        .fetch_one(&pool)
        .await
        .expect("check table");
        assert_eq!(table_exists, 1);

        for index in [
            "idx_analysis_documents_source_key",
            "idx_analysis_documents_source_published",
            "idx_analysis_documents_kind_source_published",
            "idx_analysis_documents_ref",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
            )
            .bind(index)
            .fetch_one(&pool)
            .await
            .expect("check index");
            assert_eq!(exists, 1, "missing index {index}");
        }

        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 'tg1', 'Telegram', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed source");

        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at,
                ingested_at, content_kind, has_media, content_zstd
             ) VALUES (10, 1, '10', 'telegram_message', 'alice', 100, 100, 'text_only', 0, x'01')",
        )
        .execute(&pool)
        .await
        .expect("seed item");

        sqlx::query(
            "INSERT INTO analysis_documents (
                source_id, item_id, document_key, document_kind, source_type,
                source_subtype, external_id, author, published_at, document_order,
                ref, content_zstd, created_at, updated_at
             ) VALUES (
                1, 10, 'item:10', 'telegram_message', 'telegram',
                'supergroup', '10', 'alice', 100, 10,
                's1-i10', x'01', 100, 100
             )",
        )
        .execute(&pool)
        .await
        .expect("valid item-backed document");

        let invalid_synthetic = sqlx::query(
            "INSERT INTO analysis_documents (
                source_id, item_id, document_key, document_kind, source_type,
                source_subtype, external_id, published_at, document_order,
                ref, content_zstd, created_at, updated_at
             ) VALUES (
                1, 10, 'youtube:description', 'youtube_description', 'youtube',
                'video', 'description:v1', 100, -1,
                's1-i0', x'01', 100, 100
             )",
        )
        .execute(&pool)
        .await;
        assert!(invalid_synthetic.is_err());
    }
}
