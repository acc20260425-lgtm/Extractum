# YouTube Summary Result Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a backend result-level validation gate for YouTube Summary canonical results before final result/projection persistence.

**Architecture:** Implement a pure pack-specific validator in `youtube_summary/result_validation.rs`, then add an atomic validation-aware persistence wrapper that stores result-level findings and either persists projections or marks the run failed. Refactor the existing final persistence path so both old and new callers use one transaction-aware projection helper.

**Tech Stack:** Rust, Tauri backend, `serde_json`, `sqlx` SQLite transactions, existing Prompt Pack migrations and test helpers.

---

## File Structure

- Create `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`
  - Owns `PromptPackResultValidationFinding`, `YoutubeSummaryResultValidationContext`, pure canonical-result validation, result-level finding persistence, and `validate_and_persist_final_result_transaction`.
  - Keeps YouTube Summary validation near the pack execution code.
- Modify `src-tauri/src/prompt_packs/youtube_summary/mod.rs`
  - Adds `mod result_validation;`.
- Modify `src-tauri/src/prompt_packs/projections.rs`
  - Extracts transaction-aware result/projection persistence helpers.
  - Keeps `persist_final_result_transaction(&SqlitePool, ...)` as the public thin wrapper.
- Modify `src-tauri/src/prompt_packs/youtube_summary/execution.rs`
  - Replaces direct `persist_final_result_transaction` call with `validate_and_persist_final_result_transaction`.
- Modify tests in the same modules
  - Pure validator tests live inside `result_validation.rs`.
  - Projection refactor tests stay in `projections.rs`.
  - Execution regression tests stay in `youtube_summary/execution_tests.rs`.

## Validation Rule Codes

Use stable codes only:

- `VR-YS-001`: missing YouTube pack shape fields.
- `VR-YS-002`: complete non-`narrative_only` result has empty `videos`.
- `VR-YS-005`: single-video result must have `synthesis = null`.
- `RV-RESULT-001`: duplicate backend-owned id.
- `RV-RESULT-002`: unknown reference.
- `RV-RESULT-003`: invalid canonical shape outside an existing `VR-*` rule.
- `RV-RESULT-004`: missing or blank backend-owned id.
- `RV-RESULT-005`: advisory quality flag surfaced as finding.

## Task 1: Add Pure Validator Types And First Failing Tests

**Files:**
- Create: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/mod.rs`

- [ ] **Step 1: Register the new module**

Add this line to `src-tauri/src/prompt_packs/youtube_summary/mod.rs` with the other private modules:

```rust
mod result_validation;
```

- [ ] **Step 2: Create the validator file with types and a minimal fixture**

Create `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs` with this starting content:

```rust
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
    let _ = canonical;
    let _ = context;
    Vec::new()
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

        let findings = validate_youtube_summary_canonical_result(
            &canonical,
            &context("complete", "standard"),
        );

        assert_has_error(&findings, "RV-RESULT-001", "$.source_refs[1].source_ref_id");
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
}
```

- [ ] **Step 3: Run the focused test and confirm the expected failure**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib duplicate_source_ref_id_returns_error
```

Expected: FAIL because `validate_youtube_summary_canonical_result` currently returns no findings.

- [ ] **Step 4: Keep the failing validator shell uncommitted**

Do not commit this red state if the branch policy is green-only. Leave the
new validator file staged or unstaged locally and make the first commit after
Task 2 passes.

## Task 2: Implement Shape And Identity Rules

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`

- [ ] **Step 1: Add shape and identity tests**

Add these tests to the existing `tests` module:

```rust
#[test]
fn missing_required_top_level_array_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical.as_object_mut().expect("canonical object").remove("claims");

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

    assert_has_error(&findings, "RV-RESULT-003", "$.claims");
}

#[test]
fn run_id_mismatch_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["run_id"] = serde_json::json!(43);

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

    assert_has_error(&findings, "RV-RESULT-003", "$.run_id");
}

#[test]
fn blank_video_id_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["video_id"] =
        serde_json::json!(" ");

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

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

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

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

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

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

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

    assert_has_error(&findings, "RV-RESULT-001", "$.evidence[1].evidence_id");
}

#[test]
fn synthesis_object_missing_required_array_returns_error() {
    let mut canonical = valid_canonical_result_with_synthesis();
    canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]
        .as_object_mut()
        .expect("synthesis object")
        .remove("relation_refs");

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

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

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

    assert_has_error(
        &findings,
        "RV-RESULT-001",
        "$.outputs.pack_data.youtube_summary.synthesis.common_claims[0].common_claim_id",
    );
}
```

Add this fixture helper:

```rust
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
```

- [ ] **Step 2: Run shape/identity tests and confirm they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation
```

Expected: the new invalid-result tests fail because the validator does not inspect shape or ids yet.

- [ ] **Step 3: Implement shape and identity validation**

Replace the body of `validate_youtube_summary_canonical_result` and add helpers:

```rust
use std::collections::HashSet;

pub(crate) fn validate_youtube_summary_canonical_result(
    canonical: &Value,
    context: &YoutubeSummaryResultValidationContext,
) -> Vec<PromptPackResultValidationFinding> {
    let mut findings = Vec::new();

    expect_string_value(canonical, "schema_version", "1.0", "$.schema_version", &mut findings);
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
        canonical.pointer("/outputs/pack_data/youtube_summary/videos").and_then(Value::as_array),
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
```

- [ ] **Step 4: Run tests and verify shape/identity pass**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation
```

Expected: PASS for the validator tests added so far.

- [ ] **Step 5: Commit the green validator shell plus shape and identity validation**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\mod.rs src-tauri\src\prompt_packs\youtube_summary\result_validation.rs
git commit -m "feat: validate youtube summary result shape and ids"
```

## Task 3: Implement Reference Rules, Pack Rules, And Advisory Findings

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`

`synthesis.relation_refs` is only shape-validated in this MVP. The current
canonical result does not expose a top-level relation registry such as
`claim_relations`, so known-relation validation and derived-union consistency
for `relation_refs` are explicitly out of scope for this task.

- [ ] **Step 1: Add reference, pack-rule, and advisory tests**

Add these tests:

```rust
#[test]
fn video_with_unknown_source_ref_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["source_ref_id"] =
        serde_json::json!("source_ref_missing");

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

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

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

    assert_has_error(&findings, "RV-RESULT-002", "$.evidence[0].claim_id");
}

#[test]
fn synthesis_top_level_unknown_claim_ref_returns_error() {
    let mut canonical = valid_canonical_result_with_synthesis();
    canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["claim_refs"] =
        serde_json::json!(["claim_missing"]);

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

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
    canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["cross_video_themes"][0]
        ["claim_refs"] = serde_json::json!(["claim_999"]);

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

    assert_has_error(
        &findings,
        "RV-RESULT-002",
        "$.outputs.pack_data.youtube_summary.synthesis.cross_video_themes[0].claim_refs[0]",
    );
}

#[test]
fn nested_synthesis_unknown_video_ref_returns_error() {
    let mut canonical = valid_canonical_result_with_synthesis();
    canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["cross_video_themes"][0]
        ["video_refs"] = serde_json::json!(["video_missing"]);

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

    assert_has_error(
        &findings,
        "RV-RESULT-002",
        "$.outputs.pack_data.youtube_summary.synthesis.cross_video_themes[0].video_refs[0]",
    );
}

#[test]
fn complete_standard_result_with_empty_videos_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"] = serde_json::json!([]);

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

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
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"] =
        serde_json::json!([canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0].clone()]);

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

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

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

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

    let findings = validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "standard"),
    );

    assert!(!findings.iter().any(|finding| finding.code == "RV-RESULT-005"));
}
```

- [ ] **Step 2: Run the focused tests and confirm expected failures**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation
```

Expected: the newly added reference, pack-rule, and advisory tests fail.

- [ ] **Step 3: Implement reference, pack-rule, and advisory checks**

Extend `validate_youtube_summary_canonical_result` after identity validation:

```rust
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
let claim_ids = collect_string_ids(canonical.get("claims").and_then(Value::as_array), "claim_id");
let evidence_ids = collect_string_ids(
    canonical.get("evidence").and_then(Value::as_array),
    "evidence_id",
);

validate_result_refs(canonical, &source_ids, &video_ids, &claim_ids, &evidence_ids, &mut findings);
validate_youtube_pack_rules(canonical, context, &mut findings);
add_advisory_quality_flag_findings(canonical, &mut findings);
```

Add these helper functions:

```rust
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
            let base = format!("$.outputs.pack_data.youtube_summary.synthesis.{array_key}[{index}]");
            validate_ref_array(item.get("source_refs"), source_ids, &format!("{base}.source_refs"), findings);
            validate_ref_array(item.get("claim_refs"), claim_ids, &format!("{base}.claim_refs"), findings);
            validate_ref_array(item.get("evidence_refs"), evidence_ids, &format!("{base}.evidence_refs"), findings);
            validate_ref_array(item.get("video_refs"), video_ids, &format!("{base}.video_refs"), findings);
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
    for (index, item) in value.and_then(Value::as_array).into_iter().flatten().enumerate() {
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
```

- [ ] **Step 4: Run validator tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation
```

Expected: PASS.

- [ ] **Step 5: Commit reference and advisory validation**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\result_validation.rs
git commit -m "feat: validate youtube summary result refs"
```

## Task 4: Refactor Final Persistence To Use A Real Transaction

**Files:**
- Modify: `src-tauri/src/prompt_packs/projections.rs`

- [ ] **Step 1: Add a rollback regression test**

Add this test to `projections.rs` tests:

```rust
#[tokio::test]
async fn low_level_result_persistence_rolls_back_when_projection_insert_fails() {
    let pool = test_pool_with_canonical_result_ready().await;
    // This deliberately bypasses result validation to test the lower-level
    // projection persistence transaction boundary.
    let mut canonical = test_canonical_result();
    canonical["source_refs"] = serde_json::json!([
        { "source_ref_id": "source_ref_1", "source_snapshot_id": 501, "title": "Video" },
        { "source_ref_id": "source_ref_1", "source_snapshot_id": 502, "title": "Duplicate" }
    ]);

    let error = persist_final_result_transaction(&pool, 42, canonical, "complete")
        .await
        .expect_err("projection unique constraint should fail");

    assert!(
        error.message.contains("Database error"),
        "unexpected error: {error:?}"
    );
    let result_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_results WHERE run_id = 42")
            .fetch_one(&pool)
            .await
            .expect("result count");
    let source_projection_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_source_refs WHERE run_id = 42",
    )
    .fetch_one(&pool)
    .await
    .expect("source projection count");

    assert_eq!(result_rows, 0);
    assert_eq!(source_projection_rows, 0);
}
```

- [ ] **Step 2: Run the rollback test and confirm it fails**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib low_level_result_persistence_rolls_back_when_projection_insert_fails
```

Expected: FAIL because the current helper inserts `prompt_pack_results` before projection insertion fails.

- [ ] **Step 3: Extract transaction-aware helpers**

Update imports:

```rust
use sqlx::{Sqlite, SqlitePool, Transaction};
```

Replace `persist_final_result_transaction` with a thin wrapper:

```rust
pub(crate) async fn persist_final_result_transaction(
    pool: &SqlitePool,
    run_id: i64,
    canonical_result: serde_json::Value,
    terminal_status: &str,
) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    persist_final_result_in_transaction(&mut tx, run_id, &canonical_result, terminal_status).await?;
    tx.commit().await.map_err(AppError::database)?;
    Ok(())
}
```

Add the transaction-aware helper directly below it:

```rust
pub(crate) async fn persist_final_result_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: i64,
    canonical_result: &serde_json::Value,
    terminal_status: &str,
) -> AppResult<()> {
    let canonical_json = canonical_result.to_string();
    let now = crate::time::now_rfc3339_utc();
    let result_row_id: i64 = sqlx::query_scalar(
        "INSERT INTO prompt_pack_results (
            run_id, result_id, result_status, schema_version, canonical_hash,
            canonical_json_zstd, projection_updated_at, created_at, updated_at
         )
         VALUES (?, ?, ?, ?, ?, ?, NULL, ?, ?)
         ON CONFLICT(run_id) DO UPDATE SET
            result_status = excluded.result_status,
            canonical_hash = excluded.canonical_hash,
            canonical_json_zstd = excluded.canonical_json_zstd,
            updated_at = excluded.updated_at
         RETURNING id",
    )
    .bind(run_id)
    .bind(canonical_result["result_id"].as_str().unwrap_or("result"))
    .bind(terminal_status)
    .bind(canonical_result["schema_version"].as_str().unwrap_or("1.0"))
    .bind(format!("sha384-{}", sha384_hex(canonical_json.as_bytes())))
    .bind(compress_text(&canonical_json).map_err(AppError::internal)?)
    .bind(&now)
    .bind(&now)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    rebuild_projection_rows_in_transaction(tx, result_row_id, run_id, canonical_result).await?;
    sqlx::query(
        "UPDATE prompt_pack_results SET projection_updated_at = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(&now)
    .bind(result_row_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = ?, result_status = ?, completed_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(terminal_status)
    .bind(terminal_status)
    .bind(&now)
    .bind(&now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    sqlx::query(
        "INSERT INTO prompt_pack_audit_events (run_id, event_kind, message, created_at)
         VALUES (?, 'terminal_result_persisted', 'Prompt Pack result persisted', ?)",
    )
    .bind(run_id)
    .bind(&now)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

Move and rename the current `rebuild_projection_rows` body into
`rebuild_projection_rows_in_transaction` and change only the executor plumbing
from `pool` to `&mut **tx`. Do not manually retype the projection logic from
this plan if the source file already differs; preserving the current projection
body avoids losing small future changes.

```rust
async fn rebuild_projection_rows_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    result_row_id: i64,
    run_id: i64,
    canonical: &serde_json::Value,
) -> AppResult<()> {
    for table in [
        "prompt_pack_result_source_refs",
        "prompt_pack_result_claims",
        "prompt_pack_result_evidence",
        "prompt_pack_result_ref_edges",
        "prompt_pack_result_unknowns",
        "prompt_pack_result_verification_tasks",
        "prompt_pack_result_warnings",
        "prompt_pack_result_limitations",
        "prompt_pack_result_quality_flags",
        "prompt_pack_result_audit_refs",
        "prompt_pack_youtube_videos",
        "prompt_pack_youtube_segments",
        "prompt_pack_youtube_key_points",
        "prompt_pack_youtube_quotes",
        "prompt_pack_youtube_action_items",
        "prompt_pack_youtube_open_questions",
        "prompt_pack_youtube_synthesis_items",
    ] {
        sqlx::query(&format!("DELETE FROM {table} WHERE result_row_id = ?"))
            .bind(result_row_id)
            .execute(&mut **tx)
            .await
            .map_err(AppError::database)?;
    }

    for source_ref in canonical
        .get("source_refs")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        sqlx::query(
            "INSERT INTO prompt_pack_result_source_refs (
                result_row_id, run_id, source_ref_id, source_snapshot_id, title
             )
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(result_row_id)
        .bind(run_id)
        .bind(source_ref["source_ref_id"].as_str().unwrap_or(""))
        .bind(source_ref["source_snapshot_id"].as_i64().unwrap_or(0))
        .bind(source_ref["title"].as_str())
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    for claim in canonical
        .get("claims")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        sqlx::query(
            "INSERT INTO prompt_pack_result_claims (
                result_row_id, run_id, claim_id, source_ref_id, text
             )
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(result_row_id)
        .bind(run_id)
        .bind(claim["claim_id"].as_str().unwrap_or(""))
        .bind(claim["source_ref_id"].as_str())
        .bind(claim["text"].as_str().unwrap_or(""))
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    for evidence in canonical
        .get("evidence")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        sqlx::query(
            "INSERT INTO prompt_pack_result_evidence (
                result_row_id, run_id, evidence_id, claim_id, material_ref_id, text
             )
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(result_row_id)
        .bind(run_id)
        .bind(evidence["evidence_id"].as_str().unwrap_or(""))
        .bind(evidence["claim_id"].as_str())
        .bind(evidence["material_ref_id"].as_str())
        .bind(evidence["text"].as_str().unwrap_or(""))
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    for video in canonical["outputs"]["pack_data"]["youtube_summary"]["videos"]
        .as_array()
        .into_iter()
        .flatten()
    {
        sqlx::query(
            "INSERT INTO prompt_pack_youtube_videos (
                result_row_id, run_id, video_id, source_ref_id, title, summary_text
             )
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(result_row_id)
        .bind(run_id)
        .bind(video["video_id"].as_str().unwrap_or(""))
        .bind(video["source_ref_id"].as_str().unwrap_or(""))
        .bind(video["title"].as_str())
        .bind(video["summary_text"].as_str())
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    if let Some(synthesis) =
        canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"].as_object()
    {
        for item in synthesis
            .get("cross_video_themes")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            insert_youtube_synthesis_projection_item(
                tx,
                result_row_id,
                run_id,
                item["theme_id"].as_str().unwrap_or(""),
                item["theme_text"].as_str().unwrap_or(""),
            )
            .await?;
        }

        for item in synthesis
            .get("common_claims")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            insert_youtube_synthesis_projection_item(
                tx,
                result_row_id,
                run_id,
                item["common_claim_id"].as_str().unwrap_or(""),
                item["summary_text"].as_str().unwrap_or(""),
            )
            .await?;
        }

        for item in synthesis
            .get("contradictions_across_videos")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            insert_youtube_synthesis_projection_item(
                tx,
                result_row_id,
                run_id,
                item["contradiction_id"].as_str().unwrap_or(""),
                item["description"].as_str().unwrap_or(""),
            )
            .await?;
        }
    }

    Ok(())
}
```

Then change `insert_youtube_synthesis_projection_item` to take `tx`:

```rust
async fn insert_youtube_synthesis_projection_item(
    tx: &mut Transaction<'_, Sqlite>,
    result_row_id: i64,
    run_id: i64,
    synthesis_id: &str,
    text: &str,
) -> AppResult<()> {
    if synthesis_id.is_empty() || text.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO prompt_pack_youtube_synthesis_items (
            result_row_id, run_id, synthesis_id, text
         )
         VALUES (?, ?, ?, ?)",
    )
    .bind(result_row_id)
    .bind(run_id)
    .bind(synthesis_id)
    .bind(text)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

Update `repair_prompt_pack_result_projections` to use a transaction for rebuild and update:

```rust
let mut tx = pool.begin().await.map_err(AppError::database)?;
rebuild_projection_rows_in_transaction(&mut tx, result_row_id, run_id, &canonical).await?;
sqlx::query("UPDATE prompt_pack_results SET projection_updated_at = ? WHERE id = ?")
    .bind(crate::time::now_rfc3339_utc())
    .bind(result_row_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
tx.commit().await.map_err(AppError::database)?;
```

- [ ] **Step 4: Run projection tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib projections
```

Expected: PASS.

- [ ] **Step 5: Commit transaction refactor**

```powershell
git add src-tauri\src\prompt_packs\projections.rs
git commit -m "refactor: make prompt pack result persistence atomic"
```

## Task 5: Add Validation-Aware Persistence Wrapper

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`
- Modify: `src-tauri/src/prompt_packs/projections.rs` only if helper visibility from Task 4 needs adjustment

- [ ] **Step 1: Add persistence tests**

Add these async tests to `result_validation.rs`:

```rust
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
    assert_eq!(finding, (None, "warning".to_string(), "RV-RESULT-005".to_string()));
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
    canonical["claims"][0]["claim_id"] = serde_json::json!("");

    let error = validate_and_persist_final_result_transaction(&pool, 42, canonical, "complete")
        .await
        .expect_err("validation failure");

    assert!(error.message.contains("canonical result validation failed"));
    assert_eq!(count(&pool, "prompt_pack_results").await, 0);
    assert_eq!(count(&pool, "prompt_pack_result_validation_findings").await, 1);
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
    canonical["claims"][0]["claim_id"] = serde_json::json!("");

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
async fn validation_wrapper_rolls_back_result_findings_when_persistence_fails_after_validation() {
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

    assert!(error.message.contains("Database error"));
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
```

Add test helpers:

```rust
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
```

- [ ] **Step 2: Run persistence tests and confirm they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation
```

Expected: FAIL because `validate_and_persist_final_result_transaction` is not implemented.

- [ ] **Step 3: Implement validation-aware persistence**

Add imports:

```rust
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::error::{AppError, AppResult};
use crate::prompt_packs::projections::persist_final_result_in_transaction;
```

Add the wrapper and helpers below the pure validator helpers:

```rust
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
    let context = YoutubeSummaryResultValidationContext::new(
        run_id,
        terminal_status,
        &evidence_mode,
    );
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
    persist_final_result_in_transaction(&mut tx, run_id, &canonical_result, terminal_status).await?;
    tx.commit().await.map_err(AppError::database)?;
    Ok(())
}

async fn load_run_evidence_mode(pool: &SqlitePool, run_id: i64) -> AppResult<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT evidence_mode FROM prompt_pack_runs WHERE id = ?",
    )
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
```

- [ ] **Step 4: Run validation persistence tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation
```

Expected: PASS.

- [ ] **Step 5: Commit validation-aware persistence**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\result_validation.rs src-tauri\src\prompt_packs\projections.rs
git commit -m "feat: gate youtube summary result persistence"
```

## Task 6: Wire Execution To The Validation Gate

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/execution.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/execution_tests.rs`

- [ ] **Step 1: Replace the execution import and call**

In `execution.rs`, replace:

```rust
use crate::prompt_packs::{
    projections::persist_final_result_transaction,
    result_builder::build_youtube_summary_canonical_result,
};
```

with:

```rust
use crate::prompt_packs::result_builder::build_youtube_summary_canonical_result;
use crate::prompt_packs::youtube_summary::result_validation::validate_and_persist_final_result_transaction;
```

Then replace:

```rust
persist_final_result_transaction(pool, run_id, canonical, terminal_status).await?;
```

with:

```rust
validate_and_persist_final_result_transaction(pool, run_id, canonical, terminal_status).await?;
```

- [ ] **Step 2: Add execution regression test for the valid path**

Prefer extending the existing `execute_queued_run_with_stage_executor_finishes_complete`
test with the extra projection/finding assertions below, because it already
uses the production `execute_youtube_summary_run_with_stage_executor` path.
Only add a new test if changing the existing one makes it harder to read. Do
not use `execute_youtube_summary_run_with_fake_completions` here: that helper
persists via `persist_minimal_execution_result` and bypasses the production
final result path.

```rust
#[tokio::test]
async fn youtube_summary_valid_run_persists_result_after_result_validation() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;

    let outcome = execute_youtube_summary_run_with_stage_executor(&pool, 1, |request| async move {
        match request {
            YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => Ok(
                fake_completion_with_valid_transcript_analysis_json_for_source(
                    &request.source_ref_id,
                ),
            ),
            YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                panic!("single-video run should not request synthesis")
            }
            YoutubeSummaryStageExecutionRequest::JsonRepair(_) => {
                panic!("valid single-video run should not request repair")
            }
        }
    })
    .await
    .expect("execute run");

    let results: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_results WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("result count");
    let videos: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_youtube_videos WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("video projections");
    let result_findings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_validation_findings
         WHERE run_id = 1 AND stage_run_id IS NULL AND severity = 'error'",
    )
    .fetch_one(&pool)
    .await
    .expect("result finding count");

    assert_eq!(outcome.run_status, "complete");
    assert_eq!(results, 1);
    assert_eq!(videos, 1);
    assert_eq!(result_findings, 0);
}
```

- [ ] **Step 3: Add an invalid-canonical execution regression test through a test-only helper**

Extract the current body of `execute_youtube_summary_run_with_stage_executor`
into a private internal helper that accepts a final-result mutator. Keep the
production function name and behavior unchanged. The internal helper must remain
private (`async fn`, not `pub(crate) async fn`) so production code does not gain
another validation-bypass entry point:

```rust
pub(crate) async fn execute_youtube_summary_run_with_stage_executor<F, Fut>(
    pool: &SqlitePool,
    run_id: i64,
    execute_stage: F,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
{
    execute_youtube_summary_run_with_stage_executor_internal(
        pool,
        run_id,
        execute_stage,
        |_| {},
    )
    .await
}
```

The private internal helper signature should be:

```rust
async fn execute_youtube_summary_run_with_stage_executor_internal<F, Fut, M>(
    pool: &SqlitePool,
    run_id: i64,
    mut execute_stage: F,
    mutate_final_result: M,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
    M: FnOnce(&mut serde_json::Value),
```

Move the current production body into that internal helper. In the moved body,
change only the final result build block to:

```rust
let mut canonical = build_youtube_summary_canonical_result(pool, run_id).await?;
mutate_final_result(&mut canonical);
validate_and_persist_final_result_transaction(pool, run_id, canonical, terminal_status).await?;
```

Add this `#[cfg(test)]` helper in `execution.rs` so the mutator is only exposed
to tests:

```rust
#[cfg(test)]
pub(crate) async fn execute_youtube_summary_run_with_stage_executor_and_result_mutator<F, Fut, M>(
    pool: &SqlitePool,
    run_id: i64,
    execute_stage: F,
    mutate_final_result: M,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
    M: FnOnce(&mut serde_json::Value),
{
    execute_youtube_summary_run_with_stage_executor_internal(
        pool,
        run_id,
        execute_stage,
        mutate_final_result,
    )
    .await
}
```

Add this test to `execution_tests.rs`:

```rust
#[tokio::test]
async fn youtube_summary_invalid_final_result_records_result_level_findings() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let outcome = super::execution::execute_youtube_summary_run_with_stage_executor_and_result_mutator(
        &pool,
        1,
        |request| async move {
            match request {
                YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => Ok(
                    fake_completion_with_valid_transcript_analysis_json_for_source(
                        &request.source_ref_id,
                    ),
                ),
                YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                    panic!("single-video run should not request synthesis")
                }
                YoutubeSummaryStageExecutionRequest::JsonRepair(_) => {
                    panic!("valid single-video run should not request repair")
                }
            }
        },
        |canonical| {
            canonical["claims"][0]["claim_id"] = serde_json::json!("");
        },
    )
    .await;

    assert!(outcome.is_err());
    let result_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_results WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("result rows");
    let result_findings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_validation_findings
         WHERE run_id = 1 AND stage_run_id IS NULL AND severity = 'error'",
    )
    .fetch_one(&pool)
    .await
    .expect("result findings");
    let run_status: String =
        sqlx::query_scalar("SELECT run_status FROM prompt_pack_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("run status");

    assert_eq!(result_rows, 0);
    assert!(result_findings > 0);
    assert_eq!(run_status, "failed");
}
```

- [ ] **Step 4: Run execution tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_valid_run_persists_result_after_result_validation
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_invalid_final_result_records_result_level_findings
```

Expected: PASS.

- [ ] **Step 5: Commit execution wiring**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\execution.rs src-tauri\src\prompt_packs\youtube_summary\execution_tests.rs
git commit -m "feat: validate youtube summary final result during execution"
```

## Task 7: Focused And Broad Verification

**Files:**
- No code changes expected unless verification exposes a defect.

- [ ] **Step 1: Run focused validator and projection suites**

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation
cargo test --manifest-path src-tauri\Cargo.toml --lib projections
```

Expected: both commands PASS.

- [ ] **Step 2: Run YouTube Summary execution and Prompt Pack suites**

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs
```

Expected: both commands PASS.

- [ ] **Step 3: Run compile check**

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
```

Expected: PASS.

- [ ] **Step 4: Inspect changed files**

```powershell
git diff --check
git status --short
```

Expected: `git diff --check` has no output. `git status --short` shows only the intended source changes and this plan file.

- [ ] **Step 5: Commit final cleanup if verification required fixes**

Only run this if Step 1-4 required additional edits:

```powershell
git add src-tauri\src\prompt_packs\projections.rs src-tauri\src\prompt_packs\youtube_summary\execution.rs src-tauri\src\prompt_packs\youtube_summary\execution_tests.rs src-tauri\src\prompt_packs\youtube_summary\mod.rs src-tauri\src\prompt_packs\youtube_summary\result_validation.rs
git commit -m "test: cover youtube summary result validation"
```

## Self-Review Checklist

- Spec coverage:
  - Pure result validation: Tasks 1-3.
  - Result-level findings with `stage_run_id = NULL`: Task 5.
  - Error hard gate before result/projection persistence: Task 5.
  - Warning/info advisory persistence without blocking: Task 5.
  - Atomic persistence and rollback: Tasks 4-5.
  - Existing execution integration: Task 6.
  - No migration, UI command, quarantine, or repair changes: File structure and Task 6 boundaries.
- Red-flag scan:
  - This plan contains concrete file paths, commands, codes, and test names.
- Type consistency:
  - `PromptPackResultValidationFinding`, `YoutubeSummaryResultValidationContext`, `validate_youtube_summary_canonical_result`, and `validate_and_persist_final_result_transaction` are defined before later tasks use them.
