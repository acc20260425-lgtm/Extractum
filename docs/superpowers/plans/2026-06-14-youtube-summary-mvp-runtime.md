# YouTube Summary Prompt Pack Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the YouTube Summary run runtime up to stage skeleton creation, without executing LLM calls.

**Architecture:** Keep runtime orchestration inside `src-tauri/src/prompt_packs`. `preflight_youtube_summary_run` is a preview command. `start_youtube_summary_run` recomputes preflight, freezes snapshots, creates stage rows, registers active state, emits lifecycle events, and then stops at `not_implemented` execution until the execution/result plan is applied.

**Tech Stack:** Rust/Tauri 2, SQLite via `sqlx`, existing YouTube Library tables, existing LLM profile/model limit helpers, Tauri events, Svelte-friendly DTOs.

---

## Dependencies

Complete `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-foundation.md` first.

---

## File Structure

- Modify `src-tauri/src/prompt_packs/mod.rs`: expose runtime commands and state.
- Create `src-tauri/src/prompt_packs/dto.rs`: preflight, start, run, stage, and event DTOs.
- Create `src-tauri/src/prompt_packs/runtime.rs`: active run state, cancellation, events, active list, startup cleanup.
- Create `src-tauri/src/prompt_packs/youtube_summary.rs`: source expansion, preflight partitions, deterministic snapshot creation, and stage skeleton creation.
- Modify `src-tauri/src/prompt_packs/store.rs`: run/scope/source/material/stage insert and query helpers.
- Modify `src-tauri/src/lib.rs`: manage `PromptPackRunState`, register commands, call cleanup on startup.
- Modify `src/lib/types/prompt-packs.ts`: frontend DTO types for runtime commands.
- Modify `src/lib/api/prompt-packs.ts`: command wrappers and event listener.
- Modify `src/lib/api/prompt-packs.test.ts`: wrapper tests.

---

## Task 1: Runtime DTOs and Event Contract

**Files:**
- Create/modify: `src-tauri/src/prompt_packs/dto.rs`
- Modify: `src-tauri/src/prompt_packs/mod.rs`
- Modify: `src/lib/types/prompt-packs.ts`

- [ ] **Step 1: Define Rust DTOs**

Create DTOs matching the approved spec:

```rust
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightYoutubeSummaryRunRequest {
    pub project_id: Option<i64>,
    pub source_ids: Vec<i64>,
    pub profile_id: Option<String>,
    pub model_override: Option<String>,
    pub output_language: String,
    pub control_preset: String,
    pub evidence_mode: String,
    pub include_comments: bool,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSummaryPreflightResponse {
    pub pack_id: String,
    pub pack_version: String,
    pub included_videos: Vec<YoutubeSummaryPreflightVideo>,
    pub skipped_videos: Vec<YoutubeSummaryPreflightSkippedVideo>,
    pub blocking_failures: Vec<YoutubeSummaryPreflightFailure>,
    pub estimated_input_tokens: i64,
    pub selected_model_input_limit: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackRunEvent {
    pub run_id: i64,
    pub event_kind: String,
    pub run_status: String,
    pub stage_name: Option<String>,
    pub source_ref_id: Option<String>,
    pub message: Option<String>,
}
```

- [ ] **Step 2: Define TypeScript DTOs**

Mirror the Rust DTOs in `src/lib/types/prompt-packs.ts`. Use string unions for statuses:

```ts
export type PromptPackRunStatus =
  | "queued"
  | "running"
  | "completed"
  | "partial"
  | "failed"
  | "cancel_requested"
  | "cancelled";
```

- [ ] **Step 3: Commit**

```powershell
git add src-tauri/src/prompt_packs/dto.rs src-tauri/src/prompt_packs/mod.rs src/lib/types/prompt-packs.ts
git commit -m "feat: add prompt pack runtime dto contracts"
```

---

## Task 2: YouTube Summary Preflight

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`
- Modify: `src-tauri/src/prompt_packs/store.rs`

- [ ] **Step 1: Write preflight tests**

Add tests in `youtube_summary.rs` using an in-memory migrated DB:

```rust
#[tokio::test]
async fn preflight_explicit_video_without_transcript_is_blocking_failure() {
    let pool = test_pool_with_youtube_video_without_transcript().await;

    let response = preflight_youtube_summary_in_pool(
        &pool,
        PreflightYoutubeSummaryRunRequest {
            project_id: None,
            source_ids: vec![901],
            profile_id: None,
            model_override: Some("test-model".to_string()),
            output_language: "en".to_string(),
            control_preset: "standard".to_string(),
            evidence_mode: "standard".to_string(),
            include_comments: false,
        },
        ModelBudget { input_token_limit: Some(32_000) },
    )
    .await
    .expect("preflight");

    assert!(response.included_videos.is_empty());
    assert_eq!(response.blocking_failures[0].reason, "no_usable_transcript");
}

#[tokio::test]
async fn preflight_playlist_video_without_transcript_is_skipped() {
    let pool = test_pool_with_playlist_one_ready_one_missing_transcript().await;

    let response = preflight_youtube_summary_in_pool(
        &pool,
        request_for_playlist(701),
        ModelBudget { input_token_limit: Some(32_000) },
    )
    .await
    .expect("preflight");

    assert_eq!(response.included_videos.len(), 1);
    assert_eq!(response.skipped_videos[0].reason, "no_usable_transcript");
    assert!(response.blocking_failures.is_empty());
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::youtube_summary::tests::preflight_
```

Expected: fail because preflight implementation does not exist.

- [ ] **Step 3: Implement source expansion**

Rules:

- accept only `sources.source_type = 'youtube'`;
- explicit video source must have `source_subtype = 'video'`;
- playlist source must have `source_subtype = 'playlist'`;
- playlist expands through non-removed `youtube_playlist_items` ordered by `position ASC`, then `id ASC`;
- linked playlist items use `video_source_id`;
- unlinked playlist items become `skipped_videos` with `reason = "unlinked_playlist_item"`;
- an explicit invalid source becomes `blocking_failures`.

- [ ] **Step 4: Implement transcript and budget checks**

Rules:

- included videos require at least one transcript segment in `youtube_transcript_segments`;
- preflight does not use description or comments as transcript fallback;
- estimate input tokens from transcript text, description, optional comments, and fixed stage overhead;
- if a playlist video exceeds the selected model budget, it becomes `skipped_videos` with `reason = "input_budget_exceeded"`;
- if an explicit video exceeds the selected model budget, it becomes `blocking_failures` with `reason = "input_budget_exceeded"`;
- start is allowed only when `included_videos` is non-empty and `blocking_failures` is empty.

- [ ] **Step 5: Run preflight tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::youtube_summary
```

Expected: pass.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary.rs src-tauri/src/prompt_packs/store.rs
git commit -m "feat: add youtube summary preflight"
```

---

## Task 3: Snapshot Freeze and Stage Skeleton

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`
- Modify: `src-tauri/src/prompt_packs/store.rs`

- [ ] **Step 1: Write snapshot tests**

Add tests:

```rust
#[tokio::test]
async fn start_freezes_one_canonical_video_snapshot_with_multiple_origins() {
    let pool = test_pool_with_same_video_selected_explicitly_and_from_playlist().await;
    let request = request_for_video_and_playlist(901, 701);

    let run_id = create_youtube_summary_run_skeleton_in_pool(&pool, request, test_pack_version())
        .await
        .expect("create run");

    let snapshot_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_run_source_snapshots WHERE run_id = ?",
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("snapshot count");

    let origin_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_run_source_origins WHERE run_id = ?",
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("origin count");

    assert_eq!(snapshot_count, 1);
    assert_eq!(origin_count, 2);
}

#[tokio::test]
async fn comment_snapshot_selection_is_deterministic_when_enabled() {
    let pool = test_pool_with_comments_out_of_order().await;

    let first = freeze_comment_material_refs(&pool, 901, test_comment_policy())
        .await
        .expect("first freeze");
    let second = freeze_comment_material_refs(&pool, 901, test_comment_policy())
        .await
        .expect("second freeze");

    assert_eq!(first, second);
    assert_eq!(first[0].external_id.as_deref(), Some("comment-oldest"));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::youtube_summary::tests::start_freezes
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::youtube_summary::tests::comment_snapshot
```

Expected: fail because snapshot creation does not exist.

- [ ] **Step 3: Implement run creation transaction**

Inside one DB transaction:

- insert `prompt_pack_runs` with `run_status = 'queued'`;
- insert `prompt_pack_run_scopes` for selected video/playlist scopes;
- insert canonical video snapshots unique by `(run_id, video_id)`;
- insert source origins unique by `(run_id, origin_scope_id, video_id)`;
- insert transcript, description, and optional comment material snapshots;
- insert stage skeleton rows:
  - `source_ingestion`;
  - `youtube_summary/transcript_analysis` once per included video;
  - skipped MVP rows for `segment_extraction`, `key_point_extraction`, `quote_extraction`;
  - `youtube_summary/synthesis` as `skipped` or `not_implemented`;
  - `final_synthesis`;
  - `validation`.

- [ ] **Step 4: Implement deterministic comments**

Use the policy from the approved spec:

- default `comment_count_cap = 50`;
- default `comment_budget_ratio = 0.15`;
- default `comment_token_cap = 4000`;
- stable order: `published_at IS NULL ASC`, `published_at ASC`, `external_id ASC`, `items.id ASC`;
- truncate each included comment by token estimate and write truncation metadata;
- write excluded/truncated counts into snapshot metadata and stage input metadata.

- [ ] **Step 5: Run snapshot tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::youtube_summary
```

Expected: pass.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/prompt_packs/youtube_summary.rs src-tauri/src/prompt_packs/store.rs
git commit -m "feat: freeze youtube summary run snapshots"
```

---

## Task 4: Runtime State, Events, Commands, and Cancel

**Files:**
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Modify: `src-tauri/src/prompt_packs/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/api/prompt-packs.ts`
- Modify: `src/lib/api/prompt-packs.test.ts`

- [ ] **Step 1: Add runtime state tests**

Add tests in `runtime.rs`:

```rust
#[tokio::test]
async fn prompt_pack_run_state_tracks_active_and_cancel_requested_runs() {
    let state = PromptPackRunState::new();

    state.track(42).await.expect("track");
    assert!(state.active_run_ids().await.contains(&42));

    state.request_cancel(42).await.expect("cancel");
    assert!(state.is_cancel_requested(42).await);

    state.finish(42).await;
    assert!(!state.active_run_ids().await.contains(&42));
}
```

- [ ] **Step 2: Implement state**

Implement `PromptPackRunState` with:

- active run ids;
- cancel-requested run ids;
- duplicate prevention per run id;
- `list_active_prompt_pack_runs` support.

- [ ] **Step 3: Implement backend commands**

Expose:

```rust
#[tauri::command]
pub async fn preflight_youtube_summary_run(...) -> AppResult<YoutubeSummaryPreflightResponse>

#[tauri::command]
pub async fn start_youtube_summary_run(...) -> AppResult<PromptPackRunSummaryDto>

#[tauri::command]
pub async fn cancel_prompt_pack_run(...) -> AppResult<()>

#[tauri::command]
pub async fn list_active_prompt_pack_runs(...) -> AppResult<Vec<PromptPackRunSummaryDto>>

#[tauri::command]
pub async fn list_prompt_pack_run_stages(...) -> AppResult<Vec<PromptPackStageRunDto>>
```

`start_youtube_summary_run` must recompute preflight from current DB state and store the recomputed response in `prompt_pack_runs.preflight_json_zstd`.

- [ ] **Step 4: Emit lifecycle events**

Define:

```rust
pub const PROMPT_PACK_RUN_EVENT: &str = "prompt-pack://run";
```

Emit events for `queued`, `started`, `progress`, `cancel_requested`, `cancelled`, `failed`, `completed`, and `partial`. The runtime plan can emit `failed` with `status_reason = "execution_not_implemented"` after skeleton creation until the execution/result plan replaces this behavior.

- [ ] **Step 5: Register commands and state**

In `src-tauri/src/lib.rs`:

- `mod prompt_packs;`
- `.manage(PromptPackRunState::new())`
- startup cleanup for interrupted active prompt-pack runs;
- add commands to `tauri::generate_handler!`.

- [ ] **Step 6: Add frontend wrappers**

In `src/lib/api/prompt-packs.ts` expose:

```ts
export const PROMPT_PACK_RUN_EVENT = "prompt-pack://run";

export function preflightYoutubeSummaryRun(input: PreflightYoutubeSummaryRunInput) {
  return invoke<YoutubeSummaryPreflightResponse>("preflight_youtube_summary_run", { ...input });
}

export function startYoutubeSummaryRun(input: StartYoutubeSummaryRunInput) {
  return invoke<PromptPackRunSummary>("start_youtube_summary_run", { ...input });
}

export function cancelPromptPackRun(runId: number) {
  return invoke<void>("cancel_prompt_pack_run", { runId });
}
```

- [ ] **Step 7: Run runtime tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs
npm test -- --run src/lib/api/prompt-packs.test.ts
```

Expected: pass.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri/src/prompt_packs src-tauri/src/lib.rs src/lib/api/prompt-packs.ts src/lib/api/prompt-packs.test.ts src/lib/types/prompt-packs.ts
git commit -m "feat: add prompt pack run runtime"
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

- preflight partitions behave as specified;
- snapshots are deterministic and run-local;
- stage skeleton rows are created;
- active run/cancel state works;
- frontend wrappers call the expected Tauri commands.
