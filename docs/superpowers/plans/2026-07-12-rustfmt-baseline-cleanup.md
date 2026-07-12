# Rustfmt Baseline Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a clean Rust formatting baseline by applying the pinned default rustfmt output to an exact 17-file allowlist without behavioral edits.

**Architecture:** Verify the formatter version and failing baseline before mutation, run `cargo fmt` once, and reject any changed path outside the recorded allowlist. Validate formatting, compilation, warnings, and the full Rust suite before one isolated style commit.

**Tech Stack:** Rust 2021, Cargo, rustfmt 1.9.0-stable.

## Global Constraints

- Use exactly `rustfmt 1.9.0-stable (59807616e1 2026-04-14)` with its default configuration.
- Do not create `rustfmt.toml` or `.rustfmt.toml` in this slice.
- Run only the formatter for implementation; do not make manual source edits.
- Do not change documentation, Cargo manifests, lockfiles, migrations, TypeScript, Svelte, JSON, configuration, generated assets, or `docs/value-registry.md`.
- Accept only repo-relative changed paths from the 17-file allowlist in Task 1.
- Keep enforcement and `.git-blame-ignore-revs` for the required follow-up after the style commit hash exists.

---

### Task 1: Apply and Verify the Repository Rustfmt Baseline

**Files:**
- Format: `src-tauri/src/gemini_browser/cdp_chrome.rs`
- Format: `src-tauri/src/gemini_browser/mod.rs`
- Format: `src-tauri/src/gemini_browser/sidecar.rs`
- Format: `src-tauri/src/gemini_browser/state.rs`
- Format: `src-tauri/src/lib.rs`
- Format: `src-tauri/src/llm/profiles.rs`
- Format: `src-tauri/src/process_tree.rs`
- Format: `src-tauri/src/projects/mod.rs`
- Format: `src-tauri/src/prompt_packs/result_builder.rs`
- Format: `src-tauri/src/youtube/captions.rs`
- Format: `src-tauri/src/youtube/comments.rs`
- Format: `src-tauri/src/youtube/jobs.rs`
- Format: `src-tauri/src/youtube/metadata.rs`
- Format: `src-tauri/src/youtube/mod.rs`
- Format: `src-tauri/src/youtube/preview.rs`
- Format: `src-tauri/src/youtube/process_runtime.rs`
- Format: `src-tauri/src/youtube/ytdlp.rs`

**Interfaces:**
- Produces no new or changed Rust interface.
- Preserves all functions, types, imports, values, tests, and behavior while changing only rustfmt-controlled layout.

- [ ] **Step 1: Verify the clean-tree and formatter-version preconditions**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
$version = cargo fmt --version
$expected = 'rustfmt 1.9.0-stable (59807616e1 2026-04-14)'
"STATUS_COUNT=$($status.Count)"
"RUSTFMT_VERSION=$version"
if ($status.Count -ne 0 -or $version -ne $expected) { exit 1 }
```

Expected: `STATUS_COUNT=0` and `RUSTFMT_VERSION` exactly matches the pinned
value.

- [ ] **Step 2: Record the failing formatter RED baseline**

Run:

```powershell
& cargo fmt --manifest-path src-tauri/Cargo.toml -- --check *> $null
$fmtExit = $LASTEXITCODE
"RUSTFMT_RED_EXIT=$fmtExit"
if ($fmtExit -eq 0) { exit 1 }
```

Expected: `RUSTFMT_RED_EXIT=1`.

- [ ] **Step 3: Run the formatter once**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: command exits 0. Do not manually edit any formatter output.

- [ ] **Step 4: Verify the exact changed-file allowlist**

Run:

```powershell
$allowed = @(
    'src-tauri/src/gemini_browser/cdp_chrome.rs',
    'src-tauri/src/gemini_browser/mod.rs',
    'src-tauri/src/gemini_browser/sidecar.rs',
    'src-tauri/src/gemini_browser/state.rs',
    'src-tauri/src/lib.rs',
    'src-tauri/src/llm/profiles.rs',
    'src-tauri/src/process_tree.rs',
    'src-tauri/src/projects/mod.rs',
    'src-tauri/src/prompt_packs/result_builder.rs',
    'src-tauri/src/youtube/captions.rs',
    'src-tauri/src/youtube/comments.rs',
    'src-tauri/src/youtube/jobs.rs',
    'src-tauri/src/youtube/metadata.rs',
    'src-tauri/src/youtube/mod.rs',
    'src-tauri/src/youtube/preview.rs',
    'src-tauri/src/youtube/process_runtime.rs',
    'src-tauri/src/youtube/ytdlp.rs'
)
$changed = @(git status --porcelain=v1 --untracked-files=all | ForEach-Object {
    $_.Substring(3).Replace('\', '/')
})
$unexpected = @($changed | Where-Object { $_ -notin $allowed })
$missing = @($allowed | Where-Object { $_ -notin $changed })
"CHANGED_COUNT=$($changed.Count)"
"UNEXPECTED=$($unexpected -join ',')"
"MISSING=$($missing -join ',')"
if ($unexpected.Count -ne 0 -or $missing.Count -ne 0) { exit 1 }
```

Expected: `CHANGED_COUNT=17`, with empty `UNEXPECTED` and `MISSING` values.
The comparison uses Git's repo-relative paths, not rustfmt's Windows absolute
paths.

- [ ] **Step 5: Verify the formatting GREEN state**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: command exits 0 with no diff output.

- [ ] **Step 6: Verify all Rust targets with zero warnings**

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

- [ ] **Step 7: Run the full Rust test suite**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: the full suite passes with zero failures.

- [ ] **Step 8: Review and commit the formatter output**

Run:

```powershell
git diff --check
git diff --stat
git status --short --untracked-files=all
git add src-tauri/src/gemini_browser/cdp_chrome.rs `
    src-tauri/src/gemini_browser/mod.rs `
    src-tauri/src/gemini_browser/sidecar.rs `
    src-tauri/src/gemini_browser/state.rs `
    src-tauri/src/lib.rs `
    src-tauri/src/llm/profiles.rs `
    src-tauri/src/process_tree.rs `
    src-tauri/src/projects/mod.rs `
    src-tauri/src/prompt_packs/result_builder.rs `
    src-tauri/src/youtube/captions.rs `
    src-tauri/src/youtube/comments.rs `
    src-tauri/src/youtube/jobs.rs `
    src-tauri/src/youtube/metadata.rs `
    src-tauri/src/youtube/mod.rs `
    src-tauri/src/youtube/preview.rs `
    src-tauri/src/youtube/process_runtime.rs `
    src-tauri/src/youtube/ytdlp.rs
git commit -m "style: format rust sources"
```

Expected: only the 17 allowlisted files are staged, the style commit succeeds,
and the working tree is clean afterward. Record the resulting commit hash for
the required enforcement and `.git-blame-ignore-revs` follow-up.
