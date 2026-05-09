use serde_json::Value;

use crate::error::{AppError, AppResult};

use super::dto::{
    YoutubeAvailabilityStatus, YoutubeCaptionsEstimate, YoutubeChapter,
    YoutubePlaylistItemMetadata, YoutubePlaylistMetadata, YoutubePreview, YoutubePreviewKind,
    YoutubeVideoForm, YoutubeVideoMetadata,
};
use super::url::{YoutubeParsedUrl, YoutubeUrlKind};

pub(crate) fn video_metadata_from_ytdlp(
    value: Value,
    parsed: &YoutubeParsedUrl,
    video_form: YoutubeVideoForm,
) -> AppResult<YoutubeVideoMetadata> {
    let video_id = string_field(&value, "id")
        .or_else(|| video_id_from_parsed(parsed))
        .ok_or_else(|| AppError::validation("YouTube video metadata is missing id"))?;

    Ok(YoutubeVideoMetadata {
        video_id,
        canonical_url: parsed.canonical_url.clone(),
        title: string_field(&value, "title"),
        channel_title: first_string_field(&value, &["channel", "uploader"]),
        channel_id: first_string_field(&value, &["channel_id", "uploader_id"])
            .filter(|value| !value.starts_with('@')),
        channel_handle: channel_handle(&value),
        channel_url: first_string_field(&value, &["channel_url", "uploader_url"]),
        author_display: first_string_field(&value, &["uploader", "channel"]),
        published_at: published_at(&value),
        duration_seconds: i64_field(&value, "duration"),
        description: string_field(&value, "description"),
        thumbnail_url: thumbnail_url(&value),
        tags: string_array_field(&value, "tags"),
        chapters: chapters(&value),
        view_count: i64_field(&value, "view_count"),
        like_count: i64_field(&value, "like_count"),
        comment_count: i64_field(&value, "comment_count"),
        category: category(&value),
        video_form,
        availability_status: availability_status(&value),
        raw_metadata_json: value,
    })
}

pub(crate) fn playlist_metadata_from_ytdlp(
    value: Value,
    parsed: &YoutubeParsedUrl,
) -> AppResult<YoutubePlaylistMetadata> {
    let playlist_id = string_field(&value, "id")
        .or_else(|| playlist_id_from_parsed(parsed))
        .ok_or_else(|| AppError::validation("YouTube playlist metadata is missing id"))?;

    let items = value
        .get("entries")
        .and_then(Value::as_array)
        .map(|entries| playlist_items(entries))
        .unwrap_or_default();

    Ok(YoutubePlaylistMetadata {
        playlist_id,
        canonical_url: parsed.canonical_url.clone(),
        title: string_field(&value, "title"),
        channel_title: first_string_field(&value, &["channel", "uploader"]),
        channel_id: first_string_field(&value, &["channel_id", "uploader_id"])
            .filter(|value| !value.starts_with('@')),
        channel_handle: channel_handle(&value),
        channel_url: first_string_field(&value, &["channel_url", "uploader_url"]),
        thumbnail_url: thumbnail_url(&value),
        video_count: first_i64_field(&value, &["playlist_count", "n_entries"])
            .or_else(|| Some(items.len() as i64).filter(|count| *count > 0)),
        items,
        availability_status: availability_status(&value),
        raw_metadata_json: value,
    })
}

pub(crate) fn video_preview_from_metadata(metadata: &YoutubeVideoMetadata) -> YoutubePreview {
    YoutubePreview {
        kind: YoutubePreviewKind::Video,
        external_id: metadata.video_id.clone(),
        canonical_url: metadata.canonical_url.clone(),
        title: metadata.title.clone(),
        channel_title: metadata.channel_title.clone(),
        channel_id: metadata.channel_id.clone(),
        channel_handle: metadata.channel_handle.clone(),
        channel_url: metadata.channel_url.clone(),
        thumbnail_url: metadata.thumbnail_url.clone(),
        duration_seconds: metadata.duration_seconds,
        published_at: metadata.published_at.clone(),
        playlist_video_count: None,
        captions_estimate: captions_estimate(&metadata.raw_metadata_json),
        availability_status: metadata.availability_status.clone(),
        warnings: Vec::new(),
    }
}

pub(crate) fn playlist_preview_from_metadata(metadata: &YoutubePlaylistMetadata) -> YoutubePreview {
    let mut warnings = Vec::new();
    let previewed_count = metadata.items.len() as i64;
    if previewed_count >= 50
        && metadata
            .video_count
            .map_or(true, |video_count| video_count > previewed_count)
    {
        warnings.push("Preview only includes the first 50 playlist videos.".to_string());
    }

    YoutubePreview {
        kind: YoutubePreviewKind::Playlist,
        external_id: metadata.playlist_id.clone(),
        canonical_url: metadata.canonical_url.clone(),
        title: metadata.title.clone(),
        channel_title: metadata.channel_title.clone(),
        channel_id: metadata.channel_id.clone(),
        channel_handle: metadata.channel_handle.clone(),
        channel_url: metadata.channel_url.clone(),
        thumbnail_url: metadata.thumbnail_url.clone(),
        duration_seconds: None,
        published_at: None,
        playlist_video_count: metadata.video_count,
        captions_estimate: None,
        availability_status: metadata.availability_status.clone(),
        warnings,
    }
}

fn playlist_items(entries: &[Value]) -> Vec<YoutubePlaylistItemMetadata> {
    entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let video_id = first_string_field(entry, &["id", "url"])?;
            Some(YoutubePlaylistItemMetadata {
                video_id,
                position: i64_field(entry, "playlist_index").or(Some(index as i64 + 1)),
                title_snapshot: string_field(entry, "title"),
                url: first_string_field(entry, &["webpage_url", "url"]),
                thumbnail_url: thumbnail_url(entry),
                availability_status: availability_status(entry),
                raw_metadata_json: entry.clone(),
            })
        })
        .collect()
}

fn availability_status(value: &Value) -> YoutubeAvailabilityStatus {
    if matches!(
        string_field(value, "live_status").as_deref(),
        Some("is_upcoming")
    ) {
        return YoutubeAvailabilityStatus::Upcoming;
    }
    if matches!(
        string_field(value, "live_status").as_deref(),
        Some("is_live")
    ) {
        return YoutubeAvailabilityStatus::LiveNow;
    }
    if matches!(
        string_field(value, "live_status").as_deref(),
        Some("was_live")
    ) {
        return YoutubeAvailabilityStatus::LiveEndedTranscriptPending;
    }

    match string_field(value, "availability")
        .unwrap_or_else(|| "available".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "public" | "unlisted" | "available" => YoutubeAvailabilityStatus::Available,
        "private" | "needs_auth" | "login_required" => {
            YoutubeAvailabilityStatus::PrivateOrAuthRequired
        }
        "subscriber_only" | "members_only" | "premium_only" => {
            YoutubeAvailabilityStatus::MembersOnly
        }
        "age_restricted" => YoutubeAvailabilityStatus::AgeRestricted,
        "geo_restricted" | "geo_blocked" => YoutubeAvailabilityStatus::GeoBlocked,
        "deleted" | "removed" => YoutubeAvailabilityStatus::Deleted,
        "no_captions" => YoutubeAvailabilityStatus::NoCaptions,
        _ => YoutubeAvailabilityStatus::UnavailableUnknown,
    }
}

fn captions_estimate(value: &Value) -> Option<YoutubeCaptionsEstimate> {
    let manual_languages = object_keys(value.get("subtitles"));
    let auto_languages = object_keys(value.get("automatic_captions"));

    if manual_languages.is_empty() && auto_languages.is_empty() {
        return None;
    }

    let mut languages = manual_languages.clone();
    languages.extend(auto_languages.iter().cloned());
    languages.sort();
    languages.dedup();

    Some(YoutubeCaptionsEstimate {
        has_manual: !manual_languages.is_empty(),
        has_auto: !auto_languages.is_empty(),
        languages,
    })
}

fn chapters(value: &Value) -> Vec<YoutubeChapter> {
    value
        .get("chapters")
        .and_then(Value::as_array)
        .map(|chapters| {
            chapters
                .iter()
                .enumerate()
                .filter_map(|(index, chapter)| {
                    let title = string_field(chapter, "title")?;
                    Some(YoutubeChapter {
                        index: index as i64,
                        title,
                        start_ms: optional_seconds_to_ms(chapter.get("start_time")),
                        end_ms: chapter.get("end_time").map(seconds_to_ms),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn thumbnail_url(value: &Value) -> Option<String> {
    string_field(value, "thumbnail").or_else(|| {
        value
            .get("thumbnails")
            .and_then(Value::as_array)
            .and_then(|thumbnails| {
                thumbnails
                    .iter()
                    .rev()
                    .find_map(|item| string_field(item, "url"))
            })
    })
}

fn published_at(value: &Value) -> Option<String> {
    first_string_field(value, &["upload_date", "release_date"]).map(|date| {
        if date.len() == 8 && date.chars().all(|ch| ch.is_ascii_digit()) {
            format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8])
        } else {
            date
        }
    })
}

fn category(value: &Value) -> Option<String> {
    string_field(value, "category").or_else(|| {
        value
            .get("categories")
            .and_then(Value::as_array)
            .and_then(|categories| categories.iter().find_map(value_to_string))
    })
}

fn channel_handle(value: &Value) -> Option<String> {
    string_field(value, "channel_handle")
        .or_else(|| string_field(value, "uploader_id").filter(|id| id.starts_with('@')))
        .or_else(|| {
            first_string_field(value, &["channel_url", "uploader_url"])
                .and_then(|url| url.split("/@").nth(1).map(|handle| format!("@{handle}")))
        })
}

fn video_id_from_parsed(parsed: &YoutubeParsedUrl) -> Option<String> {
    match &parsed.kind {
        YoutubeUrlKind::Video { video_id }
        | YoutubeUrlKind::Short { video_id }
        | YoutubeUrlKind::Live { video_id } => Some(video_id.clone()),
        YoutubeUrlKind::Playlist { .. } => None,
    }
}

fn playlist_id_from_parsed(parsed: &YoutubeParsedUrl) -> Option<String> {
    match &parsed.kind {
        YoutubeUrlKind::Playlist { playlist_id } => Some(playlist_id.clone()),
        _ => None,
    }
}

fn first_string_field(value: &Value, fields: &[&str]) -> Option<String> {
    fields.iter().find_map(|field| string_field(value, field))
}

fn first_i64_field(value: &Value, fields: &[&str]) -> Option<i64> {
    fields.iter().find_map(|field| i64_field(value, field))
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(value_to_string)
}

fn value_to_string(value: &Value) -> Option<String> {
    value.as_str().and_then(|raw| {
        let trimmed = raw.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn string_array_field(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(value_to_string).collect())
        .unwrap_or_default()
}

fn object_keys(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_object)
        .map(|object| object.keys().cloned().collect())
        .unwrap_or_default()
}

fn i64_field(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(|value| {
        value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|number| i64::try_from(number).ok()))
            .or_else(|| value.as_f64().map(|number| number.round() as i64))
    })
}

fn seconds_to_ms(value: &Value) -> i64 {
    value
        .as_f64()
        .map(|seconds| (seconds * 1000.0).round() as i64)
        .or_else(|| value.as_i64().map(|seconds| seconds * 1000))
        .unwrap_or_default()
}

fn optional_seconds_to_ms(value: Option<&Value>) -> i64 {
    value.map(seconds_to_ms).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        playlist_metadata_from_ytdlp, playlist_preview_from_metadata, video_metadata_from_ytdlp,
        video_preview_from_metadata,
    };
    use crate::youtube::dto::{YoutubeAvailabilityStatus, YoutubeVideoForm};
    use crate::youtube::url::{parse_youtube_url, YoutubeParsedUrl};

    fn parsed(input: &str) -> YoutubeParsedUrl {
        parse_youtube_url(input).expect("parse youtube URL")
    }

    #[test]
    fn video_fixture_maps_metadata_and_preview_fields() {
        let fixture = json!({
            "id": "abc123",
            "webpage_url": "https://www.youtube.com/watch?v=abc123",
            "title": "Demo Video",
            "channel": "Demo Channel",
            "channel_id": "UCdemo",
            "channel_url": "https://www.youtube.com/@demo",
            "uploader_id": "@demo",
            "thumbnail": "https://img.youtube.com/vi/abc123/hqdefault.jpg",
            "duration": 120,
            "upload_date": "20260501",
            "availability": "public",
            "subtitles": { "en": [{ "ext": "vtt" }] },
            "automatic_captions": { "de": [{ "ext": "vtt" }] },
            "chapters": [
                { "title": "Intro", "start_time": 0, "end_time": 12.5 }
            ],
            "view_count": 42,
            "like_count": 7,
            "comment_count": 3,
            "categories": ["Education"],
            "tags": ["rust", "tauri"]
        });

        let metadata = video_metadata_from_ytdlp(
            fixture,
            &parsed("https://www.youtube.com/watch?v=abc123"),
            YoutubeVideoForm::Regular,
        )
        .expect("normalize video metadata");
        let preview = video_preview_from_metadata(&metadata);

        assert_eq!(metadata.video_id, "abc123");
        assert_eq!(
            metadata.canonical_url,
            "https://www.youtube.com/watch?v=abc123"
        );
        assert_eq!(metadata.title.as_deref(), Some("Demo Video"));
        assert_eq!(metadata.channel_title.as_deref(), Some("Demo Channel"));
        assert_eq!(metadata.channel_id.as_deref(), Some("UCdemo"));
        assert_eq!(
            metadata.channel_url.as_deref(),
            Some("https://www.youtube.com/@demo")
        );
        assert_eq!(metadata.channel_handle.as_deref(), Some("@demo"));
        assert_eq!(
            metadata.thumbnail_url.as_deref(),
            Some("https://img.youtube.com/vi/abc123/hqdefault.jpg")
        );
        assert_eq!(metadata.duration_seconds, Some(120));
        assert_eq!(metadata.published_at.as_deref(), Some("2026-05-01"));
        assert_eq!(
            metadata.availability_status,
            YoutubeAvailabilityStatus::Available
        );
        assert_eq!(metadata.tags, vec!["rust", "tauri"]);
        assert_eq!(metadata.chapters[0].start_ms, 0);
        assert_eq!(metadata.chapters[0].end_ms, Some(12_500));

        assert_eq!(preview.external_id, "abc123");
        assert_eq!(preview.title.as_deref(), Some("Demo Video"));
        assert_eq!(preview.duration_seconds, Some(120));
        assert_eq!(preview.published_at.as_deref(), Some("2026-05-01"));
        assert_eq!(
            preview.availability_status,
            YoutubeAvailabilityStatus::Available
        );
        assert_eq!(preview.captions_estimate.as_ref().unwrap().has_manual, true);
        assert_eq!(preview.captions_estimate.as_ref().unwrap().has_auto, true);
    }

    #[test]
    fn video_fixture_missing_optional_fields_maps_to_none() {
        let metadata = video_metadata_from_ytdlp(
            json!({ "id": "abc123", "availability": "public" }),
            &parsed("https://youtu.be/abc123"),
            YoutubeVideoForm::Regular,
        )
        .expect("normalize minimal video metadata");

        assert_eq!(metadata.video_id, "abc123");
        assert_eq!(metadata.title, None);
        assert_eq!(metadata.channel_title, None);
        assert_eq!(metadata.duration_seconds, None);
        assert_eq!(metadata.thumbnail_url, None);
        assert_eq!(metadata.tags, Vec::<String>::new());
        assert!(metadata.chapters.is_empty());
    }

    #[test]
    fn playlist_fixture_maps_metadata_entries_and_preview_warning() {
        let entries: Vec<_> = (1..=50)
            .map(|index| {
                json!({
                    "id": format!("video{index:02}"),
                    "title": format!("Video {index}"),
                    "url": format!("https://www.youtube.com/watch?v=video{index:02}"),
                    "playlist_index": index,
                    "availability": "public"
                })
            })
            .collect();
        let fixture = json!({
            "id": "PLabc123",
            "webpage_url": "https://www.youtube.com/playlist?list=PLabc123",
            "title": "Demo Playlist",
            "channel": "Demo Channel",
            "channel_id": "UCdemo",
            "channel_url": "https://www.youtube.com/@demo",
            "thumbnail": "https://img.youtube.com/playlist.jpg",
            "playlist_count": 75,
            "availability": "public",
            "entries": entries
        });

        let metadata = playlist_metadata_from_ytdlp(
            fixture,
            &parsed("https://www.youtube.com/playlist?list=PLabc123"),
        )
        .expect("normalize playlist metadata");
        let preview = playlist_preview_from_metadata(&metadata);

        assert_eq!(metadata.playlist_id, "PLabc123");
        assert_eq!(metadata.title.as_deref(), Some("Demo Playlist"));
        assert_eq!(metadata.channel_title.as_deref(), Some("Demo Channel"));
        assert_eq!(metadata.video_count, Some(75));
        assert_eq!(metadata.items.len(), 50);
        assert_eq!(metadata.items[0].video_id, "video01");
        assert_eq!(metadata.items[0].position, Some(1));
        assert_eq!(metadata.items[0].title_snapshot.as_deref(), Some("Video 1"));
        assert_eq!(
            metadata.availability_status,
            YoutubeAvailabilityStatus::Available
        );

        assert_eq!(preview.external_id, "PLabc123");
        assert_eq!(preview.playlist_video_count, Some(75));
        assert_eq!(preview.warnings.len(), 1);
    }

    #[test]
    fn availability_values_map_to_statuses() {
        let cases = [
            ("public", YoutubeAvailabilityStatus::Available),
            ("unlisted", YoutubeAvailabilityStatus::Available),
            ("private", YoutubeAvailabilityStatus::PrivateOrAuthRequired),
            (
                "needs_auth",
                YoutubeAvailabilityStatus::PrivateOrAuthRequired,
            ),
            ("subscriber_only", YoutubeAvailabilityStatus::MembersOnly),
            ("age_restricted", YoutubeAvailabilityStatus::AgeRestricted),
            ("geo_restricted", YoutubeAvailabilityStatus::GeoBlocked),
            ("deleted", YoutubeAvailabilityStatus::Deleted),
            ("unavailable", YoutubeAvailabilityStatus::UnavailableUnknown),
        ];

        for (raw, expected) in cases {
            let metadata = video_metadata_from_ytdlp(
                json!({ "id": "abc123", "availability": raw }),
                &parsed("https://www.youtube.com/watch?v=abc123"),
                YoutubeVideoForm::Regular,
            )
            .expect("normalize availability");

            assert_eq!(metadata.availability_status, expected);
        }
    }
}
