# Analysis Fixtures Seed Refactor Design

**Date:** 2026-07-03
**Status:** active spec; implementation not started as of 2026-07-03 because `src-tauri/src/analysis/fixtures/seed.rs` does not exist.
**Scope:** internal Rust refactor of debug-only analysis redesign fixture seed/data-writing logic.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/fixtures.rs` by extracting fixture data-writing and seed orchestration helpers into a focused private child module, without changing debug Tauri commands, active-run state behavior, cancellation waiters, clear behavior, fixture rows, summary counts, SQL, metadata payloads, tests, or public debug command paths.

This is the first conservative fixtures slice after the store refactors. It intentionally avoids moving the debug command surface, `AnalysisState` active-run helpers, clear/delete logic, or fixture tests.

## Current Shape

`src-tauri/src/analysis/fixtures.rs` currently owns:

- debug-only Tauri commands:
  - `seed_analysis_redesign_fixtures`
  - `clear_analysis_redesign_fixtures`
  - `clear_analysis_redesign_fixture_active_runs`
- `AnalysisRedesignFixtureSummary`;
- fixture marker constants and deterministic timestamps;
- active fixture run tracking and cancellation waiter helpers;
- fixture seed/data-writing helpers for accounts, prompt templates, LLM profile settings, Telegram sources, YouTube sources, source groups, content, runs, snapshots, traces, and chat messages;
- clear/delete logic for fixture rows;
- inline tests for summary serialization, clear behavior, seed behavior, seeded run states, trace refs, snapshot states, active run state, cancellation waiters, and deterministic reseeding.

The production seed cluster currently lives directly in `fixtures.rs`:

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

Current consumers:

- `seed_analysis_redesign_fixtures` calls `seed_analysis_redesign_fixtures_in_pool`;
- inline `fixtures.rs` tests call `seed_analysis_redesign_fixtures_in_pool`;
- `analysis/mod.rs` re-exports debug commands from `fixtures.rs` under `#[cfg(debug_assertions)]`;
- `lib.rs` registers those debug commands under `#[cfg(debug_assertions)]`.

## Proposed Architecture

Create a private child module declared from `src-tauri/src/analysis/fixtures.rs`:

- `src-tauri/src/analysis/fixtures/seed.rs`

Keep `src-tauri/src/analysis/fixtures.rs` as the debug command and fixture lifecycle facade:

- add `mod seed;`;
- add a private root import:

```rust
use self::seed::seed_analysis_redesign_fixtures_in_pool;
```

- keep public debug commands in `fixtures.rs`;
- keep active-run registration, active-run removal, and cancellation waiter helpers in `fixtures.rs`;
- keep `clear_analysis_redesign_fixtures_in_pool` and `rows_to_i64` in `fixtures.rs`;
- keep all current tests in `fixtures.rs` for this slice.

Move these items from `fixtures.rs` to `fixtures/seed.rs`:

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

Keep these items in `fixtures.rs` for this slice:

- all fixture marker constants;
- `AnalysisRedesignFixtureSummary`;
- debug Tauri commands;
- `fixture_run_ids`;
- `register_fixture_active_runs`;
- `remove_fixture_active_runs`;
- `finish_cancelled_fixture_run`;
- `spawn_fixture_cancellation_waiters`;
- `clear_analysis_redesign_fixtures_in_pool`;
- `rows_to_i64`;
- all current tests.

The inline test module stays in `fixtures.rs` for this slice. Moving fixture tests can be a later test-only refactor.

## Visibility

`fixtures/seed.rs` should expose only the seed entry point needed by the parent fixture facade:

```rust
pub(super) async fn seed_analysis_redesign_fixtures_in_pool(
    pool: &Pool<Sqlite>,
) -> AppResult<AnalysisRedesignFixtureSummary>;
```

All other moved seed helpers stay private inside `seed.rs`:

- `json_zstd`
- all `insert_*` helpers;
- `FixtureIds`;
- `mark_fixture_snapshot_captured`;
- `mark_fixture_snapshot_capture_failed`;
- `trace_zstd`;
- `first_item_id`.

`clear_analysis_redesign_fixtures_in_pool` remains private in `fixtures.rs`. The child seed module may call it through `super::clear_analysis_redesign_fixtures_in_pool`; no visibility widening is required because child modules can access private parent items.

Expected production API changes outside `analysis::fixtures`: none.

Expected root re-export changes in `analysis/mod.rs`: none.

Expected debug command registration changes in `lib.rs`: none.

## Imports

`fixtures/seed.rs` should own imports needed by seed/data-writing logic:

- `sqlx::{Pool, Sqlite}`
- `crate::error::{AppError, AppResult}`
- `crate::youtube::dto::{YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata}`
- parent constants and types from `super`, including `AnalysisRedesignFixtureSummary`, `clear_analysis_redesign_fixtures_in_pool`, and the fixture marker/status/label constants used by moved seed logic.

The parent imports from `super` should be explicit, not glob imports. The implementation plan should list all imported constants that the moved code actually uses, including:

- `FIXTURE_MARKER`
- `FIXTURE_EXTERNAL_PREFIX`
- `FIXTURE_PROFILE_ID`
- `FIXTURE_NOW`
- `FIXTURE_PERIOD_FROM`
- `FIXTURE_PERIOD_TO`
- `TELEGRAM_CHANNEL_LABEL`
- `TELEGRAM_SUPERGROUP_LABEL`
- `YOUTUBE_VIDEO_LABEL`
- `YOUTUBE_PLAYLIST_LABEL`
- `YOUTUBE_FIXTURE_VIDEO_ID`
- `YOUTUBE_FIXTURE_PLAYLIST_ID`
- `TELEGRAM_FIXTURE_CHANNEL_PEER_ID`
- `TELEGRAM_FIXTURE_SUPERGROUP_PEER_ID`
- `TELEGRAM_GROUP_LABEL`
- `COMPLETED_SNAPSHOT_RUN_LABEL`
- `MISSING_SNAPSHOT_RUN_LABEL`
- `CAPTURE_FAILED_SNAPSHOT_RUN_LABEL`
- `CAPTURE_FAILED_SNAPSHOT_ERROR`
- `RUNNING_RUN_LABEL`
- `FAILED_RUN_LABEL`
- `CANCELLED_RUN_LABEL`
- `GROUP_SNAPSHOT_RUN_LABEL`
- `LLM_PROFILE_LABEL`
- `FIXTURE_SNAPSHOT_CAPTURED_AT`

`fixtures.rs` should remove production imports that only moved seed helpers use after extraction:

- YouTube DTO imports, if only `seed.rs` uses them;
- any moved-only compression helper imports, if introduced during implementation.

`fixtures.rs` should keep imports needed by debug commands, active run helpers, clear logic, and tests:

- `serde::Serialize`
- `sqlx::{Pool, Sqlite}`
- `tauri::{AppHandle, Manager, State}`
- `super::store::set_run_status`
- `super::AnalysisState`
- `crate::db::get_pool`
- `crate::error::{AppError, AppResult}`
- `crate::time::now_secs`

The implementation plan must include a production-import guard that checks the section of `fixtures.rs` before `#[cfg(test)] mod tests`; moved-only imports must not remain in the parent production import block.

## Data Flow

No runtime data flow changes:

1. `seed_analysis_redesign_fixtures` still obtains the app SQLite pool, removes active fixture runs, seeds fixtures, registers active running fixture runs, spawns cancellation waiters, and returns the same summary.
2. `seed_analysis_redesign_fixtures_in_pool` still clears existing fixtures before starting the seed transaction.
3. Seed insertion order stays the same: account, prompt template, LLM profile settings, Telegram sources, YouTube video source, YouTube playlist source, source group, Telegram content, YouTube content, analysis runs, commit.
4. Fixture ids and labels remain deterministic.
5. YouTube video and playlist metadata payloads stay byte-for-byte equivalent after serialization.
6. Telegram item rows, YouTube item rows, transcript segments, playlist items, snapshot messages, trace refs, chat messages, and run status rows keep the same values.
7. The returned `AnalysisRedesignFixtureSummary` stays:

```rust
AnalysisRedesignFixtureSummary {
    accounts: 1,
    llm_profiles: 1,
    sources: 4,
    source_groups: 1,
    prompt_templates: 1,
    runs: 7,
    snapshot_messages: 4,
    chat_messages: 2,
    youtube_transcript_segments: 3,
    youtube_playlist_items: 2,
}
```

8. Clear behavior stays in `fixtures.rs` and is not changed in this slice.
9. Active run cancellation waiter behavior stays in `fixtures.rs` and is not changed in this slice.

## Error Handling

Preserve current error behavior exactly:

- JSON serialization errors still map through `AppError::internal`;
- compression failures still map through `AppError::internal`;
- database failures still use `AppError::database`;
- transaction begin and commit failures still use `AppError::database`;
- seed still calls clear first and propagates clear failures before opening the seed transaction;
- no new error codes, messages, SQL filters, DTO fields, migrations, or user-facing strings are introduced.

The implementation plan must include source guards for these literals and behavior markers after the move:

```powershell
rg -n -F "analysis_redesign_fixture" src-tauri/src/analysis/fixtures/seed.rs
rg -n -F "Fixture timestamp segment supports Show in source." src-tauri/src/analysis/fixtures/seed.rs
rg -n -F "transcript_description_comments" src-tauri/src/analysis/fixtures/seed.rs
rg -n -F "INSERT INTO analysis_runs" src-tauri/src/analysis/fixtures/seed.rs
rg -n -F "INSERT INTO analysis_run_messages" src-tauri/src/analysis/fixtures/seed.rs
rg -n -F "INSERT INTO analysis_chat_messages" src-tauri/src/analysis/fixtures/seed.rs
rg -n -F "AnalysisRedesignFixtureSummary {" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: all moved seed behavior markers are present in `seed.rs`.

## Non-Goals

This slice does not:

- move debug Tauri command functions;
- move active-run registration, removal, or cancellation waiter logic;
- move clear/delete logic;
- move `AnalysisRedesignFixtureSummary`;
- split fixture tests into files;
- change `analysis/mod.rs` re-exports or `lib.rs` command registration;
- change fixture marker strings, labels, timestamps, peer ids, source ids, metadata payloads, SQL, row counts, status values, snapshot values, trace refs, compression, database schema, migrations, frontend code, Tauri command payloads, or event payloads.

## Implementation Hygiene

The implementation plan must include a pre-edit worktree guard:

```powershell
git status --short --untracked-files=all
```

Target files must be clean before editing. If `src-tauri/src/analysis/fixtures.rs` or `src-tauri/src/analysis/fixtures/seed.rs` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline commit before starting this refactor. This is required because the implementation plan should use full-file staging for the two target Rust files.

Inspect tracked target-file diffs before editing:

```powershell
git diff -- src-tauri/src/analysis/fixtures.rs
git diff --cached -- src-tauri/src/analysis/fixtures.rs
```

If `src-tauri/src/analysis/fixtures/seed.rs` exists before execution, inspect it directly because untracked files are not shown by normal `git diff`:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/fixtures/seed.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/fixtures/seed.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/fixtures/seed.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/fixtures/seed.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/fixtures/seed.rs'
}
```

Do not stage unrelated dirty files, such as local tool settings. Unrelated dirty files must remain unstaged and must be accounted for in baseline/final status comparisons.

The implementation plan must capture pre-edit status using a unique tag and persist the paths for later PowerShell sessions:

```powershell
$tag = "analysis-fixtures-seed-refactor-" + (Get-Date -Format "yyyyMMddHHmmss")
$preEditStatusPath = Join-Path $env:TEMP "$tag-status-before.txt"
$preEditStatusPointerPath = Join-Path $env:TEMP "analysis-fixtures-seed-refactor-status-pointer.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $preEditStatusPath
$preEditStatusPath | Set-Content -LiteralPath $preEditStatusPointerPath
$preEditStatusPointerPath
```

Expected: the command prints the pointer-file path. Later commands must read that pointer file to recover the actual status snapshot path, so the workflow works across separate PowerShell sessions.

Before commit, compare the final status to the captured baseline and confirm no new unintended files or diffs exist outside:

- `src-tauri/src/analysis/fixtures.rs`
- `src-tauri/src/analysis/fixtures/seed.rs`

Use a command that reads the persisted pointer instead of relying on an in-memory PowerShell variable:

```powershell
$preEditStatusPointerPath = Join-Path $env:TEMP "analysis-fixtures-seed-refactor-status-pointer.txt"
$preEditStatusPath = Get-Content -LiteralPath $preEditStatusPointerPath
$before = Get-Content -LiteralPath $preEditStatusPath
$afterPath = Join-Path $env:TEMP "analysis-fixtures-seed-refactor-status-after.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $afterPath
$after = Get-Content -LiteralPath $afterPath
Compare-Object -ReferenceObject $before -DifferenceObject $after
```

Expected: differences are limited to the two intended target files. Pre-existing unrelated files may appear in both before and after and must remain unstaged.

If `cargo fmt` rewrites unrelated Rust files, resolve that drift before the refactor commit by making a separate format-only commit or restoring only implementation-owned formatting changes after review. Final status should return to the captured baseline except for intended staged refactor files.

Stage only implementation-owned files for this refactor. Do not stage local tool settings or unrelated docs.

## Source Guards

The implementation plan must include source guards after the move.

Private module declaration:

```powershell
rg -n "^mod seed;" src-tauri/src/analysis/fixtures.rs
rg -n "^pub.*mod seed" src-tauri/src/analysis/fixtures.rs
```

Expected: first command has one match; second command has no matches. `rg` exit code `1` is expected for no-match guards.

Private root import:

```powershell
rg -n "^use self::seed::seed_analysis_redesign_fixtures_in_pool;" src-tauri/src/analysis/fixtures.rs
rg -n "^pub.*seed_analysis_redesign_fixtures_in_pool" src-tauri/src/analysis/fixtures.rs
```

Expected: first command has one match; second command has no matches. The seed entry point should not be publicly re-exported.

Moved definitions must not remain in `fixtures.rs`:

```powershell
rg -n "^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn) (json_zstd|insert_fixture_account|insert_fixture_prompt_template|insert_fixture_llm_profile|insert_telegram_source|insert_youtube_video_source|insert_youtube_playlist_source|insert_fixture_source_group|insert_item|insert_telegram_content|insert_youtube_content|insert_run|insert_snapshot_message|mark_fixture_snapshot_captured|mark_fixture_snapshot_capture_failed|trace_zstd|first_item_id|insert_analysis_runs|seed_analysis_redesign_fixtures_in_pool)\b" src-tauri/src/analysis/fixtures.rs
rg -n "^\s*(pub(\([^)]*\))?\s+)?struct FixtureIds\b" src-tauri/src/analysis/fixtures.rs
```

Expected: no matches. `rg` exit code `1` is expected. Test calls may still mention `seed_analysis_redesign_fixtures_in_pool`, but production definitions must be gone.

Moved definitions must exist in `seed.rs`:

```powershell
rg -n "^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn) (json_zstd|insert_fixture_account|insert_fixture_prompt_template|insert_fixture_llm_profile|insert_telegram_source|insert_youtube_video_source|insert_youtube_playlist_source|insert_fixture_source_group|insert_item|insert_telegram_content|insert_youtube_content|insert_run|insert_snapshot_message|mark_fixture_snapshot_captured|mark_fixture_snapshot_capture_failed|trace_zstd|first_item_id|insert_analysis_runs)\b" src-tauri/src/analysis/fixtures/seed.rs
rg -n "^pub\(super\) async fn seed_analysis_redesign_fixtures_in_pool" src-tauri/src/analysis/fixtures/seed.rs
rg -n "^struct FixtureIds\b" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: all moved seed helpers exist in `seed.rs`, and only the seed entry point is `pub(super)`.

No unintended public API in `seed.rs`:

```powershell
rg -n "^\s*pub(\([^)]*\))?\s+" src-tauri/src/analysis/fixtures/seed.rs
```

Expected: exactly one match, and it must be `pub(super) async fn seed_analysis_redesign_fixtures_in_pool`. Any other public or restricted-public item, including `pub(super)` helpers, `pub(crate)` items, `pub(in crate::analysis)` items, or plain `pub` items, is a failure.

Inline tests should keep covering the parent facade instead of the private child module:

```powershell
$testSection = [regex]::Split((Get-Content -Raw src-tauri/src/analysis/fixtures.rs), "#\[cfg\(test\)\]", 2)[1]
$testSection | Select-String -Pattern "super::seed|seed::seed_analysis_redesign_fixtures_in_pool|crate::analysis::fixtures::seed"
```

Expected: no matches. Inline tests should call `seed_analysis_redesign_fixtures_in_pool` through the parent private import, not through `super::seed` or any direct private-module path.

Production import cleanup guard:

```powershell
$beforeTests = [regex]::Split((Get-Content -Raw src-tauri/src/analysis/fixtures.rs), "#\[cfg\(test\)\]", 2)[0]
$beforeTests | Select-String -Pattern "YoutubeAvailabilityStatus|YoutubePlaylistMetadata|YoutubeVideoForm|YoutubeVideoMetadata|compress_text|compress_json_bytes|crate::compression"
```

Expected: no moved-only YouTube DTO or compression imports remain in the production section of `fixtures.rs`.

Public debug exports remain unchanged:

```powershell
rg -n "pub use self::fixtures::\{" src-tauri/src/analysis/mod.rs
rg -n "seed_analysis_redesign_fixtures|clear_analysis_redesign_fixtures|clear_analysis_redesign_fixture_active_runs" src-tauri/src/analysis/mod.rs src-tauri/src/lib.rs
```

Expected: existing debug command re-exports and command registration are still present.

## Testing

Run required commands from the repository root with `--manifest-path src-tauri/Cargo.toml`. Run each command separately or through a wrapper that stops on non-zero `$LASTEXITCODE`; do not place multiple `cargo` commands in one plain PowerShell block.

Baseline before editing:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_safe_account_prompt_profile_sources_and_group
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_fixture_runs_with_statuses_templates_and_snapshots
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::fixture_active_state_tracks_seeded_running_run
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_twice_keeps_one_deterministic_fixture_set
```

Expected: every baseline command passes in the default dev test profile and is not a green `0 tests` run. Do not use `--release`; the fixture module is gated by `#[cfg(debug_assertions)]`.

Post-change verification:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_safe_account_prompt_profile_sources_and_group
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_creates_fixture_runs_with_statuses_templates_and_snapshots
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::fixture_active_state_tracks_seeded_running_run
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::fixture_cancel_waiter_marks_running_run_cancelled
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed_twice_keeps_one_deterministic_fixture_set
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected:

- every focused fixture test passes in the default dev test profile and is not a green `0 tests` run;
- `analysis::fixtures::tests::` passes in the default dev test profile and is not a green `0 tests` run;
- `cargo check --all-targets` passes, covering debug command re-exports, `lib.rs` command registration, parent-to-child seed module visibility, and tests;
- `cargo fmt -- --check` passes after any formatting fix. If formatting fixes are required, run `cargo fmt`, inspect changed files with `git status --short --untracked-files=all`, resolve unrelated drift, then run `cargo fmt -- --check` again before staging.

## Commit Shape

The implementation should produce one focused refactor commit that contains only:

- `src-tauri/src/analysis/fixtures.rs`
- `src-tauri/src/analysis/fixtures/seed.rs`

Documentation hardening commits may be separate, as in prior refactor slices.

Before committing:

```powershell
git status --short --untracked-files=all
git diff -- src-tauri/src/analysis/fixtures.rs src-tauri/src/analysis/fixtures/seed.rs
git diff --cached --check
```

Run the git commands separately or through a stopping wrapper. Do not rely on a plain multi-command PowerShell block for failure handling.
