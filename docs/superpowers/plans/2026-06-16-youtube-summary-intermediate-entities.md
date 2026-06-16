# YouTube Summary Intermediate Entities Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build backend-owned source-scoped intermediate entity graphs for YouTube Summary, expose complete allowed refs to synthesis input, validate synthesis refs, and let the canonical result builder prefer graph claims/evidence with observable legacy fallback.

**Architecture:** Add a focused `youtube_summary/entities.rs` module that builds and validates source-scoped `intermediate_entities` JSON from successful transcript-analysis parsed outputs. Source graph refs are source-qualified (`{source_ref_id}_claim_1`, `{source_ref_id}_evidence_1`, etc.) so merge can concatenate artifacts without ref remapping or duplicate allowed refs. `source_ref_id` is a backend-generated run-local safe token (`source_ref_N`) and may be embedded into derived refs without escaping. Persist one graph artifact per successful transcript stage attempt after `metrics` (`prompt_input #1`, `raw_output #2`, `parsed_output #3`, `metrics #4`, `intermediate_entities #5`), merge those artifacts into synthesis input by `prompt_pack_run_source_snapshots.id ASC`, and keep v1 synthesis output limited to source/claim/evidence refs. Result building uses graph claims/evidence only when the run has a complete graph set; otherwise it falls back to legacy parsed-output assembly with an explicit quality flag.

**Tech Stack:** Rust, sqlx/SQLite, serde_json, existing Prompt Pack stage artifacts, existing YouTube Summary test fixtures.

---

## File Structure

- Create `src-tauri/src/prompt_packs/youtube_summary/entities.rs`
  - Pure-ish graph builder from `TranscriptAnalysisStageInput` + parsed transcript output.
  - Graph artifact constants and artifact insertion helper.
  - DB loaders for latest graph artifacts by source and run-level merge helpers.
- Create `src-tauri/src/prompt_packs/youtube_summary/entities_tests.rs`
  - Focused tests for graph shape, malformed candidates, material refs, source-local refs, textless segments, and merge order.
- Modify `src-tauri/src/prompt_packs/youtube_summary/mod.rs`
  - Add `pub(crate) mod entities;` and test module declaration.
- Modify `src-tauri/src/prompt_packs/youtube_summary/outputs.rs`
  - Persist `intermediate_entities` after transcript-analysis parsed output validates.
- Modify `src-tauri/src/prompt_packs/youtube_summary/transcript_execution.rs`
  - Add attempt-aware transcript failure marking for the execution layer.
- Modify `src-tauri/src/prompt_packs/youtube_summary/execution.rs`
  - Keep terminal transcript failure ownership in the execution layer and avoid duplicate error artifacts.
- Modify `src-tauri/src/prompt_packs/json_repair.rs`
  - Persist `intermediate_entities` for repaired transcript-analysis success with matching `attempt_number = 2`.
- Modify `src-tauri/src/prompt_packs/youtube_summary/synthesis_input.rs`
  - Merge graph artifacts into `canonical_graph` and complete `allowed_refs`.
  - Keep legacy loose candidate arrays during transition.
  - Do not forward graph `warnings`.
- Modify `src-tauri/src/prompt_packs/youtube_summary/synthesis_input_tests.rs`
  - Assert graph merge shape, full allowed refs, no warnings in synthesis input, deterministic source order.
- Modify `src-tauri/src/prompt_packs/runtime.rs`
  - Update transcript-analysis prompt instructions to allow candidate indexes and forbid backend refs.
  - Update synthesis prompt instructions to allow checked claim/evidence/source refs and forbid direct segment/key point/quote refs in v1 output.
- Modify `src-tauri/src/prompt_packs/validation.rs`
  - Validate synthesis output refs against complete allowed refs.
  - Reject direct `segment_refs`, `key_point_refs`, and `quote_refs` inside the `synthesis_candidate` subtree.
  - Provide a reusable quarantine helper for graph build validation failures.
- Modify `src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs`
  - Assert `intermediate_entities` artifact is written for transcript and repaired transcript success.
- Modify `src-tauri/src/prompt_packs/youtube_summary/execution_tests.rs`
  - Assert graph-build validation failures are marked once by the execution layer.
- Modify `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`
  - Keep transcript fixture helpers aligned with the real output path so succeeded transcript fixtures include `intermediate_entities`.
- Modify `src-tauri/src/prompt_packs/result_builder.rs`
  - Prefer graph claims/evidence when every successful transcript stage has a graph artifact.
  - Fall back all-or-nothing with `intermediate_entities_legacy_fallback` quality flag when graph artifacts are partially missing.
- Modify or extend result builder tests in `src-tauri/src/prompt_packs/result_builder.rs`
  - Graph preferred path, legacy fallback path, mixed graph/legacy observable fallback.

---

## Task 1: Add Source-Scoped Entity Graph Builder

**Files:**
- Create: `src-tauri/src/prompt_packs/youtube_summary/entities.rs`
- Create: `src-tauri/src/prompt_packs/youtube_summary/entities_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/mod.rs`

- [x] **Step 1: Add module declarations**

In `src-tauri/src/prompt_packs/youtube_summary/mod.rs`, add the production module near the other domain modules:

```rust
pub(crate) mod entities;
```

Add the test module near the other `#[cfg(test)]` modules:

```rust
#[cfg(test)]
mod entities_tests;
```

- [x] **Step 2: Write failing graph builder tests**

Create `src-tauri/src/prompt_packs/youtube_summary/entities_tests.rs` with these tests:

```rust
use super::entities::{
    build_source_intermediate_entities, INTERMEDIATE_ENTITIES_ARTIFACT_KIND,
    YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND,
};
use crate::prompt_packs::stage_io::{
    TranscriptAnalysisStageInput, TranscriptSegmentRegistryEntry,
};

fn input() -> TranscriptAnalysisStageInput {
    TranscriptAnalysisStageInput {
        stage_io_version: "1.0".to_string(),
        schema_version: "1.0".to_string(),
        stage: "youtube_summary/transcript_analysis".to_string(),
        pack_id: "youtube_summary".to_string(),
        pack_version: "1.0.0".to_string(),
        run_id: 42,
        source_ref_id: "source_ref_1".to_string(),
        allowed_source_ref_ids: vec!["source_ref_1".to_string()],
        allowed_material_refs: vec!["material_ref_1".to_string(), "material_ref_2".to_string()],
        transcript_segment_registry: vec![TranscriptSegmentRegistryEntry {
            material_ref_id: "material_ref_1".to_string(),
            text: "Transcript text".to_string(),
        }],
        comment_selection_policy: serde_json::json!({}),
        control_preset: "standard".to_string(),
        evidence_mode: "standard".to_string(),
        output_language: "en".to_string(),
    }
}

fn parsed_output() -> serde_json::Value {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/transcript_analysis",
        "video_candidate": {
            "summary_text": "Video summary",
            "segment_candidates": [
                { "title": "Intro", "summary_text": "Opening", "material_refs": ["material_ref_1"] }
            ],
            "key_point_candidates": [
                { "text": "Main point", "segment_candidate_index": 0, "material_refs": ["material_ref_1"] }
            ],
            "quote_candidates": [
                { "text": "quoted text", "segment_candidate_index": 0, "material_refs": ["material_ref_1"] }
            ],
            "action_item_candidates": [],
            "open_question_candidates": []
        },
        "claim_candidates": [
            { "text": "Claim text", "material_refs": ["material_ref_1"] }
        ],
        "evidence_fragment_candidates": [
            { "text": "Evidence text", "material_refs": ["material_ref_1"], "quote_candidate_index": 0 }
        ],
        "warning_candidates": []
    })
}

#[test]
fn graph_constants_match_contract() {
    assert_eq!(INTERMEDIATE_ENTITIES_ARTIFACT_KIND, "intermediate_entities");
    assert_eq!(
        YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND,
        "youtube_summary_intermediate_entities"
    );
}

#[test]
fn build_source_graph_assigns_backend_refs_and_allowed_refs() {
    let graph = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed_output(), 1)
        .expect("graph");

    assert_eq!(graph["graph_kind"], YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND);
    assert_eq!(graph["sources"][0]["source_ref_id"], "source_ref_1");
    assert_eq!(graph["segments"][0]["segment_ref"], "source_ref_1_segment_1");
    assert_eq!(graph["key_points"][0]["key_point_ref"], "source_ref_1_key_point_1");
    assert_eq!(graph["quotes"][0]["quote_ref"], "source_ref_1_quote_1");
    assert_eq!(graph["claims"][0]["claim_id"], "source_ref_1_claim_1");
    assert_eq!(graph["evidence"][0]["evidence_id"], "source_ref_1_evidence_1");
    assert_eq!(graph["evidence"][0]["quote_ref"], "source_ref_1_quote_1");
    assert_eq!(graph["allowed_refs"]["segment_refs"][0], "source_ref_1_segment_1");
    assert_eq!(graph["allowed_refs"]["key_point_refs"][0], "source_ref_1_key_point_1");
    assert_eq!(graph["allowed_refs"]["quote_refs"][0], "source_ref_1_quote_1");
    assert_eq!(graph["allowed_refs"]["claim_refs"][0], "source_ref_1_claim_1");
    assert_eq!(graph["allowed_refs"]["evidence_refs"][0], "source_ref_1_evidence_1");
}

#[test]
fn textless_segment_is_kept_as_structural_navigation() {
    let mut parsed = parsed_output();
    parsed["video_candidate"]["segment_candidates"] =
        serde_json::json!([{ "title": null, "summary_text": null, "material_refs": ["material_ref_1"] }]);

    let graph = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect("graph");

    assert_eq!(graph["segments"].as_array().expect("segments").len(), 1);
    assert!(graph["segments"][0]["title"].is_null());
    assert!(graph["segments"][0]["summary_text"].is_null());
}

#[test]
fn blank_key_point_is_skipped_with_graph_warning() {
    let mut parsed = parsed_output();
    parsed["video_candidate"]["key_point_candidates"] =
        serde_json::json!([{ "text": "   ", "material_refs": ["material_ref_1"] }]);

    let graph = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect("graph");

    assert!(graph["key_points"].as_array().expect("key_points").is_empty());
    assert_eq!(graph["warnings"][0]["code"], "blank_key_point_candidate");
}

#[test]
fn malformed_candidate_container_is_rejected() {
    let mut parsed = parsed_output();
    parsed["video_candidate"]["quote_candidates"] = serde_json::json!({ "not": "an array" });

    let error = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect_err("malformed container rejected");

    assert!(error.message.contains("quote_candidates must be an array"));
    assert_eq!(error.object_path.as_deref(), Some("$.video_candidate.quote_candidates"));
}

#[test]
fn invalid_material_ref_is_rejected() {
    let mut parsed = parsed_output();
    parsed["claim_candidates"][0]["material_refs"] = serde_json::json!(["live_library_ref"]);

    let error = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect_err("unknown material ref rejected");

    assert!(error.message.contains("unknown material_ref live_library_ref"));
}

#[test]
fn evidence_quote_candidate_index_must_point_to_retained_quote_candidate() {
    let mut parsed = parsed_output();
    parsed["evidence_fragment_candidates"][0]["quote_candidate_index"] = serde_json::json!(999);

    let error = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect_err("unknown quote candidate index rejected");

    assert!(error.message.contains("unknown quote_candidate_index 999"));
}

#[test]
fn provider_output_must_not_supply_backend_refs_or_ids() {
    for backend_key in [
        "segment_ref",
        "key_point_ref",
        "quote_ref",
        "claim_id",
        "evidence_id",
        "source_ref_id",
        "provider_id",
    ] {
        let mut parsed = parsed_output();
        parsed["claim_candidates"][0][backend_key] = serde_json::json!("provider-owned");

        let error =
            build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
                .expect_err("provider-owned backend ref rejected");

        let expected_path = format!("$.claim_candidates[0].{backend_key}");
        assert!(error.message.contains(&format!("{backend_key} is backend-owned")));
        assert_eq!(
            error.object_path.as_deref(),
            Some(expected_path.as_str())
        );
    }
}

#[test]
fn evidence_index_pointing_to_skipped_quote_candidate_is_rejected() {
    let mut parsed = parsed_output();
    parsed["video_candidate"]["quote_candidates"][0]["text"] = serde_json::json!(" ");
    parsed["evidence_fragment_candidates"][0]["quote_candidate_index"] = serde_json::json!(0);

    let error = build_source_intermediate_entities(&input(), 1001, Some("Video title"), &parsed, 1)
        .expect_err("index pointing to skipped quote rejected");

    assert!(error.message.contains("quote_candidate_index 0 points to skipped quote candidate"));
    assert_eq!(
        error.object_path.as_deref(),
        Some("$.evidence_fragment_candidates[0].quote_candidate_index")
    );
}
```

- [x] **Step 3: Run tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary::entities_tests
```

Expected: FAIL because `entities` module and `build_source_intermediate_entities` do not exist.

- [x] **Step 4: Implement graph builder**

Create `src-tauri/src/prompt_packs/youtube_summary/entities.rs` with these public items and helper shape:

```rust
use std::collections::HashSet;

use sqlx::SqlitePool;

use crate::compression::{compress_text, decompress_text};
use crate::error::{AppError, AppResult};
use crate::prompt_packs::stage_io::{
    build_transcript_analysis_stage_input, insert_stage_artifact_in_pool,
    TranscriptAnalysisStageInput,
};
use crate::prompt_packs::validation::{
    quarantine_prompt_pack_validation_error, PromptPackValidationError,
};

pub(crate) const INTERMEDIATE_ENTITIES_ARTIFACT_KIND: &str = "intermediate_entities";
pub(crate) const YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND: &str =
    "youtube_summary_intermediate_entities";

pub(crate) fn build_source_intermediate_entities(
    input: &TranscriptAnalysisStageInput,
    source_snapshot_id: i64,
    title: Option<&str>,
    parsed: &serde_json::Value,
    attempt_number: i64,
) -> Result<serde_json::Value, PromptPackValidationError> {
    validate_graph_kind_for_pack(&input.pack_id)?;
    let allowed_material_refs = input.allowed_material_refs.iter().cloned().collect::<HashSet<_>>();
    let mut warnings = Vec::new();

    let video_candidate = parsed.get("video_candidate").ok_or_else(|| validation_error(
        "missing required key video_candidate",
        "$.video_candidate",
    ))?;

    let segments = build_segments(video_candidate, &allowed_material_refs, &mut warnings)?;
    let quotes = build_quotes(video_candidate, &allowed_material_refs, &segments, &mut warnings)?;
    let key_points = build_key_points(video_candidate, &allowed_material_refs, &segments, &mut warnings)?;
    let claims = build_claims(parsed, &allowed_material_refs, &mut warnings)?;
    let evidence = build_evidence(parsed, &allowed_material_refs, &quotes, &mut warnings)?;

    Ok(serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "graph_kind": YOUTUBE_SUMMARY_INTERMEDIATE_GRAPH_KIND,
        "run_id": input.run_id,
        "attempt_number": attempt_number,
        "sources": [{
            "source_ref_id": input.source_ref_id,
            "source_snapshot_id": source_snapshot_id,
            "title": title
        }],
        "segments": segments,
        "key_points": key_points,
        "quotes": quotes,
        "claims": claims,
        "evidence": evidence,
        "warnings": warnings,
        "allowed_refs": allowed_refs(&input.source_ref_id, &segments, &key_points, &quotes, &claims, &evidence)
    }))
}

fn validate_graph_kind_for_pack(pack_id: &str) -> Result<(), PromptPackValidationError> {
    if pack_id == "youtube_summary" {
        Ok(())
    } else {
        Err(validation_error(
            format!("unsupported intermediate graph pack_id {pack_id}"),
            "$.pack_id",
        ))
    }
}

fn validation_error(
    message: impl Into<String>,
    object_path: impl Into<String>,
) -> PromptPackValidationError {
    PromptPackValidationError {
        message: message.into(),
        object_path: Some(object_path.into()),
    }
}
```

Implement the helper functions named above with these exact rules:

- `candidate_array(parent, key, path, missing_is_empty)` returns an empty vector only for missing `video_candidate.segment_candidates`, `video_candidate.key_point_candidates`, and `video_candidate.quote_candidates`.
- Present non-array candidate containers return `PromptPackValidationError`.
- Candidate items must be objects.
- `material_refs` absent means `[]`; present non-array or non-string item returns `PromptPackValidationError`.
- Unknown material refs return `PromptPackValidationError` and never consult live Library tables.
- Segment candidates are structural navigation entities. A segment may be retained with both `title` and `summary_text` null/blank as long as the candidate shape and material refs are valid.
- Key points, quotes, claims, and evidence are semantic entities. They require non-blank `text`; blank text is skipped with a warning instead of becoming a textless entity.
- Transcript-analysis parsed output must not contain backend-owned fields in candidate objects. Deny these exact keys anywhere under candidate containers: `segment_ref`, `key_point_ref`, `quote_ref`, `claim_id`, `evidence_id`, `source_ref_id`, and any key ending in `_id`.
- In this MVP, candidate-level provider metadata IDs are also outside the transcript-analysis output contract. A field like `provider_id` is rejected by the `*_id` rule; provider relationships must be expressed through `material_refs` and optional candidate indexes only.
- Candidate-to-candidate linkage uses zero-based provider candidate indexes, not backend refs: optional `segment_candidate_index` on key point/quote candidates and optional `quote_candidate_index` on evidence candidates.
- Present candidate index fields must be non-negative integers. Out-of-range indexes, or indexes pointing at a skipped candidate, return `PromptPackValidationError`.
- The builder maps valid candidate indexes to newly assigned backend refs in graph output. Missing candidate index fields become `null` graph refs.
- Base `validate_transcript_analysis_output` may allow `segment_candidate_index` and `quote_candidate_index` structurally. The graph builder owns their semantic validation, including type, range, skipped-candidate checks, and quarantine object paths. Do not move candidate-index semantic checks into the base validator in this MVP.
- `source_ref_id` is a backend-generated run-local safe identifier (`source_ref_N`) and may be embedded into derived refs.
- Refs are source-qualified in source graph artifacts: `{source_ref_id}_segment_1`, `{source_ref_id}_key_point_1`, `{source_ref_id}_quote_1`, `{source_ref_id}_claim_1`, `{source_ref_id}_evidence_1`.
- Never emit local-only refs like `claim_1` or `evidence_1` inside a source-scoped graph artifact. This keeps merged `allowed_refs` unique without a merge-time remap.
- Segments get `{source_ref_id}_segment_1`, `{source_ref_id}_segment_2`, by retained candidate order and may keep null/blank text fields.
- Key points get `{source_ref_id}_key_point_1`, `{source_ref_id}_key_point_2`, by retained candidate order and blank `text` skips with warning code `blank_key_point_candidate`.
- Quotes get `{source_ref_id}_quote_1`, `{source_ref_id}_quote_2`, by retained candidate order and blank `text` skips with warning code `blank_quote_candidate`.
- Claims get `{source_ref_id}_claim_1`, `{source_ref_id}_claim_2`, by retained candidate order and blank `text` skips with warning code `blank_claim_candidate`.
- Evidence gets `{source_ref_id}_evidence_1`, `{source_ref_id}_evidence_2`, by retained candidate order and blank `text` skips with warning code `blank_evidence_candidate`.
- Graph `segment_ref` on key points/quotes is assigned only from valid `segment_candidate_index`.
- Graph `quote_ref` on evidence is assigned only from valid `quote_candidate_index`.

- [x] **Step 5: Run entity tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary::entities_tests
```

Expected: PASS for all entity builder tests.

- [x] **Step 6: Commit**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\mod.rs src-tauri\src\prompt_packs\youtube_summary\entities.rs src-tauri\src\prompt_packs\youtube_summary\entities_tests.rs
git commit -m "feat: build youtube summary intermediate entities"
```

---

## Task 2: Persist Intermediate Entity Artifacts For Transcript Success

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/entities.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/outputs.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/transcript_execution.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/execution.rs`
- Modify: `src-tauri/src/prompt_packs/json_repair.rs`
- Modify: `src-tauri/src/prompt_packs/validation.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/execution_tests.rs`

- [ ] **Step 1: Add failing artifact persistence tests**

In `src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs`, add:

```rust
#[tokio::test]
async fn execute_transcript_analysis_stage_persists_intermediate_entities_artifact() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    execute_transcript_analysis_stage_with_completion(
        &pool,
        stage_id,
        fake_completion_with_valid_transcript_analysis_json(),
    )
    .await
    .expect("execute transcript stage");

    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    assert!(artifacts.contains(&("metrics".to_string(), 1, 4)));
    assert!(artifacts.contains(&("intermediate_entities".to_string(), 1, 5)));
}

#[tokio::test]
async fn repaired_transcript_analysis_persists_intermediate_entities_for_repair_attempt() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    crate::prompt_packs::json_repair::execute_transcript_analysis_stage_repair_completion(
        &pool,
        stage_id,
        fake_completion_with_valid_transcript_analysis_json(),
        2,
    )
    .await
    .expect("repair transcript stage");

    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    assert!(artifacts.contains(&("metrics".to_string(), 2, 4)));
    assert!(artifacts.contains(&("intermediate_entities".to_string(), 2, 5)));
}

#[tokio::test]
async fn malformed_intermediate_candidates_are_quarantined_without_graph_artifact() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    let error = execute_transcript_analysis_stage_with_completion(
        &pool,
        stage_id,
        fake_completion_with_malformed_intermediate_candidates_json(),
    )
    .await
    .expect_err("malformed graph candidates fail stage");

    assert!(error.message.contains("quote_candidates must be an array"));
    let (status, error_message): (String, Option<String>) = sqlx::query_as(
        "SELECT stage_status, error_message FROM prompt_pack_stage_runs WHERE id = ?",
    )
    .bind(stage_id)
    .fetch_one(&pool)
    .await
    .expect("stage status");
    assert_eq!(status, "running");
    assert!(error_message.is_none());
    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    assert!(!artifacts.contains(&("metrics".to_string(), 1, 4)));
    assert!(!artifacts.iter().any(|(kind, _, _)| kind == "intermediate_entities"));
    assert!(!artifacts.iter().any(|(kind, _, _)| kind == "error"));
    assert_quarantine_count(&pool, stage_id, 1).await;
}

#[tokio::test]
async fn repair_graph_build_failure_does_not_write_repaired_parsed_output() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    crate::prompt_packs::json_repair::execute_transcript_analysis_stage_repair_completion(
        &pool,
        stage_id,
        fake_completion_with_malformed_intermediate_candidates_json(),
        2,
    )
    .await
    .expect_err("repair graph failure");

    let (status, error_message): (String, Option<String>) = sqlx::query_as(
        "SELECT stage_status, error_message FROM prompt_pack_stage_runs WHERE id = ?",
    )
    .bind(stage_id)
    .fetch_one(&pool)
    .await
    .expect("stage status");
    assert_eq!(status, "running");
    assert!(error_message.is_none());
    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    assert!(!artifacts.contains(&("parsed_output".to_string(), 2, 3)));
    assert!(!artifacts.contains(&("metrics".to_string(), 2, 4)));
    assert!(!artifacts.contains(&("intermediate_entities".to_string(), 2, 5)));
    assert!(!artifacts.iter().any(|(kind, _, _)| kind == "error"));
    assert_quarantine_count(&pool, stage_id, 1).await;
}

#[tokio::test]
async fn execution_graph_build_failure_after_failed_repair_marks_transcript_failed_once() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let repair_calls = Arc::new(AtomicUsize::new(0));

    let outcome = execute_youtube_summary_run_with_stage_executor(&pool, 1, |request| {
        let repair_calls = Arc::clone(&repair_calls);
        async move {
            match request {
                YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(_) => {
                    Ok(fake_completion_with_malformed_intermediate_candidates_json())
                }
                YoutubeSummaryStageExecutionRequest::JsonRepair(_) => {
                    repair_calls.fetch_add(1, Ordering::SeqCst);
                    Ok(fake_completion_with_malformed_intermediate_candidates_json())
                }
                YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                    panic!("single-video failed transcript run should not request synthesis")
                }
            }
        }
    })
    .await
    .expect("execute run");

    let stage_id = transcript_analysis_stage_id(&pool, 1).await;
    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    let error_artifacts = artifacts
        .iter()
        .filter(|(kind, _, _)| kind == "error")
        .collect::<Vec<_>>();
    let (status, error_message): (String, Option<String>) = sqlx::query_as(
        "SELECT stage_status, error_message FROM prompt_pack_stage_runs WHERE id = ?",
    )
    .bind(stage_id)
    .fetch_one(&pool)
    .await
    .expect("stage status");

    assert_eq!(outcome.run_status, "failed");
    assert_eq!(repair_calls.load(Ordering::SeqCst), 1);
    assert_eq!(status, "failed");
    assert!(error_message.unwrap_or_default().contains("quote_candidates must be an array"));
    assert_eq!(error_artifacts.len(), 1);
    assert_eq!(error_artifacts[0], &("error".to_string(), 2, 99));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary::outputs_tests
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary::execution_tests
```

Expected: FAIL because no `intermediate_entities` artifact is written, graph build failures are not quarantined, and execution does not yet prove single-owner terminal failure marking.

- [ ] **Step 3: Add quarantine helper and split build/insert helpers**

In `validation.rs`, expose a reusable helper for validation-style quarantine rows:

```rust
pub(crate) async fn quarantine_prompt_pack_validation_error(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    output: &serde_json::Value,
    error: PromptPackValidationError,
) -> AppResult<()> {
    let object_path = error.object_path.clone().unwrap_or_else(|| "$".to_string());
    let validation_message = error.message.clone();
    let candidate = value_at_path(output, &object_path).unwrap_or(output);
    let content = serde_json::to_string(candidate).unwrap_or_else(|_| "{}".to_string());
    sqlx::query(
        "INSERT INTO prompt_pack_result_quarantine_artifacts (
            run_id, stage_run_id, object_path, reason, content_json_zstd, created_at
         )
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(stage_run_id)
    .bind(&object_path)
    .bind(&validation_message)
    .bind(compress_text(&content).unwrap_or_default())
    .bind(crate::time::now_rfc3339_utc())
    .execute(pool)
    .await
    .map_err(|db_error| {
        AppError::internal(format!(
            "quarantine prompt pack validation error `{validation_message}` failed: {db_error}"
        ))
    })?;
    Ok(())
}
```

Update existing transcript/synthesis quarantine helpers to call this helper instead of duplicating insert logic, then return their existing validation error.

MVP note: `value_at_path` is best-effort. It must store the exact `object_path` and `reason`, but if the resolver cannot locate nested paths such as `$.video_candidate.quote_candidates` or deep synthesis refs, it may quarantine the whole output JSON. Do not expand the path resolver in this task unless a test already depends on exact nested content extraction.

In `transcript_execution.rs`, add an attempt-aware failure helper for the execution layer:

```rust
pub(crate) async fn mark_transcript_stage_failed_for_attempt(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    attempt_number: i64,
    error: &str,
) -> AppResult<()> {
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "error",
        attempt_number,
        99,
        &serde_json::json!({ "error": error }).to_string(),
    )
    .await?;
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'failed',
             error_message = ?,
             latest_message = ?,
             completed_at = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(error)
    .bind(error)
    .bind(crate::time::now_rfc3339_utc())
    .bind(crate::time::now_rfc3339_utc())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

Keep existing `mark_transcript_stage_failed` as a compatibility wrapper that calls this helper with `attempt_number = 1`.

Ownership rule: transcript output helpers (`outputs.rs` and transcript repair completion in `json_repair.rs`) do not mark transcript stages terminal. They may set a stage to `running`, write raw/quarantine artifacts, and return errors. `execution.rs` is the single owner for terminal transcript failure marking and `error` artifacts.

In `entities.rs`, add:

```rust
pub(crate) async fn load_intermediate_entities_context_for_transcript_stage(
    pool: &SqlitePool,
    stage_run_id: i64,
) -> AppResult<(TranscriptAnalysisStageInput, i64, Option<String>)> {
    let input = build_transcript_analysis_stage_input(pool, stage_run_id).await?;
    let (source_snapshot_id, title): (i64, Option<String>) = sqlx::query_as(
        "SELECT snapshots.id, snapshots.title
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_run_source_snapshots snapshots
           ON snapshots.id = stages.source_snapshot_id
          AND snapshots.run_id = stages.run_id
         WHERE stages.id = ?",
    )
    .bind(stage_run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok((input, source_snapshot_id, title))
}

pub(crate) async fn build_or_quarantine_intermediate_entities_for_transcript_stage(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    parsed: &serde_json::Value,
    attempt_number: i64,
) -> AppResult<serde_json::Value> {
    let (input, source_snapshot_id, title) =
        load_intermediate_entities_context_for_transcript_stage(pool, stage_run_id).await?;
    match build_source_intermediate_entities(
        &input,
        source_snapshot_id,
        title.as_deref(),
        parsed,
        attempt_number,
    ) {
        Ok(graph) => Ok(graph),
        Err(error) => {
            let validation_message = error.message.clone();
            quarantine_prompt_pack_validation_error(pool, run_id, stage_run_id, parsed, error).await?;
            Err(AppError::validation(validation_message))
        }
    }
}

pub(crate) async fn insert_intermediate_entities_artifact(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    graph: &serde_json::Value,
    attempt_number: i64,
) -> AppResult<()> {
    let content = serde_json::to_string(&graph)
        .map_err(|error| AppError::internal(format!("serialize intermediate entities: {error}")))?;

    // Artifact index 5 is intentionally after metrics (#4) for stable audit ordering.
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        INTERMEDIATE_ENTITIES_ARTIFACT_KIND,
        attempt_number,
        5,
        &content,
    )
    .await
}
```

Implementation notes:

- `load_intermediate_entities_context_for_transcript_stage` handles DB/input-loading failures as `AppError`; `build_or_quarantine_intermediate_entities_for_transcript_stage` handles graph validation failures by writing a quarantine row and returning `AppError::validation`.
- Graph artifact ordering is fixed: `prompt_input #1`, `raw_output #2`, `parsed_output #3`, `metrics #4`, `intermediate_entities #5`.
- The graph artifact helper must only be called after `metrics` has been inserted in normal and repair paths.

- [ ] **Step 4: Call graph build/persistence in normal transcript output path**

In `outputs.rs`, import:

```rust
use super::entities::{
    build_or_quarantine_intermediate_entities_for_transcript_stage,
    insert_intermediate_entities_artifact,
};
```

After transcript output validation succeeds and before `metrics`, build the graph in memory:

```rust
let intermediate_graph = build_or_quarantine_intermediate_entities_for_transcript_stage(
    pool,
    run_id,
    stage_run_id,
    &parsed,
    1,
)
.await?;
```

After inserting `metrics #4` and before marking the transcript stage `succeeded`, persist the graph as `intermediate_entities #5`:

```rust
insert_intermediate_entities_artifact(pool, run_id, stage_run_id, &intermediate_graph, 1).await?;
```

- [ ] **Step 5: Call graph build/persistence in repaired transcript output path**

In `src-tauri/src/prompt_packs/json_repair.rs`, import:

```rust
use super::youtube_summary::entities::{
    build_or_quarantine_intermediate_entities_for_transcript_stage,
    insert_intermediate_entities_artifact,
};
```

In `execute_transcript_analysis_stage_repair_completion`, build the graph after repaired transcript validation and before writing repaired `parsed_output`:

```rust
let intermediate_graph = build_or_quarantine_intermediate_entities_for_transcript_stage(
    pool,
    run_id,
    stage_run_id,
    &parsed,
    attempt_number,
)
.await?;
```

Then write repaired `parsed_output #3`, repaired `metrics #4`, and `intermediate_entities #5` in that order. If graph build fails, only repaired `raw_output #2` should exist for that attempt; repaired `parsed_output`, `metrics`, and `intermediate_entities` must not be written.

- [ ] **Step 6: Keep terminal transcript failure marking in execution layer**

In `execution.rs`, import `mark_transcript_stage_failed_for_attempt` from `transcript_execution`.

When `execute_transcript_analysis_stage_repair_completion(..., attempt_number = 2)` returns `Err(error)`, mark the transcript stage failed with attempt 2:

```rust
mark_transcript_stage_failed_for_attempt(
    pool,
    run_id,
    stage.stage_run_id,
    2,
    &error.message,
)
.await?;
```

Apply the same attempt-2 marking when the JSON repair provider request itself returns `YoutubeSummaryStageExecutionError::Failed(error)`.

Do not add terminal failure marking inside `execute_transcript_analysis_stage_with_completion` or `execute_transcript_analysis_stage_repair_completion`; doing so creates duplicate `error` artifacts in full execution.

- [ ] **Step 7: Run artifact and execution tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary::outputs_tests
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary::execution_tests
```

Expected: PASS with graph artifacts after metrics, graph validation failures quarantined, low-level helpers not writing `error` artifacts, and full execution writing exactly one terminal `error` artifact.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\entities.rs src-tauri\src\prompt_packs\youtube_summary\outputs.rs src-tauri\src\prompt_packs\youtube_summary\outputs_tests.rs src-tauri\src\prompt_packs\youtube_summary\execution.rs src-tauri\src\prompt_packs\youtube_summary\execution_tests.rs src-tauri\src\prompt_packs\youtube_summary\transcript_execution.rs src-tauri\src\prompt_packs\json_repair.rs src-tauri\src\prompt_packs\validation.rs
git commit -m "feat: persist youtube summary intermediate entities"
```

---

## Task 3: Merge Entity Graphs Into Synthesis Input

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/entities.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/synthesis_input.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/synthesis_input_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs` if fixture side-effect expectations need updates.
- Modify: `src-tauri/src/prompt_packs/youtube_summary/execution_tests.rs` if fixture side-effect expectations need updates.

- [ ] **Step 1: Update transcript fixture helpers to use real output path**

In `test_support.rs`, update `persist_succeeded_transcript_stage_fixtures` so it no longer manually updates `stage_status` or inserts only a `parsed_output` artifact.

Instead, for each transcript stage row, call `execute_transcript_analysis_stage_with_completion` with:

```rust
LlmCompletion {
    text: transcript_analysis_json(fixture.summary, fixture.claim, fixture.evidence),
    input_tokens: Some(10),
    output_tokens: Some(10),
    latency_ms: 5,
}
```

This helper depends on Task 2 and must write the same artifact sequence as production transcript execution, including `metrics #4` and `intermediate_entities #5`.

This is an intentional fixture fidelity change. Existing tests that inspect artifact counts, artifact order, timestamps, or transcript stage status may need expectation updates because the helper now also writes `prompt_input`, `raw_output`, `metrics`, and `intermediate_entities`. Keep fixture inputs deterministic (`latency_ms`, token counts, completion text), but do not try to suppress production-path side effects in this helper.

- [ ] **Step 2: Run fixture regression tests**

Run the broader YouTube Summary test filter immediately after changing `persist_succeeded_transcript_stage_fixtures` and before adding new failing synthesis-input tests:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
```

Expected: PASS with more than zero tests run. If existing tests fail because they assert the old hand-written fixture artifact shape, update those expectations in the same step so they match the production-path fixture behavior.

- [ ] **Step 3: Add failing synthesis input tests**

In `synthesis_input_tests.rs`, add:

Import `load_merged_intermediate_entities_for_run` from `super::entities` in this test module.

```rust
#[tokio::test]
async fn build_synthesis_stage_input_merges_intermediate_graphs_and_allowed_refs() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "First summary",
                claim: "First claim",
                evidence: "First evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("fixtures");

    let input = build_synthesis_stage_input(&pool, 1)
        .await
        .expect("synthesis input");

    assert_eq!(input["canonical_graph"]["sources"].as_array().expect("sources").len(), 2);
    assert_eq!(input["allowed_refs"]["claim_refs"].as_array().expect("claim refs").len(), 2);
    assert_eq!(input["allowed_refs"]["evidence_refs"].as_array().expect("evidence refs").len(), 2);
    assert_ne!(
        input["allowed_refs"]["claim_refs"][0],
        input["allowed_refs"]["claim_refs"][1],
        "source-qualified graph refs must not collide across source artifacts"
    );
    assert!(
        input["allowed_refs"]["claim_refs"]
            .as_array()
            .expect("claim refs")
            .iter()
            .all(|value| value.as_str().unwrap_or("").contains("_claim_"))
    );
    assert!(input["allowed_refs"].get("segment_refs").is_some());
    assert!(input["allowed_refs"].get("key_point_refs").is_some());
    assert!(input["allowed_refs"].get("quote_refs").is_some());
    assert!(input["canonical_graph"].get("warnings").is_none());
}

#[tokio::test]
async fn build_synthesis_stage_input_orders_graph_by_source_snapshot_id() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "First summary",
                claim: "First claim",
                evidence: "First evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("fixtures");

    let input = build_synthesis_stage_input(&pool, 1).await.expect("input");
    let sources = input["canonical_graph"]["sources"].as_array().expect("sources");

    let first_id = sources[0]["source_snapshot_id"].as_i64().expect("first id");
    let second_id = sources[1]["source_snapshot_id"].as_i64().expect("second id");
    assert!(first_id < second_id);
}

#[tokio::test]
async fn load_merged_intermediate_entities_rejects_duplicate_refs_across_sources() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "First summary",
                claim: "First claim",
                evidence: "First evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("fixtures");

    overwrite_intermediate_entities_artifact_with_local_refs(&pool, 1, "source_ref_1", "claim_1").await;
    overwrite_intermediate_entities_artifact_with_local_refs(&pool, 1, "source_ref_2", "claim_1").await;

    let error = load_merged_intermediate_entities_for_run(&pool, 1)
        .await
        .expect_err("duplicate refs rejected");

    assert!(error.message.contains("duplicate ref claim_1"));
    assert!(error.message.contains("allowed_refs.claim_refs"));
}
```

Add `overwrite_intermediate_entities_artifact_with_local_refs` as a test helper that inserts a latest `intermediate_entities` artifact for the requested source with local-only refs (`claim_1`, `evidence_1`) and `artifact_index = 5`. This helper intentionally bypasses `build_source_intermediate_entities` so the duplicate-ref merge guard is tested independently from the builder.

- [ ] **Step 4: Run synthesis input tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary::synthesis_input_tests
```

Expected: FAIL because synthesis input does not include `canonical_graph` or complete `allowed_refs`.

- [ ] **Step 5: Add graph merge loader**

In `entities.rs`, add:

```rust
pub(crate) async fn load_merged_intermediate_entities_for_run(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Option<serde_json::Value>> {
    let rows = sqlx::query_as::<_, (i64, i64, String, Vec<u8>)>(
        "SELECT snapshots.id, stages.id, snapshots.source_ref_id, artifacts.content_zstd
         FROM prompt_pack_run_source_snapshots snapshots
         JOIN prompt_pack_stage_runs stages
           ON stages.run_id = snapshots.run_id
          AND stages.source_snapshot_id = snapshots.id
          AND stages.stage_name = 'youtube_summary/transcript_analysis'
          AND stages.stage_status = 'succeeded'
         JOIN prompt_pack_stage_artifacts artifacts
           ON artifacts.stage_run_id = stages.id
          AND artifacts.artifact_kind = 'intermediate_entities'
          AND artifacts.id = (
              SELECT latest.id
              FROM prompt_pack_stage_artifacts latest
              WHERE latest.stage_run_id = stages.id
                AND latest.artifact_kind = 'intermediate_entities'
              ORDER BY latest.attempt_number DESC, latest.artifact_index DESC, latest.id DESC
              LIMIT 1
          )
         WHERE snapshots.run_id = ?
         ORDER BY snapshots.id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    if rows.is_empty() {
        return Ok(None);
    }

    let mut merged = empty_run_graph(run_id);
    for (_source_snapshot_id, _stage_run_id, _source_ref_id, content_zstd) in rows {
        let text = decompress_text(&content_zstd).map_err(AppError::internal)?;
        let graph: serde_json::Value = serde_json::from_str(&text)
            .map_err(|error| AppError::internal(format!("parse intermediate entities: {error}")))?;
        merge_source_graph(&mut merged, &graph)?;
    }
    Ok(Some(merged))
}
```

Implement `empty_run_graph` and `merge_source_graph` so merged output includes:

- `sources`
- `segments`
- `key_points`
- `quotes`
- `claims`
- `evidence`
- `allowed_refs.source_refs`
- `allowed_refs.segment_refs`
- `allowed_refs.key_point_refs`
- `allowed_refs.quote_refs`
- `allowed_refs.claim_refs`
- `allowed_refs.evidence_refs`

Do not merge source graph `warnings` into the returned merged graph.

`merge_source_graph` must also reject duplicate refs across source artifacts for every allowed-ref bucket. With source-qualified refs this should never happen; if it does, return an `AppError::internal` that names the duplicated ref and bucket.

- [ ] **Step 6: Add graph fields to synthesis input**

In `synthesis_input.rs`, import:

```rust
use super::entities::load_merged_intermediate_entities_for_run;
```

Before `Ok(serde_json::json!({ ... }))`, add:

```rust
let merged_graph = load_merged_intermediate_entities_for_run(pool, run_id).await?;
let canonical_graph = merged_graph
    .as_ref()
    .map(|graph| {
        serde_json::json!({
            "sources": graph.get("sources").cloned().unwrap_or_else(|| serde_json::json!([])),
            "segments": graph.get("segments").cloned().unwrap_or_else(|| serde_json::json!([])),
            "key_points": graph.get("key_points").cloned().unwrap_or_else(|| serde_json::json!([])),
            "quotes": graph.get("quotes").cloned().unwrap_or_else(|| serde_json::json!([])),
            "claims": graph.get("claims").cloned().unwrap_or_else(|| serde_json::json!([])),
            "evidence": graph.get("evidence").cloned().unwrap_or_else(|| serde_json::json!([]))
        })
    })
    .unwrap_or_else(|| serde_json::json!({
        "sources": [],
        "segments": [],
        "key_points": [],
        "quotes": [],
        "claims": [],
        "evidence": []
    }));
let allowed_refs = merged_graph
    .as_ref()
    .and_then(|graph| graph.get("allowed_refs").cloned())
    .unwrap_or_else(|| serde_json::json!({
        "source_refs": [],
        "segment_refs": [],
        "key_point_refs": [],
        "quote_refs": [],
        "claim_refs": [],
        "evidence_refs": []
    }));
```

Add these fields to the returned synthesis input object:

```rust
"canonical_graph": canonical_graph,
"allowed_refs": allowed_refs
```

- [ ] **Step 7: Run synthesis input tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary::synthesis_input_tests
```

Expected: PASS.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\entities.rs src-tauri\src\prompt_packs\youtube_summary\test_support.rs src-tauri\src\prompt_packs\youtube_summary\synthesis_input.rs src-tauri\src\prompt_packs\youtube_summary\synthesis_input_tests.rs src-tauri\src\prompt_packs\youtube_summary\outputs_tests.rs src-tauri\src\prompt_packs\youtube_summary\execution_tests.rs
git commit -m "feat: add intermediate graph to synthesis input"
```

---

## Task 4: Update Runtime Prompts And Ref Validation

**Files:**
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Modify: `src-tauri/src/prompt_packs/validation.rs`
- Modify: tests inside `src-tauri/src/prompt_packs/validation.rs`
  - Add transcript-analysis prompt coverage for candidate indexes and backend-ref denial.

- [ ] **Step 1: Add failing synthesis validation tests**

In `validation.rs` tests, add:

```rust
#[test]
fn synthesis_output_rejects_unknown_claim_ref() {
    let output = valid_synthesis_output_with_refs(
        serde_json::json!(["source_ref_1"]),
        serde_json::json!(["claim_999"]),
        serde_json::json!([])
    );
    let allowed_sources = HashSet::from(["source_ref_1".to_string()]);
    let allowed_claims = HashSet::from(["source_ref_1_claim_1".to_string()]);
    let allowed_evidence = HashSet::from(["source_ref_1_evidence_1".to_string()]);

    let error = validate_synthesis_output_with_allowed_refs(
        &output,
        &allowed_sources,
        &allowed_claims,
        &allowed_evidence,
    )
    .expect_err("unknown claim rejected");

    assert!(error.message.contains("unknown claim_ref claim_999"));
}

#[test]
fn synthesis_output_rejects_direct_segment_key_point_or_quote_refs_inside_synthesis_candidate() {
    for key in ["segment_refs", "key_point_refs", "quote_refs"] {
        let mut output = valid_synthesis_output_with_refs(
            serde_json::json!(["source_ref_1"]),
            serde_json::json!([]),
            serde_json::json!([])
        );
        output["synthesis_candidate"]["cross_video_themes"][0][key] = serde_json::json!(["not_allowed"]);

        let allowed_sources = HashSet::from(["source_ref_1".to_string()]);
        let allowed_claims = HashSet::new();
        let allowed_evidence = HashSet::new();
        let error = validate_synthesis_output_with_allowed_refs(
            &output,
            &allowed_sources,
            &allowed_claims,
            &allowed_evidence,
        )
        .expect_err("direct intermediate ref rejected inside synthesis_candidate");

        assert!(error.message.contains(key));
    }
}

#[test]
fn synthesis_output_rejects_non_array_or_non_string_ref_values() {
    let allowed_sources = HashSet::from(["source_ref_1".to_string()]);
    let allowed_claims = HashSet::from(["source_ref_1_claim_1".to_string()]);
    let allowed_evidence = HashSet::from(["source_ref_1_evidence_1".to_string()]);

    let mut non_array = valid_synthesis_output_with_refs(
        serde_json::json!(["source_ref_1"]),
        serde_json::json!("source_ref_1_claim_1"),
        serde_json::json!([])
    );
    let error = validate_synthesis_output_with_allowed_refs(
        &non_array,
        &allowed_sources,
        &allowed_claims,
        &allowed_evidence,
    )
    .expect_err("claim_refs string rejected");
    assert!(error.message.contains("claim_refs must be an array of strings"));

    non_array["synthesis_candidate"]["cross_video_themes"][0]["claim_refs"] =
        serde_json::json!(["source_ref_1_claim_1", 42]);
    let error = validate_synthesis_output_with_allowed_refs(
        &non_array,
        &allowed_sources,
        &allowed_claims,
        &allowed_evidence,
    )
    .expect_err("non-string claim ref rejected");
    assert!(error.message.contains("claim_refs must be an array of strings"));
}
```

Add helper in tests:

```rust
fn valid_synthesis_output_with_refs(
    source_refs: serde_json::Value,
    claim_refs: serde_json::Value,
    evidence_refs: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "synthesis_candidate": {
            "summary_text": "Summary",
            "cross_video_themes": [{
                "theme_text": "Theme",
                "source_refs": source_refs,
                "claim_refs": claim_refs,
                "evidence_refs": evidence_refs
            }],
            "common_claims": [],
            "contradictions_across_videos": []
        },
        "limitations": [],
        "warning_candidates": []
    })
}
```

In `runtime.rs` tests, add:

```rust
#[test]
fn transcript_analysis_llm_request_describes_candidate_indexes_and_forbids_backend_refs() {
    let request = build_transcript_analysis_llm_request(
        &TranscriptAnalysisStageExecutionRequest {
            run_id: 42,
            stage_run_id: 1001,
            source_snapshot_id: 501,
            source_ref_id: "source_ref_1".to_string(),
            prompt_input_json: "{\"stage\":\"youtube_summary/transcript_analysis\"}".to_string(),
        },
        None,
        Some("model".to_string()),
        Some(1024),
    );
    let prompt = &request.messages[1].content;

    assert!(prompt.contains("segment_candidate_index"));
    assert!(prompt.contains("quote_candidate_index"));
    assert!(prompt.contains("zero-based"));
    assert!(prompt.contains("Do not include backend-owned refs or IDs"));
    assert!(prompt.contains("segment_ref"));
    assert!(prompt.contains("quote_ref"));
    assert!(prompt.contains("source_ref_id"));
}

#[test]
fn synthesis_llm_request_describes_allowed_refs_and_forbids_direct_intermediate_refs() {
    let request = build_synthesis_llm_request(
        42,
        2001,
        "{\"allowed_refs\":{}}".to_string(),
        None,
        Some("model".to_string()),
        Some(1024),
    );
    let prompt = &request.messages[1].content;

    assert!(prompt.contains("allowed_refs.source_refs"));
    assert!(prompt.contains("allowed_refs.claim_refs"));
    assert!(prompt.contains("allowed_refs.evidence_refs"));
    assert!(prompt.contains("Do not emit segment_refs"));
    assert!(prompt.contains("key_point_refs"));
    assert!(prompt.contains("quote_refs"));
}
```

- [ ] **Step 2: Run validation and prompt tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs::validation::tests
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs::runtime::tests
```

Expected: FAIL because `validate_synthesis_output_with_allowed_refs` does not exist and the runtime prompts do not yet describe candidate indexes / graph allowed refs.

- [ ] **Step 3: Implement allowed-ref validation**

In `validation.rs`, add:

```rust
pub(crate) fn validate_synthesis_output_with_allowed_refs(
    output: &serde_json::Value,
    allowed_source_refs: &HashSet<String>,
    allowed_claim_refs: &HashSet<String>,
    allowed_evidence_refs: &HashSet<String>,
) -> Result<(), PromptPackValidationError> {
    validate_synthesis_output(output, allowed_source_refs)?;
    let candidate = output.get("synthesis_candidate").ok_or_else(|| PromptPackValidationError {
        message: "missing required key synthesis_candidate".to_string(),
        object_path: Some("$.synthesis_candidate".to_string()),
    })?;
    reject_direct_intermediate_refs(candidate, "$.synthesis_candidate")?;
    reject_unknown_refs_in_synthesis(candidate, "$.synthesis_candidate", "claim_refs", allowed_claim_refs)?;
    reject_unknown_refs_in_synthesis(candidate, "$.synthesis_candidate", "evidence_refs", allowed_evidence_refs)?;
    Ok(())
}
```

Implement helpers:

```rust
fn reject_direct_intermediate_refs(
    value: &serde_json::Value,
    path: &str,
) -> Result<(), PromptPackValidationError> {
    match value {
        serde_json::Value::Object(map) => {
            for forbidden in ["segment_refs", "key_point_refs", "quote_refs"] {
                if map.contains_key(forbidden) {
                    return Err(PromptPackValidationError {
                        message: format!("direct {forbidden} are not allowed in synthesis output v1"),
                        object_path: Some(format!("{path}.{forbidden}")),
                    });
                }
            }
            for (key, child) in map {
                reject_direct_intermediate_refs(child, &format!("{path}.{key}"))?;
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                reject_direct_intermediate_refs(child, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
```

`reject_direct_intermediate_refs` is intentionally scoped to the `synthesis_candidate` subtree. Do not scan top-level `limitations` or `warning_candidates`, so backend diagnostics can use similarly named fields later without breaking v1 model-output validation.

`reject_unknown_refs_in_synthesis` should walk the `synthesis_candidate` subtree recursively. When it sees key `claim_refs` or `evidence_refs`, the value must be an array of strings; a scalar value, object value, or non-string array item is a validation error. Every `claim_refs` string item must exist in `allowed_claim_refs`, and every `evidence_refs` string item must exist in `allowed_evidence_refs`.

- [ ] **Step 4: Update transcript-analysis and synthesis prompts**

In `runtime.rs`, update the transcript-analysis prompt text in `build_transcript_analysis_llm_request`.

In the required JSON shape, show candidate-index fields as optional examples:

```text
"key_point_candidates": [{ "text": "point", "segment_candidate_index": 0, "material_refs": ["allowed material ref"] }],
"quote_candidates": [{ "text": "short quote", "segment_candidate_index": 0, "material_refs": ["allowed material ref"] }],
"evidence_fragment_candidates": [{ "text": "evidence quote or paraphrase", "quote_candidate_index": 0, "material_refs": ["allowed material ref"] }]
```

Replace the backend-owned ID warning with:

```text
Do not include backend-owned refs or IDs such as segment_ref, key_point_ref, quote_ref, claim_id, evidence_id, source_ref_id, segment_id, key_point_id, quote_id, action_item_id, or open_question_id. For optional candidate-to-candidate linkage, use only zero-based segment_candidate_index and quote_candidate_index. Omit candidate index fields when no clear candidate link exists.
```

In `runtime.rs`, update the synthesis prompt text in `build_synthesis_llm_request`:

Replace the sentence:

```text
For this slice, keep claim_refs, evidence_refs, and relation_refs as empty arrays because the backend has not exposed allowed claim/evidence/relation ref maps to synthesis output.
```

With:

```text
Use only source_refs from allowed_refs.source_refs, claim_refs from allowed_refs.claim_refs, and evidence_refs from allowed_refs.evidence_refs. You may use segment_refs, key_point_refs, and quote_refs from allowed_refs only for reasoning over canonical_graph. Do not emit segment_refs, key_point_refs, or quote_refs in the output. Leave claim_refs or evidence_refs empty when no supporting allowed ref exists.
```

- [ ] **Step 5: Run validation and runtime tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs::validation
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs::runtime::tests
```

Expected: PASS.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri\src\prompt_packs\validation.rs src-tauri\src\prompt_packs\runtime.rs
git commit -m "feat: validate youtube synthesis refs"
```

---

## Task 5: Quarantine Invalid Synthesis Refs During Execution

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/entities.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/outputs.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs`
- Modify: `src-tauri/src/prompt_packs/validation.rs`

- [ ] **Step 1: Add failing quarantine tests**

In `outputs_tests.rs`, add:

```rust
#[tokio::test]
async fn execute_synthesis_stage_rejects_unknown_claim_ref_with_quarantine() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture { summary: "A", claim: "Claim A", evidence: "Evidence A" },
            TranscriptStageFixture { summary: "B", claim: "Claim B", evidence: "Evidence B" },
        ],
    )
    .await
    .expect("fixtures");

    let stage_run_id: i64 = sqlx::query_scalar(
        "SELECT id FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
    )
    .fetch_one(&pool)
    .await
    .expect("synthesis stage");

    let completion = LlmCompletion {
        text: serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/synthesis",
            "synthesis_candidate": {
                "summary_text": "Combined",
                "cross_video_themes": [{
                    "theme_text": "Theme",
                    "source_refs": ["source_ref_1"],
                    "claim_refs": ["claim_999"],
                    "evidence_refs": []
                }],
                "common_claims": [],
                "contradictions_across_videos": []
            },
            "limitations": [],
            "warning_candidates": []
        })
        .to_string(),
        input_tokens: Some(10),
        output_tokens: Some(10),
        latency_ms: 5,
    };

    let error = execute_synthesis_stage_with_completion(&pool, stage_run_id, completion)
        .await
        .expect_err("unknown claim ref rejected");

    assert!(error.message.contains("unknown claim_ref claim_999"));
    let quarantine_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts
         WHERE run_id = 1 AND stage_run_id = ?",
    )
    .bind(stage_run_id)
    .fetch_one(&pool)
    .await
    .expect("quarantine count");
    assert_eq!(quarantine_count, 1);
}

#[tokio::test]
async fn execute_synthesis_stage_requires_complete_intermediate_graph() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "First summary",
                claim: "First claim",
                evidence: "First evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("fixtures");
    delete_intermediate_entities_artifact_for_source(&pool, 1, "source_ref_2").await;

    let stage_run_id = synthesis_stage_id(&pool, 1).await;
    let completion = fake_completion_with_valid_synthesis_json_without_claim_refs();

    let error = execute_synthesis_stage_with_completion(&pool, stage_run_id, completion)
        .await
        .expect_err("missing graph is an execution error");

    assert!(error.message.contains("missing complete intermediate_entities graph"));
    let (status, error_message): (String, Option<String>) = sqlx::query_as(
        "SELECT stage_status, error_message FROM prompt_pack_stage_runs WHERE id = ?",
    )
    .bind(stage_run_id)
    .fetch_one(&pool)
    .await
    .expect("synthesis stage status");
    assert_eq!(status, "failed");
    assert!(error_message.unwrap_or_default().contains("missing complete intermediate_entities graph"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary::outputs_tests
```

Expected: FAIL because synthesis output does not validate against graph allowed refs and live synthesis execution does not require a complete graph.

- [ ] **Step 3: Load allowed refs for synthesis stage**

In `entities.rs`, add:

```rust
pub(crate) async fn load_required_allowed_refs_for_live_synthesis(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<AllowedSynthesisRefs> {
    let expected_sources = sqlx::query_as::<_, (i64, String)>(
        "SELECT snapshots.id, snapshots.source_ref_id
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_run_source_snapshots snapshots
           ON snapshots.id = stages.source_snapshot_id
          AND snapshots.run_id = stages.run_id
         WHERE stages.run_id = ?
           AND stages.stage_name = 'youtube_summary/transcript_analysis'
           AND stages.stage_status = 'succeeded'
         ORDER BY snapshots.id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    let graph = load_merged_intermediate_entities_for_run(pool, run_id).await?;
    let Some(graph) = graph else {
        if !expected_sources.is_empty() {
            return Err(AppError::validation(
                "missing complete intermediate_entities graph for live synthesis".to_string(),
            ));
        }
        return Ok(AllowedSynthesisRefs::empty());
    };
    let graph_sources = graph
        .get("sources")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let graph_source_keys = graph_sources
        .iter()
        .map(|source| {
            Ok((
                source
                    .get("source_snapshot_id")
                    .and_then(serde_json::Value::as_i64)
                    .ok_or_else(|| AppError::internal("intermediate graph source missing source_snapshot_id"))?,
                source
                    .get("source_ref_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| AppError::internal("intermediate graph source missing source_ref_id"))?
                    .to_string(),
            ))
        })
        .collect::<AppResult<Vec<_>>>()?;
    if graph_source_keys != expected_sources {
        return Err(AppError::validation(
            "missing complete intermediate_entities graph for live synthesis".to_string(),
        ));
    }
    let allowed = graph.get("allowed_refs").cloned().unwrap_or_else(|| serde_json::json!({}));
    Ok(AllowedSynthesisRefs {
        source_refs: string_set(allowed.get("source_refs")),
        claim_refs: string_set(allowed.get("claim_refs")),
        evidence_refs: string_set(allowed.get("evidence_refs")),
    })
}

pub(crate) struct AllowedSynthesisRefs {
    pub(crate) source_refs: HashSet<String>,
    pub(crate) claim_refs: HashSet<String>,
    pub(crate) evidence_refs: HashSet<String>,
}

impl AllowedSynthesisRefs {
    fn empty() -> Self {
        Self {
            source_refs: HashSet::new(),
            claim_refs: HashSet::new(),
            evidence_refs: HashSet::new(),
        }
    }
}
```

Keep the optional `load_merged_intermediate_entities_for_run` behavior for synthesis input construction and legacy tests. Only live synthesis execution uses the required loader above.

Completeness is an exact ordered set check against successful transcript stage source snapshots: same `(source_snapshot_id, source_ref_id)` values, same order, and no duplicates. Do not use only `graph["sources"].len()`.

- [ ] **Step 4: Reuse validation quarantine helper for ref errors**

Task 2 introduced `quarantine_prompt_pack_validation_error`. Use that same helper for synthesis ref-check failures instead of adding a second SQL insert path.

No new helper is needed in `validation.rs`.

- [ ] **Step 5: Wire validation into synthesis persistence**

In `outputs.rs`, import:

```rust
use super::entities::load_required_allowed_refs_for_live_synthesis;
use crate::prompt_packs::validation::{
    quarantine_prompt_pack_validation_error, validate_synthesis_output_with_allowed_refs,
};
```

In `execute_synthesis_stage_with_completion`, keep the current schema/quarantine validation first:

```rust
if let Err(error) =
    validate_and_quarantine_synthesis_output(pool, run_id, stage_run_id, &parsed).await
{
    mark_synthesis_stage_failed(pool, stage_run_id, &error.message).await?;
    return Err(error);
}
```

Immediately after that block, add ref validation:

```rust
let allowed_refs = match load_required_allowed_refs_for_live_synthesis(pool, run_id).await {
    Ok(allowed_refs) => allowed_refs,
    Err(error) => {
        mark_synthesis_stage_failed(pool, stage_run_id, &error.message).await?;
        return Err(error);
    }
};
if let Err(error) = validate_synthesis_output_with_allowed_refs(
    &parsed,
    &allowed_refs.source_refs,
    &allowed_refs.claim_refs,
    &allowed_refs.evidence_refs,
) {
    let validation_message = error.message.clone();
    quarantine_prompt_pack_validation_error(pool, run_id, stage_run_id, &parsed, error.clone())
        .await?;
    mark_synthesis_stage_failed(pool, stage_run_id, &error.message).await?;
    return Err(AppError::validation(validation_message));
}
```

- [ ] **Step 6: Run quarantine tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary::outputs_tests
```

Expected: PASS with one quarantine row for the unknown-ref case and a live execution error for missing complete graph.

- [ ] **Step 7: Commit**

```powershell
git add src-tauri\src\prompt_packs\youtube_summary\entities.rs src-tauri\src\prompt_packs\youtube_summary\outputs.rs src-tauri\src\prompt_packs\youtube_summary\outputs_tests.rs src-tauri\src\prompt_packs\validation.rs
git commit -m "feat: quarantine invalid youtube synthesis refs"
```

---

## Task 6: Prefer Graph Claims And Evidence In Canonical Result Builder

**Files:**
- Modify: `src-tauri/src/prompt_packs/result_builder.rs`

- [ ] **Step 1: Add failing result builder tests**

In `result_builder.rs` tests, add:

```rust
#[tokio::test]
async fn build_canonical_result_uses_intermediate_graph_claims_and_evidence() {
    let pool = isolated_result_builder_pool().await;
    insert_isolated_result_builder_run(&pool, 42, 2).await;
    insert_transcript_stage_with_parsed_output(&pool, 42, 1001, 501, "source_ref_1", "Legacy claim", "Legacy evidence")
        .await
        .expect("legacy output");
    insert_intermediate_entities_artifact(
        &pool,
        42,
        1001,
        501,
        "source_ref_1",
        "Graph claim",
        "Graph evidence",
        1,
    )
    .await
    .expect("graph artifact");

    let result = build_youtube_summary_canonical_result(&pool, 42)
        .await
        .expect("canonical result");

    assert_eq!(result["claims"][0]["text"], "Graph claim");
    assert_eq!(result["evidence"][0]["text"], "Graph evidence");
}

#[tokio::test]
async fn build_canonical_result_mixed_graph_availability_falls_back_with_quality_flag() {
    let pool = isolated_result_builder_pool().await;
    insert_isolated_result_builder_run(&pool, 42, 2).await;
    insert_transcript_stage_with_parsed_output(&pool, 42, 1001, 501, "source_ref_1", "Legacy first", "Evidence first")
        .await
        .expect("first output");
    insert_transcript_stage_with_parsed_output(&pool, 42, 1002, 502, "source_ref_2", "Legacy second", "Evidence second")
        .await
        .expect("second output");
    insert_intermediate_entities_artifact(
        &pool,
        42,
        1001,
        501,
        "source_ref_1",
        "Graph first",
        "Graph evidence first",
        1,
    )
    .await
    .expect("graph artifact");

    let result = build_youtube_summary_canonical_result(&pool, 42)
        .await
        .expect("canonical result");

    assert_eq!(result["claims"][0]["text"], "Legacy first");
    assert!(has_quality_flag(&result, "intermediate_entities_legacy_fallback"));
    assert!(
        result["limitations"]
            .as_array()
            .expect("limitations")
            .iter()
            .any(|value| value.as_str().unwrap_or("").contains("intermediate entity graph artifacts were incomplete"))
    );
}
```

Add test helper:

```rust
async fn insert_intermediate_entities_artifact(
    pool: &sqlx::SqlitePool,
    run_id: i64,
    source_snapshot_id: i64,
    stage_run_id: i64,
    source_ref_id: &str,
    claim: &str,
    evidence: &str,
    attempt_number: i64,
) -> AppResult<()> {
    let claim_ref = format!("{source_ref_id}_claim_1");
    let evidence_ref = format!("{source_ref_id}_evidence_1");
    let graph = serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "graph_kind": "youtube_summary_intermediate_entities",
        "run_id": run_id,
        "attempt_number": attempt_number,
        "sources": [{ "source_ref_id": source_ref_id, "source_snapshot_id": source_snapshot_id, "title": null }],
        "segments": [],
        "key_points": [],
        "quotes": [],
        "claims": [{ "claim_id": claim_ref.clone(), "source_ref_id": source_ref_id, "text": claim, "material_refs": [] }],
        "evidence": [{ "evidence_id": evidence_ref.clone(), "source_ref_id": source_ref_id, "text": evidence, "material_refs": [], "quote_ref": null }],
        "warnings": [],
        "allowed_refs": {
            "source_refs": [source_ref_id],
            "segment_refs": [],
            "key_point_refs": [],
            "quote_refs": [],
            "claim_refs": [claim_ref],
            "evidence_refs": [evidence_ref]
        }
    });
    crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "intermediate_entities",
        attempt_number,
        5,
        &graph.to_string(),
    )
    .await
}
```

- [ ] **Step 2: Run result builder tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs::result_builder::tests
```

Expected: FAIL because result builder ignores graph artifacts.

- [ ] **Step 3: Implement graph preference**

In `result_builder.rs`, add helper:

```rust
async fn load_complete_intermediate_graph_for_result(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<GraphLoadOutcome> {
    let expected_sources = sqlx::query_as::<_, (i64, String)>(
        "SELECT snapshots.id, snapshots.source_ref_id
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_run_source_snapshots snapshots
           ON snapshots.id = stages.source_snapshot_id
          AND snapshots.run_id = stages.run_id
         WHERE stages.run_id = ?
           AND stages.stage_name = 'youtube_summary/transcript_analysis'
           AND stages.stage_status = 'succeeded'
         ORDER BY snapshots.id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let graph_rows = load_intermediate_graph_rows(pool, run_id).await?;
    if graph_rows.is_empty() {
        return Ok(GraphLoadOutcome::NoGraph);
    }
    let graph_row_sources = graph_rows
        .iter()
        .map(|row| (row.source_snapshot_id, row.source_ref_id.clone()))
        .collect::<Vec<_>>();
    if graph_row_sources != expected_sources {
        return Ok(GraphLoadOutcome::PartialGraph);
    }
    let merged = merge_result_graph_rows(graph_rows)?;
    let merged_sources = graph_source_keys(&merged)?;
    if merged_sources != expected_sources {
        return Ok(GraphLoadOutcome::PartialGraph);
    }
    Ok(GraphLoadOutcome::Complete(merged))
}

enum GraphLoadOutcome {
    Complete(serde_json::Value),
    PartialGraph,
    NoGraph,
}

struct IntermediateGraphRow {
    source_snapshot_id: i64,
    source_ref_id: String,
    content_zstd: Vec<u8>,
}
```

Use the same artifact selection rule as synthesis input: latest `attempt_number`, `artifact_index`, `id`, ordered by `prompt_pack_run_source_snapshots.id ASC`.

Completeness is the same exact source set contract as live synthesis: successful transcript stage `(source_snapshot_id, source_ref_id)` values must exactly equal both the selected graph row source keys and the merged graph `sources[]` keys. Count-only checks are not sufficient.

Add `graph_source_keys(&serde_json::Value) -> AppResult<Vec<(i64, String)>>` to extract graph `sources[]` keys, reject missing `source_snapshot_id`/`source_ref_id`, and reject duplicate source keys before returning the vector.

In `build_youtube_summary_canonical_result`, place the graph logic immediately after `source_rows` are loaded and before the loop that builds `videos`, `claims`, and `evidence`.

Move base limitation/flag initialization up to that same area:

```rust
let mut limitations = build_base_limitations(pool, run_id).await?;
let mut quality_flags = build_base_quality_flags(pool, run_id).await?;
let graph_outcome = load_complete_intermediate_graph_for_result(pool, run_id).await?;
let use_graph_entities = matches!(graph_outcome, GraphLoadOutcome::Complete(_));
```

Remove the later duplicate `let mut limitations = ...` and `let mut quality_flags = ...` declarations near synthesis handling.

During the source loop, always build `source_refs` and `videos` from source snapshots and parsed output. Only append legacy `claims` and `evidence` inside the loop when `!use_graph_entities`.

After the source loop, when graph is complete, replace `claims` and `evidence` from graph arrays. When partial, keep legacy path and add the observable fallback immediately to the already-initialized vectors:

```rust
limitations.push(
    "Intermediate entity graph artifacts were incomplete, so claims and evidence were assembled through the legacy parsed-output path.".to_string(),
);
push_quality_flag(
    &mut quality_flags,
    "intermediate_entities_legacy_fallback",
    "warning",
);
```

- [ ] **Step 4: Run result builder tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs::result_builder
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri\src\prompt_packs\result_builder.rs
git commit -m "feat: use youtube summary intermediate graph in results"
```

---

## Task 7: End-To-End Verification And Documentation Update

**Files:**
- Modify: `docs/superpowers/plans/2026-06-16-youtube-summary-intermediate-entities.md`

- [ ] **Step 1: Run focused YouTube Summary tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
```

Expected: more than zero tests run, all pass.

- [ ] **Step 2: Run Prompt Pack tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs
```

Expected: more than zero tests run, all pass.

- [ ] **Step 3: Run compile and formatting checks**

Run:

```powershell
cargo fmt --manifest-path src-tauri\Cargo.toml --check
cargo check --manifest-path src-tauri\Cargo.toml
git diff --check
```

Expected: all commands exit 0. Existing unchanged dead-code warnings are acceptable only if they are the same warnings already present before this feature.

- [ ] **Step 4: Mark plan complete**

Edit this plan file and check off completed task steps. Do not mark a step complete until its command has been run and its output read.

- [ ] **Step 5: Commit final plan state**

```powershell
git add docs\superpowers\plans\2026-06-16-youtube-summary-intermediate-entities.md
git commit -m "docs: complete youtube summary intermediate entities plan"
```

## Self-Review

- Spec coverage: tasks cover source-scoped transcript-stage artifacts, latest attempt selection, complete allowed refs, no graph warnings in synthesis input, no direct segment/key point/quote refs in synthesis output, synthesis ref validation/quarantine, all-or-nothing result fallback, observable quality flag, and deterministic source snapshot ordering.
- Placeholder scan: this plan contains no placeholder tokens or unnamed implementation steps.
- Type consistency: plan uses `intermediate_entities`, `canonical_graph`, `allowed_refs`, `claim_id`, `evidence_id`, `claim_refs`, `evidence_refs`, `segment_ref`, `key_point_ref`, and `quote_ref` consistently with the approved spec.
