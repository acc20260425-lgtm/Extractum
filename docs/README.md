# Extractum Docs

Start here when you need project context. Current product and architecture
state belongs in the root docs listed below. Historical specs and pre-baseline
migrations live in archives; completed plans remain available through Git
history.

## Current State

- `project.md`: product scope, supported workflows, and implementation reading
  order.
- `design-document.md`: product/design overview and user-facing workflow
  direction.
- `architecture-deep-dive.md`: broader architecture notes.
- `backend-architecture-simplification-analysis.md`: current backend
  maintainability direction and remaining simplification work.
- `frontend-architecture-evolution-analysis.md`: current frontend workspace
  evolution guidance informed by Telegram Desktop reference review.
- `desktop-product-evolution-analysis.md`: cross-cutting desktop product
  maturity guidance for diagnostics, settings, exports, privacy, and release
  health informed by Telegram Desktop reference review.
- `database-schema.md`: current supported SQLite schema, migration baseline,
  and post-baseline migration authoring requirements.
- `backlog.md`: open work only. Shipped work should not accumulate here.
- `../research/youtube_pipeline/README.md`: local YouTube summary research
  prototype notes. The detailed boundary between the legacy direct-LLM runner
  and the file-backed agentic workflow lives in
  `../research/youtube_pipeline/RUNNER_AND_AGENTIC_WORKFLOW.md`.

## Focused Decisions And Analysis

- `database-schema-read-model-decision.md`: provider-neutral archive/read model
  decision, implementation status, and follow-up boundaries.
- `takeout-source-import.md`: Telegram Takeout import behavior and validation
  notes.

## Archives

- `archive/migrations-pre-baseline-reset/`: pre-baseline SQL and runner-managed
  Rust migration history. It is reference-only; active migrations start at the
  frozen `src-tauri/migrations/0001_current_schema_baseline.sql` and continue
  with numbered post-baseline migrations.
- `archive/database-schema-legacy-analysis.md`: historical schema debt
  analysis. Use it for background, then confirm current state in
  `database-schema.md`.
- `archive/`: documentation archive root.
- `superpowers/archive/specs/`: historical Superpowers design specs for
  shipped or superseded work.
- `superpowers/archive/verification/`: historical manual verification records.

## Superpowers Working Docs

- `superpowers/plans/`: active implementation plans only.
- `superpowers/specs/`: active or still-relevant design specs only.
- `superpowers/verification/`: active or reusable verification notes only.

Completed Superpowers plans should be removed from the working tree after their
outcome is captured in current-state docs, tests, backlog, or Git history.
Historical specs and verification notes can stay under `superpowers/archive/`
when they remain useful as design or regression context.

## Maintenance Rules

- Keep root docs as the source of truth for current behavior.
- Keep `backlog.md` limited to open work.
- Move stale specs to `superpowers/archive/specs/`.
- Delete completed plans instead of leaving execution logs in active folders.
- When a file becomes historical, say so at the top and link to the current
  source of truth.
