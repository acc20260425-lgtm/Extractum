# Apalis Jobs Inspector Design

## Goal

Add a top-level `Jobs` page for inspecting Apalis queue rows stored in the local
SQLite `extractum.db` database. The first version is read-only and manually
refreshed. It must help diagnose queue state without changing job execution,
retry, cancellation, or cleanup behavior.

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
- Select a row and inspect serialized job payload, `last_result`, and metadata
- Show loading, empty, and error states

Out of scope for v1:

- Cancel, kill, retry, resume, reschedule, delete, or cleanup actions
- Automatic polling
- Reconciliation or mutation of product run logs
- Treating Apalis internals as product state for other screens
- Exporting job payloads
- Cross-database inspection

## Backend Design

Add a small read-only Tauri API dedicated to Apalis job inspection. A suggested
command name is:

```text
apalis_jobs_list
```

The command reads through the existing app database pool from
`crate::db::get_pool(handle)`. It does not create, update, delete, lock, resume,
or acknowledge jobs.

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
- Query results are ordered by the most operationally useful timestamp:
  `COALESCE(run_at, lock_at, done_at)` descending, with a stable fallback to `id`

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

`total_matching` reflects rows matching filters before limit. Counts are derived
from the same filtered set where practical. If this becomes too expensive later,
counts can be limited to the current result set, but v1 should prefer accurate
filtered counts.

### Row DTO

Based on the current Apalis SQL row shape documented by Context7, include these
fields:

```text
ApalisJobRow
- id: String
- job_type: String
- status: String
- attempts: u32
- max_attempts: Option<u32>
- run_at: Option<String>
- lock_at: Option<String>
- lock_by: Option<String>
- done_at: Option<String>
- priority: Option<u32>
- idempotency_key: Option<String>
- job_preview: Option<String>
- job_json: Option<serde_json::Value>
- last_result: Option<serde_json::Value>
- metadata: Option<serde_json::Value>
```

The current Apalis core statuses are expected to include `Pending`, `Queued`,
`Running`, `Done`, `Failed`, and `Killed`. The API must not reject unknown
statuses; unknown values should pass through as strings so the UI can display
future Apalis changes.

### Payload Handling

Apalis stores serialized job payloads in an internal SQL column. The inspector
may decode JSON payloads when the storage format is JSON-compatible. If decoding
fails, return a short lossy textual preview and leave `job_json` as `None`.

Payloads may contain prompts or provider configuration. This is a local-only
diagnostic page, but v1 must avoid adding copy/export/share controls.

### Error Handling

- If the `Jobs` table does not exist, return an empty list with counts instead
  of surfacing a scary database error.
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

The right pane contains selected job details:

- Stable identity block: id, idempotency key, job type
- Execution block: status, attempts, priority, lock owner
- Timing block: run_at, lock_at, done_at
- JSON sections for `job_json`, `last_result`, and `metadata`

If no job is selected, show an empty detail state asking the user to select a job.

### Interaction

- Page loads once on mount
- Refresh button reloads data
- Changing filters reloads or filters via local state, whichever keeps the
  implementation simpler and consistent with existing app patterns
- If the selected job disappears after refresh, select the first row if present,
  otherwise clear selection
- No automatic polling in v1
- No mutating controls in v1

## Navigation

Add `Jobs` to both `legacyNavItems` and `projectsNavItems` in the root layout so
the page is independent of the current workspace mode. The route should not force
switching between legacy and projects modes.

The root topbar route label should recognize `/jobs` and display `Jobs`.

## Testing Plan

Use TDD for implementation.

Backend tests:

- `apalis_jobs_list_returns_rows_from_jobs_table`
- `apalis_jobs_list_filters_by_status_job_type_and_search`
- `apalis_jobs_list_clamps_limit`
- `apalis_jobs_list_returns_empty_when_jobs_table_missing`
- `apalis_jobs_list_does_not_mutate_jobs`

Frontend API tests:

- wrapper calls `apalis_jobs_list`
- request fields are passed as expected
- response typing covers nullable JSON fields

UI/source contract tests:

- sidebar includes top-level `Jobs` item in both nav modes
- `/jobs` route renders manual refresh, filters, table, and detail panel
- refresh calls the API again
- selecting a row displays job details
- empty and error states render without layout collapse

Verification commands:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis-jobs --lib apalis_jobs
npm.cmd test -- src/lib/api/apalis-jobs.test.ts src/lib/apalis-jobs-panel.test.ts
npm.cmd run check
git diff --check
```

## Open Decisions

None for v1. The page is read-only, manually refreshed, and implemented as a
top-level split inspector.

## Self-Review

- No placeholder TODOs remain.
- The design keeps the inspector read-only and avoids lifecycle side effects.
- The page is explicitly separate from Runs, Projects, Workspace, Diagnostics,
  and Settings.
- The SQL inspection is scoped to local diagnostics and does not become a source
  of product truth for existing UI.
- Unknown Apalis status values are displayed rather than rejected.
