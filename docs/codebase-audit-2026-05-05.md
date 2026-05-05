# Codebase Audit - 2026-05-05

## Scope

This audit covered the Extractum codebase end to end, with security findings
intentionally out of scope. The focus was product correctness, data integrity,
operational reliability, maintainability, testability, and release stability.

CodeRabbit could not be used in this environment because `coderabbit --version`
failed with `Wsl/Service/E_ACCESSDENIED`. The findings below are from a manual
code audit plus local verification.

## Executive Summary

The project has a solid amount of pure-logic and storage test coverage, and the
current frontend and backend test suites pass. The most serious remaining risks
are around large real-world archives and long-running operations:

- analysis runs can load an unbounded corpus and enqueue one LLM map request per
  chunk without a preflight budget;
- account deletion bypasses the source ingest lock model and can race active
  sync/Takeout work;
- Takeout import writes partial rows directly into the main `items` table, so
  failed/cancelled imports become indistinguishable from completed history;
- the Rust lint baseline is not green and there is no single full-project
  verification command or CI gate;
- Telegram integration depends on `grammers` from a moving `master` branch.

## Findings

### 1. Critical: Analysis runs have no corpus or request budget

Evidence:

- `src-tauri/src/analysis/corpus.rs:55` loads all matching analysis rows for the
  selected sources and date range.
- `src-tauri/src/analysis/corpus.rs:84` materializes the whole result with
  `fetch_all`.
- `src-tauri/src/analysis/report.rs:715` loads that full corpus before any
  budget check.
- `src-tauri/src/analysis/report.rs:738` chunks the already-loaded corpus.
- `src-tauri/src/analysis/report.rs:434` spawns one map task per chunk.

Impact:

Large Telegram archives can create memory pressure, long local stalls, large
LLM costs, and very large request queues. The scheduler limits concurrent
execution, but it does not limit how many chunk tasks are created for a run or
how much corpus is loaded before chunking. The UI only validates date order and
template/scope selection; it does not provide a preflight message count, token
estimate, or hard cap.

Suggested fix:

1. Add a backend preflight step that counts eligible text messages and estimates
   chunk count/tokens before creating the run.
2. Enforce configurable limits for max messages, max chunks, max estimated input
   tokens, and max background requests per run.
3. Stream or page corpus loading into chunk builders instead of materializing all
   rows at once.
4. Surface the estimate in the UI and require explicit confirmation for large
   runs.
5. Add tests for rejection at the budget boundary and for chunk construction
   without full-corpus materialization.

### 2. Critical: Account deletion bypasses active ingest coordination

Evidence:

- `src-tauri/src/source_ingest.rs:40` coordinates active operations by
  `source_id`.
- `src-tauri/src/sources/sync.rs:204`, `src-tauri/src/takeout_import/mod.rs:200`,
  and `src-tauri/src/sources/store.rs:37` use that lock for sync, Takeout
  import, and source deletion.
- `src-tauri/src/accounts.rs:88` defines `delete_account`, but
  `src-tauri/src/accounts.rs:94` deletes the account directly and
  `src-tauri/src/accounts.rs:99` clears runtime state without acquiring source
  locks, cancelling Takeout jobs, or checking `rows_affected`.

Impact:

Deleting an account while one of its sources is syncing or importing can cascade
delete sources/items underneath active tasks. Those tasks may continue with
cloned Telegram clients and stale source ids, then fail late or leave confusing
runtime state. A request to delete a nonexistent account also returns success.

Suggested fix:

1. Move account deletion into an account-deletion service.
2. In one transaction, load the account's linked source ids and reject or cancel
   deletion if any source has an active sync/Takeout/delete operation.
3. Cancel active Takeout jobs and owned LLM analysis work before deleting the
   account, or block deletion until they finish.
4. Check `rows_affected` and return `not_found` for missing accounts.
5. Add backend tests for deleting a missing account and deleting an account with
   active source work.

### 3. Major: Takeout imports leave unqualified partial corpus rows

Evidence:

- `src-tauri/src/takeout_import/mod.rs:852` inserts Takeout messages directly
  into the main `items` table during page processing.
- `docs/takeout-source-import.md:152` documents that failed/cancelled jobs leave
  partial inserted rows and do not advance `last_sync_state`.
- `docs/architecture-deep-dive.md:109` documents the same production behavior.
- `src-tauri/src/takeout_import/mod.rs:660` detects migrated supergroup history,
  and `src-tauri/src/takeout_import/mod.rs:662` warns that migrated history is
  deferred because current item identity can collide.

Impact:

After a failed or cancelled Takeout job, partial rows are immediately available
to analysis and NotebookLM export, but the database does not record which rows
belong to an incomplete import batch. A later retry may skip duplicates, but the
user cannot tell whether the local archive is complete for that historical
range. Migrated supergroup history is known to be omitted because the current
`(source_id, external_id)` identity cannot represent both current supergroup and
migrated small-group messages safely.

Suggested fix:

1. Add an `ingest_batches` table with `source_id`, `kind`, `status`, started and
   finished timestamps, warnings, and error.
2. Add `ingest_batch_id` and `ingest_status` or equivalent provenance to
   `items`, or import into a staging table and promote rows only when the batch
   completes.
3. Exclude incomplete batches from analysis/export by default, with an explicit
   UI override if needed.
4. Decide and implement a stable item identity for migrated histories, such as
   source substream/origin plus Telegram message id, before importing migrated
   small-group history.
5. Add storage tests for cancelled imports, retries, and migrated-id collision
   scenarios.

### 4. Major: The full verification baseline is not green or centralized

Evidence:

- `package.json:10` and `package.json:12` define frontend-only `test` and
  `check` scripts.
- There is no root CI configuration in `.github`.
- `cargo test` passes locally with 141 tests.
- `npm.cmd test` passes with 17 test files and 136 tests when run outside the
  Windows sandbox.
- `npm.cmd run check` passes with 0 errors and 0 warnings when run outside the
  Windows sandbox.
- `cargo clippy --all-targets -- -D warnings` fails with 19 errors, concentrated
  in `src-tauri/src/takeout_import/mod.rs` (`too_many_arguments` and
  `needless_borrow`).

Impact:

The project can pass its usual frontend checks and backend tests while still
having a failing Rust lint baseline. Without a single local command and CI gate,
large refactors can regress backend quality, Tauri wiring, or frontend contracts
without an obvious stop sign.

Suggested fix:

1. Fix the current clippy failures by introducing small Takeout context structs
   and removing needless borrows, or explicitly allow intentional signatures
   with local justification.
2. Add a root verification command or documented script that runs:
   `npm.cmd test`, `npm.cmd run check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`,
   and `git diff --check`.
3. Add CI for the same baseline.
4. Keep live Telegram validation separate from deterministic CI, but record the
   required manual smoke matrix in docs.

### 5. Major: Telegram crates are pinned through a moving git branch

Evidence:

- `src-tauri/Cargo.toml:24`, `src-tauri/Cargo.toml:25`, and
  `src-tauri/Cargo.toml:26` depend on `grammers-*` crates from
  `https://github.com/Lonami/grammers` with `branch = "master"`.
- `src-tauri/Cargo.lock:1682` and related lock entries currently pin one commit,
  but the manifest still points dependency resolution at the moving branch.

Impact:

The project relies on Telegram protocol behavior from an upstream branch that
can change underneath future lockfile refreshes. A routine `cargo update`, a
lockfile regeneration, or a new contributor environment can pick up breaking
API or behavior changes in the app's most important integration surface.

Suggested fix:

1. Pin the `grammers` dependencies with an explicit `rev`, or move to a tagged
   fork/release owned by the project.
2. Document an upgrade procedure that includes `cargo test`, Takeout pagination
   tests, source-resolution tests, and at least one live Telegram smoke pass.
3. Keep upstream upgrade commits small and isolated so behavioral changes are
   reviewable.

## Not Selected As Top Findings

The previous review's maintainability findings remain real, especially the large
`src/routes/analysis/+page.svelte` composition file and remaining raw Tauri
command surfaces. They were not selected as top-five here because the issues
above have a clearer path to data loss, incomplete archives, failed long-running
jobs, or unreproducible releases.

## Verification

- `cargo test`: passed, 141 tests.
- `npm.cmd test`: failed inside the sandbox with `spawn EPERM`; rerun outside the
  sandbox passed, 17 test files and 136 tests.
- `npm.cmd run check`: failed inside the sandbox with `spawn EPERM`; rerun
  outside the sandbox passed with 0 errors and 0 warnings.
- `cargo clippy --all-targets -- -D warnings`: failed with 19 errors in
  `src-tauri/src/takeout_import/mod.rs`.

## Recommended Fix Order

1. Add analysis preflight budgets and hard run limits.
2. Make account deletion coordinate with active ingest and analysis work.
3. Introduce Takeout import batch provenance or staging.
4. Fix clippy and add a full-project verification command plus CI.
5. Pin the `grammers` dependency policy to an explicit revision or owned release.

## Proposed Commit Message

```text
docs(audit): record 2026-05-05 codebase risks
```
