#![allow(dead_code)]

use serde_json::Value;
use sqlx::{Executor, Sqlite};

use crate::compression::compress_json_bytes;
use crate::error::{AppError, AppResult};

use super::dto::{
    YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata,
};
use super::url::{parse_youtube_url, YoutubeUrlKind};

#[allow(dead_code)]
pub(crate) const YOUTUBE_RAW_METADATA_VERSION: i64 = 1;

#[allow(dead_code)]
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

#[allow(dead_code)]
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
        YoutubeAvailabilityStatus::LiveEndedTranscriptPending => {
            "live_ended_transcript_pending"
        }
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
        assert_eq!(columns.raw_metadata_version, Some(YOUTUBE_RAW_METADATA_VERSION));

        let raw = decode_raw_payload_for_test(columns.raw_metadata_zstd.as_deref().unwrap());
        assert_eq!(raw["caption_language_override"], "en");
        assert!(raw.get("http_headers").is_none());
        assert!(raw.get("command_args").is_none());
        assert!(!raw.to_string().contains("secret"));
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
        assert_eq!(columns.raw_metadata_version, Some(YOUTUBE_RAW_METADATA_VERSION));
        let raw = decode_raw_payload_for_test(columns.raw_metadata_zstd.as_deref().unwrap());
        assert!(raw.get("headers").is_none());
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
