pub(crate) async fn memory_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query("CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT)")
        .execute(&pool)
        .await
        .expect("create app_settings");
    pool
}

pub(crate) async fn memory_pool_with_sources() -> sqlx::SqlitePool {
    let pool = memory_pool().await;
    sqlx::query(
        r#"
        CREATE TABLE sources (
            id INTEGER PRIMARY KEY,
            source_type TEXT NOT NULL,
            source_subtype TEXT,
            account_id INTEGER,
            external_id TEXT NOT NULL,
            title TEXT,
            metadata_zstd BLOB,
            last_sync_state INTEGER,
            last_synced_at INTEGER,
            is_active INTEGER NOT NULL DEFAULT 1,
            is_member INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create sources");
    create_source_identity_tables(&pool).await;
    pool
}

pub(crate) async fn create_source_identity_tables(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(
        r#"
        CREATE TABLE IF NOT EXISTS source_identity_repair_notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id INTEGER NOT NULL,
            issue_code TEXT NOT NULL,
            detail TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
            UNIQUE(source_id, issue_code)
        );

        CREATE TABLE IF NOT EXISTS telegram_sources (
            source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
            account_id INTEGER NOT NULL,
            source_subtype TEXT NOT NULL,
            peer_kind TEXT NOT NULL,
            peer_id INTEGER NOT NULL,
            resolution_strategy TEXT NOT NULL,
            username TEXT,
            access_hash INTEGER,
            avatar_cache_key TEXT,
            identity_refreshed_at INTEGER,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            CHECK (source_subtype IN ('channel', 'supergroup', 'group')),
            CHECK (peer_kind IN ('channel', 'chat')),
            CHECK (
                (source_subtype IN ('channel', 'supergroup') AND peer_kind = 'channel')
                OR
                (source_subtype = 'group' AND peer_kind = 'chat')
            ),
            CHECK (resolution_strategy IN ('username', 'dialog', 'legacy_metadata', 'unknown'))
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_telegram_sources_account_peer
            ON telegram_sources(account_id, peer_kind, peer_id);

        CREATE INDEX IF NOT EXISTS idx_telegram_sources_account_subtype
            ON telegram_sources(account_id, source_subtype);

        CREATE INDEX IF NOT EXISTS idx_telegram_sources_account_username
            ON telegram_sources(account_id, username)
            WHERE username IS NOT NULL;
        "#,
    )
    .execute(pool)
    .await
    .expect("create source identity bridge schema");
}

pub(crate) async fn create_youtube_typed_source_tables(pool: &sqlx::SqlitePool) {
    crate::youtube::source_metadata::create_youtube_typed_source_tables(pool)
        .await
        .expect("create youtube typed source metadata tables");
}

pub(crate) async fn create_canonical_telegram_identity_index(pool: &sqlx::SqlitePool) {
    sqlx::query(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_telegram_identity
            ON sources(account_id, source_type, source_subtype, external_id)
            WHERE source_type = 'telegram'
        "#,
    )
    .execute(pool)
    .await
    .expect("create canonical telegram identity index");
}

pub(crate) async fn memory_pool_with_source_items_and_topics() -> sqlx::SqlitePool {
    let pool = memory_pool_with_sources().await;
    sqlx::query(
        r#"
        CREATE TABLE items (
            id INTEGER PRIMARY KEY,
            source_id INTEGER NOT NULL,
            external_id TEXT NOT NULL,
            item_kind TEXT NOT NULL DEFAULT 'telegram_message',
            author TEXT,
            published_at INTEGER NOT NULL,
            ingested_at INTEGER NOT NULL,
            content_zstd BLOB,
            raw_data_zstd BLOB,
            content_kind TEXT NOT NULL,
            has_media INTEGER NOT NULL DEFAULT 0,
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
    create_item_identity_indexes(&pool).await;
    sqlx::query(
        r#"
        CREATE TABLE telegram_forum_topics (
            id INTEGER PRIMARY KEY,
            source_id INTEGER NOT NULL,
            topic_id INTEGER NOT NULL,
            top_message_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            icon_color INTEGER,
            icon_emoji_id INTEGER,
            is_closed INTEGER NOT NULL DEFAULT 0,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            is_hidden INTEGER NOT NULL DEFAULT 0,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER,
            last_seen_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
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
    .expect("create telegram_forum_topics unique index");
    pool
}

pub(crate) async fn create_item_identity_indexes(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS ux_items_non_telegram_external
            ON items(source_id, external_id)
            WHERE item_kind <> 'telegram_message';

        CREATE INDEX IF NOT EXISTS idx_items_source_external
            ON items(source_id, external_id);
        "#,
    )
    .execute(pool)
    .await
    .expect("create item identity indexes");
}

#[cfg(test)]
mod tests {
    use super::{
        create_canonical_telegram_identity_index, memory_pool_with_source_items_and_topics,
    };

    #[tokio::test]
    async fn source_fixture_creates_expected_tables() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_canonical_telegram_identity_index(&pool).await;

        for table in [
            "app_settings",
            "sources",
            "source_identity_repair_notes",
            "telegram_sources",
            "items",
            "telegram_forum_topics",
        ] {
            sqlx::query("SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?")
                .bind(table)
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| panic!("expected {table} table"));
        }

        sqlx::query(
            "SELECT name FROM sqlite_master WHERE type = 'index' AND name = 'idx_sources_unique_telegram_identity'",
        )
        .fetch_one(&pool)
        .await
        .expect("expected canonical telegram identity index helper to create index");
    }
}
