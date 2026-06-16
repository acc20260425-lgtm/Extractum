# YouTube Summary Derived Traversal Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add result-level derived traversal validation for YouTube Summary canonical results.

**Architecture:** Keep the slice inside `validate_youtube_summary_canonical_result` and reuse existing result-level findings. Add pure helper functions for ordered union derivation, duplicate detection, and guarded `videos[]` traversal validation. Do not change persistence, execution, projections, schema, or UI.

**Tech Stack:** Rust, `serde_json::Value`, `std::collections::{HashMap, HashSet}`, existing Prompt Pack result validation tests.

---

## Source Spec

- `docs/superpowers/specs/2026-06-16-youtube-summary-derived-traversal-validation-design.md`
- Existing validator: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`

## File Structure

- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`
  - Add `HashMap` import.
  - Add derived traversal helper calls inside `validate_youtube_summary_canonical_result`.
  - Add ordered unique reference collection helpers.
  - Add synthesis derived union validation helpers.
  - Add `videos[]` traversal validation helper.
  - Add pure validator tests using existing canonical fixture style.

No new Rust modules are needed. The affected validator file already owns shape, identity, known-ref, pack-rule, and result-persistence tests for this result-validation layer.

---

### Task 1: Synthesis Derived Traversal Tests

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`

- [ ] **Step 1: Write failing tests for synthesis traversal unions**

Add these tests in the existing `#[cfg(test)] mod tests` block after `nested_synthesis_unknown_video_ref_returns_error`:

```rust
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
        ["source_refs"] = serde_json::json!([]);
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
        ["source_refs"] = serde_json::json!([]);
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
```

- [ ] **Step 2: Run tests and verify the new tests fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation -- --nocapture
```

Expected: the new missing/extra/duplicate `VR-YS-015` tests fail because no
derived traversal validation exists yet. Existing tests should still compile.
`synthesis_order_difference_in_top_level_union_is_allowed` is a regression guard
and may pass both before and after implementation.

- [ ] **Step 3: Commit failing tests**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\result_validation.rs
git commit -m "test: cover youtube summary synthesis traversal validation"
```

---

### Task 2: Synthesis Derived Traversal Implementation

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`

- [ ] **Step 1: Add `HashMap` import**

Replace the existing import:

```rust
use std::collections::HashSet;
```

with:

```rust
use std::collections::{HashMap, HashSet};
```

- [ ] **Step 2: Wire synthesis traversal validation into the top-level validator**

In `validate_youtube_summary_canonical_result`, immediately after `validate_result_refs(...)` and before `validate_youtube_pack_rules(...)`, add:

```rust
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
```

The surrounding section should read:

```rust
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
```

- [ ] **Step 3: Add helper functions for ordered unions**

Steps 3, 4, and 5 form one contiguous helper block. Insert Step 3 immediately
after `validate_ref_array(...)` and before `validate_youtube_pack_rules(...)`.
Then append Step 4 immediately after Step 3, and Step 5 immediately after Step
4.

Add these helpers:

```rust
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
```

- [ ] **Step 4: Add derivation helpers**

Append these functions immediately after the ordered-union helpers from Step 3:

```rust
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
```

- [ ] **Step 5: Add comparison and finding helpers**

Append these functions immediately after the derivation helpers from Step 4:

```rust
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
```

- [ ] **Step 6: Add synthesis-null regression test**

Add this test immediately after
`synthesis_unknown_video_ref_does_not_cascade_to_source_union_error`:

```rust
#[test]
fn synthesis_null_skips_derived_traversal_validation() {
    let canonical = valid_canonical_result();

    let findings =
        validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

    assert!(!findings.iter().any(|finding| finding.code == "VR-YS-015"));
}
```

This is a regression guard after the traversal validator is wired. It is not part
of Task 1 because it would pass before implementation.

- [ ] **Step 7: Run focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation -- --nocapture
```

Expected: all synthesis derived traversal tests from Task 1 pass. Video traversal tests are not present yet.

- [ ] **Step 8: Commit implementation**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\result_validation.rs
git commit -m "feat: validate youtube summary synthesis traversal refs"
```

---

### Task 3: Video Traversal Tests

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`

- [ ] **Step 1: Write failing tests for guarded `videos[]` traversal fields**

Add these tests after the synthesis traversal tests from Task 1:

```rust
#[test]
fn video_source_refs_missing_self_ref_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["source_refs"] =
        serde_json::json!([]);

    let findings =
        validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

    assert_has_error(
        &findings,
        "VR-YS-004",
        "$.outputs.pack_data.youtube_summary.videos[0].source_refs",
    );
}

#[test]
fn video_source_refs_malformed_shape_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["source_refs"] =
        serde_json::json!("not_an_array");

    let findings =
        validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

    assert_has_error(
        &findings,
        "VR-YS-020",
        "$.outputs.pack_data.youtube_summary.videos[0].source_refs",
    );
}

#[test]
fn video_source_refs_with_non_string_item_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["source_refs"] =
        serde_json::json!([42]);

    let findings =
        validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

    assert_has_error(
        &findings,
        "VR-YS-020",
        "$.outputs.pack_data.youtube_summary.videos[0].source_refs",
    );
}

#[test]
fn video_source_refs_unknown_ref_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["source_refs"] =
        serde_json::json!(["source_ref_missing"]);

    let findings =
        validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

    assert_has_error(
        &findings,
        "RV-RESULT-002",
        "$.outputs.pack_data.youtube_summary.videos[0].source_refs[0]",
    );
}

#[test]
fn missing_video_source_refs_is_allowed() {
    let canonical = valid_canonical_result();

    let findings =
        validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

    assert!(
        findings.iter().all(|finding| {
            finding.object_path.as_deref()
                != Some("$.outputs.pack_data.youtube_summary.videos[0].source_refs")
        }),
        "{findings:#?}"
    );
}

#[test]
fn video_claim_refs_unknown_ref_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["claim_refs"] =
        serde_json::json!(["claim_missing"]);

    let findings =
        validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

    assert_has_error(
        &findings,
        "RV-RESULT-002",
        "$.outputs.pack_data.youtube_summary.videos[0].claim_refs[0]",
    );
}

#[test]
fn video_evidence_refs_unknown_ref_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["evidence_refs"] =
        serde_json::json!(["evidence_missing"]);

    let findings =
        validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

    assert_has_error(
        &findings,
        "RV-RESULT-002",
        "$.outputs.pack_data.youtube_summary.videos[0].evidence_refs[0]",
    );
}

#[test]
fn video_claim_refs_malformed_shape_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["claim_refs"] =
        serde_json::json!("not_an_array");

    let findings =
        validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

    assert_has_error(
        &findings,
        "VR-YS-020",
        "$.outputs.pack_data.youtube_summary.videos[0].claim_refs",
    );
}

#[test]
fn video_evidence_refs_with_non_string_item_returns_error() {
    let mut canonical = valid_canonical_result();
    canonical["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["evidence_refs"] =
        serde_json::json!([42]);

    let findings =
        validate_youtube_summary_canonical_result(&canonical, &context("complete", "standard"));

    assert_has_error(
        &findings,
        "VR-YS-020",
        "$.outputs.pack_data.youtube_summary.videos[0].evidence_refs",
    );
}
```

- [ ] **Step 2: Run tests and verify the new video tests fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation -- --nocapture
```

Expected: Task 2 tests still pass. The new video traversal tests fail because
`validate_video_traversal_refs` is not wired yet. In particular, no existing
validator should report `$.outputs.pack_data.youtube_summary.videos[0].source_refs`,
`$.outputs.pack_data.youtube_summary.videos[0].claim_refs`, or
`$.outputs.pack_data.youtube_summary.videos[0].evidence_refs` before Task 4.

- [ ] **Step 3: Commit failing video tests**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\result_validation.rs
git commit -m "test: cover youtube summary video traversal validation"
```

---

### Task 4: Video Traversal Implementation

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`

- [ ] **Step 1: Wire video traversal validation into the top-level validator**

In `validate_youtube_summary_canonical_result`, after `validate_synthesis_derived_traversal_refs(...)` and before `validate_youtube_pack_rules(...)`, add:

```rust
    validate_video_traversal_refs(
        videos,
        &source_ids,
        &claim_ids,
        &evidence_ids,
        &mut findings,
    );
```

The full block should read:

```rust
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
    validate_video_traversal_refs(
        videos,
        &source_ids,
        &claim_ids,
        &evidence_ids,
        &mut findings,
    );
    validate_youtube_pack_rules(canonical, context, &mut findings);
```

- [ ] **Step 2: Add guarded traversal field helper functions**

Add these helpers after `compare_derived_traversal_refs(...)` and before `validate_youtube_pack_rules(...)`:

```rust
fn validate_video_traversal_refs(
    videos: Option<&Vec<Value>>,
    source_ids: &HashSet<String>,
    claim_ids: &HashSet<String>,
    evidence_ids: &HashSet<String>,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    for (index, video) in videos.into_iter().flatten().enumerate() {
        let base_path = format!("$.outputs.pack_data.youtube_summary.videos[{index}]");
        validate_optional_video_ref_array(
            video.get("source_refs"),
            source_ids,
            &format!("{base_path}.source_refs"),
            findings,
        );
        validate_optional_video_ref_array(
            video.get("claim_refs"),
            claim_ids,
            &format!("{base_path}.claim_refs"),
            findings,
        );
        validate_optional_video_ref_array(
            video.get("evidence_refs"),
            evidence_ids,
            &format!("{base_path}.evidence_refs"),
            findings,
        );

        if let Some(source_refs) = video.get("source_refs").and_then(Value::as_array) {
            let source_ref_id = video
                .get("source_ref_id")
                .and_then(Value::as_str)
                .map(str::trim);
            if let Some(source_ref_id) = source_ref_id {
                let includes_self = source_refs
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|value| value == source_ref_id);
                if !source_ref_id.is_empty() && !includes_self {
                    findings.push(finding(
                        "error",
                        "VR-YS-004",
                        format!(
                            "video.source_refs must include self source_ref_id `{source_ref_id}`"
                        ),
                        Some(format!("{base_path}.source_refs")),
                    ));
                }
            }
        }
    }
}

fn validate_optional_video_ref_array(
    value: Option<&Value>,
    allowed: &HashSet<String>,
    path: &str,
    findings: &mut Vec<PromptPackResultValidationFinding>,
) {
    // This intentionally differs from validate_ref_array(...), which skips
    // non-string refs for raw synthesis-output compatibility. Video traversal
    // arrays are canonical result fields introduced by this slice, so malformed
    // shapes are result-level VR-YS-020 errors.
    let Some(value) = value else {
        return;
    };
    let Some(items) = value.as_array() else {
        findings.push(finding(
            "error",
            "VR-YS-020",
            format!("{path} must be an array of strings"),
            Some(path.to_string()),
        ));
        return;
    };

    for (index, item) in items.iter().enumerate() {
        let Some(ref_id) = item.as_str() else {
            findings.push(finding(
                "error",
                "VR-YS-020",
                format!("{path} must be an array of strings"),
                Some(path.to_string()),
            ));
            continue;
        };
        if !allowed.contains(ref_id) {
            findings.push(finding(
                "error",
                "RV-RESULT-002",
                format!("unknown ref `{ref_id}`"),
                Some(format!("{path}[{index}]")),
            ));
        }
    }
}
```

- [ ] **Step 3: Run focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation -- --nocapture
```

Expected: all result-validation tests pass.

- [ ] **Step 4: Commit implementation**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\result_validation.rs
git commit -m "feat: validate youtube summary video traversal refs"
```

---

### Task 5: Full Verification

**Files:**
- No code changes expected.

- [ ] **Step 1: Run formatter check**

```powershell
cargo fmt --manifest-path src-tauri\Cargo.toml --check
```

Expected: command exits successfully.

- [ ] **Step 2: Run focused result-validation tests**

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib result_validation
```

Expected: command exits successfully.

- [ ] **Step 3: Run YouTube Summary library tests**

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
```

Expected: command exits successfully.

- [ ] **Step 4: Run prompt pack library tests**

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs
```

Expected: command exits successfully.

- [ ] **Step 5: Run cargo check**

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
```

Expected: command exits successfully.

- [ ] **Step 6: Run diff check**

```powershell
git diff --check
```

Expected: no whitespace errors. On Windows, the existing LF/CRLF warning is acceptable if there are no `trailing whitespace` or `space before tab` errors.

- [ ] **Step 7: Inspect git status**

```powershell
git status --short
```

Expected: only intended files are modified or committed. Do not stage `tmp/`.

- [ ] **Step 8: Commit final verification metadata if any code changed after previous commits**

If `git status --short` shows no uncommitted intended code changes after the
previous task commits, skip this step. If formatting changed files, commit them:

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\result_validation.rs
git commit -m "chore: format youtube summary traversal validation"
```

---

## Self-Review

**Spec coverage**

- `VR-YS-015` derived unions for `synthesis.claim_refs`, `synthesis.evidence_refs`, and `synthesis.source_refs`: covered by Tasks 1 and 2.
- Deterministic first-seen expected order with order-insensitive comparison: covered by Task 2 helpers and `synthesis_order_difference_in_top_level_union_is_allowed`.
- Unknown nested `video_refs[]` skip in source derivation without cascade: covered by `synthesis_unknown_video_ref_does_not_cascade_to_source_union_error`.
- `synthesis = null` skip: covered by `synthesis_null_skips_derived_traversal_validation`, a regression guard added in Task 2.
- Existing `relation_refs` shape check preserved: no task modifies `validate_synthesis_shape`.
- `video.source_refs`, `video.claim_refs`, and `video.evidence_refs` guarded validation: covered by Tasks 3 and 4.
- `VR-YS-004` only for missing self source ref: covered by `video_source_refs_missing_self_ref_returns_error`.
- `VR-YS-020` for malformed video traversal field shape: covered by `video_source_refs_malformed_shape_returns_error`, `video_source_refs_with_non_string_item_returns_error`, `video_claim_refs_malformed_shape_returns_error`, and `video_evidence_refs_with_non_string_item_returns_error`.
- Existing unknown-ref code for video traversal unknown refs: covered by `video_source_refs_unknown_ref_returns_error`, `video_claim_refs_unknown_ref_returns_error`, and `video_evidence_refs_unknown_ref_returns_error`.
- No persistence, execution, schema, projection, or UI changes: all tasks touch only `result_validation.rs`.

**Placeholder scan**

- No placeholder markers.
- No vague implementation steps without code snippets.
- No references to files outside the listed file structure.

**Type consistency**

- `video_source_by_id` is consistently `HashMap<String, String>` mapping `video_id -> source_ref_id`.
- `source_ids`, `claim_ids`, and `evidence_ids` are consistently `HashSet<String>`.
- Helper signatures match the approved spec and the call sites in `validate_youtube_summary_canonical_result`.
