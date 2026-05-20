-- Version 22 is applied by src-tauri/src/migrations/topic_membership_materialization.rs.
-- The Rust runner owns schema creation, source-level membership rebuilds,
-- state rows, invariant checks, and migration-history recording.
SELECT extractum_runner_managed_migration_22();
