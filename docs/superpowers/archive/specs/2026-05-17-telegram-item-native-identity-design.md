# Telegram Item Native Identity - Historical Note

> Status: shipped and archived. Current identity fields are documented in
> `docs/database-schema.md`.

## Decision

Telegram items use native Telegram identity fields rather than source-local
surrogate identity alone. Migrated history and duplicate detection require the
original Telegram message identity to remain visible.

## Rationale

- Telegram message identity is provider-native and stable across some import
  paths where local source rows differ.
- Migrated history can represent an old chat domain and needs careful duplicate
  reasoning.
- Native identity fields let import, browsing, and diagnostics explain why rows
  are considered the same or different.

## Preserved Contract

- Preserve the native identity tuple needed for duplicate detection.
- Keep migrated-history identity reasoning explicit.
- Do not collapse provider identity into local row ids in runtime logic.
