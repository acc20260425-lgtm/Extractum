# Analysis Fixtures Seed Runs Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** active implementation plan; design approved, implementation not started as of 2026-07-03 because `src-tauri/src/analysis/fixtures/seed/runs.rs` does not exist.

**Goal:** Extract fixture run, snapshot-message, trace, and chat-message seed writers from `src-tauri/src/analysis/fixtures/seed.rs` into `src-tauri/src/analysis/fixtures/seed/runs.rs` without behavior changes.

**Architecture:** Keep `fixtures/seed.rs` as the source/content setup and seed orchestration module. Add a private child module `fixtures/seed/runs.rs` that owns `FixtureIds`, `insert_analysis_runs`, and the private run/snapshot helper functions. Parent `seed.rs` constructs `FixtureIds` and calls `insert_analysis_runs`; all fixture constants needed by `runs.rs` are imported directly from the grandparent fixture module with explicit named imports.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite, Cargo test/check/fmt with `--manifest-path src-tauri/Cargo.toml`, PowerShell on Windows.

## Global Constraints

- This is an internal Rust refactor; do not change debug Tauri commands, fixture rows, run statuses, snapshot states, trace payloads, chat messages, SQL, summary counts, test coverage, or public debug command paths.
- Do not move account, prompt template, LLM profile, source, source group, item, Telegram content, or YouTube content writers.
- Do not move or edit debug Tauri commands.
- Do not move or edit fixture tests.
- Do not change `AnalysisRedesignFixtureSummary`.
- Do not change fixture constants in `fixtures.rs`.
- Do not change SQL, fixture data, trace payloads, snapshot states, status strings, chat messages, compressed payloads, database migrations, frontend code, or Tauri command payloads.
- Keep `json_zstd` private in parent `seed.rs`; `runs.rs` may call it through `super::json_zstd`.
- `runs.rs` exposes only `pub(super) struct FixtureIds` and `pub(super) async fn insert_analysis_runs`.
- `FixtureIds` fields use `pub(super)` only because parent `seed.rs` constructs the struct.
- All other moved helpers in `runs.rs` remain private.
- Import fixture constants in `runs.rs` from `super::super` with an explicit named import list. Do not depend on constants reimported by `seed.rs`.
- Do not use glob imports.
- Run commands from the repository root with `--manifest-path src-tauri/Cargo.toml`.
- Run every fixture test command in the default dev test profile; do not use `--release` for required fixture slices.
- Run each `cargo`, `git`, and guard command separately or through a wrapper that stops on non-zero `$LASTEXITCODE`; plain multi-command PowerShell blocks can hide failures.
- Every filtered `cargo test` command in this plan must run real tests, not green `0 tests` runs.
- Target files must be clean before editing. If `src-tauri/src/analysis/fixtures/seed.rs` or `src-tauri/src/analysis/fixtures/seed/` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline decision before starting.
- Do not stage unrelated dirty files, including `.claude/settings.local.json`.

---

## File Structure

- Modify: `src-tauri/src/analysis/fixtures/seed.rs`
  - Add `mod runs;`.
  - Add `use self::runs::{insert_analysis_runs, FixtureIds};`.
  - Keep source/content setup and orchestration code.
  - Remove the moved run/snapshot helper cluster.
  - Remove moved-only constants from the parent fixture import list.

- Create: `src-tauri/src/analysis/fixtures/seed/runs.rs`
  - Own `FixtureIds`.
  - Own `insert_analysis_runs`.
  - Own private helpers `insert_run`, `insert_snapshot_message`, `mark_fixture_snapshot_captured`, `mark_fixture_snapshot_capture_failed`, `trace_zstd`, and `first_item_id`.
  - Import `super::json_zstd`.
  - Import fixture constants from `super::super` with a named import list.

---

### Task 1: Extract Fixture Seed Run Writers

**Files:**
- Modify: `src-tauri/src/analysis/fixtures/seed.rs`
- Create: `src-tauri/src/analysis/fixtures/seed/runs.rs`

**Interfaces:**
- Consumes:
  - `super::json_zstd(value: serde_json::Value) -> AppResult<Vec<u8>>` from parent `seed.rs`.
  - Fixture constants from grandparent `analysis::fixtures`.
  - `sqlx::Transaction<'_, Sqlite>` transaction passed from parent `seed.rs`.
- Produces:
  - `pub(super) struct FixtureIds` in `seed/runs.rs`:

```rust
pub(super) struct FixtureIds {
    pub(super) prompt_template_id: i64,
    pub(super) telegram_channel_id: i64,
    pub(super) telegram_supergroup_id: i64,
    pub(super) youtube_video_id: i64,
    pub(super) source_group_id: i64,
}
```

  - `pub(super) async fn insert_analysis_runs(tx: &mut sqlx::Transaction<'_, Sqlite>, ids: FixtureIds) -> AppResult<()>` in `seed/runs.rs`.
  - Private helper functions in `seed/runs.rs`.

- [x] **Step 1: Capture pre-edit worktree status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected:

- `src-tauri/src/analysis/fixtures/seed.rs` is not modified or staged.
- `src-tauri/src/analysis/fixtures/seed/` does not exist, or the executor stops for an explicit baseline decision before editing.
- Unrelated local files such as `.claude/settings.local.json` may exist, but must remain unstaged throughout this task.

- [x] **Step 2: Persist a pre-edit status snapshot**

Run:

```powershell
$tag = "analysis-fixtures-seed-runs-refactor-" + (Get-Date -Format "yyyyMMddHHmmss")
$preEditStatusPath = Join-Path $env:TEMP "$tag-status-before.txt"
$pointerPath = Join-Path $env:TEMP "extractum-analysis-fixtures-seed-runs-refactor-status-pointer.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $preEditStatusPath
$preEditStatusPath | Set-Content -LiteralPath $pointerPath
$pointerPath
Get-Content -LiteralPath $pointerPath
```

Expected: PowerShell prints the pointer file path and then the saved status snapshot path. Later status comparison reads the path from the pointer file, so it works across separate shell sessions.

- [x] **Step 3: Inspect target-file baseline**

Run:

```powershell
git diff -- src-tauri/src/analysis/fixtures/seed.rs src-tauri/src/analysis/fixtures/seed
```

Expected: no diff.

Run:

```powershell
git diff --cached -- src-tauri/src/analysis/fixtures/seed.rs src-tauri/src/analysis/fixtures/seed
```

Expected: no staged diff.

Run:

```powershell
git ls-files src-tauri/src/analysis/fixtures/seed
```

Expected: no output. If any tracked `fixtures/seed` file appears, stop and make a separate baseline decision before continuing.

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/fixtures/seed') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/fixtures/seed
    Get-ChildItem -Recurse -Force -LiteralPath 'src-tauri/src/analysis/fixtures/seed'
    Get-ChildItem -Recurse -File -Force -LiteralPath 'src-tauri/src/analysis/fixtures/seed' |
        ForEach-Object { $_.FullName; Get-Content -Raw -LiteralPath $_.FullName }
    throw "fixtures/seed already exists; stop for a baseline decision"
}
```

Expected: no output if the directory does not exist. If it exists in any form, this command prints the baseline and stops.

- [x] **Step 4: Run baseline fixture tests and compile check**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::snapshot::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::active_runs::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass. This establishes crate-wide compile coverage before the module-boundary refactor.

If any baseline command fails, stop. Record the failure as pre-existing and do not edit production code in this task.

- [x] **Step 5: Add child module wiring in `seed.rs`**

At the top of `src-tauri/src/analysis/fixtures/seed.rs`, add:

```rust
mod runs;

use self::runs::{insert_analysis_runs, FixtureIds};
```

Keep `mod runs;` before the child import.

- [x] **Step 6: Create `seed/runs.rs` with imports**

Create `src-tauri/src/analysis/fixtures/seed/runs.rs` with this import skeleton:

```rust
use sqlx::Sqlite;

use super::json_zstd;
use super::super::{
    CANCELLED_RUN_LABEL, CAPTURE_FAILED_SNAPSHOT_ERROR, CAPTURE_FAILED_SNAPSHOT_RUN_LABEL,
    COMPLETED_SNAPSHOT_RUN_LABEL, FAILED_RUN_LABEL, FIXTURE_EXTERNAL_PREFIX, FIXTURE_NOW,
    FIXTURE_PERIOD_FROM, FIXTURE_PERIOD_TO, FIXTURE_PROFILE_ID, FIXTURE_SNAPSHOT_CAPTURED_AT,
    GROUP_SNAPSHOT_RUN_LABEL, LLM_PROFILE_LABEL, MISSING_SNAPSHOT_RUN_LABEL, RUNNING_RUN_LABEL,
    YOUTUBE_VIDEO_LABEL,
};
use crate::error::{AppError, AppResult};
```

Do not use a glob import. Do not import fixture constants through `super`.

- [x] **Step 7: Move `FixtureIds` and run/snapshot helpers into `runs.rs`**

Move these definitions from `seed.rs` to `seed/runs.rs`, preserving bodies exactly except for visibility and imports:

- `FixtureIds`
- `insert_run`
- `insert_snapshot_message`
- `mark_fixture_snapshot_captured`
- `mark_fixture_snapshot_capture_failed`
- `trace_zstd`
- `first_item_id`
- `insert_analysis_runs`

Change `FixtureIds` to:

```rust
pub(super) struct FixtureIds {
    pub(super) prompt_template_id: i64,
    pub(super) telegram_channel_id: i64,
    pub(super) telegram_supergroup_id: i64,
    pub(super) youtube_video_id: i64,
    pub(super) source_group_id: i64,
}
```

Change only the `insert_analysis_runs` signature visibility:

```rust
pub(super) async fn insert_analysis_runs(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    ids: FixtureIds,
) -> AppResult<()> {
```

Keep these helper signatures private:

```rust
async fn insert_run(
```

```rust
async fn insert_snapshot_message(
```

```rust
async fn mark_fixture_snapshot_captured(
```

```rust
async fn mark_fixture_snapshot_capture_failed(
```

```rust
fn trace_zstd(refs: serde_json::Value) -> AppResult<Vec<u8>> {
```

```rust
async fn first_item_id(
```

Do not change SQL, string literals, trace payloads, inserted statuses, chat rows, or snapshot marker updates.

- [x] **Step 8: Clean moved-only imports from `seed.rs`**

After the move, remove constants from the parent fixture import list in `seed.rs` that are used only by moved run/snapshot code.

The `seed.rs` parent import list should keep constants still used by source/content setup and orchestration, including:

```rust
clear_analysis_redesign_fixtures_in_pool, AnalysisRedesignFixtureSummary, FIXTURE_EXTERNAL_PREFIX,
FIXTURE_MARKER, FIXTURE_NOW, FIXTURE_PERIOD_FROM, FIXTURE_PROFILE_ID, TELEGRAM_CHANNEL_LABEL,
TELEGRAM_FIXTURE_CHANNEL_PEER_ID, TELEGRAM_FIXTURE_SUPERGROUP_PEER_ID, TELEGRAM_GROUP_LABEL,
TELEGRAM_SUPERGROUP_LABEL, YOUTUBE_FIXTURE_PLAYLIST_ID, YOUTUBE_FIXTURE_VIDEO_ID,
YOUTUBE_PLAYLIST_LABEL, YOUTUBE_VIDEO_LABEL
```

The moved-only run constants must not remain in `seed.rs`:

```text
CANCELLED_RUN_LABEL
CAPTURE_FAILED_SNAPSHOT_ERROR
CAPTURE_FAILED_SNAPSHOT_RUN_LABEL
COMPLETED_SNAPSHOT_RUN_LABEL
FAILED_RUN_LABEL
FIXTURE_PERIOD_TO
FIXTURE_SNAPSHOT_CAPTURED_AT
GROUP_SNAPSHOT_RUN_LABEL
LLM_PROFILE_LABEL
MISSING_SNAPSHOT_RUN_LABEL
RUNNING_RUN_LABEL
```

- [x] **Step 9: Run rustfmt**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: command completes successfully.

Run:

```powershell
git status --short --untracked-files=all
```

Expected: changed files are limited to implementation-owned files plus pre-existing unrelated files. If rustfmt changes unrelated Rust files, inspect the drift and resolve it before continuing.

- [x] **Step 10: Run source guards for `seed.rs`**

Run:

```powershell
foreach ($pattern in @(
    "^mod runs;$",
    "^use self::runs::\{insert_analysis_runs, FixtureIds\};"
)) {
    rg -n $pattern src-tauri/src/analysis/fixtures/seed.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing seed.rs child-module wiring pattern: $pattern"
    }
}
```

Expected: one match for each required pattern.

Run:

```powershell
$movedNames = @(
    "FixtureIds",
    "insert_run",
    "insert_snapshot_message",
    "mark_fixture_snapshot_captured",
    "mark_fixture_snapshot_capture_failed",
    "trace_zstd",
    "first_item_id",
    "insert_analysis_runs"
)
foreach ($name in $movedNames) {
    $matches = @(rg -n "^\s*(pub(\([^)]*\))?\s+)?(struct|async\s+fn|fn) $name\b" src-tauri/src/analysis/fixtures/seed.rs)
    if ($matches.Count -ne 0) {
        $matches
        throw "moved run seed definition remains in seed.rs: $name"
    }
}
```

Expected: no output and no throw.

Run:

```powershell
foreach ($constName in @(
    "CANCELLED_RUN_LABEL",
    "CAPTURE_FAILED_SNAPSHOT_ERROR",
    "CAPTURE_FAILED_SNAPSHOT_RUN_LABEL",
    "COMPLETED_SNAPSHOT_RUN_LABEL",
    "FAILED_RUN_LABEL",
    "FIXTURE_PERIOD_TO",
    "FIXTURE_SNAPSHOT_CAPTURED_AT",
    "GROUP_SNAPSHOT_RUN_LABEL",
    "LLM_PROFILE_LABEL",
    "MISSING_SNAPSHOT_RUN_LABEL",
    "RUNNING_RUN_LABEL"
)) {
    $matches = @(rg -n "\b$constName\b" src-tauri/src/analysis/fixtures/seed.rs)
    if ($matches.Count -ne 0) {
        $matches
        throw "moved-only run constant remains in seed.rs: $constName"
    }
}
```

Expected: no output and no throw.

- [x] **Step 11: Run source guards for `seed/runs.rs` visibility and imports**

Run:

```powershell
foreach ($pattern in @(
    "^pub\(super\) struct FixtureIds\b",
    "^pub\(super\) async fn insert_analysis_runs\b"
)) {
    rg -n $pattern src-tauri/src/analysis/fixtures/seed/runs.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing expected public seed/runs item: $pattern"
    }
}
```

Expected: one match for each required item.

Run:

```powershell
foreach ($field in @(
    "prompt_template_id",
    "telegram_channel_id",
    "telegram_supergroup_id",
    "youtube_video_id",
    "source_group_id"
)) {
    rg -n "^\s+pub\(super\) $field: i64," src-tauri/src/analysis/fixtures/seed/runs.rs
    if ($LASTEXITCODE -ne 0) {
        throw "FixtureIds field is missing pub(super) visibility: $field"
    }
}
```

Expected: one match for each field.

Run:

```powershell
$publicItems = @(rg -n "^\s*pub(\([^)]*\))?\s+" src-tauri/src/analysis/fixtures/seed/runs.rs)
$allowed = @(
    "pub(super) struct FixtureIds",
    "pub(super) prompt_template_id: i64,",
    "pub(super) telegram_channel_id: i64,",
    "pub(super) telegram_supergroup_id: i64,",
    "pub(super) youtube_video_id: i64,",
    "pub(super) source_group_id: i64,",
    "pub(super) async fn insert_analysis_runs"
)
foreach ($item in $publicItems) {
    $ok = $false
    foreach ($allowedText in $allowed) {
        if ($item -like "*$allowedText*") {
            $ok = $true
        }
    }
    if (-not $ok) {
        $publicItems
        throw "unexpected public or restricted-public item in seed/runs.rs"
    }
}
```

Expected: no throw.

Run:

```powershell
$globMatches = @(rg -n "use\s+.*::\*" src-tauri/src/analysis/fixtures/seed/runs.rs)
if ($globMatches.Count -ne 0) {
    $globMatches
    throw "seed/runs.rs must use explicit imports"
}
```

Expected: no output and no throw.

Run:

```powershell
rg -n "use super::super::\{" src-tauri/src/analysis/fixtures/seed/runs.rs
if ($LASTEXITCODE -ne 0) {
    throw "seed/runs.rs must import fixture constants from super::super"
}
$parentConstantImportMatches = @(rg -n "use super::\{.*(RUNNING_RUN_LABEL|COMPLETED_SNAPSHOT_RUN_LABEL|FIXTURE_PROFILE_ID)" src-tauri/src/analysis/fixtures/seed/runs.rs)
if ($parentConstantImportMatches.Count -ne 0) {
    $parentConstantImportMatches
    throw "seed/runs.rs must not depend on moved-only constants reimported by seed.rs"
}
```

Expected: one named import list from `super::super` exists and no parent constant import matches are found.

- [x] **Step 12: Run source guards for private helpers and moved markers**

Run:

```powershell
foreach ($name in @(
    "insert_run",
    "insert_snapshot_message",
    "mark_fixture_snapshot_captured",
    "mark_fixture_snapshot_capture_failed",
    "trace_zstd",
    "first_item_id"
)) {
    rg -n "^\s*(async\s+fn|fn) $name\b" src-tauri/src/analysis/fixtures/seed/runs.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing private moved helper in seed/runs.rs: $name"
    }
}
```

Expected: one match for each helper.

Run:

```powershell
$requiredMarkers = @(
    "This capture-failed fixture report remains readable.",
    "Fixture failure: provider request failed without changing user data",
    "Fixture cancellation: run was cancelled before snapshot capture",
    "Fixture timestamp segment supports Show in source.",
    "fixture channel update: result-first analysis now has source evidence",
    "Fixture evidence highlights saved snapshots, YouTube timestamps, and Telegram source context.",
    "transcript_description_comments"
)
foreach ($marker in $requiredMarkers) {
    rg -n -F $marker src-tauri/src/analysis/fixtures/seed/runs.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing moved fixture run marker in seed/runs.rs: $marker"
    }
}
```

Expected: every marker is present.

- [x] **Step 13: Run focused fixture tests**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::snapshot::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::active_runs::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

- [x] **Step 14: Run full fixture tests, compile, and fmt check**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass.

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: pass. If it fails, run `cargo fmt --manifest-path src-tauri/Cargo.toml`, inspect `git status --short --untracked-files=all`, resolve unrelated drift, and rerun `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.

- [x] **Step 15: Inspect final worktree diff before staging**

Run:

```powershell
git diff --stat -- src-tauri/src/analysis/fixtures/seed.rs src-tauri/src/analysis/fixtures/seed/runs.rs
```

Expected: changed files are limited to:

- `src-tauri/src/analysis/fixtures/seed.rs`
- `src-tauri/src/analysis/fixtures/seed/runs.rs`

Run:

```powershell
git diff --check -- src-tauri/src/analysis/fixtures/seed.rs src-tauri/src/analysis/fixtures/seed/runs.rs
```

Expected: no whitespace errors.

Run:

```powershell
git status --short --untracked-files=all
```

Expected: only implementation-owned files plus pre-existing unrelated files. `.claude/settings.local.json`, if present, remains untracked and unstaged.

- [x] **Step 16: Compare final status against pre-edit snapshot**

Run:

```powershell
$pointerPath = Join-Path $env:TEMP "extractum-analysis-fixtures-seed-runs-refactor-status-pointer.txt"
$preEditStatusPath = Get-Content -LiteralPath $pointerPath
$afterPath = Join-Path $env:TEMP "analysis-fixtures-seed-runs-refactor-status-after.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $afterPath
Compare-Object (Get-Content -LiteralPath $preEditStatusPath) (Get-Content -LiteralPath $afterPath)
```

Expected: differences are only the intended implementation-owned files. Pre-existing unrelated entries must not be modified or staged.

- [x] **Step 17: Stage implementation files**

Run:

```powershell
git add -- src-tauri/src/analysis/fixtures/seed.rs src-tauri/src/analysis/fixtures/seed/runs.rs
```

Expected: only the fixture seed run-writer files are staged.

- [x] **Step 18: Verify staged diff**

Run:

```powershell
git diff --cached --check
```

Expected: no whitespace errors.

Run:

```powershell
git diff --cached --name-status
```

Expected staged files:

```text
M	src-tauri/src/analysis/fixtures/seed.rs
A	src-tauri/src/analysis/fixtures/seed/runs.rs
```

Run:

```powershell
git status --short --untracked-files=all
```

Expected: implementation files are staged. Pre-existing unrelated files remain unstaged.

- [x] **Step 19: Commit the Rust refactor**

Run:

```powershell
git commit -m "refactor: extract fixture seed run writers"
```

Expected: commit succeeds with only the staged fixture seed run-writer files.

- [x] **Step 20: Record post-commit status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: no dirty implementation files remain. Pre-existing unrelated files may remain if they were present before this task.

## Final Verification Checklist

Before reporting the implementation complete, confirm the execution log includes:

- [x] pre-edit `git status --short --untracked-files=all` captured;
- [x] target-file baseline proved `seed.rs` was clean and `fixtures/seed/` did not contain pre-existing tracked or untracked work without an explicit baseline decision;
- [x] baseline focused fixture tests passed before editing and were not green `0 tests` runs;
- [x] baseline full `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::` passed before editing and was not a green `0 tests` run;
- [x] baseline `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` passed before editing;
- [x] source guards proved `seed.rs` declares `mod runs;` and imports `insert_analysis_runs` / `FixtureIds`;
- [x] source guards proved moved definitions no longer remain in `seed.rs`;
- [x] source guards proved moved-only run constants no longer remain in `seed.rs`;
- [x] source guards proved `FixtureIds`, its fields, and `insert_analysis_runs` have only `pub(super)` visibility;
- [x] source guards proved all other moved helpers are private in `seed/runs.rs`;
- [x] source guards proved `runs.rs` uses explicit imports and imports fixture constants from `super::super`;
- [x] source guards proved moved behavior markers live in `runs.rs`;
- [x] focused fixture tests passed and were not green `0 tests` runs;
- [x] full `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::` passed and was not a green `0 tests` run;
- [x] `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` passed;
- [x] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passed;
- [x] staged diff contained only `seed.rs` and `seed/runs.rs`;
- [x] post-commit `git status --short --untracked-files=all` has no dirty implementation files.
