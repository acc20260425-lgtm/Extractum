# Takeout Migrated History Opt-In - Historical Note

> Status: shipped and archived. The original design predated the final import
> implementation, so this note keeps only durable decisions.

## Decision

Takeout migrated-history import is opt-in and treats old-chat identity as a
separate historical domain. Imported rows can be browsed or exported only when a
workflow explicitly supports migrated-history scope.

## Rationale

- Migrated history can have different peer identity from the current Telegram
  dialog.
- Import must preserve provenance and privacy boundaries without pretending the
  rows are ordinary current history.
- Capability checks make it clear which workflows can include migrated rows.

## Preserved Contract

- Lock and provenance behavior follows normal Takeout import safety rules.
- Old-chat identity fields are retained for diagnostics and duplicate
  reasoning.
- Product surfaces opt in through explicit scope controls.
- Unsupported actions stay current-only or show an unavailable state.
