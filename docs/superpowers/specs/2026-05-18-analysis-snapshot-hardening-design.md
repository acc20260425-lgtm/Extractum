# Analysis Snapshot Hardening Design

Date: 2026-05-18

## Goal

Make saved analysis runs snapshot-first in practice, not only in intent.

New runs should capture their frozen source corpus before any provider call.
Saved-run APIs should expose an explicit snapshot state so downstream code does
not infer provenance from status, message counts, or legacy heuristics.

## Context

The current saved-run model already treats `analysis_run_messages` as the frozen
corpus for report evidence, source browsing, and follow-up chat. The recent
provider-neutral document layer moved live analysis loading to
`analysis_documents`, but saved run snapshots still have two hardening gaps:

- report execution persists `analysis_run_messages` after map/reduce provider
  work, so failed, cancelled, or interrupted runs can lose the exact attempted
  corpus;
- the snapshot schema was extended over time with nullable compatibility fields,
  and one saved-run helper can reconstruct from live source data when no
  snapshot rows exist.

This slice closes those gaps without doing a UI redesign or migration baseline
cleanup.

## Non-Goals

- Do not rebuild `analysis_run_messages` to make older nullable columns
  physically non-null.
- Do not add a full snapshot lifecycle subsystem.
- Do not add durable `capturing` state.
- Do not redesign `/analysis` UI badges, alerts, or empty states.
- Do not move NotebookLM export or source browsing onto a new read model.
- Do not do migration baseline cleanup.

## Approved Approach

Use durable marker columns plus computed DTO state and early snapshot capture.

Add marker columns to `analysis_runs`, keep `analysis_run_messages` as the
snapshot payload table, compute `snapshot_state` in Rust DTO mapping, and move
snapshot persistence to the start of the report pipeline after corpus loading.

Provider prompts, trace building, evidence resolution, source browsing, and
follow-up chat should treat the captured snapshot as authoritative for saved
runs. Live corpus reconstruction must not be a normal fallback for completed
snapshotless runs.

Empty live corpora remain a launch/preflight rejection. A report run should not
be created, captured, or marked as legacy merely because the selected scope has
zero eligible documents.

## Schema

Migration `25.sql` adds:

```sql
ALTER TABLE analysis_runs ADD COLUMN snapshot_captured_at TEXT;
ALTER TABLE analysis_runs ADD COLUMN snapshot_error TEXT;
```

`snapshot_captured_at` records the boundary time when a run's frozen corpus was
successfully persisted, reloaded, and verified as usable.

`snapshot_error` records only capture-preventing failures:

- corpus preload failed before any snapshot could be persisted;
- snapshot insert or verification failed before provider execution.

Provider failures after successful capture must not write `snapshot_error`.
Those failures belong in the existing run `error` field.

`snapshot_error` must be sanitized before storage:

- maximum 512 Unicode scalar values;
- single line, with control characters and CR/LF replaced by spaces;
- no stack traces or debug backtraces;
- no local file paths, app-data paths, or `file://` URLs;
- no URL query strings or fragments, because those can contain tokens;
- no API keys, cookies, bearer tokens, secure-storage keys, or raw provider
  request/response bodies;
- if sanitization cannot confidently preserve a useful safe message, store a
  short generic category such as `Corpus preload failed` or
  `Snapshot capture failed`.

`analysis_run_messages` is not rebuilt in this slice. The Rust writer contract
for new rows becomes strict:

- `ref` is required;
- `content_zstd` is required;
- `item_kind` is required;
- `source_type` is required;
- `source_subtype` is required for source types that define subtypes;
- `metadata_zstd` remains nullable because Telegram documents may not need an
  envelope, while YouTube timestamp and synthetic refs should preserve the
  metadata needed for trace/source resolution.

## DTO Contract

Expose a computed snapshot state on saved-run DTOs:

```ts
type AnalysisSnapshotState =
  | "captured"
  | "missing_legacy"
  | "capture_failed";
```

`AnalysisRunSummary` and `AnalysisRunDetail` should include:

```ts
snapshot_state: AnalysisSnapshotState | null;
snapshot_captured_at: string | null;
snapshot_error: string | null;
```

State rules:

- `captured`: `snapshot_captured_at IS NOT NULL` and `snapshot_error IS NULL`.
  Snapshot row count is not part of this state rule. The non-empty corpus
  invariant is enforced before run creation and by capture verification.
- `missing_legacy`: `snapshot_captured_at IS NULL`, `snapshot_error IS NULL`,
  the run is a completed historical report, and no snapshot rows exist.
- `capture_failed`: `snapshot_error IS NOT NULL`, or a failed/cancelled
  terminal run has no captured marker.
- `null`: active or queued runs that have been created but have not yet reached
  a terminal/captured snapshot classification. In the expected steady state,
  this is only a short backend-only window before capture succeeds or fails.

The DTO state is computed, not stored as an enum. This keeps historical rows
readable while making the current invariant observable and testable.

## Pipeline

The new report execution order is:

```text
resolve scope
load prompt template
preflight
create run
load live corpus from analysis_documents
persist analysis_run_messages
reload captured snapshot from analysis_run_messages
verify captured snapshot is usable
set snapshot_captured_at
build provider input from captured snapshot
run map/reduce provider phases
build trace from captured snapshot
persist result + trace
```

The important boundary is reload-after-write. Once snapshot persistence
succeeds, later provider and trace phases should use the reloaded frozen corpus
instead of the live `corpus` variable produced by the loader.

`snapshot_captured_at` should be written only after rows have been persisted,
reloaded, and verified. If possible, perform insert, reload verification, marker
update, and commit as one capture transaction. A failed reload or verification
must not leave `snapshot_captured_at` populated.

## Failure Semantics

If live corpus loading fails before snapshot capture:

- mark the run `failed`;
- set `snapshot_error` to a bounded, sanitized explanation;
- do not call the provider;
- return `snapshot_state = "capture_failed"`.

If snapshot insert, marker update, or reload-after-write verification fails:

- mark the run `failed`;
- set `snapshot_error`;
- do not call the provider;
- return `snapshot_state = "capture_failed"`.

If provider execution fails after snapshot capture:

- mark the run `failed` through the existing run error path;
- keep `snapshot_error = NULL`;
- keep `snapshot_state = "captured"`.

If cancellation happens after snapshot capture:

- mark the run `cancelled`;
- keep the snapshot available;
- keep `snapshot_state = "captured"`.

Historical completed snapshotless runs remain readable as report artifacts, but
evidence, follow-up chat, and source browsing should not silently resolve
against current live source data.

Failures before `insert_analysis_run` continue to return from
`start_analysis_report` without creating a saved run. Any failure after
`insert_analysis_run` but before successful capture must set `snapshot_error` so
new failed runs are classified as `capture_failed`, not `missing_legacy`.

## Read Path Rules

Saved-run read paths should agree on snapshot provenance:

- `list_analysis_runs` and `get_analysis_run` expose the same computed
  `snapshot_state`;
- `list_analysis_run_messages` remains snapshot-only and never falls back to
  live rows;
- trace/evidence resolution uses snapshot rows when present and returns no
  live reconstruction for completed snapshotless runs;
- follow-up chat requires a saved snapshot for completed runs;
- `load_run_corpus_messages` must not use live source reconstruction as a
  normal saved-run fallback.

The old YouTube `TranscriptDescription` default fallback should be removed from
normal saved-run flow or constrained to an explicitly legacy/test-contained
helper that new saved-run behavior does not call.

## Frontend Boundary

No new UI workflow is part of this slice.

Frontend work is limited to type/API compatibility:

- add `snapshot_state`, `snapshot_captured_at`, and `snapshot_error` to analysis
  run TypeScript types;
- keep existing degraded/error rendering behavior;
- avoid adding new badges, alerts, or layout changes unless required to keep
  existing states safe.

## Testing

Implementation should add focused tests before changing behavior:

- migration registration and fresh schema contain `snapshot_captured_at` and
  `snapshot_error`;
- DTO mapping returns `captured` when the captured marker exists without
  `snapshot_error`;
- DTO mapping returns `missing_legacy` only for completed historical rows without
  marker, error, or snapshot rows;
- DTO mapping returns `capture_failed` when `snapshot_error` exists;
- DTO mapping returns `capture_failed` for failed/cancelled terminal rows
  without a captured marker;
- DTO mapping returns `null` for active created runs before capture completes;
- empty corpus preflight rejects before creating a saved run;
- stored snapshot errors are bounded and sanitized;
- report pipeline persists snapshot before provider execution;
- `snapshot_captured_at` is set only after reload-after-write verification
  succeeds;
- provider is not called when corpus preload or snapshot persistence fails;
- provider failure after capture leaves `snapshot_state = "captured"` and
  `snapshot_error = NULL`;
- cancellation after capture leaves `snapshot_state = "captured"`;
- saved-run corpus/evidence/chat paths do not reconstruct from live source rows
  for completed snapshotless runs;
- YouTube saved-run fallback no longer defaults normal saved-run reconstruction
  to `TranscriptDescription`.

Verification should include targeted Rust tests for analysis store/corpus/report
behavior, migration tests, relevant TypeScript type/API tests, full
`cargo test --manifest-path src-tauri/Cargo.toml`, frontend check/tests if
frontend types change, and `git diff --check`.

## Acceptance Criteria

- New report runs capture `analysis_run_messages` before any provider request.
- Provider and trace phases use the captured snapshot after reload-after-write.
- `snapshot_captured_at` means the frozen snapshot was persisted, reloaded, and
  verified as usable.
- Empty corpus selections are rejected before saved run creation.
- Snapshot capture failures stop before provider execution and expose
  `capture_failed`.
- Provider failures after successful capture still expose `captured`.
- Saved-run DTOs expose explicit snapshot state and marker fields.
- Historical snapshotless runs are classified as `missing_legacy`.
- Completed snapshotless runs remain readable as reports without live-source
  evidence/chat/source reconstruction.
- The slice does not include UI redesign, NotebookLM migration, playlist
  simplification, or migration baseline cleanup.

## Implementation Plan Boundary

The next implementation plan should be named
`docs/superpowers/plans/2026-05-18-analysis-snapshot-hardening.md` and should
break the work into small TDD tasks:

1. Schema migration and DTO state mapping.
2. Strict new snapshot writer contract.
3. Early capture and reload-after-write pipeline.
4. Failure semantics for preload, snapshot write, provider failure, and cancel.
5. Saved-run read path fallback removal.
6. Frontend type/API compatibility.
7. Verification and documentation.
