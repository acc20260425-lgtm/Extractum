use serde_json::Value;

use crate::error::{AppError, AppResult};

use super::dto::{YoutubePreview, YoutubeVideoForm};
use super::metadata::{
    playlist_metadata_from_ytdlp, playlist_preview_from_metadata, video_metadata_from_ytdlp,
    video_preview_from_metadata,
};
use super::url::{parse_youtube_url, YoutubeParsedUrl, YoutubeUrlKind};
use super::ytdlp::{preview_playlist_args, preview_video_args, run_ytdlp};

#[tauri::command]
pub async fn preview_youtube_source(url: String) -> AppResult<YoutubePreview> {
    let parsed = parse_youtube_url(&url)?;
    fetch_preview(parsed).await
}

pub(crate) async fn fetch_preview(parsed: YoutubeParsedUrl) -> AppResult<YoutubePreview> {
    let args = match parsed.kind {
        YoutubeUrlKind::Playlist { .. } => preview_playlist_args(&parsed.canonical_url),
        YoutubeUrlKind::Video { .. }
        | YoutubeUrlKind::Short { .. }
        | YoutubeUrlKind::Live { .. } => preview_video_args(&parsed.canonical_url),
    };
    let output = run_ytdlp(&args).await?;
    let json = ytdlp_stdout_json(&output.stdout)?;

    preview_from_ytdlp_json(&parsed, json)
}

pub(crate) fn preview_from_ytdlp_json(
    parsed: &YoutubeParsedUrl,
    value: Value,
) -> AppResult<YoutubePreview> {
    match parsed.kind {
        YoutubeUrlKind::Playlist { .. } => {
            let metadata = playlist_metadata_from_ytdlp(value, parsed)?;
            Ok(playlist_preview_from_metadata(&metadata))
        }
        YoutubeUrlKind::Video { .. }
        | YoutubeUrlKind::Short { .. }
        | YoutubeUrlKind::Live { .. } => {
            let metadata = video_metadata_from_ytdlp(value, parsed, video_form(parsed))?;
            Ok(video_preview_from_metadata(&metadata))
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
