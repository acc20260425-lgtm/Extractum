# Analysis State and Events Refactor Design

**Date:** 2026-06-30
**Status:** agreed first slice, ready for implementation planning after review
**Scope:** internal Rust refactor of `src-tauri/src/analysis/` only.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/mod.rs` without changing behavior. The first refactor slice extracts analysis run state and event emission helpers into focused modules so later `corpus`, `store`, and `report` refactors have cleaner internal boundaries.

This slice is intentionally conservative: no database migrations, no Tauri command contract changes, no event payload changes, and no frontend changes.

## Current Shape

`analysis/mod.rs` currently owns several unrelated concerns:

- module declarations and public re-exports;
- shared constants for run statuses, scope types, template kinds, and event names;
- `AnalysisState`, including active report run IDs and cancellation tokens;
- `emit_analysis_event` and `emit_analysis_chat_event`;
- validation helpers for chat turns;
- top-level Tauri commands for listing runs, messages, traces, and deletion.

The state helper is small, but it is a central dependency for `report.rs`,
`analysis/report_commands.rs`, debug-only `fixtures.rs`, `analysis/mod.rs`
command handlers, `accounts.rs`, account deletion logic, the project analysis
command, and Tauri state registration in `lib.rs`. The event helpers are used
by `report.rs` and `chat.rs`. Keeping both helper groups in `mod.rs` makes the
module root harder to scan and makes future extraction work noisier.

## Proposed Architecture

Add two new internal modules:

- `src-tauri/src/analysis/state.rs`
- `src-tauri/src/analysis/events.rs`

`state.rs` will contain:

- `AnalysisState`;
- its existing methods, with explicit visibility after the module move:
  - `new` stays `pub`;
  - `insert_active_report_run`, `remove_active_report_run`,
    `active_report_run_ids`, and `report_run_child_token` stay `pub(crate)`;
  - `request_report_run_cancel` and `is_report_run_cancelled` must become
    `pub(super)` because `report.rs` and debug-only `fixtures.rs` are sibling
    modules after the extraction;
  - `ensure_report_run_token` stays private inside `state.rs`;
- imports for `HashMap`, `HashSet`, `Mutex`, and `CancellationToken`.

`events.rs` will contain:

- `emit_analysis_event` as `pub(super)`;
- `emit_analysis_chat_event` as `pub(super)`;
- imports for `AppHandle`, `Emitter`, and
  `super::models::{AnalysisRunEvent, AnalysisChatEvent}`.
- imports for `super::{ANALYSIS_RUN_EVENT, ANALYSIS_CHAT_EVENT}` because the
  constants stay in `mod.rs`.

`mod.rs` will keep:

- module declarations;
- constants such as `ANALYSIS_RUN_EVENT`, `ANALYSIS_CHAT_EVENT`, statuses, template kinds, and scope types;
- `pub use self::state::AnalysisState`;
- Tauri command functions;
- chat validation helpers and default template body for now.

This keeps the first slice small and avoids mixing this change with decisions about typed statuses or deeper command/service extraction.

## Data Flow

No runtime data flow changes:

1. Tauri state still exposes `AnalysisState`.
2. Report startup still inserts active runs through `AnalysisState`.
3. Cancellation still uses the existing cancellation token map.
4. Report and chat code still emit the same event names and payloads.
5. Internal command wrappers and external modules keep their current
   `AnalysisState` paths:
   `analysis/report_commands.rs` through `super::AnalysisState`,
   `analysis/mod.rs` command handlers through the local re-export,
   `accounts.rs`, `account_deletion.rs`, `projects/mod.rs` through
   `start_project_analysis`, and `lib.rs` through `.manage(AnalysisState::new())`.

The only behavior-preserving changes are moving Rust definitions and making the
minimal visibility adjustments required by that move.

## Error Handling

No error behavior changes are expected. State methods keep their current behavior, including silent no-op removal and boolean cancellation request results. Event emission continues to ignore emission errors as it does today.

## Testing

Run focused Rust validation for the affected module boundaries. The commands
below are written for the repository root; equivalently, run them from
`src-tauri/` without `--manifest-path src-tauri/Cargo.toml`.

- `cargo test --manifest-path src-tauri/Cargo.toml analysis::tests::`
- `cargo test --manifest-path src-tauri/Cargo.toml analysis::state::tests::`
- `cargo test --manifest-path src-tauri/Cargo.toml analysis_state_cancels_report_run_child_tokens`
- `cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::`
- `cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::`
- `cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::`
- `cargo test --manifest-path src-tauri/Cargo.toml account_deletion::tests::`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`

`analysis::state::tests::` is included because the existing
`analysis_state_cancels_report_run_child_tokens` unit test should move with
`AnalysisState` from `mod.rs` to `state.rs`.

The named `analysis_state_cancels_report_run_child_tokens` command must be
checked for real execution. The acceptable output must include the test under
the `analysis::state::tests::` path and show it passing; a successful Cargo run
with `0 tests` for this filter is not acceptable.

`analysis::fixtures::tests::` is included because `fixtures.rs` directly exercises
the `AnalysisState` cancellation API through debug-only fixture commands and active
fixture-run behavior.

`account_deletion::tests::` is included because account deletion uses
`AnalysisState::active_report_run_ids` and `insert_active_report_run` to block
deletion while source-owned analysis work is active. `cargo check --all-targets`
covers command-boundary and external imports such as `super::AnalysisState` in
`analysis/report_commands.rs`, local command handler imports in `analysis/mod.rs`,
`crate::analysis::AnalysisState` in `accounts.rs`, `account_deletion.rs`,
`projects/mod.rs`, and `lib.rs`, plus test-only import drift, but not this
behavior.

`projects/mod.rs` does not need an additional behavior test in this slice:
`start_project_analysis` only accepts `tauri::State<'_, crate::analysis::AnalysisState>`
and forwards `state.inner()` to `start_analysis_report_run`. Compile coverage from
`cargo check --all-targets` is enough for that command path.

`cargo fmt --check` is included because move-only Rust refactors can compile and
pass tests while leaving formatting dirty. The implementation can use `cargo fmt`
to fix formatting, but that command may rewrite unrelated Rust files across the
crate. After running `cargo fmt`, inspect `git diff --name-only`; either stage
only files intended for this refactor or make a separate format-only commit for
unrelated rustfmt drift.

If the shell or sandbox blocks `cargo`, rerun the same command with the required approval path rather than claiming success.

## Non-Goals

- Do not split `corpus.rs`, `store.rs`, or `report.rs` in this slice.
- Do not introduce typed status/scope enums.
- Do not change `docs/value-registry.md`; no string values are added or changed.
- Do not alter Tauri command names, arguments, return shapes, or frontend bindings.
- Do not change SQL or migrations.

## Implementation Notes

The implementation should be mostly mechanical:

1. Move `AnalysisState` and its impl from `mod.rs` to `state.rs`.
2. Move the `analysis_state_cancels_report_run_child_tokens` unit test from
   `mod.rs` to `state.rs` with `AnalysisState`.
3. Move event helper functions from `mod.rs` to `events.rs`.
4. Declare the new modules in `mod.rs`.
5. Re-export `AnalysisState` from `analysis` so outside imports remain stable.
6. Update `report.rs` and `chat.rs` to import event helpers directly from
   `super::events::{...}`. Do not re-export event helpers from `analysis/mod.rs`;
   their intended visibility is limited to sibling analysis modules.
7. Remove imports from `mod.rs` that moved to `state.rs` or `events.rs`, including
   `HashMap`, `HashSet`, `Mutex`, `CancellationToken`, `Emitter`,
   `AnalysisRunEvent`, and `AnalysisChatEvent` when they are no longer used by
   the module root.
8. Run `cargo fmt --manifest-path src-tauri/Cargo.toml` from the repository root
   if Rust formatting changes. Equivalently, run `cargo fmt` from `src-tauri/`.
   Because rustfmt operates on the crate, inspect `git diff --name-only` after
   formatting. Stage only intended state/events files for this refactor, or make
   a separate format-only commit for unrelated Rust files.

The resulting `mod.rs` should read more like a module facade plus command host, not a dumping ground for shared runtime helpers.
