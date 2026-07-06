# Projects Next v11 Source Slice Design

Status: ready for user review
Date: 2026-07-06

## Objective

Bring the current `/projects/next` screen closer to the canonical v11 handoff, limited to the main source-management slice:

- `ProjectToolbar`
- project tabs
- source filter/stats bar
- selected-source bulk bar
- source filter row
- source table

The goal is visual and interaction fidelity for this slice without changing backend behavior, project data flow, source actions, or visible product copy.

## Confirmed Brief

The visual source is `reference/tauri-mcp-bridge-connection/project/design_handoff_research_projects_v11`.

The approved approach is an incremental component slice rather than a full screen rewrite. Sidebar/rail, inspector, and run dock stay out of this iteration. Existing controls remain fully functional: period/prompt/model/run, tabs, filters, add source, connect from Library, selection, sync, delete from Library, remove from project, active row, and table sorting.

Use existing app tokens from `src/lib/styles/base.css` / `--extractum-*` as much as possible. Do not copy hex fallbacks from `.dc.html` files into production styles except where the app already allows fixed brand colors or white text on primary buttons.

Visible copy is not part of this visual slice. Keep the current button/dialog text and keep each visible label, `aria-label`, and `title` synchronized. Full Russian localization or copy normalization is a separate task.

## Reviewed Code Reality

- `/projects/next/+page.svelte` already owns the state and backend calls for this screen.
- `ResearchProjectsShell.svelte` composes the exact slice: `ProjectToolbar`, `ProjectTabs`, `SourcesFilterBar`, `SourcesBulkBar`, `SourcesFilterRow`, and `SourcesGrid`.
- `ProjectToolbar.svelte` already implements the v11-style container-query split, wide selectors, narrow parameters popover, and run button.
- `SourcesGrid.svelte` uses `ExtractumDataGrid` / SVAR DataGrid, which matches the v11 handoff guidance to avoid hand-rolled source tables.
- `research-projects-source-row.ts` already defines sortable columns for source title, type, materials, last sync, and status.
- `SourcesFilterBar.svelte` and `SourcesBulkBar.svelte` have the right functional wiring, but their visual hierarchy and button treatment still feel less like the v11 reference.
- `SourcesFilterBar.test.ts` queries `Add source` and `Connect from Library` by accessible name, so changing visible copy without changing tests and accessibility labels would be a coordinated localization change, not a pure visual update.
- `SourcesBulkBar.svelte` currently mixes Russian bulk actions with English `Delete from Library` copy. This spec treats that as existing product copy and does not make the mixture better or worse in this slice.
- `base.css` already contains the v11 token set: surfaces, borders, text, primary, status, provider colors, density row/control height, radius, and font.
- `--extractum-density-row-height` is currently `34px`, matching the v11 dense-row target.
- There is no saved Product Design user context, so this spec is grounded in the repo and the v11 reference only.

## Design Target

Match the v11 source area as closely as the current production architecture allows:

- dense desktop-app layout, not card-heavy web layout;
- 54px toolbar, 40px tab row, compact filter/bulk band, dense 34px table rows via `--extractum-density-row-height`;
- raised white/surface toolbar and table area over subtle app surfaces;
- token-driven borders, muted labels, primary active states, and warning/success/danger status colors;
- sortable table headers with clear active sort affordance;
- bulk bar overlays the stats/filter bar when rows are selected;
- filter row aligns visually with the configured table columns.

This is a production Svelte/SVAR implementation. The `.dc.html` files, inline prototype markup, and `support.js` are reference-only and must not be ported.

## Scope

### `ResearchProjectsShell.svelte`

Make the main column read as one cohesive v11 source work area:

- keep the current composition and props;
- tighten vertical seams between toolbar, tabs, stats/bulk bar, filter row, and grid;
- ensure the source grid consumes remaining height without page-level scroll leaks;
- keep `SourcesBulkBar` overlay inside the statsbar container;
- avoid layout changes to `ProjectRailPanel`, `Inspector`, and `RunDock`.

### `ProjectToolbar.svelte`

Keep existing behavior and refine only what is needed for v11 fidelity:

- preserve the wide/narrow container-query behavior;
- keep one source of state per selector and current callbacks;
- align spacing, button density, trigger open state, title/eyebrow, and run button treatment with v11;
- do not add new model-loading or prompt-loading behavior.

### `ProjectTabs.svelte`

Align the tab row with v11:

- 40px row;
- compact text, muted inactive state, primary active underline;
- no routing or state-model change.

### `SourcesFilterBar.svelte`

Make the stats/filter band match v11 while preserving existing actions:

- left side: filter button, active chips, reset action, `N из M` counter;
- right side: existing `Add source` and `Connect from Library` actions;
- keep `data-ui-action="add-source"` and `data-ui-action="connect-library"` for tests and automation;
- keep visible text, accessible name, and `title` in the same language for each action;
- use icon+text buttons where an icon exists;
- keep both actions visible because they represent two different workflows in the current app.

Do not rename these buttons in this slice. If a later localization task translates them, it must update visible text, `aria-label`, `title`, and accessible-name tests together.

### `SourcesBulkBar.svelte`

Keep existing behavior and confirmation dialogs, but visually align with v11:

- overlay the stats/filter band;
- show selected count, clear selection, sync, delete from Library, and remove actions;
- preserve disabled-state logic for YouTube-video-only Library deletion;
- preserve current visible copy and confirmation copy, including the existing English `Delete from Library` strings;
- do not change backend calls, confirmation copy, or delete safety semantics in this slice.

### `SourcesFilterRow.svelte`

Keep the current filter model and align the row to the source table:

- use one shared source-table layout contract for both the filter row and DataGrid column definitions;
- retain search, type, material range, date range, and status filters;
- keep status options exactly `active | syncing | error | unavailable`;
- preserve client-side filtering in `/projects/next/+page.svelte`.

The implementation plan must not rely on reading live SVAR column positions from the DOM. Instead, define the source table geometry once in UI code and consume it from both sides:

- numeric widths for SVAR columns: select `34`, type `116`, materials `116`, last sync `150`, status `104`;
- title column as the only flexible column: `minmax(0, 1fr)` in the filter row, `flexgrow: 1` in SVAR;
- a derived filter-row grid template equivalent to `34px minmax(0, 1fr) 116px 116px 150px 104px`.

This makes alignment deterministic for the configured grid. Perfect runtime alignment after user-driven column resizing is out of scope because the current DataGrid does not expose those live widths to `SourcesFilterRow`.

### `SourcesGrid.svelte` / Source Table

Use SVAR DataGrid, not a hand-rolled CSS-grid table:

- preserve row selection, active row, row click, multi-select, select-all, sorting, and date formatting;
- keep raw date/time values plus `dateTimeFormat`, in line with `AGENTS.md`;
- tune DataGrid theme variables or wrapper styles to approximate v11 header/row density and borders;
- consume the same source-table layout constants used by `SourcesFilterRow`;
- keep column semantics and sort comparators from `research-projects-source-row.ts`.

If SVAR cannot reproduce one small v11 visual detail exactly, prefer a stable production grid over bespoke markup.

## Out Of Scope

- Redesigning the left rail/project list.
- Redesigning the inspector.
- Redesigning the run dock.
- Adding new backend commands or changing source sync/delete semantics.
- Changing Library dialogs or add/connect workflows beyond visual entry points in the source slice.
- Translating or normalizing visible copy across the slice.
- Supporting filter-row alignment after user-driven DataGrid column resizing.
- Persisting filters, sort state, or selected rows across projects/sessions.
- Replacing SVAR DataGrid with custom table markup.
- Loading model options from profiles.

## Token And Styling Rules

- Prefer existing `--extractum-*` tokens from `base.css`.
- Prefer `color-mix(...)` with existing tokens for tints instead of new literal colors.
- Use `--extractum-density-control-height`, `--extractum-density-row-height`, `--extractum-radius`, `--extractum-font`, and the existing surface/border/text/status/provider tokens.
- `--extractum-density-row-height` is the row-height target for this slice; do not introduce a second hard-coded row-height value.
- Do not introduce a one-off palette or new theme file.
- Do not add decorative gradients/orbs/background art.
- Keep text within button and table cell bounds at desktop and narrow widths.
- Use existing icon patterns/lucide-style SVG usage already present in this area; do not introduce a new icon dependency for this slice.

## Interaction Contract

The following must keep working after the visual update:

- selecting a project loads sources and data range;
- toolbar period/prompt/model selections update route state;
- run button remains disabled under the existing conditions;
- tabs switch sections and show the existing empty-state text for non-source sections;
- filter button toggles the filter row;
- chips remove their matching filters;
- clear resets all filters;
- `Add source` opens `LibraryAddSourceDialog`;
- `Connect from Library` opens `ConnectFromLibrary`;
- table sorting works by clicking source, type, materials, last sync, and status headers;
- row click updates the inspector selection;
- row checkboxes and header checkbox update selection;
- selected rows show the bulk bar;
- sync selected sources syncs YouTube video sources with metadata, transcripts, and comments;
- delete from Library remains limited to exactly one selected YouTube video source;
- remove deletes only the project membership.

## Testing And Verification

Implementation plan should include:

- focused Vitest coverage for any changed component contracts, especially stable `data-ui-action` hooks and accessible names;
- focused tests for source-grid column behavior if table theme or column definitions change;
- a unit-level assertion that the filter-row layout template and DataGrid column widths come from the same source-table layout contract;
- `npm.cmd run check` after Svelte/TypeScript changes;
- Svelte MCP/autofixer pass during implementation after component edits;
- Tauri MCP visual verification on the live `/projects/next` screen at the current desktop viewport, including:
  - no overlap between toolbar controls, tabs, filter/bulk bar, and table;
  - source table headers remain clickable and sorting changes row order;
  - selecting rows swaps the stats bar for the bulk bar;
  - filter row aligns with the configured table columns for the current SVAR grid;
  - add/connect/sync/delete/remove entry points remain reachable.

## Acceptance Criteria

- The first viewport of `/projects/next` visually reads as the v11 source-management slice: dense toolbar, compact tabs, v11-like source controls, and a coherent source table.
- No source workflow regresses.
- Existing visible copy, accessible names, and titles stay synchronized for source-slice actions.
- No new backend behavior is introduced.
- No `.dc.html` runtime/prototype code is copied.
- The implementation remains token-driven and uses existing Extractum/SVAR/shadcn patterns.
