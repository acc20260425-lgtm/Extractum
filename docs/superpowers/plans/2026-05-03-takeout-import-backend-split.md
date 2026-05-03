# Takeout Import Backend Split

## Status

Completed and merged into `main` on 2026-05-03.

Final commit:

```text
013ecc0 refactor(takeout): split import state pagination and export dc
```

## Outcome

The Takeout import backend was split into focused Rust modules without changing user-visible
behavior. `src-tauri/src/takeout_import/mod.rs` remains the command and orchestration facade.

Final module structure:

- `src-tauri/src/takeout_import/mod.rs`
- `src-tauri/src/takeout_import/state.rs`
- `src-tauri/src/takeout_import/pagination.rs`
- `src-tauri/src/takeout_import/export_dc.rs`
- `src-tauri/src/takeout_import/raw_parse.rs`

The old single-file backend `src-tauri/src/takeout_import.rs` was removed.

## Preserved Contracts

- Tauri commands:
  - `start_takeout_source_import`
  - `cancel_takeout_source_import`
  - `list_takeout_source_import_jobs`
  - `run_takeout_export_dc_spike`
- Event name: `sources://takeout-import`
- Response DTO shapes:
  - `StartTakeoutImportResponse`
  - `CancelTakeoutImportResponse`
  - `TakeoutImportJobRecord`
  - `TakeoutExportDcSpikeResult`
- Status and phase string values.
- Pagination behavior, including `TAKEOUT_HISTORY_PAGE_LIMIT = 100` and fallback warning text.
- Export-DC behavior, including `export_dc_id = home_dc_id + 4 * 10000`.
- Cancellation behavior and same-source lock cleanup.
- Public import surface in `src-tauri/src/lib.rs`:

```rust
use takeout_import::{
    cancel_takeout_source_import, list_takeout_source_import_jobs, run_takeout_export_dc_spike,
    start_takeout_source_import, TakeoutImportState,
};
```

## Module Ownership

`state.rs` owns job DTOs, job maps, active source tracking, cancellation tracking, event emission,
terminal status handling, and state-focused tests.

`pagination.rs` owns pure Takeout pagination types, cursor advancement, split selection, page
parsing, fallback warning generation, message range helpers, and pagination-focused tests.

`export_dc.rs` owns export-DC aliasing, Takeout init request construction, export-DC invocation
fallback handling, session finishing, and export-DC-focused tests.

`mod.rs` intentionally keeps command entrypoints, source loading, account runtime flow, peer
validation, history probes, history import orchestration, warning accumulation, and calls into
`raw_parse::parse_raw_message`.

## Verification

Final verification after merge into `main`:

```powershell
Set-Location src-tauri
cargo test
Set-Location ..
npm.cmd test
npm.cmd run check
git diff --check
```

Results:

- `cargo test`: 130 passed.
- `npm.cmd test`: 10 test files and 97 tests passed.
- `npm.cmd run check`: 0 errors and 0 warnings.
- `git diff --check`: passed with no output.

## Follow-Up

`src-tauri/src/sources.rs` remains the next backend split target. Split it only along behavior
boundaries already covered by tests.
