pub(crate) fn normalize_transcript_analysis_output_for_runtime(
    output: &serde_json::Value,
) -> serde_json::Value {
    let mut normalized = output.clone();
    let Some(map) = normalized.as_object_mut() else {
        return normalized;
    };

    map.entry("warning_candidates".to_string())
        .or_insert_with(|| serde_json::json!([]));

    normalized
}

pub(crate) fn normalize_transcript_analysis_output_for_schema(
    output: &serde_json::Value,
) -> serde_json::Value {
    normalize_transcript_analysis_output_for_runtime(output)
}

pub(crate) fn normalize_synthesis_output_for_runtime(
    output: &serde_json::Value,
) -> serde_json::Value {
    let mut normalized = output.clone();
    let Some(map) = normalized.as_object_mut() else {
        return normalized;
    };

    map.entry("limitations".to_string())
        .or_insert_with(|| serde_json::json!([]));
    map.entry("warning_candidates".to_string())
        .or_insert_with(|| serde_json::json!([]));
    normalize_string_array_items(map, "limitations", "text");
    normalize_string_array_items(map, "warning_candidates", "text");
    if let Some(candidate) = map
        .get_mut("synthesis_candidate")
        .and_then(serde_json::Value::as_object_mut)
    {
        normalize_string_array_items(candidate, "cross_video_themes", "theme_text");
        normalize_string_array_items(candidate, "common_claims", "summary_text");
        normalize_string_array_items(candidate, "contradictions_across_videos", "description");
    }

    normalized
}

fn normalize_string_array_items(
    map: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    text_key: &str,
) {
    let Some(items) = map.get_mut(key).and_then(serde_json::Value::as_array_mut) else {
        return;
    };
    for item in items {
        if let Some(text) = item.as_str().map(ToString::to_string) {
            *item = serde_json::json!({ text_key: text });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_synthesis_output_for_runtime;

    #[test]
    fn synthesis_runtime_normalization_defaults_readable_arrays() {
        let output = serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/synthesis",
            "synthesis_candidate": {
                "summary_text": "Summary",
                "cross_video_themes": [],
                "common_claims": [],
                "contradictions_across_videos": []
            }
        });

        let normalized = normalize_synthesis_output_for_runtime(&output);

        assert_eq!(normalized["limitations"], serde_json::json!([]));
        assert_eq!(normalized["warning_candidates"], serde_json::json!([]));
    }
}
