# Analysis Fixtures Tests Refactor Design

**Date:** 2026-07-03
**Status:** active spec; implementation not started as of 2026-07-03 because `src-tauri/src/analysis/fixtures/tests/` does not exist.
**Scope:** internal Rust test-only refactor of `src-tauri/src/analysis/fixtures.rs` into nested fixture test modules.

## Goal

Reduce the size and review burden of `src-tauri/src/analysis/fixtures.rs` by moving its large inline `#[cfg(test)] mod tests` body into focused nested test modules, without changing production behavior, debug command paths, active-run state behavior, cancellation behavior, fixture rows, summary counts, SQL, metadata payloads, assertions, or test coverage.

This is the next conservative fixtures slice after extracting fixture seed/data-writing logic into `src-tauri/src/analysis/fixtures/seed.rs`. The production fixture facade is now small; most remaining size in `fixtures.rs` is test code.

## Current Shape

`src-tauri/src/analysis/fixtures.rs` currently owns:

- debug-only Tauri commands:
  - `seed_analysis_redesign_fixtures`
  - `clear_analysis_redesign_fixtures`
  - `clear_analysis_redesign_fixture_active_runs`
- `AnalysisRedesignFixtureSummary`;
- fixture marker constants and deterministic timestamps;
- active fixture run tracking and cancellation waiter helpers;
- clear/delete logic for fixture rows;
- private facade import for `seed_analysis_redesign_fixtures_in_pool`;
- a large inline `#[cfg(test)] mod tests`.

The inline test module mixes these concerns:

- shared in-memory SQLite harness:
  - `fixture_pool`
  - `count`
  - migration setup and `PRAGMA foreign_keys`;
- summary serialization;
- clear/idempotency behavior:
  - `insert_minimal_clear_fixture`
  - preserving non-fixture accounts, sources, groups, and members
  - deleting child rows through fixture parent ids;
- seed behavior:
  - safe account/prompt/profile/source/group rows
  - post-sync reader content
  - typed YouTube detail metadata
  - source identity repair compatibility
  - compressed fields
  - fixture run status/template/snapshot rows
  - deterministic reseeding;
- snapshot/read-model behavior:
  - captured snapshot state
  - trace refs
  - missing snapshot run state
  - capture-failed snapshot state and sanitized error text;
- active-run behavior:
  - registering seeded running runs in `AnalysisState`
  - cancelling fixture child tokens
  - marking the running fixture run cancelled.

## Proposed Architecture

Move only test code into a nested test tree:

- `src-tauri/src/analysis/fixtures/tests/mod.rs`
- `src-tauri/src/analysis/fixtures/tests/harness.rs`
- `src-tauri/src/analysis/fixtures/tests/summary.rs`
- `src-tauri/src/analysis/fixtures/tests/clear.rs`
- `src-tauri/src/analysis/fixtures/tests/seed.rs`
- `src-tauri/src/analysis/fixtures/tests/snapshot.rs`
- `src-tauri/src/analysis/fixtures/tests/active_runs.rs`

Keep production code in `src-tauri/src/analysis/fixtures.rs`.

Replace the inline test module in `fixtures.rs` with:

```rust
#[cfg(test)]
mod tests;
```

`tests/mod.rs` should declare the thematic test modules:

```rust
mod active_runs;
mod clear;
mod harness;
mod seed;
mod snapshot;
mod summary;
```

`tests/harness.rs` owns helpers shared by more than one thematic module. The thematic modules import helpers through `super::harness`.

This split keeps the existing Cargo test path prefix under `analysis::fixtures::tests::`. Individual tests gain one more module segment, such as `analysis::fixtures::tests::seed::seed_creates_safe_account_prompt_profile_sources_and_group`. The implementation plan must update verification filters accordingly and must prevent green `0 tests` runs.

## File Responsibilities

`src-tauri/src/analysis/fixtures.rs`

- Keep all production definitions and debug command functions.
- Keep `mod seed;` and the private parent import for `seed_analysis_redesign_fixtures_in_pool`.
- Keep only `#[cfg(test)] mod tests;` for tests.
- Do not gain new test helper code.
- Do not change debug command exports, command registration, or root re-export behavior.

`src-tauri/src/analysis/fixtures/tests/mod.rs`

- Declare child test modules.
- Avoid owning test logic directly unless it is only module wiring.

`src-tauri/src/analysis/fixtures/tests/harness.rs`

- Own shared test helpers:
  - `fixture_pool`
  - `count`
- Own the test that validates the fixture test pool has required tables.
- Do not own fixture row seed helpers used by only one thematic module.

`src-tauri/src/analysis/fixtures/tests/summary.rs`

- Own `summary_serializes_with_camel_case_keys`.

`src-tauri/src/analysis/fixtures/tests/clear.rs`

- Own `insert_minimal_clear_fixture`.
- Own clear/delete tests:
  - `clear_removes_only_fixture_rows_and_is_idempotent`
  - `clear_preserves_non_fixture_groups_and_members`
  - `clear_deletes_child_rows_through_fixture_parent_ids`

`src-tauri/src/analysis/fixtures/tests/seed.rs`

- Own seed row/content tests:
  - `seed_creates_safe_account_prompt_profile_sources_and_group`
  - `seed_creates_post_sync_reader_content`
  - `seed_creates_valid_typed_youtube_detail_metadata`
  - `seed_creates_sources_that_pass_identity_repair`
  - `compressed_fixture_fields_are_readable`
  - `seed_creates_fixture_runs_with_statuses_templates_and_snapshots`
  - `seed_twice_keeps_one_deterministic_fixture_set`

`src-tauri/src/analysis/fixtures/tests/snapshot.rs`

- Own snapshot/read-model fixture tests:
  - `seeded_snapshot_runs_expose_captured_snapshot_state`
  - `fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot`
  - `missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages`
  - `capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report`

`src-tauri/src/analysis/fixtures/tests/active_runs.rs`

- Own active-run state tests:
  - `fixture_active_state_tracks_seeded_running_run`
  - `fixture_cancel_waiter_marks_running_run_cancelled`

## Visibility

This refactor is test-only, but moving tests out of the inline child module changes where helper code lives.

Use this visibility strategy:

1. Keep production visibility unchanged.
2. Keep thematic helper functions private to their module when only one module uses them.
3. Move only genuinely shared helpers into `tests/harness.rs`.
4. Mark shared harness helpers `pub(super)`, never `pub(crate)` or `pub`.
5. Do not widen production item visibility solely for this test split.

Expected production visibility changes: none.

Expected test-helper visibility changes:

```rust
pub(super) async fn fixture_pool() -> Pool<Sqlite>;
pub(super) async fn count(pool: &Pool<Sqlite>, sql: &str) -> i64;
```

`insert_minimal_clear_fixture` should stay private in `tests/clear.rs`.

Moved test modules may access private parent fixture items through `super::super`, because they are descendants of `analysis::fixtures`. This includes:

- `seed_analysis_redesign_fixtures_in_pool`
- `clear_analysis_redesign_fixtures_in_pool`
- `fixture_run_ids`
- `register_fixture_active_runs`
- `remove_fixture_active_runs`
- `finish_cancelled_fixture_run`
- fixture marker constants and fixture labels.

Do not expose these parent items as `pub(super)`, `pub(crate)`, or `pub`.

Facade coverage is intentional: tests should call the parent fixture facade import `super::super::seed_analysis_redesign_fixtures_in_pool`, not the private production child module path `super::super::seed::seed_analysis_redesign_fixtures_in_pool` or `crate::analysis::fixtures::seed`.

`active_runs.rs` should import `AnalysisState` explicitly through `crate::analysis::AnalysisState`. Do not rely on a parent glob import to make it available.

## Data Flow

No runtime data flow changes:

1. In-memory SQLite setup still applies all migrations and enables foreign keys.
2. Clear tests still insert the same minimal fixture rows and non-fixture control rows.
3. Seed tests still call `seed_analysis_redesign_fixtures_in_pool` through the parent fixture facade.
4. Snapshot tests still read fixture runs through `crate::analysis::store` facade functions.
5. Active-run tests still use `AnalysisState` and fixture lifecycle helpers through the parent fixture module.
6. Debug Tauri commands remain unchanged and are not moved in this slice.

Only test module paths change. The full command:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

must continue to run all fixture tests.

## Error Handling

Preserve current test assertions and error expectations exactly:

- summary serialization still asserts camelCase fields such as `llmProfiles`, `sourceGroups`, `promptTemplates`, `snapshotMessages`, `youtubeTranscriptSegments`, and `youtubePlaylistItems`;
- clear tests still assert fixture rows are removed while non-fixture rows survive;
- seed tests still assert no fixture API key is written;
- YouTube detail tests still assert `analysis_fixture_video` and raw metadata `{ "fixture": true }`;
- identity repair tests still assert no fatal errors;
- snapshot tests still assert `Captured` and `CaptureFailed` snapshot states through store read models;
- capture-failed tests still assert `CAPTURE_FAILED_SNAPSHOT_ERROR`;
- active-run tests still assert cancellation through `AnalysisState` and `ANALYSIS_STATUS_CANCELLED`;
- no SQL, fixture literal, status string, result markdown, or assertion message is changed.

The implementation plan must include source guards for these assertion markers after the move:

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

Expected: all moved assertion markers are present in the thematic test modules.

## Non-Goals

This slice does not:

- move or edit production seed logic in `src-tauri/src/analysis/fixtures/seed.rs`;
- move debug Tauri commands out of `fixtures.rs`;
- split clear/delete production logic into a new module;
- change `AnalysisRedesignFixtureSummary`;
- change root exports in `analysis/mod.rs`;
- change command registration in `lib.rs`;
- change SQL, fixture data, compressed payloads, trace payloads, snapshot-state behavior, cancellation behavior, database migrations, frontend code, or Tauri command payloads;
- add new behavior tests beyond path/coverage guards needed for the move;
- delete or weaken any current fixture test.

## Implementation Notes

The implementation plan should:

1. Require a pre-edit worktree snapshot with target-file checks:
   - `git status --short --untracked-files=all`
   - `git diff -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/tests`
   - `git diff --cached -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/tests`
   - `git ls-files src-tauri/src/analysis/fixtures/tests`
   - if `src-tauri/src/analysis/fixtures/tests/` exists in any form, print its contents with `Get-ChildItem -Recurse`; for untracked files, also print `Get-Content -Raw` for each file before continuing.
2. Stop before editing if `fixtures.rs` is dirty, if any target `fixtures/tests/*` file is dirty, or if `fixtures/tests/` already exists tracked or untracked without an explicit baseline decision.
3. Run baseline verification before editing:
   - `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::`
   - `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`
4. Require the fixture baseline test run to be in the default dev test profile; do not use `--release` for the required fixture slice.
5. Replace the inline test module with `#[cfg(test)] mod tests;`.
6. Move helpers and tests into thematic files without changing test bodies except imports and module paths.
7. Keep imports explicit in each test module. Avoid glob imports from `crate`, and do not use `use super::super::*`; import parent fixture functions, constants, and types by name.
8. Preserve parent facade access for seed and lifecycle helpers.
9. Run `cargo fmt --manifest-path src-tauri/Cargo.toml` only if needed, then run `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.
10. Inspect `git status --short --untracked-files=all` and changed file list after formatting. Behavioral diff should be limited to:
    - `src-tauri/src/analysis/fixtures.rs`
    - `src-tauri/src/analysis/fixtures/tests/mod.rs`
    - `src-tauri/src/analysis/fixtures/tests/harness.rs`
    - `src-tauri/src/analysis/fixtures/tests/summary.rs`
    - `src-tauri/src/analysis/fixtures/tests/clear.rs`
    - `src-tauri/src/analysis/fixtures/tests/seed.rs`
    - `src-tauri/src/analysis/fixtures/tests/snapshot.rs`
    - `src-tauri/src/analysis/fixtures/tests/active_runs.rs`
11. Stage only intended files. Existing unrelated files, including ignored/generated files, must not be staged.

For PowerShell snippets that verify absence, use fail-fast checks rather than commands that merely print matches. Example:

```powershell
$matches = @(rg -n "super::super::seed::|crate::analysis::fixtures::seed::" src-tauri/src/analysis/fixtures/tests)
if ($matches.Count -ne 0) {
    $matches
    throw "fixture tests must use parent facade access, not the private seed module path"
}
```

Use per-symbol checks when verifying required moved tests or modules; do not rely on a single alternation `rg` command whose success only proves one match.

## Source Guards

Run source guards after the move.

`fixtures.rs` should keep only test module wiring:

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

No inline test bodies remain in `fixtures.rs`:

```powershell
$inlineTestMatches = @(rg -n "#\[tokio::test\]|^mod tests \{|use super::\*|SqlitePoolOptions|^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn) (fixture_pool|count|insert_minimal_clear_fixture)\b" src-tauri/src/analysis/fixtures.rs)
if ($inlineTestMatches.Count -ne 0) {
    $inlineTestMatches
    throw "inline fixture test body or helper remains in fixtures.rs"
}
```

Expected: no output and no throw.

Production helper visibility remains private:

```powershell
rg -n "^\s*pub(\([^)]*\))?\s+(async\s+fn|fn) (fixture_run_ids|register_fixture_active_runs|remove_fixture_active_runs|finish_cancelled_fixture_run|spawn_fixture_cancellation_waiters|clear_analysis_redesign_fixtures_in_pool|rows_to_i64)\b" src-tauri/src/analysis/fixtures.rs
```

Expected: no matches. Exit code `1` is expected.

Fixture constants remain private:

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

Required test files exist:

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

`tests/mod.rs` declares every thematic module independently:

```powershell
foreach ($module in @("active_runs", "clear", "harness", "seed", "snapshot", "summary")) {
    rg -n "^mod $module;$" src-tauri/src/analysis/fixtures/tests/mod.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing fixtures test module declaration: $module"
    }
}
```

No test module imports the private production seed module directly:

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

`tests/mod.rs` stays wiring-only:

```powershell
$testModBodyMatches = @(rg -n "#\[tokio::test\]|^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn)\b|use super::|use crate::" src-tauri/src/analysis/fixtures/tests/mod.rs)
if ($testModBodyMatches.Count -ne 0) {
    $testModBodyMatches
    throw "fixtures tests/mod.rs must contain module declarations only"
}
```

Expected: no output and no throw.

Test modules avoid parent and crate glob imports:

```powershell
$globImportMatches = @(rg -n "use\s+super::super::\*|use\s+crate::.*::\*" src-tauri/src/analysis/fixtures/tests)
if ($globImportMatches.Count -ne 0) {
    $globImportMatches
    throw "fixture test modules must use explicit imports, not parent or crate glob imports"
}
```

Expected: no output and no throw.

Required tests moved to expected modules:

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

Expected: every required test is checked independently; the command throws on the first missing test.

## Testing

Run commands separately, not as one PowerShell block, unless using an explicit stopping wrapper that checks `$LASTEXITCODE`.

Baseline before editing:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass. This establishes crate-wide compile coverage before the test module-boundary refactor.

Post-change focused module slices:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::summary::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::harness::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::clear::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::snapshot::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::active_runs::
```

Expected for each focused module slice: pass in the default dev test profile and not a green `0 tests` run.

Post-change test inventory:

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

Post-change full fixture slice:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

Post-change compile and format:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: both pass.

Because `analysis::fixtures::tests::` depends on debug-only fixture commands and debug assertions, all required fixture test commands must run in the default dev test profile. Do not use `--release` for the required fixture slices.

Before accepting a test command as coverage, check that the output includes real tests for the intended module. A green `0 tests` run is a failure for every filtered command listed here.

## Commit Shape

Expected implementation commit:

- `refactor: split analysis fixture tests`

Expected files in that commit:

- `src-tauri/src/analysis/fixtures.rs`
- `src-tauri/src/analysis/fixtures/tests/mod.rs`
- `src-tauri/src/analysis/fixtures/tests/harness.rs`
- `src-tauri/src/analysis/fixtures/tests/summary.rs`
- `src-tauri/src/analysis/fixtures/tests/clear.rs`
- `src-tauri/src/analysis/fixtures/tests/seed.rs`
- `src-tauri/src/analysis/fixtures/tests/snapshot.rs`
- `src-tauri/src/analysis/fixtures/tests/active_runs.rs`

Do not include unrelated rustfmt drift. If `cargo fmt` changes unrelated Rust files, inspect the drift and either make a separate format-only commit or restore only implementation-owned formatting changes after review. The final implementation status should be clean except for explicitly pre-existing unrelated files.

## Open Questions

None. The design intentionally keeps production fixture lifecycle code in `fixtures.rs` and only moves tests.
