# Library Prototype Design

Date: 2026-06-13

## Goal

Design the first Library prototype as a separate work screen at `/projects/library` inside the new projects shell.

The prototype validates the layout, navigation model, source filtering, source selection, and contextual command area. It does not implement full source CRUD or a new durable library schema.

## Confirmed Brief

- `Library` opens as its own nested route: `/projects/library`.
- The existing `IconRail` remains the shared left navigation for `/projects/*`.
- `ProjectRail` remains only on the `Projects` screen.
- The Library screen replaces `ProjectRail` with a source filter tree.
- The first prototype is fast and mostly static, but it includes several real interactions:
  - route-level navigation to Library;
  - collapsible left filter rail from `240px` to `64px`;
  - source table filtering by selected filter tree row;
  - source row selection;
  - Inspector content for the selected source;
  - draggable Inspector width from `380px` to `500px`.
- CRUD commands `Add`, `Edit`, `Delete`, and `Refresh` are visible.
- `Add` is a placeholder for a future add-source flow.
- Source rows should use real data from the current adapter/view-model as far as it already supports.

## Chosen Approach

Use `Nested Route Workbench`.

`/projects` and `/projects/library` share the same app-level shell, but each route owns its second navigation rail and workspace:

- `/projects`: `IconRail + ProjectRail + ProjectWorkspace`.
- `/projects/library`: `IconRail + LibraryFilterRail + LibraryWorkspace + LibraryInspector`.

This is slightly more work than branching inside the current `ProjectsShell`, but it keeps Projects and Library responsibilities separate from the start.

## Non-Goals

- Do not rename `/projects` to `/sources`.
- Do not replace the current `/analysis` UI.
- Do not create a new durable `library_sources` table.
- Do not implement full Add/Edit/Delete backend mutations.
- Do not implement full YouTube channel support if the backend does not expose it yet.
- Do not make RSS/forum/Web connect behavior look fully supported when persistence is not ready.

## Layout

### App Shell

The route family `/projects/*` should have a shared shell around `IconRail`.

`IconRail` should show `Library` as active on `/projects/library` and `Projects` as active on `/projects`.

### A. Left Panel

Component: `LibraryFilterRail`.

Width:

- expanded: `240px`;
- collapsed: `64px`;
- fixed width in both states;
- no content-driven resizing.

Contents:

- tree filter built with the new `ExtractumTreeDataGrid` wrapper;
- collapse/expand control;
- initially selected row: `All sources`;
- first hierarchy:
  - `All sources`;
  - `YouTube`;
  - `YouTube / Videos`;
  - `YouTube / Playlists`;
  - `YouTube / Channels`;
  - `Telegram`.

Data constraint:

Current `AnalysisSourceOption` exposes `source_type`, but not `source_subtype`. The first prototype can filter real data reliably by top-level provider (`youtube`, `telegram`, etc.).

For the first prototype, YouTube subtype rows (`Videos`, `Playlists`, `Channels`) should appear as disabled planned-refinement rows. They should show the intended taxonomy without implying that subtype filtering is already backed by the adapter. The disabled reason should explain that subtype filtering requires source subtype metadata.

### B. Main Panel

Component: `LibraryWorkspace`.

Width:

- dynamic;
- calculated by CSS grid as remaining space after `IconRail`, `LibraryFilterRail`, Inspector handle, and Inspector panel.

Contents:

- compact top toolbar;
- search input;
- command buttons:
  - `Add`;
  - `Edit`;
  - `Delete`;
  - `Refresh`;
- source table using the existing `ExtractumDataGrid` wrapper;
- empty state when the selected filter returns no sources.

The table should use real `LibrarySourceView[]` rows from the current workflow and adapter. It should not invent a separate mock source model.

Minimum table columns:

- source title and subtitle;
- provider/type;
- status;
- project count;
- local copy/material count;
- last collected.

### C. Right Context Panel

Component: `LibraryInspector`.

Width:

- default: `380px`;
- user-resizable through a drag handle;
- minimum: `380px`;
- maximum: `500px`;
- width should persist only in local component state for the prototype.

Contents are bound to the selected source row, not to the selected filter row.

First Inspector sections:

- source title;
- provider/type badge;
- status badge;
- source id;
- project count;
- local copy/material count;
- last collected;
- disabled/connectability reason when present;
- contextual command buttons such as `Open`, `Sync`, `Connect`, and `Run report`.

When no source is selected, the Inspector should show a neutral empty state prompting row selection.

When no source is selected, source-specific toolbar commands such as `Edit` and `Delete` must be disabled.

## Component Boundaries

### Shared Route Components

Recommended structure:

- `src/routes/projects/+layout.svelte` for shared `IconRail` shell.
- `src/routes/projects/+page.svelte` for the Projects screen.
- `src/routes/projects/library/+page.svelte` for the Library screen.

The exact route extraction can be adjusted during planning, but the design intent is that Library is not implemented as an internal branch of `ProjectWorkspace`.

### Extractum Wrappers

Library screen code must use product wrappers from `src/lib/components/extractum-ui`.

New wrapper:

- `ExtractumTreeDataGrid`.

Existing wrapper:

- `ExtractumDataGrid`.

Wrapper constraints:

- SVAR Grid must not be imported directly by Library feature screens.
- `ExtractumTreeDataGrid` owns stable height, stable row ids, wrapper-managed selection, empty state, theme bridge, and scoped `.wx-*` overrides.
- Tree rows must use stable string ids such as `all`, `provider:youtube`, `provider:youtube/subtype:video`.
- Feature screens communicate with the tree wrapper through typed props and callbacks, not raw SVAR events.

## View Model

Reuse the current research projects workflow data for the prototype:

- `sources`;
- `groups`;
- `runs`;
- `sourceJobs`;
- derived `librarySources`.

The Library prototype can reuse `buildLibrarySourcesView` for the source table.

Add a small Library-specific view model layer if needed:

```ts
export type LibraryFilterTreeRow = {
  id: string;
  label: string;
  provider: LibrarySourceProvider | "all";
  subtype?: "video" | "playlist" | "channel";
  count: number;
  disabled?: boolean;
  disabledReason?: string;
  data?: LibraryFilterTreeRow[];
};
```

Filtering rules:

- `all` returns all library sources;
- `provider:youtube` returns `source.provider === "youtube"`;
- `provider:telegram` returns `source.provider === "telegram"`;
- YouTube subtype filters require subtype data and should be disabled or empty until available.

## Interactions

### Route Navigation

Clicking `Library` in `IconRail` navigates to `/projects/library`.

Clicking `Projects` navigates to `/projects`.

### Filter Selection

Selecting a tree row updates the visible source table.

The selected filter state is local to the Library screen for the prototype.

### Tree Collapse

Expanded mode shows tree labels and counts.

Collapsed mode shows compact provider tokens/icons and should preserve the selected filter.

Collapsed mode must remain keyboard and tooltip accessible in the implementation plan.

### Table Selection

Selecting a table row updates `LibraryInspector`.

If the current filter removes the selected source from the table, selection should move to the first visible row. If the filtered table is empty, selection becomes empty and the Inspector shows its neutral empty state.

### Inspector Resize

The vertical handle between the table and Inspector supports pointer drag.

The implementation clamps width to `380-500px`.

The prototype may keep the width in component state without persistence.

### CRUD Buttons

For the first prototype:

- `Add` opens no real flow and can show placeholder feedback.
- `Edit` is disabled unless a source is selected.
- `Delete` is disabled unless a source is selected.
- `Refresh` can reload the current workspace data if that is cheap, or remain a visible placeholder with status feedback.

## Accessibility

The first implementation plan should treat keyboard access as part of the prototype, not as later polish.

Minimum expectations:

- `IconRail` links are reachable with `Tab` and expose clear accessible labels.
- The filter tree can be traversed with keyboard focus; selected and disabled rows expose their state.
- Collapsed filter tokens have accessible names and tooltips.
- The source table supports keyboard navigation between rows with arrow keys where the grid wrapper supports it.
- `Enter` or `Space` selects the focused source row when supported by the wrapper.
- Toolbar buttons are reachable with `Tab`; disabled states are exposed as real disabled controls.
- The Inspector resize handle has an accessible label and keyboard fallback if practical for the first slice.
- Focus order should move predictably from `IconRail` to filter rail to table toolbar/table to Inspector commands.

## Styling

- Desktop-first dense control-deck layout.
- Use full-height bordered work surfaces, not nested cards.
- Keep panels aligned to CSS grid tracks so collapse and resize do not shift unrelated regions.
- Use icons from `@lucide/svelte` through feature components where needed.
- Keep SVAR `.wx-*` overrides inside `ExtractumTreeDataGrid` or existing grid wrapper/theme bridge.
- Avoid introducing a new color palette for this prototype.

## Testing And Verification

Minimum tests for the implementation plan:

- route smoke test for `/projects/library`;
- component test for Library filter tree selection;
- component test for Inspector width clamping;
- component/source import-boundary test preventing direct SVAR imports from Library feature screens;
- component/source import-boundary test preventing direct shadcn imports from feature screens;
- existing `ExtractumDataGrid` tests continue to pass.

Manual verification:

- open `/projects`;
- open `/projects/library` through `IconRail`;
- confirm `ProjectRail` is gone on Library and present on Projects;
- collapse and expand the Library filter rail;
- select provider filters;
- confirm YouTube subtype rows are disabled until subtype metadata is available;
- select source rows and confirm Inspector changes;
- change filters and confirm selection moves to the first visible source or becomes empty when no rows remain;
- drag Inspector width and confirm it clamps to `380-500px`;
- confirm `Edit` and `Delete` are disabled when no source row is selected;
- keyboard through `IconRail`, filter rail, toolbar, table, and Inspector commands;
- check desktop and narrower laptop-width viewports for text overflow and panel overlap.

## Acceptance Criteria

- `/projects/library` is a separate route.
- `IconRail` active state reflects the current route.
- Library screen uses a `240px` collapsible filter rail and a `380-500px` resizable Inspector.
- `ProjectRail` appears only on `/projects`, not on `/projects/library`.
- The source table uses real `LibrarySourceView` rows from the current adapter.
- Filter tree selection affects table rows.
- YouTube subtype filter rows are disabled in the first prototype until subtype metadata is exposed.
- Table row selection affects Inspector content.
- Filter changes select the first visible source when possible.
- `Edit` and `Delete` are disabled when no source is selected.
- CRUD command buttons are visible with safe prototype behavior.
- Core interactive regions are keyboard reachable.
- SVAR Grid and tree usage goes through Extractum wrappers.
- The prototype does not imply unsupported durable backend capabilities.
