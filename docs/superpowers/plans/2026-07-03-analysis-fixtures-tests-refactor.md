# Analysis Fixtures Tests Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** active implementation plan; design approved, implementation not started as of 2026-07-03 because `src-tauri/src/analysis/fixtures/tests/` does not exist.

**Goal:** Move the inline `#[cfg(test)] mod tests` body out of `src-tauri/src/analysis/fixtures.rs` into focused nested fixture test modules without changing production behavior or test assertions.

**Architecture:** Keep `fixtures.rs` as the debug command and fixture lifecycle facade, with only `#[cfg(test)] mod tests;` for tests. Create `src-tauri/src/analysis/fixtures/tests/` with shared harness, summary, clear, seed, snapshot, and active-run test modules. Keep tests exercising the parent fixture facade through explicit named `super::super` imports rather than private production child modules.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite in-memory tests, Cargo test/check/fmt with `--manifest-path src-tauri/Cargo.toml`, PowerShell on Windows.

## Global Constraints

- This is a Rust test-only refactor; do not change production behavior, debug command paths, active-run state behavior, cancellation behavior, fixture rows, summary counts, SQL, metadata payloads, assertions, or test coverage.
- Do not move or edit production seed logic in `src-tauri/src/analysis/fixtures/seed.rs`.
- Do not move debug Tauri commands out of `src-tauri/src/analysis/fixtures.rs`.
- Do not split clear/delete production logic into a new module in this slice.
- Do not change `AnalysisRedesignFixtureSummary`, root exports in `analysis/mod.rs`, or command registration in `lib.rs`.
- Keep production visibility unchanged; do not widen helper functions or fixture constants for this test split.
- Shared test helpers may use `pub(super)` only; do not use `pub(crate)` or `pub` in `src-tauri/src/analysis/fixtures/tests/`.
- Tests must exercise the parent fixture facade. Do not import any path starting with `super::super::seed::`, `super::seed::`, or `crate::analysis::fixtures::seed::` from fixture tests.
- Keep imports explicit. Do not use `use super::super::*` or crate glob imports in fixture test modules.
- Run commands from the repository root with `--manifest-path src-tauri/Cargo.toml`.
- Run every fixture test command in the default dev test profile; do not use `--release` for required fixture slices.
- Run each `cargo`, `git`, and guard command separately or through a wrapper that stops on non-zero `$LASTEXITCODE`; plain multi-command PowerShell blocks can hide failures.
- Every filtered `cargo test` command in this plan must run real tests, not green `0 tests` runs.
- Target files must be clean before editing. If `src-tauri/src/analysis/fixtures.rs` or `src-tauri/src/analysis/fixtures/tests/` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline decision before starting.
- Do not stage unrelated dirty files, including `.claude/settings.local.json`.

---

## File Structure

- Modify: `src-tauri/src/analysis/fixtures.rs`
  - Keep all production definitions and debug command functions.
  - Keep `mod seed;` and the private parent import for `seed_analysis_redesign_fixtures_in_pool`.
  - Replace the inline test module body with `#[cfg(test)] mod tests;`.

- Create: `src-tauri/src/analysis/fixtures/tests/mod.rs`
  - Declare child test modules only.

- Create: `src-tauri/src/analysis/fixtures/tests/harness.rs`
  - Own shared `fixture_pool` and `count` helpers.
  - Own `fixture_test_pool_has_required_tables`.

- Create: `src-tauri/src/analysis/fixtures/tests/summary.rs`
  - Own summary serialization coverage.

- Create: `src-tauri/src/analysis/fixtures/tests/clear.rs`
  - Own `insert_minimal_clear_fixture`.
  - Own clear/delete/idempotency tests.

- Create: `src-tauri/src/analysis/fixtures/tests/seed.rs`
  - Own seed row/content/detail/identity/compression/run-state/reseed tests.

- Create: `src-tauri/src/analysis/fixtures/tests/snapshot.rs`
  - Own snapshot/read-model/trace/capture-failed fixture tests.

- Create: `src-tauri/src/analysis/fixtures/tests/active_runs.rs`
  - Own active-run state and cancellation tests.

---

### Task 1: Split Fixtures Tests Into Nested Modules

**Files:**
- Modify: `src-tauri/src/analysis/fixtures.rs`
- Create: `src-tauri/src/analysis/fixtures/tests/mod.rs`
- Create: `src-tauri/src/analysis/fixtures/tests/harness.rs`
- Create: `src-tauri/src/analysis/fixtures/tests/summary.rs`
- Create: `src-tauri/src/analysis/fixtures/tests/clear.rs`
- Create: `src-tauri/src/analysis/fixtures/tests/seed.rs`
- Create: `src-tauri/src/analysis/fixtures/tests/snapshot.rs`
- Create: `src-tauri/src/analysis/fixtures/tests/active_runs.rs`

**Interfaces:**
- Consumes:
  - Current inline `#[cfg(test)] mod tests` from `src-tauri/src/analysis/fixtures.rs`.
  - Parent fixture facade imports through explicit named `super::super` import lists inside thematic test modules.
  - `crate::analysis::AnalysisState` in `active_runs.rs`.
  - `sqlx::{Pool, Sqlite}` and `sqlx::sqlite::SqlitePoolOptions` only where test helpers need them.
- Produces:
  - `#[cfg(test)] mod tests;` in `src-tauri/src/analysis/fixtures.rs`.
  - `tests/mod.rs` declaring `active_runs`, `clear`, `harness`, `seed`, `snapshot`, and `summary`.
  - Same test functions under new paths such as `analysis::fixtures::tests::seed::seed_creates_safe_account_prompt_profile_sources_and_group`.

- [ ] **Step 1: Capture pre-edit worktree status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected:

- `src-tauri/src/analysis/fixtures.rs` is not modified or staged.
- `src-tauri/src/analysis/fixtures/tests/` does not exist, or the executor stops for an explicit baseline decision before editing.
- Unrelated local files such as `.claude/settings.local.json` may exist, but must remain unstaged throughout this task.

- [ ] **Step 2: Persist a pre-edit status snapshot**

Run:

```powershell
$tag = "analysis-fixtures-tests-refactor-" + (Get-Date -Format "yyyyMMddHHmmss")
$preEditStatusPath = Join-Path $env:TEMP "$tag-status-before.txt"
$pointerPath = Join-Path $env:TEMP "extractum-analysis-fixtures-tests-refactor-status-pointer.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $preEditStatusPath
$preEditStatusPath | Set-Content -LiteralPath $pointerPath
$pointerPath
Get-Content -LiteralPath $pointerPath
```

Expected: PowerShell prints the pointer file path and then the saved status snapshot path. Later status comparison reads the path from the pointer file, so it works across separate shell sessions.

- [ ] **Step 3: Inspect target-file baseline**

Run:

```powershell
git diff -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/tests
```

Expected: no diff.

Run:

```powershell
git diff --cached -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/tests
```

Expected: no staged diff.

Run:

```powershell
git ls-files src-tauri/src/analysis/fixtures/tests
```

Expected: no output. If any tracked `fixtures/tests` file appears, stop and make a separate baseline decision before continuing.

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/fixtures/tests') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/fixtures/tests
    Get-ChildItem -Recurse -Force -LiteralPath 'src-tauri/src/analysis/fixtures/tests'
    Get-ChildItem -Recurse -File -Force -LiteralPath 'src-tauri/src/analysis/fixtures/tests' |
        ForEach-Object { $_.FullName; Get-Content -Raw -LiteralPath $_.FullName }
    throw "fixtures/tests already exists; stop for a baseline decision"
}
```

Expected: no output if the directory does not exist. If it exists in any form, this command prints the baseline and stops.

- [ ] **Step 4: Run baseline fixture tests and compile check**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run. Current snapshot at plan authoring has the inline fixture tests under `analysis::fixtures::tests::`; do not require an exact count if nearby tests change before execution.

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass. This establishes crate-wide compile coverage before the module-boundary refactor.

If either baseline command fails, stop. Record the failure as pre-existing and do not edit production code in this task.

- [ ] **Step 5: Create the nested test module declarations**

Create `src-tauri/src/analysis/fixtures/tests/mod.rs`:

```rust
mod active_runs;
mod clear;
mod harness;
mod seed;
mod snapshot;
mod summary;
```

Create empty files:

```text
src-tauri/src/analysis/fixtures/tests/harness.rs
src-tauri/src/analysis/fixtures/tests/summary.rs
src-tauri/src/analysis/fixtures/tests/clear.rs
src-tauri/src/analysis/fixtures/tests/seed.rs
src-tauri/src/analysis/fixtures/tests/snapshot.rs
src-tauri/src/analysis/fixtures/tests/active_runs.rs
```

- [ ] **Step 6: Move shared harness helpers and harness test**

Move these items from the inline `mod tests` in `src-tauri/src/analysis/fixtures.rs` into `src-tauri/src/analysis/fixtures/tests/harness.rs`:

- `fixture_pool`
- `count`
- `fixture_test_pool_has_required_tables`

Use this import and visibility shape in `harness.rs`:

```rust
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

pub(super) async fn fixture_pool() -> Pool<Sqlite> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    crate::migrations::apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("enable foreign keys");
    pool
}

pub(super) async fn count(pool: &Pool<Sqlite>, sql: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|error| panic!("count query failed: {sql}: {error}"))
}
```

Keep `fixture_test_pool_has_required_tables` below those helpers with its current body. It should call `fixture_pool()` and the local `count` helper.

- [ ] **Step 7: Move summary serialization test**

Move `summary_serializes_with_camel_case_keys` into `src-tauri/src/analysis/fixtures/tests/summary.rs`.

Use this import shape:

```rust
use super::super::AnalysisRedesignFixtureSummary;
```

Keep the test body unchanged.

- [ ] **Step 8: Move clear fixture tests**

Move these items into `src-tauri/src/analysis/fixtures/tests/clear.rs`:

- `insert_minimal_clear_fixture`
- `clear_removes_only_fixture_rows_and_is_idempotent`
- `clear_preserves_non_fixture_groups_and_members`
- `clear_deletes_child_rows_through_fixture_parent_ids`

Use this import shape:

```rust
use super::harness::{count, fixture_pool};
use super::super::{clear_analysis_redesign_fixtures_in_pool, AnalysisRedesignFixtureSummary};
use sqlx::{Pool, Sqlite};
```

Keep `insert_minimal_clear_fixture` private:

```rust
async fn insert_minimal_clear_fixture(pool: &Pool<Sqlite>) {
```

Keep the moved SQL and assertions unchanged.

- [ ] **Step 9: Move seed behavior tests**

Move these tests into `src-tauri/src/analysis/fixtures/tests/seed.rs`:

- `seed_creates_safe_account_prompt_profile_sources_and_group`
- `seed_creates_post_sync_reader_content`
- `seed_creates_valid_typed_youtube_detail_metadata`
- `seed_creates_sources_that_pass_identity_repair`
- `compressed_fixture_fields_are_readable`
- `seed_creates_fixture_runs_with_statuses_templates_and_snapshots`
- `seed_twice_keeps_one_deterministic_fixture_set`

Use this import shape:

```rust
use super::harness::{count, fixture_pool};
use super::super::{
    seed_analysis_redesign_fixtures_in_pool, FIXTURE_MARKER, TELEGRAM_CHANNEL_LABEL,
    YOUTUBE_PLAYLIST_LABEL, YOUTUBE_VIDEO_LABEL,
};
```

Keep direct crate-qualified calls such as `crate::youtube::detail::get_youtube_video_detail_from_pool`, `crate::sources::identity_repair::repair_source_identity`, and `crate::compression::decompress_text` as they are.

- [ ] **Step 10: Move snapshot/read-model fixture tests**

Move these tests into `src-tauri/src/analysis/fixtures/tests/snapshot.rs`:

- `seeded_snapshot_runs_expose_captured_snapshot_state`
- `fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot`
- `missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages`
- `capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report`

Use this import shape:

```rust
use super::harness::{count, fixture_pool};
use super::super::{
    seed_analysis_redesign_fixtures_in_pool, CAPTURE_FAILED_SNAPSHOT_ERROR,
    CAPTURE_FAILED_SNAPSHOT_RUN_LABEL, COMPLETED_SNAPSHOT_RUN_LABEL, GROUP_SNAPSHOT_RUN_LABEL,
    MISSING_SNAPSHOT_RUN_LABEL,
};
```

Keep the store read-model calls through the `crate::analysis::store` facade as they are.

- [ ] **Step 11: Move active-run tests**

Move these tests into `src-tauri/src/analysis/fixtures/tests/active_runs.rs`:

- `fixture_active_state_tracks_seeded_running_run`
- `fixture_cancel_waiter_marks_running_run_cancelled`

Use this import shape:

```rust
use super::harness::fixture_pool;
use super::super::{
    finish_cancelled_fixture_run, fixture_run_ids, register_fixture_active_runs,
    remove_fixture_active_runs, seed_analysis_redesign_fixtures_in_pool, RUNNING_RUN_LABEL,
};
use crate::analysis::AnalysisState;
```

Replace current `super::super::AnalysisState::new()` calls with:

```rust
let state = AnalysisState::new();
```

Keep cancellation, status, and timeout assertions unchanged.

- [ ] **Step 12: Replace the inline test module in `fixtures.rs`**

In `src-tauri/src/analysis/fixtures.rs`, replace the entire inline test module, starting at `#[cfg(test)] mod tests {` and ending at that module's closing brace, with:

```rust
#[cfg(test)]
mod tests;
```

Do not change production code above that point. In particular, keep production helper functions and fixture constants private.

- [ ] **Step 13: Run rustfmt**

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

- [ ] **Step 14: Run source guards for `fixtures.rs` wiring and production visibility**

Run:

```powershell
$lines = Get-Content src-tauri/src/analysis/fixtures.rs
$cfgIndexes = @(
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^\s*#\[cfg\(test\)\]\s*$') { $i }
    }
)
$modIndexes = @(
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^\s*mod tests;\s*$') { $i }
    }
)
if ($cfgIndexes.Count -ne 1 -or $modIndexes.Count -ne 1 -or $modIndexes[0] -ne ($cfgIndexes[0] + 1)) {
    throw "fixtures.rs must contain exactly one adjacent #[cfg(test)] / mod tests; pair"
}
$lines[$cfgIndexes[0]]
$lines[$modIndexes[0]]
```

Expected: exactly two adjacent lines are printed: `#[cfg(test)]` followed by `mod tests;`.

Run:

```powershell
$inlineTestMatches = @(rg -n "#\[tokio::test\]|^mod tests \{|use super::\*|SqlitePoolOptions|^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn) (fixture_pool|count|insert_minimal_clear_fixture)\b" src-tauri/src/analysis/fixtures.rs)
if ($inlineTestMatches.Count -ne 0) {
    $inlineTestMatches
    throw "inline fixture test body or helper remains in fixtures.rs"
}
```

Expected: no output and no throw.

Run:

```powershell
$widenedHelperMatches = @(rg -n "^\s*pub(\([^)]*\))?\s+(async\s+fn|fn) (fixture_run_ids|register_fixture_active_runs|remove_fixture_active_runs|finish_cancelled_fixture_run|spawn_fixture_cancellation_waiters|clear_analysis_redesign_fixtures_in_pool|rows_to_i64)\b" src-tauri/src/analysis/fixtures.rs)
if ($widenedHelperMatches.Count -ne 0) {
    $widenedHelperMatches
    throw "fixture production helper visibility was widened"
}
```

Expected: no output and no throw.

Run:

```powershell
foreach ($constName in @(
    "FIXTURE_MARKER",
    "FIXTURE_EXTERNAL_PREFIX",
    "FIXTURE_PROFILE_ID",
    "FIXTURE_NOW",
    "FIXTURE_PERIOD_FROM",
    "FIXTURE_PERIOD_TO",
    "TELEGRAM_CHANNEL_LABEL",
    "TELEGRAM_SUPERGROUP_LABEL",
    "YOUTUBE_VIDEO_LABEL",
    "YOUTUBE_PLAYLIST_LABEL",
    "YOUTUBE_FIXTURE_VIDEO_ID",
    "YOUTUBE_FIXTURE_PLAYLIST_ID",
    "TELEGRAM_FIXTURE_CHANNEL_PEER_ID",
    "TELEGRAM_FIXTURE_SUPERGROUP_PEER_ID",
    "TELEGRAM_GROUP_LABEL",
    "COMPLETED_SNAPSHOT_RUN_LABEL",
    "MISSING_SNAPSHOT_RUN_LABEL",
    "CAPTURE_FAILED_SNAPSHOT_RUN_LABEL",
    "CAPTURE_FAILED_SNAPSHOT_ERROR",
    "CANCELLED_RUN_MESSAGE",
    "RUNNING_RUN_LABEL",
    "FAILED_RUN_LABEL",
    "CANCELLED_RUN_LABEL",
    "GROUP_SNAPSHOT_RUN_LABEL",
    "LLM_PROFILE_LABEL",
    "FIXTURE_SNAPSHOT_CAPTURED_AT"
)) {
    $matches = @(rg -n "^\s*pub(\([^)]*\))?\s+const $constName\b" src-tauri/src/analysis/fixtures.rs)
    if ($matches.Count -ne 0) {
        $matches
        throw "fixture constant visibility was widened: $constName"
    }
}
```

Expected: no output and no throw.

- [ ] **Step 15: Run source guards for test module shape**

Run:

```powershell
foreach ($file in @(
    "mod.rs",
    "harness.rs",
    "summary.rs",
    "clear.rs",
    "seed.rs",
    "snapshot.rs",
    "active_runs.rs"
)) {
    $path = "src-tauri/src/analysis/fixtures/tests/$file"
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "missing fixture test module: $path"
    }
}
```

Expected: no output and no throw.

Run:

```powershell
foreach ($module in @("active_runs", "clear", "harness", "seed", "snapshot", "summary")) {
    rg -n "^mod $module;$" src-tauri/src/analysis/fixtures/tests/mod.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing fixtures test module declaration: $module"
    }
}
```

Expected: one match for each module.

Run:

```powershell
$seedPathMatches = @(rg -n "super::super::seed::|crate::analysis::fixtures::seed::" src-tauri/src/analysis/fixtures/tests)
if ($seedPathMatches.Count -ne 0) {
    $seedPathMatches
    throw "fixture tests must use parent facade access, not the private seed module path"
}
$testModSeedPathMatches = @(rg -n "super::seed::" src-tauri/src/analysis/fixtures/tests/mod.rs)
if ($testModSeedPathMatches.Count -ne 0) {
    $testModSeedPathMatches
    throw "fixtures tests/mod.rs must not reach into the private production seed module"
}
```

Expected: no output and no throw.

Run:

```powershell
$testModBodyMatches = @(rg -n "#\[tokio::test\]|^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn)\b|use super::|use crate::" src-tauri/src/analysis/fixtures/tests/mod.rs)
if ($testModBodyMatches.Count -ne 0) {
    $testModBodyMatches
    throw "fixtures tests/mod.rs must contain module declarations only"
}
```

Expected: no output and no throw.

Run:

```powershell
$globImportMatches = @(rg -n "use\s+super::super::\*|use\s+crate::.*::\*" src-tauri/src/analysis/fixtures/tests)
if ($globImportMatches.Count -ne 0) {
    $globImportMatches
    throw "fixture test modules must use explicit imports, not parent or crate glob imports"
}
```

Expected: no output and no throw.

- [ ] **Step 16: Run source guards for required tests and assertion markers**

Run:

```powershell
$requiredTests = @{
    "summary.rs" = @("summary_serializes_with_camel_case_keys")
    "harness.rs" = @("fixture_test_pool_has_required_tables")
    "clear.rs" = @(
        "clear_removes_only_fixture_rows_and_is_idempotent",
        "clear_preserves_non_fixture_groups_and_members",
        "clear_deletes_child_rows_through_fixture_parent_ids"
    )
    "seed.rs" = @(
        "seed_creates_safe_account_prompt_profile_sources_and_group",
        "seed_creates_post_sync_reader_content",
        "seed_creates_valid_typed_youtube_detail_metadata",
        "seed_creates_sources_that_pass_identity_repair",
        "compressed_fixture_fields_are_readable",
        "seed_creates_fixture_runs_with_statuses_templates_and_snapshots",
        "seed_twice_keeps_one_deterministic_fixture_set"
    )
    "snapshot.rs" = @(
        "seeded_snapshot_runs_expose_captured_snapshot_state",
        "fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot",
        "missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages",
        "capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report"
    )
    "active_runs.rs" = @(
        "fixture_active_state_tracks_seeded_running_run",
        "fixture_cancel_waiter_marks_running_run_cancelled"
    )
}
foreach ($entry in $requiredTests.GetEnumerator()) {
    foreach ($testName in $entry.Value) {
        $path = "src-tauri/src/analysis/fixtures/tests/$($entry.Key)"
        $content = Get-Content -Raw $path
        $pattern = "(?s)#\[tokio::test\]\s*async fn $([regex]::Escape($testName))\b"
        if ($content -notmatch $pattern) {
            throw "missing #[tokio::test] async fixture test $testName in $path"
        }
    }
}
```

Expected: no output and no throw.

Run:

```powershell
$requiredMarkers = @{
    "summary.rs" = @("llmProfiles")
    "clear.rs" = @("SELECT COUNT(*) FROM accounts WHERE label = 'Personal'")
    "seed.rs" = @("analysis_fixture_video", "repair seeded fixture identities")
    "snapshot.rs" = @("This capture-failed fixture report remains readable.")
    "active_runs.rs" = @("ANALYSIS_STATUS_CANCELLED")
}
foreach ($entry in $requiredMarkers.GetEnumerator()) {
    foreach ($marker in $entry.Value) {
        $path = "src-tauri/src/analysis/fixtures/tests/$($entry.Key)"
        rg -n -F $marker $path
        if ($LASTEXITCODE -ne 0) {
            throw "missing fixture assertion marker '$marker' in $path"
        }
    }
}
```

Expected: every required marker is printed once or more and no throw occurs.

- [ ] **Step 17: Run focused fixture module tests**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::summary::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::harness::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::clear::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

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

- [ ] **Step 18: Verify Cargo test inventory**

Run:

```powershell
$testList = cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests:: -- --list
if ($LASTEXITCODE -ne 0) {
    throw "fixture test inventory command failed"
}
foreach ($testPath in @(
    "analysis::fixtures::tests::summary::summary_serializes_with_camel_case_keys",
    "analysis::fixtures::tests::harness::fixture_test_pool_has_required_tables",
    "analysis::fixtures::tests::clear::clear_removes_only_fixture_rows_and_is_idempotent",
    "analysis::fixtures::tests::clear::clear_preserves_non_fixture_groups_and_members",
    "analysis::fixtures::tests::clear::clear_deletes_child_rows_through_fixture_parent_ids",
    "analysis::fixtures::tests::seed::seed_creates_safe_account_prompt_profile_sources_and_group",
    "analysis::fixtures::tests::seed::seed_creates_post_sync_reader_content",
    "analysis::fixtures::tests::seed::seed_creates_valid_typed_youtube_detail_metadata",
    "analysis::fixtures::tests::seed::seed_creates_sources_that_pass_identity_repair",
    "analysis::fixtures::tests::seed::compressed_fixture_fields_are_readable",
    "analysis::fixtures::tests::seed::seed_creates_fixture_runs_with_statuses_templates_and_snapshots",
    "analysis::fixtures::tests::seed::seed_twice_keeps_one_deterministic_fixture_set",
    "analysis::fixtures::tests::snapshot::seeded_snapshot_runs_expose_captured_snapshot_state",
    "analysis::fixtures::tests::snapshot::fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot",
    "analysis::fixtures::tests::snapshot::missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages",
    "analysis::fixtures::tests::snapshot::capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report",
    "analysis::fixtures::tests::active_runs::fixture_active_state_tracks_seeded_running_run",
    "analysis::fixtures::tests::active_runs::fixture_cancel_waiter_marks_running_run_cancelled"
)) {
    if ($testList -notmatch [regex]::Escape($testPath)) {
        throw "fixture test is missing from cargo test --list output: $testPath"
    }
}
```

Expected: every moved test path appears in Cargo's test inventory.

- [ ] **Step 19: Run full fixture tests, compile, and fmt check**

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

- [ ] **Step 20: Inspect final worktree diff before staging**

Run:

```powershell
git diff --stat -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/tests
```

Expected: changed files are limited to:

- `src-tauri/src/analysis/fixtures.rs`
- `src-tauri/src/analysis/fixtures/tests/mod.rs`
- `src-tauri/src/analysis/fixtures/tests/harness.rs`
- `src-tauri/src/analysis/fixtures/tests/summary.rs`
- `src-tauri/src/analysis/fixtures/tests/clear.rs`
- `src-tauri/src/analysis/fixtures/tests/seed.rs`
- `src-tauri/src/analysis/fixtures/tests/snapshot.rs`
- `src-tauri/src/analysis/fixtures/tests/active_runs.rs`

Run:

```powershell
git diff --check -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/tests
```

Expected: no whitespace errors.

Run:

```powershell
git status --short --untracked-files=all
```

Expected: only implementation-owned files plus pre-existing unrelated files. `.claude/settings.local.json`, if present, remains untracked and unstaged.

- [ ] **Step 21: Compare final status against pre-edit snapshot**

Run:

```powershell
$pointerPath = Join-Path $env:TEMP "extractum-analysis-fixtures-tests-refactor-status-pointer.txt"
$preEditStatusPath = Get-Content -LiteralPath $pointerPath
$afterPath = Join-Path $env:TEMP "analysis-fixtures-tests-refactor-status-after.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $afterPath
Compare-Object (Get-Content -LiteralPath $preEditStatusPath) (Get-Content -LiteralPath $afterPath)
```

Expected: differences are only the intended implementation-owned files. Pre-existing unrelated entries must not be modified or staged.

- [ ] **Step 22: Stage implementation files**

Run:

```powershell
git add -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/tests/mod.rs src-tauri/src/analysis/fixtures/tests/harness.rs src-tauri/src/analysis/fixtures/tests/summary.rs src-tauri/src/analysis/fixtures/tests/clear.rs src-tauri/src/analysis/fixtures/tests/seed.rs src-tauri/src/analysis/fixtures/tests/snapshot.rs src-tauri/src/analysis/fixtures/tests/active_runs.rs
```

Expected: only the fixture test split files are staged.

- [ ] **Step 23: Verify staged diff**

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
M	src-tauri/src/analysis/fixtures.rs
A	src-tauri/src/analysis/fixtures/tests/mod.rs
A	src-tauri/src/analysis/fixtures/tests/harness.rs
A	src-tauri/src/analysis/fixtures/tests/summary.rs
A	src-tauri/src/analysis/fixtures/tests/clear.rs
A	src-tauri/src/analysis/fixtures/tests/seed.rs
A	src-tauri/src/analysis/fixtures/tests/snapshot.rs
A	src-tauri/src/analysis/fixtures/tests/active_runs.rs
```

Run:

```powershell
git status --short --untracked-files=all
```

Expected: implementation files are staged. Pre-existing unrelated files remain unstaged.

- [ ] **Step 24: Commit the Rust refactor**

Run:

```powershell
git commit -m "refactor: split analysis fixture tests"
```

Expected: commit succeeds with only the staged fixture test split files.

- [ ] **Step 25: Record post-commit status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: no dirty implementation files remain. Pre-existing unrelated files may remain if they were present before this task.

## Final Verification Checklist

Before reporting the implementation complete, confirm the execution log includes:

- [ ] pre-edit `git status --short --untracked-files=all` captured;
- [ ] target-file baseline proved `fixtures.rs` was clean and `fixtures/tests/` did not contain pre-existing tracked or untracked work without an explicit baseline decision;
- [ ] baseline `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::` passed before editing and was not a green `0 tests` run;
- [ ] baseline `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` passed before editing;
- [ ] source guards proved `fixtures.rs` contains exactly one adjacent `#[cfg(test)]` / `mod tests;` pair;
- [ ] source guards proved `fixtures.rs` no longer contains inline test body helpers or `#[tokio::test]` attributes;
- [ ] source guards proved production helper and fixture constant visibility was not widened;
- [ ] source guards proved all required test files exist as files, not directories;
- [ ] source guards proved `tests/mod.rs` declares only modules and no test/helper logic;
- [ ] source guards proved tests do not import the private production seed module directly;
- [ ] source guards proved fixture test modules do not use parent or crate glob imports;
- [ ] source guards proved every required moved test has `#[tokio::test]` immediately before its `async fn`;
- [ ] source guards proved assertion markers moved to the expected thematic modules;
- [ ] every focused fixture module test command passed and was not a green `0 tests` run;
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests:: -- --list` contained every expected moved test path;
- [ ] full `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::` passed and was not a green `0 tests` run;
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` passed;
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passed;
- [ ] staged diff contained only the expected fixture test split files;
- [ ] post-commit `git status --short --untracked-files=all` has no dirty implementation files.
