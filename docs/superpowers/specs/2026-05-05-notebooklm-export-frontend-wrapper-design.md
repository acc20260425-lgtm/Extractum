# NotebookLM Export Frontend Wrapper Design

## Purpose

Add a focused frontend API wrapper for the existing NotebookLM export Tauri
command and event listener. This continues the boundary cleanup already done
for core Sources and Takeout import while keeping the implementation deliberately
narrow.

The current `/analysis` route still owns the raw NotebookLM export command
string and the `notebooklm://export` listener directly. This work centralizes
that frontend/backend boundary in `$lib/api/notebooklm-export.ts` without
changing backend Rust code, event payloads, route state shape, dialog behavior,
or UI composition.

## Scope

Included:

- Create `$lib/api/notebooklm-export.ts`.
- Add unit tests for the wrapper.
- Replace only NotebookLM export raw `invoke(...)` and `listen(...)` usage in
  `src/routes/analysis/+page.svelte`.
- Keep existing NotebookLM export types from `$lib/types/sources`.

Excluded:

- Backend command or event changes.
- NotebookLM DTO camelCase migration.
- NotebookLM export workflow/controller extraction.
- Folder picker abstraction; `openDialog(...)` remains route-local UI behavior.
- Chat, template, source group, Takeout, or source management refactors.
- Rust-to-TypeScript type generation.

## Frontend Contract

`src/lib/api/notebooklm-export.ts` exposes:

```ts
export const NOTEBOOKLM_EXPORT_EVENT = "notebooklm://export";

export function exportSourceToNotebookLm(
  request: NotebookLmExportRequest,
): Promise<NotebookLmExportResult>;

export function listenToNotebookLmExportEvents(
  handler: (event: Event<NotebookLmExportEvent>) => void,
): Promise<UnlistenFn>;
```

The wrapper uses the existing command name:

```text
export_source_to_notebooklm
```

The event name stays:

```text
notebooklm://export
```

## Data Shape

Use the existing NotebookLM export types from `src/lib/types/sources.ts`:

- `NotebookLmExportRequest`
- `NotebookLmExportResult`
- `NotebookLmExportEvent`

The UI continues to consume existing snake_case fields such as `export_id`,
`source_id`, `output_dir`, `period_from`, `period_to`, `progress_current`,
`progress_total`, `file_path`, and `exported_message_count`.

The route continues to build requests with `notebookLmExportRequestFromForm(...)`
from `src/lib/analysis-state.ts`. That helper stays outside the API wrapper
because it maps route form state into the existing backend wire shape.

## Analysis Route Migration

Only the NotebookLM export boundary moves from
`src/routes/analysis/+page.svelte` into the wrapper:

- `invoke<NotebookLmExportResult>("export_source_to_notebooklm", { request })`
  becomes `exportSourceToNotebookLm(request)`.
- `listen<NotebookLmExportEvent>("notebooklm://export", ...)` becomes
  `listenToNotebookLmExportEvents(...)`.

The route keeps raw `invoke` and `listen` imports because other non-NotebookLM
boundaries still use them. The route also keeps `openDialog(...)` for choosing
the output directory.

## Testing

Add `src/lib/api/notebooklm-export.test.ts` with Vitest mocks for:

- `@tauri-apps/api/core`
- `@tauri-apps/api/event`

Tests verify the command name, `{ request }` payload shape, event constant, and
listener event forwarding.

Required verification:

```powershell
npm.cmd test -- notebooklm-export
npm.cmd test -- analysis-state notebooklm-export takeout-import analysis-runs sources
npm.cmd test
npm.cmd run check
git diff --check
```

Also verify the migrated route does not retain raw NotebookLM export command or
event strings:

```powershell
rg -n "export_source_to_notebooklm|notebooklm://export" src\routes\analysis\+page.svelte
```

Expected result:

```text
no matches
```

If Vite, esbuild, or Svelte preprocessing fails with `spawn EPERM` in the
default Windows sandbox, rerun frontend verification outside the sandbox after
approval, matching the existing repository notes.
