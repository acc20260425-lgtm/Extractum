# Evidence Source Navigation - Historical Note

> Status: shipped and archived. Current behavior is implemented by the analysis
> evidence/source navigation helpers and analysis workspace components.

## Decision

Evidence entries can focus the source surface that contains the underlying
material. Navigation must preserve whether the evidence came from live/current
state or from a saved-run snapshot.

## Rationale

- Evidence is most useful when the user can inspect nearby source context
  without losing their place in the report.
- Saved-run evidence must not silently cross into live source state.
- Repeated navigation should be resilient to stale async loads and changing
  selection state.

## Preserved Contract

- Map evidence trace targets to the correct live or snapshot source basis.
- Highlight the target item when possible.
- Provide a "Back to evidence" affordance after source focus.
- Treat pending, unavailable, or missing snapshot state as status-only rather
  than opening a live source browser.
- Guard request identity so stale loads cannot overwrite newer user intent.

## Current Pointers

- Navigation helper: `src/lib/analysis-evidence-source-navigation.ts`
- Tests: `src/lib/analysis-evidence-source-navigation.test.ts`
- Source/report surface components under `src/lib/components/analysis/`
