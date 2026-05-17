use std::io::Cursor;

use super::models::{AnalysisTraceData, AnalysisTraceRef, CorpusMessage};
use crate::compression::decompress_bytes;
use crate::error::{AppError, AppResult};

const TRACE_EXCERPT_MAX_CHARS: usize = 480;

#[allow(dead_code)]
pub(crate) fn compress_trace_data(trace_data: &AnalysisTraceData) -> Result<Vec<u8>, String> {
    let json = serde_json::to_vec(trace_data).map_err(|e| e.to_string())?;
    zstd::encode_all(Cursor::new(json), 3).map_err(|e| e.to_string())
}

pub(crate) fn decode_trace_data(bytes: Option<&[u8]>) -> Result<AnalysisTraceData, String> {
    let Some(bytes) = bytes else {
        return Ok(AnalysisTraceData::default());
    };

    let decoded = zstd::decode_all(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    serde_json::from_slice(&decoded).map_err(|e| e.to_string())
}

pub(crate) fn normalize_ref(candidate: &str) -> Option<String> {
    let candidate = candidate.trim().trim_matches('[').trim_matches(']');
    for separator in ["-i", "-m"] {
        let Some((source_part, item_part)) = candidate.split_once(separator) else {
            continue;
        };
        if !source_part.starts_with('s') {
            return None;
        }
        let source_digits = &source_part[1..];
        if source_digits.is_empty() || !source_digits.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }

        let (item_digits, timestamp_suffix) = match item_part.split_once('@') {
            Some((digits, suffix)) if separator == "-i" => {
                (digits, Some(normalize_timestamp_suffix(suffix)?))
            }
            Some(_) => return None,
            None => (item_part, None),
        };

        if item_digits.is_empty() || !item_digits.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }

        let suffix = timestamp_suffix.unwrap_or_default();
        return Some(format!("s{source_digits}{separator}{item_digits}{suffix}"));
    }

    None
}

fn normalize_timestamp_suffix(suffix: &str) -> Option<String> {
    let body = suffix.strip_suffix("ms")?;
    if let Some((start, end)) = body.split_once('-') {
        let start_ms = parse_ref_millis(start)?;
        let end_ms = parse_ref_millis(end)?;
        if end_ms < start_ms {
            return None;
        }
        return Some(format!("@{start_ms}-{end_ms}ms"));
    }

    let start_ms = parse_ref_millis(body)?;
    Some(format!("@{start_ms}ms"))
}

fn parse_ref_millis(value: &str) -> Option<i64> {
    if value.is_empty() || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    value.parse::<i64>().ok()
}

pub(crate) fn extract_cited_refs(markdown: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut cursor = 0usize;

    while let Some(relative_start) = markdown[cursor..].find('[') {
        let start = cursor + relative_start;
        let Some(relative_end) = markdown[start + 1..].find(']') else {
            break;
        };
        let end = start + 1 + relative_end;
        let inside = &markdown[start + 1..end];
        for part in inside.split(',') {
            if let Some(reference) = normalize_ref(part) {
                if !refs.contains(&reference) {
                    refs.push(reference);
                }
            }
        }
        cursor = end + 1;
    }

    refs
}

fn clip_excerpt(content: &str, max_chars: usize) -> String {
    let mut chars = content.chars();
    let clipped = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{clipped}...")
    } else {
        content.to_string()
    }
}

pub(crate) fn build_trace_refs(refs: &[String], corpus: &[CorpusMessage]) -> Vec<AnalysisTraceRef> {
    try_build_trace_refs(refs, corpus).unwrap_or_default()
}

pub(crate) fn try_build_trace_refs(
    refs: &[String],
    corpus: &[CorpusMessage],
) -> AppResult<Vec<AnalysisTraceRef>> {
    let mut trace_refs = Vec::new();

    for reference in refs {
        if let Some(message) = find_trace_message_checked(reference, corpus)? {
            let parsed_ref = parse_structured_ref(reference);
            let (youtube_url, youtube_timestamp_seconds, youtube_display_label) =
                youtube_trace_fields(reference, message, parsed_ref.as_ref());
            trace_refs.push(AnalysisTraceRef {
                r#ref: reference.clone(),
                item_id: message.item_id,
                source_id: message.source_id,
                external_id: message.external_id.clone(),
                published_at: message.published_at,
                excerpt: clip_excerpt(&message.content, TRACE_EXCERPT_MAX_CHARS),
                youtube_url,
                youtube_timestamp_seconds,
                youtube_display_label,
                is_synthetic: is_synthetic_message(message),
            });
        }
    }

    Ok(trace_refs)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TraceRefKind {
    Item,
    LegacyMessage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedTraceRef {
    source_id: i64,
    item_id: i64,
    timestamp_ms: Option<i64>,
    kind: TraceRefKind,
}

fn parse_structured_ref(reference: &str) -> Option<ParsedTraceRef> {
    let reference = normalize_ref(reference)?;
    let (source_part, item_part, kind) = if let Some((source_part, item_part)) =
        reference.split_once("-i")
    {
        (source_part, item_part, TraceRefKind::Item)
    } else {
        let (source_part, item_part) = reference.split_once("-m")?;
        (source_part, item_part, TraceRefKind::LegacyMessage)
    };
    let source_id = source_part.strip_prefix('s')?.parse::<i64>().ok()?;
    let (item_digits, timestamp_ms) = match item_part.split_once('@') {
        Some((digits, suffix)) => {
            let suffix = suffix.strip_suffix("ms")?;
            let start = suffix
                .split_once('-')
                .map(|(start, _)| start)
                .unwrap_or(suffix);
            (digits, Some(start.parse::<i64>().ok()?))
        }
        None => (item_part, None),
    };
    let item_id = item_digits.parse::<i64>().ok()?;

    Some(ParsedTraceRef {
        source_id,
        item_id,
        timestamp_ms,
        kind,
    })
}

fn find_trace_message_checked<'a>(
    reference: &str,
    corpus: &'a [CorpusMessage],
) -> AppResult<Option<&'a CorpusMessage>> {
    if let Some(message) = corpus.iter().find(|message| message.r#ref == reference) {
        return Ok(Some(message));
    }

    let Some(parsed) = parse_structured_ref(reference) else {
        return Ok(None);
    };

    match parsed.kind {
        TraceRefKind::Item => Ok(corpus
            .iter()
            .find(|message| {
                message.source_id == parsed.source_id && message.item_id == parsed.item_id
            })),
        TraceRefKind::LegacyMessage => {
            let message_id = parsed.item_id.to_string();
            let candidates = corpus
                .iter()
                .filter(|message| {
                    message.source_id == parsed.source_id
                        && message.external_id == message_id
                        && message.item_kind.as_deref() == Some("telegram_message")
                })
                .collect::<Vec<_>>();

            match candidates.len() {
                0 => Ok(None),
                1 => Ok(Some(candidates[0])),
                _ => Err(AppError::conflict(format!(
                    "Legacy Telegram ref {reference} is ambiguous across Telegram history domains"
                ))),
            }
        }
    }
}

fn is_synthetic_message(message: &CorpusMessage) -> bool {
    message.item_id == 0 || message.item_kind.as_deref() == Some("youtube_description")
}

fn youtube_trace_fields(
    reference: &str,
    message: &CorpusMessage,
    parsed_ref: Option<&ParsedTraceRef>,
) -> (Option<String>, Option<i64>, Option<String>) {
    let Some(metadata) = message
        .metadata_zstd
        .as_deref()
        .and_then(decode_metadata_json)
    else {
        return (None, None, None);
    };

    let Some(canonical_url) = metadata
        .get("canonical_url")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
    else {
        return (None, None, None);
    };

    let timestamp_ms = parsed_ref
        .and_then(|parsed| parsed.timestamp_ms)
        .or_else(|| parse_structured_ref(&message.r#ref).and_then(|parsed| parsed.timestamp_ms))
        .or_else(|| {
            metadata
                .get("segment_start_ms")
                .and_then(|value| value.as_i64())
        });
    let timestamp_seconds = timestamp_ms.map(|value| value / 1000);

    let title = metadata
        .get("title")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty());
    let youtube_url = match timestamp_seconds {
        Some(seconds) => Some(append_youtube_timestamp(canonical_url, seconds)),
        None => Some(canonical_url.to_string()),
    };
    let youtube_display_label = match (title, timestamp_seconds) {
        (Some(title), Some(seconds)) => {
            Some(format!("{title} at {}", format_youtube_timestamp(seconds)))
        }
        (None, Some(seconds)) => Some(format!("YouTube at {}", format_youtube_timestamp(seconds))),
        (Some(title), None) => Some(title.to_string()),
        (None, None) if reference.starts_with('s') => Some("YouTube".to_string()),
        (None, None) => None,
    };

    (youtube_url, timestamp_seconds, youtube_display_label)
}

fn decode_metadata_json(bytes: &[u8]) -> Option<serde_json::Value> {
    let decoded = decompress_bytes(bytes).ok()?;
    serde_json::from_slice(&decoded).ok()
}

fn append_youtube_timestamp(canonical_url: &str, seconds: i64) -> String {
    let separator = if canonical_url.contains('?') {
        '&'
    } else {
        '?'
    };
    format!("{canonical_url}{separator}t={seconds}")
}

fn format_youtube_timestamp(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

pub(crate) fn build_trace_data(markdown: &str, corpus: &[CorpusMessage]) -> AnalysisTraceData {
    let refs = extract_cited_refs(markdown);
    let trace_refs = build_trace_refs(&refs, corpus);

    AnalysisTraceData { refs: trace_refs }
}

#[cfg(test)]
mod tests {
    use super::{build_trace_refs, clip_excerpt, normalize_ref, try_build_trace_refs};
    use crate::analysis::models::CorpusMessage;
    use crate::compression::compress_json_bytes;

    fn metadata_zstd(value: serde_json::Value) -> Vec<u8> {
        let json = serde_json::to_vec(&value).expect("serialize metadata");
        compress_json_bytes(&json).expect("compress metadata")
    }

    fn youtube_segment_message() -> CorpusMessage {
        CorpusMessage {
            item_id: 400,
            source_id: 12,
            external_id: "transcript:video123:en:manual".to_string(),
            published_at: 1_710_000_000,
            author: Some("Channel".to_string()),
            content: "Segment text".to_string(),
            r#ref: "s12-i400@754000ms".to_string(),
            item_kind: Some("youtube_transcript".to_string()),
            source_type: Some("youtube".to_string()),
            source_subtype: Some("video".to_string()),
            metadata_zstd: Some(metadata_zstd(serde_json::json!({
                "video_id": "video123",
                "canonical_url": "https://www.youtube.com/watch?v=video123",
                "title": "Video title",
                "channel_title": "Channel",
                "channel_handle": "@channel",
                "caption_language": "en",
                "caption_track_kind": "manual",
                "segment_start_ms": 754000,
                "segment_end_ms": 790000,
                "item_kind": "youtube_transcript"
            }))),
        }
    }

    #[test]
    fn clip_excerpt_truncates_on_char_boundary() {
        let content = "и".repeat(481);

        let excerpt = clip_excerpt(&content, 480);

        assert_eq!(excerpt.chars().count(), 483);
        assert!(excerpt.ends_with("..."));
    }

    #[test]
    fn build_trace_refs_handles_multibyte_excerpt() {
        let refs = vec!["s1-m1".to_string()];
        let corpus = vec![CorpusMessage {
            item_id: 1,
            source_id: 1,
            external_id: "1".to_string(),
            published_at: 1_710_000_000,
            author: None,
            content: "Индекс рынка акций ".repeat(40),
            r#ref: "s1-m1".to_string(),
            item_kind: Some("telegram_message".to_string()),
            source_type: Some("telegram".to_string()),
            source_subtype: None,
            metadata_zstd: None,
        }];

        let trace_refs = build_trace_refs(&refs, &corpus);

        assert_eq!(trace_refs.len(), 1);
        assert!(trace_refs[0].excerpt.ends_with("..."));
    }

    #[test]
    fn normalize_ref_accepts_item_refs_and_legacy_message_refs() {
        assert_eq!(normalize_ref("[s12-i845]").as_deref(), Some("s12-i845"));
        assert_eq!(normalize_ref("s12-m845").as_deref(), Some("s12-m845"));
        assert_eq!(
            normalize_ref("s12-i400@754000ms").as_deref(),
            Some("s12-i400@754000ms")
        );
        assert_eq!(
            normalize_ref("[s12-i400@754000-790000ms]").as_deref(),
            Some("s12-i400@754000-790000ms")
        );
        assert_eq!(normalize_ref("s12-m400@754000ms"), None);
        assert_eq!(normalize_ref("s12-i400@790000-754000ms"), None);
        assert_eq!(normalize_ref("s12-iabc"), None);
        assert_eq!(normalize_ref("x12-i845"), None);
    }

    #[test]
    fn build_trace_refs_resolves_exact_youtube_timestamp_refs() {
        let refs = vec!["s12-i400@754000ms".to_string()];
        let corpus = vec![youtube_segment_message()];

        let trace_refs = build_trace_refs(&refs, &corpus);

        assert_eq!(trace_refs.len(), 1);
        assert_eq!(trace_refs[0].r#ref, "s12-i400@754000ms");
        assert_eq!(trace_refs[0].youtube_timestamp_seconds, Some(754));
        assert_eq!(
            trace_refs[0].youtube_url.as_deref(),
            Some("https://www.youtube.com/watch?v=video123&t=754")
        );
        assert_eq!(
            trace_refs[0].youtube_display_label.as_deref(),
            Some("Video title at 12:34")
        );
        assert!(!trace_refs[0].is_synthetic);
    }

    #[test]
    fn build_trace_refs_falls_back_to_base_item_refs() {
        let refs = vec!["s12-i400".to_string()];
        let corpus = vec![youtube_segment_message()];

        let trace_refs = build_trace_refs(&refs, &corpus);

        assert_eq!(trace_refs.len(), 1);
        assert_eq!(trace_refs[0].item_id, 400);
    }

    #[test]
    fn build_trace_refs_does_not_treat_legacy_message_ref_as_local_item_id() {
        let refs = vec!["s12-m400".to_string()];
        let corpus = vec![youtube_segment_message()];

        let trace_refs = build_trace_refs(&refs, &corpus);

        assert!(trace_refs.is_empty());
    }

    #[test]
    fn build_trace_refs_resolves_unique_legacy_message_ref_by_external_message_id() {
        let refs = vec!["s1-m42".to_string()];
        let corpus = vec![CorpusMessage {
            item_id: 900,
            source_id: 1,
            external_id: "42".to_string(),
            published_at: 1,
            author: None,
            content: "telegram".to_string(),
            r#ref: "s1-i900".to_string(),
            item_kind: Some("telegram_message".to_string()),
            source_type: Some("telegram".to_string()),
            source_subtype: Some("supergroup".to_string()),
            metadata_zstd: None,
        }];

        let trace_refs = try_build_trace_refs(&refs, &corpus).expect("resolve refs");

        assert_eq!(trace_refs.len(), 1);
        assert_eq!(trace_refs[0].item_id, 900);
        assert_eq!(trace_refs[0].r#ref, "s1-m42");
    }

    #[test]
    fn build_trace_refs_returns_conflict_for_ambiguous_legacy_message_ref() {
        let refs = vec!["s1-m42".to_string()];
        let first = CorpusMessage {
            item_id: 900,
            source_id: 1,
            external_id: "42".to_string(),
            published_at: 1,
            author: None,
            content: "current".to_string(),
            r#ref: "s1-i900".to_string(),
            item_kind: Some("telegram_message".to_string()),
            source_type: Some("telegram".to_string()),
            source_subtype: Some("supergroup".to_string()),
            metadata_zstd: None,
        };
        let second = CorpusMessage {
            item_id: 901,
            content: "migrated".to_string(),
            r#ref: "s1-i901".to_string(),
            ..first.clone()
        };

        let error = try_build_trace_refs(&refs, &[first, second])
            .expect_err("ambiguous legacy ref conflicts");

        assert_eq!(error.kind, crate::error::AppErrorKind::Conflict);
    }

    #[test]
    fn analysis_trace_ref_serializes_youtube_fields_as_null_for_telegram_refs() {
        let reference = crate::analysis::models::AnalysisTraceRef {
            r#ref: "s1-i2".to_string(),
            item_id: 2,
            source_id: 1,
            external_id: "2".to_string(),
            published_at: 1_710_000_000,
            excerpt: "Telegram excerpt".to_string(),
            youtube_url: None,
            youtube_timestamp_seconds: None,
            youtube_display_label: None,
            is_synthetic: false,
        };

        let json = serde_json::to_value(reference).expect("serialize trace ref");

        assert!(json["youtube_url"].is_null());
        assert!(json["youtube_timestamp_seconds"].is_null());
        assert!(json["youtube_display_label"].is_null());
        assert_eq!(json["is_synthetic"], false);
    }

    #[test]
    fn build_trace_refs_marks_youtube_description_refs_as_synthetic() {
        let refs = vec!["s12-i0".to_string()];
        let corpus = vec![CorpusMessage {
            item_id: 0,
            source_id: 12,
            external_id: "description:video123".to_string(),
            published_at: 1_710_000_000,
            author: Some("Channel".to_string()),
            content: "Synthetic description".to_string(),
            r#ref: "s12-i0".to_string(),
            item_kind: Some("youtube_description".to_string()),
            source_type: Some("youtube".to_string()),
            source_subtype: Some("video".to_string()),
            metadata_zstd: Some(metadata_zstd(serde_json::json!({
                "video_id": "video123",
                "canonical_url": "https://www.youtube.com/watch?v=video123",
                "title": "Video title",
                "item_kind": "youtube_description"
            }))),
        }];

        let trace_refs = build_trace_refs(&refs, &corpus);

        assert_eq!(trace_refs.len(), 1);
        assert_eq!(trace_refs[0].item_id, 0);
        assert!(trace_refs[0].is_synthetic);
    }
}
