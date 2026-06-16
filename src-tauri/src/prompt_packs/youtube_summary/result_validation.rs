use std::collections::{HashMap, HashSet};

use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::error::{AppError, AppResult};
use crate::prompt_packs::projections::persist_final_result_in_transaction;

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
    expect_string_value(
        canonical,
        "pack_id",
        "youtube_summary",
        "$.pack_id",
        &mut findings,
    );
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

    let source_ids = collect_string_ids(
        canonical.get("source_refs").and_then(Value::as_array),
        "source_ref_id",
    );
    let video_ids = collect_string_ids(
        canonical
            .pointer("/outputs/pack_data/youtube_summary/videos")
            .and_then(Value::as_array),
        "video_id",
    );
    let claim_ids = collect_string_ids(
        canonical.get("claims").and_then(Value::as_array),
        "claim_id",
    );
    let evidence_ids = collect_string_ids(
        canonical.get("evidence").and_then(Value::as_array),
        "evidence_id",
    );

    validate_result_refs(
        canonical,
        &source_ids,
        &video_ids,
        &claim_ids,
        &evidence_ids,
        &mut findings,
    );
    let videos = canonical
        .pointer("/outputs/pack_data/youtube_summary/videos")
        .and_then(Value::as_array);
    let video_source_by_id = collect_video_source_by_id(videos, &source_ids);
    if let Some(synthesis) = canonical
        .pointer("/outputs/pack_data/youtube_summary/synthesis")
        .and_then(Value::as_object)
    {
        validate_synthesis_derived_traversal_refs(
            synthesis,
            &video_source_by_id,
            &mut findings,
        );
    }
    validate_youtube_pack_rules(canonical, context, &mut findings);
    add_advisory_quality_flag_findings(canonical, &mut findings);

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

#[cfg(test)]
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

fn collect_string_ids(items: Option<&Vec<Value>>, key: &str) -> HashSet<String> {
    items
        .into_iter()
        .flatten()
        .filter_map(|item| item.get(key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn validate_result_refs(
    canonical: &Value,
    source_ids: &HashSet<String>,
    video_ids: &HashSet<String>,
    claim_ids: &HashSet<String>,
    evidence_ids: &HashSet<String>,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    validate_object_ref_field(
        canonical
            .pointer("/outputs/pack_data/youtube_summary/videos")
            .and_then(Value::as_array),
        "source_ref_id",
        source_ids,
        "$.outputs.pack_data.youtube_summary.videos",
        findings,
    );
    validate_nullable_object_ref_field(
        canonical.get("claims").and_then(Value::as_array),
        "source_ref_id",
        source_ids,
        "$.claims",
        findings,
    );
    validate_nullable_object_ref_field(
        canonical.get("evidence").and_then(Value::as_array),
        "source_ref_id",
        source_ids,
        "$.evidence",
        findings,
    );
    validate_nullable_object_ref_field(
        canonical.get("evidence").and_then(Value::as_array),
        "claim_id",
        claim_ids,
        "$.evidence",
        findings,
    );

    let Some(synthesis) = canonical
        .pointer("/outputs/pack_data/youtube_summary/synthesis")
        .and_then(Value::as_object)
    else {
        return;
    };

    validate_ref_array(
        synthesis.get("source_refs"),
        source_ids,
        "$.outputs.pack_data.youtube_summary.synthesis.source_refs",
        findings,
    );
    validate_ref_array(
        synthesis.get("claim_refs"),
        claim_ids,
        "$.outputs.pack_data.youtube_summary.synthesis.claim_refs",
        findings,
    );
    validate_ref_array(
        synthesis.get("evidence_refs"),
        evidence_ids,
        "$.outputs.pack_data.youtube_summary.synthesis.evidence_refs",
        findings,
    );

    for array_key in [
        "cross_video_themes",
        "common_claims",
        "contradictions_across_videos",
    ] {
        let Some(items) = synthesis.get(array_key).and_then(Value::as_array) else {
            continue;
        };
        for (index, item) in items.iter().enumerate() {
            let base =
                format!("$.outputs.pack_data.youtube_summary.synthesis.{array_key}[{index}]");
            validate_ref_array(
                item.get("source_refs"),
                source_ids,
                &format!("{base}.source_refs"),
                findings,
            );
            validate_ref_array(
                item.get("claim_refs"),
                claim_ids,
                &format!("{base}.claim_refs"),
                findings,
            );
            validate_ref_array(
                item.get("evidence_refs"),
                evidence_ids,
                &format!("{base}.evidence_refs"),
                findings,
            );
            validate_ref_array(
                item.get("video_refs"),
                video_ids,
                &format!("{base}.video_refs"),
                findings,
            );
        }
    }
}

fn validate_object_ref_field(
    items: Option<&Vec<Value>>,
    field: &str,
    allowed: &HashSet<String>,
    base_path: &str,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    for (index, item) in items.into_iter().flatten().enumerate() {
        let path = format!("{base_path}[{index}].{field}");
        match item.get(field).and_then(Value::as_str).map(str::trim) {
            Some(value) if allowed.contains(value) => {}
            Some(value) => findings.push(finding(
                "error",
                "RV-RESULT-002",
                format!("unknown {field} `{value}`"),
                Some(path),
            )),
            None => findings.push(finding(
                "error",
                "RV-RESULT-004",
                format!("{field} must be a non-empty string"),
                Some(path),
            )),
        }
    }
}

fn validate_nullable_object_ref_field(
    items: Option<&Vec<Value>>,
    field: &str,
    allowed: &HashSet<String>,
    base_path: &str,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    for (index, item) in items.into_iter().flatten().enumerate() {
        let Some(value) = item.get(field) else {
            continue;
        };
        if value.is_null() {
            continue;
        }
        let path = format!("{base_path}[{index}].{field}");
        match value.as_str().map(str::trim) {
            Some(ref_id) if allowed.contains(ref_id) => {}
            Some(ref_id) => findings.push(finding(
                "error",
                "RV-RESULT-002",
                format!("unknown {field} `{ref_id}`"),
                Some(path),
            )),
            None => findings.push(finding(
                "error",
                "RV-RESULT-003",
                format!("{field} must be a string when present"),
                Some(path),
            )),
        }
    }
}

fn validate_ref_array(
    value: Option<&Value>,
    allowed: &HashSet<String>,
    base_path: &str,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    for (index, item) in value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        // Applies to top-level synthesis refs and nested synthesis item refs,
        // including video_refs/source_refs/claim_refs/evidence_refs. Non-string
        // ref items are not detected in this result-validation MVP; raw
        // synthesis-output validation owns that before canonical assembly.
        let Some(ref_id) = item.as_str() else {
            continue;
        };
        if !allowed.contains(ref_id) {
            findings.push(finding(
                "error",
                "RV-RESULT-002",
                format!("unknown ref `{ref_id}`"),
                Some(format!("{base_path}[{index}]")),
            ));
        }
    }
}

fn collect_video_source_by_id(
    videos: Option<&Vec<Value>>,
    source_ids: &HashSet<String>,
) -> HashMap<String, String> {
    let mut video_source_by_id = HashMap::new();
    for video in videos.into_iter().flatten() {
        let Some(video_id) = video.get("video_id").and_then(Value::as_str).map(str::trim) else {
            continue;
        };
        if video_id.is_empty() {
            continue;
        }
        let Some(source_ref_id) = video
            .get("source_ref_id")
            .and_then(Value::as_str)
            .map(str::trim)
        else {
            continue;
        };
        if source_ids.contains(source_ref_id) {
            video_source_by_id.insert(video_id.to_string(), source_ref_id.to_string());
        }
    }
    video_source_by_id
}

fn synthesis_item_arrays<'a>(
    synthesis: &'a serde_json::Map<String, Value>,
) -> impl Iterator<Item = &'a Vec<Value>> {
    [
        "cross_video_themes",
        "common_claims",
        "contradictions_across_videos",
    ]
    .into_iter()
    .filter_map(|key| synthesis.get(key).and_then(Value::as_array))
}

fn push_ordered_unique(values: &mut Vec<String>, seen: &mut HashSet<String>, value: &str) {
    if !seen.contains(value) {
        let value = value.to_string();
        seen.insert(value.clone());
        values.push(value);
    }
}

fn collect_ordered_unique_ref_array_items(
    value: Option<&Value>,
    values: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    for item in value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        push_ordered_unique(values, seen, item);
    }
}

fn ref_array_strings(value: Option<&Value>) -> Option<Vec<String>> {
    // Synthesis traversal arrays are canonical-builder output. Non-string refs
    // are intentionally ignored here because raw synthesis-output validation
    // owns that shape error before canonical assembly.
    value.and_then(Value::as_array).map(|items| {
        items
            .iter()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect()
    })
}

fn derive_synthesis_claim_refs(synthesis: &serde_json::Map<String, Value>) -> Vec<String> {
    let mut values = Vec::new();
    let mut seen = HashSet::new();
    for items in synthesis_item_arrays(synthesis) {
        for item in items {
            collect_ordered_unique_ref_array_items(item.get("claim_refs"), &mut values, &mut seen);
        }
    }
    values
}

fn derive_synthesis_evidence_refs(synthesis: &serde_json::Map<String, Value>) -> Vec<String> {
    let mut values = Vec::new();
    let mut seen = HashSet::new();
    for items in synthesis_item_arrays(synthesis) {
        for item in items {
            collect_ordered_unique_ref_array_items(
                item.get("evidence_refs"),
                &mut values,
                &mut seen,
            );
        }
    }
    values
}

fn derive_synthesis_source_refs(
    synthesis: &serde_json::Map<String, Value>,
    video_source_by_id: &HashMap<String, String>,
) -> Vec<String> {
    let mut values = Vec::new();
    let mut seen = HashSet::new();
    for items in synthesis_item_arrays(synthesis) {
        for item in items {
            collect_ordered_unique_ref_array_items(item.get("source_refs"), &mut values, &mut seen);
            for video_id in item
                .get("video_refs")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                if let Some(source_ref_id) = video_source_by_id.get(video_id) {
                    push_ordered_unique(&mut values, &mut seen, source_ref_id);
                }
            }
        }
    }
    values
}

fn validate_synthesis_derived_traversal_refs(
    synthesis: &serde_json::Map<String, Value>,
    video_source_by_id: &HashMap<String, String>,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    compare_derived_traversal_refs(
        synthesis.get("claim_refs"),
        derive_synthesis_claim_refs(synthesis),
        "$.outputs.pack_data.youtube_summary.synthesis.claim_refs",
        findings,
    );
    compare_derived_traversal_refs(
        synthesis.get("evidence_refs"),
        derive_synthesis_evidence_refs(synthesis),
        "$.outputs.pack_data.youtube_summary.synthesis.evidence_refs",
        findings,
    );
    compare_derived_traversal_refs(
        synthesis.get("source_refs"),
        derive_synthesis_source_refs(synthesis, video_source_by_id),
        "$.outputs.pack_data.youtube_summary.synthesis.source_refs",
        findings,
    );
}

fn compare_derived_traversal_refs(
    actual_value: Option<&Value>,
    expected: Vec<String>,
    path: &str,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    let Some(actual) = ref_array_strings(actual_value) else {
        return;
    };

    let mut duplicate_seen = HashSet::new();
    let has_duplicate = actual
        .iter()
        .any(|value| !duplicate_seen.insert(value.to_string()));

    let actual_set: HashSet<String> = actual.iter().cloned().collect();
    let expected_set: HashSet<String> = expected.iter().cloned().collect();

    let missing: Vec<String> = expected
        .iter()
        .filter(|value| !actual_set.contains(*value))
        .cloned()
        .collect();
    let extra: Vec<String> = actual
        .iter()
        .filter(|value| !expected_set.contains(*value))
        .cloned()
        .collect();

    if has_duplicate || !missing.is_empty() || !extra.is_empty() {
        let mut parts = Vec::new();
        if has_duplicate {
            parts.push("duplicates present".to_string());
        }
        if !missing.is_empty() {
            parts.push(format!("missing: {}", serde_json::json!(missing)));
        }
        if !extra.is_empty() {
            parts.push(format!("extra: {}", serde_json::json!(extra)));
        }
        findings.push(finding(
            "error",
            "VR-YS-015",
            format!("{path} {}", parts.join("; ")),
            Some(path.to_string()),
        ));
    }
}

fn validate_youtube_pack_rules(
    canonical: &Value,
    context: &YoutubeSummaryResultValidationContext,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    let videos = canonical
        .pointer("/outputs/pack_data/youtube_summary/videos")
        .and_then(Value::as_array);
    if context.terminal_status == "complete"
        && context.evidence_mode != "narrative_only"
        && videos.is_some_and(Vec::is_empty)
    {
        findings.push(finding(
            "error",
            "VR-YS-002",
            "complete YouTube Summary result must include videos outside narrative_only evidence mode",
            Some("$.outputs.pack_data.youtube_summary.videos".to_string()),
        ));
    }

    let synthesis = canonical.pointer("/outputs/pack_data/youtube_summary/synthesis");
    if videos.is_some_and(|items| items.len() == 1) && synthesis.is_some_and(Value::is_object) {
        findings.push(finding(
            "error",
            "VR-YS-005",
            "single-video YouTube Summary result must not include cross-video synthesis object",
            Some("$.outputs.pack_data.youtube_summary.synthesis".to_string()),
        ));
    }
}

fn add_advisory_quality_flag_findings(
    canonical: &Value,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    let Some(flags) = canonical.get("quality_flags").and_then(Value::as_array) else {
        return;
    };
    for (index, item) in flags.iter().enumerate() {
        let Some(flag) = item.get("flag").and_then(Value::as_str) else {
            continue;
        };
        let advisory = match flag {
            "intermediate_entities_legacy_fallback" => Some((
                "warning",
                "Canonical result used legacy parsed-output assembly for claims/evidence.",
            )),
            "synthesis_not_applicable_single_video" => Some((
                "info",
                "Synthesis was intentionally skipped for a single-video run.",
            )),
            "synthesis_failed" => Some((
                "warning",
                "Synthesis failed; result was persisted without cross-video synthesis.",
            )),
            "synthesis_skipped_insufficient_successes" => Some((
                "warning",
                "Synthesis was skipped because fewer than two transcript stages succeeded.",
            )),
            _ => None,
        };
        if let Some((severity, message)) = advisory {
            findings.push(finding(
                severity,
                "RV-RESULT-005",
                message,
                Some(format!("$.quality_flags[{index}]")),
            ));
        }
    }
}

pub(crate) async fn validate_and_persist_final_result_transaction(
    pool: &SqlitePool,
    run_id: i64,
    canonical_result: Value,
    terminal_status: &str,
) -> AppResult<()> {
    validate_and_persist_final_result_transaction_internal(
        pool,
        run_id,
        canonical_result,
        terminal_status,
        |_| {},
    )
    .await
}

#[cfg(test)]
pub(crate) async fn validate_and_persist_final_result_transaction_with_result_mutator_for_test<M>(
    pool: &SqlitePool,
    run_id: i64,
    canonical_result: Value,
    terminal_status: &str,
    mutate_after_validation: M,
) -> AppResult<()>
where
    M: FnOnce(&mut Value),
{
    validate_and_persist_final_result_transaction_internal(
        pool,
        run_id,
        canonical_result,
        terminal_status,
        mutate_after_validation,
    )
    .await
}

async fn validate_and_persist_final_result_transaction_internal<M>(
    pool: &SqlitePool,
    run_id: i64,
    mut canonical_result: Value,
    terminal_status: &str,
    mutate_after_validation: M,
) -> AppResult<()>
where
    M: FnOnce(&mut Value),
{
    let evidence_mode = load_run_evidence_mode(pool, run_id).await?;
    let context =
        YoutubeSummaryResultValidationContext::new(run_id, terminal_status, &evidence_mode);
    let findings = validate_youtube_summary_canonical_result(&canonical_result, &context);

    let mut tx = pool.begin().await.map_err(AppError::database)?;
    replace_result_level_findings_in_transaction(&mut tx, run_id, &findings).await?;

    if findings.iter().any(|finding| finding.severity == "error") {
        mark_result_validation_failed_in_transaction(&mut tx, run_id).await?;
        tx.commit().await.map_err(AppError::database)?;
        return Err(AppError::validation(
            "canonical result validation failed; result was not persisted",
        ));
    }

    mutate_after_validation(&mut canonical_result);
    persist_final_result_in_transaction(&mut tx, run_id, &canonical_result, terminal_status)
        .await?;
    tx.commit().await.map_err(AppError::database)?;
    Ok(())
}

async fn load_run_evidence_mode(pool: &SqlitePool, run_id: i64) -> AppResult<String> {
    sqlx::query_scalar::<_, String>("SELECT evidence_mode FROM prompt_pack_runs WHERE id = ?")
        .bind(run_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::database)
}

async fn replace_result_level_findings_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: i64,
    findings: &[PromptPackResultValidationFinding],
) -> AppResult<()> {
    sqlx::query(
        "DELETE FROM prompt_pack_result_validation_findings
         WHERE run_id = ? AND stage_run_id IS NULL",
    )
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;

    let now = crate::time::now_rfc3339_utc();
    for finding in findings {
        sqlx::query(
            "INSERT INTO prompt_pack_result_validation_findings (
                run_id, stage_run_id, severity, code, message, object_path, created_at
             )
             VALUES (?, NULL, ?, ?, ?, ?, ?)",
        )
        .bind(run_id)
        .bind(&finding.severity)
        .bind(&finding.code)
        .bind(&finding.message)
        .bind(&finding.object_path)
        .bind(&now)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    Ok(())
}

async fn mark_result_validation_failed_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: i64,
) -> AppResult<()> {
    let now = crate::time::now_rfc3339_utc();
    let message = "Canonical result validation failed; result was not persisted";
    sqlx::query("DELETE FROM prompt_pack_results WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = 'failed',
             result_status = 'failed',
             latest_message = ?,
             completed_at = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(message)
    .bind(&now)
    .bind(&now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    sqlx::query(
        "INSERT INTO prompt_pack_audit_events (run_id, event_kind, message, created_at)
         VALUES (?, 'terminal_result_validation_failed', ?, ?)",
    )
    .bind(run_id)
    .bind(message)
    .bind(&now)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
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
        canonical
            .as_object_mut()
            .expect("canonical object")
            .remove("claims");

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

    #[test]
    fn video_with_unknown_source_ref_returns_error() {
        let mut canonical = valid_canonical_result();
        canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["source_ref_id"] =
            serde_json::json!("source_ref_missing");

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "RV-RESULT-002",
            "$.outputs.pack_data.youtube_summary.videos[0].source_ref_id",
        );
    }

    #[test]
    fn evidence_with_unknown_claim_id_returns_error() {
        let mut canonical = valid_canonical_result();
        canonical["evidence"][0]["claim_id"] = serde_json::json!("claim_missing");

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(&findings, "RV-RESULT-002", "$.evidence[0].claim_id");
    }

    #[test]
    fn synthesis_top_level_unknown_claim_ref_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["claim_refs"] =
            serde_json::json!(["claim_missing"]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "RV-RESULT-002",
            "$.outputs.pack_data.youtube_summary.synthesis.claim_refs[0]",
        );
    }

    #[test]
    fn nested_synthesis_unknown_claim_ref_returns_error_when_top_level_union_empty() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["claim_refs"] =
            serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["cross_video_themes"]
            [0]["claim_refs"] = serde_json::json!(["claim_999"]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "RV-RESULT-002",
            "$.outputs.pack_data.youtube_summary.synthesis.cross_video_themes[0].claim_refs[0]",
        );
    }

    #[test]
    fn nested_synthesis_unknown_video_ref_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["cross_video_themes"]
            [0]["video_refs"] = serde_json::json!(["video_missing"]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "RV-RESULT-002",
            "$.outputs.pack_data.youtube_summary.synthesis.cross_video_themes[0].video_refs[0]",
        );
    }

    #[test]
    fn synthesis_missing_nested_claim_ref_in_top_level_union_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["claim_refs"] =
            serde_json::json!([]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "VR-YS-015",
            "$.outputs.pack_data.youtube_summary.synthesis.claim_refs",
        );
    }

    #[test]
    fn synthesis_extra_top_level_claim_ref_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["claims"] = serde_json::json!([
            { "claim_id": "claim_1", "source_ref_id": "source_ref_1", "text": "Claim 1" },
            { "claim_id": "claim_2", "source_ref_id": "source_ref_2", "text": "Claim 2" }
        ]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["claim_refs"] =
            serde_json::json!(["claim_1", "claim_2"]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "VR-YS-015",
            "$.outputs.pack_data.youtube_summary.synthesis.claim_refs",
        );
    }

    #[test]
    fn synthesis_duplicate_top_level_claim_ref_returns_error_at_field_path() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["claim_refs"] =
            serde_json::json!(["claim_1", "claim_1"]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "VR-YS-015",
            "$.outputs.pack_data.youtube_summary.synthesis.claim_refs",
        );
    }

    #[test]
    fn synthesis_missing_nested_evidence_ref_in_top_level_union_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["evidence_refs"] =
            serde_json::json!([]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "VR-YS-015",
            "$.outputs.pack_data.youtube_summary.synthesis.evidence_refs",
        );
    }

    #[test]
    fn synthesis_extra_top_level_evidence_ref_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["evidence"] = serde_json::json!([
            {
                "evidence_id": "evidence_1",
                "claim_id": "claim_1",
                "source_ref_id": "source_ref_1",
                "text": "Evidence 1"
            },
            {
                "evidence_id": "evidence_2",
                "claim_id": "claim_1",
                "source_ref_id": "source_ref_2",
                "text": "Evidence 2"
            }
        ]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["evidence_refs"] =
            serde_json::json!(["evidence_1", "evidence_2"]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "VR-YS-015",
            "$.outputs.pack_data.youtube_summary.synthesis.evidence_refs",
        );
    }

    #[test]
    fn synthesis_missing_source_ref_derived_from_video_ref_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        // The fixture has video_2 -> source_ref_2 and source_ref_2 is present in
        // canonical source_refs[]. The only missing value is in synthesis.source_refs.
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["source_refs"] =
            serde_json::json!(["source_ref_1"]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "VR-YS-015",
            "$.outputs.pack_data.youtube_summary.synthesis.source_refs",
        );
    }

    #[test]
    fn synthesis_extra_top_level_source_ref_not_in_nested_items_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["cross_video_themes"][0]
            ["video_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["cross_video_themes"][0]
            ["source_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["common_claims"][0]
            ["video_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["common_claims"][0]
            ["source_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]
            ["contradictions_across_videos"][0]["video_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]
            ["contradictions_across_videos"][0]["source_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["source_refs"] =
            serde_json::json!(["source_ref_2"]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "VR-YS-015",
            "$.outputs.pack_data.youtube_summary.synthesis.source_refs",
        );
    }

    #[test]
    fn synthesis_order_difference_in_top_level_union_is_allowed() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["source_refs"] =
            serde_json::json!(["source_ref_2", "source_ref_1"]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert!(
            findings.iter().all(|finding| {
                finding.code != "VR-YS-015"
                    || finding.object_path.as_deref()
                        != Some("$.outputs.pack_data.youtube_summary.synthesis.source_refs")
            }),
            "{findings:#?}"
        );
    }

    #[test]
    fn synthesis_unknown_video_ref_does_not_cascade_to_source_union_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["cross_video_themes"][0]
            ["video_refs"] = serde_json::json!(["video_missing"]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["cross_video_themes"][0]
            ["source_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["common_claims"][0]
            ["video_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["common_claims"][0]
            ["source_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]
            ["contradictions_across_videos"][0]["video_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]
            ["contradictions_across_videos"][0]["source_refs"] = serde_json::json!([]);
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["source_refs"] =
            serde_json::json!([]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "RV-RESULT-002",
            "$.outputs.pack_data.youtube_summary.synthesis.cross_video_themes[0].video_refs[0]",
        );
        assert!(
            findings.iter().all(|finding| {
                finding.code != "VR-YS-015"
                    || finding.object_path.as_deref()
                        != Some("$.outputs.pack_data.youtube_summary.synthesis.source_refs")
            }),
            "{findings:#?}"
        );
    }

    #[test]
    fn synthesis_null_skips_derived_traversal_validation() {
        let canonical = valid_canonical_result();

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert!(!findings.iter().any(|finding| finding.code == "VR-YS-015"));
    }

    #[test]
    fn complete_standard_result_with_empty_videos_returns_error() {
        let mut canonical = valid_canonical_result();
        canonical["outputs"]["pack_data"]["youtube_summary"]["videos"] = serde_json::json!([]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "VR-YS-002",
            "$.outputs.pack_data.youtube_summary.videos",
        );
    }

    #[test]
    fn complete_narrative_only_result_allows_empty_videos() {
        let mut canonical = valid_canonical_result();
        canonical["outputs"]["pack_data"]["youtube_summary"]["videos"] = serde_json::json!([]);

        let findings = validate_youtube_summary_canonical_result(
            &canonical,
            &context("complete", "narrative_only"),
        );

        assert!(
            findings.iter().all(|finding| finding.code != "VR-YS-002"),
            "{findings:#?}"
        );
    }

    #[test]
    fn single_video_with_synthesis_object_returns_error() {
        let mut canonical = valid_canonical_result_with_synthesis();
        canonical["outputs"]["pack_data"]["youtube_summary"]["videos"] = serde_json::json!([
            canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0].clone()
        ]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert_has_error(
            &findings,
            "VR-YS-005",
            "$.outputs.pack_data.youtube_summary.synthesis",
        );
    }

    #[test]
    fn known_quality_flag_emits_advisory_finding_without_error() {
        let mut canonical = valid_canonical_result();
        canonical["quality_flags"] = serde_json::json!([
            { "flag": "intermediate_entities_legacy_fallback", "severity": "warning" }
        ]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert!(!has_error(&findings), "{findings:#?}");
        assert!(
            findings.iter().any(|finding| {
                finding.severity == "warning"
                    && finding.code == "RV-RESULT-005"
                    && finding.object_path.as_deref() == Some("$.quality_flags[0]")
            }),
            "{findings:#?}"
        );
    }

    #[test]
    fn unknown_quality_flag_is_ignored_by_mvp_validator() {
        let mut canonical = valid_canonical_result();
        canonical["quality_flags"] = serde_json::json!([
            { "flag": "custom_future_flag", "severity": "warning" }
        ]);

        let findings =
            validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

        assert!(!findings
            .iter()
            .any(|finding| finding.code == "RV-RESULT-005"));
    }

    #[tokio::test]
    async fn validation_persistence_writes_warning_findings_and_persists_result() {
        let pool = test_pool_with_canonical_result_ready().await;
        let mut canonical = valid_canonical_result();
        canonical["quality_flags"] = serde_json::json!([
            { "flag": "intermediate_entities_legacy_fallback", "severity": "warning" }
        ]);

        validate_and_persist_final_result_transaction(&pool, 42, canonical, "complete")
            .await
            .expect("persist valid result with warning");

        assert_eq!(count(&pool, "prompt_pack_results").await, 1);
        assert_eq!(count(&pool, "prompt_pack_youtube_videos").await, 1);
        let finding: (Option<i64>, String, String) = sqlx::query_as(
            "SELECT stage_run_id, severity, code
             FROM prompt_pack_result_validation_findings
             WHERE run_id = 42",
        )
        .fetch_one(&pool)
        .await
        .expect("warning finding");
        assert_eq!(
            finding,
            (None, "warning".to_string(), "RV-RESULT-005".to_string())
        );
    }

    #[tokio::test]
    async fn validation_persistence_replaces_previous_result_level_findings_on_success() {
        let pool = test_pool_with_canonical_result_ready().await;
        let mut first = valid_canonical_result();
        first["quality_flags"] = serde_json::json!([
            { "flag": "intermediate_entities_legacy_fallback", "severity": "warning" }
        ]);
        validate_and_persist_final_result_transaction(&pool, 42, first, "complete")
            .await
            .expect("first persist");

        let mut second = valid_canonical_result();
        second["quality_flags"] = serde_json::json!([
            { "flag": "synthesis_not_applicable_single_video", "severity": "info" }
        ]);
        validate_and_persist_final_result_transaction(&pool, 42, second, "complete")
            .await
            .expect("second persist");

        let findings: Vec<(String, String)> = sqlx::query_as(
            "SELECT severity, message
             FROM prompt_pack_result_validation_findings
             WHERE run_id = 42 AND stage_run_id IS NULL
             ORDER BY id ASC",
        )
        .fetch_all(&pool)
        .await
        .expect("result findings");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].0, "info");
        assert!(findings[0].1.contains("single-video run"));
    }

    #[tokio::test]
    async fn validation_error_writes_findings_marks_run_failed_and_skips_result() {
        let pool = test_pool_with_canonical_result_ready().await;
        let mut canonical = valid_canonical_result();
        canonical["pack_id"] = serde_json::json!("other_pack");

        let error = validate_and_persist_final_result_transaction(&pool, 42, canonical, "complete")
            .await
            .expect_err("validation failure");

        assert!(
            error.message.contains("canonical result validation failed"),
            "{error:?}"
        );
        assert_eq!(count(&pool, "prompt_pack_results").await, 0);
        assert_eq!(
            count(&pool, "prompt_pack_result_validation_findings").await,
            1
        );
        let run: (String, String) =
            sqlx::query_as("SELECT run_status, result_status FROM prompt_pack_runs WHERE id = 42")
                .fetch_one(&pool)
                .await
                .expect("run status");
        assert_eq!(run, ("failed".to_string(), "failed".to_string()));
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_audit_events
             WHERE run_id = 42 AND event_kind = 'terminal_result_validation_failed'",
        )
        .fetch_one(&pool)
        .await
        .expect("audit count");
        assert_eq!(audit_count, 1);
    }

    #[tokio::test]
    async fn validation_error_keeps_stage_level_findings() {
        let pool = test_pool_with_canonical_result_ready().await;
        sqlx::query(
            "INSERT INTO prompt_pack_stage_runs (
                id, run_id, stage_name, stage_order, stage_status, created_at, updated_at
             )
             VALUES (77, 42, 'youtube_summary/transcript_analysis', 1, 'failed',
                '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("stage row");
        sqlx::query(
            "INSERT INTO prompt_pack_result_validation_findings (
                run_id, stage_run_id, severity, code, message, object_path, created_at
             )
             VALUES (42, 77, 'error', 'VR-STAGE-001', 'stage finding', '$', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("stage finding");

        let mut canonical = valid_canonical_result();
        canonical["pack_id"] = serde_json::json!("other_pack");

        validate_and_persist_final_result_transaction(&pool, 42, canonical, "complete")
            .await
            .expect_err("validation failure");

        let stage_findings: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_result_validation_findings
             WHERE run_id = 42 AND stage_run_id = 77",
        )
        .fetch_one(&pool)
        .await
        .expect("stage finding count");
        let result_findings: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_result_validation_findings
             WHERE run_id = 42 AND stage_run_id IS NULL",
        )
        .fetch_one(&pool)
        .await
        .expect("result finding count");

        assert_eq!(stage_findings, 1);
        assert_eq!(result_findings, 1);
    }

    #[tokio::test]
    async fn validation_error_removes_stale_persisted_result_and_projections() {
        let pool = test_pool_with_canonical_result_ready().await;
        validate_and_persist_final_result_transaction(
            &pool,
            42,
            valid_canonical_result(),
            "complete",
        )
        .await
        .expect("initial persist");

        let mut invalid = valid_canonical_result();
        invalid["claims"][0]["claim_id"] = serde_json::json!("");
        validate_and_persist_final_result_transaction(&pool, 42, invalid, "complete")
            .await
            .expect_err("validation failure");

        assert_eq!(count(&pool, "prompt_pack_results").await, 0);
        assert_eq!(count(&pool, "prompt_pack_youtube_videos").await, 0);
    }

    #[tokio::test]
    async fn validation_wrapper_rolls_back_result_findings_when_persistence_fails_after_validation()
    {
        let pool = test_pool_with_canonical_result_ready().await;
        sqlx::query(
            "INSERT INTO prompt_pack_result_validation_findings (
                run_id, stage_run_id, severity, code, message, object_path, created_at
             )
             VALUES (42, NULL, 'warning', 'RV-RESULT-005', 'old finding', '$.quality_flags[0]',
                '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("old result finding");

        let mut canonical = valid_canonical_result();
        canonical["quality_flags"] = serde_json::json!([
            { "flag": "intermediate_entities_legacy_fallback", "severity": "warning" }
        ]);

        let error = validate_and_persist_final_result_transaction_with_result_mutator_for_test(
            &pool,
            42,
            canonical,
            "complete",
            |canonical| {
                canonical["source_refs"] = serde_json::json!([
                    { "source_ref_id": "source_ref_1", "source_snapshot_id": 501, "title": "Video" },
                    { "source_ref_id": "source_ref_1", "source_snapshot_id": 502, "title": "Duplicate" }
                ]);
            },
        )
        .await
        .expect_err("low-level projection persistence should fail");

        assert!(error.message.contains("Database error"), "{error:?}");
        assert_eq!(count(&pool, "prompt_pack_results").await, 0);
        let findings: Vec<String> = sqlx::query_scalar(
            "SELECT message FROM prompt_pack_result_validation_findings
             WHERE run_id = 42 AND stage_run_id IS NULL
             ORDER BY id ASC",
        )
        .fetch_all(&pool)
        .await
        .expect("result findings");

        assert_eq!(findings, vec!["old finding".to_string()]);
    }

    fn context(
        terminal_status: &str,
        evidence_mode: &str,
    ) -> YoutubeSummaryResultValidationContext {
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

    async fn test_pool_with_canonical_result_ready() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        crate::migrations::apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed");
        sqlx::query(
            "INSERT INTO prompt_pack_runs (
                id, pack_version_id, pack_id, pack_version, schema_version,
                run_status, result_status, output_language, control_preset,
                evidence_mode, include_comments, created_at, updated_at
             )
             VALUES (42, 1, 'youtube_summary', '1.0.0', '1.0',
                'running', 'none', 'en', 'standard', 'standard', 0,
                '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert run");
        pool
    }

    async fn count(pool: &sqlx::SqlitePool, table: &str) -> i64 {
        sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table} WHERE run_id = 42"))
            .fetch_one(pool)
            .await
            .expect("count")
    }
}
