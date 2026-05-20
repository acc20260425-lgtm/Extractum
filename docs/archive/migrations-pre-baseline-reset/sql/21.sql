-- Version 21 is applied by src-tauri/src/migrations/telegram_item_native_identity.rs.
-- This sentinel is registered so SQLx validates the applied checksum, but
-- direct plugin-managed execution must fail because v21 performs best-effort
-- typed Telegram item backfill, data-integrity checks, and index replacement
-- in one Rust-owned transaction.
SELECT extractum_runner_managed_migration_21();
