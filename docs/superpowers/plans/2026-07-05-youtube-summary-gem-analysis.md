# YouTube Summary Gem Analysis Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the approved single-video `Gem analysis` YouTube Summary mode that runs independent transcript/comment/transcript Gem parts and assembles one Markdown report into `video_candidate.summary_text`.

**Architecture:** Keep `gem_analysis` inside the existing YouTube Summary prompt pack as a `control_preset`. Freeze transcript material from bounded ordered transcript segments as the single source of truth, then use a focused Gem execution helper to build per-part prompts, call the selected runtime, repair part JSON once, and persist one normal transcript-analysis output. Existing `standard` and `detailed_report` flows stay on the current single-completion path.

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
- Cancellation checkpoints are required before part 1, before part 2, before part 3, and before final persistence.
- v1 has no partial per-part result cache; a required part failure makes a retry rerun all parts.
- No new `artifact_kind` values in this slice.
- Update `docs/value-registry.md` for new `control_preset` and any new event `phase` values.
- Use `npm.cmd` for npm scripts on Windows.

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
- `src-tauri/src/prompt_packs/runtime.rs` builds/runs Gem part and repair requests, optional browser discriminators, output budgets, and part prompt wrappers.
- `src-tauri/src/prompt_packs/youtube_summary/gem_analysis.rs` is a new focused orchestrator for material loading, prompt input assembly, input-budget checks, part output parsing/repair decisions, final Markdown assembly, and final transcript-analysis JSON assembly.
- `src-tauri/src/prompt_packs/youtube_summary/execution.rs` branches to `execute_gem_analysis_transcript_stage` when `control_preset == "gem_analysis"`.
- `src-tauri/src/prompt_packs/youtube_summary/outputs.rs` accepts an optional metrics extension when persisting assembled Gem output.
- Existing Rust test files in `youtube_summary` and `runtime.rs` receive focused tests near the code they verify.

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

- [ ] **Step 4: Add backend preflight tests**

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

If `test_pool_with_playlist_two_ready_videos` does not exist, add it to `test_support.rs` by composing two ready linked playlist children with transcript rows, following the existing playlist helpers.

- [ ] **Step 5: Run preflight tests and verify the new multi-video test fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib preflight_gem_analysis
```

Expected: FAIL because the guard is not implemented.

- [ ] **Step 6: Implement the preflight guard**

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

- [ ] **Step 7: Update value registry**

In `docs/value-registry.md`, update the `Prompt-pack control preset` row to include:

```text
`standard`, `detailed_report`, `gem_analysis`
```

Mention `gem_analysis` is single-video and sequential multi-request.

- [ ] **Step 8: Run focused verification**

Run:

```powershell
npm.cmd run test -- src/lib/youtube-summary-launch-contract.test.ts
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib preflight_gem_analysis
```

Expected: both PASS.

- [ ] **Step 9: Commit Task 1**

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
    insert_empty_comment_material_snapshot_for_test(&pool, 1).await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

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
    "task": passport_prompt_body()
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
    "task": comments_prompt_body()
})
```

Do not include transcript in comments input. Do not include comments in transcript inputs.

- [ ] **Step 6: Implement input budget helpers**

Add:

```rust
const GEM_INPUT_ESTIMATOR_OVERHEAD_TOKENS: i64 = 1_500;
const GEM_BROWSER_INPUT_TOKEN_CAP: i64 = 24_000;

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

The cap source is supplied by execution/runtime in Task 5. For Task 4, unit-test the pure helper with a small cap.

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
- Produces persistence helper `execute_transcript_analysis_stage_with_completion_and_metrics_extension(...)`.
- Consumes `YoutubeSummaryStageExecutionRequest::GemAnalysisPart` and `GemAnalysisPartRepair`.

- [ ] **Step 1: Add output persistence test for metrics extension**

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

Add `load_stage_artifact_json` as a test helper if no equivalent exists.

- [ ] **Step 2: Implement metrics extension persistence**

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

Before writing metrics, merge object fields from `metrics_extension` into the base metrics object. Reject non-object extensions with `AppError::internal("metrics extension must be a JSON object")`.

- [ ] **Step 3: Add Gem execution tests**

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

Implement test helpers:

```rust
fn fake_gem_part_completion(part: GemAnalysisPart) -> LlmCompletion {
    LlmCompletion {
        text: format!(
            r#"{{"part":"{}","markdown":"### Section\nContent"}}"#,
            part.as_str()
        ),
        input_tokens: Some(10),
        output_tokens: Some(20),
        latency_ms: 30,
    }
}
```

Also add tests:

- `gem_analysis_skips_comments_when_trimmed_comment_text_is_empty`
- `gem_analysis_repairs_invalid_required_part_once`
- `gem_analysis_required_part_failure_fails_stage`
- `gem_analysis_optional_comments_failure_persists_report_with_failure_note`
- `gem_analysis_does_not_start_next_part_after_cancellation_checkpoint`

- [ ] **Step 4: Run execution tests and verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_executes
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_skips_comments
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_repairs
```

Expected: FAIL because execution branch is not implemented.

- [ ] **Step 5: Branch from normal execution**

In `execute_youtube_summary_run_with_stage_executor_internal`, after building `input`, branch:

```rust
if input.control_preset == "gem_analysis" {
    match execute_gem_analysis_transcript_stage(
        pool,
        run_id,
        &stage,
        input,
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

Use the actual pending stage row type from `transcript_execution.rs`; expose fields through an existing or new crate-private struct if needed.

- [ ] **Step 6: Implement Gem execution helper**

`execute_gem_analysis_transcript_stage` flow:

1. Verify the run has exactly one included source snapshot by counting `prompt_pack_run_source_snapshots`.
2. Load Gem materials for the current stage.
3. Build part 1 prompt input and enforce input budget.
4. Check `is_run_cancelled`.
5. Call `execute_stage(YoutubeSummaryStageExecutionRequest::GemAnalysisPart(request))`.
6. Parse or repair part 1.
7. Check cancellation.
8. If comments are present, run part 2; if missing, store skipped status.
9. Check cancellation.
10. Run part 3 as required.
11. Check cancellation.
12. Assemble Markdown and transcript-analysis JSON.
13. Persist through `execute_transcript_analysis_stage_with_completion_and_metrics_extension`.

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

- [ ] **Step 7: Persist assembled output and metrics**

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

- [ ] **Step 8: Run focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib gem_analysis_
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gem-analysis --lib transcript_stage_metrics_can_include_gem_analysis_extension
```

Expected: PASS.

- [ ] **Step 9: Commit Task 5**

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

- [ ] **Step 1: Add final path regression tests**

Add tests named:

```rust
#[tokio::test]
async fn gem_analysis_final_output_builds_canonical_single_video_result() {
    let pool = test_pool_with_ready_video().await;
    let run_id = seed_completed_gem_analysis_run_for_test(&pool).await;

    let canonical = build_youtube_summary_canonical_result(&pool, run_id)
        .await
        .expect("canonical");
    validate_youtube_summary_canonical_result(
        &canonical,
        &context("complete", "gem_analysis"),
    )
    .expect("valid canonical");

    assert!(canonical["outputs"]["videos"][0]["summary_text"]
        .as_str()
        .unwrap()
        .starts_with("# Gem-анализ"));
}
```

Use existing helper names where available. If `context` is test-local in `result_validation.rs`, place the validation test in that module and a builder-only test in `result_builder` tests.

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
- Type consistency: `GemAnalysisPart`, `GemAnalysisPartStageExecutionRequest`, `GemAnalysisPartRepairRequest`, and `GemAnalysisPartOutput` are introduced before execution tasks use them.
- Scope: the plan stays inside existing YouTube Summary prompt pack and does not introduce a separate pack, new artifact kinds, web-search stage, or multi-video Gem synthesis.
