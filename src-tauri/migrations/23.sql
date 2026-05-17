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
    'mixed_partial'
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
