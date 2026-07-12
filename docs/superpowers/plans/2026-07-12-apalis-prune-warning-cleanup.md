# Apalis Prune Warning Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the production dead-code warning for the default Apalis prune test wrapper without changing pruning behavior.

**Architecture:** Express the existing boundary directly by compiling `apalis_jobs_prune_terminal_from_pool` only in test builds. Keep the production Tauri command, parameterized pool helper, retention constant, SQL, DTOs, and existing tests unchanged.

**Tech Stack:** Rust 2021, Cargo, Tokio, SQLx, Tauri.

## Global Constraints

- Modify only `src-tauri/src/apalis_jobs.rs` during implementation.
- Do not add `allow(dead_code)` or another warning suppression.
- Do not change Tauri commands, DTOs, SQL, retention values, TypeScript code, or serialized values.
- Do not edit `docs/project.md` or `docs/value-registry.md`.
- Preserve the remaining `youtube/jobs.rs` warning for its own later slice.

---

### Task 1: Gate the Default Prune Test Wrapper

**Files:**
- Modify: `src-tauri/src/apalis_jobs.rs:161-167`
- Test in place: `src-tauri/src/apalis_jobs.rs:1008-1141`

**Interfaces:**
- Keeps production `apalis_jobs_prune_terminal(handle, request)` unchanged.
- Keeps production `apalis_jobs_prune_terminal_from_pool_with_hours(pool, now_secs, older_than_hours)` unchanged.
- Keeps test-only `apalis_jobs_prune_terminal_from_pool(pool, now_secs)` with the same signature and default 24-hour retention behavior.

- [ ] **Step 1: Verify the clean-tree precondition**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: no output.

- [ ] **Step 2: Record the warning RED baseline**

Run:

```powershell
$output = & cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1
$cargoExit = $LASTEXITCODE
$text = $output | Out-String
$warnings = ($text -split "`r?`n") | Where-Object { $_ -match 'warning:' -and $_ -notmatch '^warning: `extractum`' }
"CARGO_EXIT=$cargoExit"
"INFORMATIONAL_WARNING_COUNT=$($warnings.Count)"
$warnings
if ($text -notmatch 'src\\apalis_jobs.rs.*apalis_jobs_prune_terminal_from_pool') { exit 1 }
exit $cargoExit
```

Expected: `CARGO_EXIT=0`, informational count 2, and one diagnostic names
`apalis_jobs_prune_terminal_from_pool` in `src\apalis_jobs.rs`. The named
diagnostic is the RED condition; the repository-wide count is informational.
On PowerShell 5.1, native stderr may render its first `ErrorRecord` across
multiple lines and make the informational count appear as 3 instead of 2; this
does not affect the required substring assertion.

- [ ] **Step 3: Add the test-only compilation boundary**

Add this attribute immediately above the existing wrapper declaration:

```rust
#[cfg(test)]
```

Do not reformat the wrapper body. Do not change `apalis_jobs_prune_terminal` or
`apalis_jobs_prune_terminal_from_pool_with_hours`.

- [ ] **Step 4: Run focused Apalis tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml apalis_jobs::tests -- --nocapture
```

Expected: all `apalis_jobs` tests pass, including
`apalis_jobs_prune_terminal_deletes_only_old_done_killed_and_terminal_failed_jobs`
and `apalis_jobs_prune_terminal_returns_zero_when_jobs_table_missing`.

- [ ] **Step 5: Verify the warning GREEN state**

Run:

```powershell
$output = & cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1
$cargoExit = $LASTEXITCODE
$text = $output | Out-String
$warnings = ($text -split "`r?`n") | Where-Object { $_ -match 'warning:' -and $_ -notmatch '^warning: `extractum`' }
"CARGO_EXIT=$cargoExit"
"INFORMATIONAL_WARNING_COUNT=$($warnings.Count)"
$warnings
if ($text -match 'src\\apalis_jobs.rs.*warning:') { exit 1 }
if ($text -notmatch 'src\\youtube\\jobs.rs.*run_source_job_step_with_cancel') { exit 1 }
exit $cargoExit
```

Expected: `CARGO_EXIT=0`, no warning from `apalis_jobs.rs`, and informational
count 1 naming only `run_source_job_step_with_cancel` in `youtube/jobs.rs`.
On PowerShell 5.1, the informational count may be one higher because of native
stderr `ErrorRecord` rendering; the two path assertions remain authoritative.

- [ ] **Step 6: Review and commit the implementation**

Run:

```powershell
git diff --check
git diff -- src-tauri/src/apalis_jobs.rs
git status --short --untracked-files=all
git add src-tauri/src/apalis_jobs.rs
git commit -m "chore: gate apalis prune test helper"
```

Expected: the diff contains only the `#[cfg(test)]` attribute, the commit
succeeds, and the working tree is clean afterward.
