-- Current schema baseline created from the legacy migration path.
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    label TEXT NOT NULL,
    api_id INTEGER NOT NULL,
    api_hash TEXT NOT NULL,
    phone TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE analysis_chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
);

CREATE TABLE analysis_documents (
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

CREATE TABLE analysis_prompt_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    template_kind TEXT NOT NULL,
    body TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_builtin BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE analysis_run_messages (
    run_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    source_id INTEGER NOT NULL,
    external_id TEXT NOT NULL,
    author TEXT,
    published_at INTEGER NOT NULL,
    ref TEXT NOT NULL,
    content_zstd BLOB NOT NULL,
    item_kind TEXT,
    source_type TEXT,
    source_subtype TEXT,
    metadata_zstd BLOB,
    PRIMARY KEY (run_id, ref),
    FOREIGN KEY (run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
);

CREATE TABLE analysis_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_type TEXT NOT NULL,
    scope_type TEXT NOT NULL,
    source_id INTEGER,
    period_from INTEGER NOT NULL,
    period_to INTEGER NOT NULL,
    output_language TEXT NOT NULL,
    prompt_template_id INTEGER,
    prompt_template_version INTEGER NOT NULL,
    provider_profile TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    status TEXT NOT NULL,
    result_markdown TEXT,
    trace_data_zstd BLOB,
    error TEXT,
    created_at INTEGER NOT NULL,
    completed_at INTEGER,
    source_group_id INTEGER,
    scope_label_snapshot TEXT,
    youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description'
    CHECK (youtube_corpus_mode IN (
        'transcript_only',
        'transcript_description',
        'transcript_description_comments'
    )),
    snapshot_captured_at TEXT,
    snapshot_error TEXT
);

CREATE TABLE analysis_source_group_members (
    group_id INTEGER NOT NULL,
    source_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (group_id, source_id),
    FOREIGN KEY (group_id) REFERENCES analysis_source_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE
);

CREATE TABLE analysis_source_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    source_type TEXT NOT NULL DEFAULT 'telegram'
);

CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT
);

CREATE TABLE archive_read_items (
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

CREATE TABLE archive_read_model_state (
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

CREATE TABLE ingest_batch_warnings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  batch_id INTEGER NOT NULL REFERENCES ingest_batches(id) ON DELETE CASCADE,

  code TEXT NOT NULL,
  message TEXT NOT NULL,

  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

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

CREATE TABLE item_topic_memberships (
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

CREATE TABLE items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    external_id TEXT NOT NULL,
    author TEXT,
    published_at INTEGER NOT NULL, -- Unix Timestamp, UTC
    ingested_at INTEGER NOT NULL,  -- Unix Timestamp, UTC
    content_zstd BLOB,
    raw_data_zstd BLOB,
    content_kind TEXT NOT NULL DEFAULT 'text_only',
    has_media BOOLEAN NOT NULL DEFAULT 0,
    media_kind TEXT,
    media_metadata_zstd BLOB,
    reply_to_msg_id INTEGER,
    reply_to_peer_kind TEXT,
    reply_to_peer_id TEXT,
    reply_to_top_id INTEGER,
    reaction_count INTEGER,
    item_kind TEXT NOT NULL DEFAULT 'telegram_message',
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE
);

CREATE TABLE source_identity_repair_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    issue_code TEXT NOT NULL,
    detail TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
    UNIQUE(source_id, issue_code)
);

CREATE TABLE "sources" (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_type TEXT NOT NULL,
    source_subtype TEXT,
    external_id TEXT NOT NULL,
    title TEXT,
    metadata_zstd BLOB,
    last_sync_state INTEGER,
    is_active BOOLEAN DEFAULT 1,
    is_member BOOLEAN DEFAULT 0,
    created_at INTEGER NOT NULL,
    account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    last_synced_at INTEGER,
    CHECK (
        source_type <> 'telegram'
        OR (
            account_id IS NOT NULL
            AND source_subtype IS NOT NULL
            AND source_subtype IN ('channel', 'supergroup', 'group')
        )
    ),
    CHECK (
        source_type <> 'youtube'
        OR (
            account_id IS NULL
            AND source_subtype IS NOT NULL
            AND source_subtype IN ('video', 'playlist')
        )
    )
);

CREATE TABLE telegram_forum_topics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    topic_id INTEGER NOT NULL,
    top_message_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    icon_color INTEGER,
    icon_emoji_id INTEGER,
    is_closed BOOLEAN NOT NULL DEFAULT 0,
    is_pinned BOOLEAN NOT NULL DEFAULT 0,
    is_hidden BOOLEAN NOT NULL DEFAULT 0,
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
    sort_order INTEGER,
    last_seen_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE
);

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
    CHECK (reply_to_msg_id IS NULL OR reply_to_msg_id > 0),
    CHECK (
        reply_to_peer_kind IS NULL
        OR reply_to_peer_kind IN ('channel', 'chat', 'user')
    ),
    CHECK (reply_to_peer_id IS NULL OR reply_to_peer_id > 0),
    CHECK (reply_to_top_id IS NULL OR reply_to_top_id > 0),
    CHECK (reaction_count IS NULL OR reaction_count >= 0)
);

CREATE TABLE telegram_sources (
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

CREATE TABLE telegram_topic_resolution_state (
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

CREATE TABLE youtube_playlist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    video_source_id INTEGER REFERENCES sources(id) ON DELETE SET NULL,
    video_id TEXT NOT NULL,
    position INTEGER,
    title_snapshot TEXT,
    url TEXT,
    thumbnail_url TEXT,
    availability_status TEXT NOT NULL,
    is_removed_from_playlist INTEGER NOT NULL DEFAULT 0,
    last_seen_at INTEGER,
    metadata_zstd BLOB,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (availability_status IN (
        'available',
        'upcoming',
        'live_now',
        'live_ended_transcript_pending',
        'no_captions',
        'private_or_auth_required',
        'members_only',
        'age_restricted',
        'geo_blocked',
        'deleted',
        'removed_from_playlist',
        'unavailable_unknown'
    )),
    UNIQUE(playlist_source_id, video_id)
);

CREATE TABLE youtube_playlist_sources (
    source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    playlist_id TEXT NOT NULL,
    canonical_url TEXT NOT NULL,
    title TEXT,
    channel_title TEXT,
    channel_id TEXT,
    channel_handle TEXT,
    channel_url TEXT,
    thumbnail_url TEXT,
    video_count INTEGER,
    availability_status TEXT NOT NULL,
    raw_metadata_version INTEGER,
    raw_metadata_zstd BLOB,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (availability_status IN (
        'available',
        'upcoming',
        'live_now',
        'live_ended_transcript_pending',
        'no_captions',
        'private_or_auth_required',
        'members_only',
        'age_restricted',
        'geo_blocked',
        'deleted',
        'removed_from_playlist',
        'unavailable_unknown'
    ))
);

CREATE TABLE youtube_transcript_segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    segment_index INTEGER NOT NULL,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER,
    text TEXT NOT NULL,
    chapter_index INTEGER,
    caption_language TEXT,
    caption_track_kind TEXT,
    is_auto_generated INTEGER NOT NULL DEFAULT 0,
    metadata_zstd BLOB,
    UNIQUE(item_id, segment_index)
);

CREATE TABLE youtube_video_sources (
    source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    video_id TEXT NOT NULL,
    canonical_url TEXT NOT NULL,
    title TEXT,
    channel_title TEXT,
    channel_id TEXT,
    channel_handle TEXT,
    channel_url TEXT,
    author_display TEXT,
    published_at TEXT,
    duration_seconds INTEGER,
    description TEXT,
    thumbnail_url TEXT,
    tags_json TEXT NOT NULL DEFAULT '[]',
    chapters_json TEXT NOT NULL DEFAULT '[]',
    view_count INTEGER,
    like_count INTEGER,
    comment_count INTEGER,
    category TEXT,
    video_form TEXT NOT NULL,
    availability_status TEXT NOT NULL,
    caption_language_override TEXT,
    raw_metadata_version INTEGER,
    raw_metadata_zstd BLOB,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (video_form IN ('regular', 'short', 'live')),
    CHECK (availability_status IN (
        'available',
        'upcoming',
        'live_now',
        'live_ended_transcript_pending',
        'no_captions',
        'private_or_auth_required',
        'members_only',
        'age_restricted',
        'geo_blocked',
        'deleted',
        'removed_from_playlist',
        'unavailable_unknown'
    ))
);

CREATE INDEX idx_analysis_chat_messages_run_created
    ON analysis_chat_messages(run_id, created_at ASC, id ASC);

CREATE INDEX idx_analysis_documents_kind_source_published
    ON analysis_documents(document_kind, source_id, published_at, document_order, id);

CREATE INDEX idx_analysis_documents_ref
    ON analysis_documents(ref);

CREATE UNIQUE INDEX idx_analysis_documents_source_key
    ON analysis_documents(source_id, document_key);

CREATE INDEX idx_analysis_documents_source_published
    ON analysis_documents(source_id, published_at, document_order, id);

CREATE INDEX idx_analysis_prompt_templates_kind_name
    ON analysis_prompt_templates(template_kind, name);

CREATE INDEX idx_analysis_run_messages_run_published
    ON analysis_run_messages(run_id, published_at ASC, ref ASC);

CREATE INDEX idx_analysis_run_messages_run_source
    ON analysis_run_messages(run_id, source_id);

CREATE INDEX idx_analysis_runs_source_created
    ON analysis_runs(source_id, created_at DESC);

CREATE INDEX idx_analysis_runs_source_group_created
    ON analysis_runs(source_group_id, created_at DESC);

CREATE INDEX idx_analysis_runs_status_created
    ON analysis_runs(status, created_at DESC);

CREATE INDEX idx_analysis_source_group_members_source_id
    ON analysis_source_group_members(source_id);

CREATE INDEX idx_analysis_source_groups_source_type
    ON analysis_source_groups(source_type);

CREATE INDEX idx_analysis_source_groups_updated_at
    ON analysis_source_groups(updated_at DESC);

CREATE UNIQUE INDEX idx_archive_read_items_ref
    ON archive_read_items(ref);

CREATE INDEX idx_archive_read_items_source_published
    ON archive_read_items(source_id, published_at DESC, item_id DESC);

CREATE INDEX idx_archive_read_items_source_topic_published
    ON archive_read_items(source_id, forum_topic_id, published_at DESC, item_id DESC);

CREATE INDEX idx_ingest_batch_warnings_batch
    ON ingest_batch_warnings(batch_id);

CREATE INDEX idx_ingest_batches_source_started
    ON ingest_batches(source_id, started_at DESC);

CREATE INDEX idx_ingest_batches_status
    ON ingest_batches(status);

CREATE INDEX idx_ingest_item_observations_batch
    ON ingest_item_observations(batch_id);

CREATE INDEX idx_ingest_item_observations_batch_outcome
    ON ingest_item_observations(batch_id, outcome);

CREATE INDEX idx_ingest_item_observations_identity
    ON ingest_item_observations(source_id, provider_identity_kind, provider_identity);

CREATE INDEX idx_ingest_item_observations_item
    ON ingest_item_observations(item_id)
    WHERE item_id IS NOT NULL;

CREATE INDEX idx_item_topic_memberships_source_item
    ON item_topic_memberships(source_id, item_id);

CREATE INDEX idx_item_topic_memberships_source_topic
    ON item_topic_memberships(source_id, topic_id);

CREATE INDEX idx_items_author ON items(author);

CREATE INDEX idx_items_source_date ON items(source_id, published_at DESC);

CREATE INDEX idx_items_source_external
    ON items(source_id, external_id);

CREATE INDEX idx_items_source_kind_published
    ON items(source_id, item_kind, published_at DESC);

CREATE INDEX idx_items_source_reply_to_top
    ON items(source_id, reply_to_top_id);

CREATE UNIQUE INDEX idx_sources_unique_telegram_identity
    ON sources(account_id, source_type, source_subtype, external_id)
    WHERE source_type = 'telegram';

CREATE UNIQUE INDEX idx_sources_unique_youtube_playlist
    ON sources(source_type, source_subtype, external_id)
    WHERE source_type = 'youtube' AND source_subtype = 'playlist';

CREATE UNIQUE INDEX idx_sources_unique_youtube_video
    ON sources(source_type, source_subtype, external_id)
    WHERE source_type = 'youtube' AND source_subtype = 'video';

CREATE INDEX idx_telegram_forum_topics_source_top_message
    ON telegram_forum_topics(source_id, top_message_id);

CREATE UNIQUE INDEX idx_telegram_forum_topics_source_topic
    ON telegram_forum_topics(source_id, topic_id);

CREATE INDEX idx_telegram_messages_source_message
    ON telegram_messages(source_id, telegram_message_id);

CREATE INDEX idx_telegram_messages_source_reply_top
    ON telegram_messages(source_id, reply_to_top_id);

CREATE UNIQUE INDEX idx_telegram_sources_account_peer
    ON telegram_sources(account_id, peer_kind, peer_id);

CREATE INDEX idx_telegram_sources_account_subtype
    ON telegram_sources(account_id, source_subtype);

CREATE INDEX idx_telegram_sources_account_username
    ON telegram_sources(account_id, username)
    WHERE username IS NOT NULL;

CREATE INDEX idx_telegram_takeout_batches_account
    ON telegram_takeout_batches(account_id);

CREATE INDEX idx_youtube_playlist_items_playlist_position
    ON youtube_playlist_items(playlist_source_id, position);

CREATE INDEX idx_youtube_playlist_items_video_id
    ON youtube_playlist_items(video_id);

CREATE INDEX idx_youtube_playlist_items_video_source
    ON youtube_playlist_items(video_source_id);

CREATE INDEX idx_youtube_playlist_sources_playlist_id
    ON youtube_playlist_sources(playlist_id);

CREATE INDEX idx_youtube_transcript_segments_item_time
    ON youtube_transcript_segments(item_id, start_ms);

CREATE INDEX idx_youtube_transcript_segments_source
    ON youtube_transcript_segments(source_id);

CREATE INDEX idx_youtube_video_sources_video_id
    ON youtube_video_sources(video_id);

CREATE UNIQUE INDEX ux_items_non_telegram_external
    ON items(source_id, external_id)
    WHERE item_kind <> 'telegram_message';

CREATE UNIQUE INDEX ux_telegram_messages_native_identity
    ON telegram_messages (
        source_id,
        history_peer_kind,
        history_peer_id,
        telegram_message_id
    );
