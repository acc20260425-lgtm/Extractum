#![allow(dead_code)]

use crate::error::AppResult;

use super::errors::invalid_youtube_url;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum YoutubeUrlKind {
    Video { video_id: String },
    Playlist { playlist_id: String },
    Short { video_id: String },
    Live { video_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct YoutubeParsedUrl {
    pub(crate) kind: YoutubeUrlKind,
    pub(crate) canonical_url: String,
    pub(crate) original_url: String,
}

pub(crate) fn parse_youtube_url(input: &str) -> AppResult<YoutubeParsedUrl> {
    let original_url = input.trim();
    if original_url.is_empty() {
        return Err(invalid_youtube_url("URL cannot be empty"));
    }

    let parsed = url::Url::parse(original_url).map_err(|_| invalid_youtube_url("invalid URL"))?;
    let host = parsed
        .host_str()
        .map(|host| host.to_ascii_lowercase())
        .ok_or_else(|| invalid_youtube_url("missing host"))?;

    if !is_supported_youtube_host(&host) {
        return Err(invalid_youtube_url(format!("unsupported host '{host}'")));
    }

    match host.as_str() {
        "youtu.be" => {
            let video_id = first_path_segment(&parsed)
                .ok_or_else(|| invalid_youtube_url("missing youtu.be video id"))?;
            Ok(parsed_video(video_id, original_url))
        }
        _ => parse_youtube_com_url(&parsed, original_url),
    }
}

fn parse_youtube_com_url(parsed: &url::Url, original_url: &str) -> AppResult<YoutubeParsedUrl> {
    let path = parsed.path();
    if path == "/watch" {
        if let Some(video_id) = query_param(parsed, "v") {
            return Ok(parsed_video(video_id, original_url));
        }
        if let Some(playlist_id) = query_param(parsed, "list") {
            return Ok(parsed_playlist(playlist_id, original_url));
        }
        return Err(invalid_youtube_url("missing watch video id"));
    }

    if let Some(video_id) = path_segment_after(parsed, "shorts") {
        return Ok(YoutubeParsedUrl {
            canonical_url: format!("https://www.youtube.com/shorts/{video_id}"),
            original_url: original_url.to_string(),
            kind: YoutubeUrlKind::Short { video_id },
        });
    }

    if let Some(video_id) = path_segment_after(parsed, "live") {
        return Ok(YoutubeParsedUrl {
            canonical_url: format!("https://www.youtube.com/live/{video_id}"),
            original_url: original_url.to_string(),
            kind: YoutubeUrlKind::Live { video_id },
        });
    }

    if let Some(playlist_id) = query_param(parsed, "list") {
        return Ok(parsed_playlist(playlist_id, original_url));
    }

    Err(invalid_youtube_url("unsupported YouTube URL shape"))
}

fn is_supported_youtube_host(host: &str) -> bool {
    host == "youtu.be" || host == "youtube.com" || host.ends_with(".youtube.com")
}

fn query_param(parsed: &url::Url, name: &str) -> Option<String> {
    parsed
        .query_pairs()
        .find(|(key, value)| key == name && !value.trim().is_empty())
        .map(|(_, value)| value.into_owned())
}

fn first_path_segment(parsed: &url::Url) -> Option<String> {
    parsed
        .path_segments()?
        .find(|segment| !segment.trim().is_empty())
        .map(str::to_string)
}

fn path_segment_after(parsed: &url::Url, prefix: &str) -> Option<String> {
    let mut segments = parsed.path_segments()?;
    if segments.next()? != prefix {
        return None;
    }

    segments
        .find(|segment| !segment.trim().is_empty())
        .map(str::to_string)
}

fn parsed_video(video_id: String, original_url: &str) -> YoutubeParsedUrl {
    YoutubeParsedUrl {
        canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
        original_url: original_url.to_string(),
        kind: YoutubeUrlKind::Video { video_id },
    }
}

fn parsed_playlist(playlist_id: String, original_url: &str) -> YoutubeParsedUrl {
    YoutubeParsedUrl {
        canonical_url: format!("https://www.youtube.com/playlist?list={playlist_id}"),
        original_url: original_url.to_string(),
        kind: YoutubeUrlKind::Playlist { playlist_id },
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_youtube_url, YoutubeParsedUrl, YoutubeUrlKind};

    fn parse(input: &str) -> YoutubeParsedUrl {
        parse_youtube_url(input).expect("parse youtube URL")
    }

    #[test]
    fn parses_watch_video_url() {
        let parsed = parse("https://www.youtube.com/watch?v=abc123");

        assert_eq!(
            parsed.kind,
            YoutubeUrlKind::Video {
                video_id: "abc123".to_string()
            }
        );
        assert_eq!(
            parsed.canonical_url,
            "https://www.youtube.com/watch?v=abc123"
        );
        assert_eq!(
            parsed.original_url,
            "https://www.youtube.com/watch?v=abc123"
        );
    }

    #[test]
    fn parses_short_youtu_be_url() {
        let parsed = parse("https://youtu.be/abc123?t=42");

        assert_eq!(
            parsed.kind,
            YoutubeUrlKind::Video {
                video_id: "abc123".to_string()
            }
        );
        assert_eq!(
            parsed.canonical_url,
            "https://www.youtube.com/watch?v=abc123"
        );
    }

    #[test]
    fn parses_playlist_url() {
        let parsed = parse("https://www.youtube.com/playlist?list=PLabc123");

        assert_eq!(
            parsed.kind,
            YoutubeUrlKind::Playlist {
                playlist_id: "PLabc123".to_string()
            }
        );
        assert_eq!(
            parsed.canonical_url,
            "https://www.youtube.com/playlist?list=PLabc123"
        );
    }

    #[test]
    fn parses_shorts_url() {
        let parsed = parse("https://youtube.com/shorts/abc123");

        assert_eq!(
            parsed.kind,
            YoutubeUrlKind::Short {
                video_id: "abc123".to_string()
            }
        );
        assert_eq!(
            parsed.canonical_url,
            "https://www.youtube.com/shorts/abc123"
        );
    }

    #[test]
    fn parses_live_url() {
        let parsed = parse("https://www.youtube.com/live/abc123");

        assert_eq!(
            parsed.kind,
            YoutubeUrlKind::Live {
                video_id: "abc123".to_string()
            }
        );
        assert_eq!(parsed.canonical_url, "https://www.youtube.com/live/abc123");
    }

    #[test]
    fn watch_url_with_playlist_parameter_parses_selected_video() {
        let parsed = parse("https://www.youtube.com/watch?v=video123&list=PLabc123");

        assert_eq!(
            parsed.kind,
            YoutubeUrlKind::Video {
                video_id: "video123".to_string()
            }
        );
        assert_eq!(
            parsed.canonical_url,
            "https://www.youtube.com/watch?v=video123"
        );
    }

    #[test]
    fn rejects_invalid_host() {
        let error = parse_youtube_url("https://example.com/watch?v=abc123")
            .expect_err("reject unsupported host");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    }

    #[test]
    fn rejects_empty_input() {
        let error = parse_youtube_url("   ").expect_err("reject empty input");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    }
}
