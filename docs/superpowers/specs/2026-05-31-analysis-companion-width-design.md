# Analysis Companion Width Design

Date: 2026-05-31
Status: ready for review

## Context

The analysis workspace uses a three-column desktop grid:

- compact source rail;
- primary canvas;
- run companion panel with Evidence, Chat, Chunks, and Runs tabs.

On a 1920px viewport observed through the running Tauri app, the grid measured
approximately:

- workspace: 1640px;
- canvas: 1105px;
- companion: 430px;
- Evidence content after panel padding: about 350px;
- Evidence trace layout: about 160px for the ref list and 177px for details.

That makes the Evidence tab feel cramped even on a wide desktop. The global
companion column is narrow, and the Evidence tab switches to a two-column layout
based on viewport width instead of the actual panel width.

## Goal

Make the run companion, especially the Evidence tab, readable on desktop without
turning the whole workspace into a modal or redesigning the analysis surface.

The first slice should improve the default layout with conservative CSS/state
changes:

- give the companion more useful width on large screens;
- let Evidence use two columns only when the companion has enough room;
- keep smaller screens and stacked layouts predictable.

## Scope

In scope:

- desktop analysis workspace column sizing;
- Evidence tab list/detail reflow;
- targeted tests or raw component contracts for the layout thresholds;
- no changes to Evidence data, trace selection, or source navigation behavior.

Out of scope:

- resizable drag handles;
- persisted user-controlled companion width;
- a full companion focus mode;
- moving Evidence into a modal;
- changing Chat, Chunks, or Runs behavior beyond inheriting the wider companion
  column.

## Proposed Design

Use a hybrid of the two simplest layout fixes:

1. Widen the companion column on large screens.
2. Reflow the Evidence tab based on available panel width, not broad viewport
   width alone.

The current workspace rule:

```css
grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1.6fr) minmax(320px, 430px);
```

should move to a wider but still bounded companion column, for example:

```css
grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1.45fr) minmax(420px, 560px);
```

or an equivalent `clamp(...)` shape if it fits the existing CSS style better.
The important contract is that wide desktop gets a companion panel around
520-560px instead of 430px, while the canvas remains the dominant workspace
surface.

The existing breakpoint that turns Evidence into two columns at viewport
`min-width: 1280px` is too blunt. It should become panel-aware. The preferred
implementation is a container query on the Evidence tab root:

```css
.run-evidence-tab {
  container-type: inline-size;
}

@container (min-width: 34rem) {
  .trace-layout {
    grid-template-columns: minmax(12rem, 0.9fr) minmax(16rem, 1.1fr);
  }
}
```

If container queries are not already acceptable in the project, use a local
class or CSS custom property driven by the companion surface width. Do not keep
the current viewport-only rule if it can still create two unreadably narrow
columns inside the companion.

## User Experience

On wide desktop:

- the companion panel is visibly wider;
- Evidence can show the ref list and detail side by side;
- the detail quote has enough width to read without severe wrapping;
- the canvas still has enough space for Source and Report review.

On medium widths:

- the companion remains under the canvas according to the existing workspace
  breakpoint;
- Evidence can stack list above detail if the panel width is not enough for
  two useful columns.

On narrow/mobile widths:

- the existing single-column workspace behavior remains;
- Evidence uses the stacked list/detail layout.

## Non-Goals And Future Options

Resizable companion width and a companion focus mode are useful future options,
but they should not be part of this first fix.

A later slice can add:

- a compact `Expand companion` action;
- persisted companion width preference;
- keyboard shortcuts or command-palette actions for switching focus.

Those features need more state, persistence, and testing. The current problem is
mostly default layout and responsive reflow.

## Testing

Add targeted coverage for:

- the analysis workspace desktop grid no longer caps the companion at 430px;
- the companion still collapses below the existing medium-width breakpoint;
- Evidence layout no longer uses a viewport-only `min-width: 1280px` rule for
  two columns;
- Evidence exposes a container or equivalent local width rule before enabling
  list/detail two-column layout;
- existing Evidence trace selection and `Show in source` contracts keep
  passing.

If tests remain raw CSS contract tests, normalize line endings like the existing
raw source tests do so they work in both the main checkout and worktrees on
Windows.

## Acceptance Criteria

- On a 1920px desktop viewport, the companion panel is wider than the previous
  430px cap.
- Evidence no longer renders a two-column list/detail layout when each column
  would be too narrow to read comfortably.
- The primary canvas remains usable and visually dominant on desktop.
- Medium and narrow breakpoints continue to stack the companion predictably.
- No Evidence data flow, trace selection, snapshot, or Source navigation
  behavior changes.
- Tests cover the width/reflow contract.
