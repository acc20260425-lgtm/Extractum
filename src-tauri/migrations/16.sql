ALTER TABLE items ADD COLUMN item_kind TEXT NOT NULL DEFAULT 'telegram_message';

CREATE INDEX IF NOT EXISTS idx_items_source_kind_published
    ON items(source_id, item_kind, published_at DESC);

CREATE TABLE IF NOT EXISTS youtube_playlist_items (
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

CREATE INDEX IF NOT EXISTS idx_youtube_playlist_items_playlist_position
    ON youtube_playlist_items(playlist_source_id, position);

CREATE INDEX IF NOT EXISTS idx_youtube_playlist_items_video_source
    ON youtube_playlist_items(video_source_id);

CREATE INDEX IF NOT EXISTS idx_youtube_playlist_items_video_id
    ON youtube_playlist_items(video_id);

CREATE TABLE IF NOT EXISTS youtube_transcript_segments (
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

CREATE INDEX IF NOT EXISTS idx_youtube_transcript_segments_item_time
    ON youtube_transcript_segments(item_id, start_ms);

CREATE INDEX IF NOT EXISTS idx_youtube_transcript_segments_source
    ON youtube_transcript_segments(source_id);

ALTER TABLE analysis_run_messages ADD COLUMN item_kind TEXT;
ALTER TABLE analysis_run_messages ADD COLUMN source_type TEXT;
ALTER TABLE analysis_run_messages ADD COLUMN source_subtype TEXT;
ALTER TABLE analysis_run_messages ADD COLUMN metadata_zstd BLOB;

ALTER TABLE analysis_source_groups ADD COLUMN source_type TEXT NOT NULL DEFAULT 'telegram';

CREATE INDEX IF NOT EXISTS idx_analysis_source_groups_source_type
    ON analysis_source_groups(source_type);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_youtube_video
    ON sources(source_type, source_subtype, external_id)
    WHERE source_type = 'youtube' AND source_subtype = 'video';

CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_youtube_playlist
    ON sources(source_type, source_subtype, external_id)
    WHERE source_type = 'youtube' AND source_subtype = 'playlist';

INSERT OR IGNORE INTO app_settings (key, value)
VALUES
    ('youtube.auth.enabled', 'false'),
    ('youtube.captions.preferred_language', 'original'),
    ('youtube.sync.delay_between_requests_ms', '1000'),
    ('youtube.sync.max_parallel_video_syncs', '1'),
    ('youtube.sync.max_parallel_comment_syncs', '1'),
    ('youtube.sync.pause_on_auth_challenge', 'true'),
    ('youtube.sync.daily_soft_limit', '0'),
    ('youtube.sync.retry_backoff_ms', '3000'),
    ('youtube.sync.stop_after_consecutive_failures', '3');
