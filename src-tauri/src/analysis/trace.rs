use std::io::Cursor;

use super::models::{AnalysisTraceData, AnalysisTraceRef, CorpusMessage};

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
    let (source_part, message_part) = candidate.split_once("-m")?;
    if !source_part.starts_with('s') {
        return None;
    }
    let source_digits = &source_part[1..];
    if source_digits.is_empty()
        || message_part.is_empty()
        || !source_digits.chars().all(|c| c.is_ascii_digit())
        || !message_part.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }

    Some(format!("s{source_digits}-m{message_part}"))
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

pub(crate) fn build_trace_refs(refs: &[String], corpus: &[CorpusMessage]) -> Vec<AnalysisTraceRef> {
    let mut trace_refs = Vec::new();

    for reference in refs {
        if let Some(message) = corpus.iter().find(|message| message.r#ref == *reference) {
            let excerpt = if message.content.len() > 480 {
                format!("{}...", &message.content[..480])
            } else {
                message.content.clone()
            };

            trace_refs.push(AnalysisTraceRef {
                r#ref: reference.clone(),
                item_id: message.item_id,
                source_id: message.source_id,
                external_id: message.external_id.clone(),
                published_at: message.published_at,
                excerpt,
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
