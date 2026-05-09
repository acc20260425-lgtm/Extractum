use std::io::Cursor;

use super::models::{AnalysisTraceData, AnalysisTraceRef, CorpusMessage};

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
    let mut trace_refs = Vec::new();

    for reference in refs {
        if let Some(message) = corpus.iter().find(|message| message.r#ref == *reference) {
            trace_refs.push(AnalysisTraceRef {
                r#ref: reference.clone(),
                item_id: message.item_id,
                source_id: message.source_id,
                external_id: message.external_id.clone(),
                published_at: message.published_at,
                excerpt: clip_excerpt(&message.content, TRACE_EXCERPT_MAX_CHARS),
            });
        }
    }

    trace_refs
}

pub(crate) fn build_trace_data(markdown: &str, corpus: &[CorpusMessage]) -> AnalysisTraceData {
    let refs = extract_cited_refs(markdown);
    let trace_refs = build_trace_refs(&refs, corpus);

    AnalysisTraceData { refs: trace_refs }
}

#[cfg(test)]
mod tests {
    use super::{build_trace_refs, clip_excerpt, normalize_ref};
    use crate::analysis::models::CorpusMessage;

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
}
