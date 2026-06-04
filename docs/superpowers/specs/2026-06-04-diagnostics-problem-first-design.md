# Diagnostics Problem-First Layout Design

Date: 2026-06-04
Status: active design
Scope: `/diagnostics` frontend layout and tests only

## Goal

Make Diagnostics feel problem-first when the operator chooses `Only issues`.
The current issue filtering works, but the first issue table can sit far below
the first viewport because healthy overview cards still occupy the top of the
page. The next follow-up should make issue details visible immediately without
removing the full diagnostic overview.

## Approved Approach

Use mode-dependent section order:

- In `Only issues`, render issue table sections directly after the table mode
  controls.
- In `All tables`, keep the current report-like order: health strip, summary
  cards, then tables.
- Keep the health and privacy overview available in both modes. In issue mode,
  move it below the issue tables as supporting context.

This keeps `Only issues` focused on action and preserves `All tables` as the
complete local health summary.

## Architecture

Keep ownership inside the existing Diagnostics route and table component:

- `src/routes/diagnostics/+page.svelte` owns section ordering and mode state.
- `src/lib/diagnostics-view-model.ts` keeps the existing issue-row helpers.
- `src/lib/components/diagnostics/DiagnosticCountTable.svelte` remains the
  reusable table renderer and does not learn page-level layout policy.

No backend commands, diagnostics payload shape, database behavior, or Tauri IPC
contracts should change.

## Components And Layout

The route should split Diagnostics content into three conceptual blocks:

- `diagnostics-table-controls`: unchanged controls for `Only issues` and
  `All tables`.
- `diagnostics-table-area`: table sections filtered by the selected mode.
- `diagnostics-overview-area`: status strip, summary cards, and privacy
  boundary.

The order depends on mode:

- `Only issues`: controls, table area, overview area.
- `All tables`: controls, overview area, table area.

If issue mode has no issue rows, the empty state should appear in the table
area immediately under the controls. The overview remains below it.

## Data Flow

The route already computes table sections and filters rows with
`visibleDiagnosticRows`. This follow-up should reuse that logic and only change
where the resulting table area appears.

The existing `totalRows` behavior should remain, so issue mode continues to
show filtered counts such as `1/2 rows`.

## Error Handling

This is a layout follow-up. Existing loading, refresh, backend error, and empty
diagnostics behavior should not change.

The route should not hide healthy overview data in issue mode. Operators still
need the full context when reviewing local health.

## Testing

Add or update raw-source and view-model tests around the route contract:

- `Only issues` places the diagnostics table area before the overview area.
- `All tables` preserves the overview-before-tables report order.
- Issue mode still filters healthy rows from mixed tables.
- Empty issue mode renders the table-area empty state before overview context.

Run focused Diagnostics tests, then `npm.cmd run test` and `npm.cmd run check`
after implementation.

## Acceptance Criteria

- At normal desktop width, `Only issues` shows issue table sections in the first
  viewport when issue rows exist.
- At about `900px` width, the first issue table is no longer pushed below all
  healthy overview cards.
- `All tables` continues to show the complete diagnostic overview before the
  complete table report.
- No horizontal overflow appears at checked desktop or narrow widths.
- No backend or data model behavior changes.
