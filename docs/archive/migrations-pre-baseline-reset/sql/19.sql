-- Version 19 is applied by src-tauri/src/migrations/source_identity_cleanup.rs.
-- This sentinel is registered so SQLx validates the applied checksum, but
-- direct plugin-managed execution must fail because v19 needs a pre-transaction
-- foreign-key-off rebuild and runner-side PRAGMA foreign_key_check assertions.
SELECT extractum_runner_managed_migration_19();
