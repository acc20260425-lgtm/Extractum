# Gemini Browser Warning Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove all production Rust warnings from `gemini_browser/` by expressing existing test-only boundaries without changing runtime behavior.

**Architecture:** First gate the three genuinely test-only job/queue symbols in
`jobs.rs`, make `Default` reuse the production timeout constructor, and verify
the job tests. Then gate test-only reexports and helpers in `mod.rs`,
`sidecar.rs`, `state.rs`, and `types.rs`, reducing the repository warning
baseline from 11 to 2.

**Tech Stack:** Rust 2021, Cargo, Tokio, Apalis, Tauri.

## Global Constraints

- Do not add `allow(dead_code)` or other warning suppressions.
- Do not split `jobs.rs` or change queue/reconciliation behavior.
- Production remains `DegradedRunLogOnly`; `Supported` remains available in tests.
- Do not change serialized enum variants, JSON fields, Tauri commands, TypeScript types, or persisted values.
- Do not edit `docs/project.md` or `docs/value-registry.md`.
- Follow warning-RED → focused GREEN → production warning GREEN for each task.
- Precondition: commit `048221a2` (`chore: clean youtube process runtime warnings`)
  is present and `git status --short --untracked-files=all` is clean before
  Task 1 begins.

---

### Task 1: Clean Gemini Job Test Infrastructure

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs:17-44,83-157,593-599`
- Test in place: `src-tauri/src/gemini_browser/jobs.rs:1106-2974`

**Interfaces:**
- Retains production `ApalisQueueInspectionMode::DegradedRunLogOnly`.
- Retains `startup_reconciliation_checks_queued_runs_against_apalis(mode) -> bool` on all builds.
- Retains `Supported`, queue-status mapping, and run-log cancellation lookup in
  test builds; production `Default` reuses custom timeout construction.

- [ ] **Step 0: Verify the YouTube-slice precondition**

Run:

```powershell
git merge-base --is-ancestor 048221a2 HEAD
git status --short --untracked-files=all
```

Expected: the ancestry command exits 0 and status prints nothing.

- [ ] **Step 1: Record the job warning RED baseline**

Run:

```powershell
$output = & cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1
$text = $output | Out-String
($text -split "`r?`n") | Where-Object { $_ -match 'gemini_browser\\jobs.rs.*warning:' }
```

Expected: four warnings name `Supported`, `run_status_for_queue_state`, `new_with_timeouts`, and `run_log_is_cancelled`.

- [ ] **Step 2: Gate the supported queue mode and keep the predicate exhaustive**

Change the enum and predicate to:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApalisQueueInspectionMode {
    #[cfg(test)]
    Supported,
    DegradedRunLogOnly,
}

pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
    mode: ApalisQueueInspectionMode,
) -> bool {
    match mode {
        #[cfg(test)]
        ApalisQueueInspectionMode::Supported => true,
        ApalisQueueInspectionMode::DegradedRunLogOnly => false,
    }
}
```

Do not change `apalis_queue_inspection_mode`; it continues returning `DegradedRunLogOnly`.

- [ ] **Step 3: Reuse timeout construction and gate the remaining test helpers**

Make `Default` delegate to the existing timeout constructor with the same values:

```rust
impl Default for GeminiBrowserJobRuntime {
    fn default() -> Self {
        Self::new_with_timeouts(
            std::time::Duration::from_secs(DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS + 5),
            std::time::Duration::from_secs(DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS),
            std::time::Duration::from_secs(DEFAULT_WORKER_EXECUTION_TIMEOUT_SECS + 15),
        )
    }
}
```

Keep `new_with_timeouts` available in production without `#[cfg(test)]`. Add
`#[cfg(test)]` immediately above only these two existing helpers, without
changing their bodies:

```rust
#[cfg(test)]
pub(crate) fn run_status_for_queue_state(
    state: &str,
) -> Option<crate::gemini_browser::GeminiBrowserRunStatus> {
    match state {
        "Pending" => Some(crate::gemini_browser::GeminiBrowserRunStatus::Queued),
        "Running" => Some(crate::gemini_browser::GeminiBrowserRunStatus::Running),
        "Done" => Some(crate::gemini_browser::GeminiBrowserRunStatus::Ok),
        "Failed" => Some(crate::gemini_browser::GeminiBrowserRunStatus::Failed),
        "Killed" => Some(crate::gemini_browser::GeminiBrowserRunStatus::Failed),
        _ => None,
    }
}

#[cfg(test)]
fn run_log_is_cancelled(
    runs_root: &std::path::Path,
    run_id: &str,
) -> crate::error::AppResult<bool> {
    Ok(run_log_entry_by_id(runs_root, run_id)?.is_some_and(|run| {
        run.status == crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
    }))
}
```

- [ ] **Step 4: Run focused job tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml gemini_browser::jobs -- --nocapture
```

Expected: all job tests pass, including supported/degraded inspection, queue-state mapping, custom timeouts, reconciliation, cancellation, and worker guards.

- [ ] **Step 5: Verify the intermediate warning baseline**

Run:

```powershell
$output = & cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1
$cargoExit = $LASTEXITCODE
$text = $output | Out-String
$warnings = ($text -split "`r?`n") | Where-Object { $_ -match 'warning:' -and $_ -notmatch '^warning: `extractum`' }
"CARGO_EXIT=$cargoExit"
"INFORMATIONAL_WARNING_COUNT=$($warnings.Count)"
$warnings
if ($text -match 'src\\gemini_browser\\jobs.rs.*warning:') { exit 1 }
exit $cargoExit
```

Expected: `CARGO_EXIT=0`, no warning from `gemini_browser/jobs.rs`, and an
informational repository-wide count of 7. The per-file assertion is the pass/fail
criterion; the count only records the current repository baseline.

- [ ] **Step 6: Commit Task 1**

```powershell
git diff --check
git add src-tauri/src/gemini_browser/jobs.rs
git commit -m "chore: gate gemini job test helpers"
```

---

### Task 2: Gate Gemini Reexports and Component Test Helpers

**Files:**
- Modify: `src-tauri/src/gemini_browser/mod.rs:29-40`
- Modify: `src-tauri/src/gemini_browser/sidecar.rs:263-293`
- Modify: `src-tauri/src/gemini_browser/state.rs:49-61`
- Modify: `src-tauri/src/gemini_browser/types.rs:143-146`
- Tests in place: the respective files' `#[cfg(test)]` modules and dependent prompt-pack/run-log tests.

**Interfaces:**
- Keeps serialized debug structs/enums in `types.rs`.
- Makes their short `crate::gemini_browser::*` reexport test-only.
- Keeps production sidecar transport, cached status reads, terminal-state logic, and wire values unchanged.

- [ ] **Step 1: Record the remaining Gemini warning RED baseline**

Run:

```powershell
$output = & cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1
$text = $output | Out-String
($text -split "`r?`n") | Where-Object { $_ -match 'src\\gemini_browser\\.*warning:' }
```

Expected: six unused symbols produce five compiler diagnostics: the two unused
reexports share one diagnostic, followed by two sidecar helpers, one state
helper, and one status helper.

- [ ] **Step 2: Make debug-type reexports test-only**

Replace the normal type reexport block with the complete list below, then place
the test-only type reexport beside the existing `#[cfg(test)]` jobs reexport near
the top of `mod.rs`:

```rust
pub use types::{
    GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs,
    GeminiBrowserProviderConfig, GeminiBrowserProviderMode, GeminiBrowserProviderStatus,
    GeminiBrowserProviderStatusKind, GeminiBrowserRun, GeminiBrowserRunLogSummary,
    GeminiBrowserRunRequest, GeminiBrowserRunResult, GeminiBrowserRunStatus,
    GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope, GeminiBrowserSidecarResponse,
    GeminiBrowserStartChromeResult,
};

#[cfg(test)]
pub(crate) use types::{GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary};
```

Do not change the type definitions or serialized fields in `types.rs`.

- [ ] **Step 3: Gate sidecar parsing fixtures**

Add `#[cfg(test)]` immediately above the existing definitions:

```rust
#[cfg(test)]
fn decode_sidecar_line(
    id: &str,
    response_line: &str,
) -> AppResult<GeminiBrowserSidecarResponse> {
    decode_sidecar_line_for_request(id, response_line)?
        .ok_or_else(|| AppError::internal("Gemini browser sidecar response id mismatch"))
}

#[cfg(test)]
fn take_complete_jsonl_line(buffer: &mut String) -> Option<String> {
    while let Some(newline_index) = buffer.find('\n') {
        let line: String = buffer.drain(..=newline_index).collect();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        return Some(trimmed.to_string());
    }
    None
}
```

Only attributes are added; production transport parsing code is untouched.

- [ ] **Step 4: Gate direct test setters and predicates**

Add `#[cfg(test)]` above the existing methods:

```rust
#[cfg(test)]
pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    *self.status_snapshot.write() = Some(status);
}

#[cfg(test)]
pub fn is_success(&self) -> bool {
    matches!(self, Self::Ok | Self::Ready)
}
```

Keep `status_snapshot`, `ensure_status_snapshot`, and `is_terminal` production-visible.

- [ ] **Step 5: Run focused Gemini tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml gemini_browser -- --nocapture
```

Expected: all Gemini Browser tests pass, including jobs, commands, run log, sidecar framing, state snapshots, and status predicates.

- [ ] **Step 6: Verify the final warning baseline**

Run:

```powershell
$output = & cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1
$cargoExit = $LASTEXITCODE
$text = $output | Out-String
$warnings = ($text -split "`r?`n") | Where-Object { $_ -match 'warning:' -and $_ -notmatch '^warning: `extractum`' }
"CARGO_EXIT=$cargoExit"
"INFORMATIONAL_WARNING_COUNT=$($warnings.Count)"
$warnings
if ($text -match 'src\\gemini_browser\\.*warning:') { exit 1 }
exit $cargoExit
```

Expected: `CARGO_EXIT=0`, no warning from `gemini_browser/`, and an informational
repository-wide count of 2 with warnings only in `apalis_jobs.rs` and
`youtube/jobs.rs`. The path assertion, not the count, is the pass/fail criterion.

- [ ] **Step 7: Run full Rust verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: the full Rust suite passes with zero failures and all targets check
successfully with no warnings from `gemini_browser/`.

- [ ] **Step 8: Commit Task 2**

```powershell
git diff --check
git status --short --untracked-files=all
git add src-tauri/src/gemini_browser/mod.rs src-tauri/src/gemini_browser/sidecar.rs src-tauri/src/gemini_browser/state.rs src-tauri/src/gemini_browser/types.rs
git commit -m "chore: gate gemini component test helpers"
```

Expected: the working tree is clean after the commit.
