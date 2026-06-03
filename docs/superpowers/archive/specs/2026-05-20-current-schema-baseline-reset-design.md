# Current Schema Baseline Reset - Historical Note

> Status: shipped and archived. Current schema details live in
> `docs/database-schema.md` and active migrations.

## Decision

The project reset the migration baseline so new installations start from the
current schema instead of replaying obsolete transitional migrations.

## Rationale

- Historical migration code made current schema reasoning harder.
- A clean baseline helps tests and new installs match the shipped product
  model.
- Pre-reset artifacts still matter for archaeology but should not define current
  runtime behavior.

## Preserved Contract

- `0001_current_schema_baseline.sql` defines the active baseline.
- Pre-reset migration artifacts remain archived separately.
- New migrations should be additive after the baseline.
- Schema docs and migration checksums are the current source of truth.
