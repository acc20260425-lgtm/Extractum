# Analysis Result-First Redesign - Historical Note

> Status: shipped and archived. Current analysis workspace docs supersede most
> implementation detail in the original design.

## Decision

The analysis workspace moved to a result-first layout: the report canvas is the
primary surface, while sources, evidence, chat, and tools support the selected
result instead of competing as equal top-level panels.

## Rationale

- Users read and evaluate generated analysis more often than they configure it.
- Source and evidence context should stay close to the report without stealing
  focus from the result.
- Saved-run snapshots require a state model that preserves what was captured,
  not just what is currently available live.

## Preserved Contract

- Report canvas is the center of the workflow.
- Companion surfaces should reflect the selected run/result basis.
- Snapshot-backed runs must stay trustworthy even when live source state has
  changed.
- Component ownership should separate report rendering, source browsing,
  evidence focus, and run metadata.

## Current Pointers

Use `docs/project.md`, `docs/design-document.md`, and the components under
`src/lib/components/analysis/` for current implementation details.
