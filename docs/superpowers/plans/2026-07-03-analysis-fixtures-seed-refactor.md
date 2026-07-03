# Analysis Fixtures Seed Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract debug-only analysis redesign fixture seed/data-writing helpers from `src-tauri/src/analysis/fixtures.rs` into private `src-tauri/src/analysis/fixtures/seed.rs` without changing fixture behavior or debug command paths.

**Architecture:** Keep `fixtures.rs` as the debug command, active-run lifecycle, clear/delete, and test facade. Add a private child module `seed` and import only `seed_analysis_redesign_fixtures_in_pool` back into the parent. Move the contiguous seed/data-writing block from `json_zstd` through `seed_analysis_redesign_fixtures_in_pool` into `fixtures/seed.rs`; keep all other helpers and tests in `fixtures.rs`.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite, debug-only module gated by `#[cfg(debug_assertions)]`, Cargo tests with `--manifest-path src-tauri/Cargo.toml`, PowerShell on Windows.

## Global Constraints

- This is a move-only Rust refactor; do not change debug Tauri commands, active-run state behavior, cancellation waiters, clear behavior, fixture rows, summary counts, SQL, metadata payloads, tests, or public debug command paths.
- Do not change `analysis/mod.rs` re-exports or `lib.rs` command registration.
- Keep `fixtures/seed.rs` private: use `mod seed;`, not `pub mod seed;` or `pub(crate) mod seed;`.
- Expose only `pub(super) async fn seed_analysis_redesign_fixtures_in_pool` from `fixtures/seed.rs`.
- Keep all other moved seed helpers private inside `seed.rs`.
- Keep `clear_analysis_redesign_fixtures_in_pool`, `rows_to_i64`, active-run helpers, debug commands, `AnalysisRedesignFixtureSummary`, constants, and inline tests in `fixtures.rs`.
- Run fixture tests in the default dev test profile; do not use `--release` because the module is gated by `#[cfg(debug_assertions)]`.
- Run commands from the repository root with `--manifest-path src-tauri/Cargo.toml`.
- Run each `cargo`, `git`, and guard command separately or through a wrapper that stops on non-zero `$LASTEXITCODE`; plain multi-command PowerShell blocks can hide failures.
- `rg` returns exit code `1` for expected no-match guards; treat that as success only where the step explicitly says no matches are expected.
- Target files must be clean before editing. If `src-tauri/src/analysis/fixtures.rs` or `src-tauri/src/analysis/fixtures/seed.rs` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline commit before starting.
- Do not stage unrelated dirty files, including `.claude/settings.local.json`.

---

## File Structure

- Modify: `src-tauri/src/analysis/fixtures.rs`
  - Add private `mod seed;`.
  - Add private `use self::seed::seed_analysis_redesign_fixtures_in_pool;`.
  - Remove moved seed/data-writing definitions.
  - Remove production imports used only by moved seed logic.
  - Keep debug commands, active-run helpers, clear logic, constants, summary type, and tests.

- Create: `src-tauri/src/analysis/fixtures/seed.rs`
  - Own seed/data-writing helpers.
  - Own YouTube DTO imports and seed-only imports.
  - Expose only `seed_analysis_redesign_fixtures_in_pool` as `pub(super)`.

---

### Task 1: Extract Fixture Seed Module

**Files:**
- Modify: `src-tauri/src/analysis/fixtures.rs`
- Create: `src-tauri/src/analysis/fixtures/seed.rs`

**Interfaces:**
- Consumes:
  - `super::AnalysisRedesignFixtureSummary`
  - `super::clear_analysis_redesign_fixtures_in_pool`
  - fixture constants from `super`
  - `sqlx::{Pool, Sqlite}`
  - `crate::error::{AppError, AppResult}`
  - `crate::youtube::dto::{YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata}`
- Produces:
  - `pub(super) async fn seed_analysis_redesign_fixtures_in_pool(pool: &Pool<Sqlite>) -> AppResult<AnalysisRedesignFixtureSummary>`

- [ ] **Step 1: Capture pre-edit worktree status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected:

- `src-tauri/src/analysis/fixtures.rs` is not modified or staged.
- `src-tauri/src/analysis/fixtures/seed.rs` does not exist, or it is not modified/staged/untracked.
- Unrelated local files such as `.claude/settings.local.json` may exist, but must remain unstaged throughout this task.

- [ ] **Step 2: Persist a pre-edit status snapshot**

Run:

```powershell
$tag = "analysis-fixtures-seed-refactor-" + (Get-Date -Format "yyyyMMddHHmmss")
$preEditStatusPath = Join-Path $env:TEMP "$tag-status-before.txt"
$preEditStatusPointerPath = Join-Path $env:TEMP "analysis-fixtures-seed-refactor-status-pointer.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $preEditStatusPath
$preEditStatusPath | Set-Content -LiteralPath $preEditStatusPointerPath
$preEditStatusPointerPath
```

Expected: PowerShell prints the pointer-file path `analysis-fixtures-seed-refactor-status-pointer.txt`. Later steps must read this pointer file to recover the actual status snapshot path across separate PowerShell sessions.

- [ ] **Step 3: Inspect target-file baseline**

Run:

```powershell
git diff -- src-tauri/src/analysis/fixtures.rs
```

Expected: no diff.

Run:

```powershell
git diff --cached -- src-tauri/src/analysis/fixtures.rs
```

Expected: no staged diff.

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/fixtures/seed.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/fixtures/seed.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/fixtures/seed.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/fixtures/seed.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/fixtures/seed.rs'
}
```

Expected: no output if `seed.rs` does not exist. If it exists or shows any status, stop and make a separate baseline commit before continuing.

- [ ] **Step 4: Run baseline fixture tests and compile check**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_safe_account_prompt_profile_sources_and_group
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_fixture_runs_with_statuses_templates_and_snapshots
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::fixture_active_state_tracks_seeded_running_run
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_twice_keeps_one_deterministic_fixture_set
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass. This establishes that crate-wide compile coverage was green before the module-boundary refactor.

If any baseline test or baseline compile check fails, stop. Record the failure as pre-existing and do not edit production code in this task.

- [ ] **Step 5: Add private seed module wiring in `fixtures.rs`**

In `src-tauri/src/analysis/fixtures.rs`, add this module declaration and import after the existing import block and before constants:

```rust
mod seed;

use self::seed::seed_analysis_redesign_fixtures_in_pool;
```

Keep the module private. Do not add any `pub use` for the seed module or seed entry point.

- [ ] **Step 6: Create `fixtures/seed.rs` with seed imports**

Create `src-tauri/src/analysis/fixtures/seed.rs` with this import block:

```rust
use sqlx::{Pool, Sqlite};

use super::{
    clear_analysis_redesign_fixtures_in_pool, AnalysisRedesignFixtureSummary,
    CAPTURE_FAILED_SNAPSHOT_ERROR, CAPTURE_FAILED_SNAPSHOT_RUN_LABEL, CANCELLED_RUN_LABEL,
    COMPLETED_SNAPSHOT_RUN_LABEL, FAILED_RUN_LABEL, FIXTURE_EXTERNAL_PREFIX, FIXTURE_MARKER,
    FIXTURE_NOW, FIXTURE_PERIOD_FROM, FIXTURE_PERIOD_TO, FIXTURE_PROFILE_ID,
    FIXTURE_SNAPSHOT_CAPTURED_AT, GROUP_SNAPSHOT_RUN_LABEL, LLM_PROFILE_LABEL,
    MISSING_SNAPSHOT_RUN_LABEL, RUNNING_RUN_LABEL, TELEGRAM_CHANNEL_LABEL,
    TELEGRAM_FIXTURE_CHANNEL_PEER_ID, TELEGRAM_FIXTURE_SUPERGROUP_PEER_ID, TELEGRAM_GROUP_LABEL,
    TELEGRAM_SUPERGROUP_LABEL, YOUTUBE_FIXTURE_PLAYLIST_ID, YOUTUBE_FIXTURE_VIDEO_ID,
    YOUTUBE_PLAYLIST_LABEL, YOUTUBE_VIDEO_LABEL,
};
use crate::error::{AppError, AppResult};
use crate::youtube::dto::{
    YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata,
};
```

Do not use `use super::*;`.

- [ ] **Step 7: Move the contiguous seed block into `seed.rs`**

Move the production block that starts at this function in `src-tauri/src/analysis/fixtures.rs`:

```rust
fn json_zstd(value: serde_json::Value) -> AppResult<Vec<u8>> {
```

and ends after the closing brace of:

```rust
async fn seed_analysis_redesign_fixtures_in_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
```

into `src-tauri/src/analysis/fixtures/seed.rs` after the import block from Step 6.

The moved block must include:

- `json_zstd`
- `insert_fixture_account`
- `insert_fixture_prompt_template`
- `insert_fixture_llm_profile`
- `insert_telegram_source`
- `insert_youtube_video_source`
- `insert_youtube_playlist_source`
- `insert_fixture_source_group`
- `insert_item`
- `insert_telegram_content`
- `insert_youtube_content`
- `FixtureIds`
- `insert_run`
- `insert_snapshot_message`
- `mark_fixture_snapshot_captured`
- `mark_fixture_snapshot_capture_failed`
- `trace_zstd`
- `first_item_id`
- `insert_analysis_runs`
- `seed_analysis_redesign_fixtures_in_pool`

Do not move:

- `fixture_run_ids`
- `register_fixture_active_runs`
- `remove_fixture_active_runs`
- `finish_cancelled_fixture_run`
- `spawn_fixture_cancellation_waiters`
- `clear_analysis_redesign_fixtures_in_pool`
- `rows_to_i64`
- `#[cfg(test)] mod tests`

After moving, change only the visibility of the seed entry point:

```rust
pub(super) async fn seed_analysis_redesign_fixtures_in_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary> {
```

Keep all other moved helper definitions private.

- [ ] **Step 8: Clean parent production imports**

In `src-tauri/src/analysis/fixtures.rs`, remove this moved-only import block:

```rust
use crate::youtube::dto::{
    YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata,
};
```

Keep these parent imports:

```rust
use serde::Serialize;
use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager, State};

use super::store::set_run_status;
use super::AnalysisState;
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::time::now_secs;
```

Do not add compression imports to `fixtures.rs`. The moved code can continue using fully qualified `crate::compression::compress_text` and `crate::compression::compress_json_bytes` from `seed.rs`.

- [ ] **Step 9: Run rustfmt**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: command exits 0. If unrelated Rust files changed, inspect them before proceeding and resolve drift before staging.

- [ ] **Step 10: Run focused post-change fixture tests**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_safe_account_prompt_profile_sources_and_group
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_fixture_runs_with_statuses_templates_and_snapshots
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::fixture_active_state_tracks_seeded_running_run
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::fixture_cancel_waiter_marks_running_run_cancelled
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_twice_keeps_one_deterministic_fixture_set
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

- [ ] **Step 11: Run source guards**

Private module declaration:

```powershell
rg -n "^mod seed;" src-tauri/src/analysis/fixtures.rs
```

Expected: one match.

```powershell
rg -n "^pub.*mod seed" src-tauri/src/analysis/fixtures.rs
```

Expected: no matches. Exit code `1` is expected.

Private root import:

```powershell
rg -n "^use self::seed::seed_analysis_redesign_fixtures_in_pool;" src-tauri/src/analysis/fixtures.rs
```

Expected: one match.

```powershell
rg -n "^pub.*seed_analysis_redesign_fixtures_in_pool" src-tauri/src/analysis/fixtures.rs
```

Expected: no matches. Exit code `1` is expected.

Moved definitions absent from `fixtures.rs`:

```powershell
rg -n "^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn) (json_zstd|insert_fixture_account|insert_fixture_prompt_template|insert_fixture_llm_profile|insert_telegram_source|insert_youtube_video_source|insert_youtube_playlist_source|insert_fixture_source_group|insert_item|insert_telegram_content|insert_youtube_content|insert_run|insert_snapshot_message|mark_fixture_snapshot_captured|mark_fixture_snapshot_capture_failed|trace_zstd|first_item_id|insert_analysis_runs|seed_analysis_redesign_fixtures_in_pool)\b" src-tauri/src/analysis/fixtures.rs
```

Expected: no matches. Exit code `1` is expected.

```powershell
rg -n "^\s*(pub(\([^)]*\))?\s+)?struct FixtureIds\b" src-tauri/src/analysis/fixtures.rs
```

Expected: no matches. Exit code `1` is expected.

Moved definitions present in `seed.rs`:

```powershell
$requiredSeedHelpers = @(
    "json_zstd",
    "insert_fixture_account",
    "insert_fixture_prompt_template",
    "insert_fixture_llm_profile",
    "insert_telegram_source",
    "insert_youtube_video_source",
    "insert_youtube_playlist_source",
    "insert_fixture_source_group",
    "insert_item",
    "insert_telegram_content",
    "insert_youtube_content",
    "insert_run",
    "insert_snapshot_message",
    "mark_fixture_snapshot_captured",
    "mark_fixture_snapshot_capture_failed",
    "trace_zstd",
    "first_item_id",
    "insert_analysis_runs"
)
foreach ($name in $requiredSeedHelpers) {
    rg -n "^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn) $name\b" src-tauri/src/analysis/fixtures/seed.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing moved seed helper in seed.rs: $name"
    }
}
```

Expected: every required helper is checked independently; the command throws on the first missing helper.

```powershell
rg -n "^pub\(super\) async fn seed_analysis_redesign_fixtures_in_pool" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: one match.

```powershell
rg -n "^struct FixtureIds\b" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: one match.

No unintended public API in `seed.rs`:

```powershell
$publicItems = @(rg -n "^\s*pub(\([^)]*\))?\s+" src-tauri/src/analysis/fixtures/seed.rs)
if ($LASTEXITCODE -ne 0) {
    throw "expected the seed entry point to be pub(super)"
}
if ($publicItems.Count -ne 1 -or $publicItems[0] -notmatch "pub\(super\) async fn seed_analysis_redesign_fixtures_in_pool") {
    $publicItems
    throw "unexpected public or restricted-public API in seed.rs"
}
$publicItems
```

Expected: the command prints exactly the `pub(super) async fn seed_analysis_redesign_fixtures_in_pool` line and throws for any additional public or restricted-public item.

Inline tests use the parent facade:

```powershell
$testSection = [regex]::Split((Get-Content -Raw src-tauri/src/analysis/fixtures.rs), "#\[cfg\(test\)\]", 2)[1]
$seedPathMatches = @($testSection | Select-String -Pattern "super::seed|seed::seed_analysis_redesign_fixtures_in_pool|crate::analysis::fixtures::seed")
if ($seedPathMatches.Count -ne 0) {
    $seedPathMatches
    throw "inline tests must use the parent fixture facade, not the private seed module"
}
```

Expected: no output and no throw. Inline tests should call `seed_analysis_redesign_fixtures_in_pool` through the parent private import.

Production import cleanup:

```powershell
$beforeTests = [regex]::Split((Get-Content -Raw src-tauri/src/analysis/fixtures.rs), "#\[cfg\(test\)\]", 2)[0]
$movedImportMatches = @($beforeTests | Select-String -Pattern "YoutubeAvailabilityStatus|YoutubePlaylistMetadata|YoutubeVideoForm|YoutubeVideoMetadata|compress_text|compress_json_bytes|crate::compression")
if ($movedImportMatches.Count -ne 0) {
    $movedImportMatches
    throw "moved-only imports remain in fixtures.rs production section"
}
```

Expected: no output and no throw.

Seed behavior markers:

```powershell
rg -n -F "analysis_redesign_fixture" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: at least one match.

```powershell
rg -n -F "Fixture timestamp segment supports Show in source." src-tauri/src/analysis/fixtures/seed.rs
```

Expected: at least one match.

```powershell
rg -n -F "transcript_description_comments" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: at least one match.

```powershell
rg -n -F "INSERT INTO analysis_runs" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: one match.

```powershell
rg -n -F "INSERT INTO analysis_run_messages" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: one match.

```powershell
rg -n -F "INSERT INTO analysis_chat_messages" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: one match.

```powershell
rg -n -F "AnalysisRedesignFixtureSummary {" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: one match.

Public debug exports remain unchanged:

```powershell
rg -n "pub use self::fixtures::\{" src-tauri/src/analysis/mod.rs
```

Expected: one match.

```powershell
foreach ($symbol in @(
    "seed_analysis_redesign_fixtures",
    "clear_analysis_redesign_fixtures",
    "clear_analysis_redesign_fixture_active_runs"
)) {
    rg -n $symbol src-tauri/src/analysis/mod.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing debug fixture command export in analysis/mod.rs: $symbol"
    }
    rg -n $symbol src-tauri/src/lib.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing debug fixture command registration in lib.rs: $symbol"
    }
}
```

Expected: each of the three debug command symbols is checked independently in both `analysis/mod.rs` and `lib.rs`; the command throws on the first missing symbol.

- [ ] **Step 12: Run full post-change verification**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass. This covers debug command re-exports, `lib.rs` command registration, parent-to-child seed module visibility, and tests.

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: pass. If it fails, run `cargo fmt --manifest-path src-tauri/Cargo.toml`, inspect `git status --short --untracked-files=all`, resolve unrelated drift, and then rerun `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.

- [ ] **Step 13: Compare final worktree to the pre-edit status snapshot**

Run:

```powershell
$preEditStatusPointerPath = Join-Path $env:TEMP "analysis-fixtures-seed-refactor-status-pointer.txt"
$preEditStatusPath = Get-Content -LiteralPath $preEditStatusPointerPath
$before = Get-Content -LiteralPath $preEditStatusPath
$afterPath = Join-Path $env:TEMP "analysis-fixtures-seed-refactor-status-after.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $afterPath
$after = Get-Content -LiteralPath $afterPath
Compare-Object -ReferenceObject $before -DifferenceObject $after
```

Expected: differences are limited to intended changes in:

- `src-tauri/src/analysis/fixtures.rs`
- `src-tauri/src/analysis/fixtures/seed.rs`

Unrelated pre-existing files such as `.claude/settings.local.json` may appear in both before and after and must not be staged.

- [ ] **Step 14: Inspect implementation diff**

Run:

```powershell
git diff -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/seed.rs
```

Expected:

- `fixtures.rs` adds private `mod seed;`.
- `fixtures.rs` adds private `use self::seed::seed_analysis_redesign_fixtures_in_pool;`.
- `fixtures.rs` removes only moved seed/data-writing helpers and moved-only imports.
- `fixtures.rs` keeps debug commands, active-run helpers, clear logic, constants, summary type, and tests.
- `seed.rs` contains moved helpers with unchanged SQL, metadata payloads, fixture labels, summary counts, compression behavior, and insertion order.
- only `seed_analysis_redesign_fixtures_in_pool` is `pub(super)`.

Run:

```powershell
git diff --check -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/seed.rs
```

Expected: no whitespace errors.

- [ ] **Step 15: Stage implementation files only**

Run:

```powershell
git add -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/seed.rs
```

Expected: only the two implementation files are staged.

Run:

```powershell
git diff --cached --name-status
```

Expected:

```text
M       src-tauri/src/analysis/fixtures.rs
A       src-tauri/src/analysis/fixtures/seed.rs
```

Run:

```powershell
git diff --cached --check
```

Expected: no whitespace errors.

- [ ] **Step 16: Commit the refactor**

Run:

```powershell
git commit -m "refactor: extract analysis fixture seed logic"
```

Expected: commit succeeds with only:

- `src-tauri/src/analysis/fixtures.rs`
- `src-tauri/src/analysis/fixtures/seed.rs`

- [ ] **Step 17: Record post-commit status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: no new implementation files remain unstaged. Pre-existing unrelated files may remain untracked, but no refactor files should be dirty.

---

## Final Verification Checklist

Before reporting the implementation complete, confirm the execution log includes:

- [ ] focused baseline fixture seed tests passed before editing and were not green `0 tests` runs;
- [ ] baseline `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::` passed before editing and was not a green `0 tests` run;
- [ ] baseline `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` passed before editing;
- [ ] focused post-change fixture seed and active-run tests passed and were not green `0 tests` runs;
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::` passed in the default dev profile and was not a green `0 tests` run;
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` passed;
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passed after any formatting fixes;
- [ ] source guards proved `seed` is private and the seed entry point is only privately imported by the parent;
- [ ] source guards proved moved definitions are absent from `fixtures.rs` and present in `fixtures/seed.rs`;
- [ ] source guards proved `seed.rs` has no unintended public API beyond `pub(super) async fn seed_analysis_redesign_fixtures_in_pool`;
- [ ] source guards proved inline tests do not call `super::seed` or direct private-module paths;
- [ ] source guards proved moved-only YouTube DTO and compression imports are absent from parent production imports;
- [ ] source guards proved debug command re-exports and command registration remain present;
- [ ] staged files were limited to `src-tauri/src/analysis/fixtures.rs` and `src-tauri/src/analysis/fixtures/seed.rs`;
- [ ] post-commit `git status --short --untracked-files=all` has no dirty refactor files.
