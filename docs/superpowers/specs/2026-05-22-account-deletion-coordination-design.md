# Account Deletion Coordination Design

## Goal

Make `delete_account(account_id)` safe around active work by blocking deletion
with a typed `conflict` when any running or queued work depends on sources
owned by that account.

## Policy

`delete_account(account_id)` must:

1. run a read-only preflight before deleting anything;
2. never auto-cancel source, analysis, or LLM work;
3. return typed `not_found` if the account row does not exist;
4. return typed `conflict` if active work depends on sources owned by the
   account;
5. delete the account row and then clear Telegram runtime, session data, and
   API-hash secrets only after the preflight passes.

The user must explicitly stop, cancel, or wait for active work before deleting
the account.

## Concurrency Boundary

This slice blocks deletion when related work is already active at preflight
time. It does not introduce a new account-deletion semaphore that prevents new
work from starting in the narrow window between a clean preflight and the first
delete mutation.

That race is an accepted boundary for this desktop, single-user slice. The
preflight still removes the current high-risk behavior: deleting an account
while known active sync/import/source-job/analysis/LLM work is already running.
Existing operation start paths still validate source existence and use their
own source/job guards. A stronger start-after-preflight exclusion mechanism can
be added later if the app needs multi-actor or cross-window hard serialization.

## Blocking Work

The preflight loads owned source ids from `sources.account_id = account_id`.
Unrelated active work must not block deletion.

Deletion is blocked when any of these are true:

- an owned source id has an active `SourceIngestLocks` guard for sync, Takeout
  import, or source deletion;
- `TakeoutImportState` has a non-terminal job for an owned source id;
- `SourceJobState` has a non-terminal source job whose `source_id` or
  `related_source_id` is in the owned-source set;
- `AnalysisState` has an active analysis run whose direct `source_id` is owned
  by the account;
- `AnalysisState` has an active analysis run whose `source_group_id` contains
  at least one owned source through `analysis_source_group_members`;
- `LlmSchedulerState` has an active request whose `owner_run_id` belongs to a
  relevant active analysis run.

Provider smoke tests, model/settings checks, and other standalone LLM requests
with `owner_run_id = None` do not block account deletion. Active analysis or
source jobs for sources not owned by the account do not block deletion.

An active source deletion guard is still a blocker even though it is also a
delete-oriented operation. Account deletion must not compete with a source-level
delete that may already be cascading rows, releasing locks, or reporting errors
for that source. The user can retry account deletion after the source deletion
finishes.

## LLM Ownership

Report-generation LLM requests already use `owner_run_id = Some(run_id)` and
must keep doing so.

Analysis follow-up chat requests must also use `owner_run_id = Some(run_id)`.
This lets account deletion recognize that a live follow-up answer is reading a
saved-run context or writing chat output tied to sources owned by the account.

Provider smoke tests and unrelated LLM checks keep `owner_run_id = None`.

## Implementation Shape

Add a focused Rust module, `src-tauri/src/account_deletion.rs`, instead of
expanding `accounts.rs`.

The module exposes a read-only preflight helper with a shape like:

```rust
pub(crate) struct AccountDeletionPlan {
    pub(crate) account_id: i64,
    pub(crate) owned_source_ids: Vec<i64>,
}

pub(crate) async fn check_account_deletion(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    account_id: i64,
    source_locks: &SourceIngestLocks,
    takeout_state: &TakeoutImportState,
    source_job_state: &SourceJobState,
    analysis_state: &AnalysisState,
    llm_scheduler: &LlmSchedulerState,
) -> AppResult<AccountDeletionPlan>;
```

The helper may maintain an internal structured enum for tests and diagnostics,
and it should collect all blocking categories it can observe before returning a
conflict. The public error must stay sanitized and generic. Do not include
source titles, usernames, prompt text, message text, provider payloads, phone
numbers, session data, or secrets.

Use this conflict message:

```text
Cannot delete account while source sync, import, source job, or analysis work is running for its sources. Stop or wait for the active work, then try again.
```

`accounts::delete_account` calls the preflight before any mutation. If the
preflight returns `conflict`, account rows, sources, runtime state, session
data, and secrets remain unchanged.

After a clean preflight, deletion keeps the current destructive ordering:

1. delete the account row from SQLite, allowing database cascades to remove
   owned sources and dependent rows;
2. clear Telegram runtime/session data for that account;
3. delete the API-hash secret.

There is no atomic transaction spanning SQLite, runtime memory, session files,
and the OS secret store. If post-row cleanup fails, `delete_account` returns the
typed cleanup error after the database row has already been deleted. It must not
silently report success, and it must not recreate the account row. This keeps
the database as the source of truth while making the cleanup failure visible for
manual retry or later orphan-cleanup tooling.

## State API Additions

Small read-only helpers are needed so the preflight can inspect active work
without duplicating state internals:

- `SourceIngestLocks`: report whether any source id in a set is currently
  locked, with the active kind retained internally for tests.
- `TakeoutImportState`: list active source ids or active jobs for a source-id
  set.
- `SourceJobState`: list active jobs whose `source_id` or `related_source_id`
  is in a source-id set.
- `AnalysisState`: expose active report run ids, which already exists
  internally and can stay crate-private.
- `LlmSchedulerState`: report active snapshots by `owner_run_id`, reusing the
  existing request snapshot machinery or a small owner-id helper.

These helpers must be read-only and must not cancel, finish, or release work.

## Tests

Use TDD for implementation. Add focused Rust tests around the preflight and
small state helpers.

Required cases:

- missing account returns typed `not_found`;
- account exists but owns zero sources passes preflight and deletion still
  attempts runtime/session/secret cleanup;
- active `SourceIngestLocks` guard on an owned source blocks deletion;
- active Takeout job on an owned source blocks deletion;
- active generic source job on an owned source blocks deletion;
- active source job on a non-owned source does not block deletion;
- active direct-source analysis run for an owned source blocks deletion;
- active source-group analysis run blocks when any group member source is owned,
  even when the run itself has no direct `source_id`;
- active analysis run for another account/source does not block deletion;
- active LLM scheduler request blocks only when `owner_run_id` is in the
  relevant active run set;
- provider smoke test or model/settings request with `owner_run_id = None` does
  not block deletion;
- preflight conflict does not delete the account row, sources, runtime state, or
  secrets;
- preflight records multiple internal blocking categories when more than one
  related active-work type is present, while the returned app error uses the
  sanitized generic conflict message;
- post-row cleanup failure returns an error without resurrecting the deleted
  account row;
- analysis follow-up chat requests register
  `LlmRequestKind::AnalysisChat` with `owner_run_id = Some(run_id)`.

Existing frontend delete-account API shape can remain unchanged unless tests
show the UI needs to surface the typed conflict differently.

## Non-Goals

- Do not add auto-cancel behavior for analysis, LLM, Takeout, source jobs, or
  source ingest.
- Do not change saved-run retention semantics.
- Do not broaden deletion to remove unrelated provider settings or LLM profile
  state.
- Do not add private titles, usernames, prompt text, message text, or provider
  payloads to errors or tracked docs.
