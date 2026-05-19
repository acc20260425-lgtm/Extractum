# Backend Architecture Simplification Analysis

> Date: 2026-05-19
> Scope: Rust/Tauri backend under `src-tauri/src`, local SQLite storage, and
> current architecture docs.

## Executive Summary

The backend does not need a large rewrite. The current direction is sound:
canonical provider data lives in `items` plus typed provider tables, while
provider-neutral derived models serve specific consumers such as analysis and
archive browsing/export.

The remaining maintainability wins are boundary and lifecycle improvements:

1. keep remaining Tauri command handlers thin and move workflow logic into
   services;
2. extract small in-memory job helpers before considering any generic job
   runtime;
3. introduce a current-schema baseline for fresh installs after the archive
   read-model boundary settles;
4. continue migrating service and storage APIs toward typed errors.

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

### Large Mixed-Responsibility Files

Several backend files are large enough that local changes require too much
context:

- `analysis/corpus.rs`
- `takeout_import/mod.rs`
- `sources/items.rs`
- `notebooklm_export/query.rs`
- `youtube/detail.rs`
- `analysis/store.rs`
- `youtube/source_metadata.rs`

The issue is not just line count. The expensive pattern is when one file owns
IPC-adjacent DTOs, validation, SQL, workflow orchestration, mapping, and tests
at the same time.

### Remaining Thin Boundary Between Commands And Services

Tauri commands mostly work, but some command handlers still own workflow or
storage orchestration. `lib.rs` manually imports and registers many commands,
and some domain modules still expose command functions directly beside
lower-level helpers.

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

### 1. Keep Tauri Commands As Adapters

For new backend work, use this rule:

- command function: IPC shape, state extraction, basic command-level
  validation;
- service function: workflow orchestration and domain decisions;
- store/query function: SQL and row mapping.

Remaining candidate:

- NotebookLM export query: separate loader selection/querying from export row
  mapping.

This can be incremental. There is no need to rewrite every command.

### 2. Extract In-Memory Job Helpers Before A Runtime

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

### 3. Introduce A Current-Schema Baseline

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

### 4. Continue Migrating Toward Typed Errors

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
  Clearer service boundaries should be enough.
- Do not start in-memory job cleanup with a fully generic
  `JobRuntime<TRecord, TKey>`. Extract cancellation, active-guard, finish, and
  emit helpers first.
- Do not make YouTube jobs durable as part of the shared job-runtime cleanup.
  Persistence/resume is a product slice, not a refactor prerequisite.

## Suggested Order

1. Finish the remaining adapter/service split for NotebookLM export query.
2. Extract common in-memory job helpers, then decide whether a shared runtime
   is still useful.
3. Plan and implement current-schema baseline.
4. Gradually convert storage/service APIs from `Result<T, String>` to
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
tightening the remaining service boundaries around them.
