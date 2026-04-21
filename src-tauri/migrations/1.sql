-- Database initialization for Extractum MVP

-- 1. Sources table (Telegram channels, etc.)
CREATE TABLE IF NOT EXISTS sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_type TEXT NOT NULL,
    external_id TEXT NOT NULL,
    title TEXT,
    metadata_zstd BLOB,
    last_sync_state INTEGER, -- message_id of the last synced message
    is_active BOOLEAN DEFAULT 1,
    is_member BOOLEAN DEFAULT 0, -- whether the user is subscribed to this channel
    created_at INTEGER NOT NULL -- Unix Timestamp, UTC
);

-- Ensure uniqueness for sources
CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_ext ON sources(source_type, external_id);

-- 2. Items table (Telegram messages)
CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    external_id TEXT NOT NULL,
    author TEXT,
    published_at INTEGER NOT NULL, -- Unix Timestamp, UTC
    ingested_at INTEGER NOT NULL,  -- Unix Timestamp, UTC
    content_zstd BLOB,
    raw_data_zstd BLOB,
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE
);

-- Unique index for messages per source
CREATE UNIQUE INDEX IF NOT EXISTS idx_items_ext ON items(source_id, external_id);

-- Index for browsing/filtering by date
CREATE INDEX IF NOT EXISTS idx_items_source_date ON items(source_id, published_at DESC);

-- Index for author search
CREATE INDEX IF NOT EXISTS idx_items_author ON items(author);

-- 3. App Settings table
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT
);
