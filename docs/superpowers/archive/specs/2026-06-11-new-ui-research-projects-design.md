# New UI Research Projects Design

Date: 2026-06-11

## Goal

Design a new Extractum interface from scratch around research projects, evidence, and a global source library, using a dense Ultra HD desktop control-deck layout.

This spec defines the product model, information architecture, visual layout, component-library boundaries, transition adapter approach, and first implementation slice. It does not start implementation; it is the design contract for the next planning step.

## Design Brief

Build the new UI as a desktop-first Tauri workspace optimized for Ultra HD / 4K screens. The interface should not be a component-by-component migration of the current UI. It should rethink IA and UX around:

- `Research Projects` as the first screen and main user context.
- `Evidence` as the central working object inside a project.
- `Library` as a global source concept, initially backed by current source data through an adapter.
- `Project Dashboard + Library Connect` as the first vertical slice.

The chosen implementation approach is `Transition Adapter`: introduce the new product language and UI contracts first, map them to existing sources/source groups/runs underneath, and defer heavy backend schema migration.

## Visual References

- Live reference inspected at `http://127.0.0.1:5173/`, title `Research Control Deck`.
- User-provided Ultra HD mockup showing:
  - narrow icon rail;
  - project rail;
  - top command bar with period, prompt preset, model, run and export actions;
  - project workspace tabs;
  - dense sources table;
  - large `Connect from Library` dialog/sheet with filters, source table, project filters, change log, and primary action.

These references define visual density, layout rhythm, and interaction scale. They do not lock Extractum to the same exact tab structure or sample domain data.

## Product Principles

- Treat Extractum as a research control deck, not a marketing site or generic admin dashboard.
- Prefer dense, scannable, bordered work surfaces over large cards and decorative panels.
- Keep the first-class product language clean: project, library, source, evidence, report, run.
- Use old implementation concepts such as source groups only behind the adapter layer.
- Make source connection and evidence readiness visible before report generation.
- Keep the current UI functional as fallback until the new UI is ready for cutover.

## Product Model

### Research Project

A `Research Project` is the primary workspace container. It represents a topic or investigation and owns the visible research context:

- title and optional description;
- period;
- prompt preset;
- selected model;
- connected sources;
- evidence and material counts;
- reports;
- runs and job status.

In the first slice, a project can be mapped from current source groups or synthetic workspace state. The UI should still present it as a project.

### Library Source

A `Library Source` is a global source that can be connected to one or more projects. It can represent Telegram, YouTube, forums, RSS, or Web sources.

In the first slice, library sources are projected from existing source records. A new durable `library_sources` table is out of scope.

### Project Source Link

A `Project Source Link` connects a library source to a project and stores project-specific filtering intent:

- period;
- material types;
- include comments;
- include transcripts;
- include/exclude tags;
- local project constraints;
- connection status.

The UI should make the link concept visible even if the initial implementation maps it to existing source/source-group relationships.

Transition constraint: the current durable source-group API supports provider-scoped groups for Telegram and YouTube. The first slice must not imply durable cross-provider project membership where the backend cannot safely persist it. Library sources for RSS, forums, Web, or other future providers can appear in the library inventory, but their connect rows must be disabled with a clear reason until the transition adapter has a safe persistence path.

## Information Architecture

### App-Level Shell

The app shell should have two left-side navigation layers:

- Icon rail: `Projects`, `Library`, `Runs/Activity`, `Diagnostics`, `Settings`.
- Project rail: project search, all projects, favorites/archive, current project list, source summary.

`Research Projects` is the first screen after launch. Opening a project enters the project workspace.

### Project Workspace

Project-level tabs:

- `Overview`
- `Sources`
- `Evidence`
- `Reports`
- `Runs`
- `Prompts`

The first implementation slice focuses on `Overview` enough to orient the user and `Sources` enough to connect library sources to a project.

### Sources Tab

The `Sources` tab should include:

- connected source summary;
- provider/status filters;
- project source grid or dense list;
- source readiness states;
- action `Add source` / `Connect from Library`;
- bottom queue visibility for sync/LLM jobs.

### Connect From Library

`Connect from Library` is a large working dialog/sheet, not a small modal. It should preserve background context while offering enough space for table work.

It contains:

- source search;
- provider filter chips;
- SVAR grid with selectable sources;
- columns for source, type, projects, last collection, local copy, status;
- selected count;
- project filter panel;
- existing-source explanation;
- change log panel;
- primary action `Connect selected`;
- secondary actions such as cancel and global settings.

## Ultra HD Layout

Primary target: desktop Tauri app on Ultra HD / 4K displays.

Recommended persistent regions:

- left icon rail, fixed width;
- project rail, fixed or resizable width;
- top command bar, fixed height;
- central workspace, tab-driven;
- optional right inspector/sheet for prompt, memory, source detail, or connect workflow;
- pinned bottom queue for jobs and run status.

The design can use more simultaneous panels than a responsive web app because mobile/tablet are not first-class targets for this workstream.

For smaller desktop windows, panels may collapse or hide behind drawers. Mobile support is out of scope except avoiding catastrophic layout breakage.

## Component Boundary

Use a clear boundary between shadcn-svelte and SVAR.

### Extractum UI Ownership

Extractum owns the product visual system. shadcn-svelte and SVAR provide behavior, accessibility primitives, widget anatomy, and data-grid capability, but application screens should not depend on their raw components directly.

Application and prototype screens should import product-facing components from an Extractum wrapper layer. The wrapper layer owns:

- compact desktop density;
- spacing and sizing;
- typography;
- active, selected, hover, disabled, and focus states;
- surface grammar;
- stable CSS hooks and data attributes;
- provider/status anatomy;
- integration with project-level tokens.

Recommended layering:

1. `src/lib/styles/base.css` or an equivalent project style entry defines product tokens: color, radius, typography, focus, selected states, hover states, surface grammar, and compact desktop density.
2. `src/lib/components/ui/*` contains shadcn-svelte primitives and shadcn-compatible generated/local code.
3. `src/lib/components/extractum-ui/*` contains product wrappers used by app and prototype screens.
4. `src/lib/components/research-projects/*` contains feature components that depend on `extractum-ui` and the new view-model layer.
5. SVAR widgets are introduced through product wrappers, not directly from feature screens.
6. Feature CSS may tune layout for a concrete view, but must not become the source of shared component appearance.

Allowed direct imports from shadcn-svelte or SVAR should be limited to wrapper components, low-level component tests, and short-lived experiments. The implementation plan should add contract tests or raw-source checks to keep feature screens from importing lower-level libraries directly.

### shadcn-svelte

Use shadcn-svelte for app shell primitives and interaction controls:

- buttons;
- inputs and search;
- selects;
- checkboxes;
- badges;
- tabs;
- dialog/sheet;
- dropdowns;
- tooltips;
- toasts;
- forms;
- command/search palette if needed.

Generated or copied shadcn-svelte code should stay in the `ui` namespace. Product components should be exported from `extractum-ui`, even when they wrap a single shadcn primitive.

When shadcn primitives are wrapped for product use:

- preserve existing CSS hooks, class names, data attributes, dimensions, and active-state selectors needed by the product shell;
- reset shadcn defaults at the wrapper boundary when default margins, radius, transitions, or display styles could change dense workspace layout;
- keep application imports clear by path, for example feature code imports `ExtractumButton` or `ProjectTabs` from `extractum-ui`, not shadcn internals;
- keep shadcn blocks out of product architecture decisions. Do not replace the app shell, source viewer, runbar, inspector, drawers, or modal architecture with generic shadcn blocks.

The current custom components under `src/lib/components/ui/*` are not the product foundation for the new UI. They remain available for the old UI during the first slice. shadcn-generated primitives should use the default lower-case shadcn component directories under the same namespace, while new feature code imports only `extractum-ui` wrappers.

### SVAR

Use SVAR for dense data-heavy work surfaces:

- library sources grid;
- project sources grid;
- evidence inventory;
- materials grid;
- runs queue;
- activity/change log.

The initial package focus should be `@svar-ui/svelte-grid`. Additional SVAR packages should be introduced only when a concrete UI need appears.

SVAR components must be wrapped in product components before use by screens. Initial wrapper candidates:

- `DataGrid.svelte` for SVAR Grid with Extractum sizing, selection, empty states, and theme bridge.
- `GridToolbar.svelte` only if SVAR Toolbar is needed after the first grid pass.
- `FilterBuilder.svelte` only if project/source filtering outgrows shadcn controls.

SVAR customization should prefer:

- scoped theme CSS variables;
- wrapper-provided sizing and height containers;
- stable row ids from adapter data;
- documented `rowStyle`, `columnStyle`, `cellStyle`, header/footer `css`, and theme hooks;
- custom cell/header/body components when rows must match Extractum provider/status anatomy.

Important SVAR Grid constraints:

- the grid root uses `height: 100%`, so the product wrapper must provide a stable height;
- grid rows should receive stable `id` values from the adapter to avoid generated temporary ids;
- selection state should be synchronized through the wrapper instead of being read ad hoc from feature screens;
- checkbox cells should avoid accidental row-click selection by using the documented ignore-click pattern;
- wrapper props should expose project vocabulary such as selected source ids and provider filters rather than raw SVAR action names;
- direct `.wx-*` selector overrides are allowed only inside the wrapper/theme bridge and should be narrowly scoped.

SVAR theme integration should import the widget theme from the widget package, for example `Willow` or `WillowDark` from `@svar-ui/svelte-grid`, because widget themes add package-specific variables. If Extractum owns fonts, pass `fonts={false}` and map SVAR font variables to Extractum typography tokens.

SVAR localization should be handled at the wrapper or app-shell boundary. If SVAR displays static UI strings in the new Russian-first interface, merge the relevant package locale with core locale data instead of leaving widget fallbacks in English.

### Icons And Tokens

- Use `@lucide/svelte` for general command icons.
- Use provider-specific marks where they improve scanning.
- Use Tailwind and CSS variables for shared theme tokens.
- Add a SVAR theme bridge that maps SVAR Willow/WillowDark variables to Extractum tokens.
- Keep token naming product-owned. Library variables should be mapped from Extractum tokens, not treated as the source of truth.

## Data And Adapter Flow

Add a pure adapter/view-model layer for the new UI. Suggested shape:

```ts
export type ResearchProjectView = {
  id: string;
  title: string;
  description: string | null;
  periodLabel: string;
  sourceCount: number;
  evidenceCount: number;
  materialCount: number;
  lastRunLabel: string | null;
  status: "ready" | "running" | "needs_attention" | "empty";
};

export type LibrarySourceView = {
  id: string;
  provider: "telegram" | "youtube" | "forum" | "rss" | "web" | "other";
  title: string;
  subtitle: string | null;
  projectCount: number;
  lastCollectedLabel: string | null;
  localCopyLabel: string | null;
  status: "active" | "needs_account" | "syncing" | "error" | "unavailable";
  disabledReason: string | null;
  alreadyConnected: boolean;
};

export type ProjectSourceLinkView = {
  projectId: string;
  sourceId: string;
  provider: LibrarySourceView["provider"];
  title: string;
  connectionStatus: "connected" | "pending" | "failed" | "already_connected";
  filterSummary: string;
};
```

The adapter can assemble these views from existing APIs:

- analysis sources;
- source groups;
- source jobs;
- YouTube summaries;
- analysis runs;
- workspace state.

The route and component tree should consume the new view-model types so old terms do not leak into the UI.

Project persistence in the first slice should be explicit:

- projects backed by an existing Telegram or YouTube source group may persist connected source changes through `updateAnalysisSourceGroup`;
- projects without a safe source-group mapping may show library inventory and project shell state, but `Connect selected` must be disabled with clear copy;
- unsupported provider rows must not silently pretend to connect.

## Error And Status Handling

`Connect from Library` must handle:

- source already connected to project;
- source unavailable because account/settings are missing;
- active sync or import jobs;
- provider-specific errors;
- provider not supported by the current project-link persistence path;
- partial connect success;
- no sources matching filters;
- empty library.

Rows should show disabled state and reason instead of allowing invalid actions. The primary action should count only connectable selected rows.

The bottom queue should aggregate active source jobs and LLM jobs in a consistent compact format.

## First Implementation Slice

The first slice is `Project Dashboard + Library Connect`.

In scope:

- Set up Tailwind/shadcn-svelte structure and required primitives.
- Add SVAR Grid and theme bridge.
- Add the new `/projects` UI route/shell without removing the current Analysis route.
- Add transition adapter/view-model module for projects and library sources.
- Render project rail and top command bar in the new shell.
- Render project `Sources` tab with connected source summary and source list/grid.
- Implement `Connect from Library` dialog/sheet with:
  - search;
  - provider filters;
  - SVAR grid;
  - multi-select;
  - project filters panel;
  - selected count;
  - change log/status panel;
  - connect action that persists only safe Telegram/YouTube source-group-backed links and explains refused provider/project combinations.
- Preserve old UI as fallback.

Out of scope:

- Full Evidence Workspace.
- Full report builder replacement.
- New durable database schema for projects/library.
- Full mobile/tablet responsive redesign.
- Full replacement of Accounts, Diagnostics, Settings.
- Full backend migration from source groups to projects.
- Durable cross-provider project membership for providers not supported by the current source-group API.

## Architecture Notes

Suggested file boundaries:

- `src/lib/ui/research-projects-model.ts` for pure view-model types and adapters.
- `src/lib/styles/base.css` or equivalent for new UI product tokens.
- `src/lib/components/ui/*` for shadcn-svelte primitives and shadcn-compatible generated/local code.
- `src/lib/components/extractum-ui/*` for product wrappers over shadcn and SVAR.
- `src/lib/components/research-projects/*` for feature components.
- `src/routes/projects/+page.svelte` for the first new UI route.

Use `src/lib/ui/*` for product UI contracts, view models, adapters, state helpers, and other non-component UI logic. Keep `src/lib/components/ui/*` reserved for shadcn-svelte primitives so the two namespaces stay visually similar but semantically distinct.

The current `/analysis` experience stays available while the new UI is built.

Keep existing PascalCase legacy components in `src/lib/components/ui/*` for the old UI during the first slice. Add shadcn-generated primitives in the default lower-case shadcn component directories under `src/lib/components/ui/*`, for example `src/lib/components/ui/button/*`. New feature code must import product wrappers from `extractum-ui`; it must not import legacy PascalCase components or raw shadcn primitives directly.

## Testing Strategy

Use tests proportional to the blast radius:

- adapter/view-model unit tests for project and library projections;
- contract tests for route composition and component prop threading;
- component/source tests for `Connect from Library` selection state;
- raw-source import-boundary tests so feature screens use `extractum-ui` wrappers rather than direct shadcn/SVAR imports;
- focused tests for SVAR grid wrapper selection, disabled rows, and stable row ids;
- focused Playwright CLI or Tauri QA for the visual workflow once implemented;
- `npm.cmd run check` before claiming implementation completion.

For this design phase, no source-code verification is required beyond writing and reviewing the spec.

## Acceptance Criteria For First Slice

- User can open the new project dashboard route.
- User sees a dense Ultra HD-oriented shell with icon rail, project rail, top command bar, and project workspace.
- User can open a project and reach the `Sources` tab.
- User can open `Connect from Library`.
- Library dialog/sheet shows searchable/filterable source inventory in SVAR grid.
- User can select multiple connectable sources and see selected count.
- Already-connected and unavailable sources are visibly distinct and not connectable.
- Unsupported provider rows are visible when present, but disabled with explicit persistence-copy instead of pretending to connect.
- Project filters are visible in the connect workflow.
- Connect action persists Telegram/YouTube source-group-backed project links through the current API and refuses unsafe provider/project combinations with visible reasons.
- Feature screens import product wrappers instead of raw shadcn/SVAR components.
- SVAR grids are mounted through Extractum wrappers with stable height, stable row ids, and token-mapped theme variables.
- Old Analysis route remains functional.

## Self-Review

- No placeholder requirements remain.
- The design is scoped to one implementation plan: foundation plus Project Dashboard and Library Connect.
- The spec does not require immediate backend schema migration.
- The component boundary between shadcn-svelte and SVAR is explicit.
- The Extractum wrapper layer owns product appearance and prevents direct library imports from feature screens.
- Ultra HD desktop is the primary target; mobile is intentionally out of scope.
- The visual references are described by layout and density rather than copied as fixed tab/content requirements.
