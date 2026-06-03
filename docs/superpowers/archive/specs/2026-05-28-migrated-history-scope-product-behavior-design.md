# Migrated History Scope Product Behavior - Historical Note

> Status: shipped and archived. Current behavior is documented in
> `docs/project.md`, `docs/takeout-source-import.md`, and
> `docs/database-schema.md`.

## Decision

Migrated Telegram history is excluded from normal current-history browsing by
default. Users opt into migrated-history scope explicitly for browsing, export,
and analysis contexts that support it.

## Rationale

- Migrated history often represents an old chat identity and should not silently
  merge into current chat history.
- Explicit scope keeps source browsing, NotebookLM export, and analysis runs
  honest about what history basis they include.
- The database can retain migrated history while the product keeps current-only
  defaults safe.

## Preserved Contract

- Current-only remains the default scope.
- `telegram_history_scope` records the selected basis.
- Supported workflows expose migrated-history scope deliberately.
- Unsupported workflows should fail or disable clearly rather than silently
  including migrated rows.
