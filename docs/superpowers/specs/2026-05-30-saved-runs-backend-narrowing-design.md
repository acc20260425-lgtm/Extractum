# Saved Runs Backend Narrowing Design

Status: approved design, pending implementation plan

## Summary

Make the existing `/analysis` Runs companion filters narrow saved runs in the
backend before `ORDER BY created_at DESC LIMIT ?` is applied. The current UI
already exposes search, current/all scope, status, date, provider, model, and
template filters, but saved runs are loaded as a small recent slice and then
filtered in the frontend. For large histories this can hide matching older
runs.

The slice keeps the current Runs tab layout and makes filtering truthful across
the saved-run history.

## Goals

- Search and filters should apply to all saved runs that match the requested
  criteria, not only to the first loaded page.
- Preserve the current Runs tab controls and active-run behavior.
- Keep active runs client-filtered because they are already loaded from a small
  active-state list.
- Keep `runsFilter` as the canonical frontend filter state.
- Avoid a larger redesign, pagination model, or cleanup workflow in this slice.

## Current State

Frontend:

- `RunCompanionRunsTab` already renders the controls:
  - text search;
  - `Current scope` / `All runs`;
  - status segmented buttons;
  - advanced date/provider/model/template filters.
- `filterCompanionRuns` merges active and saved runs, filters them locally, and
  sorts by `created_at`.
- `analysis-run-workflow` calls `listRuns({ sourceId, sourceGroupId, limit: 50 })`.

Backend:

- `list_analysis_runs` accepts `source_id`, `source_group_id`, and `limit`.
- It has three near-identical SQL branches for single-source, source-group, and
  global history.
- The query applies only scope filters before `ORDER BY runs.created_at DESC`
  and `LIMIT ?`.

## Product Contract

When a user changes saved-run filters, saved-run results should be reloaded from
the backend using those filters. A matching older run should appear even when it
would not be in the newest unfiltered `limit` rows. Because pagination is out of
scope, this guarantee means the run appears when it is within the top `limit`
rows after backend filters are applied.

The visible Runs tab behavior stays familiar:

- active and saved entries still appear in one list;
- active runs still update from `list_active_analysis_runs`;
- saved runs still exclude queued/running statuses after loading as a safety
  guard;
- when the status filter is `queued_running`, the visible list is expected to be
  driven mainly by active runs; any queued/running rows returned by saved-run
  history may still be removed by the existing client safety guard;
- clearing filters resets `runsFilter` to defaults;
- current scope keeps using the selected source or source group;
- all scope searches across saved runs regardless of current workspace.

## Backend API

Extend `ListAnalysisRunsInput` and the Tauri command arguments with optional
filter fields:

```ts
interface ListAnalysisRunsInput {
  sourceId: number | null;
  sourceGroupId: number | null;
  limit: number;
  query?: string;
  status?: "all" | "completed" | "failed" | "cancelled" | "queued_running";
  provider?: string;
  model?: string;
  template?: string;
  dateFrom?: string;
  dateTo?: string;
}
```

Command argument names continue to use the existing camelCase frontend shape and
Tauri snake_case Rust parameter mapping.

`run_type` remains implicitly report-oriented because the existing analysis run
list is report-run history for this workspace. This slice does not add a
run-type filter.

## Filter Semantics

Scope:

- `sourceId` and `sourceGroupId` remain mutually exclusive.
- Passing both non-null is a typed validation error, not silent precedence.
- `sourceId` filters `runs.source_id`.
- `sourceGroupId` filters `runs.source_group_id`.
- both null means global saved-run history.

Status:

- `"all"` or empty status adds no status predicate.
- `"queued_running"` means `runs.status IN ('queued', 'running')`.
- other status values match `runs.status = ?`.
- The frontend still removes active statuses from the saved-run list after
  loading, so queued/running saved rows are harmless if present.

Date:

- The frontend sends date input values as `YYYY-MM-DD`.
- The backend parses only `YYYY-MM-DD`.
- `dateFrom` is inclusive UTC start of day.
- `dateTo` is inclusive UTC end of day.
- Invalid or empty date strings are ignored rather than causing a failed load.
  They may be debug-noted if a local logging pattern already exists, but they
  should not surface as user-facing load failures.

Text fields:

- `provider`, `model`, and `template` are case-insensitive contains filters.
- `template` means template name contains and matches `templates.name`.
  This slice does not filter by `runs.prompt_template_id`.
- `query` is split into whitespace terms after trimming/lowercasing.
- Every query term must match at least one of:
  - `runs.scope_label_snapshot`;
  - `sources.title`;
  - `groups.name`;
  - `templates.name`;
  - `runs.provider_profile`;
  - `runs.provider`;
  - `runs.model`;
  - `runs.error`.

This mirrors the current frontend search text as closely as practical.
Search input is treated as literal contains text: `%`, `_`, and the SQL escape
character must be escaped before building `LIKE` patterns.

Provider, model, and template filters remain free-text inputs for this slice.
The implementation does not add globally complete provider/model/template option
discovery or a facets endpoint.

## SQL Shape

Replace the three duplicated SQL branches with one query builder that appends
optional predicates and bind values in order. The select list and joins should
stay the same as the current `AnalysisRunRow` query.

The final query shape is:

```sql
SELECT ...
FROM analysis_runs runs
LEFT JOIN sources ...
LEFT JOIN analysis_source_groups ...
LEFT JOIN analysis_prompt_templates ...
LEFT JOIN snapshot_counts ...
WHERE 1 = 1
  -- optional predicates
ORDER BY runs.created_at DESC
LIMIT ?
```

Query terms are expressed as `AND` across terms and `OR` across fields. For
example, each term appends a predicate shaped like:

```sql
AND (
  lower(coalesce(runs.scope_label_snapshot, '')) LIKE ? ESCAPE '\'
  OR lower(coalesce(sources.title, '')) LIKE ? ESCAPE '\'
  OR lower(coalesce(groups.name, '')) LIKE ? ESCAPE '\'
  OR lower(coalesce(templates.name, '')) LIKE ? ESCAPE '\'
  OR lower(coalesce(runs.provider_profile, '')) LIKE ? ESCAPE '\'
  OR lower(coalesce(runs.provider, '')) LIKE ? ESCAPE '\'
  OR lower(coalesce(runs.model, '')) LIKE ? ESCAPE '\'
  OR lower(coalesce(runs.error, '')) LIKE ? ESCAPE '\'
)
```

Use structured SQL helpers where the codebase already has them. Avoid string
concatenation of user input; dynamic SQL should only concatenate fixed predicate
fragments and bind all user values.

## Frontend Wiring

`runsFilter` becomes the canonical state for saved-run filtering. The older
`runFilter` field can remain for persistence compatibility and legacy state
parsing, but new saved-run loading should read from `runsFilter`.

`changeRunsFilter(next)` should:

- update `runsFilter`;
- keep `historyScope = next.scope` for existing route behavior;
- trigger a saved-run reload through the reactive load effect.

`analysis-run-workflow` should receive the current `runsFilter` or a
backend-filter projection of it when calling `listRuns`.

To avoid a backend request per keystroke, text-like filter changes should be
debounced before reloading saved runs. Scope/status/date changes may reload
immediately if that matches the simpler route-state implementation.

Saved-run reloads also need a stale-response guard. Increment a request sequence
or token before invoking the backend, capture it for the request, and apply the
response only if it is still the latest saved-run load. This prevents slower
older search results from overwriting newer filter results.

## Client Filtering

Keep `filterCompanionRuns` for the combined list because active runs still need
local filtering and sorting with saved runs. After this slice, saved runs should
already be narrowed by the backend, while the client filter acts as a final
consistency layer.

This means filtered saved runs may be narrower than the backend limit, but they
should not miss older matches due to an unfiltered backend slice.

## Testing

Backend tests:

- `list_analysis_runs` applies query before limit.
- filters by source and source group remain mutually exclusive and work with
  additional filters.
- passing both source and source group returns a validation error.
- status supports completed, failed, cancelled, and queued/running.
- provider, model, template, date range, and query terms are case-insensitive,
  literal-contains, or date-inclusive as specified.
- date filters parse only `YYYY-MM-DD` and ignore invalid values.
- query terms can match scope label, source title, group name, template,
  provider profile, provider, model, or error.
- search terms combine as `AND` across terms and `OR` across fields.

Frontend tests:

- `ListAnalysisRunsInput` includes the new optional filter fields.
- `analysis-run-workflow` passes a backend filter projection when loading saved
  runs.
- changing `runsFilter` is enough to reload saved runs.
- stale saved-run responses cannot overwrite newer filter results.
- `filterCompanionRuns` remains the local merge/filter guard for active and
  saved entries.

Existing smoke:

- The opt-in `npm.cmd run smoke:analysis` should continue to pass without new
  scenarios unless the implementation changes visible Runs tab behavior.

## Non-Goals

- Cursor pagination or infinite saved-run history scrolling.
- Bulk delete or cleanup workflows.
- Redesigning Runs tab layout.
- Adding a separate saved-runs route.
- Changing active run event behavior.
- Removing legacy persisted `runFilter` in this slice.
- Adding complete provider/model/template option discovery.

## Acceptance

- Searching or filtering saved runs can return a matching older run that is not
  in the newest unfiltered backend result slice, as long as it is within the top
  `limit` rows after filters are applied.
- Backend rejects `sourceId` and `sourceGroupId` being set together.
- Date strings use `YYYY-MM-DD`; invalid date strings are ignored.
- Query terms are applied before limit as `AND` across terms and `OR` across
  fields, with literal escaping for `%` and `_`.
- Stale saved-run load responses cannot overwrite newer filter results.
- Provider/model/template option discovery is not expanded.
- Existing Runs tab controls keep working.
- Active runs remain visible and filterable in the combined list.
- `npm.cmd run verify` passes.
- If GUI automation is available, `npm.cmd run smoke:analysis` still passes.
