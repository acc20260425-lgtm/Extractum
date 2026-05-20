-- Version 20 is applied by src-tauri/src/migrations/youtube_typed_source_metadata.rs.
-- This sentinel is registered so SQLx validates the applied checksum, but
-- direct plugin-managed execution must fail because v20 needs Rust-side zstd
-- JSON decode, typed validation, and transactional source-blob clearing.
SELECT extractum_runner_managed_migration_20();
