# YouTube Summary Folder Move Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move all YouTube Summary prompt-pack modules into `src-tauri/src/prompt_packs/youtube_summary/` while preserving the existing external module API.

**Architecture:** This is a mechanical module-layout refactor. `prompt_packs/mod.rs` will keep only `pub mod youtube_summary;`, and `youtube_summary/mod.rs` will declare the domain submodules, keep the existing facade code, and re-export the crate-visible contracts used by sibling prompt-pack modules. No runtime behavior, database schema, or provider logic changes.

**Tech Stack:** Rust modules, Tauri backend, existing Prompt Pack tests.

---

## File Structure

Move these files:

- `src-tauri/src/prompt_packs/youtube_summary.rs` -> `src-tauri/src/prompt_packs/youtube_summary/mod.rs`
- `src-tauri/src/prompt_packs/youtube_summary_execution.rs` -> `src-tauri/src/prompt_packs/youtube_summary/execution.rs`
- `src-tauri/src/prompt_packs/youtube_summary_preflight.rs` -> `src-tauri/src/prompt_packs/youtube_summary/preflight.rs`
- `src-tauri/src/prompt_packs/youtube_summary_run_store.rs` -> `src-tauri/src/prompt_packs/youtube_summary/run_store.rs`
- `src-tauri/src/prompt_packs/youtube_summary_snapshots.rs` -> `src-tauri/src/prompt_packs/youtube_summary/snapshots.rs`
- `src-tauri/src/prompt_packs/youtube_summary_sources.rs` -> `src-tauri/src/prompt_packs/youtube_summary/sources.rs`
- `src-tauri/src/prompt_packs/youtube_summary_stage_outputs.rs` -> `src-tauri/src/prompt_packs/youtube_summary/stage_outputs.rs`
- `src-tauri/src/prompt_packs/youtube_summary_synthesis_input.rs` -> `src-tauri/src/prompt_packs/youtube_summary/synthesis_input.rs`
- `src-tauri/src/prompt_packs/youtube_summary_test_support.rs` -> `src-tauri/src/prompt_packs/youtube_summary/test_support.rs`
- `src-tauri/src/prompt_packs/youtube_summary_execution_tests.rs` -> `src-tauri/src/prompt_packs/youtube_summary/execution_tests.rs`
- `src-tauri/src/prompt_packs/youtube_summary_facade_tests.rs` -> `src-tauri/src/prompt_packs/youtube_summary/facade_tests.rs`
- `src-tauri/src/prompt_packs/youtube_summary_preflight_tests.rs` -> `src-tauri/src/prompt_packs/youtube_summary/preflight_tests.rs`
- `src-tauri/src/prompt_packs/youtube_summary_snapshots_tests.rs` -> `src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs`
- `src-tauri/src/prompt_packs/youtube_summary_stage_outputs_tests.rs` -> `src-tauri/src/prompt_packs/youtube_summary/stage_outputs_tests.rs`
- `src-tauri/src/prompt_packs/youtube_summary_synthesis_input_tests.rs` -> `src-tauri/src/prompt_packs/youtube_summary/synthesis_input_tests.rs`

## Task 1: Move Files And Register Nested Modules

- [x] **Step 1: Move files into `youtube_summary/`**

Create `src-tauri/src/prompt_packs/youtube_summary/` and move each file listed above to its new path.

- [x] **Step 2: Update `prompt_packs/mod.rs`**

Remove all top-level `youtube_summary_*` module declarations and leave only:

```rust
pub mod youtube_summary;
```

- [x] **Step 3: Update `youtube_summary/mod.rs`**

Add nested module declarations at the top:

```rust
pub(crate) mod execution;
pub(crate) mod preflight;
pub(crate) mod run_store;
pub(crate) mod snapshots;
pub(crate) mod sources;
pub(crate) mod stage_outputs;
pub(crate) mod synthesis_input;
#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod execution_tests;
#[cfg(test)]
mod facade_tests;
#[cfg(test)]
mod preflight_tests;
#[cfg(test)]
mod snapshots_tests;
#[cfg(test)]
mod stage_outputs_tests;
#[cfg(test)]
mod synthesis_input_tests;
```

Keep crate-visible re-exports for contracts currently consumed outside this nested module.

## Task 2: Rewrite Module Paths

- [x] **Step 1: Rewrite imports inside nested modules**

Replace references like `super::youtube_summary::...` with `super::...`, and sibling references like `super::youtube_summary_sources::...` with `super::sources::...`.

- [x] **Step 2: Rewrite imports outside nested modules**

For sibling prompt-pack modules, keep imports through `super::youtube_summary::{...}` where possible. If direct nested-module imports are needed in tests, use `crate::prompt_packs::youtube_summary::...`.

- [x] **Step 3: Search for stale top-level module names**

Run:

```powershell
rg "youtube_summary_(execution|preflight|run_store|snapshots|sources|stage_outputs|synthesis_input|test_support)" src-tauri\src
```

Expected: no Rust module path references remain outside string literals/test names.

## Task 3: Verification And Commit

- [x] **Step 1: Focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
```

Expected: runs more than zero tests and passes.

- [x] **Step 2: Full prompt-pack verification**

Run:

```powershell
cargo fmt --manifest-path src-tauri\Cargo.toml --check
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs
cargo check --manifest-path src-tauri\Cargo.toml
git diff --check
```

Expected: all commands exit 0. Existing dead-code warnings are acceptable if unchanged.

- [x] **Step 3: Commit**

Run:

```powershell
git add src-tauri/src/prompt_packs docs/superpowers/plans/2026-06-15-youtube-summary-folder-move.md
git commit -m "refactor: move youtube summary modules into folder"
```

## Self-Review

- Spec coverage: moves every `youtube_summary*.rs` module into the new folder and preserves `prompt_packs::youtube_summary` as the external entry point.
- Placeholder scan: no placeholders remain.
- Type consistency: no type names change; this is a path-only refactor.
