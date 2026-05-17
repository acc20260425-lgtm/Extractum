# Telegram Item Native Identity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed Telegram message identity rows so Telegram duplicate detection and topic/message matching no longer depend on `items.external_id`, while keeping existing `items`-based analysis, browsing, and export compatibility.

**Architecture:** Add runner-managed migration 21 with a `telegram_messages` child table, best-effort backfill, integrity checks, and replacement item uniqueness for non-Telegram rows. Split Telegram item insertion from generic item upserts: Telegram writes `items` plus `telegram_messages` in one writer transaction; YouTube keeps deterministic `items` upserts through a partial unique index. Topic joins prefer typed message ids and isolate the legacy external-id cast fallback.

**Tech Stack:** Rust, Tauri commands, SQLx SQLite, grammers Telegram types, existing zstd compression helpers, runner-managed migration pattern from migrations 19 and 20.

---

## File Structure

- Create `src-tauri/migrations/21.sql`
  - Sentinel migration so SQLx migration history records version 21 while Rust owns the actual migration.
- Create `src-tauri/src/migrations/telegram_item_native_identity.rs`
  - Own migration 21 registration checks, DDL execution, best-effort backfill, partial index replacement, integrity validation, migration-history recording, and migration tests.
- Modify `src-tauri/src/migrations.rs`
  - Register migration 21 and run it after migration 20 in startup/test migration flow.
- Modify `src-tauri/src/sources/types.rs`
  - Add Telegram peer-kind/item-identity wire constants and typed identity structs.
- Modify `src-tauri/src/sources/test_support.rs`
  - Add helpers for `telegram_messages` table/index creation and replacement item uniqueness in in-memory tests.
- Modify `src-tauri/src/sources/items.rs`
  - Split generic item insertion from Telegram insertion, add `insert_telegram_source_item`, update YouTube upserts to target the non-Telegram partial unique index, and add runtime tests.
- Modify `src-tauri/src/sources/sync.rs`
  - Build native Telegram message identity from live grammers messages and route normal sync through the new helper.
- Modify `src-tauri/src/takeout_import/raw_parse.rs`
  - Extract raw history peer identity from TL messages and carry it into `SourceItemInsert`.
- Modify `src-tauri/src/takeout_import/mod.rs`
  - Route Takeout import through the Telegram insert helper and ensure migrated-history tests use the production parse/insert boundary.
- Modify `src-tauri/src/forum_topics.rs`
  - Prefer typed `telegram_messages.telegram_message_id` in topic predicates and isolate legacy external-id casts.
- Modify `src-tauri/src/sources/items/query.rs`
  - Join `telegram_messages` for item browsing topic resolution tests.
- Modify `src-tauri/src/notebooklm_export/query.rs`
  - Use the same typed topic join for export queries.
- Modify `src-tauri/src/analysis/trace.rs`
  - Resolve legacy `s{source_id}-m{message_id}` refs by Telegram message id when unique, and surface ambiguous legacy refs as conflicts through the public resolver path.
- Modify `src-tauri/src/analysis/mod.rs`
  - Call the fallible trace-ref resolver for explicit ref resolution.
- Modify `docs/database-schema.md`
  - Document `telegram_messages`, the new `items.external_id` boundary, and replacement item indexes.
- Modify `docs/backlog.md`
  - Keep only remaining open schema-simplification follow-ups.
- Modify this plan as tasks complete.

## Task 0: Branch Guard And Baseline

**Files:**
- Read: `docs/superpowers/specs/2026-05-17-telegram-item-native-identity-design.md`
- Verify: Git status and focused baseline tests

- [x] **Step 1: Confirm clean starting point**

Run:

```powershell
git status --short --branch
git --no-pager log -8 --oneline --decorate
```

Expected:

```text
## main
a10c123 (HEAD -> main) docs: tighten telegram item row invariants
```

If the working tree contains user changes, inspect them and keep them intact.

- [x] **Step 2: Create an implementation branch or worktree**

Use `superpowers:using-git-worktrees` before execution. A safe branch name is:

```powershell
git switch -c feature/telegram-item-native-identity
```

If using a linked worktree, use:

```powershell
git worktree add .worktrees/telegram-item-native-identity -b feature/telegram-item-native-identity
```

- [x] **Step 3: Run focused baseline tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations:: sources::items:: sources::sync:: takeout_import::raw_parse:: forum_topics:: notebooklm_export::query:: analysis::trace::
```

Expected:

```text
test result: ok
```

- [x] **Step 4: Confirm no baseline changes**

Run:

```powershell
git status --short --branch
```

Expected: only the branch header, with no modified files.

## Task 1: Migration 21 Sentinel And Registration

**Files:**
- Create: `src-tauri/migrations/21.sql`
- Create: `src-tauri/src/migrations/telegram_item_native_identity.rs`
- Modify: `src-tauri/src/migrations.rs`

- [x] **Step 1: Add RED migration registration tests**

In `src-tauri/src/migrations.rs`, add the module declaration near the other migration modules:

```rust
pub(crate) mod telegram_item_native_identity;
```

Inside `#[cfg(test)] mod tests`, add:

```rust
#[test]
fn includes_runner_managed_telegram_item_native_identity_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 21)
        .expect("version 21 migration is registered");

    assert_eq!(migration.description, "add telegram item native identity");
    assert!(
        migration
            .sql
            .contains("extractum_runner_managed_migration_21"),
        "v21 must fail if plugin-managed SQL applies it directly"
    );
}

#[test]
fn plugin_migration_list_keeps_v21_as_sentinel_only() {
    let migration = build_migrations()
        .into_iter()
        .find(|migration| migration.version == 21)
        .expect("version 21 migration is registered");

    assert!(!migration.sql.contains("CREATE TABLE telegram_messages"));
    assert!(!migration.sql.contains("DROP INDEX idx_items_ext"));
    assert!(!migration.sql.contains("INSERT INTO telegram_messages"));
}
```

Update the existing version-list assertion:

```rust
assert_eq!(versions, (1_i64..=21_i64).collect::<Vec<_>>());
```

- [x] **Step 2: Run registration tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_runner_managed_telegram_item_native_identity_migration migrations::tests::plugin_migration_list_keeps_v21_as_sentinel_only migrations::tests::build_migrations_contains_all_versions_for_sqlx_validation
```

Expected: fail because migration 21 is not registered yet.

- [x] **Step 3: Add the sentinel SQL file**

Create `src-tauri/migrations/21.sql`:

```sql
-- Version 21 is applied by src-tauri/src/migrations/telegram_item_native_identity.rs.
-- This sentinel is registered so SQLx validates the applied checksum, but
-- direct plugin-managed execution must fail because v21 performs best-effort
-- typed Telegram item backfill, data-integrity checks, and index replacement
-- in one Rust-owned transaction.
SELECT extractum_runner_managed_migration_21();
```

- [x] **Step 4: Register migration 21**

In `src-tauri/src/migrations.rs`, append this entry after version 20:

```rust
Migration {
    version: 21,
    description: "add telegram item native identity",
    sql: include_str!("../migrations/21.sql"),
    kind: MigrationKind::Up,
},
```

Update `patch_migrations` to run v21 after v20:

```rust
youtube_typed_source_metadata::apply_youtube_typed_source_metadata_if_needed(&url).await?;
telegram_item_native_identity::apply_telegram_item_native_identity_if_needed(&url).await
```

Update `apply_all_migrations_for_test_pool` to run v21 after v20:

```rust
youtube_typed_source_metadata::apply_youtube_typed_source_metadata_on_connection(conn).await?;
telegram_item_native_identity::apply_telegram_item_native_identity_on_connection(conn).await
```

- [x] **Step 5: Add the runner-managed migration skeleton**

Create `src-tauri/src/migrations/telegram_item_native_identity.rs`:

```rust
use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(super) const TELEGRAM_ITEM_NATIVE_IDENTITY_VERSION: i64 = 21;
pub(super) const TELEGRAM_ITEM_NATIVE_IDENTITY_DESCRIPTION: &str =
    "add telegram item native identity";
pub(super) const TELEGRAM_ITEM_NATIVE_IDENTITY_SENTINEL_SQL: &str =
    include_str!("../../migrations/21.sql");

pub(super) async fn apply_telegram_item_native_identity_if_needed(
    db_url: &str,
) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_telegram_item_native_identity_on_connection(&mut conn).await
}

pub(super) async fn apply_telegram_item_native_identity_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_previous_migration_recorded(conn).await?;
    if migration_21_recorded(conn).await? {
        return Ok(());
    }

    let started_at = Instant::now();
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result = async {
        create_telegram_messages_schema(conn).await?;
        backfill_telegram_messages(conn).await?;
        replace_item_identity_indexes(conn).await?;
        assert_post_migration_integrity(conn).await
    }
    .await;

    match result {
        Ok(()) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            return Err(error);
        }
    }

    record_migration_success(
        conn,
        TELEGRAM_ITEM_NATIVE_IDENTITY_VERSION,
        TELEGRAM_ITEM_NATIVE_IDENTITY_DESCRIPTION,
        expected_migration_21_checksum(),
        started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
    )
    .await
}

async fn ensure_previous_migration_recorded(conn: &mut SqliteConnection) -> AppResult<()> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 20 AND success = 1",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if exists == 0 {
        return Err(AppError::validation(
            "Telegram item native identity migration 21 requires migration 20",
        ));
    }
    Ok(())
}

async fn migration_21_recorded(conn: &mut SqliteConnection) -> AppResult<bool> {
    let checksum = expected_migration_21_checksum();
    let row: Option<(Vec<u8>, bool)> =
        sqlx::query_as("SELECT checksum, success FROM _sqlx_migrations WHERE version = ?")
            .bind(TELEGRAM_ITEM_NATIVE_IDENTITY_VERSION)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?;

    match row {
        None => Ok(false),
        Some((applied_checksum, true)) if applied_checksum == checksum => Ok(true),
        Some((_applied_checksum, true)) => Err(AppError::internal(
            "Migration 21 checksum does not match the runner-managed Telegram item native identity sentinel",
        )),
        Some((_applied_checksum, false)) => Err(AppError::internal(
            "Migration 21 is marked as failed in _sqlx_migrations",
        )),
    }
}

async fn record_migration_success(
    conn: &mut SqliteConnection,
    version: i64,
    description: &str,
    checksum: Vec<u8>,
    execution_time: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, ?)",
    )
    .bind(version)
    .bind(description)
    .bind(checksum)
    .bind(execution_time)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn expected_migration_21_checksum() -> Vec<u8> {
    Sha384::digest(TELEGRAM_ITEM_NATIVE_IDENTITY_SENTINEL_SQL.as_bytes()).to_vec()
}

async fn create_telegram_messages_schema(conn: &mut SqliteConnection) -> AppResult<()> {
    sqlx::raw_sql(TELEGRAM_MESSAGES_SCHEMA_SQL)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

async fn backfill_telegram_messages(_conn: &mut SqliteConnection) -> AppResult<()> {
    Ok(())
}

async fn replace_item_identity_indexes(_conn: &mut SqliteConnection) -> AppResult<()> {
    Ok(())
}

async fn assert_post_migration_integrity(_conn: &mut SqliteConnection) -> AppResult<()> {
    Ok(())
}

const TELEGRAM_MESSAGES_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS telegram_messages (
    item_id INTEGER PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    history_peer_kind TEXT NOT NULL,
    history_peer_id INTEGER NOT NULL,
    telegram_message_id INTEGER NOT NULL,
    migration_domain TEXT,
    is_migrated_history INTEGER NOT NULL DEFAULT 0,
    reply_to_msg_id INTEGER,
    reply_to_peer_kind TEXT,
    reply_to_peer_id INTEGER,
    reply_to_top_id INTEGER,
    reaction_count INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (history_peer_kind IN ('channel', 'chat', 'user')),
    CHECK (telegram_message_id > 0),
    CHECK (is_migrated_history IN (0, 1)),
    CHECK (reply_to_msg_id IS NULL OR reply_to_msg_id > 0),
    CHECK (
        reply_to_peer_kind IS NULL
        OR reply_to_peer_kind IN ('channel', 'chat', 'user')
    ),
    CHECK (reply_to_peer_id IS NULL OR reply_to_peer_id > 0),
    CHECK (reply_to_top_id IS NULL OR reply_to_top_id > 0),
    CHECK (reaction_count IS NULL OR reaction_count >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_telegram_messages_native_identity
    ON telegram_messages (
        source_id,
        history_peer_kind,
        history_peer_id,
        telegram_message_id
    );

CREATE INDEX IF NOT EXISTS idx_telegram_messages_source_message
    ON telegram_messages(source_id, telegram_message_id);

CREATE INDEX IF NOT EXISTS idx_telegram_messages_source_reply_top
    ON telegram_messages(source_id, reply_to_top_id);
"#;
```

- [x] **Step 6: Run registration tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_runner_managed_telegram_item_native_identity_migration migrations::tests::plugin_migration_list_keeps_v21_as_sentinel_only migrations::tests::build_migrations_contains_all_versions_for_sqlx_validation
```

Expected:

```text
test result: ok
```

- [x] **Step 7: Commit**

```powershell
git add src-tauri/migrations/21.sql src-tauri/src/migrations.rs src-tauri/src/migrations/telegram_item_native_identity.rs
git commit -m "feat: add telegram item identity migration sentinel"
```

## Task 2: Migration 21 Backfill And Integrity Checks

**Files:**
- Modify: `src-tauri/src/migrations/telegram_item_native_identity.rs`

- [x] **Step 1: Add RED tests for schema, backfill, skip counts, duplicate domains, and integrity**

Inside `#[cfg(test)] mod tests` in `src-tauri/src/migrations/telegram_item_native_identity.rs`, add:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_text;
    use crate::migrations::build_migrations;
    use sqlx::SqliteConnection;

    #[tokio::test]
    async fn migration_21_backfills_valid_telegram_rows_and_skips_malformed_rows() {
        let mut conn = memory_conn_with_history_through_20().await;
        seed_telegram_source(&mut conn, 101, "channel", 12345).await;
        insert_telegram_item(&mut conn, 301, 101, "42", Some(7), Some("channel"), Some("12345"), Some(5), Some(2)).await;
        insert_telegram_item(&mut conn, 302, 101, "bad-42", None, None, None, None, None).await;
        insert_telegram_item(&mut conn, 303, 101, " 43", None, None, None, None, None).await;

        apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect("apply v21");

        let rows: Vec<(i64, i64, String, i64, i64, Option<i64>, Option<String>, Option<i64>, Option<i64>, Option<i64>)> =
            sqlx::query_as(
                "SELECT item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id, reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id, reaction_count FROM telegram_messages ORDER BY item_id",
            )
            .fetch_all(&mut conn)
            .await
            .expect("load typed rows");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, 301);
        assert_eq!(rows[0].1, 101);
        assert_eq!(rows[0].2, "channel");
        assert_eq!(rows[0].3, 12345);
        assert_eq!(rows[0].4, 42);
        assert_eq!(rows[0].5, Some(7));
        assert_eq!(rows[0].6.as_deref(), Some("channel"));
        assert_eq!(rows[0].7, Some(12345));
        assert_eq!(rows[0].8, Some(5));
        assert_eq!(rows[0].9, Some(2));
    }

    #[tokio::test]
    async fn migration_21_allows_same_message_id_across_history_domains() {
        let mut conn = memory_conn_with_history_through_20().await;
        seed_telegram_source(&mut conn, 101, "channel", 12345).await;
        insert_telegram_item(&mut conn, 301, 101, "42", None, None, None, None, None).await;

        apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect("apply v21");

        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, ingested_at, content_zstd, content_kind, has_media)
             VALUES (302, 101, '42', 'telegram_message', 'bob', 2, 2, ?, 'text_only', 0)",
        )
        .bind(compress_text("migrated").expect("compress"))
        .execute(&mut conn)
        .await
        .expect("insert overlapping legacy item");

        sqlx::query(
            "INSERT INTO telegram_messages (item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id, is_migrated_history)
             VALUES (302, 101, 'chat', 777, 42, 1)",
        )
        .execute(&mut conn)
        .await
        .expect("insert migrated identity");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM items WHERE source_id = 101 AND external_id = '42'",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count overlapping ids");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn migration_21_rejects_null_item_kind_before_replacing_idx_items_ext() {
        let mut conn = memory_conn_with_history_through_20().await;
        sqlx::query("PRAGMA writable_schema = ON")
            .execute(&mut conn)
            .await
            .expect("enable writable schema for fixture");
        sqlx::query("UPDATE sqlite_master SET sql = replace(sql, 'item_kind TEXT NOT NULL DEFAULT ''telegram_message''', 'item_kind TEXT DEFAULT ''telegram_message''') WHERE type = 'table' AND name = 'items'")
            .execute(&mut conn)
            .await
            .expect("relax fixture schema");
        sqlx::query("PRAGMA writable_schema = OFF")
            .execute(&mut conn)
            .await
            .expect("disable writable schema");
        sqlx::query("INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, ingested_at, content_kind, has_media) VALUES (900, 1, 'null-kind', NULL, 'nobody', 1, 1, 'text_only', 0)")
            .execute(&mut conn)
            .await
            .expect("insert null item_kind fixture");

        let error = apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect_err("null item_kind blocks migration");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("item_kind"));
    }

    #[tokio::test]
    async fn migration_21_rejects_existing_non_telegram_duplicate_external_ids() {
        let mut conn = memory_conn_with_history_through_20().await;
        insert_youtube_item(&mut conn, 801, 201, "comment:dup", "youtube_comment").await;
        insert_youtube_item(&mut conn, 802, 201, "comment:dup", "youtube_comment").await;

        let error = apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect_err("duplicates block partial unique index");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("non-Telegram duplicate"));
    }

    #[tokio::test]
    async fn migration_21_records_sentinel_checksum_and_is_idempotent() {
        let mut conn = memory_conn_with_history_through_20().await;

        apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect("first v21");
        apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect("second v21");

        let row: (String, bool, Vec<u8>) = sqlx::query_as(
            "SELECT description, success, checksum FROM _sqlx_migrations WHERE version = 21",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load v21 history");
        assert_eq!(row.0, TELEGRAM_ITEM_NATIVE_IDENTITY_DESCRIPTION);
        assert!(row.1);
        assert_eq!(row.2, expected_migration_21_checksum());
    }

    async fn memory_conn_with_history_through_20() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
        .expect("create migration history");

        for migration in build_migrations()
            .into_iter()
            .filter(|migration| migration.version < 19)
        {
            sqlx::raw_sql(migration.sql)
                .execute(&mut conn)
                .await
                .unwrap_or_else(|error| panic!("apply migration {}: {error}", migration.version));
            sqlx::query("INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)")
                .bind(migration.version)
                .bind(migration.description)
                .bind(Sha384::digest(migration.sql.as_bytes()).to_vec())
                .execute(&mut conn)
                .await
                .expect("record standard migration");
        }

        crate::migrations::source_identity_cleanup::apply_source_identity_cleanup_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v19");
        crate::migrations::youtube_typed_source_metadata::apply_youtube_typed_source_metadata_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v20");

        conn
    }

    async fn seed_telegram_source(
        conn: &mut SqliteConnection,
        source_id: i64,
        peer_kind: &str,
        peer_id: i64,
    ) {
        sqlx::query("INSERT OR IGNORE INTO accounts (id, label, api_id, api_hash, created_at) VALUES (1, 'acct', 1, 'hash', 1)")
            .execute(&mut *conn)
            .await
            .expect("insert account");
        sqlx::query("INSERT OR IGNORE INTO sources (id, source_type, source_subtype, account_id, external_id, title, is_active, is_member, created_at) VALUES (?, 'telegram', 'supergroup', 1, ?, 'Forum', 1, 1, 1)")
            .bind(source_id)
            .bind(peer_id.to_string())
            .execute(&mut *conn)
            .await
            .expect("insert source");
        sqlx::query("INSERT INTO telegram_sources (source_id, account_id, source_subtype, peer_kind, peer_id, resolution_strategy) VALUES (?, 1, 'supergroup', ?, ?, 'dialog')")
            .bind(source_id)
            .bind(peer_kind)
            .bind(peer_id)
            .execute(&mut *conn)
            .await
            .expect("insert telegram source");
    }

    async fn insert_telegram_item(
        conn: &mut SqliteConnection,
        id: i64,
        source_id: i64,
        external_id: &str,
        reply_to_msg_id: Option<i64>,
        reply_to_peer_kind: Option<&str>,
        reply_to_peer_id: Option<&str>,
        reply_to_top_id: Option<i64>,
        reaction_count: Option<i64>,
    ) {
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_zstd, content_kind, has_media, reply_to_msg_id, reply_to_peer_kind,
                reply_to_peer_id, reply_to_top_id, reaction_count
             ) VALUES (?, ?, ?, 'telegram_message', 'alice', 1, 1, ?, 'text_only', 0, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(source_id)
        .bind(external_id)
        .bind(compress_text("hello").expect("compress"))
        .bind(reply_to_msg_id)
        .bind(reply_to_peer_kind)
        .bind(reply_to_peer_id)
        .bind(reply_to_top_id)
        .bind(reaction_count)
        .execute(&mut *conn)
        .await
        .expect("insert telegram item");
    }

    async fn insert_youtube_item(
        conn: &mut SqliteConnection,
        id: i64,
        source_id: i64,
        external_id: &str,
        item_kind: &str,
    ) {
        sqlx::query("INSERT OR IGNORE INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at) VALUES (?, 'youtube', 'video', 'video-1', 'Video', 1, 0, 1)")
            .bind(source_id)
            .execute(&mut *conn)
            .await
            .expect("insert youtube source");
        sqlx::query("INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, ingested_at, content_zstd, content_kind, has_media) VALUES (?, ?, ?, ?, 'yt', 1, 1, ?, 'text_only', 0)")
            .bind(id)
            .bind(source_id)
            .bind(external_id)
            .bind(item_kind)
            .bind(compress_text("youtube").expect("compress"))
            .execute(&mut *conn)
            .await
            .expect("insert youtube item");
    }
}
```

- [x] **Step 2: Run migration tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::telegram_item_native_identity::
```

Expected: fail because backfill, preflight, index replacement, and integrity checks are still stubs.

- [x] **Step 3: Implement migration preflight, backfill, index replacement, and integrity**

In `src-tauri/src/migrations/telegram_item_native_identity.rs`, replace the stub functions with:

```rust
#[derive(sqlx::FromRow)]
struct ForeignKeyCheckRow {
    table: String,
    rowid: Option<i64>,
    parent: String,
    fkid: i64,
}

#[derive(Debug, PartialEq, Eq)]
struct BackfillStats {
    backfilled: i64,
    skipped: i64,
}

async fn backfill_telegram_messages(conn: &mut SqliteConnection) -> AppResult<BackfillStats> {
    let before_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_messages")
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

    sqlx::query(
        r#"
        INSERT OR IGNORE INTO telegram_messages (
            item_id,
            source_id,
            history_peer_kind,
            history_peer_id,
            telegram_message_id,
            migration_domain,
            is_migrated_history,
            reply_to_msg_id,
            reply_to_peer_kind,
            reply_to_peer_id,
            reply_to_top_id,
            reaction_count,
            created_at,
            updated_at
        )
        SELECT
            items.id,
            items.source_id,
            telegram_sources.peer_kind,
            telegram_sources.peer_id,
            CAST(items.external_id AS INTEGER),
            NULL,
            0,
            CASE WHEN items.reply_to_msg_id > 0 THEN items.reply_to_msg_id ELSE NULL END,
            CASE
                WHEN items.reply_to_peer_kind IN ('channel', 'chat', 'user')
                THEN items.reply_to_peer_kind
                ELSE NULL
            END,
            CASE
                WHEN items.reply_to_peer_id IS NOT NULL
                 AND items.reply_to_peer_id <> ''
                 AND items.reply_to_peer_id NOT GLOB '*[^0-9]*'
                 AND CAST(items.reply_to_peer_id AS INTEGER) > 0
                THEN CAST(items.reply_to_peer_id AS INTEGER)
                ELSE NULL
            END,
            CASE WHEN items.reply_to_top_id > 0 THEN items.reply_to_top_id ELSE NULL END,
            CASE WHEN items.reaction_count >= 0 THEN items.reaction_count ELSE NULL END,
            strftime('%s','now'),
            strftime('%s','now')
        FROM items
        JOIN sources ON sources.id = items.source_id
        JOIN telegram_sources ON telegram_sources.source_id = sources.id
        WHERE items.item_kind = 'telegram_message'
          AND sources.source_type = 'telegram'
          AND items.external_id <> ''
          AND items.external_id NOT GLOB '*[^0-9]*'
          AND CAST(items.external_id AS INTEGER) > 0
        "#,
    )
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;

    let after_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_messages")
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;
    let candidate_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM items WHERE item_kind = 'telegram_message'",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;

    let backfilled = after_count - before_count;
    Ok(BackfillStats {
        backfilled,
        skipped: (candidate_count - backfilled).max(0),
    })
}

async fn replace_item_identity_indexes(conn: &mut SqliteConnection) -> AppResult<()> {
    assert_no_null_item_kind(conn).await?;
    assert_no_non_telegram_duplicate_external_ids(conn).await?;

    sqlx::raw_sql(
        r#"
        DROP INDEX IF EXISTS idx_items_ext;

        CREATE UNIQUE INDEX IF NOT EXISTS ux_items_non_telegram_external
            ON items(source_id, external_id)
            WHERE item_kind <> 'telegram_message';

        CREATE INDEX IF NOT EXISTS idx_items_source_external
            ON items(source_id, external_id);
        "#,
    )
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn assert_no_null_item_kind(conn: &mut SqliteConnection) -> AppResult<()> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM items WHERE item_kind IS NULL")
            .fetch_one(&mut *conn)
            .await
            .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Telegram item native identity migration 21 found {count} items with NULL item_kind"
        )));
    }
    Ok(())
}

async fn assert_no_non_telegram_duplicate_external_ids(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM (
            SELECT source_id, external_id
            FROM items
            WHERE item_kind <> 'telegram_message'
            GROUP BY source_id, external_id
            HAVING COUNT(*) > 1
        )
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Telegram item native identity migration 21 found {count} non-Telegram duplicate item external ids"
        )));
    }
    Ok(())
}

async fn assert_post_migration_integrity(conn: &mut SqliteConnection) -> AppResult<()> {
    assert_foreign_key_check_clean(conn).await?;
    assert_no_telegram_message_item_kind_mismatch(conn).await?;
    assert_no_telegram_message_source_mismatch(conn).await?;
    assert_no_duplicate_native_telegram_identity(conn).await?;
    Ok(())
}

async fn assert_foreign_key_check_clean(conn: &mut SqliteConnection) -> AppResult<()> {
    let rows: Vec<ForeignKeyCheckRow> = sqlx::query_as("PRAGMA foreign_key_check")
        .fetch_all(&mut *conn)
        .await
        .map_err(AppError::database)?;
    if rows.is_empty() {
        return Ok(());
    }

    let detail = rows
        .into_iter()
        .map(|row| format!("{} rowid {:?} references {} via fk {}", row.table, row.rowid, row.parent, row.fkid))
        .collect::<Vec<_>>()
        .join("; ");
    Err(AppError::validation(format!(
        "Telegram item native identity migration 21 foreign_key_check failed: {detail}"
    )))
}

async fn assert_no_telegram_message_item_kind_mismatch(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM telegram_messages tm
        JOIN items i ON i.id = tm.item_id
        WHERE i.item_kind <> 'telegram_message'
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Migration 21 found {count} telegram_messages rows pointing to non-Telegram items"
        )));
    }
    Ok(())
}

async fn assert_no_telegram_message_source_mismatch(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM telegram_messages tm
        JOIN items i ON i.id = tm.item_id
        WHERE tm.source_id <> i.source_id
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Migration 21 found {count} telegram_messages rows with source_id mismatch"
        )));
    }
    Ok(())
}

async fn assert_no_duplicate_native_telegram_identity(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM (
            SELECT source_id, history_peer_kind, history_peer_id, telegram_message_id
            FROM telegram_messages
            GROUP BY source_id, history_peer_kind, history_peer_id, telegram_message_id
            HAVING COUNT(*) > 1
        )
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Migration 21 found {count} duplicate Telegram native message identities"
        )));
    }
    Ok(())
}
```

Update the migration transaction body to bind stats without logging raw payloads:

```rust
let stats = backfill_telegram_messages(conn).await?;
let _ = stats;
```

- [x] **Step 4: Run migration tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::telegram_item_native_identity::
```

Expected:

```text
test result: ok
```

- [x] **Step 5: Run full migration module tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::
```

Expected:

```text
test result: ok
```

- [x] **Step 6: Commit**

```powershell
git add src-tauri/src/migrations/telegram_item_native_identity.rs src-tauri/src/migrations.rs
git commit -m "feat: backfill telegram item native identities"
```

## Task 3: Non-Telegram Partial Upserts

**Files:**
- Modify: `src-tauri/src/sources/test_support.rs`
- Modify: `src-tauri/src/sources/items.rs`

- [x] **Step 1: Add test-support helpers for the replacement indexes**

In `src-tauri/src/sources/test_support.rs`, replace `create items unique index` in `memory_pool_with_source_items_and_topics` with a call to a new helper:

```rust
create_item_identity_indexes(&pool).await;
```

Add:

```rust
pub(crate) async fn create_item_identity_indexes(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS ux_items_non_telegram_external
            ON items(source_id, external_id)
            WHERE item_kind <> 'telegram_message';

        CREATE INDEX IF NOT EXISTS idx_items_source_external
            ON items(source_id, external_id);
        "#,
    )
    .execute(pool)
    .await
    .expect("create item identity indexes");
}
```

- [x] **Step 2: Add RED tests proving YouTube upserts use the partial unique index**

In `src-tauri/src/sources/items.rs`, inside tests, add:

```rust
#[tokio::test]
async fn youtube_transcript_upsert_targets_non_telegram_partial_unique_index() {
    let pool = memory_pool_with_source_items_and_topics().await;
    sqlx::query("DROP INDEX IF EXISTS idx_items_ext")
        .execute(&pool)
        .await
        .expect("drop legacy index fixture");
    crate::sources::test_support::create_item_identity_indexes(&pool).await;

    let mut tx = pool.begin().await.expect("begin transaction");
    let first_id = upsert_youtube_transcript_item(
        &mut tx,
        1,
        "transcript:video01:en:manual",
        Some("Demo Channel"),
        1_700_000_000,
        "old transcript",
        &serde_json::json!({ "version": 1 }),
    )
    .await
    .expect("insert transcript");
    let second_id = upsert_youtube_transcript_item(
        &mut tx,
        1,
        "transcript:video01:en:manual",
        Some("Demo Channel"),
        1_700_000_001,
        "new transcript",
        &serde_json::json!({ "version": 2 }),
    )
    .await
    .expect("update transcript");
    tx.commit().await.expect("commit");

    assert_eq!(first_id, second_id);
}

#[tokio::test]
async fn youtube_comment_upsert_targets_non_telegram_partial_unique_index() {
    let pool = memory_pool_with_source_items_and_topics().await;
    sqlx::query("DROP INDEX IF EXISTS idx_items_ext")
        .execute(&pool)
        .await
        .expect("drop legacy index fixture");
    crate::sources::test_support::create_item_identity_indexes(&pool).await;

    let mut tx = pool.begin().await.expect("begin transaction");
    let mut comment = crate::youtube::dto::YoutubeComment {
        comment_id: "UgPartial".to_string(),
        parent_comment_id: None,
        is_reply: false,
        author: Some("Alice".to_string()),
        author_channel_id: None,
        author_channel_url: None,
        published_at: 1_700_000_000,
        text: "old comment".to_string(),
        like_count: Some(1),
        is_pinned: None,
        is_hearted: None,
        raw_payload: serde_json::json!({ "id": "UgPartial" }),
    };
    let first_id = upsert_youtube_comment_item(&mut tx, 1, &comment)
        .await
        .expect("insert comment");
    comment.text = "new comment".to_string();
    comment.like_count = Some(5);
    let second_id = upsert_youtube_comment_item(&mut tx, 1, &comment)
        .await
        .expect("update comment");
    tx.commit().await.expect("commit");

    assert_eq!(first_id, second_id);
}
```

- [x] **Step 3: Run YouTube upsert tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::youtube_transcript_upsert_targets_non_telegram_partial_unique_index sources::items::youtube_comment_upsert_targets_non_telegram_partial_unique_index
```

Expected: fail with a SQL conflict-target error until the `WHERE item_kind <> 'telegram_message'` conflict target is added.

- [x] **Step 4: Update YouTube upserts to target the partial unique index**

In both `upsert_youtube_transcript_item` and `upsert_youtube_comment_item`, replace:

```sql
ON CONFLICT(source_id, external_id) DO UPDATE SET
```

with:

```sql
ON CONFLICT(source_id, external_id)
WHERE item_kind <> 'telegram_message'
DO UPDATE SET
```

Keep the existing `DO UPDATE SET` column lists unchanged.

- [x] **Step 5: Run YouTube upsert tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::youtube_transcript_upsert_targets_non_telegram_partial_unique_index sources::items::youtube_comment_upsert_targets_non_telegram_partial_unique_index
```

Expected:

```text
test result: ok
```

- [x] **Step 6: Run all item tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::
```

Expected:

```text
test result: ok
```

- [x] **Step 7: Commit**

```powershell
git add src-tauri/src/sources/test_support.rs src-tauri/src/sources/items.rs
git commit -m "feat: target non-telegram item upsert uniqueness"
```

## Task 4: Telegram Insert Helper And Runtime Identity Types

**Files:**
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/test_support.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/mod.rs`

- [x] **Step 1: Add Telegram identity types**

In `src-tauri/src/sources/types.rs`, add constants:

```rust
pub(crate) const TELEGRAM_PEER_KIND_CHANNEL: &str = "channel";
pub(crate) const TELEGRAM_PEER_KIND_CHAT: &str = "chat";
pub(crate) const TELEGRAM_PEER_KIND_USER: &str = "user";
```

Add structs:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelegramMessageIdentity {
    /// Telegram history/origin peer for this message, not necessarily the current source peer.
    pub(crate) history_peer_kind: String,
    pub(crate) history_peer_id: i64,
    pub(crate) telegram_message_id: i64,
    pub(crate) migration_domain: Option<String>,
    pub(crate) is_migrated_history: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelegramSourcePeerIdentity {
    pub(crate) peer_kind: String,
    pub(crate) peer_id: i64,
}
```

Add validation helpers:

```rust
impl TelegramMessageIdentity {
    pub(crate) fn validate(&self) -> crate::error::AppResult<()> {
        if !matches!(
            self.history_peer_kind.as_str(),
            TELEGRAM_PEER_KIND_CHANNEL | TELEGRAM_PEER_KIND_CHAT | TELEGRAM_PEER_KIND_USER
        ) {
            return Err(crate::error::AppError::validation(format!(
                "Unsupported Telegram history peer kind '{}'",
                self.history_peer_kind
            )));
        }
        if self.telegram_message_id <= 0 {
            return Err(crate::error::AppError::validation(
                "Telegram message id must be positive",
            ));
        }
        Ok(())
    }
}
```

Add tests near the existing type tests:

```rust
#[test]
fn telegram_message_identity_validation_rejects_invalid_values() {
    let invalid_kind = TelegramMessageIdentity {
        history_peer_kind: "supergroup".to_string(),
        history_peer_id: 1,
        telegram_message_id: 1,
        migration_domain: None,
        is_migrated_history: false,
    };
    assert_eq!(
        invalid_kind.validate().expect_err("reject kind").kind,
        crate::error::AppErrorKind::Validation
    );

    let invalid_message = TelegramMessageIdentity {
        history_peer_kind: TELEGRAM_PEER_KIND_CHANNEL.to_string(),
        history_peer_id: 1,
        telegram_message_id: 0,
        migration_domain: None,
        is_migrated_history: false,
    };
    assert_eq!(
        invalid_message.validate().expect_err("reject id").kind,
        crate::error::AppErrorKind::Validation
    );
}
```

- [x] **Step 2: Add test helper for `telegram_messages`**

In `src-tauri/src/sources/test_support.rs`, add:

```rust
pub(crate) async fn create_telegram_messages_table(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(crate::migrations::telegram_item_native_identity::TELEGRAM_MESSAGES_SCHEMA_SQL)
        .execute(pool)
        .await
        .expect("create telegram_messages");
}
```

Make `TELEGRAM_MESSAGES_SCHEMA_SQL` in `telegram_item_native_identity.rs` visible to tests and source helpers:

```rust
pub(crate) const TELEGRAM_MESSAGES_SCHEMA_SQL: &str = r#"...";
```

Call `create_telegram_messages_table(&pool).await;` from `memory_pool_with_source_items_and_topics`.

- [x] **Step 3: Add RED tests for Telegram native insert behavior**

In `src-tauri/src/sources/items.rs`, add tests:

```rust
#[tokio::test]
async fn insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload() {
    let pool = memory_pool_with_source_items_and_topics().await;
    let identity = TelegramMessageIdentity {
        history_peer_kind: "channel".to_string(),
        history_peer_id: 12345,
        telegram_message_id: 42,
        migration_domain: None,
        is_migrated_history: false,
    };

    let inserted = insert_telegram_source_item(
        &pool,
        1,
        identity.clone(),
        telegram_insert("42", "first payload"),
    )
    .await
    .expect("insert first");
    assert!(inserted);

    let duplicate = insert_telegram_source_item(
        &pool,
        1,
        identity,
        telegram_insert("42", "second payload"),
    )
    .await
    .expect("skip duplicate");
    assert!(!duplicate);

    let content: Vec<u8> =
        sqlx::query_scalar("SELECT content_zstd FROM items WHERE source_id = 1 AND external_id = '42'")
            .fetch_one(&pool)
            .await
            .expect("load content");
    assert_eq!(
        decompress_text(&content).expect("decode content"),
        "first payload"
    );

    let child_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_messages")
        .fetch_one(&pool)
        .await
        .expect("count child rows");
    assert_eq!(child_count, 1);
}

#[tokio::test]
async fn insert_telegram_source_item_allows_same_message_id_in_different_history_domains() {
    let pool = memory_pool_with_source_items_and_topics().await;

    let first = TelegramMessageIdentity {
        history_peer_kind: "channel".to_string(),
        history_peer_id: 12345,
        telegram_message_id: 42,
        migration_domain: None,
        is_migrated_history: false,
    };
    let second = TelegramMessageIdentity {
        history_peer_kind: "chat".to_string(),
        history_peer_id: 777,
        telegram_message_id: 42,
        migration_domain: Some("migrated_from_chat".to_string()),
        is_migrated_history: true,
    };

    assert!(
        insert_telegram_source_item(&pool, 1, first, telegram_insert("42", "current"))
            .await
            .expect("insert current")
    );
    assert!(
        insert_telegram_source_item(&pool, 1, second, telegram_insert("42", "migrated"))
            .await
            .expect("insert migrated")
    );

    let item_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM items WHERE source_id = 1 AND external_id = '42'",
    )
    .fetch_one(&pool)
    .await
    .expect("count items");
    assert_eq!(item_count, 2);
}

fn telegram_insert(external_id: &str, content: &str) -> SourceItemInsert {
    SourceItemInsert {
        external_id: external_id.to_string(),
        item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
        author: Some("alice".to_string()),
        published_at: 1234,
        payload: ExtractedItemPayload {
            content: Some(content.to_string()),
            content_kind: CONTENT_KIND_TEXT_ONLY,
            media: None,
        },
        raw_data: serde_json::to_vec(&serde_json::json!({ "id": external_id }))
            .expect("raw json"),
        telegram_context: TelegramItemContext::default(),
    }
}
```

Update the test imports to include `insert_telegram_source_item` and `TelegramMessageIdentity`.

- [x] **Step 4: Run Telegram insert tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::insert_telegram_source_item_
```

Expected: fail because the helper does not exist.

- [x] **Step 5: Implement `insert_telegram_source_item`**

In `src-tauri/src/sources/items.rs`, import `TelegramMessageIdentity`.

Extract shared compression into:

```rust
struct PreparedSourceItem {
    content_zstd: Option<Vec<u8>>,
    raw_data_zstd: Vec<u8>,
    content_kind: String,
    has_media: bool,
    media_kind: Option<String>,
    media_metadata_zstd: Option<Vec<u8>>,
}

fn prepare_source_item(item: &SourceItemInsert) -> AppResult<Option<PreparedSourceItem>> {
    let content_zstd = item
        .payload
        .content
        .as_deref()
        .map(compress_text)
        .transpose()
        .map_err(AppError::internal)?;
    let media_kind = item.payload.media.as_ref().map(|media| media.kind.clone());
    let media_metadata_zstd = item
        .payload
        .media
        .as_ref()
        .map(|media| encode_media_metadata(&media.metadata))
        .transpose()
        .map_err(AppError::internal)?;

    if content_zstd.is_none() && media_metadata_zstd.is_none() {
        return Ok(None);
    }

    let raw_data_zstd = compress_json_bytes(&item.raw_data).map_err(AppError::internal)?;
    Ok(Some(PreparedSourceItem {
        content_zstd,
        raw_data_zstd,
        content_kind: item.payload.content_kind.clone(),
        has_media: item.payload.media.is_some(),
        media_kind,
        media_metadata_zstd,
    }))
}
```

Add:

```rust
pub(crate) async fn insert_telegram_source_item(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
) -> AppResult<bool> {
    identity.validate()?;
    if item.item_kind != crate::sources::types::ITEM_KIND_TELEGRAM_MESSAGE {
        return Err(AppError::validation(format!(
            "insert_telegram_source_item requires item_kind '{}'",
            crate::sources::types::ITEM_KIND_TELEGRAM_MESSAGE
        )));
    }
    let Some(prepared) = prepare_source_item(&item)? else {
        return Ok(false);
    };

    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result = async {
        let existing: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT item_id
            FROM telegram_messages
            WHERE source_id = ?
              AND history_peer_kind = ?
              AND history_peer_id = ?
              AND telegram_message_id = ?
            "#,
        )
        .bind(source_id)
        .bind(&identity.history_peer_kind)
        .bind(identity.history_peer_id)
        .bind(identity.telegram_message_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(AppError::database)?;
        if existing.is_some() {
            return Ok(false);
        }

        let item_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                item_kind,
                author,
                published_at,
                ingested_at,
                content_zstd,
                raw_data_zstd,
                content_kind,
                has_media,
                media_kind,
                media_metadata_zstd,
                reply_to_msg_id,
                reply_to_peer_kind,
                reply_to_peer_id,
                reply_to_top_id,
                reaction_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(source_id)
        .bind(identity.telegram_message_id.to_string())
        .bind(&item.item_kind)
        .bind(&item.author)
        .bind(item.published_at)
        .bind(now_secs())
        .bind(prepared.content_zstd)
        .bind(prepared.raw_data_zstd)
        .bind(prepared.content_kind)
        .bind(prepared.has_media)
        .bind(prepared.media_kind)
        .bind(prepared.media_metadata_zstd)
        .bind(item.telegram_context.reply_to_msg_id)
        .bind(&item.telegram_context.reply_to_peer_kind)
        .bind(&item.telegram_context.reply_to_peer_id)
        .bind(item.telegram_context.reply_to_top_id)
        .bind(item.telegram_context.reaction_count)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

        sqlx::query(
            r#"
            INSERT INTO telegram_messages (
                item_id,
                source_id,
                history_peer_kind,
                history_peer_id,
                telegram_message_id,
                migration_domain,
                is_migrated_history,
                reply_to_msg_id,
                reply_to_peer_kind,
                reply_to_peer_id,
                reply_to_top_id,
                reaction_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(item_id)
        .bind(source_id)
        .bind(&identity.history_peer_kind)
        .bind(identity.history_peer_id)
        .bind(identity.telegram_message_id)
        .bind(&identity.migration_domain)
        .bind(i64::from(identity.is_migrated_history))
        .bind(item.telegram_context.reply_to_msg_id)
        .bind(&item.telegram_context.reply_to_peer_kind)
        .bind(
            item.telegram_context
                .reply_to_peer_id
                .as_deref()
                .and_then(|value| value.parse::<i64>().ok()),
        )
        .bind(item.telegram_context.reply_to_top_id)
        .bind(item.telegram_context.reaction_count)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

        Ok(true)
    }
    .await;

    match result {
        Ok(inserted) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
            Ok(inserted)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            if error.kind == crate::error::AppErrorKind::Conflict
                || error.message.contains("telegram_messages")
            {
                return Ok(false);
            }
            Err(error)
        }
    }
}
```

Keep `insert_source_item` temporarily for compilation, but do not use it from Telegram paths after later tasks. Update `src-tauri/src/sources/mod.rs` exports:

```rust
pub(crate) use items::{
    insert_source_item, insert_telegram_source_item, upsert_youtube_comment_item,
    upsert_youtube_transcript_item, SourceItemInsert, TelegramItemContext,
};
```

- [x] **Step 6: Run Telegram insert tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::insert_telegram_source_item_
```

Expected:

```text
test result: ok
```

- [x] **Step 7: Run item and type tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items:: sources::types::
```

Expected:

```text
test result: ok
```

- [x] **Step 8: Commit**

```powershell
git add src-tauri/src/sources/types.rs src-tauri/src/sources/test_support.rs src-tauri/src/sources/items.rs src-tauri/src/sources/mod.rs src-tauri/src/migrations/telegram_item_native_identity.rs
git commit -m "feat: insert telegram items by native identity"
```

## Task 5: Normal Telegram Sync Uses Native Identity

**Files:**
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/sources/items.rs`

- [x] **Step 1: Add RED unit test for live sync identity derivation**

In `src-tauri/src/sources/sync.rs`, add a pure fallback test that does not require constructing a grammers `Message`:

```rust
#[test]
fn fallback_peer_identity_uses_telegram_history_peer_vocabulary() {
    use grammers_session::types::{PeerAuth, PeerId, PeerRef};

    let identity = fallback_message_identity_for_test(
        PeerRef {
            id: PeerId::channel(12345),
            auth: PeerAuth::from_hash(99),
        },
        42,
    );

    assert_eq!(identity.history_peer_kind, "channel");
    assert_eq!(identity.history_peer_id, 12345);
    assert_eq!(identity.telegram_message_id, 42);
    assert_eq!(identity.migration_domain, None);
    assert!(!identity.is_migrated_history);
}
```

- [x] **Step 2: Run sync tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::sync::fallback_peer_identity_uses_telegram_history_peer_vocabulary
```

Expected: fail because the helper is not implemented or grammers peer mapping is missing.

- [x] **Step 3: Implement sync identity helpers and wire insert path**

In `src-tauri/src/sources/sync.rs`, replace imports:

```rust
use super::items::{
    build_raw_payload, extract_telegram_context, insert_telegram_source_item, message_author,
    SourceItemInsert,
};
use super::types::{now_secs, SourceSyncTarget, TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE};
use grammers_session::types::PeerKind;
```

Add:

```rust
fn fallback_message_identity(
    fallback_peer: grammers_session::types::PeerRef,
    telegram_message_id: i64,
) -> TelegramMessageIdentity {
    let history_peer_kind = match fallback_peer.id.kind() {
        PeerKind::User | PeerKind::UserSelf => "user",
        PeerKind::Chat => "chat",
        PeerKind::Channel => "channel",
    }
    .to_string();

    TelegramMessageIdentity {
        history_peer_kind,
        history_peer_id: fallback_peer.id.bare_id(),
        telegram_message_id,
        migration_domain: None,
        is_migrated_history: false,
    }
}

#[cfg(test)]
fn fallback_message_identity_for_test(
    fallback_peer: grammers_session::types::PeerRef,
    telegram_message_id: i64,
) -> TelegramMessageIdentity {
    fallback_message_identity(fallback_peer, telegram_message_id)
}
```

In `persist_items`, replace:

```rust
let inserted_item = insert_source_item(
```

with:

```rust
let identity = fallback_message_identity(peer, message_id);
let inserted_item = insert_telegram_source_item(
```

and pass `identity` before `SourceItemInsert`.

- [x] **Step 4: Run sync and item tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::sync:: sources::items::
```

Expected:

```text
test result: ok
```

- [x] **Step 5: Run containment scan for old Telegram duplicate path**

Run:

```powershell
rg -n "insert_source_item\\(" src-tauri\src\sources src-tauri\src\takeout_import
```

Expected at this point: `insert_source_item` may still appear in Takeout and tests, but no normal `src-tauri\src\sources\sync.rs` call remains.

- [x] **Step 6: Commit**

```powershell
git add src-tauri/src/sources/sync.rs src-tauri/src/sources/items.rs
git commit -m "feat: sync telegram items by native identity"
```

## Task 6: Takeout Raw Identity Propagation

**Files:**
- Modify: `src-tauri/src/takeout_import/raw_parse.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`

- [x] **Step 1: Add RED raw-parse test for history peer identity**

In `src-tauri/src/takeout_import/raw_parse.rs`, add assertions to `parses_text_message_with_reply_and_reactions`:

```rust
assert_eq!(
    item.telegram_identity.as_ref().expect("identity").history_peer_kind,
    "channel"
);
assert_eq!(
    item.telegram_identity.as_ref().expect("identity").history_peer_id,
    10
);
assert_eq!(
    item.telegram_identity.as_ref().expect("identity").telegram_message_id,
    42
);
```

Add a migrated-history-style parser test that uses a different raw `peer_id` while keeping the same message id:

```rust
#[test]
fn parse_raw_message_carries_raw_history_peer_for_overlapping_message_ids() {
    let mut current = raw_message(42);
    current.message = "current".to_string();
    current.peer_id = peer_channel(12345);
    let mut migrated = raw_message(42);
    migrated.message = "migrated".to_string();
    migrated.peer_id = tl::types::PeerChat { chat_id: 777 }.into();

    let current_item = parse_raw_message(&None, current)
        .expect("parse current")
        .expect("current item");
    let migrated_item = parse_raw_message(&None, migrated)
        .expect("parse migrated")
        .expect("migrated item");

    assert_eq!(
        current_item.telegram_identity.as_ref().unwrap().history_peer_kind,
        "channel"
    );
    assert_eq!(
        migrated_item.telegram_identity.as_ref().unwrap().history_peer_kind,
        "chat"
    );
    assert_eq!(
        migrated_item.telegram_identity.as_ref().unwrap().history_peer_id,
        777
    );
    assert_eq!(current_item.external_id, migrated_item.external_id);
}
```

- [x] **Step 2: Run raw parser tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_import::raw_parse::
```

Expected: fail because `SourceItemInsert` has no `telegram_identity`.

- [x] **Step 3: Add optional identity to `SourceItemInsert` and parse it**

In `src-tauri/src/sources/items.rs`, update `SourceItemInsert`:

```rust
pub(crate) struct SourceItemInsert {
    pub(crate) external_id: String,
    pub(crate) item_kind: String,
    pub(crate) author: Option<String>,
    pub(crate) published_at: i64,
    pub(crate) payload: ExtractedItemPayload,
    pub(crate) raw_data: Vec<u8>,
    pub(crate) telegram_context: TelegramItemContext,
    pub(crate) telegram_identity: Option<TelegramMessageIdentity>,
}
```

Update all constructors in tests and YouTube code to set `telegram_identity: None`.

In `raw_parse.rs`, add:

```rust
fn raw_message_identity(message: &tl::types::Message) -> crate::sources::TelegramMessageIdentity {
    let (history_peer_kind, history_peer_id) = match &message.peer_id {
        tl::enums::Peer::User(peer) => ("user", peer.user_id),
        tl::enums::Peer::Chat(peer) => ("chat", peer.chat_id),
        tl::enums::Peer::Channel(peer) => ("channel", peer.channel_id),
    };

    crate::sources::TelegramMessageIdentity {
        history_peer_kind: history_peer_kind.to_string(),
        history_peer_id,
        telegram_message_id: i64::from(message.id),
        migration_domain: None,
        is_migrated_history: false,
    }
}
```

Set the parsed item field:

```rust
telegram_identity: Some(raw_message_identity(&message)),
```

- [x] **Step 4: Wire Takeout import through `insert_telegram_source_item`**

In `src-tauri/src/takeout_import/mod.rs`, replace imports:

```rust
finalize_sync, insert_telegram_source_item, load_source, require_source_identity_ready,
```

In `import_takeout_history_pages`, replace:

```rust
if insert_source_item(&pool, source.id, item).await? {
```

with:

```rust
let identity = item.telegram_identity.clone().ok_or_else(|| {
    AppError::validation("Parsed Takeout Telegram item is missing native message identity")
})?;
if insert_telegram_source_item(&pool, source.id, identity, item).await? {
```

- [x] **Step 5: Add Takeout boundary test**

In `src-tauri/src/takeout_import/mod.rs`, extend the test imports:

```rust
use grammers_client::tl;
use crate::sources::insert_telegram_source_item;
```

Add this test and local helper:

```rust
#[tokio::test]
async fn takeout_parsed_items_with_same_message_id_insert_under_different_history_peers() {
    let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;

    let mut current = takeout_raw_message_for_identity_test(42, tl::types::PeerChannel {
        channel_id: 12345,
    }.into(), "current");
    let mut migrated = takeout_raw_message_for_identity_test(42, tl::types::PeerChat {
        chat_id: 777,
    }.into(), "migrated");

    let current_item = raw_parse::parse_raw_message(&None, current)
        .expect("parse current")
        .expect("current item");
    let current_identity = current_item
        .telegram_identity
        .clone()
        .expect("current identity");
    let migrated_item = raw_parse::parse_raw_message(&None, migrated)
        .expect("parse migrated")
        .expect("migrated item");
    let migrated_identity = migrated_item
        .telegram_identity
        .clone()
        .expect("migrated identity");

    assert!(
        insert_telegram_source_item(&pool, 1, current_identity, current_item)
            .await
            .expect("insert current")
    );
    assert!(
        insert_telegram_source_item(&pool, 1, migrated_identity, migrated_item)
            .await
            .expect("insert migrated")
    );

    let item_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM items WHERE source_id = 1 AND external_id = '42'",
    )
    .fetch_one(&pool)
    .await
    .expect("count overlapping ids");
    assert_eq!(item_count, 2);
}

fn takeout_raw_message_for_identity_test(
    id: i32,
    peer_id: tl::enums::Peer,
    text: &str,
) -> tl::types::Message {
    tl::types::Message {
        out: false,
        mentioned: false,
        media_unread: false,
        silent: false,
        post: false,
        from_scheduled: false,
        legacy: false,
        edit_hide: false,
        pinned: false,
        noforwards: false,
        invert_media: false,
        offline: false,
        video_processing_pending: false,
        paid_suggested_post_stars: false,
        paid_suggested_post_ton: false,
        id,
        from_id: None,
        from_boosts_applied: None,
        peer_id,
        saved_peer_id: None,
        fwd_from: None,
        via_bot_id: None,
        via_business_bot_id: None,
        reply_to: None,
        date: 1234,
        message: text.to_string(),
        media: None,
        reply_markup: None,
        entities: None,
        views: None,
        forwards: None,
        replies: None,
        edit_date: None,
        post_author: None,
        grouped_id: None,
        reactions: None,
        restriction_reason: None,
        ttl_period: None,
        quick_reply_shortcut_id: None,
        effect: None,
        factcheck: None,
        report_delivery_until_date: None,
        paid_message_stars: None,
        suggested_post: None,
        schedule_repeat_period: None,
        summary_from_language: None,
    }
}
```

- [x] **Step 6: Run Takeout and item tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_import::raw_parse:: takeout_import:: sources::items::
```

Expected:

```text
test result: ok
```

- [x] **Step 7: Containment scan for generic Telegram insert path**

Run:

```powershell
rg -n "insert_source_item\\(" src-tauri\src\sources src-tauri\src\takeout_import
```

Expected: no production normal sync or Takeout import calls. Test references may remain only if they specifically exercise legacy/non-Telegram helper behavior.

- [x] **Step 8: Commit**

```powershell
git add src-tauri/src/sources/items.rs src-tauri/src/takeout_import/raw_parse.rs src-tauri/src/takeout_import/mod.rs
git commit -m "feat: propagate takeout telegram history identity"
```

## Task 7: Topic Resolution Prefers Typed Message Identity

**Files:**
- Modify: `src-tauri/src/forum_topics.rs`
- Modify: `src-tauri/src/sources/items/query.rs`
- Modify: `src-tauri/src/notebooklm_export/query.rs`

- [x] **Step 1: Add RED unit test for typed join text and isolated legacy fallback**

In `src-tauri/src/forum_topics.rs`, update tests with:

```rust
#[test]
fn resolved_topic_join_prefers_typed_telegram_message_identity() {
    let join = resolved_topic_join(&ResolvedTopicAliases {
        item: "items",
        telegram_message: "telegram_messages",
        topic: "forum_topics",
        matched_topic: "matched_topics",
    });

    assert!(join.contains("LEFT JOIN telegram_messages AS telegram_messages"));
    assert!(join.contains("telegram_messages.telegram_message_id = forum_topics.top_message_id"));
    assert!(join.contains("legacy_external_id_message_id_expr"));
}
```

The final `contains` assertion should target the helper name in Rust source if the generated SQL should not literally contain it. Use:

```rust
let source = std::fs::read_to_string("src/forum_topics.rs").expect("read forum_topics.rs");
assert!(source.contains("legacy_external_id_message_id_expr"));
```

- [x] **Step 2: Run forum topic tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml forum_topics::
```

Expected: fail because `ResolvedTopicAliases` has no `telegram_message` alias and the join does not use typed ids.

- [x] **Step 3: Implement typed topic join with explicit legacy fallback**

In `src-tauri/src/forum_topics.rs`, change aliases:

```rust
pub(crate) struct ResolvedTopicAliases<'a> {
    pub(crate) item: &'a str,
    pub(crate) telegram_message: &'a str,
    pub(crate) topic: &'a str,
    pub(crate) matched_topic: &'a str,
}
```

Add:

```rust
pub(crate) fn legacy_external_id_message_id_expr(item: &str) -> String {
    format!(
        r#"{item}.external_id <> ''
            AND {item}.external_id NOT GLOB '*[^0-9]*'
            AND CAST({item}.external_id AS INTEGER)"#
    )
}
```

Update `resolved_topic_join` to include the child join before the topic join:

```rust
format!(
    "LEFT JOIN telegram_messages AS {telegram_message}
  ON {telegram_message}.item_id = {item}.id
LEFT JOIN telegram_forum_topics AS {topic}
  ON {}",
    resolved_topic_predicate(aliases),
    item = aliases.item,
    telegram_message = aliases.telegram_message,
    topic = aliases.topic
)
```

In `resolved_topic_predicate`, replace root-message comparisons with preferred typed branch:

```sql
(
    {item}.reply_to_top_id IS NULL
    AND {telegram_message}.telegram_message_id = {topic}.top_message_id
)
OR (
    {item}.reply_to_top_id IS NULL
    AND {telegram_message}.item_id IS NULL
    AND {legacy_external_id_expr} = {topic}.top_message_id
)
```

Also update the `NOT EXISTS` branch to prefer typed `telegram_messages` and only use the legacy expression when the typed child row is absent.

- [x] **Step 4: Update query call sites**

In `src-tauri/src/sources/items/query.rs`, update:

```rust
let topic_join = resolved_topic_join(&ResolvedTopicAliases {
    item: "items",
    telegram_message: "telegram_messages",
    topic: "forum_topics",
    matched_topic: "matched_topics",
});
```

In `src-tauri/src/notebooklm_export/query.rs`, update the same call in `base_query`.

- [x] **Step 5: Add browsing/export behavior tests for typed root match**

In `src-tauri/src/sources/items/query.rs`, add a typed-root case to `load_item_rows_attaches_topic_metadata_and_root_matches`:

```rust
sqlx::query(
    "INSERT INTO telegram_messages (item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id)
     VALUES (1, 1, 'channel', 12345, 700)",
)
.execute(&pool)
.await
.expect("insert typed message identity");
```

Then change item `1` external id from `"700"` to `"not-numeric-root"` and keep the assertion:

```rust
assert_eq!(rows[0].forum_topic_id, Some(200));
```

In `src-tauri/src/notebooklm_export/query.rs`, add the same pattern to `load_export_messages_attaches_topic_metadata_for_reply_and_root_messages`: root item `external_id = 'not-numeric-root'`, typed child `telegram_message_id = 700`, expected topic metadata still attaches.

- [x] **Step 6: Run topic/browser/export tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml forum_topics:: sources::items::query:: notebooklm_export::query::
```

Expected:

```text
test result: ok
```

- [x] **Step 7: Containment scan for external-id integer casts**

Run:

```powershell
rg -n "CAST\\(.*external_id AS INTEGER\\)|external_id NOT GLOB" src-tauri\src
```

Expected: matches only in `src-tauri\src\forum_topics.rs` inside `legacy_external_id_message_id_expr` or tests.

- [x] **Step 8: Commit**

```powershell
git add src-tauri/src/forum_topics.rs src-tauri/src/sources/items/query.rs src-tauri/src/notebooklm_export/query.rs
git commit -m "feat: resolve telegram topics by typed message identity"
```

## Task 8: Legacy Message Refs Resolve Safely

**Files:**
- Modify: `src-tauri/src/analysis/trace.rs`
- Modify: `src-tauri/src/analysis/mod.rs`

- [x] **Step 1: Add RED trace tests for unique and ambiguous legacy message refs**

In `src-tauri/src/analysis/trace.rs`, add:

```rust
#[test]
fn build_trace_refs_resolves_unique_legacy_message_ref_by_external_message_id() {
    let refs = vec!["s1-m42".to_string()];
    let corpus = vec![CorpusMessage {
        item_id: 900,
        source_id: 1,
        external_id: "42".to_string(),
        published_at: 1,
        author: None,
        content: "telegram".to_string(),
        r#ref: "s1-i900".to_string(),
        item_kind: Some("telegram_message".to_string()),
        source_type: Some("telegram".to_string()),
        source_subtype: Some("supergroup".to_string()),
        metadata_zstd: None,
    }];

    let trace_refs = try_build_trace_refs(&refs, &corpus).expect("resolve refs");

    assert_eq!(trace_refs.len(), 1);
    assert_eq!(trace_refs[0].item_id, 900);
    assert_eq!(trace_refs[0].r#ref, "s1-m42");
}

#[test]
fn build_trace_refs_returns_conflict_for_ambiguous_legacy_message_ref() {
    let refs = vec!["s1-m42".to_string()];
    let first = CorpusMessage {
        item_id: 900,
        source_id: 1,
        external_id: "42".to_string(),
        published_at: 1,
        author: None,
        content: "current".to_string(),
        r#ref: "s1-i900".to_string(),
        item_kind: Some("telegram_message".to_string()),
        source_type: Some("telegram".to_string()),
        source_subtype: Some("supergroup".to_string()),
        metadata_zstd: None,
    };
    let second = CorpusMessage {
        item_id: 901,
        content: "migrated".to_string(),
        r#ref: "s1-i901".to_string(),
        ..first.clone()
    };

    let error = try_build_trace_refs(&refs, &[first, second])
        .expect_err("ambiguous legacy ref conflicts");

    assert_eq!(error.kind, crate::error::AppErrorKind::Conflict);
}
```

- [x] **Step 2: Run trace tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::trace::build_trace_refs_resolves_unique_legacy_message_ref_by_external_message_id analysis::trace::build_trace_refs_returns_conflict_for_ambiguous_legacy_message_ref
```

Expected: fail because `try_build_trace_refs` does not exist and old fallback treats `-m42` as local item id `42`.

- [x] **Step 3: Implement fallible trace ref resolution**

In `src-tauri/src/analysis/trace.rs`, import:

```rust
use crate::error::{AppError, AppResult};
```

Extend `ParsedTraceRef`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TraceRefKind {
    Item,
    LegacyMessage,
}

struct ParsedTraceRef {
    source_id: i64,
    item_id: i64,
    timestamp_ms: Option<i64>,
    kind: TraceRefKind,
}
```

Set `kind` in `parse_structured_ref` based on `-i` or `-m`.

Add:

```rust
pub(crate) fn try_build_trace_refs(
    refs: &[String],
    corpus: &[CorpusMessage],
) -> AppResult<Vec<AnalysisTraceRef>> {
    let mut trace_refs = Vec::new();

    for reference in refs {
        if let Some(message) = find_trace_message_checked(reference, corpus)? {
            let parsed_ref = parse_structured_ref(reference);
            let (youtube_url, youtube_timestamp_seconds, youtube_display_label) =
                youtube_trace_fields(reference, message, parsed_ref.as_ref());
            trace_refs.push(AnalysisTraceRef {
                r#ref: reference.clone(),
                item_id: message.item_id,
                source_id: message.source_id,
                external_id: message.external_id.clone(),
                published_at: message.published_at,
                excerpt: clip_excerpt(&message.content, TRACE_EXCERPT_MAX_CHARS),
                youtube_url,
                youtube_timestamp_seconds,
                youtube_display_label,
                is_synthetic: is_synthetic_message(message),
            });
        }
    }

    Ok(trace_refs)
}

pub(crate) fn build_trace_refs(refs: &[String], corpus: &[CorpusMessage]) -> Vec<AnalysisTraceRef> {
    try_build_trace_refs(refs, corpus).unwrap_or_default()
}
```

Add:

```rust
fn find_trace_message_checked<'a>(
    reference: &str,
    corpus: &'a [CorpusMessage],
) -> AppResult<Option<&'a CorpusMessage>> {
    if let Some(message) = corpus.iter().find(|message| message.r#ref == reference) {
        return Ok(Some(message));
    }

    let Some(parsed) = parse_structured_ref(reference) else {
        return Ok(None);
    };

    match parsed.kind {
        TraceRefKind::Item => Ok(corpus
            .iter()
            .find(|message| {
                message.source_id == parsed.source_id && message.item_id == parsed.item_id
            })),
        TraceRefKind::LegacyMessage => {
            let candidates = corpus
                .iter()
                .filter(|message| {
                    message.source_id == parsed.source_id
                        && message.external_id == parsed.item_id.to_string()
                        && message.item_kind.as_deref() == Some("telegram_message")
                })
                .collect::<Vec<_>>();
            match candidates.len() {
                0 => Ok(None),
                1 => Ok(Some(candidates[0])),
                _ => Err(AppError::conflict(format!(
                    "Legacy Telegram ref {reference} is ambiguous across Telegram history domains"
                ))),
            }
        }
    }
}
```

Keep the old non-fallible `find_trace_message` only if tests still require it, but route it through `find_trace_message_checked(reference, corpus).ok().flatten()`.

- [x] **Step 4: Use fallible resolver in explicit API**

In `src-tauri/src/analysis/mod.rs`, replace:

```rust
Ok(build_trace_refs(&normalized_refs, &corpus))
```

with:

```rust
self::trace::try_build_trace_refs(&normalized_refs, &corpus)
```

Keep report generation using `build_trace_data` best-effort so an ambiguous LLM citation does not fail report persistence after generation.

- [x] **Step 5: Run trace and analysis tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::trace:: analysis::tests::
```

Expected:

```text
test result: ok
```

- [x] **Step 6: Commit**

```powershell
git add src-tauri/src/analysis/trace.rs src-tauri/src/analysis/mod.rs
git commit -m "feat: detect ambiguous legacy telegram refs"
```

## Task 9: Compatibility, Documentation, And Containment

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/backlog.md`
- Verify: containment scans and focused tests

- [x] **Step 1: Update database schema docs**

In `docs/database-schema.md`, add a new section after `telegram_sources`:

```markdown
### 1.3 `telegram_messages`

Stores typed native identity and Telegram message context for `telegram_message`
items. `items` remains the local item/archive container; this table gives
Telegram duplicate detection and topic/ref logic a provider-native identity.

Important fields:

- `item_id`
- `source_id`
- `history_peer_kind`
- `history_peer_id`
- `telegram_message_id`
- `migration_domain`
- `is_migrated_history`
- `reply_to_msg_id`
- `reply_to_peer_kind`
- `reply_to_peer_id`
- `reply_to_top_id`
- `reaction_count`
- `created_at`
- `updated_at`

Important constraints / indexes:

- primary key and `ON DELETE CASCADE` foreign key by `item_id`
- native Telegram identity by `(source_id, history_peer_kind, history_peer_id, telegram_message_id)`
- lookup index by `(source_id, telegram_message_id)`
- topic fallback lookup index by `(source_id, reply_to_top_id)`

Notes:

- `history_peer_kind` and `history_peer_id` identify the Telegram history/origin peer for the message, not necessarily the current resolved source peer.
- for non-migrated current history, `history_peer_*` usually equals `telegram_sources.peer_*`.
- for migrated history, `history_peer_*` identifies the original Telegram history domain.
- `migration_domain` is diagnostic/future-proofing metadata in this slice and is not used for duplicate detection, topic matching, or ref resolution.
- `telegram_messages.source_id` must equal `items.source_id`, and `telegram_messages.item_id` must point to an item whose `item_kind = 'telegram_message'`; migration/runtime tests enforce this application invariant.
```

Update the `items` section:

```markdown
- Telegram duplicate detection now uses `telegram_messages`, not `(source_id, external_id)`.
- `items.external_id` remains a compatibility/display/debug value for Telegram messages and is still populated with the Telegram message id string.
- non-Telegram item uniqueness is enforced by `ux_items_non_telegram_external` on `(source_id, external_id)` where `item_kind <> 'telegram_message'`.
- legacy lookup index `idx_items_source_external` remains non-unique for browsing, exports, and compatibility refs.
```

Update the `telegram_forum_topics` note that mentions `CAST(items.external_id AS INTEGER)`:

```markdown
- root-message fallback first uses `telegram_messages.telegram_message_id`; the old `CAST(items.external_id AS INTEGER)` path remains only for legacy Telegram rows that were not backfilled into `telegram_messages`.
```

- [x] **Step 2: Update backlog open work**

In `docs/backlog.md`, under `4.4 Database Schema Simplification`, replace:

```markdown
- [ ] continue item/document identity cleanup
```

with:

```markdown
- [ ] continue item/document identity cleanup after the Telegram native item identity slice, including topic membership materialization and a later provider-neutral document layer
```

Do not add a completed-work history note; this backlog tracks open work only.

- [x] **Step 3: Run containment scans**

Run:

```powershell
rg -n "ON CONFLICT\\(source_id, external_id\\) DO NOTHING" src-tauri\src\sources src-tauri\src\takeout_import
rg -n "ON CONFLICT\\(source_id, external_id\\) DO UPDATE SET" src-tauri\src\sources\items.rs
rg -n "CAST\\(.*items\\.external_id AS INTEGER\\)|items\\.external_id NOT GLOB" src-tauri\src
rg -n "telegram_messages" src-tauri\src src-tauri\migrations docs
```

Expected:

- no Telegram production insert uses `(source_id, external_id)` as duplicate identity;
- YouTube upserts use `ON CONFLICT(source_id, external_id) WHERE item_kind <> 'telegram_message'`;
- `items.external_id` integer casts appear only in explicit legacy fallback logic or tests;
- `telegram_messages` appears in migration, test support, Telegram insert paths, topic/query code, docs, and tests.

- [x] **Step 4: Run focused compatibility tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations:: sources::items:: sources::sync:: takeout_import:: forum_topics:: notebooklm_export::query:: analysis::trace:: analysis::corpus::
```

Expected:

```text
test result: ok
```

- [x] **Step 5: Commit**

```powershell
git add docs/database-schema.md docs/backlog.md
git commit -m "docs: document telegram item native identity"
```

## Task 10: Final Verification

**Files:**
- Modify: this plan if task checkboxes were updated during execution
- Verify: full project status

- [ ] **Step 1: Run full Rust verification**

Use `superpowers:verification-before-completion`, then run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected:

```text
test result: ok
```

`cargo fmt --check` and `git diff --check` exit 0.

- [ ] **Step 2: Run frontend verification only if frontend files changed**

If any file under `src/`, `static/`, `package.json`, or Svelte/Vite config changed, run:

```powershell
npm run check
npm test -- --run
```

Expected:

```text
0 errors
```

If no frontend files changed, record that frontend verification was not required.

- [ ] **Step 3: Final git status**

Run:

```powershell
git status --short --branch
git --no-pager log -12 --oneline --decorate
```

Expected: only expected committed branch state.

- [ ] **Step 4: Commit plan checkbox updates if they were changed**

If this plan was updated during execution:

```powershell
git add docs/superpowers/plans/2026-05-17-telegram-item-native-identity.md
git commit -m "docs: update telegram item identity plan progress"
```

- [ ] **Step 5: Finish the branch**

Use `superpowers:finishing-a-development-branch`. Present merge/push/keep/discard options after verification passes.

## Self-Review

- Spec coverage: Tasks 1-2 cover migration 21 registration, schema, best-effort backfill, skip accounting, index replacement, and post-migration integrity checks. Task 3 covers deterministic YouTube upserts before relying on the replacement uniqueness. Tasks 4-6 cover typed Telegram identity types, transactional Telegram item insert, normal sync wiring, and Takeout raw peer propagation through the production parse/insert boundary. Task 7 covers typed topic matching and isolates the legacy external-id cast fallback. Task 8 covers legacy `s{source_id}-m{message_id}` ambiguity handling. Task 9 covers docs, backlog, and containment scans. Task 10 covers final verification.
- Placeholder scan: No step uses deferred-work markers. Each code-changing task names the files, tests, commands, and expected result.
- Type consistency: The plan consistently uses `telegram_messages`, `TelegramMessageIdentity`, `history_peer_kind`, `history_peer_id`, `telegram_message_id`, `ux_telegram_messages_native_identity`, `ux_items_non_telegram_external`, and migration version 21.
