# Gemini Browser Warning Cleanup Design

**Date:** 2026-07-12
**Status:** Approved for specification review

## Goal

Remove all production `cargo check` warnings originating from
`src-tauri/src/gemini_browser/` without changing Gemini Browser runtime,
queue, reconciliation, JSONL transport, cached status, or serialized API
behavior.

## Scope

This warning-cleanup slice owns only:

- `gemini_browser/mod.rs`;
- `gemini_browser/jobs.rs`;
- `gemini_browser/sidecar.rs`;
- `gemini_browser/state.rs`;
- `gemini_browser/types.rs`.

It does not split the 2,900-line `jobs.rs`, change Apalis behavior, alter Tauri
commands, or address the remaining warnings in `apalis_jobs.rs` and
`youtube/jobs.rs`.

## Selected Architecture

Express actual production/test boundaries through narrow `#[cfg(test)]`
attributes rather than suppressing dead-code diagnostics.

### Type Reexports

`GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary` remain public
Rust types because they are fields in serialized run/result structures. Remove
them from the normal `gemini_browser::` reexport list and reexport them within
the crate only under `#[cfg(test)]`, where existing run-log and prompt-pack
tests use the shorter path. Serialization and TypeScript wire shapes do not
depend on the Rust module reexport.

### Job Helpers

- Compile `ApalisQueueInspectionMode::Supported` only in tests. Production has
  one honest mode, `DegradedRunLogOnly`.
- Keep `startup_reconciliation_checks_queued_runs_against_apalis` exhaustive
  on both builds: production returns `false` for the degraded mode; the test
  build additionally returns `true` for `Supported`.
- Compile `run_status_for_queue_state` and `run_log_is_cancelled` only in tests.
  Their call sites are inside the in-file test module.
- Keep `GeminiBrowserJobRuntime::new_with_timeouts` production-visible and make
  `Default::default()` delegate to it with the existing default durations.
  This removes duplicated construction without changing timeout values.

### Sidecar, State, and Status Helpers

Compile these existing test-only helpers only in test builds:

- `decode_sidecar_line`;
- `take_complete_jsonl_line`;
- `GeminiBrowserState::set_status_snapshot`;
- `GeminiBrowserRunStatus::is_success`.

Production JSONL request/response handling continues through the active
transport path; cached status initialization and reads remain unchanged;
terminal-status logic remains production-visible.

## Alternatives Rejected

- Do not add broad or item-level `allow(dead_code)` attributes. They would hide
  future drift instead of describing compilation ownership.
- Do not delete the helpers and rebuild tests through heavier production
  fixtures. That would expand a warning cleanup into a test-architecture
  change.
- Do not split `jobs.rs` in this slice. File decomposition requires its own
  behavioral map and review after the warning baseline is clean.

## Behavioral Contracts

- Production queue inspection remains degraded/run-log-only.
- Startup reconciliation does not begin querying unsupported Apalis queue
  state.
- Test coverage for the hypothetical supported queue mode remains available.
- Runtime timeouts, cancellation, worker guards, and run-log interpretation do
  not change.
- Sidecar framing and response validation tests remain unchanged and passing.
- Cached provider status tests retain their direct state setup path.
- Serialized Rust and TypeScript values remain unchanged; no enum variant or
  JSON field is removed.

## Verification

- Run focused tests for `gemini_browser`; all existing tests must pass.
- Run `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`; it must
  exit zero and emit no warning from `src/gemini_browser/` in either normal or
  test targets.
- The warning baseline must decrease from 11 warnings to 2, excluding Cargo's
  final summary line. The two remaining locations are `apalis_jobs.rs` and
  `youtube/jobs.rs`.
- Run the full Rust suite before completion.
- Run `git diff --check` and confirm only the five scoped Gemini Browser files
  plus the spec/plan are committed.

## Documentation and Registry Impact

No runtime behavior, API value, persisted value, UI value, fixture value, or
user workflow changes. `docs/project.md` and `docs/value-registry.md` do not
require edits.

## Follow-up

After the two remaining non-Gemini warnings are handled, design a separate
module-decomposition slice for `gemini_browser/jobs.rs`. That design should
start from the clean `cargo check` baseline and split by responsibility rather
than by arbitrary line count.
