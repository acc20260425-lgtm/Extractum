# Topic Membership Materialization - Historical Note

> Status: shipped and archived.

## Decision

Telegram topic membership is materialized into explicit rows so browsing,
analysis, and export can reason about topics without re-deriving membership from
raw messages each time.

## Rationale

- Topic-rich supergroups need stable topic membership for source browsing and
  NotebookLM context.
- Real topic membership should stay distinct from derived or unrecognized
  buckets.
- Resolver readiness is part of source health, not a hidden implementation
  detail.

## Preserved Contract

- Materialize real topic membership explicitly.
- Keep unrecognized or derived buckets distinguishable.
- Track resolver readiness/state.
- Avoid using topic membership rows as a replacement for raw ingest provenance.
