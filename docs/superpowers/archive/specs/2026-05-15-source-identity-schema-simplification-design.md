# Source Identity Schema Simplification - Historical Note

> Status: shipped and archived. This was a large transitional design; current
> schema truth is `docs/database-schema.md`.

## Decision

Source identity moved toward an explicit typed model with canonical
`source_subtype`, typed Telegram source rows, repair notes, and uniqueness
contracts that avoid decoding legacy metadata in runtime paths.

## Rationale

- Source identity needs to be stable across Telegram, YouTube, Takeout, source
  groups, browsing, and analysis.
- Legacy metadata payloads made identity decisions hard to inspect and repair.
- Typed tables and explicit uniqueness rules make import, duplicate detection,
  NotebookLM export, and diagnostics safer.

## Preserved Contract

- Treat `(account_id, source_type, source_subtype, external_id)` as the
  canonical Telegram source identity shape where applicable.
- Keep `telegram_sources` as typed Telegram identity metadata.
- Record source identity repair notes for diagnostics.
- Do not make runtime behavior depend on decoding legacy provider payloads.
- Use root schema docs and active migrations for all current field-level detail.
