use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use crate::compression::compress_json_bytes;
use crate::error::{AppError, AppResult};
use crate::external_process::ExternalProcessShutdownState;

use super::dto::{
    YoutubeCaptionTrack, YoutubeCaptionTrackKind, YoutubeTranscript, YoutubeTranscriptSegment,
    YoutubeVideoMetadata,
};
use super::process_runtime::YoutubeProcessRegistry;
use super::ytdlp::{run_ytdlp_with_options, YtdlpRunOptions, YTDLP_PREVIEW_TIMEOUT};

pub(crate) const YOUTUBE_CAPTION_DOWNLOAD_TIMEOUT: Duration = YTDLP_PREVIEW_TIMEOUT;

pub(crate) async fn fetch_transcript_for_video(
    registry: &YoutubeProcessRegistry,
    shutdown: &ExternalProcessShutdownState,
    metadata: &YoutubeVideoMetadata,
    preferred_language: Option<&str>,
    override_language: Option<&str>,
    cookies: Option<String>,
) -> AppResult<YoutubeTranscript> {
    let tracks = caption_tracks_from_metadata(metadata);
    let track = select_caption_track(
        &tracks,
        override_language,
        preferred_language,
        original_language(metadata).as_deref(),
    )
    .ok_or_else(|| AppError::validation("YouTube video has no captions"))?;
    let language = track
        .language
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::validation("Selected YouTube caption track has no language"))?;

    let temp_dir = tempfile::TempDir::new()
        .map_err(|error| AppError::internal(format!("Failed to create temp dir: {error}")))?;
    let output_template = temp_dir.path().join("%(id)s.%(ext)s");
    let output_template = output_template.to_string_lossy().to_string();
    let args = caption_download_args(&metadata.canonical_url, language, &output_template);
    run_ytdlp_with_options(
        registry,
        shutdown,
        &args,
        YtdlpRunOptions {
            timeout: YOUTUBE_CAPTION_DOWNLOAD_TIMEOUT,
            cookies,
            cancellation: None,
        },
    )
    .await?;

    let caption_file = find_caption_payload(temp_dir.path())?;
    let payload = fs::read_to_string(&caption_file)
        .map_err(|error| AppError::internal(format!("Failed to read captions: {error}")))?;
    match caption_file.extension().and_then(|value| value.to_str()) {
        Some("json3") => parse_json3_transcript(
            &metadata.video_id,
            track.language.clone(),
            track.track_kind,
            &payload,
        ),
        Some("vtt") => parse_vtt_transcript(
            &metadata.video_id,
            track.language.clone(),
            track.track_kind,
            &payload,
        ),
        _ => Err(AppError::validation(
            "yt-dlp did not produce a supported caption payload",
        )),
    }
}

pub(crate) fn caption_tracks_from_metadata(
    metadata: &YoutubeVideoMetadata,
) -> Vec<YoutubeCaptionTrack> {
    let mut tracks = Vec::new();
    tracks.extend(caption_tracks_from_object(
        metadata.raw_metadata_json.get("subtitles"),
        YoutubeCaptionTrackKind::Manual,
    ));
    tracks.extend(caption_tracks_from_object(
        metadata.raw_metadata_json.get("automatic_captions"),
        YoutubeCaptionTrackKind::Auto,
    ));
    tracks
}

pub(crate) fn select_caption_track(
    tracks: &[YoutubeCaptionTrack],
    override_language: Option<&str>,
    preferred_language: Option<&str>,
    original_language: Option<&str>,
) -> Option<YoutubeCaptionTrack> {
    if let Some(language) = normalized_language(override_language) {
        if let Some(track) = find_track(
            tracks,
            Some(language),
            Some(YoutubeCaptionTrackKind::Manual),
        )
        .or_else(|| find_track(tracks, Some(language), Some(YoutubeCaptionTrackKind::Auto)))
        {
            return Some(track);
        }
    }

    if let Some(language) = normalized_language(original_language) {
        if let Some(track) = find_track(
            tracks,
            Some(language),
            Some(YoutubeCaptionTrackKind::Manual),
        )
        .or_else(|| find_track(tracks, Some(language), Some(YoutubeCaptionTrackKind::Auto)))
        {
            return Some(track);
        }
    }

    if let Some(language) =
        normalized_language(preferred_language).filter(|language| *language != "original")
    {
        if let Some(track) = find_track(
            tracks,
            Some(language),
            Some(YoutubeCaptionTrackKind::Manual),
        )
        .or_else(|| find_track(tracks, Some(language), Some(YoutubeCaptionTrackKind::Auto)))
        {
            return Some(track);
        }
    }

    find_track(tracks, Some("en"), Some(YoutubeCaptionTrackKind::Manual))
        .or_else(|| find_track(tracks, Some("en"), Some(YoutubeCaptionTrackKind::Auto)))
        .or_else(|| find_track(tracks, None, Some(YoutubeCaptionTrackKind::Manual)))
        .or_else(|| find_track(tracks, None, Some(YoutubeCaptionTrackKind::Auto)))
}

pub(crate) fn caption_download_args(
    canonical_url: &str,
    language: &str,
    output_template: &str,
) -> Vec<String> {
    vec![
        "--skip-download".to_string(),
        "--write-subs".to_string(),
        "--write-auto-subs".to_string(),
        "--sub-langs".to_string(),
        language.to_string(),
        "--sub-format".to_string(),
        "json3/vtt".to_string(),
        "--output".to_string(),
        output_template.to_string(),
        canonical_url.to_string(),
    ]
}

pub(crate) fn parse_json3_transcript(
    video_id: &str,
    language: Option<String>,
    track_kind: YoutubeCaptionTrackKind,
    payload: &str,
) -> AppResult<YoutubeTranscript> {
    let raw_payload: Value = serde_json::from_str(payload)
        .map_err(|error| AppError::validation(format!("Invalid json3 captions: {error}")))?;
    let events = raw_payload
        .get("events")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::validation("json3 captions are missing events"))?;
    let mut segments = Vec::new();

    for event in events {
        let Some(start_ms) = event.get("tStartMs").and_then(Value::as_i64) else {
            continue;
        };
        let text = event
            .get("segs")
            .and_then(Value::as_array)
            .map(|segs| {
                segs.iter()
                    .filter_map(|seg| seg.get("utf8").and_then(Value::as_str))
                    .collect::<String>()
            })
            .unwrap_or_default();
        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }
        let end_ms = event
            .get("dDurationMs")
            .and_then(Value::as_i64)
            .map(|duration| start_ms + duration);
        segments.push(YoutubeTranscriptSegment {
            index: segments.len() as i64,
            start_ms,
            end_ms,
            text,
            chapter_index: None,
        });
    }

    Ok(YoutubeTranscript {
        video_id: video_id.to_string(),
        language,
        is_auto_generated: matches!(track_kind, YoutubeCaptionTrackKind::Auto),
        track_kind,
        segments,
        raw_payload,
    })
}

pub(crate) fn parse_vtt_transcript(
    video_id: &str,
    language: Option<String>,
    track_kind: YoutubeCaptionTrackKind,
    payload: &str,
) -> AppResult<YoutubeTranscript> {
    let mut segments = Vec::new();
    let mut lines = payload.lines().peekable();

    while let Some(line) = lines.next() {
        let line = line.trim();
        if !line.contains("-->") {
            continue;
        }
        let (start_ms, end_ms) = parse_vtt_timing(line)?;
        let mut text_lines = Vec::new();
        while let Some(next) = lines.peek().copied() {
            if next.trim().is_empty() {
                lines.next();
                break;
            }
            text_lines.push(lines.next().unwrap_or_default().trim());
        }
        let text = text_lines.join("\n").trim().to_string();
        if text.is_empty() {
            continue;
        }
        segments.push(YoutubeTranscriptSegment {
            index: segments.len() as i64,
            start_ms,
            end_ms: Some(end_ms),
            text,
            chapter_index: None,
        });
    }

    Ok(YoutubeTranscript {
        video_id: video_id.to_string(),
        language,
        is_auto_generated: matches!(track_kind, YoutubeCaptionTrackKind::Auto),
        track_kind,
        segments,
        raw_payload: Value::String(payload.to_string()),
    })
}

pub(crate) fn transcript_external_id(
    video_id: &str,
    language: Option<&str>,
    track_kind: &YoutubeCaptionTrackKind,
) -> String {
    let language = language
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("und")
        .replace(':', "_");
    let kind = match track_kind {
        YoutubeCaptionTrackKind::Manual => "manual",
        YoutubeCaptionTrackKind::Auto => "auto",
        YoutubeCaptionTrackKind::Unknown => "unknown",
    };
    format!("transcript:{video_id}:{language}:{kind}")
}

pub(crate) async fn replace_transcript_segments(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    item_id: i64,
    source_id: i64,
    transcript: &YoutubeTranscript,
) -> AppResult<()> {
    sqlx::query("DELETE FROM youtube_transcript_segments WHERE item_id = ?")
        .bind(item_id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;

    for (index, segment) in transcript.segments.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO youtube_transcript_segments (
                item_id,
                source_id,
                segment_index,
                start_ms,
                end_ms,
                text,
                chapter_index,
                caption_language,
                caption_track_kind,
                is_auto_generated,
                metadata_zstd
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(item_id)
        .bind(source_id)
        .bind(index as i64)
        .bind(segment.start_ms)
        .bind(segment.end_ms)
        .bind(&segment.text)
        .bind(segment.chapter_index)
        .bind(&transcript.language)
        .bind(caption_track_kind_wire(&transcript.track_kind))
        .bind(transcript.is_auto_generated)
        .bind(segment_metadata_zstd(segment)?)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    crate::analysis_documents::rebuild_youtube_transcript_documents_for_item_on_connection(
        &mut **tx, item_id,
    )
    .await?;

    Ok(())
}

fn parse_vtt_timing(line: &str) -> AppResult<(i64, i64)> {
    let Some((start, end_and_settings)) = line.split_once("-->") else {
        return Err(AppError::validation("Invalid VTT cue timing"));
    };
    let end = end_and_settings
        .split_whitespace()
        .next()
        .ok_or_else(|| AppError::validation("Invalid VTT cue end timing"))?;
    let start_ms = parse_vtt_timestamp(start.trim())?;
    let end_ms = parse_vtt_timestamp(end.trim())?;
    if end_ms < start_ms {
        return Err(AppError::validation("Invalid VTT cue timing range"));
    }
    Ok((start_ms, end_ms))
}

fn parse_vtt_timestamp(value: &str) -> AppResult<i64> {
    let mut parts = value.split(':');
    let hours = parse_i64_part(parts.next(), "hour")?;
    let minutes = parse_i64_part(parts.next(), "minute")?;
    let seconds_and_ms = parts
        .next()
        .ok_or_else(|| AppError::validation("Invalid VTT timestamp seconds"))?;
    if parts.next().is_some() {
        return Err(AppError::validation("Invalid VTT timestamp"));
    }
    let Some((seconds, millis)) = seconds_and_ms.split_once('.') else {
        return Err(AppError::validation("Invalid VTT timestamp milliseconds"));
    };
    let seconds = parse_digits(seconds, "second")?;
    let millis = parse_digits(millis, "millisecond")?;
    if millis > 999 {
        return Err(AppError::validation("Invalid VTT timestamp milliseconds"));
    }
    Ok(hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + millis)
}

fn parse_i64_part(value: Option<&str>, name: &str) -> AppResult<i64> {
    parse_digits(
        value.ok_or_else(|| AppError::validation(format!("Invalid VTT timestamp {name}")))?,
        name,
    )
}

fn parse_digits(value: &str, name: &str) -> AppResult<i64> {
    if value.is_empty() || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(AppError::validation(format!(
            "Invalid VTT timestamp {name}"
        )));
    }
    value
        .parse::<i64>()
        .map_err(|error| AppError::validation(format!("Invalid VTT timestamp {name}: {error}")))
}

fn caption_track_kind_wire(track_kind: &YoutubeCaptionTrackKind) -> &'static str {
    match track_kind {
        YoutubeCaptionTrackKind::Manual => "manual",
        YoutubeCaptionTrackKind::Auto => "auto",
        YoutubeCaptionTrackKind::Unknown => "unknown",
    }
}

fn segment_metadata_zstd(segment: &YoutubeTranscriptSegment) -> AppResult<Vec<u8>> {
    let json =
        serde_json::to_vec(segment).map_err(|error| AppError::internal(error.to_string()))?;
    compress_json_bytes(&json).map_err(AppError::internal)
}

fn caption_tracks_from_object(
    value: Option<&Value>,
    track_kind: YoutubeCaptionTrackKind,
) -> Vec<YoutubeCaptionTrack> {
    let Some(object) = value.and_then(Value::as_object) else {
        return Vec::new();
    };
    object
        .iter()
        .map(|(language, tracks)| YoutubeCaptionTrack {
            language: Some(language.clone()),
            name: tracks
                .as_array()
                .and_then(|items| items.first())
                .and_then(|item| item.get("name"))
                .and_then(Value::as_str)
                .map(str::to_string),
            is_auto_generated: matches!(track_kind, YoutubeCaptionTrackKind::Auto),
            track_kind: track_kind.clone(),
        })
        .collect()
}

fn find_track(
    tracks: &[YoutubeCaptionTrack],
    language: Option<&str>,
    track_kind: Option<YoutubeCaptionTrackKind>,
) -> Option<YoutubeCaptionTrack> {
    tracks
        .iter()
        .find(|track| {
            language.is_none_or(|language| {
                track
                    .language
                    .as_deref()
                    .is_some_and(|candidate| language_matches(candidate, language))
            }) && track_kind
                .as_ref()
                .is_none_or(|kind| track.track_kind == *kind)
        })
        .cloned()
}

fn language_matches(candidate: &str, expected: &str) -> bool {
    let candidate = candidate.to_ascii_lowercase();
    let expected = expected.to_ascii_lowercase();
    candidate == expected || candidate.starts_with(&format!("{expected}-"))
}

fn normalized_language(language: Option<&str>) -> Option<&str> {
    language.map(str::trim).filter(|value| !value.is_empty())
}

fn original_language(metadata: &YoutubeVideoMetadata) -> Option<String> {
    metadata
        .raw_metadata_json
        .get("language")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn find_caption_payload(dir: &Path) -> AppResult<PathBuf> {
    let mut json3 = Vec::new();
    let mut vtt = Vec::new();
    for entry in fs::read_dir(dir)
        .map_err(|error| AppError::internal(format!("Failed to inspect captions: {error}")))?
    {
        let path = entry
            .map_err(|error| AppError::internal(format!("Failed to inspect captions: {error}")))?
            .path();
        match path.extension().and_then(|value| value.to_str()) {
            Some("json3") => json3.push(path),
            Some("vtt") => vtt.push(path),
            _ => {}
        }
    }
    json3
        .into_iter()
        .next()
        .or_else(|| vtt.into_iter().next())
        .ok_or_else(|| AppError::validation("yt-dlp did not produce captions"))
}

#[cfg(test)]
mod tests {
    use super::{
        caption_download_args, parse_json3_transcript, parse_vtt_transcript,
        replace_transcript_segments, select_caption_track, transcript_external_id,
    };
    use crate::error::AppErrorKind;
    use crate::youtube::dto::{
        YoutubeCaptionTrack, YoutubeCaptionTrackKind, YoutubeTranscript, YoutubeTranscriptSegment,
    };

    #[test]
    fn json3_parser_concatenates_segments_and_preserves_timing() {
        let transcript = parse_json3_transcript(
            "video01",
            Some("en".to_string()),
            YoutubeCaptionTrackKind::Manual,
            r#"{
              "events": [
                {
                  "tStartMs": 1200,
                  "dDurationMs": 2300,
                  "segs": [
                    { "utf8": "hello" },
                    { "utf8": " " },
                    { "utf8": "world" }
                  ]
                }
              ]
            }"#,
        )
        .expect("parse json3");

        assert_eq!(transcript.video_id, "video01");
        assert_eq!(transcript.language.as_deref(), Some("en"));
        assert_eq!(transcript.track_kind, YoutubeCaptionTrackKind::Manual);
        assert_eq!(transcript.segments.len(), 1);
        assert_eq!(transcript.segments[0].text, "hello world");
        assert_eq!(transcript.segments[0].start_ms, 1200);
        assert_eq!(transcript.segments[0].end_ms, Some(3500));
    }

    #[test]
    fn json3_parser_allows_missing_duration() {
        let transcript = parse_json3_transcript(
            "video01",
            None,
            YoutubeCaptionTrackKind::Auto,
            r#"{"events":[{"tStartMs":5000,"segs":[{"utf8":"line"}]}]}"#,
        )
        .expect("parse json3");

        assert_eq!(transcript.language, None);
        assert_eq!(transcript.track_kind, YoutubeCaptionTrackKind::Auto);
        assert!(transcript.is_auto_generated);
        assert_eq!(transcript.segments[0].start_ms, 5000);
        assert_eq!(transcript.segments[0].end_ms, None);
    }

    #[test]
    fn vtt_parser_reads_cues_and_skips_blank_text() {
        let transcript = parse_vtt_transcript(
            "video01",
            Some("en".to_string()),
            YoutubeCaptionTrackKind::Manual,
            "WEBVTT\n\n00:00:01.000 --> 00:00:03.500\nFirst line\n\n00:00:04.000 --> 00:00:05.000\n   \n\n00:12:34.000 --> 00:12:37.500\nSecond line\n",
        )
        .expect("parse vtt");

        assert_eq!(transcript.segments.len(), 2);
        assert_eq!(transcript.segments[0].start_ms, 1000);
        assert_eq!(transcript.segments[0].end_ms, Some(3500));
        assert_eq!(transcript.segments[0].text, "First line");
        assert_eq!(transcript.segments[1].start_ms, 754_000);
        assert_eq!(transcript.segments[1].end_ms, Some(757_500));
        assert_eq!(transcript.segments[1].text, "Second line");
    }

    #[test]
    fn vtt_parser_rejects_invalid_timing() {
        let error = parse_vtt_transcript(
            "video01",
            Some("en".to_string()),
            YoutubeCaptionTrackKind::Manual,
            "WEBVTT\n\n00:00:05.000 --> 00:00:03.500\nBad\n",
        )
        .expect_err("invalid timing should fail");

        assert_eq!(error.kind, AppErrorKind::Validation);
    }

    #[test]
    fn transcript_external_id_includes_language_and_track_kind() {
        assert_eq!(
            transcript_external_id("video01", Some("en:US"), &YoutubeCaptionTrackKind::Manual),
            "transcript:video01:en_US:manual"
        );
        assert_eq!(
            transcript_external_id("video01", None, &YoutubeCaptionTrackKind::Auto),
            "transcript:video01:und:auto"
        );
    }

    #[test]
    fn caption_selection_prefers_original_then_preferred_then_english_then_any() {
        let tracks = vec![
            track("de", YoutubeCaptionTrackKind::Manual),
            track("en", YoutubeCaptionTrackKind::Auto),
            track("fr", YoutubeCaptionTrackKind::Auto),
        ];

        assert_eq!(
            select_caption_track(&tracks, None, Some("fr"), Some("de"))
                .unwrap()
                .language
                .as_deref(),
            Some("de")
        );
        assert_eq!(
            select_caption_track(&tracks, None, Some("fr"), Some("it"))
                .unwrap()
                .language
                .as_deref(),
            Some("fr")
        );
        assert_eq!(
            select_caption_track(&tracks, None, Some("original"), Some("it"))
                .unwrap()
                .language
                .as_deref(),
            Some("en")
        );
    }

    #[test]
    fn caption_selection_honors_explicit_override_before_original_language() {
        let tracks = vec![
            track("de", YoutubeCaptionTrackKind::Manual),
            track("fr", YoutubeCaptionTrackKind::Manual),
        ];

        assert_eq!(
            select_caption_track(&tracks, Some("fr"), Some("de"), Some("de"))
                .unwrap()
                .language
                .as_deref(),
            Some("fr")
        );
    }

    #[test]
    fn caption_download_args_request_json3_and_vtt_without_media() {
        let args = caption_download_args(
            "https://www.youtube.com/watch?v=video01",
            "en",
            "C:\\Temp\\caps\\%(id)s.%(ext)s",
        );

        assert_eq!(
            args,
            vec![
                "--skip-download",
                "--write-subs",
                "--write-auto-subs",
                "--sub-langs",
                "en",
                "--sub-format",
                "json3/vtt",
                "--output",
                "C:\\Temp\\caps\\%(id)s.%(ext)s",
                "https://www.youtube.com/watch?v=video01",
            ]
        );
    }

    #[tokio::test]
    async fn replace_transcript_segments_deletes_previous_rows_and_inserts_current_segments() {
        let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;
        crate::sources::test_support::create_analysis_documents_table(&pool).await;
        sqlx::query(
            r#"
            CREATE TABLE youtube_transcript_segments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
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
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create segments table");
        sqlx::query(
            r#"
            INSERT INTO youtube_transcript_segments (
                item_id,
                source_id,
                segment_index,
                start_ms,
                text,
                is_auto_generated
            )
            VALUES (9, 1, 0, 1, 'old', 0)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert old segment");

        let transcript = YoutubeTranscript {
            video_id: "video01".to_string(),
            language: Some("en".to_string()),
            track_kind: YoutubeCaptionTrackKind::Auto,
            is_auto_generated: true,
            segments: vec![
                YoutubeTranscriptSegment {
                    index: 0,
                    start_ms: 1000,
                    end_ms: Some(2000),
                    text: "first".to_string(),
                    chapter_index: Some(1),
                },
                YoutubeTranscriptSegment {
                    index: 1,
                    start_ms: 2500,
                    end_ms: None,
                    text: "second".to_string(),
                    chapter_index: None,
                },
            ],
            raw_payload: serde_json::json!({ "events": [] }),
        };
        let mut tx = pool.begin().await.expect("begin transaction");

        replace_transcript_segments(&mut tx, 9, 1, &transcript)
            .await
            .expect("replace segments");
        tx.commit().await.expect("commit");

        type SegmentRow = (
            i64,
            i64,
            Option<i64>,
            String,
            Option<i64>,
            Option<String>,
            String,
            i64,
        );
        let rows: Vec<SegmentRow> = sqlx::query_as(
            r#"
                SELECT
                    segment_index,
                    start_ms,
                    end_ms,
                    text,
                    chapter_index,
                    caption_language,
                    caption_track_kind,
                    is_auto_generated
                FROM youtube_transcript_segments
                WHERE item_id = 9
                ORDER BY segment_index
                "#,
        )
        .fetch_all(&pool)
        .await
        .expect("load segments");

        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            (
                0,
                1000,
                Some(2000),
                "first".to_string(),
                Some(1),
                Some("en".to_string()),
                "auto".to_string(),
                1,
            )
        );
        assert_eq!(rows[1].3, "second");
    }

    #[tokio::test]
    async fn replace_transcript_segments_rebuilds_analysis_documents_by_segment_order() {
        let pool = transcript_pool().await;
        crate::sources::test_support::create_analysis_documents_table(&pool).await;
        seed_video_source_and_transcript_item(&pool, 2, 20).await;

        let transcript = YoutubeTranscript {
            video_id: "video2".to_string(),
            language: Some("en".to_string()),
            is_auto_generated: false,
            track_kind: YoutubeCaptionTrackKind::Manual,
            raw_payload: serde_json::json!({ "events": [] }),
            segments: vec![
                YoutubeTranscriptSegment {
                    index: 0,
                    start_ms: 900,
                    end_ms: Some(1_500),
                    text: "early".to_string(),
                    chapter_index: None,
                },
                YoutubeTranscriptSegment {
                    index: 1,
                    start_ms: 10_000,
                    end_ms: Some(11_000),
                    text: "late".to_string(),
                    chapter_index: None,
                },
            ],
        };

        let mut tx = pool.begin().await.expect("begin tx");
        replace_transcript_segments(&mut tx, 20, 2, &transcript)
            .await
            .expect("replace segments");
        tx.commit().await.expect("commit");

        let refs: Vec<String> = sqlx::query_scalar(
            "SELECT ref FROM analysis_documents
             WHERE source_id = 2 AND document_kind = 'youtube_transcript'
             ORDER BY document_order ASC, id ASC",
        )
        .fetch_all(&pool)
        .await
        .expect("load refs");
        assert_eq!(refs, vec!["s2-i20@900ms", "s2-i20@10000ms"]);
    }

    async fn transcript_pool() -> sqlx::SqlitePool {
        let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;
        sqlx::query(
            r#"
            CREATE TABLE youtube_transcript_segments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
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
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create segments table");
        pool
    }

    async fn seed_video_source_and_transcript_item(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        item_id: i64,
    ) {
        crate::sources::test_support::create_youtube_typed_source_tables(pool).await;
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (?, 'youtube', 'video', ?, 'Video', 1, 1, 1)",
        )
        .bind(source_id)
        .bind(format!("video{source_id}"))
        .execute(pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO youtube_video_sources (
                source_id, video_id, canonical_url, title, channel_title,
                published_at, video_form, availability_status
             ) VALUES (?, ?, ?, 'Video', 'Channel', '2026-05-01', 'regular', 'available')",
        )
        .bind(source_id)
        .bind(format!("video{source_id}"))
        .bind(format!("https://www.youtube.com/watch?v=video{source_id}"))
        .execute(pool)
        .await
        .expect("seed typed video");
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at,
                ingested_at, content_kind, has_media, content_zstd
             ) VALUES (?, ?, 'transcript:video:en:manual', 'youtube_transcript',
                'Channel', 1704067200, 1704067200, 'text_only', 0, x'01')",
        )
        .bind(item_id)
        .bind(source_id)
        .execute(pool)
        .await
        .expect("seed transcript item");
    }

    fn track(language: &str, track_kind: YoutubeCaptionTrackKind) -> YoutubeCaptionTrack {
        YoutubeCaptionTrack {
            language: Some(language.to_string()),
            name: None,
            is_auto_generated: matches!(track_kind, YoutubeCaptionTrackKind::Auto),
            track_kind,
        }
    }
}
