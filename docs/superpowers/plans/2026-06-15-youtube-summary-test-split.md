# YouTube Summary Test Split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the large `youtube_summary.rs` test module into focused test modules while keeping `youtube_summary.rs` as a small facade.

**Architecture:** Keep all production APIs unchanged. Create sibling `#[cfg(test)]` modules under `src-tauri/src/prompt_packs/` grouped by domain, each importing shared helpers from `youtube_summary_test_support`. Register the test modules from `prompt_packs/mod.rs` so private crate-visible APIs remain testable without embedding a large `mod tests` inside the facade.

**Tech Stack:** Rust, Tokio tests, SQLx SQLite test pools, existing Prompt Pack test support.

---

## File Structure

- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`
  - Remove the embedded `#[cfg(test)] mod tests`.
  - Keep facade types, constants, helper functions, and `start_youtube_summary_run_in_pool`.
- Modify: `src-tauri/src/prompt_packs/mod.rs`
  - Add `#[cfg(test)]` module declarations for the new test files.
- Create: `src-tauri/src/prompt_packs/youtube_summary_facade_tests.rs`
  - Tests small facade helpers.
- Create: `src-tauri/src/prompt_packs/youtube_summary_synthesis_input_tests.rs`
  - Tests synthesis input assembly.
- Create: `src-tauri/src/prompt_packs/youtube_summary_stage_outputs_tests.rs`
  - Tests transcript/synthesis artifact persistence and invalid-output behavior.
- Create: `src-tauri/src/prompt_packs/youtube_summary_preflight_tests.rs`
  - Tests preflight blocking/skipping behavior.
- Create: `src-tauri/src/prompt_packs/youtube_summary_snapshots_tests.rs`
  - Tests run skeleton, source/material snapshots, idempotency, and deterministic comment snapshots.
- Create: `src-tauri/src/prompt_packs/youtube_summary_execution_tests.rs`
  - Tests queued run execution, repair, partial results, and synthesis lifecycle.

## Task 1: Create Test Module Shells

**Files:**
- Modify: `src-tauri/src/prompt_packs/mod.rs`
- Create the six new `youtube_summary_*_tests.rs` files.

- [x] **Step 1: Add module declarations**

Add these declarations after `youtube_summary_test_support`:

```rust
#[cfg(test)]
mod youtube_summary_facade_tests;
#[cfg(test)]
mod youtube_summary_synthesis_input_tests;
#[cfg(test)]
mod youtube_summary_stage_outputs_tests;
#[cfg(test)]
mod youtube_summary_preflight_tests;
#[cfg(test)]
mod youtube_summary_snapshots_tests;
#[cfg(test)]
mod youtube_summary_execution_tests;
```

- [x] **Step 2: Create empty test files**

Each file starts with imports only when tests are moved into it. Do not add placeholder tests.

- [x] **Step 3: Run a baseline compile**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib now_string_uses_current_utc_time
```

Expected: PASS before moving anything.

## Task 2: Move Facade And Synthesis Input Tests

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary_facade_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary_synthesis_input_tests.rs`

- [x] **Step 1: Move facade helper test**

Move `now_string_uses_current_utc_time` into `youtube_summary_facade_tests.rs`.

- [x] **Step 2: Move synthesis input tests**

Move:

- `build_synthesis_stage_input_collects_successful_transcript_outputs`
- `build_synthesis_stage_input_uses_latest_parsed_output_wrappers`

into `youtube_summary_synthesis_input_tests.rs`.

- [x] **Step 3: Run focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib now_string_uses_current_utc_time
cargo test --manifest-path src-tauri\Cargo.toml --lib build_synthesis_stage_input
```

Expected: each filtered command runs at least one named test and passes.

## Task 3: Move Stage Output Tests

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary_stage_outputs_tests.rs`

- [x] **Step 1: Move stage output tests**

Move:

- `execute_synthesis_stage_persists_raw_parsed_and_metrics_artifacts`
- `execute_synthesis_stage_rejects_invalid_output_without_success_artifacts`
- `execute_transcript_analysis_stage_persists_raw_and_parsed_artifacts`

into `youtube_summary_stage_outputs_tests.rs`.

- [x] **Step 2: Run focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib execute_synthesis_stage
cargo test --manifest-path src-tauri\Cargo.toml --lib execute_transcript_analysis_stage
```

Expected: each filtered command runs at least one named test and passes.

## Task 4: Move Preflight And Snapshot Tests

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary_preflight_tests.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary_snapshots_tests.rs`

- [x] **Step 1: Move preflight tests**

Move:

- `preflight_explicit_video_without_transcript_is_blocking_failure`
- `preflight_playlist_video_without_transcript_is_skipped`

into `youtube_summary_preflight_tests.rs`.

- [x] **Step 2: Move snapshot/start tests**

Move:

- `start_freezes_one_canonical_video_snapshot_with_multiple_origins`
- `start_returns_existing_run_for_duplicate_client_request_id`
- `start_with_recomputed_blocking_preflight_returns_response_without_run`
- `comment_snapshot_selection_is_deterministic_when_enabled`

into `youtube_summary_snapshots_tests.rs`.

- [x] **Step 3: Run focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib preflight_
cargo test --manifest-path src-tauri\Cargo.toml --lib start_
cargo test --manifest-path src-tauri\Cargo.toml --lib comment_snapshot
```

Expected: each filtered command runs at least one named test and passes.

## Task 5: Move Execution Tests And Remove Embedded Module

**Files:**
- Modify: `src-tauri/src/prompt_packs/youtube_summary.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary_execution_tests.rs`

- [x] **Step 1: Move execution lifecycle tests**

Move the remaining queued run and synthesis lifecycle tests into `youtube_summary_execution_tests.rs`.

- [x] **Step 2: Remove empty embedded test module**

Delete the remaining `#[cfg(test)] mod tests` wrapper from `youtube_summary.rs`.

- [x] **Step 3: Run focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib execute_queued_run
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
```

Expected: each filtered command runs at least one named test and passes.

## Task 6: Final Verification And Commit

**Files:**
- All files changed in Tasks 1-5.

- [x] **Step 1: Format and verify Prompt Pack tests**

Run:

```powershell
cargo fmt --manifest-path src-tauri\Cargo.toml --check
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs
cargo check --manifest-path src-tauri\Cargo.toml
git diff --check
```

Expected:

- formatting check passes;
- Prompt Pack filtered tests run more than zero tests and pass;
- `cargo check` exits 0, allowing existing warnings if unchanged;
- `git diff --check` exits 0.

- [x] **Step 2: Commit**

Run:

```powershell
git add src-tauri/src/prompt_packs docs/superpowers/plans/2026-06-15-youtube-summary-test-split.md
git commit -m "refactor: split youtube summary tests"
```

## Self-Review

- Spec coverage: covers all test names currently embedded in `youtube_summary.rs` and keeps production APIs unchanged.
- Placeholder scan: no implementation placeholders remain; each moved group has exact source and destination.
- Type consistency: all tests continue using existing helper names from `youtube_summary_test_support`; no new runtime types are introduced.
