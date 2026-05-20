# Backend Architecture Simplification Analysis

> Date: 2026-05-19
> Scope: Rust/Tauri backend under `src-tauri/src`, local SQLite storage, and
> current architecture docs.

## Executive Summary

The backend does not need a large rewrite. The current direction is sound:
canonical provider data lives in `items` plus typed provider tables, while
provider-neutral derived models serve specific consumers such as analysis and
archive browsing/export.

The remaining maintainability wins are focused on database lifecycle and error
boundaries:

1. introduce a current-schema baseline for fresh installs after the archive
   read-model boundary settles;
2. continue migrating service and storage APIs toward typed errors.

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

### 1. Introduce A Current-Schema Baseline

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

### 2. Continue Migrating Toward Typed Errors

The backend already exposes typed `AppError`. Some analysis and LLM internals
still use `Result<T, String>`.

Current status:

- analysis report store helpers, live corpus/preflight loaders, trace helpers,
  and saved-run snapshot readers now return `AppResult` for database,
  not-found, validation, and internal snapshot/content/trace errors;
- remaining analysis string errors are limited to test-only snapshot source
  resolution and pure parser helpers; shared compression helpers still return
  `String` at the low-level utility boundary;
- LLM provider and scheduler internals still use string errors across provider
  calls and streaming boundaries.

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

## Suggested Order

1. Plan and implement current-schema baseline.
2. Gradually convert storage/service APIs from `Result<T, String>` to
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
