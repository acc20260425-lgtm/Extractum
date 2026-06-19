pub(crate) fn normalize_transcript_analysis_output_for_schema(
    output: &serde_json::Value,
) -> serde_json::Value {
    let mut normalized = output.clone();
    let Some(map) = normalized.as_object_mut() else {
        return normalized;
    };

    normalize_stage_output_envelope(map);
    map.entry("warning_candidates".to_string())
        .or_insert_with(|| serde_json::json!([]));

    normalized
}

pub(crate) fn normalize_synthesis_output_for_runtime(
    output: &serde_json::Value,
) -> serde_json::Value {
    let mut normalized = output.clone();
    let Some(map) = normalized.as_object_mut() else {
        return normalized;
    };

    normalize_stage_output_envelope(map);
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

fn normalize_stage_output_envelope(map: &mut serde_json::Map<String, serde_json::Value>) {
    copy_alias_to_canonical_key(map, "stageIoVersion", "stage_io_version");
    copy_alias_to_canonical_key(map, "schemaVersion", "schema_version");
}

fn copy_alias_to_canonical_key(
    map: &mut serde_json::Map<String, serde_json::Value>,
    alias: &str,
    canonical: &str,
) {
    if map.contains_key(canonical) {
        return;
    }
    if let Some(value) = map.get(alias).cloned() {
        map.insert(canonical.to_string(), value);
    }
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
