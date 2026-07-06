# Project Add Source Shortcut Design

Status: approved for implementation planning
Date: 2026-07-06

## Objective

Add a project-scoped `Add source` action next to every existing `Connect from Library` action. The new action opens the current full Library add-source dialog, lets the user add a YouTube or Telegram source, and automatically connects the resulting Library source to the currently selected project.

The flow must also handle sources that already exist in Library: notify the user unobtrusively, then allow connecting the existing source to the current project without forcing the user to switch to the `Connect from Library` sheet.

## Current Code Reality

- `SourcesTab.svelte` renders the visible `Connect from Library` button in the project sources tab.
- `ProjectsShell.svelte` hosts `ConnectFromLibrary` for the `src/routes/projects/+page.svelte` and `src/routes/projects/list/+page.svelte` shells.
- `src/routes/projects/next/+page.svelte` owns a separate project surface and hosts its own `ConnectFromLibrary` instance.
- `LibraryAddSourceDialog.svelte` already contains the full add-source experience with provider tabs for YouTube and Telegram.
- `LibraryAddSourceDialog` accepts:
  - `sources`, used by YouTube duplicate detection.
  - `onSourcesChanged(sourceId?: number)`, called after a source is added.
  - `onStatus(message)`, used to surface non-blocking workflow messages.
- `LibraryYoutubeSmartImport.svelte` already detects an existing YouTube source via `existingYoutubeSmartImportSource(sources, preview)`, but currently treats it as a terminal disabled state.
- `LibraryTelegramDialogImport.svelte` calls `addTelegramSource(...)` and then `onSourcesChanged(source.id)`.
- `LibraryYoutubePlaylistImport.svelte` can add multiple playlist videos and returns per-row source IDs through the batch workflow summary, but the current component calls `onSourcesChanged(...)` once with only the first non-null source ID.
- The project workflow already connects sources through `addProjectSources({ projectId, sourceIds })`.
- `add_project_sources` is idempotent on the backend via `INSERT OR IGNORE` and returns `{ added_count, already_present_count }`; final status copy should use this outcome rather than trusting only a frontend snapshot.

## Scope

In scope:

- Add an `Add source` button wherever the project UI already exposes `Connect from Library`.
- Reuse the existing full `LibraryAddSourceDialog`; do not build a separate provider-specific project dialog.
- Add a project-aware callback path that receives a source ID from the Library dialog and connects it to the active project.
- Keep the standalone Library route behavior unchanged: adding a source there still only adds/refreshes Library.
- For YouTube Smart import duplicates, change the existing Library-only disabled state into a project-aware connect path when the dialog is opened from a project.
- Make provider components project-aware through explicit props only; they render project-mode UI branches, but they must not import project APIs.
- Update `LibraryYoutubePlaylistImport` so project mode can connect every successful playlist-video source ID, not only the first one.
- Add a shared project add-source workflow helper so `ProjectsShell` and `/projects/next` do not duplicate refresh/connect/status wiring.
- Show an unobtrusive status when the source already exists in Library and then gets connected to the project.
- Show an unobtrusive status when the source is already connected to the project and skip the duplicate project connect call.
- Cover the behavior with focused workflow and UI contract tests.

Out of scope:

- A new Tauri command that atomically adds-or-connects provider sources.
- New source providers beyond the dialog's current YouTube and Telegram tabs.
- Changes to Library catalog persistence, project source schema, migrations, or source identity rules.
- Replacing `Connect from Library`.
- Changing the existing YouTube playlist materialization policy.
- Changing Telegram account authorization or dialog loading behavior.
- A visual redesign of the Library route.

## UX Design

### Project Sources Toolbar

Every project screen area that currently has `Connect from Library` also gets:

- Label: `Add source`
- Icon: `Plus`
- Position: immediately before `Connect from Library` because it is the direct primary action for new material.
- Disabled state: disabled when no project is selected, matching the surrounding project-source actions.

The existing `Connect from Library` action remains the path for browsing and connecting multiple existing Library sources at once.

### Dialog

The button opens the current `LibraryAddSourceDialog` with all existing tabs:

- YouTube
  - Smart import
  - From existing data
- Telegram

The title remains `Add source`. In project mode, provider components receive explicit project-context props and render project-aware button states such as `Connect to project` and `Already connected to this project`. This is intentional UI branching inside the existing dialog, not a separate dialog.

### Status Copy

Use short status messages rather than modal alerts. Status ownership is split by channel:

- Provider-local `status`: preview/import errors and provider-local guidance such as malformed URLs.
- Derived provider UI: persistent states such as `Already in Library`, `Connect to project`, and `Already connected to this project`.
- Project workflow `onStatus`: post-action outcomes after attempting to connect a source to a project.

Project workflow status copy:

- New source added and connected: `Source added and connected to project.`
- Existing Library source connected: `Already in Library. Connected to project.`
- Existing Library source already in project: `Already connected to this project.`
- Source added to Library but no source ID is available: `Source added to Library. Refreshing project sources.`
- Project connection failure after Library add succeeds: `Source added to Library, but connecting it to the project failed: <error>`

## Behavior

### New Source

1. User clicks `Add source` in a project.
2. `LibraryAddSourceDialog` opens.
3. User completes any existing provider add flow.
4. The provider component receives a `source.id` from the add-source API.
5. The project-aware `onSourcesChanged(source.id)` callback delegates to the shared project add-source helper for the newly created Library source.
6. The helper refreshes Library/catalog state, calls `addProjectSources({ projectId, sourceIds: [source.id] })`, uses the backend outcome to choose status copy, and reloads project state.
7. The user sees `Source added and connected to project.` when `added_count > 0`.

### Source Already In Library

For YouTube Smart import, duplicate detection already exists after preview. In project mode:

1. The UI displays `Already in Library: <title>` as informational text.
2. The action button changes from the disabled `Already in Library` state to `Connect to project` if the reactive project context does not mark the source as linked to the current project.
3. Clicking `Connect to project` invokes the project-context existing-source connect callback with the existing Library source ID.
4. The helper calls `addProjectSources` and uses its outcome:
   - `added_count > 0`: show `Already in Library. Connected to project.`
   - `already_present_count > 0`: show `Already connected to this project.`

If the existing source is already connected to the current project:

1. The UI displays `Already connected to this project.`
2. The action is disabled or omitted.
3. No `addProjectSources` call is made.

The backend outcome remains the final source of truth for status because the project source list can be stale or changed by another action.

### YouTube Smart Import Playlist

When `preview.kind === "playlist"`, Smart import creates or reuses the playlist container source because it calls `addYoutubeSource(..., { materializePlaylistVideos: false })`.

Project mode connects that playlist container source to the project. It does not automatically connect every video in the playlist. Users who want individual video sources use the existing `From existing data` tab.

### Existing Data / Multiple Adds

`LibraryYoutubePlaylistImport` can add several videos. The current implementation only calls `onSourcesChanged(...)` once with the first non-null source ID; v1 must change that component so project mode can connect every successful or known existing video source.

- `onSourcesChanged(sourceId)` is called once for each successful source ID.
- The project-aware callback connects that single source ID to the project.
- Multiple refreshes are intentional for v1 because this preserves the existing dialog contract and avoids a broader callback migration.

Rows skipped because they are already in Library should not be silently ignored in project mode if a source ID is known from the current catalog. They should connect the existing source to the project or report `Already connected to this project`.

Standalone Library mode continues to call `onSourcesChanged(...)` once after the batch to preserve the current Library refresh behavior. The per-source callback requirement applies to project mode.

### Telegram

Telegram import already returns one `source.id` after `addTelegramSource`. The project-aware callback connects that ID to the project. If `addTelegramSource` returns an existing source for an already-known Telegram dialog, the same callback path handles it.

## Data Flow

Add a small project-context layer above `LibraryAddSourceDialog` rather than moving project API calls into provider components.

Recommended shape:

- `LibraryAddSourceDialog` remains reusable and receives `projectContext?: ProjectAddSourceContext`.
- `projectContext` contains:
  - `projectId`;
  - a reactive `connectedSourceIds` set for derived provider UI;
  - an `onConnectExistingSource(sourceId, origin)` callback owned by the project workflow layer.
- Project screens pass a project-aware `onSourcesChanged(sourceId?)` callback that delegates to the shared helper for newly created Library sources.
- Provider components use `projectContext` to render project-aware UI and to invoke `onConnectExistingSource` when an existing Library source should be connected without creating a new source.
- Provider components do not import `addProjectSources`, project APIs, or route-level state directly.
- The shared project add-source helper owns:
  - refreshing Library catalog data;
  - calling `addProjectSources`;
  - interpreting `{ added_count, already_present_count }`;
  - refreshing project data after each connect attempt;
  - updating the reactive project context while the dialog remains open;
  - setting project workflow status copy.

`connectedSourceIds` is derived from `projectSources` filtered to the active project. `projectSourceLinks` is a display view and must not be the source of truth for duplicate detection.

Frontend pre-checks are UI optimizations only. If `connectedSourceIds` says a source is already connected, the UI can skip `addProjectSources`. If the UI attempts a connect, the backend outcome decides the final status.

Implement the helper in a shared UI/workflow module, `src/lib/ui/project-add-source-workflow.ts`. `createResearchProjectsWorkflow` and `/projects/next` both call this helper instead of copying the refresh/connect/status sequence.

## Component Changes

Expected touched areas:

- `SourcesTab.svelte`
  - Add `Add source` button next to `Connect from Library`.
  - Add an `onOpenAddSource` prop.

- `ProjectsShell.svelte`
  - Track whether the project add-source dialog is open.
  - Render `LibraryAddSourceDialog` for project context.
  - Pass the shared project add-source helper into `LibraryAddSourceDialog`.
  - Pass a reactive `connectedSourceIds` set derived from active project `projectSources`.

- `src/routes/projects/next/+page.svelte`
  - Wire the same `Add source` button and project dialog because this route owns a separate `ConnectFromLibrary` instance.
  - Reuse the same shared helper used by `ProjectsShell`.

- `src/routes/projects/+page.svelte`
  - Pass any new `ProjectsShell` add-source props through the existing workflow.

- `src/routes/projects/list/+page.svelte`
  - Pass any new `ProjectsShell` add-source props through the existing workflow.

- `src/lib/ui/project-add-source-workflow.ts`
  - Provide the shared project add-source helper.
  - Use `addProjectSources` outcome for status.
  - Refresh state after each successful or already-present connect outcome.

- `src/lib/ui/research-projects-workflow.ts`
  - Expose a small wrapper around the shared project add-source helper for `ProjectsShell`.

- `LibraryAddSourceDialog.svelte`
  - Accept optional project context without breaking the Library route.
  - Pass project context to provider components.

- `LibraryYoutubeSmartImport.svelte`
  - Keep existing Library duplicate detection.
  - In project mode, allow existing Library sources to be connected to the project.
  - Show `Already connected to this project` when applicable.
  - Treat playlist Smart import as connecting the playlist container source.

- `LibraryYoutubePlaylistImport.svelte`
  - In project mode, call `onSourcesChanged(sourceId)` once for each successful newly added source ID in `summary.results`.
  - In project mode, map skipped existing rows to catalog source IDs when available and connect those existing sources to the project.
  - Preserve the current standalone Library behavior.

## Error Handling

- Library add failure: keep existing provider-local error handling.
- Project connect failure after Library add success: do not roll back the Library source. Show a status explaining that the source was added but project connection failed.
- Missing project: disable `Add source` and do not open the project dialog.
- Missing source ID from a provider flow: refresh Library and project views, then show a status that the source was added to Library but could not be auto-connected.
- Backend idempotency and `already_present_count` are the final authority for connect outcomes.
- If the reactive UI context knows a source is already connected, it should not call `addProjectSources`.
- If a connect is attempted and the backend returns `already_present_count > 0`, show `Already connected to this project.` and refresh project state.

## Testing

Add or update focused tests:

- UI contract test: project sources toolbar includes `Add source` next to `Connect from Library`.
- Host coverage test: `/projects`, `/projects/list`, and `/projects/next` expose the project add-source path wherever they expose `Connect from Library`.
- Workflow test: project-mode source add calls `addProjectSources({ projectId, sourceIds: [sourceId] })` after Library add reports a source ID.
- Workflow outcome test: `already_present_count > 0` produces `Already connected to this project.`
- Workflow failure test: project connect failure after Library add success reports that Library add succeeded but project connect failed.
- Missing source ID test: project-mode callback refreshes and reports that auto-connect could not be completed.
- Reactive context test: after one connect while the dialog remains open, `connectedSourceIds` updates before the next provider action.
- Duplicate test: an existing YouTube Library source can be connected from the Smart import preview in project mode.
- Already-connected test: the project-mode duplicate path shows `Already connected to this project` and does not call `addProjectSources`.
- Smart playlist test: YouTube Smart import playlist connects the playlist container source, not the playlist videos.
- Telegram existing-source test: a returned existing Telegram source ID is routed through the same project connect helper.
- Existing Library route test: standalone Library add-source behavior remains Library-only and does not import project APIs.
- Multiple-add test: playlist-added source IDs each connect to the project in project mode, including all non-null IDs in `summary.results`.

## Acceptance Criteria

- Users can add a YouTube or Telegram source from a project without first going to the Library route.
- After successful add, the source appears in Library and in the active project.
- If a YouTube Smart import source already exists in Library, the user can connect it from the same dialog.
- If the source is already in the project, the UI reports that fact without making a duplicate connect call.
- If a connect attempt races with existing membership, the backend `already_present_count` outcome produces `Already connected to this project.`
- YouTube Smart import playlists connect the playlist container source.
- YouTube existing-data multiple adds connect every successful video source ID in project mode.
- Existing `Connect from Library` behavior remains unchanged.
- Existing standalone Library `Add source` behavior remains unchanged.
- Tests cover the new project add-source flow and duplicate handling.
