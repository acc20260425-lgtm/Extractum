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

const TELEGRAM_MESSAGES_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS telegram_messages (
    item_id INTEGER PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    history_peer_kind TEXT NOT NULL,
    history_peer_id INTEGER NOT NULL,
    telegram_message_id INTEGER NOT NULL,
    migration_domain TEXT,
    is_migrated_history INTEGER NOT NULL DEFAULT 0,
    reply_to_msg_id INTEGER,
    reply_to_peer_kind TEXT,
    reply_to_peer_id INTEGER,
    reply_to_top_id INTEGER,
    reaction_count INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (history_peer_kind IN ('channel', 'chat', 'user')),
    CHECK (telegram_message_id > 0),
    CHECK (is_migrated_history IN (0, 1)),
    CHECK (migration_domain IS NULL OR migration_domain IN ('migrated_from_chat')),
    CHECK (reply_to_msg_id IS NULL OR reply_to_msg_id > 0),
    CHECK (
        reply_to_peer_kind IS NULL
        OR reply_to_peer_kind IN ('channel', 'chat', 'user')
    ),
    CHECK (reply_to_peer_id IS NULL OR reply_to_peer_id > 0),
    CHECK (reply_to_top_id IS NULL OR reply_to_top_id > 0),
    CHECK (reaction_count IS NULL OR reaction_count >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_telegram_messages_native_identity
    ON telegram_messages (
        source_id,
        history_peer_kind,
        history_peer_id,
        telegram_message_id
    );

CREATE INDEX IF NOT EXISTS idx_telegram_messages_source_message
    ON telegram_messages(source_id, telegram_message_id);

CREATE INDEX IF NOT EXISTS idx_telegram_messages_source_reply_top
    ON telegram_messages(source_id, reply_to_top_id);
"#;

const INGEST_PROVENANCE_SCHEMA_SQL: &str = r#"
CREATE TABLE ingest_batches (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,

  provider TEXT NOT NULL,
  ingest_kind TEXT NOT NULL,

  status TEXT NOT NULL,
  completeness TEXT NOT NULL DEFAULT 'unknown',

  started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  finished_at TEXT,

  item_inserted_count INTEGER NOT NULL DEFAULT 0,
  item_observed_count INTEGER NOT NULL DEFAULT 0,
  item_duplicate_count INTEGER NOT NULL DEFAULT 0,
  item_skipped_count INTEGER NOT NULL DEFAULT 0,
  warning_count INTEGER NOT NULL DEFAULT 0,

  terminal_error TEXT,

  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CHECK (provider IN ('telegram', 'youtube')),
  CHECK (ingest_kind IN (
    'takeout',
    'sync',
    'youtube_metadata',
    'youtube_transcript',
    'youtube_comments',
    'youtube_playlist'
  )),
  CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),
  CHECK (completeness IN ('unknown', 'complete', 'partial')),
  CHECK (
    (status = 'running' AND finished_at IS NULL)
    OR
    (status IN ('completed', 'failed', 'cancelled') AND finished_at IS NOT NULL)
  ),
  CHECK (item_inserted_count >= 0),
  CHECK (item_observed_count >= 0),
  CHECK (item_duplicate_count >= 0),
  CHECK (item_skipped_count >= 0),
  CHECK (warning_count >= 0),
  CHECK (
    item_observed_count >=
    item_inserted_count + item_duplicate_count + item_skipped_count
  )
);

CREATE TABLE telegram_takeout_batches (
  batch_id INTEGER PRIMARY KEY REFERENCES ingest_batches(id) ON DELETE CASCADE,

  account_id INTEGER NOT NULL,
  source_subtype TEXT NOT NULL,

  resolved_peer_kind TEXT,
  resolved_peer_id INTEGER,
  history_peer_kind TEXT,
  history_peer_id INTEGER,

  takeout_id INTEGER,
  export_dc_id INTEGER,
  used_export_dc INTEGER NOT NULL DEFAULT 0,
  fallback_used INTEGER NOT NULL DEFAULT 0,

  history_scope TEXT NOT NULL DEFAULT 'unknown',

  migrated_history_detected INTEGER NOT NULL DEFAULT 0,
  migrated_history_imported INTEGER NOT NULL DEFAULT 0,
  only_my_messages INTEGER NOT NULL DEFAULT 0,

  split_count INTEGER,
  selected_split_count INTEGER,
  message_count_estimate INTEGER,
  max_message_id INTEGER,

  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CHECK (source_subtype IN ('channel', 'supergroup', 'group')),
  CHECK (resolved_peer_kind IS NULL OR resolved_peer_kind IN ('channel', 'chat')),
  CHECK (history_peer_kind IS NULL OR history_peer_kind IN ('channel', 'chat', 'user')),
  CHECK (history_scope IN (
    'unknown',
    'current_history',
    'current_history_with_migrated_deferred',
    'partial_private_history',
    'mixed_partial',
    'migrated_small_group_history'
  )),
  CHECK (used_export_dc IN (0, 1)),
  CHECK (fallback_used IN (0, 1)),
  CHECK (migrated_history_detected IN (0, 1)),
  CHECK (migrated_history_imported IN (0, 1)),
  CHECK (only_my_messages IN (0, 1))
);

CREATE TABLE ingest_item_observations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  batch_id INTEGER NOT NULL REFERENCES ingest_batches(id) ON DELETE CASCADE,
  source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,

  item_id INTEGER REFERENCES items(id) ON DELETE SET NULL,

  provider_item_kind TEXT NOT NULL,
  provider_identity_kind TEXT NOT NULL,
  provider_identity TEXT NOT NULL,
  provider_identity_version INTEGER NOT NULL DEFAULT 1,

  outcome TEXT NOT NULL,
  reason_code TEXT,

  observed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CHECK (provider_item_kind IN ('telegram_message')),
  CHECK (provider_identity_version >= 1),
  CHECK (outcome IN ('inserted', 'duplicate_observed', 'skipped', 'failed'))
);

CREATE TABLE ingest_batch_warnings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  batch_id INTEGER NOT NULL REFERENCES ingest_batches(id) ON DELETE CASCADE,

  code TEXT NOT NULL,
  message TEXT NOT NULL,

  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ingest_batches_source_started
ON ingest_batches(source_id, started_at DESC);

CREATE INDEX idx_ingest_batches_status
ON ingest_batches(status);

CREATE INDEX idx_telegram_takeout_batches_account
ON telegram_takeout_batches(account_id);

CREATE INDEX idx_ingest_item_observations_batch
ON ingest_item_observations(batch_id);

CREATE INDEX idx_ingest_item_observations_item
ON ingest_item_observations(item_id)
WHERE item_id IS NOT NULL;

CREATE INDEX idx_ingest_item_observations_identity
ON ingest_item_observations(source_id, provider_identity_kind, provider_identity);

CREATE INDEX idx_ingest_item_observations_batch_outcome
ON ingest_item_observations(batch_id, outcome);

CREATE INDEX idx_ingest_batch_warnings_batch
ON ingest_batch_warnings(batch_id);
"#;

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
    create_migrated_history_capability_tables(pool).await;
}

pub(crate) async fn create_migrated_history_capability_tables(pool: &sqlx::SqlitePool) {
    crate::takeout_import::migrated_history::create_migrated_history_capability_schema(pool)
        .await
        .expect("create migrated history capability schema");
}

pub(crate) async fn create_youtube_typed_source_tables(pool: &sqlx::SqlitePool) {
    crate::youtube::source_metadata::create_youtube_typed_source_tables(pool)
        .await
        .expect("create youtube typed source metadata tables");
}

pub(crate) async fn create_telegram_messages_table(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(TELEGRAM_MESSAGES_SCHEMA_SQL)
        .execute(pool)
        .await
        .expect("create telegram_messages");
}

pub(crate) async fn create_topic_membership_tables(pool: &sqlx::SqlitePool) {
    let mut conn = pool.acquire().await.expect("acquire sqlite connection");
    crate::topic_memberships::create_topic_membership_schema(&mut conn)
        .await
        .expect("create topic membership schema");
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

pub(crate) async fn create_ingest_provenance_tables(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(INGEST_PROVENANCE_SCHEMA_SQL)
        .execute(pool)
        .await
        .expect("create ingest provenance schema");
}

pub(crate) async fn create_analysis_documents_table(pool: &sqlx::SqlitePool) {
    crate::analysis_documents::create_analysis_documents_schema(pool)
        .await
        .expect("create analysis documents schema");
}

pub(crate) async fn create_archive_read_model_tables(pool: &sqlx::SqlitePool) {
    crate::archive_read_model::create_archive_read_model_schema(pool)
        .await
        .expect("create archive read model schema");
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
    create_telegram_messages_table(&pool).await;
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
    create_topic_membership_tables(&pool).await;
    create_archive_read_model_tables(&pool).await;
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
        create_analysis_documents_table, create_archive_read_model_tables,
        create_canonical_telegram_identity_index, create_ingest_provenance_tables,
        memory_pool_with_source_items_and_topics,
    };

    #[tokio::test]
    async fn source_fixture_creates_expected_tables() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_canonical_telegram_identity_index(&pool).await;
        create_ingest_provenance_tables(&pool).await;
        create_analysis_documents_table(&pool).await;
        create_archive_read_model_tables(&pool).await;

        for table in [
            "app_settings",
            "sources",
            "source_identity_repair_notes",
            "telegram_sources",
            "items",
            "telegram_forum_topics",
            "item_topic_memberships",
            "telegram_topic_resolution_state",
            "ingest_batches",
            "telegram_takeout_batches",
            "ingest_item_observations",
            "ingest_batch_warnings",
            "analysis_documents",
            "archive_read_model_state",
            "archive_read_items",
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
