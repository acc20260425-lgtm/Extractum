# Library Catalog Backend Contract Design

Date: 2026-06-14

## Goal

Move the first layer of Library Source Operations from frontend-only view
models into an explicit backend read contract.

This slice adds a `list_library_catalog` command that returns Library sources
together with activity status, latest relevant job, action capabilities,
disabled reasons, and provider/subtype filter counts. It deliberately does not
implement the full Library Source Operations backlog.

## Confirmed Brief

- Implement the first 3.6 backlog slice, not the whole section at once.
- Add a new `list_library_catalog` command beside the existing
  `list_library_sources` command.
- Keep `list_library_sources` temporarily for compatibility.
- Use backend-derived Library source activity/status instead of making the
  frontend merge `list_library_sources` with `list_source_jobs`.
- Keep frontend-owned formatting such as labels, date strings, local search,
  and selected-project connection state.
- Make `/projects/library` call one Library catalog API.
- Move `/projects` Library source loading toward the same catalog API while
  preserving project-specific `Already in project` behavior in the frontend.
- Do not implement YouTube duplicate import outcomes in this slice.
- Do not implement playlist item addability or playlist video materialization
  in this slice.
- Do not implement durable Edit/Archive source overrides in this slice.
- Do not introduce a durable `library_sources` table.

## Chosen Approach

Add a new backend catalog read model and migrate the Library-facing frontend to
it incrementally.

`list_library_catalog` becomes the source of truth for catalog activity,
capabilities, disabled reasons, and filter counts. The existing
`list_library_sources` command remains available for older call sites and for a
smaller raw source record shape.

This keeps the implementation low-risk because existing source listing SQL can
be reused, while the new contract gives the frontend an explicit place to stop
duplicating backend policy.

## Backend API

### Command

Add a Tauri command:

```rust
list_library_catalog() -> AppResult<LibraryCatalogResponse>
```

The command should require source identity repair readiness, matching
`list_library_sources`.

The command should accept `SourceJobState` as Tauri state and use it to attach
the latest relevant in-memory job for each listed source.

### Response Contract

Use snake_case fields across the Tauri boundary.

```ts
export interface LibraryCatalogResponse {
  sources: LibraryCatalogRecord[];
  filter_counts: LibraryCatalogFilterCount[];
}

export interface LibraryCatalogRecord {
  source: LibrarySourceRecord;
  latest_job: SourceJobRecord | null;
  status: LibraryCatalogStatus;
  status_detail: string | null;
  capabilities: LibraryCatalogCapabilities;
  disabled_reasons: LibraryCatalogDisabledReasons;
}

export type LibraryCatalogStatus =
  | "active"
  | "syncing"
  | "error"
  | "unavailable";

export interface LibraryCatalogCapabilities {
  can_refresh: boolean;
  can_delete: boolean;
  can_edit: boolean;
  can_connect_to_project: boolean;
}

export interface LibraryCatalogDisabledReasons {
  refresh: string | null;
  delete: string | null;
  edit: string | null;
  connect_to_project: string | null;
}

export interface LibraryCatalogFilterCount {
  provider: LibrarySourceProvider;
  source_subtype: LibrarySourceSubtype;
  count: number;
  disabled: boolean;
  disabled_reason: string | null;
}
```

`LibrarySourceRecord` remains the current source metadata contract:

- source id;
- provider;
- subtype;
- account id;
- external id;
- title and subtitle;
- canonical URL;
- created and last synced timestamps;
- item count;
- project count;
- optional YouTube details;
- optional Telegram details.

`SourceJobRecord` uses the existing source job contract.

### Latest Job Selection

For each source, attach at most one latest relevant job.

Relevant statuses:

- `queued`;
- `running`;
- `failed`.

Selection rules:

- consider jobs where `job.source_id` equals the source id;
- consider jobs where `job.related_source_id` equals the source id;
- ignore succeeded, cancelled, and cancel-requested jobs for catalog status;
- pick the job with the greatest `started_at`;
- break ties by stable job id ordering.

The catalog command may use a small helper in `SourceJobState` instead of
copying list/filter logic into the Library module.

### Status Semantics

Backend status mapping:

- `syncing` when the latest relevant job is `queued` or `running`;
- `error` when the latest relevant job is `failed`;
- `unavailable` only for source states that the backend knows are unsupported
  for the catalog action surface;
- `active` otherwise.

Status detail:

- for `syncing`, use job message when present, otherwise `Syncing`;
- for `error`, use job error when present, otherwise `Last sync failed`;
- for `active`, use `null`;
- for `unavailable`, use the most relevant backend disabled reason.

Do not add a freshness or stale-source policy in this slice.

### Capabilities And Disabled Reasons

The first catalog capabilities describe source-level operations, not
project-specific operations.

Initial capability rules:

- `can_refresh` is true for supported YouTube sources and Telegram sources that
  can be synced by the current backend.
- `can_refresh` is false while the source has a queued or running relevant job.
- `can_delete` is false when `project_count > 0`, because source deletion is
  already blocked while a source belongs to projects.
- `can_edit` is false in this slice because durable source overrides are not
  implemented yet.
- `can_connect_to_project` is true for Library sources unless the source has a
  backend-known unsupported state.

Initial disabled reason strings:

- refresh during active job: `Source is syncing.`
- delete while used by projects:
  `Source is used by projects. Remove it from projects first.`
- edit before overrides exist: `Source editing is not available yet.`
- unsupported source subtype: a clear provider/subtype-specific reason.

The frontend can still add selected-project-specific disabled reasons such as
`Already in project`, because that depends on the selected project, not on the
source alone.

### Filter Counts

Return provider/subtype buckets from backend data.

The first response must include rows for:

- YouTube videos;
- YouTube playlists;
- YouTube channels;
- Telegram channels;
- Telegram supergroups;
- Telegram groups.

Rules:

- Counts come from the same source list used by the catalog response.
- YouTube channel remains disabled until channel source support is implemented.
- Disabled reasons come from the backend, not hard-coded in the filter rail.
- Missing providers or subtypes can be omitted unless they are part of the
  stable first response rows above.

## Frontend API And Types

Add or extend:

- `src/lib/api/library-sources.ts`;
- `src/lib/types/library-sources.ts`.

New wrapper:

```ts
export function listLibraryCatalog() {
  return invoke<LibraryCatalogResponse>("list_library_catalog");
}
```

Keep:

```ts
export function listLibrarySources() {
  return invoke<LibrarySourceRecord[]>("list_library_sources");
}
```

This makes the migration explicit instead of silently changing the old wrapper.

## Library Catalog Workflow

Update the standalone Library workflow so it depends on one backend call:

```ts
listCatalog(): Promise<LibraryCatalogResponse>;
```

Remove `listSourceJobs()` from the standalone Library catalog workflow.

State should keep the catalog response in a form that supports existing UI:

- source records if still useful for compatibility;
- catalog records;
- derived table rows;
- filter counts;
- loading;
- status.

The view model maps backend catalog status and status detail directly into
`LibraryCatalogSourceView`. It should no longer derive source activity by
merging source jobs.

## Projects Workflow

Update the `/projects` workspace dependency from raw Library source loading to
catalog loading.

The Projects view model may still derive project-specific fields:

- `alreadyConnected`;
- `connectable`;
- selected-project disabled reason;
- current project source links.

Backend catalog disabled reasons should be used as the base source-level
disabled state. If the selected project already contains the source,
`Already in project` remains a frontend-selected-project rule that overrides
connectability for that project.

The `/projects` route can continue calling `listSourceJobs` for the bottom
queue if that UI still needs active job rows. The important simplification is
that Library source row status no longer depends on a separate source-job fetch.

## UI Behavior

`/projects/library` should behave the same from the user's perspective:

- source rows still show Source, Type, Status, Projects, Items, Added, and Last
  synced;
- filter rail still supports provider/subtype filtering;
- Inspector still shows selected source metadata;
- Refresh still reloads the catalog;
- existing Add Source dialog behavior remains unchanged in this slice.

Expected behavior changes:

- status and status detail come from backend catalog records;
- filter disabled reasons come from backend filter count records;
- source action availability can now be read from backend capabilities.

The existing `Edit` and `Delete` buttons may remain visually disabled according
to current prototype behavior until their mutation flows are designed. Their
backend capability data should still be present for the future Library
Inspector and toolbar work.

## Error Handling

If `list_library_catalog` fails:

- show the existing route/workspace status message pattern;
- keep previous rows if workflow state already has them;
- leave Refresh available so the user can retry.

If `SourceJobState` contains no relevant jobs:

- return catalog rows with `latest_job: null`;
- use `active` status unless another backend rule marks the source
  unavailable.

If provider metadata is missing:

- keep the base source row;
- keep optional provider detail blocks nullable;
- do not fail the whole catalog response because one source lacks optional
  provider detail.

## Compatibility

- `list_library_sources` remains registered.
- Existing Add Source flows can keep using their current APIs.
- Existing analysis source APIs are not changed.
- Existing source job commands remain available.
- No database migration is required for this first read-model slice.

## Non-Goals

- YouTube duplicate import outcome contract.
- Playlist item addability in `get_youtube_playlist_detail`.
- Playlist video materialization command.
- Source-first project connection commands.
- Durable Library source overrides.
- Edit source flow.
- Archive source flow.
- Delete source UI flow.
- Project Export.
- Mixed-provider project analysis.
- Durable Library status persistence.

## Testing Notes

Backend tests should cover:

- `query_library_catalog` returns the same base source metadata as
  `query_library_sources`;
- queued or running latest jobs produce `syncing` status and a status detail;
- failed latest jobs produce `error` status and a status detail;
- succeeded or cancelled jobs do not make the catalog row syncing or error;
- `can_delete` is false with the project-membership disabled reason when
  `project_count > 0`;
- `can_edit` is false with the edit-disabled reason;
- filter counts include YouTube video, YouTube playlist, YouTube channel,
  Telegram channel, Telegram supergroup, and Telegram group buckets;
- YouTube channel filter count is disabled with backend-provided reason.

Frontend tests should cover:

- `listLibraryCatalog` invokes `list_library_catalog`;
- Library workflow calls `listCatalog` and no longer calls `listSourceJobs`;
- Library catalog view maps backend `status` and `status_detail` directly;
- filter tree uses backend filter disabled reasons;
- `/projects/library` route uses `listLibraryCatalog`;
- `/projects` route uses catalog loading for Library source rows;
- project-specific `Already in project` behavior still works.

Manual verification should cover:

- open `/projects/library`;
- confirm Library rows load after one catalog request;
- confirm active or failed source jobs show catalog status when a job is
  present;
- confirm provider/subtype filters and counts still work;
- open `/projects`;
- confirm Add from Library still disables already connected sources;
- confirm bottom queue still shows active source jobs if the route keeps that
  queue dependency.

## Acceptance Criteria

- Backend exposes `list_library_catalog`.
- `list_library_catalog` returns source records with latest relevant job,
  backend status, status detail, capabilities, disabled reasons, and filter
  counts.
- `/projects/library` no longer needs to merge Library source records with
  `list_source_jobs` just to derive source status.
- `/projects` uses catalog source data as the base for Library source rows.
- Existing `list_library_sources` remains available.
- No durable schema change is introduced.
- Tests cover the new backend and frontend contracts.
