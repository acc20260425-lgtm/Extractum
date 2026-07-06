# Project Add Source Shortcut Design

Status: approved for implementation planning
Date: 2026-07-06

## Objective

Add a project-scoped `Add source` action next to every existing `Connect from Library` action. The new action opens the current full Library add-source dialog, lets the user add a YouTube or Telegram source, and automatically connects the resulting Library source to the currently selected project.

The flow must also handle sources that already exist in Library: notify the user unobtrusively, then allow connecting the existing source to the current project without forcing the user to switch to the `Connect from Library` sheet.

## Current Code Reality

- `SourcesTab.svelte` renders the visible `Connect from Library` button in the project sources tab.
- `ProjectsShell.svelte` and `src/routes/projects/next/+page.svelte` both host `ConnectFromLibrary` and pass project/library state plus `onConnectSelectedSources`.
- `LibraryAddSourceDialog.svelte` already contains the full add-source experience with provider tabs for YouTube and Telegram.
- `LibraryAddSourceDialog` accepts:
  - `sources`, used by YouTube duplicate detection.
  - `onSourcesChanged(sourceId?: number)`, called after a source is added.
  - `onStatus(message)`, used to surface non-blocking workflow messages.
- `LibraryYoutubeSmartImport.svelte` already detects an existing YouTube source via `existingYoutubeSmartImportSource(sources, preview)`, but currently treats it as a terminal disabled state.
- `LibraryTelegramDialogImport.svelte` calls `addTelegramSource(...)` and then `onSourcesChanged(source.id)`.
- `LibraryYoutubePlaylistImport.svelte` can add multiple playlist videos and returns per-row source IDs through the batch workflow summary.
- The project workflow already connects sources through `addProjectSources({ projectId, sourceIds })`.
- `add_project_sources` is idempotent on the backend via `INSERT OR IGNORE`, but the UI should avoid noisy duplicate actions where it can determine the source is already in the project.

## Scope

In scope:

- Add an `Add source` button wherever the project UI already exposes `Connect from Library`.
- Reuse the existing full `LibraryAddSourceDialog`; do not build a separate provider-specific project dialog.
- Add a project-aware callback path that receives a source ID from the Library dialog and connects it to the active project.
- Keep the standalone Library route behavior unchanged: adding a source there still only adds/refreshes Library.
- For YouTube Smart import duplicates, change the existing Library-only disabled state into a project-aware connect path when the dialog is opened from a project.
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
- Position: adjacent to `Connect from Library`, preferably before it because it is the direct primary action for new material.
- Disabled state: disabled when no project is selected, matching the surrounding project-source actions.

The existing `Connect from Library` action remains the path for browsing and connecting multiple existing Library sources at once.

### Dialog

The button opens the current `LibraryAddSourceDialog` with all existing tabs:

- YouTube
  - Smart import
  - From existing data
- Telegram

The title can remain `Add source`; the behavior differs through project-aware callbacks, not through a separate visual mode.

### Status Copy

Use short status messages in the existing workflow status area rather than modal alerts:

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
5. The project-aware `onSourcesChanged(source.id)` callback refreshes Library/catalog state.
6. The callback calls `addProjectSources({ projectId, sourceIds: [source.id] })`.
7. Project sources refresh.
8. The user sees `Source added and connected to project.`

### Source Already In Library

For YouTube Smart import, duplicate detection already exists after preview. In project mode:

1. The UI displays `Already in Library: <title>` as informational text.
2. The action button changes from the disabled `Already in Library` state to `Connect to project` if the source is not already linked to the current project.
3. Clicking `Connect to project` invokes the same project-aware callback with the existing Library source ID.
4. The user sees `Already in Library. Connected to project.`

If the existing source is already connected to the current project:

1. The UI displays `Already connected to this project.`
2. The action is disabled or omitted.
3. No `addProjectSources` call is made.

### Existing Data / Multiple Adds

`LibraryYoutubePlaylistImport` can add several videos. For v1, keep the existing callback shape simple and source-oriented:

- `onSourcesChanged(sourceId)` is called once for each successful source ID.
- The project-aware callback connects that single source ID to the project.
- Multiple refreshes are intentional for v1 because this preserves the existing dialog contract and avoids a broader callback migration.

Rows skipped because they are already in Library should not be silently ignored in project mode if a source ID is known from the current catalog. They should connect the existing source to the project or report `Already connected to this project`.

### Telegram

Telegram import already returns one `source.id` after `addTelegramSource`. The project-aware callback connects that ID to the project. If `addTelegramSource` returns an existing source for an already-known Telegram dialog, the same callback path handles it.

## Data Flow

Add a small project-context layer above `LibraryAddSourceDialog` rather than moving project logic into provider components.

Recommended shape:

- `LibraryAddSourceDialog` remains reusable and receives optional project-connect context.
- Project screens pass a project-aware `onSourcesChanged` callback.
- The callback owns:
  - refreshing Library catalog data;
  - checking whether the source is already present in current `projectSourceLinks` or `projectSources`;
  - calling `addProjectSources`;
  - refreshing project data;
  - setting status copy.

Provider components should not import project APIs directly. They only report source IDs and render duplicate/connect UI based on props supplied from the dialog/workflow layer.

## Component Changes

Expected touched areas:

- `SourcesTab.svelte`
  - Add `Add source` button next to `Connect from Library`.
  - Add an `onOpenAddSource` prop.

- `ProjectsShell.svelte`
  - Track whether the project add-source dialog is open.
  - Render `LibraryAddSourceDialog` for project context.
  - Provide a project-aware `onSourcesChanged`.
  - Pass current project source IDs into the dialog or duplicate helper path so the UI can detect already-connected sources.

- `src/routes/projects/next/+page.svelte`
  - Mirror the same `Add source` button and project dialog wiring because this route owns a separate `ConnectFromLibrary` instance.

- `LibraryAddSourceDialog.svelte`
  - Accept optional project context without breaking the Library route.
  - Project context includes the active project ID and a set of source IDs already connected to that project.

- `LibraryYoutubeSmartImport.svelte`
  - Keep existing Library duplicate detection.
  - In project mode, allow existing Library sources to be connected to the project.
  - Show `Already connected to this project` when applicable.

- `LibraryYoutubePlaylistImport.svelte`
  - In project mode, map skipped existing rows to catalog source IDs when available and connect those existing sources to the project.
  - Preserve the current standalone Library behavior.

## Error Handling

- Library add failure: keep existing provider-local error handling.
- Project connect failure after Library add success: do not roll back the Library source. Show a status explaining that the source was added but project connection failed.
- Missing project: disable `Add source` and do not open the project dialog.
- Missing source ID from a provider flow: refresh Library and project views, then show a status that the source was added to Library but could not be auto-connected.
- Backend idempotency is a fallback, not the primary UX. If the UI knows a source is already connected, it should not call `addProjectSources`.

## Testing

Add or update focused tests:

- UI contract test: project sources toolbar includes `Add source` next to `Connect from Library`.
- Workflow test: project-mode source add calls `addProjectSources({ projectId, sourceIds: [sourceId] })` after Library add reports a source ID.
- Duplicate test: an existing YouTube Library source can be connected from the Smart import preview in project mode.
- Already-connected test: the project-mode duplicate path shows `Already connected to this project` and does not call `addProjectSources`.
- Existing Library route test: standalone Library add-source behavior remains Library-only and does not import project APIs.
- Multiple-add test: playlist-added source IDs each connect to the project in project mode.

## Acceptance Criteria

- Users can add a YouTube or Telegram source from a project without first going to the Library route.
- After successful add, the source appears in Library and in the active project.
- If a YouTube Smart import source already exists in Library, the user can connect it from the same dialog.
- If the source is already in the project, the UI reports that fact without making a duplicate connect call.
- Existing `Connect from Library` behavior remains unchanged.
- Existing standalone Library `Add source` behavior remains unchanged.
- Tests cover the new project add-source flow and duplicate handling.
