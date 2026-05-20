# Pre-Baseline Migration Archive

This directory preserves the migration history that existed before the current
schema baseline reset.

The active application migration path now starts at
`src-tauri/migrations/0001_current_schema_baseline.sql`. Files in this archive
are reference-only and are not compiled into the active migration list.

Contents:

- `sql/`: historical SQL migrations `1.sql` through `26.sql`
- `rust/`: historical runner-managed Rust migration modules

Supported databases either start from baseline v1 or, for the one controlled
pre-reset database, pass through the backup-first baseline-history cutover that
rewrites only `_sqlx_migrations`. Future active migrations start at `0002`.
