use super::corpus::AnalysisCorpusMessage;
use super::models::{AnalysisTraceData, AnalysisTraceRef};
use extractum_core::{
    compression::{compress_json_bytes, decompress_bytes},
    error::{internal_error, AppError, AppResult},
};
use sqlx::SqlitePool;

const TRACE_EXCERPT_MAX_CHARS: usize = 480;

#[allow(dead_code)]
pub(crate) fn compress_trace_data(trace_data: &AnalysisTraceData) -> AppResult<Vec<u8>> {
    let json = serde_json::to_vec(trace_data).map_err(internal_error)?;
    compress_json_bytes(&json).map_err(internal_error)
}

pub(crate) fn decode_trace_data(bytes: Option<&[u8]>) -> AppResult<AnalysisTraceData> {
    let Some(bytes) = bytes else {
        return Ok(AnalysisTraceData::default());
    };

    let decoded = decompress_bytes(bytes).map_err(internal_error)?;
    serde_json::from_slice(&decoded).map_err(internal_error)
}

pub async fn get_analysis_run_trace_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<AnalysisTraceData> {
    let trace_data: Option<Option<Vec<u8>>> =
        sqlx::query_scalar("SELECT trace_data_zstd FROM analysis_runs WHERE id = ?")
            .bind(run_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?;
    let trace_data = trace_data
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;
    decode_trace_data(trace_data.as_deref())
}

#[derive(sqlx::FromRow)]
struct TraceResolutionRunContext {
    snapshot_captured: i64,
    snapshot_error: Option<String>,
    snapshot_message_count: i64,
}

pub async fn resolve_analysis_trace_refs_in_pool(
    pool: &SqlitePool,
    run_id: i64,
    refs: Vec<String>,
) -> AppResult<Vec<AnalysisTraceRef>> {
    let mut normalized_refs = refs
        .into_iter()
        .filter_map(|reference| normalize_ref(&reference))
        .collect::<Vec<_>>();
    normalized_refs.sort();
    normalized_refs.dedup();
    if normalized_refs.is_empty() {
        return Ok(Vec::new());
    }

    let context = sqlx::query_as::<_, TraceResolutionRunContext>(
        r#"
        SELECT
            CASE WHEN snapshot_captured_at IS NOT NULL THEN 1 ELSE 0 END AS snapshot_captured,
            snapshot_error,
            (
                SELECT COUNT(*)
                FROM analysis_run_messages
                WHERE run_id = analysis_runs.id
            ) AS snapshot_message_count
        FROM analysis_runs
        WHERE id = ?
        "#,
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;

    let corpus = super::corpus::load_trace_resolution_messages(
        pool,
        run_id,
        context.snapshot_captured != 0 && context.snapshot_error.is_none(),
        context.snapshot_message_count,
    )
    .await?;
    try_build_trace_refs(&normalized_refs, &corpus)
}

pub(crate) fn normalize_ref(candidate: &str) -> Option<String> {
    let candidate = candidate.trim().trim_matches('[').trim_matches(']');
    let (source_part, item_part) = candidate.split_once("-i")?;
    if !source_part.starts_with('s') {
        return None;
    }
    let source_digits = &source_part[1..];
    if source_digits.is_empty() || !source_digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let (item_digits, timestamp_suffix) = match item_part.split_once('@') {
        Some((digits, suffix)) => (digits, Some(normalize_timestamp_suffix(suffix)?)),
        None => (item_part, None),
    };

    if item_digits.is_empty() || !item_digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let suffix = timestamp_suffix.unwrap_or_default();
    Some(format!("s{source_digits}-i{item_digits}{suffix}"))
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

pub(crate) fn build_trace_refs(
    refs: &[String],
    corpus: &[AnalysisCorpusMessage],
) -> Vec<AnalysisTraceRef> {
    try_build_trace_refs(refs, corpus).unwrap_or_default()
}

pub(crate) fn try_build_trace_refs(
    refs: &[String],
    corpus: &[AnalysisCorpusMessage],
) -> AppResult<Vec<AnalysisTraceRef>> {
    let mut trace_refs = Vec::new();

    for reference in refs {
        if let Some(message) = find_trace_message_checked(reference, corpus)? {
            let parsed_ref = parse_structured_ref(reference);
            let (youtube_url, youtube_timestamp_seconds, youtube_display_label) =
                youtube_trace_fields(reference, message, parsed_ref.as_ref());
            trace_refs.push(AnalysisTraceRef {
                r#ref: reference.clone(),
                item_id: message.item_id(),
                source_id: message.source_id(),
                external_id: message.external_id().to_string(),
                published_at: message.published_at(),
                excerpt: clip_excerpt(message.content(), TRACE_EXCERPT_MAX_CHARS),
                youtube_url,
                youtube_timestamp_seconds,
                youtube_display_label,
                is_synthetic: is_synthetic_message(message),
            });
        }
    }

    Ok(trace_refs)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedTraceRef {
    source_id: i64,
    item_id: i64,
    timestamp_ms: Option<i64>,
}

fn parse_structured_ref(reference: &str) -> Option<ParsedTraceRef> {
    let reference = normalize_ref(reference)?;
    let (source_part, item_part) = reference.split_once("-i")?;
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
    })
}

fn find_trace_message_checked<'a>(
    reference: &str,
    corpus: &'a [AnalysisCorpusMessage],
) -> AppResult<Option<&'a AnalysisCorpusMessage>> {
    if let Some(message) = corpus
        .iter()
        .find(|message| message.reference() == reference)
    {
        return Ok(Some(message));
    }

    let Some(parsed) = parse_structured_ref(reference) else {
        return Ok(None);
    };

    Ok(corpus.iter().find(|message| {
        message.source_id() == parsed.source_id && message.item_id() == parsed.item_id
    }))
}

fn is_synthetic_message(message: &AnalysisCorpusMessage) -> bool {
    message.item_id() == 0 || message.item_kind() == Some("youtube_description")
}

fn youtube_trace_fields(
    reference: &str,
    message: &AnalysisCorpusMessage,
    parsed_ref: Option<&ParsedTraceRef>,
) -> (Option<String>, Option<i64>, Option<String>) {
    let Some(metadata) = message.metadata_zstd().and_then(decode_metadata_json) else {
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
        .or_else(|| {
            parse_structured_ref(message.reference()).and_then(|parsed| parsed.timestamp_ms)
        })
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

pub(crate) fn build_trace_data(
    markdown: &str,
    corpus: &[AnalysisCorpusMessage],
) -> AnalysisTraceData {
    let refs = extract_cited_refs(markdown);
    let trace_refs = build_trace_refs(&refs, corpus);

    AnalysisTraceData { refs: trace_refs }
}

#[cfg(test)]
mod tests {
    use super::super::corpus::AnalysisCorpusMessage;
    use super::super::models::{AnalysisTraceData, AnalysisTraceRef};
    use super::{
        build_trace_refs, clip_excerpt, compress_trace_data, decode_trace_data, normalize_ref,
    };
    use extractum_core::{compression::compress_json_bytes, error::AppErrorKind};

    fn metadata_zstd(value: serde_json::Value) -> Vec<u8> {
        let json = serde_json::to_vec(&value).expect("serialize metadata");
        compress_json_bytes(&json).expect("compress metadata")
    }

    fn youtube_segment_message() -> AnalysisCorpusMessage {
        AnalysisCorpusMessage::new(
            400,
            12,
            "transcript:video123:en:manual".to_string(),
            1_710_000_000,
            Some("Channel".to_string()),
            "Segment text".to_string(),
            "s12-i400@754000ms".to_string(),
            Some("youtube_transcript".to_string()),
            Some("youtube".to_string()),
            Some("video".to_string()),
            Some(metadata_zstd(serde_json::json!({
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
        )
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
        let refs = vec!["s1-i1".to_string()];
        let corpus = vec![AnalysisCorpusMessage::new(
            1,
            1,
            "1".to_string(),
            1_710_000_000,
            None,
            "Индекс рынка акций ".repeat(40),
            "s1-i1".to_string(),
            Some("telegram_message".to_string()),
            Some("telegram".to_string()),
            None,
            None,
        )];

        let trace_refs = build_trace_refs(&refs, &corpus);

        assert_eq!(trace_refs.len(), 1);
        assert!(trace_refs[0].excerpt.ends_with("..."));
    }

    #[test]
    fn normalize_ref_accepts_item_refs() {
        assert_eq!(normalize_ref("[s12-i845]").as_deref(), Some("s12-i845"));
        assert_eq!(
            normalize_ref("s999999999999999999999999-i999999999999999999999999").as_deref(),
            Some("s999999999999999999999999-i999999999999999999999999")
        );
        assert_eq!(
            normalize_ref("s12-i400@754000ms").as_deref(),
            Some("s12-i400@754000ms")
        );
        assert_eq!(
            normalize_ref("[s12-i400@754000-790000ms]").as_deref(),
            Some("s12-i400@754000-790000ms")
        );
        assert_eq!(normalize_ref("s12-m845"), None);
        assert_eq!(normalize_ref("s12-i400@790000-754000ms"), None);
        assert_eq!(normalize_ref("s12-iabc"), None);
        assert_eq!(normalize_ref("x12-i845"), None);
    }

    #[test]
    fn decode_trace_data_returns_typed_internal_for_invalid_zstd() {
        let error = match decode_trace_data(Some(&[0])) {
            Ok(_) => panic!("invalid trace zstd should fail"),
            Err(error) => error,
        };

        assert_eq!(error.kind, AppErrorKind::Internal);
        assert!(!error.message.is_empty());
    }

    #[test]
    fn legacy_trace_bytes_decode_after_core_compression_handoff() {
        let legacy_bytes = [
            40, 181, 47, 253, 0, 88, 89, 0, 0, 123, 34, 114, 101, 102, 115, 34, 58, 91, 93, 125,
        ];

        let decoded = decode_trace_data(Some(&legacy_bytes)).expect("decode legacy trace bytes");

        assert_eq!(decoded, AnalysisTraceData::default());
    }

    #[test]
    fn decode_trace_data_returns_typed_internal_for_invalid_json() {
        let compressed = compress_json_bytes(b"not-json").expect("compress invalid JSON bytes");

        let error = decode_trace_data(Some(&compressed)).expect_err("invalid JSON should fail");

        assert_eq!(error.kind, AppErrorKind::Internal);
        assert!(!error.message.is_empty());
    }

    #[test]
    fn trace_ref_json_is_byte_compatible_for_telegram_and_youtube() {
        let trace = AnalysisTraceData {
            refs: vec![
                AnalysisTraceRef {
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
                },
                AnalysisTraceRef {
                    r#ref: "s12-i400@754000ms".to_string(),
                    item_id: 400,
                    source_id: 12,
                    external_id: "transcript:video123:en:manual".to_string(),
                    published_at: 1_710_000_001,
                    excerpt: "YouTube excerpt".to_string(),
                    youtube_url: Some("https://www.youtube.com/watch?v=video123&t=754".to_string()),
                    youtube_timestamp_seconds: Some(754),
                    youtube_display_label: Some("Video title at 12:34".to_string()),
                    is_synthetic: false,
                },
            ],
        };

        let compressed = compress_trace_data(&trace).expect("compress trace");
        let json = extractum_core::compression::decompress_bytes(&compressed)
            .expect("decompress trace JSON");

        assert_eq!(json, serde_json::to_vec(&trace).expect("serialize trace"));
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
    fn analysis_trace_ref_serializes_youtube_fields_as_null_for_telegram_refs() {
        let reference = AnalysisTraceRef {
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
        let corpus = vec![AnalysisCorpusMessage::new(
            0,
            12,
            "description:video123".to_string(),
            1_710_000_000,
            Some("Channel".to_string()),
            "Synthetic description".to_string(),
            "s12-i0".to_string(),
            Some("youtube_description".to_string()),
            Some("youtube".to_string()),
            Some("video".to_string()),
            Some(metadata_zstd(serde_json::json!({
                "video_id": "video123",
                "canonical_url": "https://www.youtube.com/watch?v=video123",
                "title": "Video title",
                "item_kind": "youtube_description"
            }))),
        )];

        let trace_refs = build_trace_refs(&refs, &corpus);

        assert_eq!(trace_refs.len(), 1);
        assert_eq!(trace_refs[0].item_id, 0);
        assert!(trace_refs[0].is_synthetic);
    }
}
