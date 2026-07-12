# YouTube Process Runtime Warning Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate and land the prepared `youtube/process_runtime.rs` cleanup that removes its production Rust warnings without changing yt-dlp behavior.

**Architecture:** Preserve the existing user-authored working-tree diff. Verify that production-only API remains available, test-only helpers are gated, ownership semantics are unchanged, and the warning baseline falls from 23 to 11 before committing the file.

**Tech Stack:** Rust 2021, Tokio, Cargo, Windows process tests.

## Global Constraints

- Do not revert or rewrite the prepared `src-tauri/src/youtube/process_runtime.rs` changes merely to manufacture a new RED run.
- The pre-change RED evidence is the recorded `cargo check` baseline of 23 warnings, including 10 warning locations in `youtube/process_runtime.rs` plus two unnecessary-`mut` warnings.
- Do not change yt-dlp launch, cancellation, timeout, Job Object, output-draining, registry, error-classification, or cookie-lifetime behavior.
- Do not modify `docs/project.md` or `docs/value-registry.md`.
- Stage only `src-tauri/src/youtube/process_runtime.rs`; spec and plan are already separate commits.

---

### Task 1: Validate and Land the Prepared Warning Cleanup

**Files:**
- Modify (already prepared): `src-tauri/src/youtube/process_runtime.rs:23-307`
- Test in place: `src-tauri/src/youtube/process_runtime.rs:326-704`

**Interfaces:**
- Retains production entry point `run_ytdlp_managed_with_cancellation(...)`.
- Retains `CookieLifetimeGuard::new(NamedTempFile)` and ownership through managed/detached reaping.
- Restricts generic launcher and registry inspection helpers to `#[cfg(test)]`.
- Removes unused `run_ytdlp_managed(...)`.

- [ ] **Step 1: Confirm working-tree ownership and scope**

Run:

```powershell
git status --short --untracked-files=all
git diff --stat
git diff --check
git diff -- src-tauri/src/youtube/process_runtime.rs
```

Expected: only `src-tauri/src/youtube/process_runtime.rs` is modified; its diff contains the approved ownership field, test gates, wrapper removal, and two `mut` removals; diff check exits 0.

- [ ] **Step 2: Verify production and test boundaries by source inspection**

Run:

```powershell
rg -n "run_ytdlp_managed_with_cancellation|run_ytdlp_managed\(|#\[cfg\(test\)\]|_cookie|stdout_task|stderr_task" src-tauri/src/youtube/process_runtime.rs src-tauri/src/youtube/ytdlp.rs
```

Expected:

- `youtube/ytdlp.rs` imports and calls `run_ytdlp_managed_with_cancellation`;
- no definition or call of the removed `run_ytdlp_managed` wrapper remains;
- the approved helper functions are immediately gated by `#[cfg(test)]`;
- `CookieLifetimeGuard` owns `_cookie: tempfile::NamedTempFile`;
- stdout/stderr task bindings are not `mut`.

- [ ] **Step 3: Run focused behavior verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::process_runtime -- --nocapture
```

Expected: 14 tests pass, including backpressure, cancellation, timeout, detached reap, registry ownership, and cookie lifetime tests.

- [ ] **Step 4: Verify the production warning delta**

Run:

```powershell
$output = & cargo check --manifest-path src-tauri/Cargo.toml --message-format=short 2>&1
$cargoExit = $LASTEXITCODE
$warnings = $output | Where-Object { $_ -match 'warning:' -and $_ -notmatch '^warning: `extractum`' }
"CARGO_EXIT=$cargoExit"
"WARNING_COUNT=$($warnings.Count)"
$warnings
if ($warnings -match 'youtube\\process_runtime.rs') { exit 1 }
exit $cargoExit
```

Expected: `CARGO_EXIT=0`, `WARNING_COUNT=11`, and no warning path mentions `youtube\process_runtime.rs`.

- [ ] **Step 5: Run full Rust verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: the complete Rust suite passes with zero failures.

- [ ] **Step 6: Inspect the final diff and commit**

Run:

```powershell
git diff --check
git status --short --untracked-files=all
git add src-tauri/src/youtube/process_runtime.rs
git commit -m "chore: clean youtube process runtime warnings"
```

Expected: the commit contains only `src-tauri/src/youtube/process_runtime.rs`, and the working tree is clean afterward.
