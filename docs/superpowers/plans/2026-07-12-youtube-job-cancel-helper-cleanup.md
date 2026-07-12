# YouTube Job Cancellation Helper Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the final Rust warning by deleting an obsolete cancellation helper and moving its tests onto the production helper.

**Architecture:** Preserve all production call sites and the existing `run_source_job_step_with_cancel_and_processes` implementation. Migrate two cancellation tests to that production helper with fresh process registries, strengthen the cancelled result assertion, then delete the duplicate helper.

**Tech Stack:** Rust 2021, Cargo, Tokio, Tokio Util.

## Global Constraints

- Modify only `src-tauri/src/youtube/jobs.rs` during implementation.
- Apalis warning cleanup commit `8cbbfd9c` must be an ancestor of `HEAD` before implementation begins.
- Do not add `allow(dead_code)` or another warning suppression.
- Do not change production source-job call sites, process-registry behavior, yt-dlp lifecycle, Tauri commands, DTOs, status values, error text, TypeScript code, or serialized values.
- Do not edit `docs/project.md` or `docs/value-registry.md`.
- Accept that the fresh empty registry in the migrated test cannot directly prove the `registry.cancel_all()` call; a registry observation seam is outside this slice.

---

### Task 1: Migrate Cancellation Tests and Remove the Duplicate Helper

**Files:**
- Modify: `src-tauri/src/youtube/jobs.rs:722-741,1206-1215,1512-1530`
- Test in place: `src-tauri/src/youtube/jobs.rs:1205-1800`
- Related verification only: `src-tauri/src/youtube/process_runtime.rs:330-690`

**Interfaces:**
- Keeps production `run_source_job_step_with_cancel_and_processes<Fut, T>(cancellation_token: Option<CancellationToken>, registry: YoutubeProcessRegistry, future: Fut) -> AppResult<T>` unchanged.
- Removes private duplicate `run_source_job_step_with_cancel<Fut, T>`.
- Produces two renamed tests that call the production helper with `YoutubeProcessRegistry::new()`.

- [ ] **Step 1: Verify prerequisites**

Run:

```powershell
git merge-base --is-ancestor 8cbbfd9c HEAD
$ancestryExit = $LASTEXITCODE
git status --short --untracked-files=all
"ANCESTRY_EXIT=$ancestryExit"
exit $ancestryExit
```

Expected: `ANCESTRY_EXIT=0` and `git status` prints nothing.

- [ ] **Step 2: Record the named warning RED baseline**

Run:

```powershell
$output = & cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1
$cargoExit = $LASTEXITCODE
$text = $output | Out-String
$warnings = ($text -split "`r?`n") | Where-Object { $_ -match 'warning:' -and $_ -notmatch '^warning: `extractum`' }
"CARGO_EXIT=$cargoExit"
"INFORMATIONAL_WARNING_COUNT=$($warnings.Count)"
$warnings
if ($text -notmatch 'src\\youtube\\jobs.rs.*warning: function `run_source_job_step_with_cancel` is never used') { exit 1 }
exit $cargoExit
```

Expected: `CARGO_EXIT=0` and the diagnostic names
`run_source_job_step_with_cancel` in `src\youtube\jobs.rs`. The expected
repository-wide count is 1, but the named diagnostic is the authoritative RED
condition because PowerShell 5.1 may expand the first native stderr
`ErrorRecord` across multiple lines.

- [ ] **Step 3: Migrate test imports to the production helper**

Replace the test module imports with this complete block:

```rust
use super::{
    clear_source_job_cancellation_smoke_fixture,
    finish_cancelled_source_job_cancellation_smoke_fixture, retryable_playlist_video_rows,
    run_source_job_step_with_cancel_and_processes,
    seed_source_job_cancellation_smoke_fixture_in_state, SourceJobListFilter, SourceJobState,
    SourceJobStatus, SourceJobType, YoutubeProcessRegistry, YoutubeSyncOptions,
    SOURCE_JOB_CANCELLATION_SMOKE_FIXTURE_SOURCE_ID,
};
use crate::error::{AppError, AppErrorKind};
```

Keep the existing `CancellationToken` import immediately below this block.

- [ ] **Step 4: Migrate and strengthen the two tests**

Replace the two old wrapper tests with:

```rust
#[tokio::test]
async fn source_job_step_with_process_cancel_allows_completed_future() {
    let result = run_source_job_step_with_cancel_and_processes(
        None,
        YoutubeProcessRegistry::new(),
        async { Ok::<_, AppError>("done") },
    )
    .await
    .expect("step result");

    assert_eq!(result, "done");
}

#[tokio::test]
async fn source_job_step_with_process_cancel_interrupts_pending_future() {
    let token = CancellationToken::new();
    token.cancel();

    let error = run_source_job_step_with_cancel_and_processes(
        Some(token),
        YoutubeProcessRegistry::new(),
        std::future::pending::<crate::error::AppResult<()>>(),
    )
    .await
    .expect_err("cancelled source step");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(error.message, "Source job cancelled");
}
```

- [ ] **Step 5: Run the migrated tests before deleting the old helper**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml source_job_step_with_process_cancel -- --nocapture
```

Expected: both renamed tests pass. The old helper still exists at this point,
so the warning remains until Step 6.

- [ ] **Step 6: Delete the obsolete helper**

Delete this complete function and no surrounding code:

```rust
async fn run_source_job_step_with_cancel<Fut, T>(
    cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> AppResult<T>
where
    Fut: Future<Output = AppResult<T>>,
{
    let Some(cancellation_token) = cancellation_token else {
        return future.await;
    };

    if cancellation_token.is_cancelled() {
        return Err(AppError::validation("Source job cancelled"));
    }

    tokio::select! {
        result = future => result,
        _ = cancellation_token.cancelled() => Err(AppError::validation("Source job cancelled")),
    }
}
```

Do not change `run_source_job_step_with_cancel_and_processes` or its production
call sites.

- [ ] **Step 7: Run focused YouTube tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::jobs -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml youtube::process_runtime -- --nocapture
```

Expected: both focused suites pass, including the two renamed jobs tests and
the managed process cancellation/reap tests.

- [ ] **Step 8: Verify the zero-warning GREEN state**

Run:

```powershell
$output = & cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1
$cargoExit = $LASTEXITCODE
$text = $output | Out-String
$warnings = ($text -split "`r?`n") | Where-Object { $_ -match 'warning:' -and $_ -notmatch '^warning: `extractum`' }
"CARGO_EXIT=$cargoExit"
"WARNING_COUNT=$($warnings.Count)"
$warnings
if ($warnings.Count -ne 0) { exit 1 }
exit $cargoExit
```

Expected: `CARGO_EXIT=0` and `WARNING_COUNT=0`.

- [ ] **Step 9: Review and commit the implementation**

Run:

```powershell
git diff --check
git diff -- src-tauri/src/youtube/jobs.rs
git status --short --untracked-files=all
git add src-tauri/src/youtube/jobs.rs
git commit -m "chore: remove obsolete youtube job cancel helper"
```

Expected: only the old helper, test imports, test names, test calls, and stronger
error assertions change; the commit succeeds and the working tree is clean.
