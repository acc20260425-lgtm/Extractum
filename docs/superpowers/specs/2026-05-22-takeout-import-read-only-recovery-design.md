# Takeout Import Read-Only Recovery Design

## Goal

Surface durable incomplete or interrupted Telegram Takeout attempts as
read-only source recovery state so users can understand partial local history
after restart, failure, cancellation, or partial completion.

## Policy

This slice explains what happened. It does not repair, delete, resume, or retry
anything automatically.

The safe recovery path presented to users is:

```text
Run Takeout again to continue collecting available history. Messages already
saved locally will be deduplicated.
```

The existing Takeout start flow remains the only action. The recovery state is
not a job and cannot be cancelled, resumed, finished, purged, or retried through
new recovery commands.

## Backend Contract

Add a separate read-only DTO instead of reusing or faking
`TakeoutImportJobRecord`.

Suggested Rust shape:

```rust
pub(crate) struct TakeoutImportRecoveryState {
    pub(crate) batch_id: i64,
    pub(crate) source_id: i64,
    pub(crate) status: String,
    pub(crate) recovery_kind: String,
    pub(crate) completeness: String,
    pub(crate) item_inserted_count: i64,
    pub(crate) item_duplicate_count: i64,
    pub(crate) item_skipped_count: i64,
    pub(crate) item_observed_count: i64,
    pub(crate) warning_count: i64,
    pub(crate) warning_codes: Vec<String>,
    pub(crate) terminal_error: Option<String>,
    pub(crate) started_at: i64,
    pub(crate) finished_at: Option<i64>,
    pub(crate) updated_at: i64,
}
```

`status` is the durable `ingest_batches.status` value:

- `running`
- `failed`
- `cancelled`
- `completed`

`recovery_kind` is derived for display:

- `interrupted`
- `failed`
- `cancelled`
- `partial_completed`

Do not add `interrupted` to the database enum. A durable `running` batch becomes
`interrupted` only when no active in-memory Takeout job exists for that source.

Expose a Tauri command:

```rust
list_takeout_import_recovery_states() -> Vec<TakeoutImportRecoveryState>
```

The public command returns states for all sources. Internally, keep the query
helper source-scoped so future callers can pass an optional source-id filter:

```rust
list_takeout_import_recovery_states_for_sources(source_ids: Option<&[i64]>)
```

The command must have access to `TakeoutImportState` so it can suppress durable
`running` batches that still have an active in-memory job.

Only batches with a matching `telegram_takeout_batches` row participate in
Takeout recovery. Future generic `ingest_batches` rows for sync, YouTube, or
other providers must not affect this query.

The schema already has `ingest_batches.updated_at`. Emit `started_at`,
`finished_at`, and `updated_at` as Unix seconds in the DTO by converting the
durable SQLite timestamps in the query. Do not add a schema migration only to
support this read-only slice.

The public command intentionally returns all source recovery states. The
analysis page already loads the source catalog and job maps globally, and the
result is one small row per source at most. Do not add account filtering in this
slice.

## Latest Attempt Wins

Recovery state is derived from the latest Takeout batch for each source, not
from the latest matching incomplete batch.

Use a deterministic ordering such as:

```sql
ORDER BY started_at DESC, id DESC
```

or `id DESC` as the tie-breaker when selecting the latest batch per source.

SQLite has no `DISTINCT ON`. Use a deterministic SQLite shape such as a window
function:

```sql
WITH latest_takeout AS (
  SELECT
    b.*,
    ROW_NUMBER() OVER (
      PARTITION BY b.source_id
      ORDER BY b.started_at DESC, b.id DESC
    ) AS row_number
  FROM ingest_batches b
  JOIN telegram_takeout_batches t ON t.batch_id = b.id
  WHERE b.provider = 'telegram'
    AND b.ingest_kind = 'takeout'
)
SELECT ...
FROM latest_takeout
WHERE row_number = 1
```

An equivalent correlated subquery is acceptable if it preserves the same
ordering and returns exactly one latest Takeout batch per source. Do not use
`GROUP BY source_id` while selecting arbitrary non-aggregated columns.

Classification rules:

1. Latest batch is `running` and no active in-memory Takeout job exists for the
   source: return `recovery_kind = "interrupted"`.
2. Latest batch is `running` and an active in-memory Takeout job exists for the
   source: return no recovery state.
3. Latest batch is `failed`: return `recovery_kind = "failed"`.
4. Latest batch is `cancelled`: return `recovery_kind = "cancelled"`.
5. Latest batch is `completed` and `completeness = "partial"`: return
   `recovery_kind = "partial_completed"`.
6. Latest batch is `completed` and `completeness = "complete"`: return no
   recovery state.

Older failed or cancelled attempts are hidden if a newer complete attempt
exists. A newer failed attempt is still shown even if an older complete attempt
exists.

If the completeness enum expands later, use a helper such as
`is_incomplete_completeness(completeness)` rather than spreading string checks.
For the current enum, `partial` is the only incomplete completed state.

For `interrupted`, durable `completeness` will normally still be `unknown`
because `finalize_ingest_batch` has not run. The classification does not depend
on completeness for running batches. Return the durable completeness value in
the DTO for diagnostics, but do not use it to decide whether a running batch is
interrupted.

To avoid transient false `interrupted` states while a live job is finishing, the
Takeout runtime should keep the in-memory job active until durable terminal
finalization has been attempted. The success path already finalizes the batch
before emitting the completed job. Failed and cancelled paths should likewise
attempt durable finalization before releasing the in-memory job. If durable
finalization fails and the batch remains `running` after job release, the
read-only recovery state may show `interrupted`, which accurately reflects that
durable provenance was not terminalized.

## Privacy And Sanitization

`warning_codes` contains warning code strings only. Do not expose warning
messages or raw Telegram payloads in this DTO.

Return warning codes as a unique, lexicographically sorted list. This keeps UI
and snapshot tests deterministic without implying warning emission order.

`terminal_error` may be returned from the durable batch row because it is already
sanitized and bounded by ingest provenance finalization. For defense in depth,
populate `terminal_error` only when `recovery_kind = "failed"`; return `None`
for `interrupted`, `cancelled`, and `partial_completed`.

Do not include source titles, usernames, phone numbers, session data, API data,
message text, prompt text, provider payloads, or warning message bodies in the
recovery DTO.

## Frontend State

Add TypeScript types for `TakeoutImportRecoveryState` and
`TakeoutImportRecoveryKind`, plus an API wrapper:

```ts
listTakeoutImportRecoveryStates(): Promise<TakeoutImportRecoveryState[]>
```

Map recovery state by source id in `analysis-state.ts`.

Use one helper for visibility:

```ts
visibleTakeoutRecoveryForSource(
  sourceId,
  takeoutJobsBySource,
  recoveryBySource,
) =
  activeTakeoutJobBySource[sourceId] ? null : recoveryBySource[sourceId] ?? null
```

The same label, copy, facts, and severity helpers should feed both UI surfaces
so the source switcher and selected-source notice do not drift.

Refresh recovery states:

- on analysis page mount, alongside source catalog and active Takeout jobs;
- after a Takeout terminal event, alongside the existing source catalog/items
  reload triggered by completion;
- after starting a new Takeout job, so stale recovery notice disappears once the
  active job becomes visible;
- after source deletion or source catalog reload paths that can remove sources.

No polling is required in this slice. Runtime Takeout events and explicit page
loads are enough for the current desktop UI.

Use display severity helpers rather than encoding severity in the DTO:

- `failed`: `danger`
- `interrupted`: `warning`
- `cancelled`: `neutral`
- `partial_completed`: `warning`

## UI Surfaces

Show recovery state in two read-only places.

1. Source switcher row: under the existing Takeout status area for each source.
   This makes incomplete history discoverable before the source is selected.
2. Selected source surface: above the source timeline or topic selector for a
   selected Telegram source. This explains partial source material while the
   user is browsing it.

Priority:

```text
active in-memory job -> show active job UI
else recovery state -> show recovery notice
else no Takeout notice
```

Do not add a new recovery button. The existing Takeout button remains available
when normal source action rules allow starting another import.

Suggested labels:

- `interrupted`: `Previous Takeout import was interrupted`
- `failed`: `Previous Takeout import failed`
- `cancelled`: `Previous Takeout import was cancelled`
- `partial_completed`: `Previous Takeout import completed with partial history`

Suggested body:

```text
Run Takeout again to continue collecting available history. Messages already
saved locally will be deduplicated.
```

Show compact facts for nonzero counts:

- inserted
- duplicates
- skipped
- observed
- warnings

If all counts are zero, still show the notice, but replace the facts row with:

```text
No items were written in this attempt.
```

For `failed`, show sanitized terminal error detail. For `interrupted`,
`cancelled`, and `partial_completed`, do not render terminal-error text.

Show warning codes, not warning messages.

## Documentation

Update `docs/takeout-source-import.md` to document:

- read-only recovery state;
- latest-attempt-wins classification;
- `interrupted` as a derived state from durable `running` plus no active job;
- active in-memory jobs taking priority over durable recovery display;
- no resume, purge, auto-retry, or migrated-history enablement in this slice.

Update `docs/backlog.md` to keep incomplete-import recovery open for richer
future actions, using wording like:

```markdown
- [ ] define richer incomplete-import recovery actions and user policy beyond
  the shipped read-only recovery state
```

Keep the real-data validation items separate and open.

## Tests

Use TDD for implementation.

Backend tests:

- non-Takeout `ingest_batches` rows without a `telegram_takeout_batches` row do
  not participate in recovery selection;
- latest selection returns exactly one row per source and does not rely on
  arbitrary `GROUP BY` column values;
- latest completed `complete` batch returns no recovery state;
- latest completed `partial` batch returns `partial_completed`;
- latest complete batch hides older failed or cancelled attempts;
- latest failed batch returns recovery state even when an older complete batch
  exists;
- latest cancelled batch returns recovery state;
- latest running batch with no active in-memory job returns `interrupted`;
- latest running batch with active in-memory job returns no recovery state;
- interrupted classification ignores durable completeness for running batches;
- terminal job paths attempt durable finalization before releasing active
  in-memory job state;
- warning messages are not exposed;
- warning codes are unique and sorted lexicographically for the latest visible
  recovery batch;
- terminal error is sanitized and bounded before it reaches the DTO.
- terminal error is populated only for failed recovery states.

Frontend tests:

- API wrapper calls `list_takeout_import_recovery_states`;
- recovery states map by `source_id`;
- visible recovery helper suppresses recovery when an active Takeout job exists;
- labels and support copy are correct for all four recovery kinds;
- source switcher row and selected source surface use the same helper or shared
  component;
- `terminal_error` renders only for failed recovery;
- zero-count attempts render `No items were written in this attempt.`;
- active Takeout job UI still wins over durable recovery state.
- recovery severity maps failed to `danger`, partial completed and interrupted
  to `warning`, and cancelled to `neutral`.
- recovery states load on page mount and refresh after Takeout terminal events
  without adding polling.

## Non-Goals

- No resume.
- No purge or discard of partial rows.
- No automatic retry.
- No new retry/recovery action button.
- No mutation of old batch state from recovery query or UI. Existing active
  Takeout jobs still terminalize their own batch as part of normal job
  completion.
- No pseudo-jobs.
- No full provenance history view.
- No migrated-history enablement.
