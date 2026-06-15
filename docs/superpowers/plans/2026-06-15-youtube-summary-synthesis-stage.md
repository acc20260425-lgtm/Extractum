# YouTube Summary Synthesis Stage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a real run-scoped `youtube_summary/synthesis` stage for multi-video YouTube Summary runs while keeping single-video canonical `synthesis = null`.

**Architecture:** Keep the first synthesis slice backend-first and version-safe. Reuse the existing `prompt_pack_stage_runs` skeleton row for `youtube_summary/synthesis`, gate execution to runs with at least two successful transcript-analysis videos, add runtime-only bundled config for the synthesis LLM budget, persist prompt/raw/parsed/metrics artifacts, and treat provider output as `synthesis_candidate` rather than canonical `synthesis`. The backend result builder assigns canonical synthesis IDs, computes traversal refs, writes canonical JSON, and projection rebuild writes `prompt_pack_youtube_synthesis_items`. Do not modify existing seeded `1.0.0` stage template assets in this slice, because changing seeded pack content can cause bundled hash conflicts for existing local databases.

**Tech Stack:** Rust, Tauri commands, SQLx/SQLite, serde JSON, existing Prompt Pack runtime tables, existing LLM scheduler/provider adapters.

---

## File Structure

- Modify `src-tauri/src/prompt_packs/youtube_summary.rs`: stage skeleton status, synthesis input/output execution helpers, artifact persistence, terminal status behavior.
- Modify `src-tauri/src/prompt_packs/runtime.rs`: LLM request construction for `youtube_summary/synthesis`, runtime config loading, scheduler execution path. Use the existing `LlmChatRequest.max_output_tokens` field; do not add a new LLM contract field in this plan.
- Modify `src-tauri/src/prompt_packs/result_builder.rs`: convert synthesis candidate output into canonical YouTube Summary `synthesis`, preserve/merge quality flags and limitations, and include readable sections.
- Modify `src-tauri/src/prompt_packs/projections.rs`: project canonical synthesis items into `prompt_pack_youtube_synthesis_items` during normal persist and repair rebuilds.
- Modify `src-tauri/src/prompt_packs/validation.rs`: add a dedicated synthesis stage-output validator and quarantine helper, mirroring the existing transcript-analysis validation boundary.
- Create `src-tauri/prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json`: runtime-only stage config.
- Modify `docs/prompt-packs/runtime_configuration_policy.md`: document the new runtime asset.
- Create or modify `docs/superpowers/verification/2026-06-15-youtube-summary-live-workflow.md`: add verification notes after implementation.

---

## Task 1: Add Runtime Config For Synthesis

**Files:**
- Create: `src-tauri/prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`

- [x] **Step 1: Write the failing test**

Add a unit test near `transcript_analysis_output_budget_comes_from_stage_runtime_config`:

```rust
#[test]
fn synthesis_output_budget_comes_from_stage_runtime_config() {
    assert_eq!(
        synthesis_stage_max_output_token_budget().expect("load synthesis budget"),
        6_144
    );
}
```

- [x] **Step 2: Run the test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_budget_comes_from_stage_runtime_config
```

Expected: FAIL because `synthesis_stage_max_output_token_budget` does not exist.

- [x] **Step 3: Add the runtime config asset**

Create `src-tauri/prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json`:

```json
{
  "runtime_configuration": {
    "runtime_config_version": "1.0",
    "model_routing": {
      "model_routing_version": "1.0",
      "default_provider_family": "generic_chat",
      "stage_routes": {
        "fragment_candidate_mining": "cheap_extractor",
        "claim_extraction": "reasoning",
        "claim_linking": "reasoning",
        "pack_data_generation": "pack_reasoning",
        "final_synthesis": "writer",
        "retry_repair": "repair",
        "youtube_summary/synthesis": "writer"
      }
    },
    "feature_flags": {
      "retry_enabled": true,
      "object_repair_enabled": true,
      "quarantine_enabled": true,
      "strict_reference_validation": true,
      "parser_fallback_enabled": false
    },
    "budget_limits": {
      "max_retry_attempts": 2,
      "stage_timeout_seconds": 120,
      "max_prompt_tokens": 24000,
      "max_output_tokens": 6144
    },
    "retry_policy": {
      "retryable_layers": ["schema", "reference"],
      "escalate_after_attempts": 1,
      "fallback_model_class": "strong_repair"
    },
    "quarantine_policy": {
      "store": "configured-at-runtime",
      "redact_raw_provider_output": false,
      "retention_days": 30
    },
    "telemetry": {
      "enabled": true,
      "sink": "local"
    }
  }
}
```

- [x] **Step 4: Implement the budget loader**

In `src-tauri/src/prompt_packs/runtime.rs`, add:

```rust
const SYNTHESIS_STAGE_JSON: &str =
    include_str!("../../prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json");

fn synthesis_stage_max_output_token_budget() -> AppResult<i64> {
    stage_max_output_token_budget(SYNTHESIS_STAGE_JSON, "synthesis")
}
```

Refactor the existing transcript loader to share:

```rust
fn transcript_analysis_stage_max_output_token_budget() -> AppResult<i64> {
    stage_max_output_token_budget(TRANSCRIPT_ANALYSIS_STAGE_JSON, "transcript-analysis")
}

fn stage_max_output_token_budget(asset_json: &str, label: &str) -> AppResult<i64> {
    let asset = serde_json::from_str::<StageRuntimeConfigAsset>(asset_json).map_err(|error| {
        AppError::internal(format!("Parse bundled {label} runtime configuration: {error}"))
    })?;
    asset
        .runtime_configuration
        .and_then(|runtime| runtime.budget_limits)
        .and_then(|budget| budget.max_output_tokens)
        .filter(|max_output_tokens| *max_output_tokens > 0)
        .ok_or_else(|| {
            AppError::internal(format!(
                "Bundled {label} runtime configuration is missing positive max_output_tokens"
            ))
        })
}
```

- [x] **Step 5: Run the test to verify it passes**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_budget_comes_from_stage_runtime_config
```

Expected: PASS, 1 test.

- [x] **Step 6: Commit**

```powershell
git add src-tauri/prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json src-tauri/src/prompt_packs/runtime.rs
git commit -m "Add YouTube Summary synthesis runtime config"
```

---

## Task 2: Build Synthesis Stage Input

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`

- [ ] **Step 1: Write the failing test**

Add these helper definitions near the existing transcript execution test helpers:

```rust
struct TranscriptStageFixture {
    summary: &'static str,
    claim: &'static str,
    evidence: &'static str,
}

fn transcript_analysis_json(summary: &str, claim: &str, evidence: &str) -> String {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/transcript_analysis",
        "video_candidate": {
            "summary_text": summary,
            "segment_candidates": [],
            "key_point_candidates": [],
            "quote_candidates": [],
            "action_item_candidates": [],
            "open_question_candidates": []
        },
        "claim_candidates": [
            {
                "text": claim
            }
        ],
        "evidence_fragment_candidates": [
            {
                "text": evidence
            }
        ],
        "warning_candidates": []
    })
    .to_string()
}

fn synthesis_json(summary: &str) -> String {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "synthesis_candidate": {
            "summary_text": summary,
            "cross_video_themes": [
                {
                    "theme_text": "Shared theme",
                    "source_refs": ["source_ref_1", "source_ref_2"],
                    "claim_refs": [],
                    "evidence_refs": []
                }
            ],
            "common_claims": [],
            "contradictions_across_videos": []
        },
        "limitations": [],
        "warning_candidates": []
    })
    .to_string()
}

async fn persist_succeeded_transcript_stage_fixtures(
    pool: &sqlx::SqlitePool,
    run_id: i64,
    fixtures: Vec<TranscriptStageFixture>,
) -> crate::error::AppResult<()> {
    let stage_rows = sqlx::query_as::<_, (i64, i64)>(
        "SELECT id, source_snapshot_id
         FROM prompt_pack_stage_runs
         WHERE run_id = ?
           AND stage_name = 'youtube_summary/transcript_analysis'
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(crate::error::AppError::database)?;

    assert_eq!(stage_rows.len(), fixtures.len());

    for ((stage_run_id, _source_snapshot_id), fixture) in stage_rows.into_iter().zip(fixtures) {
        sqlx::query(
            "UPDATE prompt_pack_stage_runs
             SET stage_status = 'succeeded', updated_at = ?
             WHERE id = ?",
        )
        .bind(super::now_string())
        .bind(stage_run_id)
        .execute(pool)
        .await
        .map_err(crate::error::AppError::database)?;

        let parsed = transcript_analysis_json(fixture.summary, fixture.claim, fixture.evidence);
        crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
            pool,
            run_id,
            stage_run_id,
            "parsed_output",
            1,
            3,
            &parsed,
        )
        .await?;
    }

    Ok(())
}
```

Add the tests near the transcript execution tests:

```rust
#[tokio::test]
async fn build_synthesis_stage_input_collects_successful_transcript_outputs() {
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
    .expect("persist transcript fixtures");

    let input = super::build_synthesis_stage_input(&pool, 1)
        .await
        .expect("synthesis input");

    assert_eq!(input["stage"], "youtube_summary/synthesis");
    assert_eq!(input["videos"].as_array().expect("videos").len(), 2);
    assert_eq!(input["claim_candidates"].as_array().expect("claims").len(), 2);
    assert_eq!(
        input["evidence_fragment_candidates"]
            .as_array()
            .expect("evidence")
            .len(),
        2
    );
}
```

Add a second input-shape test so retries cannot duplicate candidates and pipeline-owned source metadata stays outside the provider-authored candidate object:

```rust
#[tokio::test]
async fn build_synthesis_stage_input_uses_latest_parsed_output_wrappers() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "Old first summary",
                claim: "Old first claim",
                evidence: "Old first evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("persist transcript fixtures");

    let first_stage_run_id: i64 = sqlx::query_scalar(
        "SELECT id FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/transcript_analysis'
         ORDER BY id ASC
         LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("first stage row");
    crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
        &pool,
        1,
        first_stage_run_id,
        "parsed_output",
        2,
        3,
        &transcript_analysis_json("New first summary", "New first claim", "New first evidence"),
    )
    .await
    .expect("insert retry parsed output");

    let input = super::build_synthesis_stage_input(&pool, 1)
        .await
        .expect("synthesis input");
    let claims = input["claim_candidates"].as_array().expect("claims");

    assert_eq!(claims.len(), 2);
    assert_eq!(claims[0]["source_ref_id"], "source_ref_1");
    assert_eq!(claims[0]["candidate"]["text"], "New first claim");
    assert!(claims[0]["candidate"].get("source_ref_id").is_none());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_synthesis_stage_input_collects_successful_transcript_outputs
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_synthesis_stage_input_uses_latest_parsed_output_wrappers
```

Expected: both tests FAIL because `build_synthesis_stage_input` does not exist.

- [ ] **Step 3: Implement the input builder**

The test helpers from Step 1 already mark existing transcript-analysis stage rows as `succeeded` and persist exactly one `parsed_output` artifact per stage without terminalizing the whole run. Now add the private input builder in `youtube_summary.rs`:

```rust
pub(crate) async fn build_synthesis_stage_input(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<serde_json::Value> {
    let rows = sqlx::query_as::<_, (i64, i64, String, Option<String>, Vec<u8>)>(
        "SELECT stages.id, snapshots.id, snapshots.source_ref_id, snapshots.title, artifacts.content_zstd
         FROM prompt_pack_run_source_snapshots snapshots
         JOIN prompt_pack_stage_runs stages
           ON stages.run_id = snapshots.run_id
          AND stages.source_snapshot_id = snapshots.id
          AND stages.stage_name = 'youtube_summary/transcript_analysis'
          AND stages.stage_status = 'succeeded'
         JOIN prompt_pack_stage_artifacts artifacts
           ON artifacts.stage_run_id = stages.id
          AND artifacts.artifact_kind = 'parsed_output'
          AND artifacts.id = (
              SELECT latest.id
              FROM prompt_pack_stage_artifacts latest
              WHERE latest.stage_run_id = stages.id
                AND latest.artifact_kind = 'parsed_output'
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

    let mut videos = Vec::new();
    let mut claim_candidates = Vec::new();
    let mut evidence_fragment_candidates = Vec::new();
    let mut warning_candidates = Vec::new();

    for (_stage_run_id, source_snapshot_id, source_ref_id, title, content_zstd) in rows {
        let text = crate::compression::decompress_text(&content_zstd).map_err(AppError::internal)?;
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|error| AppError::internal(format!("parse transcript parsed_output: {error}")))?;
        videos.push(serde_json::json!({
            "source_snapshot_id": source_snapshot_id,
            "source_ref_id": source_ref_id,
            "title": title,
            "video_candidate": parsed.get("video_candidate").cloned().unwrap_or_else(|| serde_json::json!({}))
        }));
        wrap_candidates(&mut claim_candidates, parsed.get("claim_candidates"), &source_ref_id);
        wrap_candidates(
            &mut evidence_fragment_candidates,
            parsed.get("evidence_fragment_candidates"),
            &source_ref_id,
        );
        wrap_candidates(&mut warning_candidates, parsed.get("warning_candidates"), &source_ref_id);
    }

    Ok(serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "run_id": run_id,
        "videos": videos,
        "claim_candidates": claim_candidates,
        "evidence_fragment_candidates": evidence_fragment_candidates,
        "warning_candidates": warning_candidates
    }))
}
```

Add:

```rust
fn wrap_candidates(target: &mut Vec<serde_json::Value>, value: Option<&serde_json::Value>, source_ref_id: &str) {
    if let Some(items) = value.and_then(serde_json::Value::as_array) {
        for item in items {
            target.push(serde_json::json!({
                "source_ref_id": source_ref_id,
                "candidate": item
            }));
        }
    }
}
```

Do not write `source_ref_id` into the LLM-authored candidate object itself. The wrapper is pipeline-owned metadata; the nested `candidate` remains the provider output exactly as validated and persisted.

- [ ] **Step 4: Run the test to verify it passes**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_synthesis_stage_input_collects_successful_transcript_outputs
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_synthesis_stage_input_uses_latest_parsed_output_wrappers
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary.rs
git commit -m "Build YouTube Summary synthesis stage input"
```

---

## Task 3: Execute And Persist Synthesis Stage

**Files:**
- Modify: `src-tauri/src/prompt_packs/validation.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`

- [ ] **Step 1: Write the failing synthesis validation tests**

In `src-tauri/src/prompt_packs/validation.rs`, add tests beside the transcript-analysis validation tests:

```rust
#[test]
fn synthesis_output_validator_accepts_valid_output() {
    let output = valid_synthesis_output();

    validate_synthesis_output(&output, &allowed_synthesis_source_refs())
        .expect("valid synthesis output");
}

#[test]
fn synthesis_output_validator_rejects_missing_summary_text() {
    let mut output = valid_synthesis_output();
    output["synthesis_candidate"]
        .as_object_mut()
        .expect("candidate")
        .remove("summary_text");

    let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
        .expect_err("missing summary rejected");

    assert!(error.message.contains("summary_text"));
    assert_eq!(
        error.object_path.as_deref(),
        Some("$.synthesis_candidate.summary_text")
    );
}

#[test]
fn synthesis_output_validator_rejects_wrong_stage_io_version() {
    let mut output = valid_synthesis_output();
    output["stage_io_version"] = serde_json::json!("2.0");

    let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
        .expect_err("stage io version rejected");

    assert!(error.message.contains("stage_io_version"));
    assert_eq!(error.object_path.as_deref(), Some("$.stage_io_version"));
}

#[test]
fn synthesis_output_validator_rejects_wrong_schema_version() {
    let mut output = valid_synthesis_output();
    output["schema_version"] = serde_json::json!("2.0");

    let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
        .expect_err("schema version rejected");

    assert!(error.message.contains("schema_version"));
    assert_eq!(error.object_path.as_deref(), Some("$.schema_version"));
}

#[test]
fn synthesis_output_validator_rejects_wrong_stage() {
    let mut output = valid_synthesis_output();
    output["stage"] = serde_json::json!("youtube_summary/transcript_analysis");

    let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
        .expect_err("wrong stage rejected");

    assert!(error.message.contains("stage"));
    assert_eq!(error.object_path.as_deref(), Some("$.stage"));
}

#[test]
fn synthesis_output_validator_rejects_non_array_fields() {
    let mut output = valid_synthesis_output();
    output["synthesis_candidate"]["cross_video_themes"] = serde_json::json!("not an array");

    let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
        .expect_err("array contract rejected");

    assert!(error.message.contains("cross_video_themes"));
    assert_eq!(
        error.object_path.as_deref(),
        Some("$.synthesis_candidate.cross_video_themes")
    );
}

#[test]
fn synthesis_output_validator_rejects_backend_owned_ids() {
    let mut output = valid_synthesis_output();
    output["synthesis_candidate"]["cross_video_themes"] = serde_json::json!([
        {
            "theme_id": "theme_1",
            "text": "Provider must not assign final IDs"
        }
    ]);

    let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
        .expect_err("backend ids rejected");

    assert!(error.message.contains("theme_id"));
}

#[test]
fn synthesis_output_validator_rejects_unknown_source_ref() {
    let mut output = valid_synthesis_output();
    output["synthesis_candidate"]["cross_video_themes"][0]["source_refs"] =
        serde_json::json!(["source_ref_999"]);

    let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
        .expect_err("unknown source ref rejected");

    assert!(error.message.contains("source_ref_999"));
    assert_eq!(
        error.object_path.as_deref(),
        Some("$.synthesis_candidate.cross_video_themes[0].source_refs[0]")
    );
}

#[test]
fn synthesis_output_validator_rejects_provider_authored_claim_ref() {
    let mut output = valid_synthesis_output();
    output["synthesis_candidate"]["cross_video_themes"][0]["claim_refs"] =
        serde_json::json!(["claim_999"]);

    let error = validate_synthesis_output(&output, &allowed_synthesis_source_refs())
        .expect_err("provider-authored claim ref rejected");

    assert!(error.message.contains("claim_refs"));
    assert_eq!(
        error.object_path.as_deref(),
        Some("$.synthesis_candidate.cross_video_themes[0].claim_refs")
    );
}

#[tokio::test]
async fn invalid_synthesis_output_is_written_to_quarantine_artifacts() {
    let pool = test_pool_with_synthesis_stage().await;
    let mut output = valid_synthesis_output();
    output["warning_candidates"] = serde_json::json!([
        {
            "source_ref_id": "source_ref_1",
            "text": "Provider must not assign backend source refs"
        }
    ]);

    validate_and_quarantine_synthesis_output(&pool, 42, 2001, &output)
        .await
        .expect_err("invalid synthesis rejected");

    let quarantine_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts
         WHERE run_id = 42 AND stage_run_id = 2001",
    )
    .fetch_one(&pool)
    .await
    .expect("quarantine count");

    assert_eq!(quarantine_count, 1);
}

#[tokio::test]
async fn invalid_synthesis_output_with_unknown_source_ref_is_quarantined() {
    let pool = test_pool_with_synthesis_stage().await;
    let mut output = valid_synthesis_output();
    output["synthesis_candidate"]["cross_video_themes"][0]["source_refs"] =
        serde_json::json!(["source_ref_999"]);

    validate_and_quarantine_synthesis_output(&pool, 42, 2001, &output)
        .await
        .expect_err("unknown source ref rejected");

    let quarantine_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts
         WHERE run_id = 42
           AND stage_run_id = 2001
           AND reason LIKE '%source_ref_999%'",
    )
    .fetch_one(&pool)
    .await
    .expect("quarantine count");

    assert_eq!(quarantine_count, 1);
}

#[tokio::test]
async fn invalid_synthesis_output_surfaces_quarantine_write_failure() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite without migrations");
    let mut output = valid_synthesis_output();
    output["synthesis_candidate"]["cross_video_themes"] = serde_json::json!([
        {
            "theme_id": "theme_1",
            "theme_text": "Provider must not assign backend IDs"
        }
    ]);

    let error = validate_and_quarantine_synthesis_output(&pool, 42, 2001, &output)
        .await
        .expect_err("quarantine write failure is surfaced");

    assert!(error.message.contains("quarantine synthesis output"));
}

fn valid_synthesis_output() -> serde_json::Value {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "synthesis_candidate": {
            "summary_text": "Combined summary",
            "cross_video_themes": [
                {
                    "theme_text": "Shared theme",
                    "source_refs": ["source_ref_1", "source_ref_2"],
                    "claim_refs": [],
                    "evidence_refs": []
                }
            ],
            "common_claims": [],
            "contradictions_across_videos": []
        },
        "limitations": [],
        "warning_candidates": []
    })
}

fn allowed_synthesis_source_refs() -> std::collections::HashSet<String> {
    ["source_ref_1", "source_ref_2"]
        .into_iter()
        .map(ToString::to_string)
        .collect()
}

async fn test_pool_with_synthesis_stage() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");
    seed_builtin_prompt_packs_in_pool(&pool).await.expect("seed");
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
    sqlx::query(
        "INSERT INTO sources (
            id, source_type, source_subtype, external_id, title,
            is_active, is_member, created_at
         )
         VALUES
            (901, 'youtube', 'video', 'provider-video-1', 'Video 1', 1, 0, 1),
            (902, 'youtube', 'video', 'provider-video-2', 'Video 2', 1, 0, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert sources");
    sqlx::query(
        "INSERT INTO prompt_pack_run_source_snapshots (
            id, run_id, source_id, source_ref_id, video_id, title, created_at
         )
         VALUES
            (501, 42, 901, 'source_ref_1', 'provider-video-1', 'Video 1', '2026-06-14T00:00:00Z'),
            (502, 42, 902, 'source_ref_2', 'provider-video-2', 'Video 2', '2026-06-14T00:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("insert snapshots");
    sqlx::query(
        "INSERT INTO prompt_pack_stage_runs (
            id, run_id, source_snapshot_id, stage_name, stage_order, stage_status,
            created_at, updated_at
         )
         VALUES (2001, 42, NULL, 'youtube_summary/synthesis', 103, 'running',
            '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("insert synthesis stage");
    pool
}
```

- [ ] **Step 2: Run the validation tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_validator
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_validator_rejects_unknown_source_ref
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_validator_rejects_provider_authored_claim_ref
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_is_written_to_quarantine_artifacts
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_with_unknown_source_ref_is_quarantined
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_surfaces_quarantine_write_failure
```

Expected:

- `synthesis_output_validator` FAILS because `validate_synthesis_output` does not exist;
- `synthesis_output_validator_rejects_unknown_source_ref` FAILS because unknown synthesis source refs are not checked against run-owned snapshots;
- `synthesis_output_validator_rejects_provider_authored_claim_ref` FAILS because provider-authored claim/evidence/relation refs are not rejected;
- `invalid_synthesis_output_is_written_to_quarantine_artifacts` FAILS because `validate_and_quarantine_synthesis_output` does not exist.
- `invalid_synthesis_output_with_unknown_source_ref_is_quarantined` FAILS because unknown source refs are not quarantined.
- `invalid_synthesis_output_surfaces_quarantine_write_failure` FAILS because `validate_and_quarantine_synthesis_output` does not exist and must not swallow quarantine write errors.

- [ ] **Step 3: Implement synthesis validation and quarantine**

In `src-tauri/src/prompt_packs/validation.rs`, add:

```rust
pub(crate) fn validate_synthesis_output(
    output: &serde_json::Value,
    allowed_source_refs: &std::collections::HashSet<String>,
) -> Result<(), PromptPackValidationError> {
    expect_string(output, "stage_io_version", "1.0")?;
    expect_string(output, "schema_version", "1.0")?;
    expect_string(output, "stage", "youtube_summary/synthesis")?;
    let candidate = output
        .get("synthesis_candidate")
        .ok_or_else(|| PromptPackValidationError {
            message: "missing required key synthesis_candidate".to_string(),
            object_path: Some("$.synthesis_candidate".to_string()),
        })?;
    expect_non_empty_string_at(candidate, "summary_text", "$.synthesis_candidate.summary_text")?;
    for key in [
        "cross_video_themes",
        "common_claims",
        "contradictions_across_videos",
    ] {
        expect_array_at(
            candidate,
            key,
            &format!("$.synthesis_candidate.{key}"),
        )?;
    }
    for key in ["limitations", "warning_candidates"] {
        expect_array(output, key)?;
    }
    reject_non_empty_synthesis_candidate_ref_arrays(candidate, "$.synthesis_candidate")?;
    reject_backend_owned_ids(output, "$")?;
    reject_unknown_synthesis_source_refs(output, "$", allowed_source_refs)?;
    Ok(())
}

pub(crate) async fn validate_and_quarantine_synthesis_output(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    output: &serde_json::Value,
) -> AppResult<()> {
    let allowed_source_refs = load_allowed_synthesis_source_refs(pool, run_id).await?;
    match validate_synthesis_output(output, &allowed_source_refs) {
        Ok(()) => Ok(()),
        Err(error) => {
            let object_path = error
                .object_path
                .clone()
                .unwrap_or_else(|| "$".to_string());
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
            .bind("2026-06-14T00:00:00Z")
            .execute(pool)
            .await
            .map_err(|db_error| {
                AppError::internal(format!(
                    "quarantine synthesis output after validation error `{validation_message}` failed: {db_error}"
                ))
            })?;
            Err(AppError::validation(validation_message))
        }
    }
}

async fn load_allowed_synthesis_source_refs(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<std::collections::HashSet<String>> {
    sqlx::query_scalar::<_, String>(
        "SELECT source_ref_id FROM prompt_pack_run_source_snapshots
         WHERE run_id = ?
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map(|refs| refs.into_iter().collect())
    .map_err(AppError::database)
}

fn expect_non_empty_string(
    output: &serde_json::Value,
    key: &str,
) -> Result<(), PromptPackValidationError> {
    expect_non_empty_string_at(output, key, &format!("$.{key}"))
}

fn expect_non_empty_string_at(
    output: &serde_json::Value,
    key: &str,
    object_path: &str,
) -> Result<(), PromptPackValidationError> {
    if output
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
    {
        Ok(())
    } else {
        Err(PromptPackValidationError {
            message: format!("{key} must be a non-empty string"),
            object_path: Some(object_path.to_string()),
        })
    }
}

fn expect_array(
    output: &serde_json::Value,
    key: &str,
) -> Result<(), PromptPackValidationError> {
    expect_array_at(output, key, &format!("$.{key}"))
}

fn expect_array_at(
    output: &serde_json::Value,
    key: &str,
    object_path: &str,
) -> Result<(), PromptPackValidationError> {
    if output.get(key).and_then(serde_json::Value::as_array).is_some() {
        Ok(())
    } else {
        Err(PromptPackValidationError {
            message: format!("{key} must be an array"),
            object_path: Some(object_path.to_string()),
        })
    }
}

fn reject_unknown_synthesis_source_refs(
    value: &serde_json::Value,
    path: &str,
    allowed_source_refs: &std::collections::HashSet<String>,
) -> Result<(), PromptPackValidationError> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                let child_path = format!("{path}.{key}");
                if key == "source_refs" {
                    let refs =
                        nested
                            .as_array()
                            .ok_or_else(|| PromptPackValidationError {
                                message: "source_refs must be an array".to_string(),
                                object_path: Some(child_path.clone()),
                            })?;
                    for (index, item) in refs.iter().enumerate() {
                        let item_path = format!("{child_path}[{index}]");
                        let source_ref = item.as_str().ok_or_else(|| {
                            PromptPackValidationError {
                                message: "source_refs entries must be strings".to_string(),
                                object_path: Some(item_path.clone()),
                            }
                        })?;
                        if !allowed_source_refs.contains(source_ref) {
                            return Err(PromptPackValidationError {
                                message: format!(
                                    "unknown synthesis source ref {source_ref}"
                                ),
                                object_path: Some(item_path),
                            });
                        }
                    }
                } else {
                    reject_unknown_synthesis_source_refs(
                        nested,
                        &child_path,
                        allowed_source_refs,
                    )?;
                }
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for (index, nested) in items.iter().enumerate() {
                reject_unknown_synthesis_source_refs(
                    nested,
                    &format!("{path}[{index}]"),
                    allowed_source_refs,
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn reject_non_empty_synthesis_candidate_ref_arrays(
    value: &serde_json::Value,
    path: &str,
) -> Result<(), PromptPackValidationError> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                let child_path = format!("{path}.{key}");
                if ["claim_refs", "evidence_refs", "relation_refs"].contains(&key.as_str()) {
                    let refs =
                        nested
                            .as_array()
                            .ok_or_else(|| PromptPackValidationError {
                                message: format!("{key} must be an array"),
                                object_path: Some(child_path.clone()),
                            })?;
                    if !refs.is_empty() {
                        return Err(PromptPackValidationError {
                            message: format!(
                                "{key} must be empty in synthesis_candidate until the backend exposes an allowed ref map"
                            ),
                            object_path: Some(child_path),
                        });
                    }
                } else {
                    reject_non_empty_synthesis_candidate_ref_arrays(nested, &child_path)?;
                }
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for (index, nested) in items.iter().enumerate() {
                reject_non_empty_synthesis_candidate_ref_arrays(
                    nested,
                    &format!("{path}[{index}]"),
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
```

Extend the existing `reject_backend_owned_ids` forbidden list to include every backend-owned ID used by transcript and synthesis canonical builders:

```rust
const FORBIDDEN: &[&str] = &[
    "claim_id",
    "evidence_id",
    "source_ref_id",
    "segment_id",
    "key_point_id",
    "quote_id",
    "action_item_id",
    "open_question_id",
    "section_id",
    "video_id",
    "synthesis_item_id",
    "theme_id",
    "common_claim_id",
    "contradiction_id",
];
```

Add `use crate::error::{AppError, AppResult};` near the top of `validation.rs` if those names are not already imported.

- [ ] **Step 4: Write the failing persistence test**

Add a test:

```rust
#[tokio::test]
async fn execute_synthesis_stage_persists_raw_parsed_and_metrics_artifacts() {
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
    .expect("persist transcript fixtures");

    let stage_run_id: i64 = sqlx::query_scalar(
        "SELECT id FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
    )
    .fetch_one(&pool)
    .await
    .expect("stage row");

    super::execute_synthesis_stage_with_completion(
        &pool,
        stage_run_id,
        LlmCompletion {
            text: synthesis_json("Combined summary"),
            input_tokens: Some(100),
            output_tokens: Some(200),
            latency_ms: 300,
        },
    )
    .await
    .expect("execute synthesis");

    let kinds: Vec<String> = sqlx::query_scalar(
        "SELECT artifact_kind FROM prompt_pack_stage_artifacts
         WHERE stage_run_id = ?
         ORDER BY artifact_index ASC",
    )
    .bind(stage_run_id)
    .fetch_all(&pool)
    .await
    .expect("artifacts");

    assert_eq!(kinds, vec!["prompt_input", "raw_output", "parsed_output", "metrics"]);
}

#[tokio::test]
async fn execute_synthesis_stage_rejects_invalid_output_without_success_artifacts() {
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
    .expect("persist transcript fixtures");

    let stage_run_id: i64 = sqlx::query_scalar(
        "SELECT id FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
    )
    .fetch_one(&pool)
    .await
    .expect("stage row");

    let invalid = r#"{
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "synthesis_candidate": {
            "summary_text": "Combined summary",
            "cross_video_themes": [{ "theme_id": "theme_1", "theme_text": "bad" }],
            "common_claims": [],
            "contradictions_across_videos": []
        },
        "limitations": [],
        "warning_candidates": []
    }"#;
    super::execute_synthesis_stage_with_completion(
        &pool,
        stage_run_id,
        LlmCompletion {
            text: invalid.to_string(),
            input_tokens: Some(100),
            output_tokens: Some(200),
            latency_ms: 300,
        },
    )
    .await
    .expect_err("invalid synthesis fails stage");

    let status: String = sqlx::query_scalar(
        "SELECT stage_status FROM prompt_pack_stage_runs WHERE id = ?",
    )
    .bind(stage_run_id)
    .fetch_one(&pool)
    .await
    .expect("stage status");
    let success_artifacts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_stage_artifacts
         WHERE stage_run_id = ? AND artifact_kind IN ('parsed_output', 'metrics')",
    )
    .bind(stage_run_id)
    .fetch_one(&pool)
    .await
    .expect("success artifacts");
    let quarantine_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts
         WHERE run_id = 1 AND stage_run_id = ?",
    )
    .bind(stage_run_id)
    .fetch_one(&pool)
    .await
    .expect("quarantine count");

    assert_eq!(status, "failed");
    assert_eq!(success_artifacts, 0);
    assert_eq!(quarantine_count, 1);
}
```

- [ ] **Step 5: Run the persistence test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib execute_synthesis_stage_persists_raw_parsed_and_metrics_artifacts
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib execute_synthesis_stage_rejects_invalid_output_without_success_artifacts
```

Expected: both tests FAIL because `execute_synthesis_stage_with_completion` does not exist.

- [ ] **Step 6: Implement persistence**

Implement `execute_synthesis_stage_with_completion` parallel to `execute_transcript_analysis_stage_with_completion`, with these differences:

```rust
const SYNTHESIS_STAGE_NAME: &str = "youtube_summary/synthesis";
// Metrics-only schema identifier for this slice. Do not seed it into
// prompt_pack_schemas until a foundation/schema task adds the asset.
const SYNTHESIS_SCHEMA_ID: &str = "stage-io/youtube_summary_synthesis_output";
```

This plan does not add a `prompt_pack_schemas` row for `SYNTHESIS_SCHEMA_ID`; the value is persisted only inside the synthesis metrics artifact for debugging and audit correlation.

It should:

- mark stage `running`;
- persist `prompt_input #1` from `build_synthesis_stage_input`;
- persist `raw_output #2`;
- parse strict JSON from provider text;
- call `validate_and_quarantine_synthesis_output(pool, run_id, stage_run_id, &parsed).await`;
- on validation failure after successful quarantine write, leave the quarantine row, mark the synthesis stage `failed`, and do not persist `parsed_output #3` or `metrics #4`;
- on quarantine write failure, return the internal quarantine error and do not silently downgrade it to a validation-only error;
- persist `parsed_output #3`;
- persist `metrics #4` with `schema_id`, token usage, latency, and `validation_error_count = 0`;
- mark stage `succeeded`.

- [ ] **Step 7: Add LLM request construction**

In `runtime.rs`, add the request builder below. `LlmChatRequest.max_output_tokens` already exists in the current LLM contract; this task must not add or rename LLM contract fields.

```rust
fn build_synthesis_llm_request(
    run_id: i64,
    stage_run_id: i64,
    prompt_input_json: String,
    profile_id: Option<String>,
    model_override: Option<String>,
    max_output_tokens: Option<i64>,
) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!("prompt-pack-run-{run_id}-stage-{stage_run_id}"),
        profile_id,
        model_override,
        max_output_tokens,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "Return strict JSON for the YouTube Summary synthesis stage. Produce a synthesis_candidate only; the backend assigns canonical IDs and traversal fields.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Synthesize the transcript-analysis candidates into one strict JSON object with stage_io_version, schema_version, stage, synthesis_candidate, limitations, and warning_candidates.\n\nRequired synthesis_candidate shape:\n{{\n  \"summary_text\": \"combined readable summary\",\n  \"cross_video_themes\": [{{ \"theme_text\": \"theme\", \"source_refs\": [\"source_ref_1\"], \"claim_refs\": [], \"evidence_refs\": [] }}],\n  \"common_claims\": [],\n  \"contradictions_across_videos\": []\n}}\n\nThe input wrapper field source_ref_id may be used only for reasoning. Do not copy the key source_ref_id into the output. If you need to cite videos, use source_refs arrays inside synthesis_candidate and only values present in the input. For this slice, keep claim_refs, evidence_refs, and relation_refs as empty arrays because the backend has not exposed allowed claim/evidence/relation ref maps to synthesis output. Do not include backend-owned IDs or keys such as source_ref_id, theme_id, common_claim_id, contradiction_id, claim_id, evidence_id, video_id, section_id, or synthesis_item_id. Do not wrap the JSON in Markdown.\n\nSynthesis input JSON:\n{}",
                    prompt_input_json
                ),
            },
        ],
    }
}
```

- [ ] **Step 8: Run validation and persistence tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_validator
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_validator_rejects_unknown_source_ref
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_validator_rejects_provider_authored_claim_ref
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_is_written_to_quarantine_artifacts
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_with_unknown_source_ref_is_quarantined
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_surfaces_quarantine_write_failure
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib execute_synthesis_stage_persists_raw_parsed_and_metrics_artifacts
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib execute_synthesis_stage_rejects_invalid_output_without_success_artifacts
```

Expected: PASS.

- [ ] **Step 9: Commit**

```powershell
git add src-tauri/src/prompt_packs/validation.rs src-tauri/src/prompt_packs/youtube_summary.rs src-tauri/src/prompt_packs/runtime.rs
git commit -m "Execute YouTube Summary synthesis stage"
```

---

## Task 4: Wire Synthesis Into Run Execution

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`

- [ ] **Step 1: Write the failing run-level tests**

Add a single-video gating test:

```rust
#[tokio::test]
async fn youtube_summary_single_video_run_skips_synthesis() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    execute_youtube_summary_run_with_fake_completions(
        &pool,
        1,
        vec![Ok(LlmCompletion {
            text: transcript_analysis_json("Only summary", "Only claim", "Only evidence"),
            input_tokens: Some(10),
            output_tokens: Some(20),
            latency_ms: 30,
        })],
    )
    .await
    .expect("execute run");

    let status: String = sqlx::query_scalar(
        "SELECT stage_status FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
    )
    .fetch_one(&pool)
    .await
    .expect("synthesis status");

    let result = crate::prompt_packs::result_builder::build_youtube_summary_canonical_result(&pool, 1)
        .await
        .expect("canonical result");

    assert_eq!(status, "skipped");
    assert!(result["outputs"]["pack_data"]["youtube_summary"]["synthesis"].is_null());

    let progress: (i64, i64) = sqlx::query_as(
        "SELECT progress_current, progress_total
         FROM prompt_pack_runs
         WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("progress");

    assert_eq!(progress, (1, 1));
}
```

Add a multi-video execution test:

```rust
#[tokio::test]
async fn youtube_summary_run_executes_synthesis_after_transcript_stages() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    execute_youtube_summary_run_with_fake_completions(
        &pool,
        1,
        vec![
            Ok(LlmCompletion {
                text: transcript_analysis_json("First summary", "First claim", "First evidence"),
                input_tokens: Some(10),
                output_tokens: Some(20),
                latency_ms: 30,
            }),
            Ok(LlmCompletion {
                text: transcript_analysis_json("Second summary", "Second claim", "Second evidence"),
                input_tokens: Some(11),
                output_tokens: Some(21),
                latency_ms: 31,
            }),
            Ok(LlmCompletion {
                text: synthesis_json("Combined summary"),
                input_tokens: Some(100),
                output_tokens: Some(200),
                latency_ms: 300,
            }),
        ],
    )
    .await
    .expect("execute run");

    let status: String = sqlx::query_scalar(
        "SELECT stage_status FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
    )
    .fetch_one(&pool)
    .await
    .expect("synthesis status");

    assert_eq!(status, "succeeded");

    let progress: (i64, i64) = sqlx::query_as(
        "SELECT progress_current, progress_total
         FROM prompt_pack_runs
         WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("progress");

    assert_eq!(progress, (3, 3));
}

#[tokio::test]
async fn youtube_summary_run_marks_partial_when_synthesis_fails() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    execute_youtube_summary_run_with_fake_completions(
        &pool,
        1,
        vec![
            Ok(LlmCompletion {
                text: transcript_analysis_json("First summary", "First claim", "First evidence"),
                input_tokens: Some(10),
                output_tokens: Some(20),
                latency_ms: 30,
            }),
            Ok(LlmCompletion {
                text: transcript_analysis_json("Second summary", "Second claim", "Second evidence"),
                input_tokens: Some(11),
                output_tokens: Some(21),
                latency_ms: 31,
            }),
            Err("synthesis provider failed".to_string()),
        ],
    )
    .await
    .expect("execute run");

    let (run_status, result_status): (String, String) = sqlx::query_as(
        "SELECT runs.run_status, results.result_status
         FROM prompt_pack_runs runs
         JOIN prompt_pack_results results ON results.run_id = runs.id
         WHERE runs.id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("run result status");
    let synthesis_status: String = sqlx::query_scalar(
        "SELECT stage_status FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
    )
    .fetch_one(&pool)
    .await
    .expect("synthesis status");

    assert_eq!(run_status, "partial");
    assert_eq!(result_status, "partial");
    assert_eq!(synthesis_status, "failed");

    let progress: (i64, i64) = sqlx::query_as(
        "SELECT progress_current, progress_total
         FROM prompt_pack_runs
         WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("progress");

    assert_eq!(progress, (2, 3));
}

#[tokio::test]
async fn youtube_summary_multi_video_partial_transcripts_skip_synthesis_and_mark_partial() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    execute_youtube_summary_run_with_fake_completions(
        &pool,
        1,
        vec![
            Ok(LlmCompletion {
                text: transcript_analysis_json("First summary", "First claim", "First evidence"),
                input_tokens: Some(10),
                output_tokens: Some(20),
                latency_ms: 30,
            }),
            Err("transcript provider failed".to_string()),
        ],
    )
    .await
    .expect("execute run");

    let (run_status, result_status): (String, String) = sqlx::query_as(
        "SELECT runs.run_status, results.result_status
         FROM prompt_pack_runs runs
         JOIN prompt_pack_results results ON results.run_id = runs.id
         WHERE runs.id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("run result status");
    let synthesis_status: String = sqlx::query_scalar(
        "SELECT stage_status FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
    )
    .fetch_one(&pool)
    .await
    .expect("synthesis status");

    assert_eq!(run_status, "partial");
    assert_eq!(result_status, "partial");
    assert_eq!(synthesis_status, "skipped");

    let progress: (i64, i64) = sqlx::query_as(
        "SELECT progress_current, progress_total
         FROM prompt_pack_runs
         WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("progress");

    assert_eq!(progress, (1, 2));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_run_executes_synthesis_after_transcript_stages
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_single_video_run_skips_synthesis
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_run_marks_partial_when_synthesis_fails
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_multi_video_partial_transcripts_skip_synthesis_and_mark_partial
```

Expected: FAIL because the synthesis stage remains `not_implemented`, is not executed, or the new terminal status cases are not wired.

- [ ] **Step 3: Gate synthesis stage skeleton by included video count**

In `insert_stage_skeleton`, remove `youtube_summary/synthesis` from the `not_implemented` tail list and insert it separately. Single-video runs must start as `skipped`; multi-video runs may start as `pending`:

```rust
for (offset, name) in [
    "segment_extraction",
    "key_point_extraction",
    "quote_extraction",
]
.iter()
.enumerate()
{
    insert_stage(
        pool,
        run_id,
        None,
        name,
        100 + offset as i64,
        "not_implemented",
        now,
    )
    .await?;
}

let synthesis_status = if included_count > 1 { "pending" } else { "skipped" };
insert_stage(
    pool,
    run_id,
    None,
    "youtube_summary/synthesis",
    103,
    synthesis_status,
    now,
)
.await?;
```

This keeps canonical `synthesis = null` for single-video runs, matching `VR-YS-005`.

- [ ] **Step 4: Execute synthesis after transcript successes**

Introduce an explicit request enum so transcript and synthesis executions cannot be confused:

```rust
pub(crate) enum YoutubeSummaryStageExecutionRequest {
    TranscriptAnalysis(TranscriptAnalysisStageExecutionRequest),
    Synthesis(SynthesisStageExecutionRequest),
}

pub(crate) struct SynthesisStageExecutionRequest {
    pub run_id: i64,
    pub stage_run_id: i64,
    pub prompt_input_json: String,
}
```

Update the executor signature:

```rust
F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
```

Update every call site in the same step:

- `execute_youtube_summary_run_with_stage_executor` accepts the enum request and wraps transcript requests as `YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request)`;
- `execute_youtube_summary_run_with_fake_completions` matches the enum and consumes the next fake completion for either transcript or synthesis;
- the real runtime closure in `src-tauri/src/prompt_packs/runtime.rs` matches `TranscriptAnalysis(request)` to the existing transcript LLM request path and `Synthesis(request)` to `build_synthesis_llm_request`;
- tests that inspect transcript requests unwrap `YoutubeSummaryStageExecutionRequest::TranscriptAnalysis`;
- no call site keeps a conditional executor-shape branch.

After the transcript stage loop and before final canonical result persistence, call:

```rust
let synthesis_status =
    execute_synthesis_if_ready(pool, run_id, successes, total, &mut execute_stage).await?;
mark_pending_mvp_tail_stages_skipped(pool, run_id).await?;
```

Do not call `mark_pending_mvp_tail_stages_skipped` before `execute_synthesis_if_ready`. Update `mark_pending_mvp_tail_stages_skipped` so it only skips pending future MVP tail stages and never skips the synthesis row:

```sql
WHERE run_id = ?
  AND stage_status = 'pending'
  AND stage_name NOT IN (
      'youtube_summary/transcript_analysis',
      'youtube_summary/synthesis'
  )
```

`execute_synthesis_if_ready` must:

- return `Ok("skipped")` without an LLM call when `successes <= 1`;
- mark the synthesis stage `skipped` when `successes <= 1` and the row is still `pending`;
- build synthesis input and execute the stage when `successes > 1`;
- return `Ok("succeeded")` after successful persistence;
- return `Ok("failed")` after marking synthesis failed;
- return a cancellation outcome when the scheduler reports cancellation.

For the MVP fake-completion helper, pop completions in order and map transcript requests to transcript completions and the synthesis request to the next completion.

Progress and UI events must use this rule:

- initialize `progress_total` to transcript stage count;
- if `successes > 1`, add one synthesis progress unit before sending the synthesis request;
- successful synthesis increments `progress_current` by one;
- failed synthesis leaves `progress_current` at transcript successes and keeps the synthesis unit in `progress_total`;
- skipped synthesis does not add a progress unit;
- the real runtime emits the same queue/start/finish event shape as transcript stages, with `phase: "synthesis"`, `stage_name: Some("youtube_summary/synthesis")`, `stage_run_id`, `source_snapshot_id: None`, and message `"Synthesizing videos"`.

- [ ] **Step 5: Define terminal status behavior**

Use:

- transcript failures with no transcript successes: `failed`;
- single-video run with one transcript success and synthesis skipped: `complete`;
- multi-video run with one transcript success and synthesis skipped: `partial`;
- multiple transcript successes and synthesis success: `complete`;
- transcript successes and synthesis failure: `partial`;
- multi-video transcript partial coverage with synthesis skipped: `partial`;
- cancellation during synthesis: `cancelled`.

- [ ] **Step 6: Run the run-level test**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_run_executes_synthesis_after_transcript_stages
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_single_video_run_skips_synthesis
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_run_marks_partial_when_synthesis_fails
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_multi_video_partial_transcripts_skip_synthesis_and_mark_partial
```

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary.rs src-tauri/src/prompt_packs/runtime.rs
git commit -m "Wire YouTube Summary synthesis into runs"
```

---

## Task 5: Add Synthesis To Canonical Result

**Files:**
- Modify: `src-tauri/src/prompt_packs/result_builder.rs`

- [ ] **Step 1: Write the failing canonical result test**

Add or update a test:

```rust
#[tokio::test]
async fn build_canonical_result_includes_synthesis_output() {
    let pool = test_pool_with_two_successful_stage_artifacts().await;
    insert_synthesis_parsed_output(&pool, 42, "Combined summary")
        .await
        .expect("insert synthesis output");

    let result = super::build_youtube_summary_canonical_result(&pool, 42)
        .await
        .expect("canonical result");

    assert_eq!(
        result["outputs"]["pack_data"]["youtube_summary"]["synthesis"]
            ["cross_video_themes"][0]["theme_text"],
        "Shared theme",
    );
    assert_eq!(
        result["outputs"]["pack_data"]["youtube_summary"]["synthesis"]["source_refs"],
        serde_json::json!(["source_ref_1", "source_ref_2"]),
    );
    assert_eq!(
        result["outputs"]["sections"][0]["title"],
        "Summary",
    );
}
```

Add the null-synthesis quality-flag tests:

```rust
#[tokio::test]
async fn build_canonical_result_marks_single_video_synthesis_not_applicable() {
    let pool = test_pool_with_successful_stage_artifacts().await;
    insert_isolated_result_builder_synthesis_stage_status(&pool, 42, "skipped")
        .await
        .expect("insert synthesis status");

    let result = super::build_youtube_summary_canonical_result(&pool, 42)
        .await
        .expect("canonical result");

    assert!(result["outputs"]["pack_data"]["youtube_summary"]["synthesis"].is_null());
    assert!(has_quality_flag(
        &result,
        "synthesis_not_applicable_single_video"
    ));
}

#[tokio::test]
async fn build_canonical_result_marks_multi_video_synthesis_failed() {
    let pool = test_pool_with_two_successful_stage_artifacts().await;
    insert_isolated_result_builder_synthesis_stage_status(&pool, 42, "failed")
        .await
        .expect("insert synthesis status");

    let result = super::build_youtube_summary_canonical_result(&pool, 42)
        .await
        .expect("canonical result");

    assert!(result["outputs"]["pack_data"]["youtube_summary"]["synthesis"].is_null());
    assert!(has_quality_flag(&result, "synthesis_failed"));
    assert!(
        result["limitations"]
            .as_array()
            .expect("limitations")
            .iter()
            .any(|value| value
                .as_str()
                .unwrap_or("")
                .contains("synthesis stage failed"))
    );
}

#[tokio::test]
async fn build_canonical_result_marks_multi_video_synthesis_skipped_insufficient_successes() {
    let pool = test_pool_with_two_sources_one_successful_stage_artifact().await;
    insert_isolated_result_builder_synthesis_stage_status(&pool, 42, "skipped")
        .await
        .expect("insert synthesis status");

    let result = super::build_youtube_summary_canonical_result(&pool, 42)
        .await
        .expect("canonical result");

    assert!(result["outputs"]["pack_data"]["youtube_summary"]["synthesis"].is_null());
    assert!(has_quality_flag(
        &result,
        "synthesis_skipped_insufficient_successes"
    ));
}

#[tokio::test]
async fn build_canonical_result_keeps_partial_result_flag_when_synthesis_is_skipped() {
    let pool = test_pool_with_two_sources_one_successful_stage_artifact().await;
    insert_isolated_result_builder_synthesis_stage_status(&pool, 42, "skipped")
        .await
        .expect("insert synthesis status");

    let result = super::build_youtube_summary_canonical_result(&pool, 42)
        .await
        .expect("canonical result");

    assert!(has_quality_flag(&result, "partial_result"));
    assert!(has_quality_flag(
        &result,
        "synthesis_skipped_insufficient_successes"
    ));
}

fn has_quality_flag(result: &serde_json::Value, flag: &str) -> bool {
    result["quality_flags"]
        .as_array()
        .expect("quality flags")
        .iter()
        .any(|value| value["flag"].as_str() == Some(flag))
}

// Result-builder tests are isolated canonical assembly tests. The real
// insert_stage_skeleton path is covered by the run-level tests in Task 4.
async fn insert_isolated_result_builder_synthesis_stage_status(
    pool: &sqlx::SqlitePool,
    run_id: i64,
    status: &str,
) -> sqlx::Result<()> {
    let run_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_runs WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await?;
    assert_eq!(run_exists, 1, "result-builder fixture must own the run");

    sqlx::query(
        "INSERT INTO prompt_pack_stage_runs (
            id, run_id, source_snapshot_id, stage_name, stage_order, stage_status,
            created_at, updated_at
         )
         VALUES (2001, ?, NULL, 'youtube_summary/synthesis', 103, ?,
            '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
    )
    .bind(run_id)
    .bind(status)
    .execute(pool)
    .await?;
    let owned_stage_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_runs runs ON runs.id = stages.run_id
         WHERE stages.id = 2001 AND runs.id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await?;
    assert_eq!(owned_stage_exists, 1, "synthesis stage must belong to fixture run");
    Ok(())
}

async fn insert_synthesis_parsed_output(
    pool: &sqlx::SqlitePool,
    run_id: i64,
    summary: &str,
) -> sqlx::Result<()> {
    insert_isolated_result_builder_synthesis_stage_status(pool, run_id, "succeeded").await?;
    let parsed = serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "synthesis_candidate": {
            "summary_text": summary,
            "cross_video_themes": [
                {
                    "theme_text": "Shared theme",
                    "source_refs": ["source_ref_1", "source_ref_2"],
                    "claim_refs": [],
                    "evidence_refs": []
                }
            ],
            "common_claims": [],
            "contradictions_across_videos": []
        },
        "limitations": [],
        "warning_candidates": []
    });
    sqlx::query(
        "INSERT INTO prompt_pack_stage_artifacts (
            run_id, stage_run_id, artifact_kind, attempt_number, artifact_index,
            content_type, content_hash, content_zstd, redaction_state, created_at
         )
         VALUES (?, 2001, 'parsed_output', 1, 3, 'application/json', 'sha384-synthesis', ?, 'none',
            '2026-06-14T00:00:00Z')",
    )
    .bind(run_id)
    .bind(compress_text(&parsed.to_string()).expect("compress synthesis"))
    .execute(pool)
    .await?;
    Ok(())
}

async fn test_pool_with_two_successful_stage_artifacts() -> sqlx::SqlitePool {
    let pool = test_pool_with_successful_stage_artifacts().await;
    insert_second_source_snapshot_and_optional_parsed_output(&pool, true).await;
    pool
}

async fn test_pool_with_two_sources_one_successful_stage_artifact() -> sqlx::SqlitePool {
    let pool = test_pool_with_successful_stage_artifacts().await;
    insert_second_source_snapshot_and_optional_parsed_output(&pool, false).await;
    pool
}

async fn insert_second_source_snapshot_and_optional_parsed_output(
    pool: &sqlx::SqlitePool,
    include_parsed_output: bool,
) {
    sqlx::query(
        "INSERT INTO sources (
            id, source_type, source_subtype, external_id, title,
            is_active, is_member, created_at
         )
         VALUES (902, 'youtube', 'video', 'provider-video-2', 'Video 2', 1, 0, 1)",
    )
    .execute(pool)
    .await
    .expect("insert second source");
    sqlx::query(
        "INSERT INTO prompt_pack_run_source_snapshots (
            id, run_id, source_id, source_ref_id, video_id, title, created_at
         )
         VALUES (502, 42, 902, 'source_ref_2', 'provider-video-2', 'Video 2', '2026-06-14T00:00:00Z')",
    )
    .execute(pool)
    .await
    .expect("insert second snapshot");

    if include_parsed_output {
        sqlx::query(
            "INSERT INTO prompt_pack_stage_runs (
                id, run_id, source_snapshot_id, stage_name, stage_order, stage_status,
                created_at, updated_at
             )
             VALUES (1002, 42, 502, 'youtube_summary/transcript_analysis', 20, 'succeeded',
                '2026-06-14T00:00:00Z', '2026-06-14T00:00:00Z')",
        )
        .execute(pool)
        .await
        .expect("insert second stage");
        let parsed = serde_json::json!({
            "video_candidate": { "summary_text": "Second summary" },
            "claim_candidates": [{ "text": "Second claim" }],
            "evidence_fragment_candidates": [{ "text": "Second evidence" }],
            "warning_candidates": []
        });
        sqlx::query(
            "INSERT INTO prompt_pack_stage_artifacts (
                run_id, stage_run_id, artifact_kind, attempt_number, artifact_index,
                content_type, content_hash, content_zstd, redaction_state, created_at
             )
             VALUES (42, 1002, 'parsed_output', 1, 3, 'application/json', 'sha384-test-2', ?, 'none',
                '2026-06-14T00:00:00Z')",
        )
        .bind(compress_text(&parsed.to_string()).expect("compress second parsed"))
        .execute(pool)
        .await
        .expect("insert second parsed artifact");
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_canonical_result_includes_synthesis_output
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_canonical_result_marks_single_video_synthesis_not_applicable
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_canonical_result_marks_multi_video_synthesis_failed
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_canonical_result_marks_multi_video_synthesis_skipped_insufficient_successes
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_canonical_result_keeps_partial_result_flag_when_synthesis_is_skipped
```

Expected: FAIL because `synthesis` is currently always `null` and no synthesis quality flags exist.

- [ ] **Step 3: Load latest synthesis parsed output**

Add:

```rust
async fn load_latest_run_stage_parsed_output(
    pool: &SqlitePool,
    run_id: i64,
    stage_name: &str,
) -> AppResult<Option<serde_json::Value>> {
    let bytes = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT artifacts.content_zstd
         FROM prompt_pack_stage_artifacts artifacts
         JOIN prompt_pack_stage_runs stages ON stages.id = artifacts.stage_run_id
         WHERE artifacts.run_id = ?
           AND stages.stage_name = ?
           AND artifacts.artifact_kind = 'parsed_output'
         ORDER BY artifacts.attempt_number DESC, artifacts.artifact_index DESC
         LIMIT 1",
    )
    .bind(run_id)
    .bind(stage_name)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    let Some(bytes) = bytes else {
        return Ok(None);
    };
    let text = decompress_text(&bytes).map_err(AppError::internal)?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|error| AppError::internal(format!("parse parsed_output artifact: {error}")))
}
```

- [ ] **Step 4: Add synthesis to canonical JSON**

Load synthesis output, but only expose it in canonical YouTube Summary pack data when the run has more than one video. Single-video results must keep `synthesis = null` to satisfy `VR-YS-005`.

```rust
let synthesis = load_latest_run_stage_parsed_output(pool, run_id, "youtube_summary/synthesis").await?;
let synthesis_stage_status = load_run_stage_status(pool, run_id, "youtube_summary/synthesis").await?;
let synthesis_candidate = synthesis
    .as_ref()
    .and_then(|value| value.get("synthesis_candidate"));
let canonical_synthesis = if videos.len() > 1 {
    synthesis_candidate
        .map(|candidate| build_canonical_synthesis(candidate, &videos))
        .transpose()?
        .unwrap_or(serde_json::Value::Null)
} else {
    serde_json::Value::Null
};
let mut limitations = build_base_limitations(pool, run_id).await?;
let mut quality_flags = build_base_quality_flags(pool, run_id).await?;
match (
    videos.len(),
    canonical_synthesis.is_null(),
    synthesis_stage_status.as_deref(),
) {
    (1, true, _) => {
        limitations.push("Synthesis is not applicable to a single-video YouTube Summary run.".to_string());
        push_quality_flag(&mut quality_flags, "synthesis_not_applicable_single_video", "info");
    }
    (count, true, Some("failed")) if count > 1 => {
        limitations.push("The synthesis stage failed, so the report only includes per-video analysis.".to_string());
        push_quality_flag(&mut quality_flags, "synthesis_failed", "warning");
    }
    (count, true, Some("skipped")) if count > 1 => {
        limitations.push("The synthesis stage was skipped because fewer than two videos produced usable transcript analysis.".to_string());
        push_quality_flag(&mut quality_flags, "synthesis_skipped_insufficient_successes", "warning");
    }
    _ => {}
}
let sections = synthesis_candidate
    .and_then(|candidate| candidate.get("summary_text"))
    .and_then(serde_json::Value::as_str)
    .map(|summary| {
        vec![serde_json::json!({
            "section_id": "section_summary",
            "title": "Summary",
            "body": summary
        })]
    })
    .unwrap_or_default();
```

Add the synthesis canonicalizer and flag helper:

```rust
fn build_canonical_synthesis(
    candidate: &serde_json::Value,
    videos: &[serde_json::Value],
) -> AppResult<serde_json::Value> {
    let source_to_video = videos
        .iter()
        .filter_map(|video| {
            Some((
                video.get("source_ref_id")?.as_str()?.to_string(),
                video.get("video_id")?.as_str()?.to_string(),
            ))
        })
        .collect::<std::collections::HashMap<_, _>>();
    let video_to_source = videos
        .iter()
        .filter_map(|video| {
            Some((
                video.get("video_id")?.as_str()?.to_string(),
                video.get("source_ref_id")?.as_str()?.to_string(),
            ))
        })
        .collect::<std::collections::HashMap<_, _>>();

    let cross_video_themes = candidate
        .get("cross_video_themes")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, theme)| {
            let source_refs = ref_strings(theme.get("source_refs"));
            let video_refs = source_refs
                .iter()
                .filter_map(|source_ref| source_to_video.get(source_ref).cloned())
                .collect::<Vec<_>>();
            serde_json::json!({
                "theme_id": format!("theme_{}", index + 1),
                "theme_text": theme.get("theme_text").and_then(serde_json::Value::as_str).unwrap_or(""),
                "video_refs": video_refs,
                "claim_refs": ref_strings(theme.get("claim_refs")),
                "evidence_refs": ref_strings(theme.get("evidence_refs"))
            })
        })
        .collect::<Vec<_>>();

    let common_claims = candidate
        .get("common_claims")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, claim)| {
            let source_refs = ref_strings(claim.get("source_refs"));
            let video_refs = source_refs
                .iter()
                .filter_map(|source_ref| source_to_video.get(source_ref).cloned())
                .collect::<Vec<_>>();
            serde_json::json!({
                "common_claim_id": format!("common_claim_{}", index + 1),
                "summary_text": claim.get("summary_text").and_then(serde_json::Value::as_str).unwrap_or(""),
                "video_refs": video_refs,
                "claim_refs": ref_strings(claim.get("claim_refs")),
                "evidence_refs": ref_strings(claim.get("evidence_refs"))
            })
        })
        .collect::<Vec<_>>();

    let contradictions = candidate
        .get("contradictions_across_videos")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, contradiction)| {
            let source_refs = ref_strings(contradiction.get("source_refs"));
            let video_refs = source_refs
                .iter()
                .filter_map(|source_ref| source_to_video.get(source_ref).cloned())
                .collect::<Vec<_>>();
            serde_json::json!({
                "contradiction_id": format!("contradiction_{}", index + 1),
                "description": contradiction.get("description").and_then(serde_json::Value::as_str).unwrap_or(""),
                "video_refs": video_refs,
                "relation_refs": ref_strings(contradiction.get("relation_refs")),
                "claim_refs": ref_strings(contradiction.get("claim_refs")),
                "evidence_refs": ref_strings(contradiction.get("evidence_refs"))
            })
        })
        .collect::<Vec<_>>();

    let mut claim_refs = Vec::new();
    let mut relation_refs = Vec::new();
    let mut evidence_refs = Vec::new();
    let mut source_refs = Vec::new();
    extend_unique_refs_from_items(&mut claim_refs, &cross_video_themes, "claim_refs");
    extend_unique_refs_from_items(&mut claim_refs, &common_claims, "claim_refs");
    extend_unique_refs_from_items(&mut claim_refs, &contradictions, "claim_refs");
    extend_unique_refs_from_items(&mut relation_refs, &contradictions, "relation_refs");
    extend_unique_refs_from_items(&mut evidence_refs, &cross_video_themes, "evidence_refs");
    extend_unique_refs_from_items(&mut evidence_refs, &common_claims, "evidence_refs");
    extend_unique_refs_from_items(&mut evidence_refs, &contradictions, "evidence_refs");
    extend_unique_source_refs_from_video_refs(&mut source_refs, &cross_video_themes, &video_to_source);
    extend_unique_source_refs_from_video_refs(&mut source_refs, &common_claims, &video_to_source);
    extend_unique_source_refs_from_video_refs(&mut source_refs, &contradictions, &video_to_source);

    Ok(serde_json::json!({
        "cross_video_themes": cross_video_themes,
        "common_claims": common_claims,
        "contradictions_across_videos": contradictions,
        "claim_refs": claim_refs,
        "relation_refs": relation_refs,
        "evidence_refs": evidence_refs,
        "source_refs": source_refs
    }))
}

fn push_quality_flag(flags: &mut Vec<serde_json::Value>, flag: &str, severity: &str) {
    if !flags.iter().any(|value| value["flag"].as_str() == Some(flag)) {
        flags.push(serde_json::json!({
            "flag": flag,
            "severity": severity
        }));
    }
}

fn ref_strings(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn push_unique_ref(target: &mut Vec<String>, value: String) {
    if !target.iter().any(|existing| existing == &value) {
        target.push(value);
    }
}

fn extend_unique_refs_from_items(
    target: &mut Vec<String>,
    items: &[serde_json::Value],
    field: &str,
) {
    for item in items {
        for value in ref_strings(item.get(field)) {
            push_unique_ref(target, value);
        }
    }
}

fn extend_unique_source_refs_from_video_refs(
    target: &mut Vec<String>,
    items: &[serde_json::Value],
    video_to_source: &std::collections::HashMap<String, String>,
) {
    for item in items {
        for video_ref in ref_strings(item.get("video_refs")) {
            if let Some(source_ref) = video_to_source.get(&video_ref) {
                push_unique_ref(target, source_ref.clone());
            }
        }
    }
}
```

Provider-authored `source_refs` are used only to map each candidate item to canonical `video_refs`. Canonical `synthesis.source_refs` must be derived from those backend-owned `video_refs` and the canonical video map; do not copy provider `source_refs` directly into the top-level traversal arrays.

`build_base_quality_flags` must include existing non-synthesis flags before synthesis flags are added. For this slice, add `partial_result` when the run has more source snapshots than successful transcript parsed outputs:

```rust
async fn build_base_quality_flags(pool: &SqlitePool, run_id: i64) -> AppResult<Vec<serde_json::Value>> {
    let total_sources: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_run_source_snapshots WHERE run_id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    let successful_transcripts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_stage_runs
         WHERE run_id = ?
           AND stage_name = 'youtube_summary/transcript_analysis'
           AND stage_status = 'succeeded'",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    let mut flags = Vec::new();
    if successful_transcripts < total_sources {
        push_quality_flag(&mut flags, "partial_result", "warning");
    }
    Ok(flags)
}

async fn build_base_limitations(_pool: &SqlitePool, _run_id: i64) -> AppResult<Vec<String>> {
    Ok(Vec::new())
}
```

Set:

```rust
"outputs": {
    "sections": sections,
    "pack_data": {
        "youtube_summary": {
            "videos": videos,
            "synthesis": canonical_synthesis
        }
    }
},
"limitations": limitations,
"quality_flags": quality_flags
```

Add the small status loader used above:

```rust
async fn load_run_stage_status(
    pool: &SqlitePool,
    run_id: i64,
    stage_name: &str,
) -> AppResult<Option<String>> {
    sqlx::query_scalar(
        "SELECT stage_status FROM prompt_pack_stage_runs
         WHERE run_id = ? AND stage_name = ?
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(run_id)
    .bind(stage_name)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}
```

- [ ] **Step 5: Run canonical result tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib result_builder
```

Expected: PASS and at least the five synthesis result-builder tests above are listed as passed.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/prompt_packs/result_builder.rs
git commit -m "Include YouTube Summary synthesis in canonical result"
```

---

## Task 6: Project Canonical Synthesis Items

**Files:**
- Modify: `src-tauri/src/prompt_packs/projections.rs`

This task targets the current applied database schema from `src-tauri/migrations/0006_prompt_pack_mvp.sql`, where `prompt_pack_youtube_synthesis_items` has `synthesis_id` and `text`. Do not add a migration to the spec-shaped `synthesis_item_kind`/`synthesis_item_id` columns in this plan; schema convergence is a separate task.

- [ ] **Step 1: Write the failing projection tests**

Add tests beside the existing projection tests:

```rust
#[tokio::test]
async fn persist_final_result_projects_youtube_synthesis_items() {
    let pool = test_pool_with_canonical_result_ready().await;

    persist_final_result_transaction(
        &pool,
        42,
        test_canonical_result_with_synthesis(),
        "complete",
    )
    .await
    .expect("persist result");

    let items: Vec<(String, String)> = sqlx::query_as(
        "SELECT synthesis_id, text
         FROM prompt_pack_youtube_synthesis_items
         WHERE run_id = 42
         ORDER BY synthesis_id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("synthesis projection rows");

    assert_eq!(
        items,
        vec![
            ("common_claim_1".to_string(), "Both videos mention pilots.".to_string()),
            ("theme_1".to_string(), "Shared theme".to_string()),
        ]
    );
}

#[tokio::test]
async fn repair_rebuilds_missing_youtube_synthesis_projection_rows() {
    let pool = test_pool_with_canonical_result_ready().await;
    persist_final_result_transaction(
        &pool,
        42,
        test_canonical_result_with_synthesis(),
        "complete",
    )
    .await
    .expect("persist result");
    sqlx::query("DELETE FROM prompt_pack_youtube_synthesis_items WHERE run_id = 42")
        .execute(&pool)
        .await
        .expect("delete synthesis projections");

    repair_prompt_pack_result_projections(&pool, 42)
        .await
        .expect("repair projections");

    let projected_items: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_youtube_synthesis_items WHERE run_id = 42",
    )
    .fetch_one(&pool)
    .await
    .expect("projected synthesis items");

    assert_eq!(projected_items, 2);
}

fn test_canonical_result_with_synthesis() -> serde_json::Value {
    let mut canonical = test_canonical_result();
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
        "cross_video_themes": [
            {
                "theme_id": "theme_1",
                "theme_text": "Shared theme",
                "video_refs": ["video_1", "video_2"],
                "claim_refs": [],
                "evidence_refs": []
            }
        ],
        "common_claims": [
            {
                "common_claim_id": "common_claim_1",
                "summary_text": "Both videos mention pilots.",
                "video_refs": ["video_1", "video_2"],
                "claim_refs": [],
                "evidence_refs": []
            }
        ],
        "contradictions_across_videos": [],
        "claim_refs": [],
        "relation_refs": [],
        "evidence_refs": [],
        "source_refs": ["source_ref_1", "source_ref_2"]
    });
    canonical
}
```

- [ ] **Step 2: Run projection tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib persist_final_result_projects_youtube_synthesis_items
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib repair_rebuilds_missing_youtube_synthesis_projection_rows
```

Expected: FAIL because `rebuild_projection_rows` deletes `prompt_pack_youtube_synthesis_items` but does not insert synthesis projection rows.

- [ ] **Step 3: Implement synthesis projection rebuild**

In `rebuild_projection_rows`, after projecting videos, add:

```rust
if let Some(synthesis) = canonical["outputs"]["pack_data"]["youtube_summary"]["synthesis"].as_object()
{
    for item in synthesis
        .get("cross_video_themes")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        insert_youtube_synthesis_projection_item(
            pool,
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
            pool,
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
            pool,
            result_row_id,
            run_id,
            item["contradiction_id"].as_str().unwrap_or(""),
            item["description"].as_str().unwrap_or(""),
        )
        .await?;
    }
}
```

Add the helper:

```rust
async fn insert_youtube_synthesis_projection_item(
    pool: &SqlitePool,
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
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

- [ ] **Step 4: Run projection tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib persist_final_result_projects_youtube_synthesis_items
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib repair_rebuilds_missing_youtube_synthesis_projection_rows
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/prompt_packs/projections.rs
git commit -m "Project YouTube Summary synthesis items"
```

---

## Task 7: Verification And Documentation

**Files:**
- Modify: `docs/prompt-packs/runtime_configuration_policy.md`
- Create or modify: `docs/superpowers/verification/2026-06-15-youtube-summary-live-workflow.md`

- [ ] **Step 1: Update runtime configuration docs**

Add `runtime/synthesis.json` next to the transcript-analysis runtime asset in
`docs/prompt-packs/runtime_configuration_policy.md`.

- [ ] **Step 2: Run automated verification**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_budget_comes_from_stage_runtime_config
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_synthesis_stage_input_collects_successful_transcript_outputs
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_synthesis_stage_input_uses_latest_parsed_output_wrappers
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_validator
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_validator_rejects_unknown_source_ref
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib synthesis_output_validator_rejects_provider_authored_claim_ref
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_is_written_to_quarantine_artifacts
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_with_unknown_source_ref_is_quarantined
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib invalid_synthesis_output_surfaces_quarantine_write_failure
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib execute_synthesis_stage_persists_raw_parsed_and_metrics_artifacts
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib execute_synthesis_stage_rejects_invalid_output_without_success_artifacts
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_run_executes_synthesis_after_transcript_stages
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_single_video_run_skips_synthesis
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_run_marks_partial_when_synthesis_fails
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib youtube_summary_multi_video_partial_transcripts_skip_synthesis_and_mark_partial
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_canonical_result_includes_synthesis_output
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_canonical_result_marks_single_video_synthesis_not_applicable
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_canonical_result_marks_multi_video_synthesis_failed
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_canonical_result_marks_multi_video_synthesis_skipped_insufficient_successes
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib build_canonical_result_keeps_partial_result_flag_when_synthesis_is_skipped
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib persist_final_result_projects_youtube_synthesis_items
cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib repair_rebuilds_missing_youtube_synthesis_projection_rows
git diff --check
```

Expected:

- the full lib test run executes more than 0 tests and passes;
- every filtered command prints at least one named test result; do not accept a green `running 0 tests` output;
- the filtered commands include the new budget, input builder, validator, persistence, run-level, result-builder, and projection test names from this plan;
- `git diff --check` exits 0.

- [ ] **Step 3: Optional live provider verification**

Ask before spending provider tokens. If approved, run a live YouTube Summary provider run through MCP Bridge and verify:

- `youtube_summary/transcript_analysis` succeeds;
- `youtube_summary/synthesis` succeeds;
- artifacts exist for both stages;
- canonical result includes non-null `outputs.pack_data.youtube_summary.synthesis`;
- `prompt_pack_youtube_synthesis_items` has projected rows for the same run.

- [ ] **Step 4: Create or update verification notes**

If `docs/superpowers/verification/2026-06-15-youtube-summary-live-workflow.md` does not exist, create it with this minimum structure. If it already exists, append a new section with the same fields:

```markdown
# YouTube Summary Live Workflow Verification

## Synthesis Stage

- Date: 2026-06-15
- Scope: YouTube Summary transcript-analysis plus synthesis stage
- Automated verification:
  - `cargo test --manifest-path src-tauri\Cargo.toml --target-dir src-tauri\target\codex-youtube-summary-synthesis --lib`
  - targeted synthesis, result-builder, and projection tests from this plan
- Live provider verification:
  - Status: not run
  - Run ID:
  - Transcript stage:
  - Synthesis stage:
  - Canonical synthesis:
  - Projection rows:
- Notes:
```

- [ ] **Step 5: Commit**

```powershell
git add docs/prompt-packs/runtime_configuration_policy.md docs/superpowers/verification/2026-06-15-youtube-summary-live-workflow.md
git commit -m "Document YouTube Summary synthesis verification"
```

---

## Self-Review

- Spec coverage: the plan adds a run-scoped synthesis stage, runtime budget config, latest-per-stage synthesis input, persisted artifacts, dedicated synthesis-candidate validation/quarantine, run execution wiring, canonical synthesis assembly, projection rebuild for `prompt_pack_youtube_synthesis_items`, quality flags for every null-synthesis reason, documentation, and verification.
- Review fixes applied during self-review: the synthesis-output canonical test now uses a two-video fixture; Task 2 defines its test helpers before the failing tests; Task 2 verifies latest parsed output and pipeline-owned candidate wrappers; Task 3 verifies invalid synthesis output does not create `parsed_output`/`metrics`; Task 4 verifies synthesis failure and multi-video insufficient-success terminal states.
- Review fixes applied after the updated review: provider output is now explicitly `synthesis_candidate`, while backend code assigns canonical synthesis IDs and traversal refs; result-builder flags are merged through `push_quality_flag` and preserve `partial_result`; terminal status rules distinguish single-video success from multi-video partial success; Task 6 adds normal and repair projection tests for `prompt_pack_youtube_synthesis_items`; quarantine write failures are surfaced instead of swallowed; result-builder synthesis-stage fixtures are marked isolated and assert run ownership; the synthesis prompt explicitly forbids copying `source_ref_id`; verification notes are create-or-modify with a minimum template.
- Remaining-risk fixes applied: Task 4 now runs synthesis before `mark_pending_mvp_tail_stages_skipped` and excludes `youtube_summary/synthesis` from tail-skip; Task 3 validates synthesis `source_refs` against run-owned source snapshots and quarantines unknown refs such as `source_ref_999`; provider-authored claim/evidence/relation refs are rejected until the backend exposes allowed ref maps; Task 5 derives top-level `synthesis.source_refs` from canonical `video_refs` rather than copying provider source lists; `SYNTHESIS_SCHEMA_ID` is explicitly metrics-only and not seeded in this slice; Task 4 defines synthesis progress/event semantics; Task 6 explicitly targets the current applied projection schema rather than migrating to the future spec shape.
- Placeholder scan: no open implementation placeholders remain. The only remaining `running 0 tests` text is the explicit verification warning that a green zero-test filtered run is not acceptable. Optional live provider verification is explicitly gated by user approval because it spends provider tokens.
- Type consistency: task snippets use existing `LlmCompletion`, `TranscriptAnalysisStageExecutionRequest`, `prompt_pack_stage_artifacts`, and `prompt_pack_stage_runs` patterns. Task 4 introduces `YoutubeSummaryStageExecutionRequest`, and the same task lists every call site that must be updated before tests run. Test-only helper references use `super::...` or `crate::...` where needed so the snippets are not relying on hidden imports.
- Verification consistency: the final automated verification starts with a real full lib test command (`cargo test --lib`) and then runs named filtered tests; filtered commands must show at least one named test result.
