# Analysis Fixtures Seed Runs Refactor Design

**Date:** 2026-07-03
**Status:** active spec; implementation not started as of 2026-07-03 because `src-tauri/src/analysis/fixtures/seed/runs.rs` does not exist.
**Scope:** internal Rust refactor of analysis redesign fixture run/snapshot seed-writing logic.

## Goal

Reduce the size and responsibility of `src-tauri/src/analysis/fixtures/seed.rs` by extracting fixture run, snapshot-message, trace, and chat-message seed logic into a focused private child module, without changing debug Tauri commands, fixture rows, run statuses, snapshot states, trace payloads, chat messages, SQL, summary counts, test coverage, or public debug command paths.

This is the next conservative fixtures slice after splitting fixture tests out of `fixtures.rs`. The remaining largest fixtures file is `fixtures/seed.rs`; its lower half is a coherent run/snapshot writer that can move without touching source/content setup.

## Current Shape

`src-tauri/src/analysis/fixtures/seed.rs` currently owns:

- compression helper `json_zstd`;
- account, prompt template, and LLM profile fixture writers;
- Telegram and YouTube source writers;
- source group writer;
- shared item writer;
- Telegram content writer;
- YouTube transcript/comment/playlist content writer;
- run/snapshot helper cluster:
  - `FixtureIds`
  - `insert_run`
  - `insert_snapshot_message`
  - `mark_fixture_snapshot_captured`
  - `mark_fixture_snapshot_capture_failed`
  - `trace_zstd`
  - `first_item_id`
  - `insert_analysis_runs`
- orchestration entry point `seed_analysis_redesign_fixtures_in_pool`.

The run/snapshot helper cluster is used only by `seed_analysis_redesign_fixtures_in_pool`. It creates:

- completed YouTube snapshot run;
- missing snapshot run;
- capture-failed snapshot run;
- running, failed, and cancelled single-source runs;
- completed Telegram source-group snapshot run;
- saved snapshot messages;
- trace refs;
- fixture analysis chat messages.

Current consumers:

- `seed_analysis_redesign_fixtures_in_pool` constructs `FixtureIds` and calls `insert_analysis_runs`;
- fixture tests exercise this behavior through the parent fixture facade and store read-model APIs;
- no code outside `analysis::fixtures::seed` calls the run/snapshot helpers directly.

## Proposed Architecture

Create a private child module declared from `src-tauri/src/analysis/fixtures/seed.rs`:

- `src-tauri/src/analysis/fixtures/seed/runs.rs`

Keep `src-tauri/src/analysis/fixtures/seed.rs` as the seed orchestration and source/content writer facade:

- add `mod runs;`;
- add a private import:

```rust
use self::runs::{insert_analysis_runs, FixtureIds};
```

Move these items from `fixtures/seed.rs` to `fixtures/seed/runs.rs`:

- `FixtureIds`
- `insert_run`
- `insert_snapshot_message`
- `mark_fixture_snapshot_captured`
- `mark_fixture_snapshot_capture_failed`
- `trace_zstd`
- `first_item_id`
- `insert_analysis_runs`

Keep these items in `fixtures/seed.rs` for this slice:

- `json_zstd`;
- account/prompt/profile writers;
- Telegram/YouTube source writers;
- source group writer;
- `insert_item`;
- `insert_telegram_content`;
- `insert_youtube_content`;
- `seed_analysis_redesign_fixtures_in_pool`.

The parent `seed.rs` should still orchestrate the order:

1. clear existing fixtures;
2. create account, prompt template, LLM profile;
3. create Telegram and YouTube sources;
4. create source group;
5. insert Telegram content;
6. insert YouTube content;
7. call `insert_analysis_runs` with `FixtureIds`;
8. commit transaction;
9. return the clear-count summary.

## Visibility

`fixtures/seed/runs.rs` should expose only the minimal surface needed by parent `seed.rs`:

```rust
pub(super) struct FixtureIds {
    pub(super) prompt_template_id: i64,
    pub(super) telegram_channel_id: i64,
    pub(super) telegram_supergroup_id: i64,
    pub(super) youtube_video_id: i64,
    pub(super) source_group_id: i64,
}

pub(super) async fn insert_analysis_runs(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    ids: FixtureIds,
) -> AppResult<()>;
```

The `FixtureIds` fields must be `pub(super)` because parent `seed.rs` constructs the struct. Do not widen the struct, fields, or function beyond `pub(super)`.

All other moved helpers stay private inside `runs.rs`:

- `insert_run`
- `insert_snapshot_message`
- `mark_fixture_snapshot_captured`
- `mark_fixture_snapshot_capture_failed`
- `trace_zstd`
- `first_item_id`

`json_zstd` remains private in parent `seed.rs`. The child `runs.rs` may call it through `super::json_zstd`; no widening is required because child modules can access private parent items.

Expected production API changes outside `analysis::fixtures::seed`: none.

Expected root re-export changes in `analysis/mod.rs`: none.

Expected debug command registration changes in `lib.rs`: none.

## Imports

`fixtures/seed/runs.rs` should own imports needed by run/snapshot seed logic:

- `sqlx::Sqlite`
- `crate::error::{AppError, AppResult}`
- parent helper import `super::json_zstd`
- fixture constants from `super::super`

The fixture constants imported by `runs.rs` should be explicit, not glob imports. Import them with a named import list from `super::super` so `seed.rs` can remove moved-only constants from its own parent import list. The moved code uses:

- `CANCELLED_RUN_LABEL`
- `CAPTURE_FAILED_SNAPSHOT_ERROR`
- `CAPTURE_FAILED_SNAPSHOT_RUN_LABEL`
- `COMPLETED_SNAPSHOT_RUN_LABEL`
- `FAILED_RUN_LABEL`
- `FIXTURE_EXTERNAL_PREFIX`
- `FIXTURE_NOW`
- `FIXTURE_PERIOD_FROM`
- `FIXTURE_PERIOD_TO`
- `FIXTURE_PROFILE_ID`
- `FIXTURE_SNAPSHOT_CAPTURED_AT`
- `GROUP_SNAPSHOT_RUN_LABEL`
- `LLM_PROFILE_LABEL`
- `MISSING_SNAPSHOT_RUN_LABEL`
- `RUNNING_RUN_LABEL`
- `YOUTUBE_VIDEO_LABEL`

After the move, remove moved-only constants from the parent `seed.rs` import list if parent source/content setup no longer uses them directly. Keep constants still used by parent seed/source/content setup in `seed.rs`.

## Data Flow

No runtime data flow changes:

1. `seed_analysis_redesign_fixtures_in_pool` still starts one transaction.
2. Source and content rows are still written before analysis run rows.
3. `insert_analysis_runs` still receives the same prompt/source/group IDs.
4. Run rows, snapshot messages, trace payloads, snapshot markers, and chat messages are written in the same transaction.
5. The transaction still commits once at the end.
6. Tests still reach the behavior through debug fixture seed facade and store read-model APIs.

Only Rust definition locations change.

## Error Handling

Preserve current error behavior exactly:

- all SQL errors still map through `AppError::database`;
- compression and JSON serialization errors still map through `AppError::internal`;
- no new error codes, messages, or user-facing strings are introduced;
- transaction behavior remains all-or-nothing.

Preserve these moved fixture strings exactly:

- `This capture-failed fixture report remains readable.`
- `Fixture failure: provider request failed without changing user data`
- `Fixture cancellation: run was cancelled before snapshot capture`
- `Fixture timestamp segment supports Show in source.`
- `fixture channel update: result-first analysis now has source evidence`
- `Fixture evidence highlights saved snapshots, YouTube timestamps, and Telegram source context.`
- `transcript_description_comments`

The implementation plan must include source guards proving these markers are present in `fixtures/seed/runs.rs` after the move.

## Non-Goals

This slice does not:

- move account, prompt template, LLM profile, source, source group, item, Telegram content, or YouTube content writers;
- split YouTube source metadata setup;
- split Telegram source setup;
- move or edit debug Tauri commands;
- move or edit fixture tests;
- change `AnalysisRedesignFixtureSummary`;
- change fixture constants in `fixtures.rs`;
- change SQL, fixture data, trace payloads, snapshot states, status strings, chat messages, compressed payloads, database migrations, frontend code, or Tauri command payloads;
- add new behavior tests beyond path/coverage guards needed for the move;
- delete or weaken any current fixture test.

## Implementation Notes

The implementation plan should:

1. Require a pre-edit worktree snapshot:
   - `git status --short --untracked-files=all`
   - `git diff -- src-tauri/src/analysis/fixtures/seed.rs src-tauri/src/analysis/fixtures/seed`
   - `git diff --cached -- src-tauri/src/analysis/fixtures/seed.rs src-tauri/src/analysis/fixtures/seed`
   - `git ls-files src-tauri/src/analysis/fixtures/seed`
   - if `src-tauri/src/analysis/fixtures/seed/` exists in any form, print its contents with `Get-ChildItem -Recurse`; for untracked files, also print `Get-Content -Raw` for each file before continuing.
2. Stop before editing if `seed.rs` is dirty, if any target `seed/*` file is dirty, or if `seed/` already exists tracked or untracked without an explicit baseline decision.
3. Run baseline verification before editing:
   - `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed::`
   - `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::snapshot::`
   - `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::active_runs::`
   - `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::`
   - `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`
4. Require fixture tests to run in the default dev test profile; do not use `--release` for required fixture slices.
5. Add `mod runs;` near the top of `seed.rs`, before imports from the child module.
6. Add `use self::runs::{insert_analysis_runs, FixtureIds};`.
7. Move only the run/snapshot helper cluster into `runs.rs`.
8. Keep imports explicit in both files. Do not use glob imports.
9. Run `cargo fmt --manifest-path src-tauri/Cargo.toml` only if needed, then run `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.
10. Inspect `git status --short --untracked-files=all` and changed file list after formatting. Behavioral diff should be limited to:
    - `src-tauri/src/analysis/fixtures/seed.rs`
    - `src-tauri/src/analysis/fixtures/seed/runs.rs`
11. Stage only intended files. Existing unrelated files, including ignored/generated files, must not be staged.

## Source Guards

Run source guards after the move.

`seed.rs` should declare and privately import the child module:

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

Moved definitions no longer remain in `seed.rs`:

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

Moved-only run constants no longer remain in `seed.rs`:

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

Moved definitions exist in `runs.rs` with expected visibility:

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

`FixtureIds` fields remain visible only to parent `seed.rs`:

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

No unintended public API in `runs.rs`:

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

Private moved helper definitions exist in `runs.rs`:

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

Moved behavior markers live in `runs.rs`:

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

`runs.rs` does not use glob imports:

```powershell
$globMatches = @(rg -n "use\s+.*::\*" src-tauri/src/analysis/fixtures/seed/runs.rs)
if ($globMatches.Count -ne 0) {
    $globMatches
    throw "seed/runs.rs must use explicit imports"
}
```

Expected: no output and no throw.

`runs.rs` imports fixture constants from the grandparent fixture module, not through parent `seed.rs` reimports:

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

## Testing

Run commands separately, not as one PowerShell block, unless using an explicit stopping wrapper that checks `$LASTEXITCODE`.

Baseline before editing:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::snapshot::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::active_runs::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected for each baseline fixture command: pass in the default dev test profile and not a green `0 tests` run.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass. This establishes crate-wide compile coverage before the module-boundary refactor.

Post-change focused tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::seed::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::snapshot::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::active_runs::
```

Expected for each focused command: pass in the default dev test profile and not a green `0 tests` run.

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

Before accepting any filtered test command, check that the output includes real tests for the intended module. A green `0 tests` run is a failure for every filtered command listed here.

## Commit Shape

Expected implementation commit:

- `refactor: extract fixture seed run writers`

Expected files in that commit:

- `src-tauri/src/analysis/fixtures/seed.rs`
- `src-tauri/src/analysis/fixtures/seed/runs.rs`

Do not include unrelated rustfmt drift. If `cargo fmt` changes unrelated Rust files, inspect the drift and either make a separate format-only commit or restore only implementation-owned formatting changes after review. The final implementation status should be clean except for explicitly pre-existing unrelated files.

## Open Questions

None. The design intentionally keeps source/content fixture setup in `seed.rs` and only moves run/snapshot seed writers.
