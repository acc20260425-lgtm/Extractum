# Diagnostics UI Design - Historical Note

> Status: shipped and archived. Current behavior is documented in
> `docs/project.md`, `docs/design-document.md`, and
> `docs/architecture-deep-dive.md`.

## Decision

Diagnostics use a dedicated read-only `/diagnostics` route rather than a
Settings panel or an automatic support bundle workflow.

The page intentionally supports manual refresh only. It presents local health,
runtime, source, provider, filesystem, browser, frontend, and database
summaries without exposing raw logs, raw payloads, file contents, provider
tokens, or one-click copying.

## Rationale

- Diagnostics is an operator/support surface, not an end-user settings flow.
- Manual refresh avoids background polling and keeps potentially sensitive
  system state behind explicit user action.
- Aggregated counts and sanitized error labels are enough for current support
  triage while keeping local data private.
- The UI should stay dense and scannable: summary cards, compact tables, and
  short descriptions rather than marketing-style explanation.

## Current Implementation Boundaries

- Backend diagnostics live under `src-tauri/src/diagnostics/`.
- `src/lib/api/diagnostics.ts` is the Tauri command boundary.
- `src/lib/types/diagnostics.ts` mirrors the camelCase DTO.
- `src/lib/diagnostics-view-model.ts` owns derived labels, tones, and safe
  display strings.
- `src/lib/components/diagnostics/DiagnosticCountTable.svelte` renders repeated
  count tables.
- `src/routes/diagnostics/+page.svelte` loads on mount and refreshes manually.

## Non-Goals Preserved

- no raw log viewer;
- no filesystem browser;
- no support bundle generation;
- no automatic polling;
- no privileged repair or cleanup actions from this route.
