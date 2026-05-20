CREATE TABLE IF NOT EXISTS archive_read_model_state (
    source_id INTEGER PRIMARY KEY,
    model_version INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'never_built',
    built_at INTEGER,
    item_count INTEGER NOT NULL DEFAULT 0,
    row_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
    CHECK (status IN ('never_built', 'building', 'ready', 'stale', 'failed')),
    CHECK (item_count >= 0),
    CHECK (row_count >= 0)
);

CREATE TABLE IF NOT EXISTS archive_read_items (
    source_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    ref TEXT NOT NULL,
    external_id TEXT NOT NULL,
    item_kind TEXT NOT NULL,
    author TEXT,
    published_at INTEGER NOT NULL,
    content_kind TEXT NOT NULL,
    has_media INTEGER NOT NULL DEFAULT 0,
    media_kind TEXT,
    content_zstd BLOB,
    media_metadata_zstd BLOB,
    has_raw_data INTEGER NOT NULL DEFAULT 0,
    forum_topic_id INTEGER,
    forum_topic_title TEXT,
    forum_topic_top_message_id INTEGER,
    reply_to_msg_id INTEGER,
    reply_to_peer_kind TEXT,
    reply_to_peer_id TEXT,
    reply_to_top_id INTEGER,
    reaction_count INTEGER,
    model_version INTEGER NOT NULL,
    built_at INTEGER NOT NULL,
    PRIMARY KEY(source_id, item_id),
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
    FOREIGN KEY(item_id) REFERENCES items(id) ON DELETE CASCADE,
    CHECK (has_media IN (0, 1)),
    CHECK (has_raw_data IN (0, 1))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_archive_read_items_ref
    ON archive_read_items(ref);

CREATE INDEX IF NOT EXISTS idx_archive_read_items_source_published
    ON archive_read_items(source_id, published_at DESC, item_id DESC);

CREATE INDEX IF NOT EXISTS idx_archive_read_items_source_topic_published
    ON archive_read_items(source_id, forum_topic_id, published_at DESC, item_id DESC);
