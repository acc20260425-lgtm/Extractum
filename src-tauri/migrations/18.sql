UPDATE sources
SET source_subtype = telegram_source_kind
WHERE source_type = 'telegram'
  AND source_subtype IS NULL
  AND telegram_source_kind IN ('channel', 'supergroup', 'group');

UPDATE sources
SET source_subtype = telegram_source_kind
WHERE source_type = 'telegram'
  AND source_subtype NOT IN ('channel', 'supergroup', 'group')
  AND telegram_source_kind IN ('channel', 'supergroup', 'group');

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
