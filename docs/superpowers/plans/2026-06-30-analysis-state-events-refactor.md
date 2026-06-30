# Analysis State and Events Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract `AnalysisState` and analysis event emitters out of `src-tauri/src/analysis/mod.rs` without changing runtime behavior or public Tauri contracts.

**Architecture:** Add two focused sibling modules, `analysis/state.rs` and `analysis/events.rs`. Keep `analysis/mod.rs` as the facade for constants, command exports, and top-level Tauri command handlers. Preserve external `crate::analysis::AnalysisState` imports through a root re-export, while keeping event helpers private to sibling analysis modules.

**Tech Stack:** Rust, Tauri, Tokio, `tokio_util::sync::CancellationToken`, Cargo tests/checks under `src-tauri/Cargo.toml`.

## Global Constraints

- Internal Rust refactor of `src-tauri/src/analysis/` only.
- No database migrations.
- No Tauri command name, argument, return-shape, or frontend binding changes.
- No event payload or event-name changes.
- No frontend changes.
- Do not introduce typed status/scope enums.
- Do not change `docs/value-registry.md`; no string values are added or changed.
- Keep `ANALYSIS_RUN_EVENT` and `ANALYSIS_CHAT_EVENT` in `analysis/mod.rs`.
- Keep `AnalysisState` externally importable as `crate::analysis::AnalysisState`.
- Do not re-export event helpers from `analysis/mod.rs`; import them in sibling modules through `super::events::{...}`.
- Run Rust commands from the repository root with `--manifest-path src-tauri/Cargo.toml`, or from `src-tauri/` without `--manifest-path`.

---

## File Structure

- Create `src-tauri/src/analysis/state.rs`: owns `AnalysisState`, cancellation token state, active run IDs, and the moved `analysis_state_cancels_report_run_child_tokens` unit test.
- Create `src-tauri/src/analysis/events.rs`: owns `emit_analysis_event` and `emit_analysis_chat_event`.
- Modify `src-tauri/src/analysis/mod.rs`: declare `mod state;` and `mod events;`, re-export `AnalysisState`, remove moved definitions/imports, keep constants and Tauri commands.
- Modify `src-tauri/src/analysis/report.rs`: import `emit_analysis_event` from `super::events`.
- Modify `src-tauri/src/analysis/chat.rs`: import `emit_analysis_chat_event` from `super::events`.
- Do not modify `src-tauri/src/account_deletion.rs`, `src-tauri/src/accounts.rs`, `src-tauri/src/projects/mod.rs`, `src-tauri/src/analysis/report_commands.rs`, or `src-tauri/src/lib.rs`; validation commands cover those consumers.

---

### Task 1: Extract AnalysisState

**Files:**
- Create: `src-tauri/src/analysis/state.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Verify: `src-tauri/src/analysis/fixtures.rs`
- Verify: `src-tauri/src/account_deletion.rs`

**Interfaces:**
- Produces: `pub struct AnalysisState`.
- Produces: `impl AnalysisState` with:
  - `pub fn new() -> Self`
  - `pub(crate) async fn insert_active_report_run(&self, run_id: i64)`
  - `pub(crate) async fn remove_active_report_run(&self, run_id: i64)`
  - `pub(crate) async fn active_report_run_ids(&self) -> HashSet<i64>`
  - `pub(super) async fn request_report_run_cancel(&self, run_id: i64) -> bool`
  - `pub(super) async fn is_report_run_cancelled(&self, run_id: i64) -> bool`
  - `pub(crate) async fn report_run_child_token(&self, run_id: i64) -> Option<CancellationToken>`
  - `async fn ensure_report_run_token(&self, run_id: i64) -> CancellationToken`
- Consumed by: `analysis/mod.rs`, `analysis/report.rs`, `analysis/report_commands.rs`, debug-only `analysis/fixtures.rs`, `account_deletion.rs`, `accounts.rs`, `projects/mod.rs`, `lib.rs`.

- [x] **Step 1: Move the state implementation and state test**

Create `src-tauri/src/analysis/state.rs` with this content:

```rust
use std::collections::{HashMap, HashSet};

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct AnalysisState {
    active_report_runs: Mutex<HashSet<i64>>,
    report_run_tokens: Mutex<HashMap<i64, CancellationToken>>,
}

impl AnalysisState {
    pub fn new() -> Self {
        Self {
            active_report_runs: Mutex::new(HashSet::new()),
            report_run_tokens: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) async fn insert_active_report_run(&self, run_id: i64) {
        self.active_report_runs.lock().await.insert(run_id);
        self.report_run_tokens
            .lock()
            .await
            .insert(run_id, CancellationToken::new());
    }

    pub(crate) async fn remove_active_report_run(&self, run_id: i64) {
        self.active_report_runs.lock().await.remove(&run_id);
        self.report_run_tokens.lock().await.remove(&run_id);
    }

    pub(crate) async fn active_report_run_ids(&self) -> HashSet<i64> {
        self.active_report_runs.lock().await.clone()
    }

    pub(super) async fn request_report_run_cancel(&self, run_id: i64) -> bool {
        let active_runs = self.active_report_runs.lock().await;
        if !active_runs.contains(&run_id) {
            return false;
        }
        drop(active_runs);
        self.ensure_report_run_token(run_id).await.cancel();
        true
    }

    pub(super) async fn is_report_run_cancelled(&self, run_id: i64) -> bool {
        self.report_run_tokens
            .lock()
            .await
            .get(&run_id)
            .is_some_and(CancellationToken::is_cancelled)
    }

    pub(crate) async fn report_run_child_token(&self, run_id: i64) -> Option<CancellationToken> {
        self.report_run_tokens
            .lock()
            .await
            .get(&run_id)
            .map(CancellationToken::child_token)
    }

    async fn ensure_report_run_token(&self, run_id: i64) -> CancellationToken {
        self.report_run_tokens
            .lock()
            .await
            .entry(run_id)
            .or_insert_with(CancellationToken::new)
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::AnalysisState;

    #[tokio::test]
    async fn analysis_state_cancels_report_run_child_tokens() {
        let state = AnalysisState::new();

        state.insert_active_report_run(42).await;
        let child = state.report_run_child_token(42).await.expect("child token");
        assert!(!child.is_cancelled());

        assert!(state.request_report_run_cancel(42).await);
        tokio::time::timeout(std::time::Duration::from_secs(1), child.cancelled())
            .await
            .expect("child token cancelled");
        assert!(state.is_report_run_cancelled(42).await);

        state.remove_active_report_run(42).await;
        assert!(state.report_run_child_token(42).await.is_none());
        assert!(!state.is_report_run_cancelled(42).await);
    }
}
```

In `src-tauri/src/analysis/mod.rs`, add the module declaration near the other module declarations:

```rust
mod state;
```

Add the root re-export near the existing `pub use` block:

```rust
pub use self::state::AnalysisState;
```

Remove the old `AnalysisState` struct and impl from `mod.rs`.

Remove this state test from `mod.rs` because it now lives in `state.rs`:

```rust
#[tokio::test]
async fn analysis_state_cancels_report_run_child_tokens() {
    let state = AnalysisState::new();

    state.insert_active_report_run(42).await;
    let child = state.report_run_child_token(42).await.expect("child token");
    assert!(!child.is_cancelled());

    assert!(state.request_report_run_cancel(42).await);
    tokio::time::timeout(std::time::Duration::from_secs(1), child.cancelled())
        .await
        .expect("child token cancelled");
    assert!(state.is_report_run_cancelled(42).await);

    state.remove_active_report_run(42).await;
    assert!(state.report_run_child_token(42).await.is_none());
    assert!(!state.is_report_run_cancelled(42).await);
}
```

Update the top-level imports in `mod.rs` so the state-only imports are gone. At this point `mod.rs` still contains event helpers, so keep `Emitter` and the event DTO imports for now:

```rust
use tauri::{AppHandle, Emitter};

use self::models::{
    AnalysisChatEvent, AnalysisChatTurn, AnalysisRunDetail, AnalysisRunEvent,
    AnalysisRunMessageCursor, AnalysisRunMessagesPage, AnalysisRunSummary, AnalysisSourceOption,
    AnalysisTraceData, AnalysisTraceRef,
};
```

Remove `AnalysisState` from the `analysis/mod.rs` test import block because the only test using it moved to `state.rs`. The grouped import inside `mod tests` should be:

```rust
use super::{
    decode_trace_data, validate_chat_role, AnalysisChatTurn, AnalysisTraceData,
    AnalysisTraceRef, TEMPLATE_KIND_REPORT,
};
```

- [x] **Step 2: Format the Rust files**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: command exits successfully with no required stdout.

- [x] **Step 3: Verify the moved state test path exists and passes**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::state::tests::
```

Expected: PASS and output includes a non-zero test count with this test path:

```text
test analysis::state::tests::analysis_state_cancels_report_run_child_tokens ... ok
```

If Cargo reports `0 tests`, stop and fix the test module path before continuing.

- [x] **Step 4: Verify the named state test is not accidentally filtered out**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis_state_cancels_report_run_child_tokens
```

Expected: PASS and output includes:

```text
test analysis::state::tests::analysis_state_cancels_report_run_child_tokens ... ok
```

If Cargo reports `0 tests`, stop and fix the test location before continuing.

- [x] **Step 5: Verify debug fixture consumers of cancellation API**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: PASS. This confirms debug-only `fixtures.rs` can still call `request_report_run_cancel` and active fixture run APIs.

- [x] **Step 6: Verify non-analysis account deletion behavior**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml account_deletion::tests::
```

Expected: PASS. This confirms account deletion still observes active analysis runs through `AnalysisState`.

- [x] **Step 7: Commit Task 1**

Run:

```powershell
git status --short
git add src-tauri/src/analysis/mod.rs src-tauri/src/analysis/state.rs
git commit -m "refactor: extract analysis state"
```

Expected: one commit containing only the state extraction and moved state test.

---

### Task 2: Extract Analysis Event Helpers

**Files:**
- Create: `src-tauri/src/analysis/events.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/report.rs`
- Modify: `src-tauri/src/analysis/chat.rs`

**Interfaces:**
- Produces: `pub(super) fn emit_analysis_event(handle: &AppHandle, event: &AnalysisRunEvent)`.
- Produces: `pub(super) fn emit_analysis_chat_event(handle: &AppHandle, event: &AnalysisChatEvent)`.
- Consumes: `super::{ANALYSIS_RUN_EVENT, ANALYSIS_CHAT_EVENT}` from `analysis/mod.rs`.
- Consumed by: `analysis/report.rs` and `analysis/chat.rs` through `super::events::{...}`.

- [x] **Step 1: Create the events module**

Create `src-tauri/src/analysis/events.rs` with this content:

```rust
use tauri::{AppHandle, Emitter};

use super::models::{AnalysisChatEvent, AnalysisRunEvent};
use super::{ANALYSIS_CHAT_EVENT, ANALYSIS_RUN_EVENT};

pub(super) fn emit_analysis_event(handle: &AppHandle, event: &AnalysisRunEvent) {
    let _ = handle.emit(ANALYSIS_RUN_EVENT, event);
}

pub(super) fn emit_analysis_chat_event(handle: &AppHandle, event: &AnalysisChatEvent) {
    let _ = handle.emit(ANALYSIS_CHAT_EVENT, event);
}
```

In `src-tauri/src/analysis/mod.rs`, add the module declaration near the other private modules:

```rust
mod events;
```

Remove the old helper functions from `mod.rs`:

```rust
fn emit_analysis_event(handle: &AppHandle, event: &AnalysisRunEvent) {
    let _ = handle.emit(ANALYSIS_RUN_EVENT, event);
}

fn emit_analysis_chat_event(handle: &AppHandle, event: &AnalysisChatEvent) {
    let _ = handle.emit(ANALYSIS_CHAT_EVENT, event);
}
```

- [x] **Step 2: Update report and chat imports**

In `src-tauri/src/analysis/report.rs`, add this import:

```rust
use super::events::emit_analysis_event;
```

Then remove `emit_analysis_event` from the grouped `use super::{ ... }` import. The grouped import should start like this:

```rust
use super::{
    now_secs, AnalysisState, ANALYSIS_FALLBACK_CHUNK_TARGET_CHARS,
    ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_COMPLETED,
    ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING, TEMPLATE_KIND_REPORT,
};
```

In `src-tauri/src/analysis/chat.rs`, add this import:

```rust
use super::events::emit_analysis_chat_event;
```

Then remove `emit_analysis_chat_event` from the grouped `use super::{ ... }` import. The grouped import should be:

```rust
use super::{now_secs, validate_chat_role, validate_chat_turns, ANALYSIS_STATUS_COMPLETED};
```

- [x] **Step 3: Clean moved imports from mod.rs**

In `src-tauri/src/analysis/mod.rs`, remove `Emitter` from the Tauri import:

```rust
use tauri::AppHandle;
```

Remove event DTOs from the `self::models` import. The import should become:

```rust
use self::models::{
    AnalysisChatTurn, AnalysisRunDetail, AnalysisRunMessageCursor, AnalysisRunMessagesPage,
    AnalysisRunSummary, AnalysisSourceOption, AnalysisTraceData, AnalysisTraceRef,
};
```

Confirm these imports are also absent from `mod.rs` after Task 1 and Task 2:

```rust
use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
```

- [x] **Step 4: Format the Rust files**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: command exits successfully with no required stdout.

- [x] **Step 5: Verify analysis module tests still pass**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::tests::
```

Expected: PASS. This confirms the remaining `analysis/mod.rs` tests still compile without the moved state test and moved event helpers.

- [x] **Step 6: Verify chat event helper consumer**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::
```

Expected: PASS. This confirms `chat.rs` can access `super::events::emit_analysis_chat_event`.

- [x] **Step 7: Verify report event helper consumer**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS. This confirms `report.rs` can access `super::events::emit_analysis_event`.

- [x] **Step 8: Verify all Rust targets and external state consumers compile**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. This covers `accounts.rs`, `account_deletion.rs`, `projects/mod.rs`, `analysis/report_commands.rs`, `lib.rs`, external `crate::analysis::AnalysisState` imports, and test-only import drift.

- [x] **Step 9: Commit Task 2**

Run:

```powershell
git status --short
git add src-tauri/src/analysis/mod.rs src-tauri/src/analysis/events.rs src-tauri/src/analysis/report.rs src-tauri/src/analysis/chat.rs
git commit -m "refactor: extract analysis events"
```

Expected: one commit containing only the event helper extraction and import cleanup.

---

## Final Verification

- [x] **Step 1: Run the complete focused validation set from the approved spec**

Run each command separately and stop on the first failure:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::tests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::state::tests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis_state_cancels_report_run_child_tokens
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml account_deletion::tests::
```

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: every command passes. The `analysis::state::tests::` command and the named state test command must both include the moved state test and must not be green `0 tests` runs. Both should include:

```text
test analysis::state::tests::analysis_state_cancels_report_run_child_tokens ... ok
```

- [x] **Step 2: Confirm no unintended files are staged**

Run:

```powershell
git status --short
```

Expected: clean worktree, or only intentional files if verification artifacts are produced. Do not stage `.playwright-mcp/` if it appears.

## Self-Review Notes

- Spec coverage: Task 1 covers `state.rs`, visibility adjustments, moved state test, fixture/account deletion behavior, and stable `AnalysisState` re-export. Task 2 covers `events.rs`, direct sibling imports, event constants imports, moved import cleanup, and all-target compile coverage.
- Marker scan: this plan contains no unresolved markers.
- Type consistency: method signatures and event helper signatures match the approved design.
