use serde_json::Value;
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::secret_store::SecretStoreState;
use crate::sources::{
    load_source_record, upsert_youtube_playlist_source, upsert_youtube_video_source, SourceRecord,
};

use super::dto::{YoutubePlaylistMetadata, YoutubePreview, YoutubeVideoForm, YoutubeVideoMetadata};
use super::metadata::{
    playlist_metadata_from_ytdlp, playlist_preview_from_metadata, video_metadata_from_ytdlp,
    video_preview_from_metadata,
};
use super::playlist::upsert_playlist_items;
use super::settings::load_youtube_auth_cookies_from_state;
use super::url::{parse_youtube_url, YoutubeParsedUrl, YoutubeUrlKind};
use super::ytdlp::{
    preview_playlist_args, preview_video_args, run_ytdlp_with_options, YtdlpRunOptions,
    YTDLP_PREVIEW_TIMEOUT,
};

pub(crate) enum YoutubeFetchedMetadata {
    Video(YoutubeVideoMetadata),
    Playlist(YoutubePlaylistMetadata),
}

#[tauri::command]
pub async fn preview_youtube_source(
    handle: AppHandle,
    secrets: tauri::State<'_, SecretStoreState>,
    url: String,
) -> AppResult<YoutubePreview> {
    let parsed = parse_youtube_url(&url)?;
    let pool = get_pool(&handle).await?;
    let cookies = load_youtube_auth_cookies_from_state(&pool, &secrets).await?;
    fetch_preview(parsed, cookies).await
}

#[tauri::command]
pub async fn add_youtube_source(
    handle: AppHandle,
    secrets: tauri::State<'_, SecretStoreState>,
    url: String,
) -> AppResult<SourceRecord> {
    let parsed = parse_youtube_url(&url)?;
    let pool = get_pool(&handle).await?;
    let cookies = load_youtube_auth_cookies_from_state(&pool, &secrets).await?;
    let metadata = fetch_metadata(parsed, cookies).await?;
    let mut tx = pool.begin().await.map_err(|e| AppError::database(e))?;

    let source_id = match metadata {
        YoutubeFetchedMetadata::Video(metadata) => {
            upsert_youtube_video_source(&mut tx, &metadata).await?
        }
        YoutubeFetchedMetadata::Playlist(metadata) => {
            let playlist_source_id = upsert_youtube_playlist_source(&mut tx, &metadata).await?;
            upsert_playlist_items(&mut tx, playlist_source_id, &metadata).await?;
            playlist_source_id
        }
    };

    tx.commit().await.map_err(|e| AppError::database(e))?;
    load_source_record(&handle, &pool, source_id).await
}

pub(crate) async fn fetch_preview(
    parsed: YoutubeParsedUrl,
    cookies: Option<String>,
) -> AppResult<YoutubePreview> {
    let metadata = fetch_metadata(parsed, cookies).await?;

    Ok(match metadata {
        YoutubeFetchedMetadata::Video(metadata) => video_preview_from_metadata(&metadata),
        YoutubeFetchedMetadata::Playlist(metadata) => playlist_preview_from_metadata(&metadata),
    })
}

pub(crate) async fn fetch_metadata(
    parsed: YoutubeParsedUrl,
    cookies: Option<String>,
) -> AppResult<YoutubeFetchedMetadata> {
    let args = match parsed.kind {
        YoutubeUrlKind::Playlist { .. } => preview_playlist_args(&parsed.canonical_url),
        YoutubeUrlKind::Video { .. }
        | YoutubeUrlKind::Short { .. }
        | YoutubeUrlKind::Live { .. } => preview_video_args(&parsed.canonical_url),
    };
    let output = run_ytdlp_with_options(
        &args,
        YtdlpRunOptions {
            timeout: YTDLP_PREVIEW_TIMEOUT,
            cookies,
        },
    )
    .await?;
    let json = ytdlp_stdout_json(&output.stdout)?;

    metadata_from_ytdlp_json(&parsed, json)
}

pub(crate) fn preview_from_ytdlp_json(
    parsed: &YoutubeParsedUrl,
    value: Value,
) -> AppResult<YoutubePreview> {
    let metadata = metadata_from_ytdlp_json(parsed, value)?;

    Ok(match metadata {
        YoutubeFetchedMetadata::Video(metadata) => video_preview_from_metadata(&metadata),
        YoutubeFetchedMetadata::Playlist(metadata) => playlist_preview_from_metadata(&metadata),
    })
}

pub(crate) fn metadata_from_ytdlp_json(
    parsed: &YoutubeParsedUrl,
    value: Value,
) -> AppResult<YoutubeFetchedMetadata> {
    match parsed.kind {
        YoutubeUrlKind::Playlist { .. } => {
            let metadata = playlist_metadata_from_ytdlp(value, parsed)?;
            Ok(YoutubeFetchedMetadata::Playlist(metadata))
        }
        YoutubeUrlKind::Video { .. }
        | YoutubeUrlKind::Short { .. }
        | YoutubeUrlKind::Live { .. } => {
            let metadata = video_metadata_from_ytdlp(value, parsed, video_form(parsed))?;
            Ok(YoutubeFetchedMetadata::Video(metadata))
        }
    }
}

fn ytdlp_stdout_json(stdout: &str) -> AppResult<Value> {
    serde_json::from_str(stdout.trim())
        .map_err(|error| AppError::validation(format!("yt-dlp returned invalid JSON: {error}")))
}

fn video_form(parsed: &YoutubeParsedUrl) -> YoutubeVideoForm {
    match parsed.kind {
        YoutubeUrlKind::Short { .. } => YoutubeVideoForm::Short,
        YoutubeUrlKind::Live { .. } => YoutubeVideoForm::Live,
        YoutubeUrlKind::Video { .. } | YoutubeUrlKind::Playlist { .. } => YoutubeVideoForm::Regular,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::preview_from_ytdlp_json;
    use crate::youtube::dto::{YoutubeAvailabilityStatus, YoutubePreviewKind};
    use crate::youtube::url::parse_youtube_url;

    #[test]
    fn preview_from_video_json_uses_parsed_url_kind() {
        let parsed =
            parse_youtube_url("https://www.youtube.com/shorts/abc123").expect("parse youtube URL");

        let preview = preview_from_ytdlp_json(
            &parsed,
            json!({
                "id": "abc123",
                "title": "Short Demo",
                "duration": 45,
                "availability": "public"
            }),
        )
        .expect("build preview");

        assert_eq!(preview.kind, YoutubePreviewKind::Video);
        assert_eq!(preview.external_id, "abc123");
        assert_eq!(
            preview.canonical_url,
            "https://www.youtube.com/shorts/abc123"
        );
        assert_eq!(
            preview.availability_status,
            YoutubeAvailabilityStatus::Available
        );
    }

    #[test]
    fn preview_from_playlist_json_returns_playlist_preview() {
        let parsed = parse_youtube_url("https://www.youtube.com/playlist?list=PLabc123")
            .expect("parse youtube URL");

        let preview = preview_from_ytdlp_json(
            &parsed,
            json!({
                "id": "PLabc123",
                "title": "Playlist Demo",
                "playlist_count": 1,
                "availability": "public",
                "entries": [{ "id": "video01", "title": "Video 1" }]
            }),
        )
        .expect("build playlist preview");

        assert_eq!(preview.kind, YoutubePreviewKind::Playlist);
        assert_eq!(preview.external_id, "PLabc123");
        assert_eq!(preview.playlist_video_count, Some(1));
    }
}
