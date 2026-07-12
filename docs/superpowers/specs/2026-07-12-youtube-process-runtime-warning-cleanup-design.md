# YouTube Process Runtime Warning Cleanup Design

**Date:** 2026-07-12
**Status:** Approved for specification review

## Goal

Remove all production `cargo check` warnings originating from
`src-tauri/src/youtube/process_runtime.rs` without changing yt-dlp launch,
cancellation, timeout, process-tree containment, output draining, or cookie
lifetime behavior.

## Scope

This is the first bounded warning-cleanup slice. It owns only
`youtube/process_runtime.rs` and its existing tests. It does not split the test
module into another file, change public Tauri commands, or address warnings in
Gemini Browser, Apalis, or `youtube/jobs.rs`.

## Selected Changes

- Keep `CookieLifetimeGuard` as the final owner of its `NamedTempFile`, but use
  a named `_cookie` field so the ownership-only purpose is explicit and does
  not produce a false dead-field warning.
- Remove the unused `run_ytdlp_managed` wrapper. The application continues to
  call `run_ytdlp_managed_with_cancellation`.
- Compile helpers used only by the in-file tests under `#[cfg(test)]`:
  - `detach_reap_with_cookie`;
  - `detach_cookie_for_test`;
  - `YoutubeProcessRegistry::is_empty`;
  - `run_ytdlp_managed_with_cookie`;
  - `run_ytdlp_managed_with_external_cancellation`;
  - `run_ytdlp_managed_with`;
  - `drain_output_while_waiting`.
- Remove unnecessary `mut` bindings from the stdout and stderr drain task
  handles. `JoinHandle::abort` requires only a shared reference.
- Tests may capture the temporary-file path directly from `NamedTempFile`
  before transferring ownership into `CookieLifetimeGuard`; a guard-level
  `path` method is not required.

## Behavioral Contracts

- The temporary cookie file remains alive while a managed child or detached
  reaper owns its guard and is removed only after that ownership ends.
- Admission rejection, spawn failure, cancellation, timeout, nonzero exit, and
  reap failure retain their existing classifications and cleanup behavior.
- Stdout and stderr continue draining concurrently with process exit so pipe
  backpressure cannot deadlock yt-dlp.
- The managed registry continues to own live operations until reaping finishes.
- No warning is hidden through a broad `allow` attribute; production/test
  compilation boundaries express actual ownership.

## Verification

- Run the focused `youtube::process_runtime` Rust tests; all 14 existing tests
  must pass.
- Run `cargo check --manifest-path src-tauri/Cargo.toml`; it must exit zero and
  report no warning whose path is `youtube/process_runtime.rs`.
- Confirm the repository warning baseline decreases from 23 warnings to 11
  warnings, excluding Cargo's final summary line from both counts.
- Run the full Rust suite before completion.
- Inspect `git diff --check` and stage only the spec/plan and
  `youtube/process_runtime.rs` files owned by this slice.

## Documentation and Registry Impact

No runtime behavior, API shape, persisted value, UI value, fixture value, or
user workflow changes. `docs/project.md` and `docs/value-registry.md` do not
require edits.

## Follow-up Slices

After this slice is complete, test extraction from `process_runtime.rs` may be
considered separately. Remaining warning groups should then be handled in
small independent slices, beginning with Gemini Browser rather than combining
all warnings and large-module refactors into one change.
