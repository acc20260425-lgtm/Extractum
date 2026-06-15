# YouTube Summary Types And Renames Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the first pass of YouTube Summary folder cleanup by extracting shared contract types and applying two local module renames.

**Architecture:** Keep `youtube_summary/mod.rs` as the domain facade and preserve external imports through `prompt_packs::youtube_summary::{...}`. Move shared request/result/error structs into `types.rs`, then rename `run_store.rs` to `store.rs` and `stage_outputs.rs` to `outputs.rs` inside the folder. No runtime behavior, schema, or provider logic changes.

**Tech Stack:** Rust modules, Tauri backend, existing Prompt Pack tests.

---

## Task 1: Extract Shared Types

**Files:**
- Create: `src-tauri/src/prompt_packs/youtube_summary/types.rs`
- Modify: `src-tauri/src/prompt_packs/youtube_summary/mod.rs`

- [x] **Step 1: Move contract types into `types.rs`**

Move these items from `mod.rs` into `types.rs`:

- `ModelBudget`
- `LlmCompletion`
- `TranscriptAnalysisStageExecutionRequest`
- `SynthesisStageExecutionRequest`
- `YoutubeSummaryStageExecutionRequest`
- `YoutubeSummaryRunExecutionOutcome`
- `YoutubeSummaryStageExecutionError`
- `SYNTHESIS_STAGE_NAME`

- [x] **Step 2: Re-export types from `mod.rs`**

Add:

```rust
mod types;
pub use types::ModelBudget;
pub(crate) use types::{
    LlmCompletion, SynthesisStageExecutionRequest, TranscriptAnalysisStageExecutionRequest,
    YoutubeSummaryRunExecutionOutcome, YoutubeSummaryStageExecutionError,
    YoutubeSummaryStageExecutionRequest, SYNTHESIS_STAGE_NAME,
};
```

- [x] **Step 3: Verify focused compile**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
```

Expected: more than zero tests run and pass.

## Task 2: Rename Store And Outputs Modules

**Files:**
- Move: `src-tauri/src/prompt_packs/youtube_summary/run_store.rs` -> `src-tauri/src/prompt_packs/youtube_summary/store.rs`
- Move: `src-tauri/src/prompt_packs/youtube_summary/stage_outputs.rs` -> `src-tauri/src/prompt_packs/youtube_summary/outputs.rs`
- Move: `src-tauri/src/prompt_packs/youtube_summary/stage_outputs_tests.rs` -> `src-tauri/src/prompt_packs/youtube_summary/outputs_tests.rs`
- Modify: imports in `youtube_summary/mod.rs`, `execution.rs`, `snapshots.rs`, and tests.

- [x] **Step 1: Rename files**

Use file moves only; do not change behavior.

- [x] **Step 2: Update module declarations and imports**

Replace:

- `run_store` with `store`
- `stage_outputs` with `outputs`
- `stage_outputs_tests` with `outputs_tests`

- [x] **Step 3: Verify focused compile**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml --lib youtube_summary_
```

Expected: more than zero tests run and pass.

## Task 3: Final Verification And Commit

- [x] **Step 1: Run full verification**

Run:

```powershell
cargo fmt --manifest-path src-tauri\Cargo.toml --check
cargo test --manifest-path src-tauri\Cargo.toml --lib prompt_packs
cargo check --manifest-path src-tauri\Cargo.toml
git diff --check
```

Expected: all commands exit 0. Existing unchanged dead-code warnings are acceptable.

- [ ] **Step 2: Commit**

Run:

```powershell
git add src-tauri/src/prompt_packs/youtube_summary docs/superpowers/plans/2026-06-15-youtube-summary-types-and-renames.md
git commit -m "refactor: tidy youtube summary module names"
```

## Self-Review

- Spec coverage: extracts the shared type block and applies the two approved local module renames.
- Placeholder scan: no placeholders remain.
- Type consistency: no type names or function names change; only file/module paths change.
