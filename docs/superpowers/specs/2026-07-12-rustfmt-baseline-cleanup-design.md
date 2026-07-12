# Rustfmt Baseline Cleanup Design

## Goal

Create a clean repository-wide Rust formatting baseline with one mechanical
`cargo fmt` change and no behavioral edits.

## Current State

`cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` exits nonzero and
reports formatting differences in exactly these 17 files:

- `src-tauri/src/gemini_browser/cdp_chrome.rs`
- `src-tauri/src/gemini_browser/mod.rs`
- `src-tauri/src/gemini_browser/sidecar.rs`
- `src-tauri/src/gemini_browser/state.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/llm/profiles.rs`
- `src-tauri/src/process_tree.rs`
- `src-tauri/src/projects/mod.rs`
- `src-tauri/src/prompt_packs/result_builder.rs`
- `src-tauri/src/youtube/captions.rs`
- `src-tauri/src/youtube/comments.rs`
- `src-tauri/src/youtube/jobs.rs`
- `src-tauri/src/youtube/metadata.rs`
- `src-tauri/src/youtube/mod.rs`
- `src-tauri/src/youtube/preview.rs`
- `src-tauri/src/youtube/process_runtime.rs`
- `src-tauri/src/youtube/ytdlp.rs`

The differences are rustfmt output accumulated across earlier focused slices.
They include line wrapping, import ordering, block layout, and expression
formatting. Keeping them outstanding adds unrelated noise to future module
decomposition work.

The repository currently has no CI workflow, hook, package script, or project
verification command that enforces `cargo fmt --check`. This slice creates a
manual baseline only; it does not prevent formatting debt from returning.

The formatter used to record the baseline is:

```text
rustfmt 1.9.0-stable (59807616e1 2026-04-14)
```

No `rustfmt.toml` or `.rustfmt.toml` exists, so the baseline uses that
formatter version's default style.

## Selected Design

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Accept only the formatter-produced changes in the 17-file allowlist above.
Do not manually edit, simplify, rename, reorder behavior, or combine this work
with another refactor.

Commit all formatter output as one isolated `style:` commit. A single commit is
preferred because it keeps mechanical review and future history filtering
simple and separates the repository-wide churn from semantic changes.

## Rejected Alternatives

- Splitting by subsystem would make each diff smaller but leave the formatter
  baseline incomplete across intermediate commits and complicate blame
  filtering.
- Formatting only recently touched files would preserve unrelated formatting
  debt and fail to establish a clean baseline.
- Manually reproducing rustfmt changes would be slower and less reliable than
  using the configured formatter.
- Mixing formatting with the upcoming `gemini_browser/jobs.rs` decomposition
  would obscure semantic review.

## Scope and Change Control

Only the 17 listed Rust files may change. No documentation, Cargo manifests,
lockfiles, migrations, TypeScript, Svelte, JSON, configuration, generated
assets, or string-value registries may change.

Before formatting, record `cargo fmt --version` and require the exact version
shown above. A version mismatch stops the slice before any files are changed;
formatter policy or toolchain upgrades require a revised design.

After formatting, compare the repo-relative paths from
`git status --short --untracked-files=all` with the allowlist. Do not compare
against formatter output because Windows rustfmt paths use the
`\\?\G:\...` absolute-path form. Any extra changed file stops the slice for
investigation. A missing allowlisted file is acceptable only if the pinned
formatter no longer changes it; the final actual set must still be a subset of
the recorded allowlist.

No value registry update is required because no status, state, kind, mode,
phase, type, provider, subtype, scope, severity, wire value, or persisted value
changes.

## Required Follow-Up

The formatting commit hash is known only after this slice completes. A small
follow-up must:

- choose and add an enforcement mechanism for `cargo fmt --check`—CI when a
  workflow exists, a repository hook, or an explicit project verification
  convention—and document the chosen developer command;
- create `.git-blame-ignore-revs` containing the formatting commit hash;
- document the optional local command
  `git config blame.ignoreRevsFile .git-blame-ignore-revs`.

Until that follow-up lands, the clean formatting baseline is not automatically
enforced. This limitation is accepted for the mechanical cleanup slice.

## Verification

- Record the exact formatter version, pre-change formatter failure, and
  17-file allowlist.
- Run the formatter once without manual edits.
- Use `git status --short --untracked-files=all` to verify that all changed
  repo-relative paths are allowlisted Rust files.
- Run `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and require
  exit 0 with no diff output.
- Run `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` and
  require exit 0 with zero warnings.
- Run `cargo test --manifest-path src-tauri/Cargo.toml` and require the full
  Rust suite to pass.
- Run `git diff --check` before commit.

## Acceptance Criteria

- The working diff contains only rustfmt output in the allowlisted files.
- No manual or behavioral edit is included.
- Repository-wide `cargo fmt --check` passes.
- `cargo check --all-targets` passes with zero warnings.
- The full Rust test suite passes.
- The formatting cleanup is committed separately with a `style:` message.
- The follow-up enforcement and blame-ignore work is explicitly handed off
  after the style commit hash is known.
