ALTER TABLE telegram_messages RENAME TO telegram_messages__pre_migrated_history_opt_in;

CREATE TABLE telegram_messages (
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

INSERT INTO telegram_messages (
    item_id,
    source_id,
    history_peer_kind,
    history_peer_id,
    telegram_message_id,
    migration_domain,
    is_migrated_history,
    reply_to_msg_id,
    reply_to_peer_kind,
    reply_to_peer_id,
    reply_to_top_id,
    reaction_count,
    created_at,
    updated_at
)
SELECT
    item_id,
    source_id,
    history_peer_kind,
    history_peer_id,
    telegram_message_id,
    migration_domain,
    is_migrated_history,
    reply_to_msg_id,
    reply_to_peer_kind,
    reply_to_peer_id,
    reply_to_top_id,
    reaction_count,
    created_at,
    updated_at
FROM telegram_messages__pre_migrated_history_opt_in;

DROP TABLE telegram_messages__pre_migrated_history_opt_in;

CREATE INDEX idx_telegram_messages_source_message
    ON telegram_messages(source_id, telegram_message_id);

CREATE INDEX idx_telegram_messages_source_reply_top
    ON telegram_messages(source_id, reply_to_top_id);

CREATE UNIQUE INDEX ux_telegram_messages_native_identity
    ON telegram_messages (
        source_id,
        history_peer_kind,
        history_peer_id,
        telegram_message_id
    );

CREATE TABLE IF NOT EXISTS telegram_migrated_history_capabilities (
    source_id INTEGER PRIMARY KEY REFERENCES telegram_sources(source_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    unavailable_reason TEXT,
    migrated_from_chat_id INTEGER,
    detected_at INTEGER,
    refreshed_at INTEGER NOT NULL,
    CHECK (status IN ('none', 'available', 'unavailable')),
    CHECK (
        unavailable_reason IS NULL
        OR unavailable_reason IN (
            'not_detected',
            'missing_migrated_from_chat_id',
            'current_source_unavailable',
            'old_chat_input_unavailable',
            'revalidation_failed'
        )
    ),
    CHECK (migrated_from_chat_id IS NULL OR migrated_from_chat_id > 0),
    CHECK (status <> 'available' OR migrated_from_chat_id IS NOT NULL),
    CHECK (status <> 'unavailable' OR unavailable_reason IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_telegram_migrated_history_capabilities_status
    ON telegram_migrated_history_capabilities(status);

ALTER TABLE telegram_takeout_batches RENAME TO telegram_takeout_batches__pre_migrated_history_opt_in;

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

INSERT INTO telegram_takeout_batches (
  batch_id,
  account_id,
  source_subtype,
  resolved_peer_kind,
  resolved_peer_id,
  history_peer_kind,
  history_peer_id,
  takeout_id,
  export_dc_id,
  used_export_dc,
  fallback_used,
  history_scope,
  migrated_history_detected,
  migrated_history_imported,
  only_my_messages,
  split_count,
  selected_split_count,
  message_count_estimate,
  max_message_id,
  created_at,
  updated_at
)
SELECT
  batch_id,
  account_id,
  source_subtype,
  resolved_peer_kind,
  resolved_peer_id,
  history_peer_kind,
  history_peer_id,
  takeout_id,
  export_dc_id,
  used_export_dc,
  fallback_used,
  history_scope,
  migrated_history_detected,
  migrated_history_imported,
  only_my_messages,
  split_count,
  selected_split_count,
  message_count_estimate,
  max_message_id,
  created_at,
  updated_at
FROM telegram_takeout_batches__pre_migrated_history_opt_in;

DROP TABLE telegram_takeout_batches__pre_migrated_history_opt_in;

CREATE INDEX idx_telegram_takeout_batches_account
    ON telegram_takeout_batches(account_id);
