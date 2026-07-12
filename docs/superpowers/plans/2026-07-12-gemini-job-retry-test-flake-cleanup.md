# Gemini Job Retry Test Flake Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove one exact duplicate Gemini Browser retry test to reduce a reproducible parallel schema-flake trigger without changing behavioral coverage or production code.

**Architecture:** Keep the retained no-retry test and its shared assertion helper unchanged. Delete only the redundant adapter, verify its source-level absence, then use repeated focused and full-suite runs to distinguish retained-test stability from suite-wide concurrency recurrence.

**Tech Stack:** Rust 2021, Cargo, Tokio, SQLx, SQLite, Apalis.

## Global Constraints

- Modify only `src-tauri/src/gemini_browser/jobs.rs` during implementation.
- Remove only `failed_gemini_browser_job_retry_is_not_attempted`.
- Keep `failed_gemini_browser_job_is_not_retried` and `assert_failed_gemini_browser_job_is_not_retried()` unchanged.
- Do not add a mutex, test serialization, retries, sleeps, warning suppressions, or migration workarounds.
- Do not change production code, application migrations, Apalis setup, database helpers, dependencies, serialized values, TypeScript code, `docs/project.md`, or `docs/value-registry.md`.
- Treat any later `Jobs` error matching `N columns but M values were supplied` in any test as a recurrence requiring deeper migration investigation.

---

### Task 1: Remove the Duplicate Retry Test Adapter

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs:2087-2097`
- Retained test helper: `src-tauri/src/gemini_browser/jobs.rs:2518-2566`

**Interfaces:**
- Keeps test `failed_gemini_browser_job_is_not_retried()` unchanged.
- Keeps helper `assert_failed_gemini_browser_job_is_not_retried()` unchanged.
- Removes only test `failed_gemini_browser_job_retry_is_not_attempted()`.

- [ ] **Step 1: Verify the clean-tree precondition**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: no output.

- [ ] **Step 2: Record the mechanical source RED state**

Run:

```powershell
$source = Get-Content -Raw src-tauri/src/gemini_browser/jobs.rs
$retained = ([regex]::Matches(
    $source,
    '(?m)^\s*async fn failed_gemini_browser_job_is_not_retried\(\)'
)).Count
$duplicate = ([regex]::Matches(
    $source,
    '(?m)^\s*async fn failed_gemini_browser_job_retry_is_not_attempted\(\)'
)).Count
"RETAINED_COUNT=$retained"
"DUPLICATE_COUNT=$duplicate"
if ($retained -ne 1 -or $duplicate -ne 1) { exit 1 }
```

Expected: `RETAINED_COUNT=1` and `DUPLICATE_COUNT=1`.

- [ ] **Step 3: Delete only the redundant adapter**

Delete exactly this function:

```rust
#[tokio::test]
async fn failed_gemini_browser_job_retry_is_not_attempted() {
    assert_failed_gemini_browser_job_is_not_retried().await;
}
```

Do not change the retained test or shared assertion helper.

- [ ] **Step 4: Verify source GREEN and retained-test stability**

Run the source contract:

```powershell
$source = Get-Content -Raw src-tauri/src/gemini_browser/jobs.rs
$retained = ([regex]::Matches(
    $source,
    '(?m)^\s*async fn failed_gemini_browser_job_is_not_retried\(\)'
)).Count
$duplicate = ([regex]::Matches(
    $source,
    '(?m)^\s*async fn failed_gemini_browser_job_retry_is_not_attempted\(\)'
)).Count
"RETAINED_COUNT=$retained"
"DUPLICATE_COUNT=$duplicate"
if ($retained -ne 1 -or $duplicate -ne 0) { exit 1 }
```

Expected: `RETAINED_COUNT=1` and `DUPLICATE_COUNT=0`.

Then run the retained test 20 times:

```powershell
$failedRuns = @()
1..20 | ForEach-Object {
    & cargo test --manifest-path src-tauri/Cargo.toml `
        gemini_browser::jobs::tests::failed_gemini_browser_job_is_not_retried `
        -- --exact *> $null
    if ($LASTEXITCODE -ne 0) { $failedRuns += $_ }
}
"RETAINED_FAILURE_COUNT=$($failedRuns.Count)"
"RETAINED_FAILED_RUNS=$($failedRuns -join ',')"
if ($failedRuns.Count -ne 0) { exit 1 }
```

Expected: `RETAINED_FAILURE_COUNT=0`. These 20 isolated repetitions verify the
retained test's own stability; they do not recreate the original parallel
collision. Suite-wide recurrence detection is provided by Step 6.

- [ ] **Step 5: Run the complete Gemini jobs test group**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml gemini_browser::jobs -- --nocapture
```

Expected: every `gemini_browser::jobs` test passes with zero failures.

- [ ] **Step 6: Run the full Rust suite three times**

Run:

```powershell
$schemaRace = 'table Jobs has \d+ columns but \d+ values were supplied'
1..3 | ForEach-Object {
    $run = $_
    $output = & cargo test --manifest-path src-tauri/Cargo.toml 2>&1
    $cargoExit = $LASTEXITCODE
    $text = $output | Out-String
    "FULL_RUN_${run}_EXIT=$cargoExit"
    if ($text -match $schemaRace) {
        $output
        Write-Error "Jobs schema-race signature recurred in full run $run"
        exit 1
    }
    if ($cargoExit -ne 0) {
        $output
        exit $cargoExit
    }
}
```

Expected: all three lines report exit 0 and no output contains a `Jobs`
`N columns but M values were supplied` error. A matching recurrence stops this
slice and reopens the deeper migration investigation.

- [ ] **Step 7: Verify the zero-warning all-targets state**

Run:

```powershell
$output = & cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1
$cargoExit = $LASTEXITCODE
$text = $output | Out-String
$warnings = ($text -split "`r?`n") | Where-Object {
    $_ -match 'warning:' -and $_ -notmatch '^warning: `extractum`'
}
"CARGO_EXIT=$cargoExit"
"WARNING_COUNT=$($warnings.Count)"
$warnings
if ($warnings.Count -ne 0) { exit 1 }
exit $cargoExit
```

Expected: `CARGO_EXIT=0` and `WARNING_COUNT=0`.

- [ ] **Step 8: Review and commit the implementation**

Run:

```powershell
git diff --check
git diff -- src-tauri/src/gemini_browser/jobs.rs
git status --short --untracked-files=all
git add src-tauri/src/gemini_browser/jobs.rs
git commit -m "test: remove duplicate gemini retry test"
```

Expected: the diff deletes exactly the four-line duplicate test adapter, the
commit succeeds, and the working tree is clean afterward.
