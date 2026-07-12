# Apalis Jobs Inspector Design

## Goal

Add a top-level `Jobs` page for inspecting Apalis queue rows stored in the local
SQLite `extractum.db` database. The page is manually refreshed and mostly
inspection-oriented, with one guarded maintenance action for pruning old
terminal jobs. It must help diagnose queue state without changing active job
execution, retry, or cancellation behavior.

## Product Shape

The page is a separate navigation destination, not part of Workspace, Projects,
Runs, Diagnostics, or Settings.

- Route: `/jobs`
- Sidebar label: `Jobs`
- Sidebar caption: `Apalis queue`
- Icon: a queue/list-oriented lucide icon such as `ListChecks`, `Rows3`, or
  `ListTree`
- Visibility: include the item in both legacy and projects sidebar modes so the
  queue inspector is always reachable
- Topbar space label: `Jobs` or `Apalis jobs`

The page uses a split inspector layout:

- Left side: filters, manual refresh button, summary counts, and a dense jobs
  table
- Right side: sticky detail panel for the selected job
- Mobile/narrow layout: table first, details below the selected row or below the
  table

## Scope

In scope for v1:

- Read Apalis rows from the `Jobs` table in the main application SQLite
  database
- Show all Apalis job types, not only Gemini Browser
- Filter by `job_type`
- Filter by status
- Search by job id and idempotency key
- Limit result count with a conservative default
- Manually refresh the list
- Delete terminal Apalis jobs older than 24 hours after explicit confirmation:
  `Done`, `Killed`, and `Failed` rows whose `attempts >= max_attempts`
- Select a row and inspect serialized job payload, `last_result`, and metadata
- Show loading, empty, and error states

Out of scope for v1:

- Cancel, kill, retry, resume, reschedule, or any cleanup action other than the
  explicit 24-hour terminal job prune
- Automatic polling
- Reconciliation or mutation of product run logs
- Treating Apalis internals as product state for other screens
- Exporting job payloads
- Cross-database inspection

## Backend Design

Add a small Tauri API dedicated to Apalis job inspection and maintenance. The
list command is read-only:

```text
apalis_jobs_list
```

The list command reads through the existing app database pool from
`crate::db::get_pool(handle)`. It does not create, update, delete, lock, resume,
or acknowledge jobs.

The maintenance command is:

```text
apalis_jobs_prune_terminal
```

It deletes only terminal rows older than 24 hours by normalized `done_at`:

- `status = 'Done'`
- `status = 'Killed'`
- `status = 'Failed' AND attempts >= max_attempts`

It must not call Apalis' raw SQLite `vacuum` because that deletes terminal rows
without the 24-hour age guard. It also must not read `job`, `last_result`, or
`metadata` payloads before deleting.

The implementation must not rely on Context7 documentation alone for the SQL
shape. The first implementation task must add a local schema discovery test
against this repository's pinned Apalis stack:

- `apalis = "=1.0.0-rc.8"`
- `apalis-sqlite = "=1.0.0-rc.8"`
- app migrations applied through `crate::migrations`
- Apalis setup applied through the same helper used by Gemini Browser storage

That test must prove the actual local table name and columns before the read
model is implemented. It should inspect SQLite metadata with a stable query such
as `PRAGMA table_info('Jobs')` or `sqlite_master`, seed at least one real Apalis
job through `TaskSink` or the existing Gemini Browser enqueue helper, and verify
the query can read the local row. If the local table name or required columns
differ from the spec, update the spec and implementation plan before building
the UI.

### Request DTO

```text
ApalisJobsListRequest
- limit: Option<u32>
- status: Option<String>
- job_type: Option<String>
- search: Option<String>
```

Rules:

- Default limit: 100
- Maximum limit: 500
- Empty strings are treated as no filter
- `status` and `job_type` are exact filters
- `search` matches `id` and `idempotency_key` with a contains query
- Query results are ordered by `last_activity_at` descending, then `id`
  descending. `last_activity_at` is the latest non-null timestamp among
  `done_at`, `lock_at`, and `run_at`, not the first non-null value. The
  implementation may compute this in Rust after reading rows if SQLite timestamp
  representation makes a correct SQL `MAX` expression awkward.

### Response DTO

```text
ApalisJobsListResponse
- jobs: Vec<ApalisJobRow>
- total_matching: u32
- status_counts: Vec<ApalisJobStatusCount>
- job_type_counts: Vec<ApalisJobTypeCount>
- refreshed_at: String
- limit: u32
```

`refreshed_at` is an RFC3339 UTC timestamp.

`total_matching` reflects rows matching all active filters before limit.

Counts use non-self filter semantics so filter chips remain useful:

- `status_counts` applies `job_type` and `search` filters, but ignores the
  current `status` filter.
- `job_type_counts` applies `status` and `search` filters, but ignores the
  current `job_type` filter.
- Counts are computed before the result limit.
- If the `Jobs` table is missing, all counts are empty.

### Prune DTOs

```text
ApalisJobsPruneTerminalRequest
- older_than_hours: Option<u32> // UI sends 24; backend defaults to 24 and never lowers below 24

ApalisJobsPruneTerminalResponse
- deleted_count: u64
- cutoff_at: String             // RFC3339 UTC
- older_than_hours: u32
```

If the `Jobs` table is missing, or the local schema lacks `status` or `done_at`,
the command returns `deleted_count = 0`.

### Row DTO

The response shape is stable. After local schema discovery confirms the current
`Jobs` table shape, every row still returns all fields below. If a compatible
column is absent in the local schema, the backend fills the corresponding DTO
field with `None` for optional fields or a documented default for required
fields. The frontend must not branch on runtime schema shape.

```text
ApalisJobRow
- id: String
- job_type: String              // "" only if the column is unexpectedly absent
- status: String                // "unknown" only if the column is unexpectedly absent
- attempts: u32
- max_attempts: Option<u32>
- run_at: Option<String>
- lock_at: Option<String>
- lock_by: Option<String>
- done_at: Option<String>
- last_activity_at: Option<String>
- priority: Option<u32>
- idempotency_key: Option<String>
- job_preview: Option<String>
- job_truncated: bool
- job_json: Option<serde_json::Value>
- last_result: Option<serde_json::Value>
- last_result_truncated: bool
- metadata: Option<serde_json::Value>
- metadata_truncated: bool
```

All timestamp strings returned to the frontend must be normalized to RFC3339 UTC
before serialization. The frontend must never receive SQLite-local time strings
or ambiguous numeric timestamps. If Apalis stores an integer epoch, the backend
converts it; if it stores a text timestamp, the backend parses and re-emits it
as RFC3339 UTC. If parsing fails, return `None` for the normalized timestamp and
keep the original value out of the UI-facing DTO.

The current Apalis core statuses are expected to include `Pending`, `Queued`,
`Running`, `Done`, `Failed`, and `Killed`. The API must not reject unknown
statuses; unknown values should pass through as strings so the UI can display
future Apalis changes.

### Payload Handling

Apalis stores serialized job payloads in an internal SQL column. The inspector
may decode JSON payloads when the storage format is JSON-compatible. If decoding
fails, return a short lossy textual preview and leave `job_json` as `None`.

Payloads may contain prompts, provider configuration, cookies, tokens, endpoint
details, or other sensitive diagnostic material. The backend must sanitize and
bound every UI-facing payload field:

- Redact object keys whose normalized name contains any normalized sensitive
  fragment: `apikey`, `authorization`, `bearer`, `cookie`, `credentials`,
  `password`, `secret`, `session`, `token`, `apihash`, or `refreshtoken`.
- Key normalization is explicit: convert to lowercase, then remove `_`, `-`,
  spaces, and all non-alphanumeric ASCII characters before matching. This means
  keys such as `apiKey`, `api-key`, `API Key`, `refreshToken`,
  `refresh_token`, and `refresh token` all match.
- Redaction is recursive for JSON objects and arrays.
- Use the exact replacement string `[redacted]`.
- Limit each of `job_json`, `last_result`, and `metadata` to at most 64 KiB of
  serialized JSON after redaction. If a section exceeds the limit, do not return
  partial arbitrary JSON. Return this valid JSON object instead:

```json
{
  "truncated": true,
  "preview": "redacted JSON preview capped at 2000 characters"
}
```

- The corresponding `*_truncated` flag must be `true` for that section.
- The preview is produced from the redacted serialized JSON only, never from raw
  unredacted bytes.
- If payload decoding fails, `job_json` remains `None`, and `job_preview`
  contains the redacted lossy text preview capped at 500 characters.
- For decode failures, `job_truncated` is `true` only when the raw lossy text was
  longer than the capped preview.
- `job_preview` is a redacted text preview capped at 500 characters.
- Do not add copy, export, open-folder, or share controls in v1.

### Error Handling

- If the `Jobs` table does not exist, return an empty list with counts instead
  of surfacing a scary database error.
- If pruning is requested while the `Jobs` table does not exist, return
  `deleted_count = 0`.
- Other database errors should surface through the existing `AppError` shape.
- Invalid limits should be clamped rather than rejected.

## Frontend Design

Add:

- `$lib/types/apalis-jobs.ts`
- `$lib/api/apalis-jobs.ts`
- `src/routes/jobs/+page.svelte`
- A focused component such as
  `$lib/components/jobs/ApalisJobsPanel.svelte`

The page should use existing application styles: `page-shell`, `page-hero`,
`desk-panel`, compact buttons, restrained borders, and existing badge/date
formatting helpers where possible.

### Layout

The left pane contains:

- Page title `Jobs`
- Subtitle describing local Apalis queue inspection
- Manual refresh button with refresh icon
- Delete old finished jobs button with trash icon, destructive styling, and browser
  confirmation before calling the prune command
- Last refreshed timestamp
- Status chips or small summary counters
- Filter row:
  - Status select
  - Job type select
  - Search input
  - Limit select or numeric input
- Dense table:
  - Status
  - Job type
  - Idempotency key or id
  - Attempts
  - Run at
  - Lock at
  - Done at
  - Last activity

The right pane contains selected job details:

- Stable identity block: id, idempotency key, job type
- Execution block: status, attempts, priority, lock owner
- Timing block: `run_at`, `lock_at`, `done_at`, and `last_activity_at`
- JSON sections for `job_json`, `last_result`, and `metadata`, with visible
  truncated markers when any `*_truncated` flag is true

If no job is selected, show an empty detail state asking the user to select a job.

### Interaction

- Page loads once on mount
- Refresh button reloads data
- Changing any filter reloads data by calling `apalis_jobs_list` with the full
  current filter request. The frontend must not derive filtered rows, counts, or
  `total_matching` by filtering an already limited local result set.
- If the selected job disappears after refresh, select the first row if present,
  otherwise clear selection
- Delete old finished jobs confirms with the user, calls
  `apalis_jobs_prune_terminal`, reports the deleted count, and reloads the list
- No automatic polling in v1
- No mutating controls besides the guarded 24-hour terminal prune

## Navigation

Add `Jobs` to both `legacyNavItems` and `projectsNavItems` in the root layout so
the page is independent of the current workspace mode. The route should not force
switching between legacy and projects modes.

The root topbar route label should recognize `/jobs` and display `Jobs`.

## Testing Plan

Use TDD for implementation.

Backend tests:

- `apalis_jobs_schema_probe_documents_local_jobs_table_shape`
- `apalis_jobs_list_returns_rows_from_jobs_table`
- `apalis_jobs_list_filters_by_status_job_type_and_search`
- `apalis_jobs_list_clamps_limit`
- `apalis_jobs_list_returns_empty_when_jobs_table_missing`
- `apalis_jobs_list_does_not_mutate_jobs`
- `apalis_jobs_list_sorts_by_latest_activity_timestamp`
- `apalis_jobs_list_returns_rfc3339_utc_timestamps`
- `apalis_jobs_counts_ignore_their_own_active_filter`
- `apalis_jobs_prune_terminal_deletes_only_old_done_killed_and_terminal_failed_jobs`
- `apalis_jobs_prune_terminal_returns_zero_when_jobs_table_missing`
- `apalis_jobs_payloads_are_redacted_and_truncated`
- `apalis_jobs_row_shape_is_stable_when_optional_columns_are_absent`

Frontend API tests:

- wrapper calls `apalis_jobs_list`
- wrapper calls `apalis_jobs_prune_terminal` with `older_than_hours = 24`
- request fields are passed as expected
- response typing covers nullable JSON fields
- response typing covers normalized timestamps and truncation flags

UI/source contract tests:

- sidebar includes top-level `Jobs` item in both nav modes
- `/jobs` route renders manual refresh, filters, table, and detail panel
- refresh calls the list API again
- delete old finished jobs uses confirmation, calls the prune API, and then refreshes
- changing filters calls the API again instead of filtering the limited local
  rows
- selecting a row displays job details
- truncated and redacted detail sections are labeled without exposing raw secret
  values
- empty and error states render without layout collapse

Verification commands:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis-jobs --lib apalis_jobs
npm.cmd test -- src/lib/api/apalis-jobs.test.ts src/lib/apalis-jobs-panel.test.ts
npm.cmd run check
git diff --check
```

## Open Decisions

None for v1. The page is manually refreshed, implemented as a top-level split
inspector, and has exactly one guarded maintenance mutation: pruning terminal
jobs older than 24 hours.

## Self-Review

- No placeholder TODOs remain.
- The design keeps active queue execution untouched and avoids lifecycle side
  effects.
- The page is explicitly separate from Runs, Projects, Workspace, Diagnostics,
  and Settings.
- The SQL inspection is scoped to local diagnostics and does not become a source
  of product truth for existing UI.
- Unknown Apalis status values are displayed rather than rejected.
- The design now requires local schema discovery before relying on Apalis SQL
  internals.
- Timestamp format, sorting, counts, stable DTO shape, filter reloads, redaction
  normalization, and truncation behavior are explicit.
