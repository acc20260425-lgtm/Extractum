# Takeout Provenance Foundation - Historical Note

> Status: shipped and archived. Current schema details live in
> `docs/database-schema.md` and `docs/takeout-source-import.md`.

## Decision

Takeout imports write durable ingest provenance: batch records, item
observations, warnings, and aggregate counters that explain what happened during
an import without storing unsafe raw detail in user-facing docs.

## Rationale

- Takeout imports can be partial, cancelled, failed, or completed with warnings.
- Provenance is needed for recovery, diagnostics, duplicate reasoning, and
  representative validation.
- Warning privacy matters: committed docs and UI summaries should use warning
  categories and counters, not raw provider data.

## Preserved Contract

- Persist started/completed/failed/cancelled batch state.
- Record item observations and warning categories.
- Preserve crash/running semantics for recovery diagnostics.
- Keep provenance separate from ordinary duplicate detection.
