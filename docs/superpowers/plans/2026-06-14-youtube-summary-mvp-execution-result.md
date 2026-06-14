# YouTube Summary Prompt Pack Execution and Result Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute the MVP combined YouTube Summary stage and persist validated canonical Prompt Pack results with rebuildable projections.

**Architecture:** Keep execution behind the runtime boundary created in the previous plan. The combined stage renders one stage input per video, calls the existing LLM backend, validates parsed output against seeded schema identity plus closed-world refs, assembles canonical JSON, and persists canonical result/projections/status in one transaction.

**Tech Stack:** Rust/Tauri 2, existing `llm` module, `serde_json`, zstd compression, SQLite transactions via `sqlx`, Tauri events.

---

## Dependencies

Complete these plans first:

- `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-foundation.md`
- `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-runtime.md`

---

## File Structure

- Modify `src-tauri/src/prompt_packs/youtube_summary.rs`: wire combined stage execution into the runtime worker/start path.
- Create `src-tauri/src/prompt_packs/stage_io.rs`: stage input construction, output parsing, and JSON extraction.
- Create `src-tauri/src/prompt_packs/validation.rs`: schema identity checks and closed-world validation.
- Create `src-tauri/src/prompt_packs/result_builder.rs`: canonical result assembly and deterministic ids.
- Create `src-tauri/src/prompt_packs/projections.rs`: projection rebuild and repair helpers.
- Modify `src-tauri/src/prompt_packs/store.rs`: stage artifact inserts, result transaction, projection queries.
- Modify `src-tauri/src/prompt_packs/runtime.rs`: terminal event emission after transaction commit.
- Modify `src-tauri/src/prompt_packs/dto.rs`: result, validation finding, and artifact DTOs.
- Modify `src-tauri/src/prompt_packs/mod.rs`: expose result/artifact/validation commands.
- Modify `src-tauri/src/lib.rs`: register result commands.
- Modify `src/lib/types/prompt-packs.ts`: result viewer types.
- Modify `src/lib/api/prompt-packs.ts`: result/artifact/validation wrappers.

---

## Task 1: Stage Input Builder and Artifact Storage

**Files:**
- Create: `src-tauri/src/prompt_packs/stage_io.rs`
- Modify: `src-tauri/src/prompt_packs/store.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`

- [x] **Step 1: Write stage input tests**

Add tests:

```rust
#[tokio::test]
async fn build_transcript_analysis_stage_input_uses_frozen_registries() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage = load_transcript_analysis_stage_for_source(&pool, 42, "source_ref_1")
        .await
        .expect("stage");

    let input = build_transcript_analysis_stage_input(&pool, stage.id)
        .await
        .expect("input");

    assert_eq!(input.stage_io_version, "1.0");
    assert_eq!(input.stage, "youtube_summary/transcript_analysis");
    assert_eq!(input.pack_id, "youtube_summary");
    assert_eq!(input.source_ref_id, "source_ref_1");
    assert!(input.allowed_material_refs.iter().all(|value| value.starts_with("m_")));
    assert!(input.transcript_segment_registry.len() > 0);
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::stage_io::tests::build_transcript_analysis_stage_input_uses_frozen_registries
```

Expected: fail because `stage_io.rs` does not exist.

- [x] **Step 3: Implement stage input builder**

Build input from run-local tables only:

- `prompt_pack_runs`;
- `prompt_pack_run_source_snapshots`;
- `prompt_pack_run_material_snapshots`;
- `prompt_pack_stage_runs`;
- seeded `prompt_pack_stage_templates`.

The input must include `comment_selection_policy` even when `comment_material_refs` is empty.

- [x] **Step 4: Implement artifact insert helper**

Add `insert_stage_artifact_in_tx` for:

- `artifact_kind = 'prompt_input'`;
- `artifact_kind = 'raw_output'`;
- `artifact_kind = 'parsed_output'`;
- `artifact_kind = 'metrics'`;
- `artifact_kind = 'error'`;
- `artifact_kind = 'repair_input'` only when a repair loop is implemented.

Store content hash, zstd JSON/text, token counts when available, redaction state, and attempt number.

- [x] **Step 5: Run tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::stage_io
```

Expected: pass.

- [x] **Step 6: Commit**

```powershell
git add src-tauri/src/prompt_packs/stage_io.rs src-tauri/src/prompt_packs/store.rs src-tauri/src/prompt_packs/youtube_summary.rs
git commit -m "feat: build youtube summary stage inputs"
```

---

## Task 2: Stage Output Parser and Validator

**Files:**
- Create: `src-tauri/src/prompt_packs/validation.rs`
- Modify: `src-tauri/src/prompt_packs/stage_io.rs`
- Modify: `src-tauri/src/prompt_packs/store.rs`

- [ ] **Step 1: Write validator tests**

Add tests:

```rust
#[test]
fn transcript_analysis_output_rejects_unknown_material_ref() {
    let input = test_stage_input_with_material_refs(["m_transcript_1"]);
    let output = serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/transcript_analysis",
        "video_candidate": {
            "summary_text": "Summary",
            "segment_candidates": [],
            "key_point_candidates": [],
            "quote_candidates": [],
            "action_item_candidates": [],
            "open_question_candidates": []
        },
        "claim_candidates": [
            {
                "text": "Claim",
                "material_refs": ["m_missing"]
            }
        ],
        "evidence_fragment_candidates": [],
        "warning_candidates": []
    });

    let error = validate_transcript_analysis_output(&input, &output)
        .expect_err("unknown material ref rejected");

    assert!(error.message.contains("m_missing"));
}

#[test]
fn transcript_analysis_output_rejects_llm_assigned_final_ids() {
    let input = test_stage_input_with_material_refs(["m_transcript_1"]);
    let output = test_output_with_claim_id("claim_1");

    let error = validate_transcript_analysis_output(&input, &output)
        .expect_err("final ids rejected");

    assert!(error.message.contains("claim_id"));
}

#[test]
fn extract_json_payload_accepts_fenced_json_object() {
    let text = "```json\n{\"stage_io_version\":\"1.0\",\"value\":1}\n```";

    let value = extract_json_payload(text).expect("json payload");

    assert_eq!(value["stage_io_version"], "1.0");
    assert_eq!(value["value"], 1);
}

#[test]
fn extract_json_payload_accepts_leading_and_trailing_prose() {
    let text = "Here is the result:\n{\"stage_io_version\":\"1.0\",\"value\":1}\nDone.";

    let value = extract_json_payload(text).expect("json payload");

    assert_eq!(value["stage_io_version"], "1.0");
    assert_eq!(value["value"], 1);
}

#[test]
fn extract_json_payload_rejects_malformed_braces() {
    let error = extract_json_payload("{\"stage_io_version\":\"1.0\"")
        .expect_err("malformed JSON rejected");

    assert!(error.message.contains("malformed"));
}

#[test]
fn extract_json_payload_rejects_multiple_json_objects() {
    let error = extract_json_payload("{\"a\":1}\n{\"b\":2}")
        .expect_err("ambiguous JSON rejected");

    assert!(error.message.contains("multiple JSON objects"));
}

#[tokio::test]
async fn invalid_candidate_is_written_to_quarantine_artifacts() {
    let pool = test_pool_with_transcript_analysis_stage().await;
    let input = test_stage_input_with_material_refs(["m_transcript_1"]);
    let output = serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/transcript_analysis",
        "video_candidate": {
            "summary_text": "Summary",
            "segment_candidates": [],
            "key_point_candidates": [],
            "quote_candidates": [],
            "action_item_candidates": [],
            "open_question_candidates": []
        },
        "claim_candidates": [
            {
                "text": "Claim",
                "material_refs": ["m_missing"]
            }
        ],
        "evidence_fragment_candidates": [],
        "warning_candidates": []
    });

    validate_and_quarantine_transcript_analysis_output(&pool, 42, 1001, &input, &output)
        .await
        .expect_err("invalid candidate rejected");

    let quarantine_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts \
         WHERE run_id = 42 AND stage_run_id = 1001 AND object_path = '$.claim_candidates[0]'",
    )
    .fetch_one(&pool)
    .await
    .expect("quarantine count");

    assert_eq!(quarantine_count, 1);
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::validation
```

Expected: fail because validator does not exist.

- [ ] **Step 3: Implement parser**

Implement `extract_json_payload` for provider text. Reuse the brace-balanced approach from `analysis/report.rs`, but keep the function local to `prompt_packs/stage_io.rs` or extract a shared helper only if both call sites are updated in the same commit.

Parser rules:

- strip a single surrounding markdown JSON fence before parsing;
- accept one JSON object with leading/trailing prose;
- reject malformed brace balance with a validation error containing `malformed`;
- reject multiple top-level JSON objects with a validation error containing `multiple JSON objects`;
- reject responses without a JSON object.

- [ ] **Step 4: Implement validation**

Validation layers:

- schema identity: `stage_io_version = "1.0"`, `schema_version = "1.0"`, `stage = "youtube_summary/transcript_analysis"`;
- required top-level keys from `stage-io/youtube_summary_transcript_analysis_output`;
- closed-world source refs against `allowed_source_ref_ids`;
- closed-world material refs against `allowed_material_refs`;
- reject LLM-provided final ids: `claim_id`, `evidence_id`, `source_ref_id`, `segment_id`, `key_point_id`, `quote_id`, `action_item_id`, `open_question_id`;
- quarantine invalid candidate objects into `prompt_pack_result_quarantine_artifacts` instead of accepting partial object fragments silently;
- write validation findings into `prompt_pack_result_validation_findings`, not as a stage artifact kind.

- [ ] **Step 5: Run validator tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::validation
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::stage_io
```

Expected: pass.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/prompt_packs/validation.rs src-tauri/src/prompt_packs/stage_io.rs src-tauri/src/prompt_packs/store.rs
git commit -m "feat: validate youtube summary stage output"
```

---

## Task 3: Combined Stage Execution

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Modify: `src-tauri/src/prompt_packs/store.rs`

- [ ] **Step 1: Write fake-provider execution test**

Add a fake execution path behind a test-only function:

```rust
#[tokio::test]
async fn execute_transcript_analysis_stage_persists_raw_and_parsed_artifacts() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 42).await;

    execute_transcript_analysis_stage_with_completion(
        &pool,
        stage_id,
        fake_completion_with_valid_transcript_analysis_json(),
    )
    .await
    .expect("execute stage");

    let artifact_kinds = list_stage_artifact_kinds(&pool, stage_id).await;
    assert_eq!(
        artifact_kinds,
        vec!["prompt_input", "raw_output", "parsed_output", "metrics"],
    );
}

#[tokio::test]
async fn execute_multi_video_run_with_one_provider_failure_finishes_partial() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;

    execute_youtube_summary_run_with_fake_completions(
        &pool,
        42,
        vec![
            Ok(fake_completion_with_valid_transcript_analysis_json_for_source("source_ref_1")),
            Err(fake_provider_failure("provider timeout for source_ref_2")),
        ],
    )
    .await
    .expect("execute partial run");

    let run_status: String =
        sqlx::query_scalar("SELECT run_status FROM prompt_pack_runs WHERE id = 42")
            .fetch_one(&pool)
            .await
            .expect("run status");
    let result_status: String =
        sqlx::query_scalar("SELECT result_status FROM prompt_pack_results WHERE run_id = 42")
            .fetch_one(&pool)
            .await
            .expect("result status");
    let error_artifacts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_stage_artifacts \
         WHERE run_id = 42 AND artifact_kind = 'error'",
    )
    .fetch_one(&pool)
    .await
    .expect("error artifacts");
    let warning_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_warnings WHERE run_id = 42",
    )
    .fetch_one(&pool)
    .await
    .expect("warning count");
    let quality_flag_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_quality_flags WHERE run_id = 42",
    )
    .fetch_one(&pool)
    .await
    .expect("quality flags");

    assert_eq!(run_status, "partial");
    assert_eq!(result_status, "partial");
    assert_eq!(error_artifacts, 1);
    assert!(warning_count > 0);
    assert!(quality_flag_count > 0);
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::youtube_summary::tests::execute_transcript_analysis_stage_persists_raw_and_parsed_artifacts
```

Expected: fail because execution helper does not exist.

- [ ] **Step 3: Implement testable execution helper**

Create a helper that accepts an already-collected `LlmCompletion` for tests and a production path that calls:

```rust
crate::llm::run_llm_collect_with_profile(...)
```

Store:

- `prompt_input` artifact before provider call;
- `raw_output` artifact after provider call;
- parsed output artifact after JSON extraction;
- `metrics` artifact after validator completes, containing token counts, schema id, validation summary counts, attempt number, and provider latency.

- [ ] **Step 4: Implement stage status transitions**

Rules:

- `pending -> running -> succeeded` for valid output;
- provider failure marks that video stage `failed` and stores an `error` artifact with a sanitized provider error;
- if at least one video succeeds and at least one fails, final run must become `partial`;
- if all videos fail, final run becomes `failed`;
- cancel request stops launching new video stages and marks active/running work as `cancelled`.

- [ ] **Step 5: Run execution tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::youtube_summary
```

Expected: pass.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary.rs src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/store.rs
git commit -m "feat: execute youtube summary combined stage"
```

---

## Task 4: Canonical Result Builder

**Files:**
- Create: `src-tauri/src/prompt_packs/result_builder.rs`
- Modify: `src-tauri/src/prompt_packs/store.rs`

- [ ] **Step 1: Write result builder tests**

Add tests:

```rust
#[tokio::test]
async fn build_canonical_result_assigns_backend_owned_ids() {
    let pool = test_pool_with_successful_stage_artifacts().await;

    let result = build_youtube_summary_canonical_result(&pool, 42)
        .await
        .expect("canonical result");

    assert_eq!(result["pack_id"], "youtube_summary");
    assert_eq!(result["run_id"], 42);
    assert_eq!(result["source_refs"][0]["source_ref_id"], "source_ref_1");
    assert_eq!(result["claims"][0]["claim_id"], "claim_1");
    assert_eq!(result["evidence"][0]["evidence_id"], "evidence_1");
    assert_eq!(
        result["outputs"]["pack_data"]["youtube_summary"]["videos"][0]["video_id"],
        "video_1",
    );
    assert_eq!(
        result["outputs"]["pack_data"]["youtube_summary"]["synthesis"],
        serde_json::Value::Null,
    );
    assert!(result.get("sources").is_none());
    assert!(result.get("pack_data").is_none());
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::result_builder
```

Expected: fail because result builder does not exist.

- [ ] **Step 3: Implement deterministic id assignment**

Rules:

- `result_id = "result_<run_id>"` or a stable generated string stored in DB;
- `source_refs` come from frozen snapshots, not LLM output;
- YouTube pack data is written under `outputs.pack_data.youtube_summary`;
- `claim_id`, `evidence_id`, nested video object ids are assigned in stable source/stage/candidate order;
- evidence can belong to exactly one claim;
- invalid candidate objects go to quarantine instead of canonical JSON;
- warnings and limitations include skipped playlist videos and failed stages.

- [ ] **Step 4: Validate canonical identity before persistence**

Before insert, assert canonical `run_id`, `pack_id`, `pack_version`, and `schema_version` match the owning `prompt_pack_runs` row and pack snapshot.

- [ ] **Step 5: Run result builder tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::result_builder
```

Expected: pass.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/prompt_packs/result_builder.rs src-tauri/src/prompt_packs/store.rs
git commit -m "feat: assemble youtube summary canonical result"
```

---

## Task 5: Transactional Result Persistence and Projection Repair

**Files:**
- Create: `src-tauri/src/prompt_packs/projections.rs`
- Modify: `src-tauri/src/prompt_packs/store.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Modify: `src-tauri/src/prompt_packs/mod.rs`

- [ ] **Step 1: Write persistence tests**

Add tests:

```rust
#[tokio::test]
async fn persist_final_result_sets_terminal_status_after_projection_rows_exist() {
    let pool = test_pool_with_canonical_result_ready().await;

    persist_final_result_transaction(&pool, 42, test_canonical_result(), "complete")
        .await
        .expect("persist result");

    let run_status: String =
        sqlx::query_scalar("SELECT run_status FROM prompt_pack_runs WHERE id = 42")
            .fetch_one(&pool)
            .await
            .expect("run status");
    let projected_videos: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_youtube_videos WHERE run_id = 42",
    )
    .fetch_one(&pool)
    .await
    .expect("projected videos");
    let result_status: String =
        sqlx::query_scalar("SELECT result_status FROM prompt_pack_results WHERE run_id = 42")
            .fetch_one(&pool)
            .await
            .expect("result status");

    assert_eq!(run_status, "complete");
    assert_eq!(result_status, "complete");
    assert!(projected_videos > 0);
}

#[tokio::test]
async fn repair_rebuilds_missing_projection_rows_from_canonical_json() {
    let pool = test_pool_with_terminal_result_and_deleted_projections().await;

    repair_prompt_pack_result_projections(&pool, 42)
        .await
        .expect("repair projections");

    let projected_claims: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_claims WHERE run_id = 42",
    )
    .fetch_one(&pool)
    .await
    .expect("projected claims");

    assert!(projected_claims > 0);
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::projections
```

Expected: fail because projections module does not exist.

- [ ] **Step 3: Implement projection rebuild**

Projection rebuild must delete existing rows for `result_row_id` and reinsert:

- source refs;
- claims;
- evidence;
- relations;
- unknowns;
- verification tasks;
- warnings;
- limitations;
- quality flags;
- audit refs;
- ref edges;
- YouTube videos, segments, key points, quotes, action items, open questions, synthesis items.

- [ ] **Step 4: Implement transaction**

`persist_final_result_transaction` must wrap:

1. insert/update `prompt_pack_results`;
2. delete projections for `result_row_id`;
3. rebuild projections;
4. set `projection_updated_at`;
5. update `prompt_pack_runs.run_status`, `result_status`, `completed_at`;
6. write terminal audit event.

Emit the terminal event only after commit.

- [ ] **Step 5: Implement repair-on-read**

`get_prompt_pack_result` and list/query helpers must call repair when:

- terminal result exists and `projection_updated_at IS NULL`;
- projection metadata is stale compared with `canonical_hash`;
- expected primary projection rows are missing.

If repair fails, return canonical JSON plus a storage warning and write `projection_repair_failed` audit event.

- [ ] **Step 6: Run projection tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::projections
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::store
```

Expected: pass.

- [ ] **Step 7: Commit**

```powershell
git add src-tauri/src/prompt_packs/projections.rs src-tauri/src/prompt_packs/store.rs src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/mod.rs
git commit -m "feat: persist prompt pack result projections"
```

---

## Task 6: Result Commands

**Files:**
- Modify: `src-tauri/src/prompt_packs/mod.rs`
- Modify: `src-tauri/src/prompt_packs/dto.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/api/prompt-packs.ts`
- Modify: `src/lib/api/prompt-packs.test.ts`
- Modify: `src/lib/types/prompt-packs.ts`

- [ ] **Step 1: Implement backend commands**

Expose:

```rust
#[tauri::command]
pub async fn get_prompt_pack_result(handle: AppHandle, run_id: i64) -> AppResult<PromptPackResultDto>

#[tauri::command]
pub async fn list_prompt_pack_stage_artifacts(handle: AppHandle, stage_run_id: i64) -> AppResult<Vec<PromptPackStageArtifactSummaryDto>>

#[tauri::command]
pub async fn get_prompt_pack_stage_artifact(
    handle: AppHandle,
    stage_run_id: i64,
    artifact_kind: String,
    attempt_number: i64,
    artifact_index: i64,
) -> AppResult<PromptPackStageArtifactDto>

#[tauri::command]
pub async fn get_prompt_pack_validation_findings(handle: AppHandle, run_id: i64) -> AppResult<Vec<PromptPackValidationFindingDto>>

#[tauri::command]
pub async fn list_prompt_pack_audit_events(handle: AppHandle, run_id: i64) -> AppResult<Vec<PromptPackAuditEventDto>>
```

Artifact access rules:

- `list_prompt_pack_stage_artifacts` returns summaries ordered by `attempt_number ASC, artifact_index ASC`;
- `get_prompt_pack_stage_artifact` requires exact `(stage_run_id, artifact_kind, attempt_number, artifact_index)`;
- if callers need the latest artifact, frontend code must call `list_prompt_pack_stage_artifacts` and choose the last matching summary explicitly.

- [ ] **Step 2: Register commands**

Add the commands to `tauri::generate_handler!` in `src-tauri/src/lib.rs`.

- [ ] **Step 3: Add frontend wrappers and tests**

Extend `src/lib/api/prompt-packs.ts` and `src/lib/api/prompt-packs.test.ts` for all result commands.

- [ ] **Step 4: Run command tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
npm test -- --run src/lib/api/prompt-packs.test.ts
```

Expected: pass.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/prompt_packs src-tauri/src/lib.rs src/lib/api/prompt-packs.ts src/lib/api/prompt-packs.test.ts src/lib/types/prompt-packs.ts
git commit -m "feat: expose prompt pack result commands"
```

---

## Plan Acceptance

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
npm test -- --run src/lib/api/prompt-packs.test.ts
git status --short
```

Expected:

- stage input and output validation tests pass;
- parser tests cover fenced JSON, prose-wrapped JSON, malformed braces, and multiple JSON objects;
- invalid candidate objects create `prompt_pack_result_quarantine_artifacts` rows;
- fake-provider execution path stores spec artifact kinds: `prompt_input`, `raw_output`, `parsed_output`, `metrics`, and `error`;
- partial multi-video execution persists `run_status = 'partial'` and `result_status = 'partial'`;
- canonical result builder creates deterministic IDs and uses `source_refs` plus `outputs.pack_data.youtube_summary`;
- final result transaction persists projections before terminal status `complete` is visible;
- repair rebuilds projections from canonical JSON;
- frontend result command wrappers call expected Tauri commands, including exact artifact fetch by `(artifact_kind, attempt_number, artifact_index)`.
