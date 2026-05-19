# Backend Architecture Simplification Analysis

> Date: 2026-05-19
> Scope: Rust/Tauri backend under `src-tauri/src`, local SQLite storage, and
> current architecture docs.

## Executive Summary

The backend does not need a large rewrite. The current direction is sound:
canonical provider data lives in `items` plus typed provider tables, while
provider-neutral derived models serve specific consumers such as analysis and
archive browsing/export.

The largest maintainability wins are small infrastructure and boundary
improvements:

1. centralize repeated backend helpers for time, SQL lists, transactions, and
   error mapping;
2. standardize a small readiness vocabulary for the derived models that are
   actually readiness-gated;
3. keep Tauri command handlers thin and move workflow logic into services;
4. extract small in-memory job helpers before considering any generic job
   runtime;
5. introduce a current-schema baseline for fresh installs after the archive
   read-model boundary settles.

These changes should reduce review cost and future feature friction without
changing product behavior.

## Current Shape

The backend is a local-first Tauri/Rust service layer that owns:

- Telegram account/session management and source sync;
- Telegram Takeout import;
- YouTube `yt-dlp` orchestration;
- SQLite schema, migrations, and data access;
- secure storage for secrets;
- analysis and LLM orchestration;
- NotebookLM export.

The architecture has already moved in the right direction:

- Telegram source identity is typed in `telegram_sources`.
- Telegram message identity is typed in `telegram_messages`.
- YouTube source runtime metadata is typed in `youtube_video_sources` and
  `youtube_playlist_sources`.
- Analysis corpus loading uses `analysis_documents`.
- Source browsing and Telegram NotebookLM export can use
  `archive_read_items` behind source-scoped readiness gates.
- YouTube playlist membership intentionally remains typed detail/list state in
  `youtube_playlist_items`, not archive item rows.

This is a good boundary: canonical provider/archive truth stays typed and
rebuildable read models serve specific consumers.

## Main Maintainability Costs

### Repeated Backend Infrastructure

Several helpers and patterns are duplicated across modules:

- `now_secs`;
- `ymd_to_unix_midnight` and `days_from_civil`;
- `push_i64_list` / `QueryBuilder` list binding helpers;
- manual `BEGIN IMMEDIATE`, `COMMIT`, and `ROLLBACK` blocks;
- mixed `Result<T, String>` and `AppResult<T>` error mapping.

Examples appear in:

- `src-tauri/src/youtube/jobs.rs`
- `src-tauri/src/youtube/detail.rs`
- `src-tauri/src/analysis_documents.rs`
- `src-tauri/src/sources/items.rs`
- `src-tauri/src/ingest_provenance.rs`

This is not conceptually wrong, but it makes storage and workflow changes more
expensive to review because every module carries its own small dialect.

### Multiple Derived Model Lifecycles

The backend now has several derived/materialized models:

- `analysis_documents`;
- `archive_read_model_state` plus `archive_read_items`;
- `telegram_topic_resolution_state` plus `item_topic_memberships`.

These models have related lifecycle ideas:

- canonical data remains elsewhere;
- derived rows can be rebuilt;
- some consumers need to know whether derived state is usable;
- failures should not corrupt canonical data.

They are not identical. `archive_read_model_state` is a direct consumer
readiness gate. `telegram_topic_resolution_state` is close, but its status is
domain-specific topic resolver state. `analysis_documents` is a materialized
corpus model, but it is not currently consumed through the same source-scoped
readiness gate.

The code currently implements these concepts per module. That is acceptable
for the first versions, but future work will be easier if the shared terms are
explicit without forcing one framework over all materialized models.

### Large Mixed-Responsibility Files

Several backend files are large enough that local changes require too much
context:

- `analysis/corpus.rs`
- `takeout_import/mod.rs`
- `sources/items.rs`
- `notebooklm_export/query.rs`
- `analysis/report.rs`
- `youtube/detail.rs`
- `analysis/store.rs`
- `youtube/jobs.rs`
- `youtube/source_metadata.rs`

The issue is not just line count. The expensive pattern is when one file owns
IPC-adjacent DTOs, validation, SQL, workflow orchestration, mapping, and tests
at the same time.

### Thin Boundary Between Commands And Services

Tauri commands mostly work, but command handlers often also own workflow and
storage orchestration. `lib.rs` manually imports and registers many commands,
and domain modules expose command functions directly beside lower-level
helpers.

This makes feature work feel convenient at first, but it increases coupling
between IPC contracts and backend internals.

### Duplicated In-Memory Job State

YouTube source jobs and Telegram Takeout import jobs have similar runtime
state mechanics:

- active job uniqueness;
- queued/running/terminal statuses;
- cancellation;
- update-and-emit;
- finish-and-release.

The records and domain phases differ. Takeout jobs involve ingest locks,
durable provenance, and Telegram session flow; YouTube jobs involve `yt-dlp`
and source-job events. The common mechanics are worth extracting carefully, but
a full generic runtime would be premature.

### Migration History Complexity

The migration layer carries necessary compatibility work:

- runner-managed migrations;
- sentinel SQL files;
- checksum and line-ending repair;
- historical schema cleanup.

This is valid for existing databases, but it is too much history for fresh
installs to conceptually inherit forever. A current-schema baseline is the
right long-term simplification once the read-model boundary is stable.

## Recommended Architecture Changes

### 1. Add Small Backend Infrastructure Modules

Recommended modules:

- `src-tauri/src/time.rs`
- `src-tauri/src/sql_helpers.rs`
- `src-tauri/src/tx.rs`

Initial contents:

- one `now_secs` helper;
- one YYYY-MM-DD to Unix-midnight parser;
- one `push_i64_bind_list` helper for `QueryBuilder`;
- a helper for `BEGIN IMMEDIATE` transaction blocks;
- consistent helpers for database and internal error mapping.

This should be the first slice because it is low risk and makes later work
cheaper.

### 2. Standardize Readiness Vocabulary Carefully

Do not create a large generic framework or a broad trait for every materialized
model. Start with a small common vocabulary and helper functions, and apply
them only where the model is actually readiness-gated:

- `ReadinessStatus`;
- `ModelVersion`;
- `is_ready_current`;
- `mark_stale`;
- `mark_failed`.

The target shape is closer to:

```rust
enum ReadinessStatus {
    NeverBuilt,
    Building,
    Ready,
    Stale,
    Failed,
}

fn is_ready_current(
    status: ReadinessStatus,
    found_version: i64,
    current_version: i64,
) -> bool;
```

`archive_read_model_state` is the clearest fit. Topic resolution can reuse
selected terms where they match, but should keep its domain-specific state
rules. `analysis_documents` should not be forced into this lifecycle unless a
future consumer actually needs readiness-gated corpus rows.

The important architectural rule should stay explicit:

> Canonical data lives in `items` plus typed provider tables. Derived read
> models are rebuildable consumer-facing state and must be readiness-gated
> when stale data would change behavior.

This should cover future archive/read-model work without creating a monster
trait or pretending that every materialized table has the same lifecycle.

### 3. Keep Tauri Commands As Adapters

For new backend work, use this rule:

- command function: IPC shape, state extraction, basic command-level
  validation;
- service function: workflow orchestration and domain decisions;
- store/query function: SQL and row mapping.

Good first candidates:

- YouTube jobs: split command adapters from job workflow execution;
- analysis report start/cancel: keep command contract separate from report
  orchestration;
- NotebookLM export query: separate loader selection/querying from export row
  mapping.

This can be incremental. There is no need to rewrite every command.

### 4. Extract In-Memory Job Helpers Before A Runtime

Start with small reusable helpers for process-local jobs instead of a generic
`JobRuntime<TRecord, TKey>`. Useful first extractions:

- cancellation token / cancellation check helper;
- finish-versus-cancel race helper;
- emit latest record helper;
- active-by-source or active-by-key guard helper;
- terminal-state release helper.

Keep YouTube and Takeout record types separate. If these helpers leave obvious
duplication behind, then a small shared runtime can be considered. Durable or
resumable jobs remain a separate product decision and should not be bundled
into this cleanup.

### 5. Introduce A Current-Schema Baseline

After the archive read-model boundary is considered stable, add a fresh-install
current schema path:

- fresh installs create the current schema directly;
- existing databases still run legacy migrations safely;
- runner-managed migration compatibility remains in a legacy upgrade path;
- checksum and line-ending repair are quarantined away from the normal
  fresh-install story.

This is likely the highest-impact Database Schema Simplification task still
open, but it should be planned carefully because it touches install/upgrade
semantics.

### 6. Continue Migrating Toward Typed Errors

The backend already exposes typed `AppError`. Some analysis and LLM internals
still use `Result<T, String>`.

Recommended path:

- keep pure parser/formatting helpers free to return `String` where useful;
- use `AppResult` for service and storage functions close to Tauri commands;
- remove text-classification dependencies from normal command behavior over
  time.

This will make frontend error behavior more predictable and reduce reliance on
message substring classification.

## What Not To Do

- Do not merge `analysis_documents` and `archive_read_items`. They serve
  different consumers and should remain separate.
- Do not turn `items` into the universal owner of all provider state. Keep
  provider-specific ownership in typed tables.
- Do not materialize YouTube playlist membership rows into
  `archive_read_items`; `youtube_playlist_items` is the right owner.
- Do not introduce a broad ORM/repository abstraction over every SQL query.
  Small SQL helpers and clearer service boundaries should be enough.
- Do not introduce a monster trait for all materialized/read models. Share
  readiness helpers only where the lifecycle really matches.
- Do not start in-memory job cleanup with a fully generic
  `JobRuntime<TRecord, TKey>`. Extract cancellation, active-guard, finish, and
  emit helpers first.
- Do not make YouTube jobs durable as part of the shared job-runtime cleanup.
  Persistence/resume is a product slice, not a refactor prerequisite.

## Suggested Order

1. Add shared time, SQL list, transaction, and error helpers.
2. Standardize derived read-model readiness vocabulary where it genuinely
   applies.
3. Split YouTube job command adapters from service/workflow logic.
4. Extract common in-memory job helpers, then decide whether a shared runtime
   is still useful.
5. Plan and implement current-schema baseline.
6. Gradually convert storage/service APIs from `Result<T, String>` to
   `AppResult<T>`.

## Expected Payoff

These changes should make the backend easier to extend in the places where the
project is actively growing:

- new provider or source-reader work;
- additional archive/export consumers;
- Takeout provenance and migrated-history enablement;
- YouTube playlist detail improvements;
- future current-schema baseline and migration cleanup.

The guiding principle is to preserve the current domain boundaries while
removing repeated backend plumbing around them.
