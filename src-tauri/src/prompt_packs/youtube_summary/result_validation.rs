use std::collections::HashSet;

use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PromptPackResultValidationFinding {
    pub(crate) severity: String,
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) object_path: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct YoutubeSummaryResultValidationContext {
    pub(crate) run_id: i64,
    pub(crate) terminal_status: String,
    pub(crate) evidence_mode: String,
}

impl YoutubeSummaryResultValidationContext {
    pub(crate) fn new(run_id: i64, terminal_status: &str, evidence_mode: &str) -> Self {
        Self {
            run_id,
            terminal_status: terminal_status.to_string(),
            evidence_mode: evidence_mode.to_string(),
        }
    }
}

pub(crate) fn validate_youtube_summary_canonical_result(
    canonical: &Value,
    context: &YoutubeSummaryResultValidationContext,
) -> Vec<PromptPackResultValidationFinding> {
    let mut findings = Vec::new();

    expect_string_value(
        canonical,
        "schema_version",
        "1.0",
        "$.schema_version",
        &mut findings,
    );
    expect_string_value(canonical, "pack_id", "youtube_summary", "$.pack_id", &mut findings);
    if canonical.get("run_id").and_then(Value::as_i64) != Some(context.run_id) {
        findings.push(finding(
            "error",
            "RV-RESULT-003",
            format!("run_id must match current run {}", context.run_id),
            Some("$.run_id".to_string()),
        ));
    }

    let youtube = canonical
        .pointer("/outputs/pack_data/youtube_summary")
        .filter(|value| value.is_object());
    if youtube.is_none() {
        findings.push(finding(
            "error",
            "VR-YS-001",
            "outputs.pack_data.youtube_summary must be an object",
            Some("$.outputs.pack_data.youtube_summary".to_string()),
        ));
    }

    for (key, path) in [
        ("source_refs", "$.source_refs"),
        ("claims", "$.claims"),
        ("evidence", "$.evidence"),
        ("warnings", "$.warnings"),
        ("limitations", "$.limitations"),
        ("quality_flags", "$.quality_flags"),
        ("audit_refs", "$.audit_refs"),
    ] {
        expect_array(canonical.get(key), path, "RV-RESULT-003", &mut findings);
    }

    if let Some(youtube) = youtube {
        expect_array(
            youtube.get("videos"),
            "$.outputs.pack_data.youtube_summary.videos",
            "VR-YS-001",
            &mut findings,
        );
        validate_synthesis_shape(youtube.get("synthesis"), &mut findings);
    }

    validate_unique_non_empty_ids(
        canonical.get("source_refs").and_then(Value::as_array),
        "source_ref_id",
        "$.source_refs",
        &mut findings,
    );
    validate_unique_non_empty_ids(
        canonical
            .pointer("/outputs/pack_data/youtube_summary/videos")
            .and_then(Value::as_array),
        "video_id",
        "$.outputs.pack_data.youtube_summary.videos",
        &mut findings,
    );
    validate_unique_non_empty_ids(
        canonical.get("claims").and_then(Value::as_array),
        "claim_id",
        "$.claims",
        &mut findings,
    );
    validate_unique_non_empty_ids(
        canonical.get("evidence").and_then(Value::as_array),
        "evidence_id",
        "$.evidence",
        &mut findings,
    );
    validate_synthesis_item_ids(canonical, &mut findings);

    findings
}

fn finding(
    severity: &str,
    code: &str,
    message: impl Into<String>,
    object_path: impl Into<Option<String>>,
) -> PromptPackResultValidationFinding {
    PromptPackResultValidationFinding {
        severity: severity.to_string(),
        code: code.to_string(),
        message: message.into(),
        object_path: object_path.into(),
    }
}

fn has_error(findings: &[PromptPackResultValidationFinding]) -> bool {
    findings.iter().any(|finding| finding.severity == "error")
}

fn expect_string_value(
    object: &Value,
    key: &str,
    expected: &str,
    path: &str,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    if object.get(key).and_then(Value::as_str) != Some(expected) {
        findings.push(finding(
            "error",
            "RV-RESULT-003",
            format!("{key} must be {expected}"),
            Some(path.to_string()),
        ));
    }
}

fn expect_array(
    value: Option<&Value>,
    path: &str,
    code: &str,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    if !value.is_some_and(Value::is_array) {
        findings.push(finding(
            "error",
            code,
            format!("{path} must be an array"),
            Some(path.to_string()),
        ));
    }
}

fn validate_unique_non_empty_ids(
    items: Option<&Vec<Value>>,
    id_key: &str,
    base_path: &str,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) -> HashSet<String> {
    let mut seen = HashSet::new();
    for (index, item) in items.into_iter().flatten().enumerate() {
        let path = format!("{base_path}[{index}].{id_key}");
        match item.get(id_key).and_then(Value::as_str).map(str::trim) {
            Some(value) if !value.is_empty() => {
                if !seen.insert(value.to_string()) {
                    findings.push(finding(
                        "error",
                        "RV-RESULT-001",
                        format!("duplicate {id_key} `{value}`"),
                        Some(path),
                    ));
                }
            }
            _ => findings.push(finding(
                "error",
                "RV-RESULT-004",
                format!("{id_key} must be a non-empty string"),
                Some(path),
            )),
        }
    }
    seen
}

fn validate_synthesis_shape(
    synthesis: Option<&Value>,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    let path = "$.outputs.pack_data.youtube_summary.synthesis";
    match synthesis {
        Some(Value::Null) => {}
        Some(Value::Object(map)) => {
            for key in [
                "cross_video_themes",
                "common_claims",
                "contradictions_across_videos",
                "claim_refs",
                "relation_refs",
                "evidence_refs",
                "source_refs",
            ] {
                expect_array(
                    map.get(key),
                    &format!("{path}.{key}"),
                    "VR-YS-001",
                    findings,
                );
            }
        }
        _ => findings.push(finding(
            "error",
            "VR-YS-001",
            "synthesis must be null or an object",
            Some(path.to_string()),
        )),
    }
}

fn validate_synthesis_item_ids(
    canonical: &Value,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    let Some(synthesis) = canonical
        .pointer("/outputs/pack_data/youtube_summary/synthesis")
        .and_then(Value::as_object)
    else {
        return;
    };

    let mut seen = HashSet::new();
    for (array_key, id_key) in [
        ("cross_video_themes", "theme_id"),
        ("common_claims", "common_claim_id"),
        ("contradictions_across_videos", "contradiction_id"),
    ] {
        let Some(items) = synthesis.get(array_key).and_then(Value::as_array) else {
            continue;
        };
        for (index, item) in items.iter().enumerate() {
            let path = format!(
                "$.outputs.pack_data.youtube_summary.synthesis.{array_key}[{index}].{id_key}"
            );
            match item.get(id_key).and_then(Value::as_str).map(str::trim) {
                Some(value) if !value.is_empty() => {
                    if !seen.insert(value.to_string()) {
                        findings.push(finding(
                            "error",
                            "RV-RESULT-001",
                            format!("duplicate synthesis item id `{value}`"),
                            Some(path),
                        ));
                    }
                }
                _ => findings.push(finding(
                    "error",
                    "RV-RESULT-004",
                    format!("{id_key} must be a non-empty string"),
                    Some(path),
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_youtube_summary_canonical_result_valid_minimal_has_no_errors() {
        let findings = validate_youtube_summary_canonical_result(
            &valid_canonical_result(),
            &context("complete", "standard"),
        );

        assert!(!has_error(&findings), "{findings:#?}");
    }

    #[test]
    fn duplicate_source_ref_id_returns_error() {
        let mut canonical = valid_canonical_result();
        canonical["source_refs"] = serde_json::json!([
            { "source_ref_id": "source_ref_1", "source_snapshot_id": 501, "title": "Video 1" },
            { "source_ref_id": "source_ref_1", "source_snapshot_id": 502, "title": "Video 2" }
        ]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(&findings, "RV-RESULT-001", "$.source_refs[1].source_ref_id");
    }

    #[test]
    fn missing_required_top_level_array_returns_error() {
        let mut canonical = valid_canonical_result();
        canonical.as_object_mut().expect("canonical object").remove("claims");

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(&findings, "RV-RESULT-003", "$.claims");
    }

    #[test]
    fn run_id_mismatch_returns_error() {
        let mut canonical = valid_canonical_result();
        canonical["run_id"] = serde_json::json!(43);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(&findings, "RV-RESULT-003", "$.run_id");
    }

    #[test]
    fn blank_video_id_returns_error() {
        let mut canonical = valid_canonical_result();
        canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["video_id"] =
            serde_json::json!(" ");

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "RV-RESULT-004",
            "$.outputs.pack_data.youtube_summary.videos[0].video_id",
        );
    }

    #[test]
    fn duplicate_video_id_returns_error() {
        let mut canonical = valid_canonical_result();
        canonical["outputs"]["pack_data"]["youtube_summary"]["videos"] = serde_json::json!([
            {
                "video_id": "video_1",
                "source_ref_id": "source_ref_1",
                "title": "Video 1",
                "summary_text": "Summary 1"
            },
            {
                "video_id": "video_1",
                "source_ref_id": "source_ref_1",
                "title": "Video 2",
                "summary_text": "Summary 2"
            }
        ]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "RV-RESULT-001",
            "$.outputs.pack_data.youtube_summary.videos[1].video_id",
        );
    }

    #[test]
    fn duplicate_claim_id_returns_error() {
        let mut canonical = valid_canonical_result();
        canonical["claims"] = serde_json::json!([
            { "claim_id": "claim_1", "source_ref_id": "source_ref_1", "text": "Claim 1" },
            { "claim_id": "claim_1", "source_ref_id": "source_ref_1", "text": "Claim 2" }
        ]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(&findings, "RV-RESULT-001", "$.claims[1].claim_id");
    }

    #[test]
    fn duplicate_evidence_id_returns_error() {
        let mut canonical = valid_canonical_result();
        canonical["evidence"] = serde_json::json!([
            {
                "evidence_id": "evidence_1",
                "claim_id": "claim_1",
                "source_ref_id": "source_ref_1",
                "text": "Evidence 1"
            },
            {
                "evidence_id": "evidence_1",
                "claim_id": "claim_1",
                "source_ref_id": "source_ref_1",
                "text": "Evidence 2"
            }
        ]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(&findings, "RV-RESULT-001", "$.evidence[1].evidence_id");
    }

    #[test]
    fn synthesis_object_missing_required_array_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]
            .as_object_mut()
            .expect("synthesis object")
            .remove("relation_refs");

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "VR-YS-001",
            "$.outputs.pack_data.youtube_summary.synthesis.relation_refs",
        );
    }

    #[test]
    fn duplicate_synthesis_item_id_across_item_kinds_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["common_claims"][0]
            ["common_claim_id"] = serde_json::json!("theme_1");

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "RV-RESULT-001",
            "$.outputs.pack_data.youtube_summary.synthesis.common_claims[0].common_claim_id",
        );
    }

    fn context(terminal_status: &str, evidence_mode: &str) -> YoutubeSummaryResultValidationContext {
        YoutubeSummaryResultValidationContext::new(42, terminal_status, evidence_mode)
    }

    fn assert_has_error(
        findings: &[PromptPackResultValidationFinding],
        code: &str,
        object_path: &str,
    ) {
        assert!(
            findings.iter().any(|finding| {
                finding.severity == "error"
                    && finding.code == code
                    && finding.object_path.as_deref() == Some(object_path)
            }),
            "missing {code} at {object_path}; findings: {findings:#?}"
        );
    }

    fn valid_canonical_result() -> Value {
        serde_json::json!({
            "schema_version": "1.0",
            "result_id": "result_42",
            "run_id": 42,
            "pack_id": "youtube_summary",
            "pack_version": "1.0.0",
            "stage": "youtube_summary/transcript_analysis",
            "created_at": "2026-06-14T00:00:00Z",
            "output_language": "en",
            "metadata": {},
            "run_context": {},
            "outputs": {
                "pack_data": {
                    "youtube_summary": {
                        "videos": [{
                            "video_id": "video_1",
                            "source_ref_id": "source_ref_1",
                            "title": "Video",
                            "summary_text": "Summary"
                        }],
                        "synthesis": null
                    }
                },
                "sections": []
            },
            "source_refs": [{
                "source_ref_id": "source_ref_1",
                "source_snapshot_id": 501,
                "title": "Video"
            }],
            "claims": [{
                "claim_id": "claim_1",
                "source_ref_id": "source_ref_1",
                "text": "Claim"
            }],
            "evidence": [{
                "evidence_id": "evidence_1",
                "claim_id": "claim_1",
                "source_ref_id": "source_ref_1",
                "text": "Evidence"
            }],
            "warnings": [],
            "limitations": [],
            "quality_flags": [],
            "audit_refs": []
        })
    }

    fn valid_canonical_result_with_synthesis() -> Value {
        let mut canonical = valid_canonical_result();
        canonical["source_refs"] = serde_json::json!([
            { "source_ref_id": "source_ref_1", "source_snapshot_id": 501, "title": "Video 1" },
            { "source_ref_id": "source_ref_2", "source_snapshot_id": 502, "title": "Video 2" }
        ]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["videos"] = serde_json::json!([
            {
                "video_id": "video_1",
                "source_ref_id": "source_ref_1",
                "title": "Video 1",
                "summary_text": "Summary 1"
            },
            {
                "video_id": "video_2",
                "source_ref_id": "source_ref_2",
                "title": "Video 2",
                "summary_text": "Summary 2"
            }
        ]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"] = serde_json::json!({
            "cross_video_themes": [{
                "theme_id": "theme_1",
                "theme_text": "Shared theme",
                "video_refs": ["video_1", "video_2"],
                "source_refs": ["source_ref_1", "source_ref_2"],
                "claim_refs": ["claim_1"],
                "evidence_refs": ["evidence_1"]
            }],
            "common_claims": [{
                "common_claim_id": "common_claim_1",
                "summary_text": "Both videos mention pilots.",
                "video_refs": ["video_1", "video_2"],
                "source_refs": ["source_ref_1", "source_ref_2"],
                "claim_refs": ["claim_1"],
                "evidence_refs": ["evidence_1"]
            }],
            "contradictions_across_videos": [{
                "contradiction_id": "contradiction_1",
                "description": "Different conclusions.",
                "video_refs": ["video_1", "video_2"],
                "source_refs": ["source_ref_1", "source_ref_2"],
                "claim_refs": ["claim_1"],
                "evidence_refs": ["evidence_1"]
            }],
            "claim_refs": ["claim_1"],
            "relation_refs": [],
            "evidence_refs": ["evidence_1"],
            "source_refs": ["source_ref_1", "source_ref_2"]
        });
        canonical
    }
}
