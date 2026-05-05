# Takeout Import Frontend Wrapper Design

## Purpose

Add a focused frontend API wrapper for the existing Takeout import Tauri
commands and event listener. This continues the boundary cleanup started by
Sources Contract V2 while keeping the implementation intentionally narrow.

The current `/analysis` route still owns Takeout command strings and the
`sources://takeout-import` listener directly. This work centralizes that
frontend/backend boundary in `$lib/api/takeout-import.ts` without changing
backend Rust code, event payloads, route behavior, or UI state shape.

## Scope

Included:

- Create `$lib/api/takeout-import.ts`.
- Add unit tests for the wrapper.
- Replace only Takeout-specific raw `invoke(...)` and `listen(...)` usage in
  `src/routes/analysis/+page.svelte`.
- Keep existing Takeout types from `$lib/types/sources`.

Excluded:

- CamelCase migration for Takeout DTO fields.
- Takeout workflow controller extraction.
- Backend command or event changes.
- NotebookLM export wrapper work.
- Source, chat, template, or group workflow refactors.

## Frontend Contract

`src/lib/api/takeout-import.ts` exposes:

```ts
export const TAKEOUT_IMPORT_EVENT = "sources://takeout-import";

export function listTakeoutSourceImportJobs(): Promise<TakeoutImportJobRecord[]>;

export function startTakeoutSourceImport(
  sourceId: number,
): Promise<StartTakeoutImportResponse>;

export function cancelTakeoutSourceImport(
  jobId: string,
): Promise<CancelTakeoutImportResponse>;

export function listenToTakeoutImportEvents(
  handler: (event: Event<TakeoutImportEvent>) => void,
): Promise<UnlistenFn>;
```

The wrapper uses existing command names:

```text
list_takeout_source_import_jobs
start_takeout_source_import
cancel_takeout_source_import
```

The event name stays:

```text
sources://takeout-import
```

## Data Shape

Use the existing Takeout types from `src/lib/types/sources.ts`:

- `TakeoutImportJobRecord`
- `TakeoutImportEvent`
- `StartTakeoutImportResponse`
- `CancelTakeoutImportResponse`

The UI continues to consume existing snake_case fields such as `job_id`,
`source_id`, `progress_current`, `progress_total`, `started_at`, and
`finished_at`.

## Testing

Add `src/lib/api/takeout-import.test.ts` with Vitest mocks for:

- `@tauri-apps/api/core`
- `@tauri-apps/api/event`

Tests verify the command names, payload shapes, event constant, and listener
event forwarding.

Required verification:

```powershell
npm.cmd test -- takeout-import
npm.cmd test -- analysis-runs sources
npm.cmd test
npm.cmd run check
git diff --check
```

If Vite or esbuild fails with `spawn EPERM` in the sandbox, rerun frontend
verification outside the sandbox, matching the existing repo note.
