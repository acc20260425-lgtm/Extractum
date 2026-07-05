# YouTube Summary Gem Analysis Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the approved single-video `Gem analysis` YouTube Summary mode that runs independent transcript/comment/transcript Gem parts and assembles one Markdown report into `video_candidate.summary_text`.

**Architecture:** Keep `gem_analysis` inside the existing YouTube Summary prompt pack as a `control_preset`. Freeze transcript material from bounded ordered transcript segments as the single source of truth, compute the effective Gem input cap in `runtime.rs` before execution starts, then use a focused Gem execution helper to build per-part prompts, check all part budgets before the first LLM call, call the selected runtime, repair part JSON once, and persist one normal transcript-analysis output. Existing `standard` and `detailed_report` flows stay on the current single-completion path.

**Tech Stack:** Svelte 5 + TypeScript frontend, Vitest contract tests, Tauri/Rust backend, SQLx SQLite, existing prompt-pack runtime and artifact tables.

## Global Constraints

- UI label is `Gem analysis`; internal `control_preset` is `gem_analysis`.
- UI default remains `detailed_report`; bundled pack default remains `standard`.
- `gem_analysis` supports exactly one included YouTube video.
- Part 1 gets transcript only; part 2 gets comments only and is skipped when trimmed comment text is empty; part 3 gets transcript only.
- Transcript snapshot `text_zstd` is rendered from bounded ordered transcript segments; structured timing metadata is written for all transcript snapshots, not only Gem runs.
- Gem part 1 and part 3 timestamped input is rendered from the same bounded transcript segment list as `text_zstd`.
- Part 1 and part 3 input overflow blocks before any provider call; v1 does not truncate Gem transcript input.
- Part 2 sentiment language is scoped to the selected comment sample and does not ask for exact percentages.
- Gem input-budget cap is computed in `runtime.rs` from the selected model input limit when known and the bundled transcript-analysis `max_prompt_tokens`; execution receives the already computed cap.
- Cancellation checkpoints are required before part 1, before part 2, before part 3, and before final persistence.
- v1 has no partial per-part result cache; a required part failure makes a retry rerun all parts.
- No new `artifact_kind` values in this slice.
- Update `docs/value-registry.md` for new `control_preset` and any new event `phase` values.
- Use `npm.cmd` for npm scripts on Windows.
- Execute this plan in an isolated branch or worktree before the first implementation task.

---

## File Structure

- `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte` adds the `Gem analysis` option and keeps `detailed_report` default.
- `src/lib/youtube-summary-launch-contract.test.ts` locks the UI option and default.
- `src-tauri/prompt-packs/youtube_summary/1.0.0/pack.json` adds allowed preset metadata only if the current pack schema has a local allowed-values field; otherwise leave defaults unchanged and rely on value registry/tests.
- `docs/value-registry.md` registers `gem_analysis` and new Gem event phases.
- `src-tauri/src/prompt_packs/youtube_summary/sources.rs` owns transcript segment loading and rendering helpers.
- `src-tauri/src/prompt_packs/youtube_summary/snapshots.rs` stores transcript segment metadata in `metadata_json_zstd` and keeps `text_zstd` rendered from the same segments.
- `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs` proves transcript text and timing metadata stay aligned.
- `src-tauri/src/prompt_packs/youtube_summary/preflight.rs` blocks `gem_analysis` unless exactly one video is included.
- `src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs` covers the single-video guard.
- `src-tauri/src/prompt_packs/youtube_summary/types.rs` adds Gem part request/repair types and enum variants.
- `src-tauri/src/prompt_packs/runtime.rs` builds/runs Gem part and repair requests, optional browser discriminators, output budgets, prompt-pack `max_prompt_tokens` reading, selected model input-limit resolution, and part prompt wrappers.
- `src-tauri/src/prompt_packs/youtube_summary/gem_analysis.rs` is a new focused orchestrator for material loading, prompt input assembly, input-budget checks using the runtime-provided cap, part output parsing/repair decisions, final Markdown assembly, and final transcript-analysis JSON assembly.
- `src-tauri/src/prompt_packs/youtube_summary/execution.rs` branches to `execute_gem_analysis_transcript_stage` when `control_preset == "gem_analysis"`.
- `src-tauri/src/prompt_packs/youtube_summary/outputs.rs` accepts an optional metrics extension when persisting assembled Gem output.
- Existing Rust test files in `youtube_summary` and `runtime.rs` receive focused tests near the code they verify.

---

### Task 0: Isolate The Implementation Workspace

**Files:**
- No repository file changes expected.

**Interfaces:**
- Consumes: current `main` branch with approved spec and plan commits.
- Produces: an isolated implementation branch or worktree before code edits start.

- [ ] **Step 1: Check the current worktree**

Run:

```powershell
git status --short
```

Expected: no tracked file changes. Untracked local IDE files such as `.claude/settings.local.json` may remain untracked and must not be staged.

- [ ] **Step 2: Create an isolated implementation branch or worktree**

Preferred if using the repository directly:

```powershell
git switch -c feature/youtube-summary-gem-analysis
```

Alternative when using the worktree flow:

```powershell
git worktree add .worktrees/youtube-summary-gem-analysis -b feature/youtube-summary-gem-analysis
```

Expected: implementation work no longer lands directly on `main`.

- [ ] **Step 3: Commit**

No commit is required for this task.

---

### Task 1: Register `gem_analysis` In UI, Registry, And Preflight

**Files:**
- Modify: `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte`
- Modify: `src/lib/youtube-summary-launch-contract.test.ts`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/preflight.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs`
- Modify: `docs/value-registry.md`

**Interfaces:**
- Consumes: existing `controlPreset` Svelte state and `PreflightYoutubeSummaryRunRequest.control_preset`.
- Produces: backend preflight failure reason `gem_analysis_requires_single_video`.

- [ ] **Step 1: Write the failing UI contract test**

Add this assertion to `wires the selected youtube summary mode into preflight and start requests`:

```ts
expect(dialog).toContain('<option value="gem_analysis">Gem analysis</option>');
expect(dialog).toContain("let controlPreset = $state(\"detailed_report\")");
expect(dialog).toContain("controlPreset = \"detailed_report\"");
```

- [ ] **Step 2: Run the UI contract test and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/youtube-summary-launch-contract.test.ts
```

Expected: FAIL because `gem_analysis` is not present in the Svelte file.

- [ ] **Step 3: Add the Svelte option**

In `YoutubeSummaryRunDialog.svelte`, add the new option without changing defaults:

```svelte
<option value="standard">Standard</option>
<option value="detailed_report">Detailed report</option>
<option value="gem_analysis">Gem analysis</option>
```

- [ ] **Step 4: Add the multi-video preflight fixture helper**

In `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`, add:

```rust
pub(crate) async fn test_pool_with_playlist_two_ready_videos() -> sqlx::SqlitePool {
    let pool = migrated_pool().await;
    seed_builtin_prompt_packs_in_pool(&pool)
        .await
        .expect("seed pack");
    insert_playlist(&pool, 701).await;
    insert_youtube_video(&pool, 901, "v-ready-1").await;
    insert_youtube_video(&pool, 902, "v-ready-2").await;
    insert_transcript(&pool, 901, "Ready transcript one").await;
    insert_transcript(&pool, 902, "Ready transcript two").await;
    insert_playlist_item(&pool, 701, Some(901), "v-ready-1", 1).await;
    insert_playlist_item(&pool, 701, Some(902), "v-ready-2", 2).await;
    pool
}
```

- [ ] **Step 5: Add backend preflight tests**

In `preflight_tests.rs`, add:

```rust
#[tokio::test]
async fn preflight_gem_analysis_allows_exactly_one_included_video() {
    let pool = test_pool_with_ready_video().await;
    let mut request = request_for_video(901);
    request.control_preset = "gem_analysis".to_string();

    let response = preflight_youtube_summary_in_pool(
        &pool,
        request,
        ModelBudget {
            input_token_limit: Some(32_000),
        },
    )
    .await
    .expect("preflight");

    assert_eq!(response.included_videos.len(), 1);
    assert!(response.blocking_failures.is_empty());
}

#[tokio::test]
async fn preflight_gem_analysis_blocks_multiple_included_videos() {
    let pool = test_pool_with_playlist_two_ready_videos().await;
    let mut request = request_for_playlist(701);
    request.control_preset = "gem_analysis".to_string();

    let response = preflight_youtube_summary_in_pool(
        &pool,
        request,
        ModelBudget {
            input_token_limit: Some(32_000),
        },
    )
    .await
    .expect("preflight");

    assert_eq!(response.blocking_failures[0].reason, "gem_analysis_requires_single_video");
    assert!(response.blocking_failures[0]
        .message
        .as_deref()
        .unwrap_or_default()
        .contains("exactly one YouTube video"));
}
```

- [ ] **Step 6: Run preflight tests and verify the new multi-video test fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib preflight_gem_analysis
```

Expected: FAIL because the guard is not implemented.

- [ ] **Step 7: Implement the preflight guard**

At the end of `preflight_youtube_summary_in_pool`, before returning, insert:

```rust
if request.control_preset == "gem_analysis" && included_videos.len() != 1 {
    blocking_failures.push(YoutubeSummaryPreflightFailure {
        source_id: None,
        reason: "gem_analysis_requires_single_video".to_string(),
        message: Some("Gem analysis supports exactly one YouTube video.".to_string()),
    });
}
```

Keep the existing included/skipped lists intact so the UI can still show what was discovered.

- [ ] **Step 8: Update value registry**

In `docs/value-registry.md`, update the `Prompt-pack control preset` row to include:

```text
`standard`, `detailed_report`, `gem_analysis`
```

Mention `gem_analysis` is single-video and sequential multi-request.

- [ ] **Step 9: Run focused verification**

Run:

```powershell
npm.cmd run test -- src/lib/youtube-summary-launch-contract.test.ts
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib preflight_gem_analysis
```

Expected: both PASS.

- [ ] **Step 10: Commit Task 1**

```powershell
git add src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte src/lib/youtube-summary-launch-contract.test.ts src-tauri/src/prompt_packs/youtube_summary/preflight.rs src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs src-tauri/src/prompt_packs/youtube_summary/test_support.rs docs/value-registry.md
git commit -m "feat(prompt-packs): register gem analysis summary mode"
```

---

### Task 2: Freeze Transcript Segment Metadata As Source Of Truth

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/sources.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/snapshots.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`

**Interfaces:**
- Produces: `TranscriptSnapshotSegment { start_ms, end_ms, text }`.
- Produces: `transcript_snapshot_segments_for_source(pool, source_id) -> AppResult<Vec<TranscriptSnapshotSegment>>`.
- Produces: `render_transcript_snapshot_text(&[TranscriptSnapshotSegment]) -> String`.
- `insert_material` gains `metadata_json: Option<&serde_json::Value>` and writes `metadata_json_zstd`.

- [ ] **Step 1: Add failing snapshot tests**

In `snapshots_tests.rs`, add:

```rust
#[tokio::test]
async fn transcript_snapshot_text_is_rendered_from_structured_segments() {
    let pool = test_pool_with_ready_video().await;
    let request = start_request("req-transcript-segments", vec![901]);

    let run = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("start")
        .expect_started("started");

    let (text_zstd, metadata_json_zstd): (Vec<u8>, Vec<u8>) = sqlx::query_as(
        "SELECT text_zstd, metadata_json_zstd
         FROM prompt_pack_run_material_snapshots
         WHERE run_id = ? AND material_kind = 'transcript'",
    )
    .bind(run.run_id)
    .fetch_one(&pool)
    .await
    .expect("transcript material");

    let text = decompress_text(&text_zstd).expect("text");
    let metadata = decompress_text(&metadata_json_zstd).expect("metadata");
    let value: serde_json::Value = serde_json::from_str(&metadata).expect("metadata json");
    let segments = value["segments"].as_array().expect("segments");

    let joined = segments
        .iter()
        .map(|segment| segment["text"].as_str().expect("segment text"))
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(text, joined);
    assert_eq!(segments[0]["start_ms"], serde_json::json!(0));
    assert!(segments[0]["end_ms"].as_i64().unwrap_or_default() >= 0);
}

#[tokio::test]
async fn transcript_text_for_source_uses_segment_renderer() {
    let pool = test_pool_with_ready_video().await;

    let segments = transcript_snapshot_segments_for_source(&pool, 901)
        .await
        .expect("segments");
    let rendered = render_transcript_snapshot_text(&segments);
    let legacy_text = transcript_text_for_source(&pool, 901)
        .await
        .expect("text");

    assert_eq!(legacy_text, rendered);
}
```

Import `transcript_snapshot_segments_for_source`, `render_transcript_snapshot_text`, and `transcript_text_for_source` from `sources.rs`.

- [ ] **Step 2: Run the new tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib transcript_snapshot
```

Expected: FAIL because metadata is not written and helper functions do not exist.

- [ ] **Step 3: Add transcript segment helpers**

In `sources.rs`, add:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct TranscriptSnapshotSegment {
    pub(crate) start_ms: i64,
    pub(crate) end_ms: i64,
    pub(crate) text: String,
}

pub(crate) async fn transcript_snapshot_segments_for_source(
    pool: &SqlitePool,
    source_id: i64,
) -> AppResult<Vec<TranscriptSnapshotSegment>> {
    sqlx::query_as::<_, (i64, i64, String)>(
        "SELECT start_ms, end_ms, text
         FROM youtube_transcript_segments
         WHERE source_id = ?
         ORDER BY segment_index ASC, id ASC",
    )
    .bind(source_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(start_ms, end_ms, text)| TranscriptSnapshotSegment {
                start_ms,
                end_ms,
                text,
            })
            .collect()
    })
    .map_err(AppError::database)
}

pub(crate) fn render_transcript_snapshot_text(
    segments: &[TranscriptSnapshotSegment],
) -> String {
    segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}
```

Change `transcript_text_for_source` to call these helpers:

```rust
let segments = transcript_snapshot_segments_for_source(pool, source_id).await?;
Ok(render_transcript_snapshot_text(&segments))
```

- [ ] **Step 4: Extend material insertion with metadata**

In `snapshots.rs`, import `compress_text` is already available through `insert_material`; add a small helper:

```rust
fn compress_metadata_json(value: &serde_json::Value) -> AppResult<Vec<u8>> {
    compress_text(&value.to_string()).map_err(AppError::internal)
}
```

Change `insert_material` signature:

```rust
async fn insert_material(
    pool: &SqlitePool,
    run_id: i64,
    source_snapshot_id: i64,
    material_ref_id: &str,
    material_kind: &str,
    external_id: Option<&str>,
    sequence_index: i64,
    text: &str,
    metadata_json: Option<&serde_json::Value>,
    now: &str,
) -> AppResult<()>
```

Change the SQL to include `metadata_json_zstd`:

```sql
INSERT OR IGNORE INTO prompt_pack_run_material_snapshots (
    run_id, source_snapshot_id, material_ref_id, material_kind,
    external_id, sequence_index, text_zstd, metadata_json_zstd, token_estimate, created_at
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
```

Bind:

```rust
.bind(metadata_json.map(compress_metadata_json).transpose()?)
```

Pass `None` for description and comments.

- [ ] **Step 5: Snapshot transcript from structured segments**

In `insert_material_snapshots`, replace the transcript branch with:

```rust
let transcript_segments = transcript_snapshot_segments_for_source(pool, source_id).await?;
let transcript = render_transcript_snapshot_text(&transcript_segments);
if !transcript.trim().is_empty() {
    let metadata = serde_json::json!({
        "kind": "youtube_transcript_segments",
        "segments": transcript_segments,
    });
    insert_material(
        pool,
        run_id,
        source_snapshot_id,
        &format!("m_{}_transcript", source_ref_id),
        "transcript",
        None,
        0,
        &transcript,
        Some(&metadata),
        now,
    )
    .await?;
}
```

Update all other `insert_material` calls with the new `metadata_json` argument.

- [ ] **Step 6: Run focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib transcript_snapshot
```

Expected: PASS.

- [ ] **Step 7: Commit Task 2**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary/sources.rs src-tauri/src/prompt_packs/youtube_summary/snapshots.rs src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs src-tauri/src/prompt_packs/youtube_summary/test_support.rs
git commit -m "feat(prompt-packs): freeze transcript segment metadata"
```

---

### Task 3: Add Gem Runtime Request Types, IDs, Prompts, And Part Parser

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/types.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/mod.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Create: `src-tauri/src/prompt_packs/youtube_summary/gem_analysis.rs`

**Interfaces:**
- Produces enum `GemAnalysisPart::{Passport, Comments, DeepRecap}`.
- Produces `GemAnalysisInputBudget { max_input_tokens: i64 }`.
- Produces request structs `GemAnalysisPartStageExecutionRequest` and `GemAnalysisPartRepairRequest`.
- Extends `YoutubeSummaryStageExecutionRequest` with `GemAnalysisPart` and `GemAnalysisPartRepair`.
- Produces `parse_gem_analysis_part_output(raw, expected_part) -> AppResult<GemAnalysisPartOutput>`.

- [ ] **Step 1: Add failing runtime ID tests**

In the `runtime.rs` tests module, add tests that call helper functions directly:

```rust
#[test]
fn browser_run_id_accepts_optional_gem_discriminator() {
    assert_eq!(
        browser_run_id_for_stage(42, 1001, None, None),
        "prompt-pack-42-stage-1001"
    );
    assert_eq!(
        browser_run_id_for_stage(42, 1001, Some(2), None),
        "prompt-pack-42-stage-1001-repair-2"
    );
    assert_eq!(
        browser_run_id_for_stage(42, 1001, None, Some("gem-passport")),
        "prompt-pack-42-stage-1001-gem-passport"
    );
    assert_eq!(
        browser_run_id_for_stage(42, 1001, None, Some("gem-deep-recap-repair-1")),
        "prompt-pack-42-stage-1001-gem-deep-recap-repair-1"
    );
}
```

Expected future signature:

```rust
fn browser_run_id_for_stage(
    run_id: i64,
    stage_run_id: i64,
    repair_attempt_number: Option<i64>,
    request_discriminator: Option<&str>,
) -> String
```

- [ ] **Step 2: Add failing part parser tests**

In `gem_analysis.rs`, include a `#[cfg(test)]` module with:

```rust
#[test]
fn parse_part_output_accepts_matching_non_empty_markdown() {
    let parsed = parse_gem_analysis_part_output(
        r#"{"part":"passport","markdown":"### Раздел\nТекст"}"#,
        GemAnalysisPart::Passport,
    )
    .expect("parse");

    assert_eq!(parsed.part, GemAnalysisPart::Passport);
    assert_eq!(parsed.markdown, "### Раздел\nТекст");
}

#[test]
fn parse_part_output_rejects_wrong_part() {
    let error = parse_gem_analysis_part_output(
        r#"{"part":"comments","markdown":"### Раздел"}"#,
        GemAnalysisPart::Passport,
    )
    .expect_err("wrong part");

    assert!(error.message.contains("expected part passport"));
}

#[test]
fn parse_part_output_rejects_empty_markdown() {
    let error = parse_gem_analysis_part_output(
        r#"{"part":"passport","markdown":"   "}"#,
        GemAnalysisPart::Passport,
    )
    .expect_err("empty markdown");

    assert!(error.message.contains("markdown"));
}

#[test]
fn parse_part_output_accepts_json_fence_with_internal_markdown_code_block() {
    let raw = "```json\n{\"part\":\"deep_recap\",\"markdown\":\"### Код\\n```rust\\nfn main() {}\\n```\\nФормула: $E=mc^2$\"}\n```";

    let parsed = parse_gem_analysis_part_output(raw, GemAnalysisPart::DeepRecap)
        .expect("parse fenced JSON with code block inside markdown string");

    assert!(parsed.markdown.contains("```rust"));
    assert!(parsed.markdown.contains("$E=mc^2$"));
}
```

- [ ] **Step 3: Run tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_part
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib browser_run_id_accepts_optional_gem_discriminator
```

Expected: FAIL because types/helpers do not exist.

- [ ] **Step 4: Add Gem types**

In `types.rs`, add:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GemAnalysisPart {
    Passport,
    Comments,
    DeepRecap,
}

impl GemAnalysisPart {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Passport => "passport",
            Self::Comments => "comments",
            Self::DeepRecap => "deep_recap",
        }
    }

    pub(crate) fn slug(self) -> &'static str {
        match self {
            Self::Passport => "passport",
            Self::Comments => "comments",
            Self::DeepRecap => "deep-recap",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisInputBudget {
    pub(crate) max_input_tokens: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisPartStageExecutionRequest {
    pub run_id: i64,
    pub stage_run_id: i64,
    pub source_snapshot_id: i64,
    pub source_ref_id: String,
    pub part: GemAnalysisPart,
    pub prompt_input_json: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisPartRepairRequest {
    pub run_id: i64,
    pub stage_run_id: i64,
    pub source_snapshot_id: i64,
    pub source_ref_id: String,
    pub part: GemAnalysisPart,
    pub attempt_number: i64,
    pub prompt_input_json: String,
    pub raw_output: String,
    pub error_message: String,
}
```

Extend the enum:

```rust
GemAnalysisPart(GemAnalysisPartStageExecutionRequest),
GemAnalysisPartRepair(GemAnalysisPartRepairRequest),
```

Re-export these types from `youtube_summary/mod.rs` next to existing request types.

- [ ] **Step 5: Add part parser**

In `gem_analysis.rs`, add:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisPartOutput {
    pub(crate) part: GemAnalysisPart,
    pub(crate) markdown: String,
}

pub(crate) fn parse_gem_analysis_part_output(
    raw: &str,
    expected_part: GemAnalysisPart,
) -> AppResult<GemAnalysisPartOutput> {
    let value = crate::prompt_packs::stage_io::extract_json_payload(raw)?;
    let part = value
        .get("part")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| AppError::validation("Gem analysis part output is missing part"))?;
    if part != expected_part.as_str() {
        return Err(AppError::validation(format!(
            "Gem analysis part output expected part {} but got {part}",
            expected_part.as_str()
        )));
    }
    let markdown = value
        .get("markdown")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|markdown| !markdown.is_empty())
        .ok_or_else(|| AppError::validation("Gem analysis part output markdown is empty"))?
        .to_string();
    Ok(GemAnalysisPartOutput {
        part: expected_part,
        markdown,
    })
}
```

- [ ] **Step 6: Update browser helper signature**

Change `browser_run_id_for_stage` and `browser_run_source_for_stage` to accept an optional discriminator:

```rust
fn browser_run_id_for_stage(
    run_id: i64,
    stage_run_id: i64,
    repair_attempt_number: Option<i64>,
    request_discriminator: Option<&str>,
) -> String
```

Rules:

```rust
match (request_discriminator, repair_attempt_number) {
    (Some(discriminator), _) => format!("prompt-pack-{run_id}-stage-{stage_run_id}-{discriminator}"),
    (None, Some(attempt_number)) => format!("prompt-pack-{run_id}-stage-{stage_run_id}-repair-{attempt_number}"),
    (None, None) => format!("prompt-pack-{run_id}-stage-{stage_run_id}"),
}
```

Pass `None` from existing transcript, synthesis, and JSON repair callers.

- [ ] **Step 7: Add Gem LLM request builders**

In `runtime.rs`, add builders:

```rust
fn gem_part_request_suffix(part: GemAnalysisPart) -> String {
    format!("gem-{}", part.slug())
}

fn gem_part_repair_request_suffix(part: GemAnalysisPart, attempt_number: i64) -> String {
    format!("gem-{}-repair-{attempt_number}", part.slug())
}
```

Add `build_gem_analysis_part_llm_request` and `build_gem_analysis_part_repair_llm_request` with request IDs:

```rust
format!(
    "prompt-pack-run-{}-stage-{}-{}",
    request.run_id,
    request.stage_run_id,
    gem_part_request_suffix(request.part)
)
```

and repair:

```rust
format!(
    "prompt-pack-run-{}-stage-{}-{}",
    request.run_id,
    request.stage_run_id,
    gem_part_repair_request_suffix(request.part, request.attempt_number)
)
```

The system message must be:

```text
Return strict JSON for one Gem analysis part. Do not include Markdown fences, prose outside JSON, comments, or backend-owned IDs. Put the complete Russian Markdown report in the markdown field.
```

The user message must include:

```text
Return exactly one strict JSON object:
{
  "part": "<part>",
  "markdown": "<full Russian Markdown report>"
}
Use only the input material provided below for this part. Do not use outputs from other Gem analysis parts. Do not invent timestamps, metadata, source titles, subscriber counts, metrics, or links. If a requested item is unavailable in the provided material, write `Недоступно во входных данных`. If transcript input has no `[MM:SS]` timestamps, state that timestamps are unavailable in the input and do not create approximate timestamps. For fact-checking, do not fabricate sources or URLs; if external verification is unavailable in the current runtime, explicitly state that limitation. Do not start markdown with # or ##; the backend assembler owns the top-level report title and part headings. Start internal headings at ###, use #### for nested headings, and avoid leading/trailing horizontal rules.
```

- [ ] **Step 8: Route new enum variants in runtime**

In the central runtime match that currently dispatches `TranscriptAnalysis`, `Synthesis`, and `JsonRepair`, add branches for:

```rust
YoutubeSummaryStageExecutionRequest::GemAnalysisPart(request) => {
    run_gem_analysis_part_stage_request(handle, pool, completion_runtime, run_cancellation_token, request).await
}
YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(request) => {
    run_gem_analysis_part_repair_request(handle, pool, completion_runtime, run_cancellation_token, request).await
}
```

Use phases:

- `gem_passport`
- `gem_comments`
- `gem_deep_recap`
- `gem_part_repair`

Use started messages:

- `Gem analysis: building analytical passport`
- `Gem analysis: analyzing comments`
- `Gem analysis: writing deep recap`
- `Gem analysis: repairing part JSON`

- [ ] **Step 9: Run focused runtime tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_part
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib browser_run_id
```

Expected: PASS.

- [ ] **Step 10: Update value registry for phases**

Add `gem_passport`, `gem_comments`, `gem_deep_recap`, and `gem_part_repair` to the event phase registry row in `docs/value-registry.md`.

- [ ] **Step 11: Commit Task 3**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary/types.rs src-tauri/src/prompt_packs/youtube_summary/mod.rs src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/youtube_summary/gem_analysis.rs docs/value-registry.md
git commit -m "feat(prompt-packs): add gem analysis runtime requests"
```

---

### Task 4: Build Gem Material Inputs, Input Budget, And Markdown Assembly

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/gem_analysis.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs`

**Interfaces:**
- Produces `GemAnalysisMaterialInput { transcript_plain, transcript_timestamped, comments_text, comments_status }`.
- Produces `build_gem_analysis_part_prompt_input(...) -> serde_json::Value`.
- Produces `assemble_gem_analysis_markdown(...) -> String`.
- Produces `assemble_gem_analysis_transcript_output(...) -> String`.

- [ ] **Step 1: Add material-loading tests**

In `gem_analysis.rs` tests, add:

```rust
#[tokio::test]
async fn load_gem_materials_formats_timestamped_transcript_from_metadata() {
    let pool = test_pool_with_ready_video().await;
    let request = start_request("req-gem-materials", vec![901]);
    let run = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("start")
        .expect_started("started");
    let stage_id = transcript_analysis_stage_id(&pool, run.run_id).await;

    let materials = load_gem_analysis_materials(&pool, stage_id)
        .await
        .expect("materials");

    assert!(materials.transcript_timestamped.contains("[00:00]"));
    assert!(materials.transcript_plain.contains("First transcript segment"));
    assert!(!materials.transcript_timestamped.contains("youtube_comment"));
}

#[tokio::test]
async fn load_gem_materials_skips_empty_comment_rows() {
    let pool = test_pool_with_ready_video().await;
    insert_comment(&pool, 901, "empty-comment", 10, "").await;
    let mut request = start_request("req-gem-empty-comments", vec![901]);
    request.include_comments = true;
    let run = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("start")
        .expect_started("started");
    let stage_id = transcript_analysis_stage_id(&pool, run.run_id).await;

    let materials = load_gem_analysis_materials(&pool, stage_id)
        .await
        .expect("materials");

    assert_eq!(materials.comments_status, GemCommentsStatus::SkippedNoComments);
    assert!(materials.comments_text.trim().is_empty());
}
```

Use existing test support for creating runs. If a helper name differs, add small local helpers in the test module that insert exactly the rows needed.

- [ ] **Step 2: Add assembly tests**

Add:

```rust
#[test]
fn assemble_gem_markdown_nests_part_markdown_under_backend_headings() {
    let markdown = assemble_gem_analysis_markdown(
        "### Метаданные\nТекст",
        Some("### Сентимент\nТекст"),
        "### Пересказ\nТекст",
    );

    assert!(markdown.starts_with("# Gem-анализ"));
    assert!(markdown.contains("## Часть 1. Аналитический паспорт видео"));
    assert!(markdown.contains("### Метаданные"));
    assert!(!markdown.contains("\n# Метаданные"));
}

#[test]
fn assemble_gem_transcript_output_contains_empty_candidate_arrays() {
    let output = assemble_gem_analysis_transcript_output("# Gem-анализ\n\nТекст")
        .expect("output");
    let value: serde_json::Value = serde_json::from_str(&output).expect("json");

    assert_eq!(value["stage"], "youtube_summary/transcript_analysis");
    assert_eq!(value["claim_candidates"], serde_json::json!([]));
    assert_eq!(value["evidence_fragment_candidates"], serde_json::json!([]));
    assert!(value["video_candidate"]["summary_text"]
        .as_str()
        .unwrap()
        .starts_with("# Gem-анализ"));
}
```

- [ ] **Step 3: Run material/assembly tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_materials
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib assemble_gem
```

Expected: FAIL because helpers are not implemented.

- [ ] **Step 4: Implement material loading**

In `gem_analysis.rs`, define:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GemCommentsStatus {
    Present,
    SkippedNoComments,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisMaterialInput {
    pub(crate) transcript_plain: String,
    pub(crate) transcript_timestamped: String,
    pub(crate) comments_text: String,
    pub(crate) comments_status: GemCommentsStatus,
}
```

Load transcript material by querying `prompt_pack_run_material_snapshots` for the stage's run/source snapshot. Decompress `text_zstd` for plain text. Decompress `metadata_json_zstd` when present and read `segments`. Render timestamped text:

```rust
fn format_timestamp_ms(start_ms: i64) -> String {
    let total_seconds = (start_ms / 1000).max(0);
    format!("[{:02}:{:02}]", total_seconds / 60, total_seconds % 60)
}

fn render_timestamped_segments(segments: &[TranscriptSnapshotSegment]) -> String {
    segments
        .iter()
        .map(|segment| format!("{} {}", format_timestamp_ms(segment.start_ms), segment.text))
        .collect::<Vec<_>>()
        .join("\n")
}
```

If metadata is absent or invalid, use plain text for `transcript_timestamped` and record a boolean used by prompt input:

```rust
"timestamps_available": false
```

For comments, concatenate decompressed `material_kind = 'comment'` rows ordered by `sequence_index, id`, using blank-line separators. Set `SkippedNoComments` when `comments_text.trim().is_empty()`.

- [ ] **Step 5: Implement prompt input builders**

In `gem_analysis.rs`, define the full part prompt literals before the builder functions:

```rust
const GEM_PASSPORT_PROMPT_BODY: &str = r#"Системная роль:
Вы - ведущий аналитик видеоконтента и эксперт по структурированию знаний. Специализация - экспресс-деконструкция медиаматериалов, создание применимых How-to руководств и аккуратная проверка доступных утверждений.

Цель:
Создать структурированный аналитический паспорт видео на основе только предоставленного transcript material.

Правила входных данных:
- Используйте только transcript material из этого запроса.
- Не используйте комментарии, описание, метаданные источника или результаты других Gem parts.
- Не придумывайте title, URL, автора, подписчиков, длительность, дату публикации, просмотры, внешние ссылки или timestamp.
- Если поле недоступно во входе, пишите: `Недоступно во входных данных`.
- Таймкоды берите только из `[MM:SS]`, реально присутствующих во входном transcript material.
- Если независимый фактчекинг недоступен в runtime, не фабрикуйте источники и ссылки.
- Markdown должен начинаться с `###`; не используйте `#` или `##`, потому что backend добавляет заголовок части.

Структура отчета:

### I. Метаданные и Контекст
- **Тип контента:** точный жанр.
- **Наличие пошаговых инструкций:** Да или Нет; укажите, есть ли готовый алгоритм действий.
- **Целевая аудитория:** кому и для каких задач полезно содержание.
- **Инфо-карта:** Название видео | Автор | Метрики: длительность, дата публикации, просмотры. Заполняйте только если это есть в transcript material; иначе `Недоступно во входных данных`.
- **Таймлайн:** хронологический список ключевых этапов с реальными `[MM:SS]`.

### II. Эссенция
- **Main Idea:** главная мысль строго одним сильным предложением.
- **Ключевые тезисы:** 3-5 фундаментальных выводов, аргументов, цифр или коротких цитат из transcript material.
- **Action Plan:** 2-3 конкретных шага для внедрения знаний после просмотра.

### III. Пошаговое руководство (How-to)
Заполняется только если описан процесс.
- **Цель инструкции:** измеримый результат.
- **Инструменты и ресурсы:** полный список необходимого.
- **Алгоритм:** нумерованный список. Для каждого шага: **Действие**, **Таймкод** `[MM:SS]`, **Нюанс/Предостережение**.

### IV. Адаптивный модуль
- Если это обучение: глоссарий из 5+ сложных терминов с простыми определениями и 1 практическое домашнее задание.
- Если это новости или аналитика: список действующих лиц/организаций и исторический, рыночный или геополитический контекст из transcript material.
- Если содержание длиннее 20 минут по доступным таймкодам: FAQ из 5 вопросов и ответов строго по transcript material.

### V. Внешний контекст и Ресурсы (упоминания и доступный фактчекинг)
- **Список упоминаний:** книги, авторы, сервисы, законы и внешние ссылки, которые озвучены в transcript material.
- **Доступный фактчекинг:** если runtime не предоставляет внешнюю проверку, напишите: `Независимый фактчекинг недоступен в текущем runtime; ниже перечислены только упоминания из транскрипта.`

Стиль:
Русский язык. Профессионально, без воды и без личных местоимений. Не используйте фразы-филлеры вроде `В данном видеоролике...`."#;

const GEM_COMMENTS_PROMPT_BODY: &str = r#"Системная роль:
Вы - эксперт по анализу общественного мнения, работе с аудиторией и сентимент-анализу.

Цель:
Провести анализ только предоставленного selected comment sample. Это ограниченная выборка комментариев, а не все комментарии и не вся аудитория.

Правила входных данных:
- Используйте только comment material из этого запроса.
- Не используйте transcript, описание, метаданные источника или результаты других Gem parts.
- Не цитируйте комментарии дословно; обобщайте.
- Не называйте выводы репрезентативными для всей аудитории.
- Не давайте точные проценты sentiment в v1. Используйте качественные формулировки: преимущественно позитивный, смешанный, скептический, нейтральный.
- Markdown должен начинаться с `###`; не используйте `#` или `##`.

Структура анализа:

### 1. Общий сентимент
Опишите качественное распределение настроений внутри предоставленной выборки комментариев и общий эмоциональный фон. Обязательно укажите, что вывод относится только к selected comment sample.

### 2. Ключевые темы обсуждения
Выделите 3-5 главных тем, сгруппируйте мнения и укажите, какие темы вызвали наибольший резонанс в выборке.

### 3. Вопросы и боли аудитории
Составьте структурированный список частых или глубоких вопросов, которые важны для зрителей и не раскрыты в предоставленных комментариях/контексте.

### 4. Ценные инсайты и дополнения
Извлеките полезные дополнения: альтернативные сервисы, личный опыт, исправления ошибок, экспертные уточнения.

### 5. Конструктивная критика
Разделите критику на категории: **техническая часть**, **подача**, **фактология**.

Стиль:
Строго русский. Объективно, основанно на данных выборки, без личных суждений об аудитории."#;

const GEM_DEEP_RECAP_PROMPT_BODY: &str = r#"Системная роль:
Вы - ведущий аналитик видеоконтента и эксперт по структурированию знаний. Специализация - деконструкция сложных видео в плотные, глубокие текстовые пересказы высокой точности.

Цель:
Создать глубокий интерактивный пересказ основного содержания на основе только provided transcript material.

Правила входных данных:
- Используйте только transcript material из этого запроса.
- Не используйте комментарии, описание, метаданные источника или результаты других Gem parts.
- Каждому ключевому тезису, факту или аргументу сопоставляйте реальный `[MM:SS]` из transcript material.
- Не придумывайте timestamp. Если timestamp недоступен для конкретного пункта, лучше опустить timestamp, чем создать приблизительный.
- Markdown должен начинаться с `###`; не используйте `#` или `##`.
- Используйте `####` для вложенных подразделов.
- Не добавляйте leading/trailing `---`; внутренние разделители используйте только когда они помогают чтению.

Требования к пересказу:

### Объем и плотность
Минимум 800-1000 слов, если transcript material достаточно содержателен. Полное отсутствие воды, вводных фраз и лирических отступлений. Только факты, методологии и логические цепочки.

### Структура
Разбейте текст на логические главы с осмысленными заголовками уровня `###`. Каждый раздел раскрывайте детально.

### Интерактивная навигация
Каждому ключевому тезису, факту или аргументу сопутствует реальный `[MM:SS]` из входа.

### Визуализация данных
Если сравниваются подходы, инструменты или концепции, оформите сравнение Markdown-таблицей. Списки используйте для свойств и этапов.

### Технический блок
Если во входе есть формулы, используйте LaTeX. Если есть код, оформляйте его fenced code block с языком.

Стиль:
Строго русский. Академический, аналитический, лаконичный. Не используйте фразы-филлеры вроде `В этом видео говорится...`, `Автор рассказывает...`, `Блогер объясняет...`."#;
```

Build per-part prompt input JSON with only the allowed material:

```rust
serde_json::json!({
    "part": part.as_str(),
    "source_ref_id": source_ref_id,
    "timestamps_available": timestamps_available,
    "input_material": {
        "kind": "transcript",
        "text": transcript_timestamped,
    },
    "task": GEM_PASSPORT_PROMPT_BODY
})
```

For comments:

```rust
serde_json::json!({
    "part": "comments",
    "input_material": {
        "kind": "selected_comment_sample",
        "sample_limit_note": "Analysis is based only on the provided selected comment sample.",
        "text": comments_text,
    },
    "task": GEM_COMMENTS_PROMPT_BODY
})
```

For deep recap:

```rust
serde_json::json!({
    "part": "deep_recap",
    "source_ref_id": source_ref_id,
    "timestamps_available": timestamps_available,
    "input_material": {
        "kind": "transcript",
        "text": transcript_timestamped,
    },
    "task": GEM_DEEP_RECAP_PROMPT_BODY
})
```

Do not include transcript in comments input. Do not include comments in transcript inputs.

- [ ] **Step 6: Implement input budget helpers**

Add:

```rust
const GEM_INPUT_ESTIMATOR_OVERHEAD_TOKENS: i64 = 1_500;

fn estimate_gem_prompt_tokens(prompt_input_json: &str, wrapper_text: &str) -> i64 {
    super::estimate_tokens(prompt_input_json) + super::estimate_tokens(wrapper_text) + GEM_INPUT_ESTIMATOR_OVERHEAD_TOKENS
}

fn enforce_gem_input_budget(part: GemAnalysisPart, estimate: i64, cap: i64) -> AppResult<()> {
    if estimate > cap {
        return Err(AppError::validation(format!(
            "Gem analysis input for {} exceeds the selected model input budget",
            part.as_str()
        )));
    }
    Ok(())
}
```

The cap value is supplied by `GemAnalysisInputBudget` from Task 5. For Task 4, unit-test the pure helper with a small cap such as `GemAnalysisInputBudget { max_input_tokens: 100 }`.

- [ ] **Step 7: Implement Markdown and transcript-output assembly**

`assemble_gem_analysis_markdown` must produce:

```markdown
# Gem-анализ

## Часть 1. Аналитический паспорт видео

<part 1 markdown>

---

## Часть 2. Анализ комментариев к видео

<part 2 markdown or skipped/failure note>

---

## Часть 3. Глубокий интерактивный пересказ

<part 3 markdown>
```

`assemble_gem_analysis_transcript_output` must serialize:

```rust
serde_json::json!({
    "stage_io_version": "1.0",
    "schema_version": "1.0",
    "stage": "youtube_summary/transcript_analysis",
    "video_candidate": {
        "summary_text": markdown,
    },
    "claim_candidates": [],
    "evidence_fragment_candidates": [],
    "warning_candidates": [],
})
```

- [ ] **Step 8: Run focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_materials
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib assemble_gem
```

Expected: PASS.

- [ ] **Step 9: Commit Task 4**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary/gem_analysis.rs src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs
git commit -m "feat(prompt-packs): build gem analysis materials"
```

---

### Task 5: Execute Gem Mini-Pipeline With Cancellation, Repair, And Metrics

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/gem_analysis.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/execution.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/outputs.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/execution` tests in the existing module.

**Interfaces:**
- Produces `execute_gem_analysis_transcript_stage(...)`.
- Produces `YoutubeSummaryExecutionOptions { gem_input_budget: GemAnalysisInputBudget }`.
- Produces persistence helper `execute_transcript_analysis_stage_with_completion_and_metrics_extension(...)`.
- Consumes `YoutubeSummaryStageExecutionRequest::GemAnalysisPart` and `GemAnalysisPartRepair`.

- [ ] **Step 1: Introduce execution options and rename the internal executor**

In `execution.rs`, add the options struct:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct YoutubeSummaryExecutionOptions {
    pub(crate) gem_input_budget: GemAnalysisInputBudget,
}
```

Rename `execute_youtube_summary_run_with_stage_executor_internal` to `execute_youtube_summary_run_with_stage_executor_with_options` and add the `options` parameter immediately, before adding any wrapper that calls it:

```rust
pub(crate) async fn execute_youtube_summary_run_with_stage_executor_with_options<F, Fut, M>(
    pool: &SqlitePool,
    run_id: i64,
    options: YoutubeSummaryExecutionOptions,
    mut execute_stage: F,
    mutate_final_result: M,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
    M: FnOnce(&mut serde_json::Value),
```

Do not use `options` yet in this step except to accept it; the compiler may allow the unused variable warning. This keeps the crate compiling before Gem branching is added.

- [ ] **Step 2: Keep the old test helper API behind `#[cfg(test)]` only**

Add a test-only unbounded helper. Do not expose this in production builds:

```rust
#[cfg(test)]
impl YoutubeSummaryExecutionOptions {
    pub(crate) fn unbounded_for_tests() -> Self {
        Self {
            gem_input_budget: GemAnalysisInputBudget {
                max_input_tokens: i64::MAX,
            },
        }
    }
}
```

Gate the 3-argument wrapper behind `#[cfg(test)]`:

```rust
#[cfg(test)]
pub(crate) async fn execute_youtube_summary_run_with_stage_executor<F, Fut>(
    pool: &SqlitePool,
    run_id: i64,
    execute_stage: F,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
{
    execute_youtube_summary_run_with_stage_executor_with_options(
        pool,
        run_id,
        YoutubeSummaryExecutionOptions::unbounded_for_tests(),
        execute_stage,
        |_| {},
    )
    .await
}
```

Update the test-only `execute_youtube_summary_run_with_stage_executor_and_result_mutator` to call the same `execute_youtube_summary_run_with_stage_executor_with_options` helper with `YoutubeSummaryExecutionOptions::unbounded_for_tests()`.

Keep the crate-level exports and runtime imports aligned with the `#[cfg(test)]` gate in the same implementation pass:

- In `youtube_summary/mod.rs`, split the executor re-exports if they are currently grouped. The options-based executor stays available in release builds, while the old 3-argument wrapper is test-only:

```rust
pub(crate) use execution::execute_youtube_summary_run_with_stage_executor_with_options;

#[cfg(test)]
pub(crate) use execution::execute_youtube_summary_run_with_stage_executor;
```

- In `runtime.rs`, after moving `execute_youtube_summary_run` to `execute_youtube_summary_run_with_stage_executor_with_options` in Step 4, remove `execute_youtube_summary_run_with_stage_executor` from the non-test import list. Import the 3-argument wrapper only from `#[cfg(test)]` test code if a test still needs it.
- Do not leave an intermediate state where release code imports or re-exports the `#[cfg(test)]` wrapper. The release guard is `cargo check --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis`, and it must compile without the old 3-argument API.

Production code must call only `execute_youtube_summary_run_with_stage_executor_with_options` with a real runtime-computed `YoutubeSummaryExecutionOptions`.

- [ ] **Step 3: Add runtime budget tests**

In `runtime.rs` tests, add:

```rust
#[test]
fn transcript_analysis_stage_max_prompt_token_budget_reads_runtime_config() {
    assert_eq!(
        transcript_analysis_stage_max_prompt_token_budget().expect("prompt budget"),
        24_000
    );
}

#[test]
fn gem_input_budget_uses_lower_known_model_limit() {
    assert_eq!(gem_input_cap(Some(8_000), 24_000), 8_000);
    assert_eq!(gem_input_cap(Some(64_000), 24_000), 24_000);
    assert_eq!(gem_input_cap(None, 24_000), 24_000);
}
```

- [ ] **Step 4: Implement runtime prompt-budget reading and cap resolution**

In `runtime.rs`, extend `StageBudgetLimits`:

```rust
#[derive(Deserialize)]
struct StageBudgetLimits {
    max_prompt_tokens: Option<i64>,
    max_output_tokens: Option<i64>,
}
```

Add:

```rust
fn stage_max_prompt_token_budget(asset_json: &str, label: &str) -> AppResult<i64> {
    let asset = serde_json::from_str::<StageRuntimeConfigAsset>(asset_json).map_err(|error| {
        AppError::internal(format!(
            "Parse bundled {label} runtime configuration: {error}"
        ))
    })?;
    asset
        .runtime_configuration
        .and_then(|runtime| runtime.budget_limits)
        .and_then(|budget| budget.max_prompt_tokens)
        .filter(|max_prompt_tokens| *max_prompt_tokens > 0)
        .ok_or_else(|| {
            AppError::internal(format!(
                "Bundled {label} runtime configuration is missing positive max_prompt_tokens"
            ))
        })
}

fn transcript_analysis_stage_max_prompt_token_budget() -> AppResult<i64> {
    stage_max_prompt_token_budget(TRANSCRIPT_ANALYSIS_STAGE_JSON, "transcript-analysis")
}

fn gem_input_cap(model_input_limit: Option<usize>, prompt_budget: i64) -> i64 {
    match model_input_limit.and_then(|limit| i64::try_from(limit).ok()).filter(|limit| *limit > 0) {
        Some(model_limit) => model_limit.min(prompt_budget),
        None => prompt_budget,
    }
}
```

In `execute_youtube_summary_run`, resolve the model input limit before calling the execution helper:

```rust
let (completion_runtime, model_input_limit) = match config.runtime_provider {
    RunRuntimeProvider::Api => {
        let profile = resolve_profile_for_backend(&handle, config.profile_id.as_deref()).await?;
        let effective_model = resolve_effective_model(&profile, config.model_override.as_deref())?;
        let model_input_limit =
            resolve_model_input_token_limit_for_backend(&profile, &effective_model).await;
        (
            RunCompletionRuntime::Api {
                profile,
                model_override: config.model_override.clone(),
            },
            model_input_limit,
        )
    }
    RunRuntimeProvider::GeminiBrowser => (
        RunCompletionRuntime::GeminiBrowser {
            browser_provider_config: config.browser_provider_config.clone(),
        },
        None,
    ),
};
let prompt_budget = transcript_analysis_stage_max_prompt_token_budget()?;
let execution_options = YoutubeSummaryExecutionOptions {
    gem_input_budget: GemAnalysisInputBudget {
        max_input_tokens: gem_input_cap(model_input_limit, prompt_budget),
    },
};
```

Call:

```rust
execute_youtube_summary_run_with_stage_executor_with_options(
    &pool,
    run_id,
    execution_options,
    move |stage_request| {
        let handle = handle.clone();
        let pool = stage_pool.clone();
        let completion_runtime = completion_runtime.clone();
        let run_cancellation_token = run_cancellation_token.clone();
        async move {
            match stage_request {
                YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => {
                    run_transcript_analysis_stage_request(
                        handle,
                        pool,
                        completion_runtime,
                        run_cancellation_token,
                        request,
                    )
                    .await
                }
                YoutubeSummaryStageExecutionRequest::Synthesis(request) => {
                    run_synthesis_stage_request(
                        handle,
                        pool,
                        completion_runtime,
                        run_cancellation_token,
                        request,
                    )
                    .await
                }
                YoutubeSummaryStageExecutionRequest::JsonRepair(request) => {
                    run_json_repair_stage_request(
                        handle,
                        pool,
                        completion_runtime,
                        run_cancellation_token,
                        request,
                    )
                    .await
                }
                YoutubeSummaryStageExecutionRequest::GemAnalysisPart(request) => {
                    run_gem_analysis_part_stage_request(
                        handle,
                        pool,
                        completion_runtime,
                        run_cancellation_token,
                        request,
                    )
                    .await
                }
                YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(request) => {
                    run_gem_analysis_part_repair_request(
                        handle,
                        pool,
                        completion_runtime,
                        run_cancellation_token,
                        request,
                    )
                    .await
                }
            }
        }
    },
    |_| {},
)
```

Import `resolve_model_input_token_limit_for_backend` alongside the existing output-limit resolver.

- [ ] **Step 5: Add output persistence test for metrics extension**

In `outputs_tests.rs`, add:

```rust
#[tokio::test]
async fn transcript_stage_metrics_can_include_gem_analysis_extension() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;
    let completion = fake_completion_with_valid_transcript_analysis_json();

    execute_transcript_analysis_stage_with_completion_and_metrics_extension(
        &pool,
        stage_id,
        completion,
        Some(serde_json::json!({
            "gem_analysis": {
                "parts": [
                    { "part": "passport", "status": "succeeded" }
                ]
            }
        })),
    )
    .await
    .expect("execute stage");

    let metrics = load_stage_artifact_json(&pool, stage_id, "metrics").await;
    assert_eq!(
        metrics["gem_analysis"]["parts"][0]["part"],
        serde_json::json!("passport")
    );
}
```

The `fake_completion_with_valid_transcript_analysis_json()` helper already exists in `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`; import it through the existing `use super::test_support::*;` pattern used by nearby tests.

Add this explicit helper in `outputs_tests.rs`:

```rust
async fn load_stage_artifact_json(
    pool: &sqlx::SqlitePool,
    stage_id: i64,
    artifact_kind: &str,
) -> serde_json::Value {
    let content_zstd: Vec<u8> = sqlx::query_scalar(
        "SELECT content_zstd FROM prompt_pack_stage_artifacts
         WHERE stage_run_id = ? AND artifact_kind = ?",
    )
    .bind(stage_id)
    .bind(artifact_kind)
    .fetch_one(pool)
    .await
    .expect("artifact content");
    let text = crate::compression::decompress_text(&content_zstd).expect("decompress artifact");
    serde_json::from_str(&text).expect("artifact json")
}
```

- [ ] **Step 6: Implement metrics extension persistence**

In `outputs.rs`, keep the existing public function as a wrapper:

```rust
pub(crate) async fn execute_transcript_analysis_stage_with_completion(
    pool: &SqlitePool,
    stage_run_id: i64,
    completion: LlmCompletion,
) -> AppResult<()> {
    execute_transcript_analysis_stage_with_completion_and_metrics_extension(
        pool,
        stage_run_id,
        completion,
        None,
    )
    .await
}
```

Add:

```rust
pub(crate) async fn execute_transcript_analysis_stage_with_completion_and_metrics_extension(
    pool: &SqlitePool,
    stage_run_id: i64,
    completion: LlmCompletion,
    metrics_extension: Option<serde_json::Value>,
) -> AppResult<()>
```

Build metrics as a mutable object immediately before `let parsed_json = ...`:

```rust
let mut metrics = serde_json::json!({
    "input_tokens": completion.input_tokens,
    "output_tokens": completion.output_tokens,
    "latency_ms": completion.latency_ms,
    "schema_id": TRANSCRIPT_ANALYSIS_OUTPUT_SCHEMA_ID,
    "validation_error_count": 0,
    "attempt_number": 1
});
if let Some(extension) = metrics_extension {
    let metrics_object = metrics
        .as_object_mut()
        .expect("base metrics is an object");
    let extension_object = extension
        .as_object()
        .ok_or_else(|| AppError::internal("metrics extension must be a JSON object"))?;
    for (key, value) in extension_object {
        metrics_object.insert(key.clone(), value.clone());
    }
}
```

Then keep the existing `insert_stage_artifact_in_transaction(..., &metrics.to_string())` call unchanged.

- [ ] **Step 7: Add Gem execution test helpers**

In `test_support.rs`, add:

```rust
pub(crate) async fn test_pool_with_ready_video_and_comments() -> sqlx::SqlitePool {
    let pool = test_pool_with_ready_video().await;
    insert_comment(&pool, 901, "comment-1", 10, "Useful comment").await;
    insert_comment(&pool, 901, "comment-2", 20, "Second useful comment").await;
    pool
}
```

In `execution_tests.rs`, add local helpers:

```rust
fn request_kind_label(request: &YoutubeSummaryStageExecutionRequest) -> &'static str {
    match request {
        YoutubeSummaryStageExecutionRequest::GemAnalysisPart(request) => match request.part {
            GemAnalysisPart::Passport => "gem_passport",
            GemAnalysisPart::Comments => "gem_comments",
            GemAnalysisPart::DeepRecap => "gem_deep_recap",
        },
        YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(_) => "gem_part_repair",
        YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(_) => "transcript_analysis",
        YoutubeSummaryStageExecutionRequest::Synthesis(_) => "synthesis",
        YoutubeSummaryStageExecutionRequest::JsonRepair(_) => "json_repair",
    }
}

fn fake_gem_part_completion(part: GemAnalysisPart) -> LlmCompletion {
    LlmCompletion {
        text: serde_json::json!({
            "part": part.as_str(),
            "markdown": "### Section\nContent",
        })
        .to_string(),
        input_tokens: Some(10),
        output_tokens: Some(20),
        latency_ms: 30,
    }
}
```

- [ ] **Step 8: Add Gem execution tests**

In `execution.rs` tests or a `gem_analysis` test module, add:

```rust
#[tokio::test]
async fn gem_analysis_executes_passport_comments_and_deep_recap_in_order() {
    let pool = test_pool_with_ready_video_and_comments().await;
    let mut request = start_request("req-gem-exec-order", vec![901]);
    request.control_preset = "gem_analysis".to_string();
    request.include_comments = true;
    let run = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("start")
        .expect_started("started");

    let mut seen = Vec::new();
    execute_youtube_summary_run_with_stage_executor(&pool, run.run_id, |request| {
        seen.push(request_kind_label(&request));
        async move {
            Ok(match request {
                YoutubeSummaryStageExecutionRequest::GemAnalysisPart(part) => {
                    fake_gem_part_completion(part.part)
                }
                _ => panic!("unexpected request"),
            })
        }
    })
    .await
    .expect("execute");

    assert_eq!(seen, vec!["gem_passport", "gem_comments", "gem_deep_recap"]);
}
```

Also add tests:

- `gem_analysis_skips_comments_when_trimmed_comment_text_is_empty`
- `gem_analysis_repairs_invalid_required_part_once`
- `gem_analysis_required_part_failure_fails_stage`
- `gem_analysis_optional_comments_failure_persists_report_with_failure_note`
- `gem_analysis_does_not_start_next_part_after_cancellation_checkpoint`

- [ ] **Step 9: Run execution tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_executes
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_skips_comments
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_repairs
```

Expected: FAIL because execution branch is not implemented.

- [ ] **Step 10: Branch from normal execution using runtime-provided options**

Inside the stage loop in `execute_youtube_summary_run_with_stage_executor_with_options`, after building `input`, branch:

```rust
if input.control_preset == "gem_analysis" {
    match execute_gem_analysis_transcript_stage(
        pool,
        run_id,
        &stage,
        input,
        options.gem_input_budget,
        &mut execute_stage,
    )
    .await
    {
        Ok(()) => successes += 1,
        Err(YoutubeSummaryStageExecutionError::Cancelled) => {
            mark_transcript_stage_cancelled(pool, stage.stage_run_id).await?;
            mark_run_cancelled(pool, run_id, successes, total).await?;
            return Ok(cancelled_outcome(run_id, successes, total));
        }
        Err(YoutubeSummaryStageExecutionError::Failed(error)) => {
            failures += 1;
            mark_transcript_stage_failed(pool, run_id, stage.stage_run_id, &error.message).await?;
        }
    }
    update_run_progress(pool, run_id, successes, total).await?;
    continue;
}
```

Use the existing pending stage row fields already referenced in the loop: `stage.stage_run_id`, `stage.source_snapshot_id`, and `stage.source_ref_id`. Do not add another adapter type for this task.

- [ ] **Step 11: Implement Gem execution helper**

`execute_gem_analysis_transcript_stage` flow:

1. Verify the run has exactly one included source snapshot by counting `prompt_pack_run_source_snapshots`. This check runs inside the per-stage loop, so keep it read-only and idempotent; if a corrupted run has multiple pending transcript stages, each stage may perform the same count before failing.
2. Load Gem materials for the current stage.
3. Build prompt inputs for all parts that may run.
4. Estimate and enforce the input budget for part 1, optional part 2, and part 3 before calling `execute_stage` for any part.
5. Check `is_run_cancelled`.
6. Call `execute_stage(YoutubeSummaryStageExecutionRequest::GemAnalysisPart(request))` for part 1.
7. Parse or repair part 1.
8. Check cancellation.
9. If comments are present, run part 2; if missing, store skipped status.
10. Check cancellation.
11. Run part 3 as required.
12. Check cancellation.
13. Assemble Markdown and transcript-analysis JSON.
14. Persist through `execute_transcript_analysis_stage_with_completion_and_metrics_extension`.

Required part repair helper:

```rust
async fn run_gem_part_with_one_repair<F, Fut>(
    execute_stage: &mut F,
    request: GemAnalysisPartStageExecutionRequest,
) -> Result<(GemAnalysisPartOutput, GemPartMetrics), YoutubeSummaryStageExecutionError>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
```

When first parse fails, build `GemAnalysisPartRepairRequest` with `attempt_number: 1`. If repair fails for part 1 or part 3, return `Failed(error)`. If comments repair fails, return a comments failure status to the caller without failing the stage.

- [ ] **Step 12: Persist assembled output and metrics**

Use:

```rust
let completion = LlmCompletion {
    text: assembled_output_json,
    input_tokens: None,
    output_tokens: None,
    latency_ms: total_latency_ms,
};
execute_transcript_analysis_stage_with_completion_and_metrics_extension(
    pool,
    stage.stage_run_id,
    completion,
    Some(gem_metrics_json),
)
.await
.map_err(YoutubeSummaryStageExecutionError::Failed)?;
```

`gem_metrics_json` shape:

```rust
serde_json::json!({
    "gem_analysis": {
        "input_budget": input_budget_metrics,
        "parts": part_metrics,
        "comments_part": comments_metrics,
    }
})
```

- [ ] **Step 13: Run focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib transcript_stage_metrics_can_include_gem_analysis_extension
```

Expected: PASS.

- [ ] **Step 14: Commit Task 5**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary/gem_analysis.rs src-tauri/src/prompt_packs/youtube_summary/execution.rs src-tauri/src/prompt_packs/youtube_summary/outputs.rs src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs
git commit -m "feat(prompt-packs): execute gem analysis mini pipeline"
```

---

### Task 6: Validate Final Result Path And Run Full Focused Verification

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_validation.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/result_builder.rs` only if existing result loading rejects the assembled output.
- Modify: `src-tauri/src/prompt_packs/youtube_summary/entities_tests.rs` only if empty candidate arrays need an explicit regression test.
- Modify: `docs/value-registry.md` if any actual phase/status names changed during implementation.

**Interfaces:**
- Consumes final transcript-analysis JSON with `video_candidate.summary_text` and empty claim/evidence arrays.
- Produces existing canonical YouTube Summary result with one video section.

**Primary Scope Risk:** This task is the main integration unknown. The assembled Gem output intentionally has empty `claim_candidates` and `evidence_fragment_candidates`, and may omit nested video candidate arrays. It still passes through `validate_transcript_analysis_output`, `normalize_transcript_analysis_output_for_runtime`, `build_or_quarantine_intermediate_entities_for_transcript_stage`, result building, and canonical validation. If any layer rejects the shape, fix that layer deliberately with focused tests for `control_preset == "gem_analysis"` instead of patching the Gem output with fabricated claims/evidence.

- [ ] **Step 1: Add final path regression tests**

Add tests named:

```rust
#[tokio::test]
async fn gem_analysis_final_output_builds_canonical_single_video_result() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;
    let output = assemble_gem_analysis_transcript_output("# Gem-анализ\n\n### Section\nContent")
        .expect("assembled output");
    execute_transcript_analysis_stage_with_completion(
        &pool,
        stage_id,
        LlmCompletion {
            text: output,
            input_tokens: Some(10),
            output_tokens: Some(20),
            latency_ms: 30,
        },
    )
    .await
    .expect("persist transcript stage");

    let canonical = build_youtube_summary_canonical_result(&pool, 1)
        .await
        .expect("canonical");

    assert!(canonical["outputs"]["videos"][0]["summary_text"]
        .as_str()
        .unwrap()
        .starts_with("# Gem-анализ"));
}
```

Place this as a builder-path test near existing result builder tests. If canonical validation rejects this output, add a separate `result_validation.rs` unit test using that file's local `context("complete", "gem_analysis")` helper and update validation deliberately for `gem_analysis`.

- [ ] **Step 2: Run final path tests and inspect failures**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_final_output
```

Expected: PASS if previous tasks produce schema-compatible output. If it fails because validation requires non-empty claims/evidence for `complete`, update validation deliberately for `control_preset == "gem_analysis"` and add an assertion that empty videos are still rejected.

- [ ] **Step 3: Run frontend verification**

Run:

```powershell
npm.cmd run test -- src/lib/youtube-summary-launch-contract.test.ts src/lib/api/prompt-packs.test.ts
```

Expected: PASS.

- [ ] **Step 4: Run Rust focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib transcript_snapshot
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib browser_run_id
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib preflight_gem_analysis
```

Expected: all PASS.

- [ ] **Step 5: Run broad backend/frontend checks**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis
npm.cmd run check
```

Expected: both PASS. Browser console errors from missing Tauri IPC are irrelevant because this task does not use Playwright/browser verification.

- [ ] **Step 6: Inspect worktree and commit final integration**

Run:

```powershell
git status --short
```

Stage only files changed for this feature, leaving unrelated local files such as `.claude/settings.local.json` unstaged.

Commit:

```powershell
git add src-tauri/src/prompt_packs/youtube_summary src-tauri/src/prompt_packs/runtime.rs src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte src/lib/youtube-summary-launch-contract.test.ts docs/value-registry.md
git commit -m "feat(prompt-packs): integrate gem analysis summary mode"
```

---

## Self-Review Checklist

- Spec coverage: tasks cover UI label/value, single-video guard, neutral transcript snapshot, timing metadata, comments-only part 2, input budget, unique request/browser IDs, part JSON repair, cancellation checkpoints, optional comments failure, final assembled Markdown, metrics, value registry, and verification.
- Red-flag scan: no step uses reserved marker strings or unnamed future helpers; helper names are defined before use.
- Type consistency: `GemAnalysisPart`, `GemAnalysisInputBudget`, `YoutubeSummaryExecutionOptions`, `GemAnalysisPartStageExecutionRequest`, `GemAnalysisPartRepairRequest`, and `GemAnalysisPartOutput` are introduced before execution tasks use them.
- Safety check: the unbounded 3-argument executor wrapper and `unbounded_for_tests()` are `#[cfg(test)]`; production must pass runtime-computed options through `execute_youtube_summary_run_with_stage_executor_with_options`.
- Risk check: Task 6 is explicitly marked as the main integration scope risk because validation, normalization, intermediate entity building, and result building may reject empty candidates or omitted nested arrays.
- Scope: the plan stays inside existing YouTube Summary prompt pack and does not introduce a separate pack, new artifact kinds, web-search stage, or multi-video Gem synthesis.
