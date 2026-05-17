use sqlx::{Executor, Sqlite};

use crate::error::{AppError, AppResult};

#[allow(dead_code)]
pub(crate) const YOUTUBE_RAW_METADATA_VERSION: i64 = 1;

#[allow(dead_code)]
pub(crate) const YOUTUBE_TYPED_SOURCE_TABLES_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS youtube_video_sources (
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

CREATE INDEX IF NOT EXISTS idx_youtube_video_sources_video_id
    ON youtube_video_sources(video_id);

CREATE TABLE IF NOT EXISTS youtube_playlist_sources (
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

CREATE INDEX IF NOT EXISTS idx_youtube_playlist_sources_playlist_id
    ON youtube_playlist_sources(playlist_id);
"#;

#[allow(dead_code)]
pub(crate) async fn create_youtube_typed_source_tables<'e, E>(executor: E) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::raw_sql(YOUTUBE_TYPED_SOURCE_TABLES_SQL)
        .execute(executor)
        .await
        .map_err(AppError::database)?;
    Ok(())
}
