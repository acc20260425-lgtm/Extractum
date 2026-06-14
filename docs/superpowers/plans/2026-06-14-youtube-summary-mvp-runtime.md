# YouTube Summary Prompt Pack Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the YouTube Summary run runtime up to stage skeleton creation, without executing LLM calls.

**Architecture:** Keep runtime orchestration inside `src-tauri/src/prompt_packs`. `preflight_youtube_summary_run` is a preview command. `start_youtube_summary_run` recomputes preflight, freezes snapshots, creates stage rows, registers active state, emits lifecycle events, and leaves execution-owned stages pending until the execution/result plan is applied.

**Tech Stack:** Rust/Tauri 2, SQLite via `sqlx`, existing YouTube Library tables, existing LLM profile/model limit helpers, Tauri events, Svelte-friendly DTOs.

---

## Dependencies

Complete `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-foundation.md` first.

---

## File Structure

- Modify `src-tauri/src/prompt_packs/mod.rs`: expose runtime commands and state.
- Create `src-tauri/migrations/0007_prompt_pack_run_idempotency.sql`: add
  `prompt_pack_runs.client_request_id TEXT` with a partial unique index for
  non-null values.
- Create `src-tauri/src/prompt_packs/dto.rs`: preflight, start, run, stage, and event DTOs.
- Create `src-tauri/src/prompt_packs/runtime.rs`: active run state, cancellation, events, active list, startup cleanup.
- Create `src-tauri/src/prompt_packs/youtube_summary.rs`: source expansion, preflight partitions, deterministic snapshot creation, and stage skeleton creation.
- Modify `src-tauri/src/prompt_packs/store.rs`: run/scope/source/material/stage insert and query helpers.
- Modify `src-tauri/src/lib.rs`: manage `PromptPackRunState`, register commands, call cleanup on startup.
- Modify `src/lib/types/prompt-packs.ts`: frontend DTO types for runtime commands.
- Modify `src/lib/api/prompt-packs.ts`: command wrappers and event listener.
- Modify `src/lib/api/prompt-packs.test.ts`: wrapper tests.

---

## Task 0: Run Idempotency Migration

**Files:**
- Create: `src-tauri/migrations/0007_prompt_pack_run_idempotency.sql`
- Modify: `src-tauri/src/prompt_packs/store.rs`

- [ ] **Step 1: Write migration constraint tests**

Add tests:

```rust
#[tokio::test]
async fn prompt_pack_runs_client_request_id_is_unique_when_present() {
    let pool = test_pool_with_prompt_pack_schema().await;

    insert_minimal_prompt_pack_run(&pool, 41, Some("req-duplicate"))
        .await
        .expect("first run");
    let duplicate = insert_minimal_prompt_pack_run(&pool, 42, Some("req-duplicate"))
        .await
        .expect_err("duplicate request id rejected");

    assert!(duplicate.to_string().contains("client_request_id"));
}

#[tokio::test]
async fn prompt_pack_runs_allow_null_client_request_id_for_pre_existing_rows() {
    let pool = test_pool_with_prompt_pack_schema().await;

    insert_minimal_prompt_pack_run(&pool, 41, None)
        .await
        .expect("first legacy-compatible run");
    insert_minimal_prompt_pack_run(&pool, 42, None)
        .await
        .expect("second legacy-compatible run");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_runs WHERE client_request_id IS NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("null request ids");

    assert_eq!(count, 2);
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::store::tests::prompt_pack_runs_client_request_id
```

Expected: fail because `client_request_id` does not exist.

- [ ] **Step 3: Add migration**

Create `src-tauri/migrations/0007_prompt_pack_run_idempotency.sql`:

```sql
ALTER TABLE prompt_pack_runs
ADD COLUMN client_request_id TEXT
CHECK (client_request_id IS NULL OR length(trim(client_request_id)) > 0);

CREATE UNIQUE INDEX IF NOT EXISTS idx_prompt_pack_runs_client_request_id_unique
ON prompt_pack_runs(client_request_id)
WHERE client_request_id IS NOT NULL;
```

- [ ] **Step 4: Run migration tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib prompt_packs::store::tests::prompt_pack_runs_client_request_id
```

Expected: pass.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/migrations/0007_prompt_pack_run_idempotency.sql src-tauri/src/prompt_packs/store.rs
git commit -m "feat: add prompt pack run idempotency migration"
```

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

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartYoutubeSummaryRunRequest {
    pub client_request_id: String,
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
    pub request_id: String,
    pub kind: String,
    pub run_status: String,
    pub phase: String,
    pub stage_run_id: Option<i64>,
    pub stage_name: Option<String>,
    pub source_snapshot_id: Option<i64>,
    pub queue_position: Option<i64>,
    pub progress_current: Option<i64>,
    pub progress_total: Option<i64>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPromptPackRunsRequest {
    pub project_id: Option<i64>,
    pub limit: Option<i64>,
}
```

- [ ] **Step 2: Define TypeScript DTOs**

Mirror the Rust DTOs in `src/lib/types/prompt-packs.ts`. Use string unions for statuses:

```ts
export type PromptPackRunStatus =
  | "queued"
  | "running"
  | "complete"
  | "partial"
  | "failed"
  | "cancelled"
  | "interrupted";

export type PromptPackRunEventKind =
  | "queued"
  | "started"
  | "progress"
  | "stage_started"
  | "stage_completed"
  | "stage_failed"
  | "completed"
  | "partial"
  | "failed"
  | "cancelled"
  | "interrupted";

export type PromptPackRunEventPhase =
  | "preflight"
  | "snapshot"
  | "stage"
  | "validation"
  | "projection"
  | "persist"
  | "terminal";

export interface StartYoutubeSummaryRunInput {
  clientRequestId: string;
  projectId: number | null;
  sourceIds: number[];
  profileId: string | null;
  modelOverride: string | null;
  outputLanguage: string;
  controlPreset: string;
  evidenceMode: string;
  includeComments: boolean;
}

export type StartYoutubeSummaryRunOutcome =
  | { kind: "started"; run: PromptPackRunSummary }
  | { kind: "blocked"; preflight: YoutubeSummaryPreflightResponse };

export interface ListPromptPackRunsInput {
  projectId?: number | null;
  limit?: number;
}
```

`cancel_requested` is an in-memory runtime flag and may appear in message text,
but it is not a persisted `run_status` and not a `PromptPackRunEvent.kind`.

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
    let request = request_for_video_and_playlist("req-freeze-1", 901, 701);

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
async fn start_returns_existing_run_for_duplicate_client_request_id() {
    let pool = test_pool_with_ready_video().await;
    let request = request_for_video("req-duplicate-start", 901);

    let first = start_youtube_summary_run_in_pool(&pool, request.clone())
        .await
        .expect("first start")
        .expect_started("first start returns a run");
    let second = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("duplicate start")
        .expect_started("duplicate start returns existing run");

    assert_eq!(first.run_id, second.run_id);

    let run_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_runs WHERE client_request_id = 'req-duplicate-start'",
    )
    .fetch_one(&pool)
    .await
    .expect("run count");
    assert_eq!(run_count, 1);
}

#[tokio::test]
async fn start_with_recomputed_blocking_preflight_returns_response_without_run() {
    let pool = test_pool_with_youtube_video_without_transcript().await;
    let request = request_for_video("req-blocked-start", 901);

    let outcome = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("start command returns structured blocking response");

    let blocking = outcome.expect_blocked("blocking response");
    assert!(blocking.included_videos.is_empty());
    assert_eq!(blocking.blocking_failures[0].reason, "no_usable_transcript");

    let run_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_runs WHERE client_request_id = 'req-blocked-start'",
    )
    .fetch_one(&pool)
    .await
    .expect("run count");
    assert_eq!(run_count, 0);
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

- reject an empty or whitespace-only `client_request_id`;
- if a run already exists for `client_request_id`, return that run summary
  without creating new rows;
- recompute preflight from current Library rows before inserting any run row;
- if recomputed preflight has `blocking_failures` or no `included_videos`,
  return a structured blocked-start response containing the fresh preflight and
  do not insert `prompt_pack_runs`;
- insert `prompt_pack_runs` with `run_status = 'queued'`;
- persist `client_request_id` with a unique DB constraint so app retry and
  double-clicks are idempotent across process restarts;
- insert `prompt_pack_run_scopes` for selected video/playlist scopes;
- insert canonical video snapshots unique by `(run_id, video_id)`;
- insert source origins unique by `(run_id, origin_scope_id, video_id)`;
- insert transcript, description, and optional comment material snapshots;
- insert stage skeleton rows:
  - `source_ingestion`;
  - `youtube_summary/transcript_analysis` once per included video;
  - skipped MVP rows for `segment_extraction`, `key_point_extraction`, `quote_extraction`;
  - `youtube_summary/synthesis` as `skipped` or `not_implemented`;
  - `final_synthesis` as `pending`;
  - `validation` as `pending`.

Runtime-only handoff rule:

- after skeleton creation, leave the run `queued` or `running` and leave
  execution-owned stage rows `pending` or `not_implemented`;
- do not mark the run `failed` with `execution_not_implemented`;
- the execution/result plan is responsible for launching LLM work and terminal
  transitions.

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
- Modify: `src-tauri/src/prompt_packs/store.rs`
- Modify: `src-tauri/src/prompt_packs/dto.rs`
- Modify: `src-tauri/src/prompt_packs/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/types/prompt-packs.ts`
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

#[tokio::test]
async fn terminal_event_removes_run_from_active_state() {
    let state = PromptPackRunState::new();

    state.track(42).await.expect("track");
    state.apply_event(PromptPackRunEvent {
        run_id: 42,
        request_id: "req-42".to_string(),
        kind: "completed".to_string(),
        run_status: "complete".to_string(),
        phase: "terminal".to_string(),
        stage_run_id: None,
        stage_name: None,
        source_snapshot_id: None,
        queue_position: None,
        progress_current: Some(1),
        progress_total: Some(1),
        message: Some("Completed".to_string()),
        error: None,
    })
    .await;

    assert!(!state.active_run_ids().await.contains(&42));
}

#[tokio::test]
async fn cleanup_interrupted_prompt_pack_runs_marks_stale_active_rows_interrupted() {
    let pool = test_pool_with_prompt_pack_runs([
        (41, "queued"),
        (42, "running"),
        (43, "complete"),
    ])
    .await;
    let state = PromptPackRunState::new();

    cleanup_interrupted_prompt_pack_runs_in_pool(&pool, &state)
        .await
        .expect("cleanup");

    let statuses = list_run_statuses(&pool).await;
    assert_eq!(statuses.get(&41).map(String::as_str), Some("interrupted"));
    assert_eq!(statuses.get(&42).map(String::as_str), Some("interrupted"));
    assert_eq!(statuses.get(&43).map(String::as_str), Some("complete"));
}

#[tokio::test]
async fn list_prompt_pack_runs_returns_recent_runs_for_project() {
    let pool = test_pool_with_prompt_pack_runs([
        (41, Some(7), "complete", "2026-06-14T10:00:00Z"),
        (42, Some(7), "running", "2026-06-14T11:00:00Z"),
        (43, Some(8), "complete", "2026-06-14T12:00:00Z"),
    ])
    .await;

    let runs = list_prompt_pack_runs_in_pool(
        &pool,
        ListPromptPackRunsRequest {
            project_id: Some(7),
            limit: Some(20),
        },
    )
    .await
    .expect("recent runs");

    assert_eq!(runs.iter().map(|run| run.run_id).collect::<Vec<_>>(), vec![42, 41]);
    assert!(runs.iter().all(|run| run.project_id == Some(7)));
}
```

- [ ] **Step 2: Implement state**

Implement `PromptPackRunState` with:

- active run ids;
- cancel-requested run ids;
- duplicate prevention per run id;
- terminal event cleanup for `completed`, `partial`, `failed`, `cancelled`, and
  `interrupted` event kinds;
- `list_active_prompt_pack_runs` support;
- `list_prompt_pack_runs` support for recent terminal and active runs.

- [ ] **Step 3: Implement backend commands**

Expose:

```rust
#[tauri::command]
pub async fn preflight_youtube_summary_run(...) -> AppResult<YoutubeSummaryPreflightResponse>

#[tauri::command]
pub async fn start_youtube_summary_run(...) -> AppResult<StartYoutubeSummaryRunOutcomeDto>

#[tauri::command]
pub async fn cancel_prompt_pack_run(...) -> AppResult<()>

#[tauri::command]
pub async fn list_prompt_pack_runs(...) -> AppResult<Vec<PromptPackRunSummaryDto>>

#[tauri::command]
pub async fn list_active_prompt_pack_runs(...) -> AppResult<Vec<PromptPackRunSummaryDto>>

#[tauri::command]
pub async fn list_prompt_pack_run_stages(...) -> AppResult<Vec<PromptPackStageRunDto>>
```

`list_prompt_pack_runs` rules:

- accepts optional `project_id` and optional `limit`;
- default `limit = 20`, maximum `limit = 100`;
- returns active and terminal Prompt Pack runs, never legacy `analysis_runs`;
- orders by `created_at DESC, id DESC`;
- includes enough summary fields for the UI runs panel: `run_id`, `project_id`,
  `pack_id`, `pack_version`, `run_status`, `result_status`, `created_at`,
  `started_at`, `completed_at`, `latest_message`, `progress_current`,
  `progress_total`, and `queue_position`.

`StartYoutubeSummaryRunOutcomeDto` must be a tagged response:

```rust
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum StartYoutubeSummaryRunOutcomeDto {
    Started { run: PromptPackRunSummaryDto },
    Blocked { preflight: YoutubeSummaryPreflightResponse },
}
```

`start_youtube_summary_run` rules:

- accepts `client_request_id`;
- validates `client_request_id` is not empty;
- returns an existing `Started` run when the same `client_request_id` was
  already persisted;
- recomputes preflight from current DB state before creating snapshots;
- if recomputed preflight has blocking failures or no included videos, returns
  `Blocked { preflight }` and creates no run row;
- if recomputed preflight can start, stores the recomputed response in
  `prompt_pack_runs.preflight_json_zstd`.

- [ ] **Step 4: Emit lifecycle events**

Define:

```rust
pub const PROMPT_PACK_RUN_EVENT: &str = "prompt-pack-run-event";
```

Emit events using the approved `PromptPackRunEvent` shape:

- `request_id` is the `client_request_id` for start-created runs or a generated
  internal id for cleanup/interruption events;
- `kind`: `queued`, `started`, `progress`, `stage_started`,
  `stage_completed`, `stage_failed`, `completed`, `partial`, `failed`,
  `cancelled`, or `interrupted`;
- `run_status`: `queued`, `running`, `complete`, `partial`, `failed`,
  `cancelled`, or `interrupted`;
- `phase`: `preflight`, `snapshot`, `stage`, `validation`, `projection`,
  `persist`, or `terminal`;
- include `stage_run_id` and `stage_name` on stage events;
- include `source_snapshot_id` for video-scoped stage events;
- include `queue_position`, `progress_current`, and `progress_total` when known;
- include `error` only for failed/interrupted paths.

Terminal events are `completed`, `partial`, `failed`, `cancelled`, and
`interrupted`; after emitting a terminal event, remove the run from active state.
Do not emit a terminal `failed` event only because execution has not been
implemented in this runtime slice.

- [ ] **Step 5: Register commands and state**

In `src-tauri/src/lib.rs`:

- `mod prompt_packs;`
- `.manage(PromptPackRunState::new())`
- startup cleanup for interrupted active prompt-pack runs;
- add commands to `tauri::generate_handler!`.

- [ ] **Step 6: Add frontend wrappers**

In `src/lib/api/prompt-packs.ts` expose:

```ts
export const PROMPT_PACK_RUN_EVENT = "prompt-pack-run-event";

export function preflightYoutubeSummaryRun(input: PreflightYoutubeSummaryRunInput) {
  return invoke<YoutubeSummaryPreflightResponse>("preflight_youtube_summary_run", { ...input });
}

export function startYoutubeSummaryRun(input: StartYoutubeSummaryRunInput) {
  return invoke<StartYoutubeSummaryRunOutcome>("start_youtube_summary_run", { ...input });
}

export function cancelPromptPackRun(runId: number) {
  return invoke<void>("cancel_prompt_pack_run", { runId });
}

export function listPromptPackRuns(input?: ListPromptPackRunsInput) {
  return invoke<PromptPackRunSummary[]>("list_prompt_pack_runs", { ...input });
}
```

Extend `src/lib/api/prompt-packs.test.ts` so it verifies:

```ts
await startYoutubeSummaryRun({
  clientRequestId: "req-ui-start-1",
  projectId: null,
  sourceIds: [901],
  profileId: null,
  modelOverride: null,
  outputLanguage: "en",
  controlPreset: "standard",
  evidenceMode: "standard",
  includeComments: false,
});

expect(invoke).toHaveBeenCalledWith("start_youtube_summary_run", {
  clientRequestId: "req-ui-start-1",
  projectId: null,
  sourceIds: [901],
  profileId: null,
  modelOverride: null,
  outputLanguage: "en",
  controlPreset: "standard",
  evidenceMode: "standard",
  includeComments: false,
});

await listenToPromptPackRunEvents(vi.fn());
expect(listen).toHaveBeenCalledWith("prompt-pack-run-event", expect.any(Function));

await listPromptPackRuns({ projectId: 7, limit: 20 });
expect(invoke).toHaveBeenCalledWith("list_prompt_pack_runs", {
  projectId: 7,
  limit: 20,
});
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

- `0007_prompt_pack_run_idempotency.sql` enforces non-null
  `client_request_id` uniqueness while allowing NULL for pre-existing rows;
- preflight partitions behave as specified;
- start recomputes preflight and returns blocked response without creating a
  failed run when start-time preflight has blocking failures;
- duplicate `client_request_id` returns the existing run instead of creating a
  duplicate;
- snapshots are deterministic and run-local;
- stage skeleton rows are created;
- active run/cancel state works without persisting `cancel_requested` as
  `run_status`;
- stale `queued` and `running` rows are marked `interrupted` during cleanup;
- `list_prompt_pack_runs` returns recent Prompt Pack runs for the requested
  project with deterministic ordering;
- terminal events remove runs from active state and use
  `prompt-pack-run-event`;
- frontend wrappers call the expected Tauri commands.
