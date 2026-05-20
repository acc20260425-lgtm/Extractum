# Extractum Docs

Start here when you need project context. Current product and architecture
state belongs in the root docs listed below. Historical plans, specs, and
pre-baseline migrations live in archives.

## Current State

- `project.md`: product scope, supported workflows, and implementation reading
  order.
- `design-document.md`: product/design overview and user-facing workflow
  direction.
- `architecture-deep-dive.md`: broader architecture notes.
- `backend-architecture-simplification-analysis.md`: current backend
  maintainability direction and remaining simplification work.
- `database-schema.md`: current supported SQLite schema and migration baseline.
- `backlog.md`: open work only. Shipped work should not accumulate here.

## Focused Decisions And Analysis

- `database-schema-read-model-decision.md`: provider-neutral archive/read model
  decision, implementation status, and follow-up boundaries.
- `database-schema-legacy-analysis.md`: historical schema debt analysis. Use it
  for background, then confirm current state in `database-schema.md`.
- `takeout-source-import.md`: Telegram Takeout import behavior and validation
  notes.

## Archives

- `archive/migrations-pre-baseline-reset/`: pre-baseline SQL and runner-managed
  Rust migration history. It is reference-only; active migrations start at
  `src-tauri/migrations/0001_current_schema_baseline.sql`.
- `superpowers/archive/specs/`: historical Superpowers design specs for
  shipped or superseded work.
- `superpowers/verification/`: retained manual verification notes that are
  still useful as regression references.

## Superpowers Working Docs

- `superpowers/plans/`: active implementation plans only.
- `superpowers/specs/`: active or still-relevant design specs only.

Completed Superpowers plans should be removed from the working tree after their
outcome is captured in current-state docs, tests, backlog, or Git history.

## Maintenance Rules

- Keep root docs as the source of truth for current behavior.
- Keep `backlog.md` limited to open work.
- Move stale specs to `superpowers/archive/specs/`.
- Delete completed plans instead of leaving execution logs in active folders.
- When a file becomes historical, say so at the top and link to the current
  source of truth.
