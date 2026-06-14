use std::collections::HashMap;

use serde_json::Value;
use sqlx::{Executor, QueryBuilder, Row, Sqlite, SqliteConnection};

use crate::compression::compress_json_bytes;
use crate::error::{AppError, AppResult};
use crate::sql_helpers::push_i64_bind_list;

use super::dto::{
    YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata,
};
use super::url::{parse_youtube_url, YoutubeUrlKind};

pub(crate) const YOUTUBE_RAW_METADATA_VERSION: i64 = 1;

#[derive(Clone, Debug)]
pub(crate) struct YoutubeVideoSourceColumns {
    pub(crate) video_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) channel_url: Option<String>,
    pub(crate) author_display: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) duration_seconds: Option<i64>,
    pub(crate) description: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) tags_json: String,
    pub(crate) chapters_json: String,
    pub(crate) view_count: Option<i64>,
    pub(crate) like_count: Option<i64>,
    pub(crate) comment_count: Option<i64>,
    pub(crate) category: Option<String>,
    pub(crate) video_form: String,
    pub(crate) availability_status: String,
    pub(crate) caption_language_override: Option<String>,
    pub(crate) raw_metadata_version: Option<i64>,
    pub(crate) raw_metadata_zstd: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub(crate) struct YoutubePlaylistSourceColumns {
    pub(crate) playlist_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) channel_url: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) video_count: Option<i64>,
    pub(crate) availability_status: String,
    pub(crate) raw_metadata_version: Option<i64>,
    pub(crate) raw_metadata_zstd: Option<Vec<u8>>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct YoutubeVideoSourceMetadata {
    pub(crate) source_id: i64,
    pub(crate) video_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) channel_url: Option<String>,
    pub(crate) author_display: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) duration_seconds: Option<i64>,
    pub(crate) description: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) view_count: Option<i64>,
    pub(crate) like_count: Option<i64>,
    pub(crate) comment_count: Option<i64>,
    pub(crate) category: Option<String>,
    pub(crate) video_form: String,
    pub(crate) availability_status: String,
    pub(crate) caption_language_override: Option<String>,
    pub(crate) raw_metadata_version: Option<i64>,
    pub(crate) raw_metadata_zstd: Option<Vec<u8>>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct YoutubePlaylistSourceMetadata {
    pub(crate) source_id: i64,
    pub(crate) playlist_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) video_count: Option<i64>,
    pub(crate) availability_status: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct YoutubeVideoDescriptionMetadata {
    pub(crate) source_id: i64,
    pub(crate) video_id: String,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) channel_title: Option<String>,
    pub(crate) channel_handle: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) description: Option<String>,
}

impl YoutubeVideoSourceMetadata {
    pub(crate) fn video_form_for_provider(&self) -> Option<YoutubeVideoForm> {
        match self.video_form.as_str() {
            "regular" => Some(YoutubeVideoForm::Regular),
            "short" => Some(YoutubeVideoForm::Short),
            "live" => Some(YoutubeVideoForm::Live),
            _ => None,
        }
    }

    pub(crate) fn to_provider_metadata(&self) -> YoutubeVideoMetadata {
        YoutubeVideoMetadata {
            video_id: self.video_id.clone(),
            canonical_url: self.canonical_url.clone(),
            title: self.title.clone(),
            channel_title: self.channel_title.clone(),
            channel_id: self.channel_id.clone(),
            channel_handle: self.channel_handle.clone(),
            channel_url: self.channel_url.clone(),
            author_display: self.author_display.clone(),
            published_at: self.published_at.clone(),
            duration_seconds: self.duration_seconds,
            description: self.description.clone(),
            thumbnail_url: self.thumbnail_url.clone(),
            tags: Vec::new(),
            chapters: Vec::new(),
            view_count: self.view_count,
            like_count: self.like_count,
            comment_count: self.comment_count,
            category: self.category.clone(),
            video_form: self
                .video_form_for_provider()
                .unwrap_or(YoutubeVideoForm::Regular),
            availability_status: availability_status_from_wire(&self.availability_status),
            raw_metadata_json: decode_raw_metadata_json(self.raw_metadata_zstd.as_deref())
                .unwrap_or(Value::Null),
        }
    }
}

impl YoutubeVideoSourceColumns {
    pub(crate) fn try_from_metadata(metadata: &YoutubeVideoMetadata) -> AppResult<Self> {
        validate_video_canonical_url(&metadata.video_id, &metadata.canonical_url)?;
        let tags_json = serde_json::to_string(&metadata.tags)
            .map_err(|error| AppError::internal(error.to_string()))?;
        let chapters_json = serde_json::to_string(&metadata.chapters)
            .map_err(|error| AppError::internal(error.to_string()))?;
        let (raw_metadata_version, raw_metadata_zstd) =
            raw_metadata_columns(&metadata.raw_metadata_json)?;

        Ok(Self {
            video_id: metadata.video_id.clone(),
            canonical_url: metadata.canonical_url.clone(),
            title: metadata.title.clone(),
            channel_title: metadata.channel_title.clone(),
            channel_id: metadata.channel_id.clone(),
            channel_handle: metadata.channel_handle.clone(),
            channel_url: metadata.channel_url.clone(),
            author_display: metadata.author_display.clone(),
            published_at: metadata.published_at.clone(),
            duration_seconds: metadata.duration_seconds,
            description: metadata.description.clone(),
            thumbnail_url: metadata.thumbnail_url.clone(),
            tags_json,
            chapters_json,
            view_count: metadata.view_count,
            like_count: metadata.like_count,
            comment_count: metadata.comment_count,
            category: metadata.category.clone(),
            video_form: video_form_wire(&metadata.video_form).to_string(),
            availability_status: availability_status_wire(&metadata.availability_status)
                .to_string(),
            caption_language_override: caption_language_override_from_raw(
                &metadata.raw_metadata_json,
            ),
            raw_metadata_version,
            raw_metadata_zstd,
        })
    }
}

impl YoutubePlaylistSourceColumns {
    pub(crate) fn try_from_metadata(metadata: &YoutubePlaylistMetadata) -> AppResult<Self> {
        validate_playlist_canonical_url(&metadata.playlist_id, &metadata.canonical_url)?;
        let (raw_metadata_version, raw_metadata_zstd) =
            raw_metadata_columns(&metadata.raw_metadata_json)?;

        Ok(Self {
            playlist_id: metadata.playlist_id.clone(),
            canonical_url: metadata.canonical_url.clone(),
            title: metadata.title.clone(),
            channel_title: metadata.channel_title.clone(),
            channel_id: metadata.channel_id.clone(),
            channel_handle: metadata.channel_handle.clone(),
            channel_url: metadata.channel_url.clone(),
            thumbnail_url: metadata.thumbnail_url.clone(),
            video_count: metadata.video_count,
            availability_status: availability_status_wire(&metadata.availability_status)
                .to_string(),
            raw_metadata_version,
            raw_metadata_zstd,
        })
    }
}

fn validate_video_canonical_url(video_id: &str, canonical_url: &str) -> AppResult<()> {
    let parsed = parse_youtube_url(canonical_url).map_err(|_| {
        AppError::validation(format!(
            "YouTube video metadata canonical_url is invalid for video {video_id}"
        ))
    })?;
    let parsed_id = match parsed.kind {
        YoutubeUrlKind::Video { video_id }
        | YoutubeUrlKind::Short { video_id }
        | YoutubeUrlKind::Live { video_id } => video_id,
        YoutubeUrlKind::Playlist { .. } => {
            return Err(AppError::validation(format!(
                "YouTube video metadata canonical_url is not a video URL for video {video_id}"
            )));
        }
    };
    if parsed_id != video_id {
        return Err(AppError::validation(format!(
            "YouTube video metadata canonical_url id does not match video {video_id}"
        )));
    }
    Ok(())
}

fn validate_playlist_canonical_url(playlist_id: &str, canonical_url: &str) -> AppResult<()> {
    let parsed = parse_youtube_url(canonical_url).map_err(|_| {
        AppError::validation(format!(
            "YouTube playlist metadata canonical_url is invalid for playlist {playlist_id}"
        ))
    })?;
    match parsed.kind {
        YoutubeUrlKind::Playlist {
            playlist_id: parsed_id,
        } if parsed_id == playlist_id => Ok(()),
        _ => Err(AppError::validation(format!(
            "YouTube playlist metadata canonical_url id does not match playlist {playlist_id}"
        ))),
    }
}

pub(crate) fn video_form_wire(form: &YoutubeVideoForm) -> &'static str {
    match form {
        YoutubeVideoForm::Regular => "regular",
        YoutubeVideoForm::Short => "short",
        YoutubeVideoForm::Live => "live",
    }
}

pub(crate) fn availability_status_wire(status: &YoutubeAvailabilityStatus) -> &'static str {
    match status {
        YoutubeAvailabilityStatus::Available => "available",
        YoutubeAvailabilityStatus::Upcoming => "upcoming",
        YoutubeAvailabilityStatus::LiveNow => "live_now",
        YoutubeAvailabilityStatus::LiveEndedTranscriptPending => "live_ended_transcript_pending",
        YoutubeAvailabilityStatus::NoCaptions => "no_captions",
        YoutubeAvailabilityStatus::PrivateOrAuthRequired => "private_or_auth_required",
        YoutubeAvailabilityStatus::MembersOnly => "members_only",
        YoutubeAvailabilityStatus::AgeRestricted => "age_restricted",
        YoutubeAvailabilityStatus::GeoBlocked => "geo_blocked",
        YoutubeAvailabilityStatus::Deleted => "deleted",
        YoutubeAvailabilityStatus::RemovedFromPlaylist => "removed_from_playlist",
        YoutubeAvailabilityStatus::UnavailableUnknown => "unavailable_unknown",
    }
}

pub(crate) async fn load_video_source_metadata_map(
    pool: &sqlx::SqlitePool,
    source_ids: &[i64],
) -> AppResult<HashMap<i64, YoutubeVideoSourceMetadata>> {
    if source_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut query = QueryBuilder::new(
        r#"
        SELECT
            s.id AS source_id,
            s.external_id,
            yvs.video_id,
            yvs.canonical_url,
            yvs.title,
            yvs.channel_title,
            yvs.channel_id,
            yvs.channel_handle,
            yvs.channel_url,
            yvs.author_display,
            yvs.published_at,
            yvs.duration_seconds,
            yvs.description,
            yvs.thumbnail_url,
            yvs.view_count,
            yvs.like_count,
            yvs.comment_count,
            yvs.category,
            yvs.video_form,
            yvs.availability_status,
            yvs.caption_language_override,
            yvs.raw_metadata_version,
            yvs.raw_metadata_zstd
        FROM sources s
        JOIN youtube_video_sources yvs ON yvs.source_id = s.id
        WHERE s.source_type = 'youtube'
          AND s.source_subtype = 'video'
          AND s.id IN (
        "#,
    );
    push_i64_bind_list(&mut query, source_ids);
    query.push(")");
    let rows = query
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;
    video_metadata_rows_to_map(rows)
}

pub(crate) async fn load_playlist_source_metadata_map(
    pool: &sqlx::SqlitePool,
    source_ids: &[i64],
) -> AppResult<HashMap<i64, YoutubePlaylistSourceMetadata>> {
    if source_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut query = QueryBuilder::new(
        r#"
        SELECT
            s.id AS source_id,
            s.external_id,
            yps.playlist_id,
            yps.canonical_url,
            yps.title,
            yps.channel_title,
            yps.channel_handle,
            yps.thumbnail_url,
            yps.video_count,
            yps.availability_status
        FROM sources s
        JOIN youtube_playlist_sources yps ON yps.source_id = s.id
        WHERE s.source_type = 'youtube'
          AND s.source_subtype = 'playlist'
          AND s.id IN (
        "#,
    );
    push_i64_bind_list(&mut query, source_ids);
    query.push(")");
    let rows = query
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;
    playlist_metadata_rows_to_map(rows)
}

#[allow(dead_code)]
pub(crate) async fn load_video_description_metadata(
    pool: &sqlx::SqlitePool,
    source_ids: &[i64],
) -> AppResult<Vec<YoutubeVideoDescriptionMetadata>> {
    if source_ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = QueryBuilder::new(
        r#"
        SELECT
            s.id AS source_id,
            s.external_id,
            yvs.video_id,
            yvs.canonical_url,
            yvs.title,
            yvs.channel_title,
            yvs.channel_handle,
            yvs.published_at,
            yvs.description,
            yvs.tags_json,
            yvs.chapters_json,
            yvs.availability_status
        FROM sources s
        JOIN youtube_video_sources yvs ON yvs.source_id = s.id
        WHERE s.source_type = 'youtube'
          AND s.source_subtype = 'video'
          AND s.id IN (
        "#,
    );
    push_i64_bind_list(&mut query, source_ids);
    query.push(") ORDER BY s.id ASC");
    let rows = query
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    let mut metadata = Vec::new();
    for row in rows {
        if let Some(row_metadata) = valid_description_metadata_from_row(row)? {
            metadata.push(row_metadata);
        }
    }
    Ok(metadata)
}

fn video_metadata_rows_to_map(
    rows: Vec<sqlx::sqlite::SqliteRow>,
) -> AppResult<HashMap<i64, YoutubeVideoSourceMetadata>> {
    let mut metadata = HashMap::new();
    for row in rows {
        let source_id = row
            .try_get::<i64, _>("source_id")
            .map_err(AppError::database)?;
        let external_id = row
            .try_get::<String, _>("external_id")
            .map_err(AppError::database)?;
        let video_id = row
            .try_get::<String, _>("video_id")
            .map_err(AppError::database)?;
        let canonical_url = row
            .try_get::<String, _>("canonical_url")
            .map_err(AppError::database)?;
        let video_form = row
            .try_get::<String, _>("video_form")
            .map_err(AppError::database)?;
        let availability_status = row
            .try_get::<String, _>("availability_status")
            .map_err(AppError::database)?;

        if external_id != video_id
            || !is_video_form_wire(&video_form)
            || !is_availability_status_wire(&availability_status)
            || validate_video_canonical_url(&video_id, &canonical_url).is_err()
        {
            continue;
        }

        metadata.insert(
            source_id,
            YoutubeVideoSourceMetadata {
                source_id,
                video_id,
                canonical_url,
                title: row.try_get("title").map_err(AppError::database)?,
                channel_title: row.try_get("channel_title").map_err(AppError::database)?,
                channel_id: row.try_get("channel_id").map_err(AppError::database)?,
                channel_handle: row.try_get("channel_handle").map_err(AppError::database)?,
                channel_url: row.try_get("channel_url").map_err(AppError::database)?,
                author_display: row.try_get("author_display").map_err(AppError::database)?,
                published_at: row.try_get("published_at").map_err(AppError::database)?,
                duration_seconds: row
                    .try_get("duration_seconds")
                    .map_err(AppError::database)?,
                description: row.try_get("description").map_err(AppError::database)?,
                thumbnail_url: row.try_get("thumbnail_url").map_err(AppError::database)?,
                view_count: row.try_get("view_count").map_err(AppError::database)?,
                like_count: row.try_get("like_count").map_err(AppError::database)?,
                comment_count: row.try_get("comment_count").map_err(AppError::database)?,
                category: row.try_get("category").map_err(AppError::database)?,
                video_form,
                availability_status,
                caption_language_override: row
                    .try_get("caption_language_override")
                    .map_err(AppError::database)?,
                raw_metadata_version: row
                    .try_get("raw_metadata_version")
                    .map_err(AppError::database)?,
                raw_metadata_zstd: row
                    .try_get("raw_metadata_zstd")
                    .map_err(AppError::database)?,
            },
        );
    }
    Ok(metadata)
}

#[allow(dead_code)]
fn valid_description_metadata_from_row(
    row: sqlx::sqlite::SqliteRow,
) -> AppResult<Option<YoutubeVideoDescriptionMetadata>> {
    let source_id = row
        .try_get::<i64, _>("source_id")
        .map_err(AppError::database)?;
    let external_id = row
        .try_get::<String, _>("external_id")
        .map_err(AppError::database)?;
    let video_id = row
        .try_get::<String, _>("video_id")
        .map_err(AppError::database)?;
    let canonical_url = row
        .try_get::<String, _>("canonical_url")
        .map_err(AppError::database)?;
    let availability_status = row
        .try_get::<String, _>("availability_status")
        .map_err(AppError::database)?;
    let tags_json = row
        .try_get::<String, _>("tags_json")
        .map_err(AppError::database)?;
    let chapters_json = row
        .try_get::<String, _>("chapters_json")
        .map_err(AppError::database)?;

    if external_id != video_id
        || !is_availability_status_wire(&availability_status)
        || validate_video_canonical_url(&video_id, &canonical_url).is_err()
        || !json_text_is_array(&tags_json)
        || !json_text_is_array(&chapters_json)
    {
        return Ok(None);
    }

    Ok(Some(YoutubeVideoDescriptionMetadata {
        source_id,
        video_id,
        canonical_url,
        title: row.try_get("title").map_err(AppError::database)?,
        channel_title: row.try_get("channel_title").map_err(AppError::database)?,
        channel_handle: row.try_get("channel_handle").map_err(AppError::database)?,
        published_at: row.try_get("published_at").map_err(AppError::database)?,
        description: row.try_get("description").map_err(AppError::database)?,
    }))
}

fn playlist_metadata_rows_to_map(
    rows: Vec<sqlx::sqlite::SqliteRow>,
) -> AppResult<HashMap<i64, YoutubePlaylistSourceMetadata>> {
    let mut metadata = HashMap::new();
    for row in rows {
        let source_id = row
            .try_get::<i64, _>("source_id")
            .map_err(AppError::database)?;
        let external_id = row
            .try_get::<String, _>("external_id")
            .map_err(AppError::database)?;
        let playlist_id = row
            .try_get::<String, _>("playlist_id")
            .map_err(AppError::database)?;
        let canonical_url = row
            .try_get::<String, _>("canonical_url")
            .map_err(AppError::database)?;
        let availability_status = row
            .try_get::<String, _>("availability_status")
            .map_err(AppError::database)?;

        if external_id != playlist_id
            || !is_availability_status_wire(&availability_status)
            || validate_playlist_canonical_url(&playlist_id, &canonical_url).is_err()
        {
            continue;
        }

        metadata.insert(
            source_id,
            YoutubePlaylistSourceMetadata {
                source_id,
                playlist_id,
                canonical_url,
                title: row.try_get("title").map_err(AppError::database)?,
                channel_title: row.try_get("channel_title").map_err(AppError::database)?,
                channel_handle: row.try_get("channel_handle").map_err(AppError::database)?,
                thumbnail_url: row.try_get("thumbnail_url").map_err(AppError::database)?,
                video_count: row.try_get("video_count").map_err(AppError::database)?,
                availability_status,
            },
        );
    }
    Ok(metadata)
}

fn is_video_form_wire(value: &str) -> bool {
    matches!(value, "regular" | "short" | "live")
}

fn is_availability_status_wire(value: &str) -> bool {
    matches!(
        value,
        "available"
            | "upcoming"
            | "live_now"
            | "live_ended_transcript_pending"
            | "no_captions"
            | "private_or_auth_required"
            | "members_only"
            | "age_restricted"
            | "geo_blocked"
            | "deleted"
            | "removed_from_playlist"
            | "unavailable_unknown"
    )
}

fn availability_status_from_wire(value: &str) -> YoutubeAvailabilityStatus {
    match value {
        "available" => YoutubeAvailabilityStatus::Available,
        "upcoming" => YoutubeAvailabilityStatus::Upcoming,
        "live_now" => YoutubeAvailabilityStatus::LiveNow,
        "live_ended_transcript_pending" => YoutubeAvailabilityStatus::LiveEndedTranscriptPending,
        "no_captions" => YoutubeAvailabilityStatus::NoCaptions,
        "private_or_auth_required" => YoutubeAvailabilityStatus::PrivateOrAuthRequired,
        "members_only" => YoutubeAvailabilityStatus::MembersOnly,
        "age_restricted" => YoutubeAvailabilityStatus::AgeRestricted,
        "geo_blocked" => YoutubeAvailabilityStatus::GeoBlocked,
        "deleted" => YoutubeAvailabilityStatus::Deleted,
        "removed_from_playlist" => YoutubeAvailabilityStatus::RemovedFromPlaylist,
        _ => YoutubeAvailabilityStatus::UnavailableUnknown,
    }
}

#[allow(dead_code)]
fn json_text_is_array(value: &str) -> bool {
    serde_json::from_str::<Value>(value).is_ok_and(|value| value.is_array())
}

fn raw_metadata_columns(raw: &Value) -> AppResult<(Option<i64>, Option<Vec<u8>>)> {
    if raw.is_null() {
        return Ok((None, None));
    }
    let sanitized = sanitize_raw_metadata(raw);
    let json =
        serde_json::to_vec(&sanitized).map_err(|error| AppError::internal(error.to_string()))?;
    let compressed = compress_json_bytes(&json).map_err(AppError::internal)?;
    Ok((Some(YOUTUBE_RAW_METADATA_VERSION), Some(compressed)))
}

fn decode_raw_metadata_json(bytes: Option<&[u8]>) -> Option<Value> {
    let json = crate::compression::decompress_bytes(bytes?).ok()?;
    serde_json::from_slice(&json).ok()
}

fn sanitize_raw_metadata(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| !is_secret_raw_key(key))
                .map(|(key, value)| (key.clone(), sanitize_raw_metadata(value)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(sanitize_raw_metadata).collect()),
        other => other.clone(),
    }
}

fn is_secret_raw_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "cookie"
            | "cookies"
            | "headers"
            | "http_headers"
            | "request_headers"
            | "command_args"
            | "argv"
            | "auth_diagnostics"
            | "logs"
            | "stderr"
            | "stdout"
    )
}

fn caption_language_override_from_raw(raw: &Value) -> Option<String> {
    raw.get("caption_language_override")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

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

#[allow(dead_code)]
pub(crate) fn decode_legacy_video_source_metadata(bytes: &[u8]) -> Option<YoutubeVideoMetadata> {
    let json = crate::compression::decompress_bytes(bytes).ok()?;
    serde_json::from_slice(&json).ok()
}

#[allow(dead_code)]
pub(crate) fn decode_legacy_playlist_source_metadata(
    bytes: &[u8],
) -> Option<YoutubePlaylistMetadata> {
    let json = crate::compression::decompress_bytes(bytes).ok()?;
    serde_json::from_slice(&json).ok()
}

#[allow(dead_code)]
pub(crate) async fn insert_video_source_metadata_on_connection(
    conn: &mut SqliteConnection,
    source_id: i64,
    metadata: &YoutubeVideoMetadata,
) -> AppResult<()> {
    let columns = YoutubeVideoSourceColumns::try_from_metadata(metadata)?;
    insert_video_source_columns(conn, source_id, &columns).await
}

#[allow(dead_code)]
pub(crate) async fn insert_playlist_source_metadata_on_connection(
    conn: &mut SqliteConnection,
    source_id: i64,
    metadata: &YoutubePlaylistMetadata,
) -> AppResult<()> {
    let columns = YoutubePlaylistSourceColumns::try_from_metadata(metadata)?;
    insert_playlist_source_columns(conn, source_id, &columns).await
}

pub(crate) async fn upsert_video_source_metadata(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    source_id: i64,
    metadata: &YoutubeVideoMetadata,
) -> AppResult<()> {
    let columns = YoutubeVideoSourceColumns::try_from_metadata(metadata)?;
    insert_video_source_columns(&mut **tx, source_id, &columns).await?;
    crate::analysis_documents::upsert_youtube_description_document_on_connection(
        &mut **tx, source_id,
    )
    .await
}

#[cfg(test)]
pub(crate) async fn insert_video_source_metadata_for_pool_test(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    metadata: &YoutubeVideoMetadata,
) {
    let columns =
        YoutubeVideoSourceColumns::try_from_metadata(metadata).expect("valid video metadata");
    let mut conn = pool.acquire().await.expect("acquire sqlite connection");
    insert_video_source_columns(&mut conn, source_id, &columns)
        .await
        .expect("insert typed video metadata");
}

pub(crate) async fn upsert_playlist_source_metadata(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    source_id: i64,
    metadata: &YoutubePlaylistMetadata,
) -> AppResult<()> {
    let columns = YoutubePlaylistSourceColumns::try_from_metadata(metadata)?;
    insert_playlist_source_columns(&mut **tx, source_id, &columns).await
}

async fn insert_video_source_columns(
    conn: &mut SqliteConnection,
    source_id: i64,
    columns: &YoutubeVideoSourceColumns,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO youtube_video_sources (
            source_id,
            video_id,
            canonical_url,
            title,
            channel_title,
            channel_id,
            channel_handle,
            channel_url,
            author_display,
            published_at,
            duration_seconds,
            description,
            thumbnail_url,
            tags_json,
            chapters_json,
            view_count,
            like_count,
            comment_count,
            category,
            video_form,
            availability_status,
            caption_language_override,
            raw_metadata_version,
            raw_metadata_zstd
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(source_id) DO UPDATE SET
            video_id = excluded.video_id,
            canonical_url = excluded.canonical_url,
            title = excluded.title,
            channel_title = excluded.channel_title,
            channel_id = excluded.channel_id,
            channel_handle = excluded.channel_handle,
            channel_url = excluded.channel_url,
            author_display = excluded.author_display,
            published_at = excluded.published_at,
            duration_seconds = excluded.duration_seconds,
            description = excluded.description,
            thumbnail_url = excluded.thumbnail_url,
            tags_json = excluded.tags_json,
            chapters_json = excluded.chapters_json,
            view_count = excluded.view_count,
            like_count = excluded.like_count,
            comment_count = excluded.comment_count,
            category = excluded.category,
            video_form = excluded.video_form,
            availability_status = excluded.availability_status,
            caption_language_override = excluded.caption_language_override,
            raw_metadata_version = excluded.raw_metadata_version,
            raw_metadata_zstd = excluded.raw_metadata_zstd,
            updated_at = strftime('%s','now')
        "#,
    )
    .bind(source_id)
    .bind(&columns.video_id)
    .bind(&columns.canonical_url)
    .bind(&columns.title)
    .bind(&columns.channel_title)
    .bind(&columns.channel_id)
    .bind(&columns.channel_handle)
    .bind(&columns.channel_url)
    .bind(&columns.author_display)
    .bind(&columns.published_at)
    .bind(columns.duration_seconds)
    .bind(&columns.description)
    .bind(&columns.thumbnail_url)
    .bind(&columns.tags_json)
    .bind(&columns.chapters_json)
    .bind(columns.view_count)
    .bind(columns.like_count)
    .bind(columns.comment_count)
    .bind(&columns.category)
    .bind(&columns.video_form)
    .bind(&columns.availability_status)
    .bind(&columns.caption_language_override)
    .bind(columns.raw_metadata_version)
    .bind(columns.raw_metadata_zstd.as_deref())
    .execute(conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn insert_playlist_source_columns(
    conn: &mut SqliteConnection,
    source_id: i64,
    columns: &YoutubePlaylistSourceColumns,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO youtube_playlist_sources (
            source_id,
            playlist_id,
            canonical_url,
            title,
            channel_title,
            channel_id,
            channel_handle,
            channel_url,
            thumbnail_url,
            video_count,
            availability_status,
            raw_metadata_version,
            raw_metadata_zstd
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(source_id) DO UPDATE SET
            playlist_id = excluded.playlist_id,
            canonical_url = excluded.canonical_url,
            title = excluded.title,
            channel_title = excluded.channel_title,
            channel_id = excluded.channel_id,
            channel_handle = excluded.channel_handle,
            channel_url = excluded.channel_url,
            thumbnail_url = excluded.thumbnail_url,
            video_count = excluded.video_count,
            availability_status = excluded.availability_status,
            raw_metadata_version = excluded.raw_metadata_version,
            raw_metadata_zstd = excluded.raw_metadata_zstd,
            updated_at = strftime('%s','now')
        "#,
    )
    .bind(source_id)
    .bind(&columns.playlist_id)
    .bind(&columns.canonical_url)
    .bind(&columns.title)
    .bind(&columns.channel_title)
    .bind(&columns.channel_id)
    .bind(&columns.channel_handle)
    .bind(&columns.channel_url)
    .bind(&columns.thumbnail_url)
    .bind(columns.video_count)
    .bind(&columns.availability_status)
    .bind(columns.raw_metadata_version)
    .bind(columns.raw_metadata_zstd.as_deref())
    .execute(conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::youtube::dto::{
        YoutubeAvailabilityStatus, YoutubeChapter, YoutubePlaylistMetadata, YoutubeVideoForm,
        YoutubeVideoMetadata,
    };

    #[test]
    fn video_metadata_columns_include_wire_values_arrays_caption_override_and_sanitized_raw() {
        let metadata = video_metadata(json!({
            "id": "video01",
            "caption_language_override": "en",
            "http_headers": { "cookie": "secret" },
            "command_args": ["--cookies", "secret"]
        }));

        let columns = YoutubeVideoSourceColumns::try_from_metadata(&metadata)
            .expect("convert video metadata");

        assert_eq!(columns.video_form, "short");
        assert_eq!(columns.availability_status, "available");
        assert_eq!(columns.caption_language_override.as_deref(), Some("en"));
        assert_eq!(columns.tags_json, r#"["tag-one"]"#);
        assert!(columns.chapters_json.contains("\"start_ms\":1000"));
        assert_eq!(
            columns.raw_metadata_version,
            Some(YOUTUBE_RAW_METADATA_VERSION)
        );

        let raw = decode_raw_payload_for_test(columns.raw_metadata_zstd.as_deref().unwrap());
        assert_eq!(raw["caption_language_override"], "en");
        assert!(raw.get("http_headers").is_none());
        assert!(raw.get("command_args").is_none());
        assert!(!raw.to_string().contains("secret"));
    }

    #[test]
    fn video_source_metadata_restores_raw_caption_metadata_for_provider_sync() {
        let metadata = video_metadata(json!({
            "id": "video01",
            "language": "ru",
            "automatic_captions": {
                "ru": [{ "ext": "json3", "name": "Russian" }]
            }
        }));
        let columns = YoutubeVideoSourceColumns::try_from_metadata(&metadata)
            .expect("convert video metadata");
        let loaded = YoutubeVideoSourceMetadata {
            source_id: 2,
            video_id: columns.video_id,
            canonical_url: columns.canonical_url,
            title: columns.title,
            channel_title: columns.channel_title,
            channel_id: columns.channel_id,
            channel_handle: columns.channel_handle,
            channel_url: columns.channel_url,
            author_display: columns.author_display,
            published_at: columns.published_at,
            duration_seconds: columns.duration_seconds,
            description: columns.description,
            thumbnail_url: columns.thumbnail_url,
            view_count: columns.view_count,
            like_count: columns.like_count,
            comment_count: columns.comment_count,
            category: columns.category,
            video_form: columns.video_form,
            availability_status: columns.availability_status,
            caption_language_override: columns.caption_language_override,
            raw_metadata_version: columns.raw_metadata_version,
            raw_metadata_zstd: columns.raw_metadata_zstd,
        };

        let provider_metadata = loaded.to_provider_metadata();
        let tracks = crate::youtube::captions::caption_tracks_from_metadata(&provider_metadata);

        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].language.as_deref(), Some("ru"));
    }

    #[test]
    fn video_metadata_rejects_wrong_canonical_url_shape() {
        let mut metadata = video_metadata(json!({ "id": "video01" }));
        metadata.canonical_url = "https://example.com/watch?v=video01".to_string();

        let error = YoutubeVideoSourceColumns::try_from_metadata(&metadata)
            .expect_err("invalid url rejected");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.to_string().contains("canonical_url"));
    }

    #[test]
    fn playlist_metadata_columns_are_versioned_and_secret_safe() {
        let metadata = YoutubePlaylistMetadata {
            playlist_id: "PLdemo".to_string(),
            canonical_url: "https://www.youtube.com/playlist?list=PLdemo".to_string(),
            title: Some("Demo playlist".to_string()),
            channel_title: Some("Demo channel".to_string()),
            channel_id: Some("channel-1".to_string()),
            channel_handle: Some("@demo".to_string()),
            channel_url: Some("https://www.youtube.com/@demo".to_string()),
            thumbnail_url: None,
            video_count: Some(2),
            items: Vec::new(),
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: json!({ "id": "PLdemo", "headers": { "cookie": "secret" } }),
        };

        let columns = YoutubePlaylistSourceColumns::try_from_metadata(&metadata)
            .expect("convert playlist metadata");

        assert_eq!(columns.playlist_id, "PLdemo");
        assert_eq!(columns.availability_status, "available");
        assert_eq!(
            columns.raw_metadata_version,
            Some(YOUTUBE_RAW_METADATA_VERSION)
        );
        let raw = decode_raw_payload_for_test(columns.raw_metadata_zstd.as_deref().unwrap());
        assert!(raw.get("headers").is_none());
    }

    #[tokio::test]
    async fn upsert_video_metadata_maintains_description_document() {
        let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;
        crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
        crate::sources::test_support::create_analysis_documents_table(&pool).await;
        seed_video_source_for_metadata_test(&pool, 2, "video2").await;

        let mut metadata = video_metadata(json!({ "id": "video2" }));
        metadata.video_id = "video2".to_string();
        metadata.canonical_url = "https://www.youtube.com/watch?v=video2".to_string();
        metadata.description = Some("First description".to_string());
        metadata.published_at = Some("2026-05-01".to_string());

        let mut tx = pool.begin().await.expect("begin tx");
        upsert_video_source_metadata(&mut tx, 2, &metadata)
            .await
            .expect("upsert metadata");
        tx.commit().await.expect("commit");

        let content: Vec<u8> = sqlx::query_scalar(
            "SELECT content_zstd FROM analysis_documents
             WHERE source_id = 2 AND document_key = 'youtube:description'",
        )
        .fetch_one(&pool)
        .await
        .expect("load description doc");
        let text = crate::compression::decompress_text(&content).expect("decompress");
        assert!(text.contains("First description"));

        metadata.description = Some("   ".to_string());
        let mut tx = pool.begin().await.expect("begin tx");
        upsert_video_source_metadata(&mut tx, 2, &metadata)
            .await
            .expect("clear metadata");
        tx.commit().await.expect("commit clear");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM analysis_documents
             WHERE source_id = 2 AND document_key = 'youtube:description'",
        )
        .fetch_one(&pool)
        .await
        .expect("count docs");
        assert_eq!(count, 0);
    }

    async fn seed_video_source_for_metadata_test(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        video_id: &str,
    ) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (?, 'youtube', 'video', ?, 'Video', 1, 1, 1)",
        )
        .bind(source_id)
        .bind(video_id)
        .execute(pool)
        .await
        .expect("seed source");
    }

    fn video_metadata(raw_metadata_json: serde_json::Value) -> YoutubeVideoMetadata {
        YoutubeVideoMetadata {
            video_id: "video01".to_string(),
            canonical_url: "https://www.youtube.com/shorts/video01".to_string(),
            title: Some("Demo video".to_string()),
            channel_title: Some("Demo channel".to_string()),
            channel_id: Some("channel-1".to_string()),
            channel_handle: Some("@demo".to_string()),
            channel_url: Some("https://www.youtube.com/@demo".to_string()),
            author_display: Some("Demo channel".to_string()),
            published_at: Some("2026-05-17".to_string()),
            duration_seconds: Some(42),
            description: Some("Description".to_string()),
            thumbnail_url: Some("https://img.youtube.com/vi/video01/hqdefault.jpg".to_string()),
            tags: vec!["tag-one".to_string()],
            chapters: vec![YoutubeChapter {
                index: 0,
                title: "Intro".to_string(),
                start_ms: 1000,
                end_ms: Some(2000),
            }],
            view_count: Some(10),
            like_count: Some(5),
            comment_count: Some(2),
            category: Some("Education".to_string()),
            video_form: YoutubeVideoForm::Short,
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json,
        }
    }

    fn decode_raw_payload_for_test(bytes: &[u8]) -> serde_json::Value {
        let decoded = crate::compression::decompress_bytes(bytes).expect("decompress raw");
        serde_json::from_slice(&decoded).expect("parse raw")
    }
}
