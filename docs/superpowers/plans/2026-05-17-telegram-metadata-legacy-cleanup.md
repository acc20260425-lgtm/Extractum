# Telegram Metadata Legacy Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop normal Telegram runtime paths from creating, replacing, or decoding Telegram-specific `sources.metadata_zstd` blobs while preserving typed identity, legacy repair, and YouTube metadata behavior.

**Architecture:** Keep `sources.metadata_zstd` in the schema and preserve existing Telegram blob bytes, but stop writing new Telegram blobs. The add/upsert flow writes generic source identity to `sources` and required typed identity plus available optional hints/cache to `telegram_sources` in one transaction. Startup repair uses valid typed identity first and reads legacy blobs only when required typed identity is missing or invalid.

**Tech Stack:** Rust, Tauri commands, SQLx SQLite, existing source identity repair helpers, existing Rust test harness under `src-tauri/src/sources`.

---

## File Structure

- Modify `src-tauri/src/sources/store.rs`
  - Remove normal Telegram add/upsert dependence on `SourceMetadata` and `encode_source_metadata`.
  - Add private helpers for atomic Telegram source + typed identity upsert.
  - Add tests for null metadata, legacy blob preservation, typed identity fields, rollback, source listing avatar behavior, and YouTube metadata preservation.
- Modify `src-tauri/src/sources/peer_resolution.rs`
  - Make the add-source resolution strategy helper callable from `store.rs`.
  - Make `ResolvedTelegramSource.access_hash` visible to `store.rs`.
  - Keep legacy `SourceMetadata` encode/decode available for repair/tests only.
- Modify `src-tauri/src/sources/identity_repair.rs`
  - Load existing typed rows with enough fields to validate required identity.
  - Build repair candidates from valid typed rows without decoding `sources.metadata_zstd`.
  - Decode legacy metadata only when typed identity is absent or invalid.
  - Add repair tests for typed-row-wins, optional enrichment gaps, minimal repair, and fatal canonical insufficiency.
- Modify `src-tauri/src/sources/sync.rs`
  - Add tests proving sync/avatar refresh does not create blobs and preserves existing legacy blobs.
- Modify `src-tauri/src/takeout_import/mod.rs`
  - Add tests proving Takeout source loading works from typed identity with corrupt/no source blob.
- Modify `src-tauri/src/sources/topics.rs`
  - Add a test proving forum-topic gates use typed identity with corrupt/no source blob.
- Modify `docs/database-schema.md`
  - Document Telegram `metadata_zstd` as legacy-only input for old rows.
- Modify `docs/backlog.md`
  - Replace the shipped Telegram metadata cleanup backlog item with an optional old-blob cleanup follow-up.
- Modify this plan as tasks complete.

## Task 0: Branch Guard And Baseline

**Files:**
- Read: `GEMINI.md`
- Read: `docs/superpowers/specs/2026-05-17-telegram-metadata-legacy-cleanup-design.md`
- Verify: git status and focused baseline tests

- [x] **Step 1: Confirm current branch and clean status**

Run:

```powershell
git status --short --branch
git --no-pager log -5 --oneline --decorate
```

Expected:

```text
## main
f8913a2 (HEAD -> main) docs: design telegram metadata legacy cleanup
```

If the working tree is dirty, inspect changes and do not overwrite unrelated user work.

- [x] **Step 2: Create an implementation branch or worktree**

Use `superpowers:using-git-worktrees` before execution. A safe branch name is:

```powershell
git switch -c feature/telegram-metadata-legacy-cleanup
```

If using a linked worktree, use:

```powershell
git worktree add .worktrees/telegram-metadata-legacy-cleanup -b feature/telegram-metadata-legacy-cleanup
```

- [x] **Step 3: Run focused baseline Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::
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

## Task 1: Stop Telegram Add/Upsert From Writing Source Metadata

**Files:**
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/sources/peer_resolution.rs`
- Test: `src-tauri/src/sources/store.rs`

- [x] **Step 1: Add RED tests for Telegram source-row metadata behavior**

In `src-tauri/src/sources/store.rs`, inside `#[cfg(test)] mod tests`, add these tests near the existing source-store tests.

Extend the test imports:

```rust
use crate::sources::types::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP};
```

```rust
#[tokio::test]
async fn telegram_source_upsert_inserts_null_metadata() {
    let pool = memory_pool_with_sources().await;
    let resolved = resolved_telegram_source(
        "12345",
        "Example channel",
        TELEGRAM_KIND_CHANNEL,
        Some("Example"),
        Some(77),
        None,
    );

    let source_id = upsert_telegram_source_with_identity(
        &pool,
        1,
        "@Example",
        None,
        &resolved,
        Some("1_channel_12345.jpg"),
    )
    .await
    .expect("upsert telegram source");

    let metadata: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT metadata_zstd FROM sources WHERE id = ?")
            .bind(source_id)
            .fetch_one(&pool)
            .await
            .expect("load metadata");

    assert_eq!(metadata, None);
}

#[tokio::test]
async fn telegram_source_upsert_preserves_existing_legacy_metadata_blob() {
    let pool = memory_pool_with_sources().await;
    crate::sources::test_support::create_canonical_telegram_identity_index(&pool).await;
    let legacy_blob = crate::compression::compress_json_bytes(
        br#"{"peer_identity":{"strategy":"username","username":"legacy","access_hash":11}}"#,
    )
    .expect("compress legacy metadata");
    sqlx::query(
        r#"
        INSERT INTO sources (
            id, source_type, source_subtype, account_id, external_id,
            title, metadata_zstd, is_active, is_member, created_at
        )
        VALUES (101, 'telegram', 'channel', 1, '12345', 'old', ?, 1, 1, 100)
        "#,
    )
    .bind(&legacy_blob)
    .execute(&pool)
    .await
    .expect("insert legacy source");

    let resolved = resolved_telegram_source(
        "12345",
        "Renamed channel",
        TELEGRAM_KIND_CHANNEL,
        Some("Example"),
        Some(77),
        None,
    );

    let source_id = upsert_telegram_source_with_identity(
        &pool,
        1,
        "@Example",
        None,
        &resolved,
        Some("1_channel_12345.jpg"),
    )
    .await
    .expect("upsert existing telegram source");

    assert_eq!(source_id, 101);
    let metadata: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT metadata_zstd FROM sources WHERE id = ?")
            .bind(source_id)
            .fetch_one(&pool)
            .await
            .expect("load metadata");
    assert_eq!(metadata.as_deref(), Some(legacy_blob.as_slice()));
}
```

- [x] **Step 2: Add RED tests for typed identity and rollback**

Add these tests in the same module.

```rust
#[tokio::test]
async fn telegram_source_upsert_writes_required_identity_and_available_optional_fields() {
    let pool = memory_pool_with_sources().await;
    let resolved = resolved_telegram_source(
        "12345",
        "Example channel",
        TELEGRAM_KIND_CHANNEL,
        Some("Example"),
        Some(77),
        None,
    );

    let source_id = upsert_telegram_source_with_identity(
        &pool,
        1,
        "@Example",
        None,
        &resolved,
        Some("1_channel_12345.jpg"),
    )
    .await
    .expect("upsert telegram source");

    let row: (i64, String, String, i64, String, Option<String>, Option<i64>, Option<String>) =
        sqlx::query_as(
            r#"
            SELECT account_id, source_subtype, peer_kind, peer_id,
                   resolution_strategy, username, access_hash, avatar_cache_key
            FROM telegram_sources
            WHERE source_id = ?
            "#,
        )
        .bind(source_id)
        .fetch_one(&pool)
        .await
        .expect("load typed identity");

    assert_eq!(row.0, 1);
    assert_eq!(row.1, TELEGRAM_KIND_CHANNEL);
    assert_eq!(row.2, "channel");
    assert_eq!(row.3, 12345);
    assert_eq!(row.4, "username");
    assert_eq!(row.5.as_deref(), Some("example"));
    assert_eq!(row.6, Some(77));
    assert_eq!(row.7.as_deref(), Some("1_channel_12345.jpg"));
}

#[tokio::test]
async fn telegram_source_upsert_rolls_back_source_when_typed_identity_fails() {
    let pool = memory_pool_with_sources().await;
    let resolved = resolved_telegram_source(
        "00123",
        "Invalid channel",
        TELEGRAM_KIND_CHANNEL,
        Some("Example"),
        Some(77),
        None,
    );

    let error = upsert_telegram_source_with_identity(
        &pool,
        1,
        "@Example",
        None,
        &resolved,
        Some("1_channel_00123.jpg"),
    )
    .await
    .expect_err("invalid typed identity fails");

    assert_eq!(error.kind, AppErrorKind::Validation);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sources WHERE external_id = '00123'")
        .fetch_one(&pool)
        .await
        .expect("count source rows");
    assert_eq!(count, 0);
}
```

- [x] **Step 3: Add test helper for resolved Telegram sources**

Still in `store.rs` tests, add:

```rust
fn resolved_telegram_source(
    external_id: &str,
    title: &str,
    source_subtype: &str,
    username: Option<&str>,
    access_hash: Option<i64>,
    avatar_bytes: Option<Vec<u8>>,
) -> ResolvedTelegramSource {
    ResolvedTelegramSource {
        external_id: external_id.to_string(),
        title: title.to_string(),
        source_subtype: source_subtype.to_string(),
        is_member: true,
        username: username.map(str::to_string),
        access_hash,
        avatar_bytes,
    }
}
```

- [x] **Step 4: Run RED tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml telegram_source_upsert_
```

Expected: failures because `upsert_telegram_source_with_identity` and the helper visibility changes do not exist yet.

- [x] **Step 5: Expose add resolution strategy and resolved access hash**

In `src-tauri/src/sources/peer_resolution.rs`, change `ResolvedTelegramSource.access_hash` to `pub(super)` and change `add_source_resolution_strategy` to `pub(super)`.

```rust
pub(super) struct ResolvedTelegramSource {
    pub(super) external_id: String,
    pub(super) title: String,
    pub(super) source_subtype: String,
    pub(super) is_member: bool,
    pub(super) username: Option<String>,
    pub(super) access_hash: Option<i64>,
    pub(super) avatar_bytes: Option<Vec<u8>>,
}
```

```rust
pub(super) fn add_source_resolution_strategy(
    source_ref: &str,
    source_subtype: Option<&str>,
) -> SourcePeerResolutionStrategy {
    if source_subtype.is_some() {
        return SourcePeerResolutionStrategy::Dialog;
    }

    let username = parse_username(source_ref);
    if username.is_empty() || username.chars().all(|char| char.is_ascii_digit()) {
        SourcePeerResolutionStrategy::Dialog
    } else {
        SourcePeerResolutionStrategy::Username
    }
}
```

- [x] **Step 6: Replace add flow metadata encoding with atomic typed upsert helpers**

In `src-tauri/src/sources/store.rs`, change imports from `peer_resolution` to:

```rust
use super::peer_resolution::{
    add_source_resolution_strategy, resolve_telegram_source, telegram_source_info_from_peer,
    ResolvedTelegramSource, SourcePeerResolutionStrategy,
};
```

Add the helper below `add_telegram_source` or near the existing typed upsert helper:

```rust
async fn upsert_telegram_source_with_identity(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    account_id: i64,
    source_ref: &str,
    expected_subtype: Option<&str>,
    resolved: &ResolvedTelegramSource,
    avatar_cache_key: Option<&str>,
) -> AppResult<i64> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    let result = async {
        let source_id = upsert_telegram_source_row(&mut tx, account_id, resolved).await?;
        upsert_telegram_source_identity_from_resolved(
            &mut tx,
            source_id,
            account_id,
            source_ref,
            expected_subtype,
            resolved,
            avatar_cache_key,
        )
        .await?;
        Ok(source_id)
    }
    .await;

    match result {
        Ok(source_id) => {
            tx.commit().await.map_err(AppError::database)?;
            Ok(source_id)
        }
        Err(error) => {
            tx.rollback().await.map_err(AppError::database)?;
            Err(error)
        }
    }
}

async fn upsert_telegram_source_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    account_id: i64,
    resolved: &ResolvedTelegramSource,
) -> AppResult<i64> {
    let now = now_secs();
    sqlx::query_scalar(
        r#"
        INSERT INTO sources (
            source_type,
            source_subtype,
            external_id,
            title,
            metadata_zstd,
            is_active,
            is_member,
            account_id,
            created_at
        )
        VALUES (?, ?, ?, ?, NULL, 1, ?, ?, ?)
        ON CONFLICT(account_id, source_type, source_subtype, external_id)
        WHERE source_type = 'telegram'
        DO UPDATE SET
            title = excluded.title,
            source_subtype = excluded.source_subtype,
            is_member = excluded.is_member,
            account_id = excluded.account_id,
            is_active = 1
        RETURNING id
        "#,
    )
    .bind(SourceType::Telegram.as_str())
    .bind(&resolved.source_subtype)
    .bind(&resolved.external_id)
    .bind(&resolved.title)
    .bind(resolved.is_member)
    .bind(account_id)
    .bind(now)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}
```

Change `add_telegram_source` after `avatar_cache_key` is computed to:

```rust
let pool = get_pool(&handle).await?;
let source_id = upsert_telegram_source_with_identity(
    &pool,
    request.account_id,
    &request.source_ref,
    expected_subtype,
    &resolved,
    avatar_cache_key.as_deref(),
)
.await?;

load_source_record(&handle, &pool, source_id).await
```

- [x] **Step 7: Change typed identity helper to avoid `SourceMetadata`**

Replace `upsert_telegram_source_identity_from_resolved` with:

```rust
async fn upsert_telegram_source_identity_from_resolved(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    source_id: i64,
    account_id: i64,
    source_ref: &str,
    expected_subtype: Option<&str>,
    resolved: &ResolvedTelegramSource,
    avatar_cache_key: Option<&str>,
) -> AppResult<()> {
    let source_subtype = TelegramSourceKind::from_source_subtype(&resolved.source_subtype)?;
    let peer_kind = TelegramPeerKind::from_source_subtype(source_subtype);
    let peer_id = canonical_telegram_external_id(&resolved.external_id)?;
    let resolution_strategy = match add_source_resolution_strategy(source_ref, expected_subtype) {
        SourcePeerResolutionStrategy::Username => TelegramResolutionStrategy::Username,
        SourcePeerResolutionStrategy::Dialog => TelegramResolutionStrategy::Dialog,
    };
    let username = normalize_telegram_username(resolved.username.as_deref());

    sqlx::query(
        r#"
        INSERT INTO telegram_sources (
            source_id, account_id, source_subtype, peer_kind, peer_id,
            resolution_strategy, username, access_hash, avatar_cache_key,
            identity_refreshed_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, strftime('%s','now'), strftime('%s','now'))
        ON CONFLICT(source_id) DO UPDATE SET
            account_id = excluded.account_id,
            source_subtype = excluded.source_subtype,
            peer_kind = excluded.peer_kind,
            peer_id = excluded.peer_id,
            resolution_strategy = excluded.resolution_strategy,
            username = excluded.username,
            access_hash = excluded.access_hash,
            avatar_cache_key = excluded.avatar_cache_key,
            identity_refreshed_at = excluded.identity_refreshed_at,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(source_id)
    .bind(account_id)
    .bind(source_subtype.as_str())
    .bind(peer_kind.as_str())
    .bind(peer_id)
    .bind(resolution_strategy.as_str())
    .bind(username)
    .bind(resolved.access_hash)
    .bind(avatar_cache_key)
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;

    Ok(())
}
```

- [x] **Step 8: Run store GREEN tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml telegram_source_upsert_
```

Expected: all tests with `telegram_source_upsert_` pass.

- [x] **Step 9: Run store module tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::store
```

Expected: store tests pass.

- [x] **Step 10: Commit**

Run:

```powershell
git status --short
git add src-tauri/src/sources/store.rs src-tauri/src/sources/peer_resolution.rs docs/superpowers/plans/2026-05-17-telegram-metadata-legacy-cleanup.md
git commit -m "feat: stop writing telegram source metadata blobs"
```

## Task 2: Make Repair Prefer Valid Typed Identity

**Files:**
- Modify: `src-tauri/src/sources/identity_repair.rs`
- Test: `src-tauri/src/sources/identity_repair.rs`

- [x] **Step 1: Add RED tests for valid typed row wins**

In `identity_repair.rs` tests, add:

```rust
#[tokio::test]
async fn repair_skips_malformed_metadata_when_typed_identity_is_valid() {
    let pool = memory_pool_with_sources().await;
    insert_telegram_source(&pool, 101, Some("channel"), Some(1), "12345", None).await;
    insert_existing_typed_projection(&pool, 101, "channel", "channel", 12345).await;
    sqlx::query(
        "UPDATE telegram_sources SET resolution_strategy = 'username', username = 'typed', access_hash = 77, avatar_cache_key = 'typed.jpg' WHERE source_id = 101",
    )
    .execute(&pool)
    .await
    .expect("enrich typed row");
    sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = 101")
        .execute(&pool)
        .await
        .expect("damage legacy metadata");

    let report = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect("valid typed row wins");

    assert!(report.fatal_errors.is_empty());
    let row: (String, Option<String>, Option<i64>, Option<String>) = sqlx::query_as(
        "SELECT resolution_strategy, username, access_hash, avatar_cache_key FROM telegram_sources WHERE source_id = 101",
    )
    .fetch_one(&pool)
    .await
    .expect("load typed row");
    assert_eq!(row.0, "username");
    assert_eq!(row.1.as_deref(), Some("typed"));
    assert_eq!(row.2, Some(77));
    assert_eq!(row.3.as_deref(), Some("typed.jpg"));
}

#[tokio::test]
async fn repair_ignores_optional_enrichment_gaps_when_typed_identity_is_valid() {
    let pool = memory_pool_with_sources().await;
    insert_telegram_source(&pool, 101, Some("group"), Some(1), "12345", None).await;
    insert_existing_typed_projection(&pool, 101, "group", "chat", 12345).await;
    sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = 101")
        .execute(&pool)
        .await
        .expect("damage legacy metadata");

    let report = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect("optional gaps are valid");

    assert!(report.fatal_errors.is_empty());
    let row: (String, Option<String>, Option<i64>, Option<String>) = sqlx::query_as(
        "SELECT resolution_strategy, username, access_hash, avatar_cache_key FROM telegram_sources WHERE source_id = 101",
    )
    .fetch_one(&pool)
    .await
    .expect("load typed row");
    assert_eq!(row.0, "unknown");
    assert_eq!(row.1, None);
    assert_eq!(row.2, None);
    assert_eq!(row.3, None);
}
```

- [x] **Step 2: Add RED tests for legacy fallback outcomes**

Add:

```rust
#[tokio::test]
async fn repair_creates_minimal_typed_identity_when_legacy_metadata_is_missing_or_malformed() {
    for metadata in [None, Some(b"not zstd metadata".as_slice())] {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(&pool, 101, Some("channel"), Some(1), "12345", metadata).await;

        let report = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect("canonical identity is enough");

        assert_eq!(report.repaired_sources, vec![101]);
        let row: (String, String, i64, String, Option<String>, Option<i64>, Option<String>) =
            sqlx::query_as(
                "SELECT source_subtype, peer_kind, peer_id, resolution_strategy, username, access_hash, avatar_cache_key FROM telegram_sources WHERE source_id = 101",
            )
            .fetch_one(&pool)
            .await
            .expect("load minimal typed identity");
        assert_eq!(row.0, "channel");
        assert_eq!(row.1, "channel");
        assert_eq!(row.2, 12345);
        assert_eq!(row.3, "unknown");
        assert_eq!(row.4, None);
        assert_eq!(row.5, None);
        assert_eq!(row.6, None);
    }
}

#[tokio::test]
async fn repair_fails_when_canonical_identity_is_invalid_even_with_legacy_peer_metadata() {
    let pool = memory_pool_with_sources().await;
    insert_telegram_source(
        &pool,
        101,
        Some("channel"),
        Some(1),
        "00123",
        Some(br#"{"peer_identity":{"strategy":"username","username":"Example","access_hash":77}}"#),
    )
    .await;

    let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect_err("non-canonical external id fails");

    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    assert!(error.message.contains("malformed_telegram_external_id"));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_sources")
        .fetch_one(&pool)
        .await
        .expect("count typed rows");
    assert_eq!(count, 0);
}
```

- [x] **Step 3: Run RED repair tests**

Run each focused filter separately because Cargo accepts one filter per command in this environment:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml repair_skips_malformed_metadata_when_typed_identity_is_valid
cargo test --manifest-path src-tauri/Cargo.toml repair_ignores_optional_enrichment_gaps_when_typed_identity_is_valid
cargo test --manifest-path src-tauri/Cargo.toml repair_creates_minimal_typed_identity_when_legacy_metadata_is_missing_or_malformed
cargo test --manifest-path src-tauri/Cargo.toml repair_fails_when_canonical_identity_is_invalid_even_with_legacy_peer_metadata
```

Expected: at least the first two fail because repair currently decodes legacy metadata and overwrites typed optional fields.

- [x] **Step 4: Extend existing typed projection row fields**

In `identity_repair.rs`, replace `ExistingTelegramSourceProjection` with:

```rust
#[derive(Clone, sqlx::FromRow)]
struct ExistingTelegramSourceProjection {
    source_id: i64,
    account_id: i64,
    source_subtype: String,
    peer_kind: String,
    peer_id: i64,
    resolution_strategy: String,
    username: Option<String>,
    access_hash: Option<i64>,
    avatar_cache_key: Option<String>,
}
```

Change the query in `repair_source_identity` to:

```rust
let existing_projections: Vec<ExistingTelegramSourceProjection> = sqlx::query_as(
    r#"
    SELECT source_id, account_id, source_subtype, peer_kind, peer_id,
           resolution_strategy, username, access_hash, avatar_cache_key
    FROM telegram_sources
    ORDER BY source_id
    "#,
)
.fetch_all(&mut *tx)
.await
.map_err(AppError::database)?;
```

- [x] **Step 5: Split required identity from legacy metadata decoding**

Add this required identity struct near `TelegramRepairCandidate`:

```rust
#[derive(Clone)]
struct TelegramRequiredIdentity {
    source_id: i64,
    account_id: i64,
    source_subtype: TelegramSourceKind,
    source_subtype_text: String,
    peer_kind: TelegramPeerKind,
    peer_id: i64,
}
```

Replace `candidate_from_row` with required-identity parsing plus a legacy-only candidate builder:

```rust
fn required_identity_from_row(
    row: &TelegramSourceRepairRow,
) -> Result<TelegramRequiredIdentity, SourceIdentityRepairDiagnostic> {
    let account_id = row
        .account_id
        .ok_or_else(|| SourceIdentityRepairDiagnostic {
            code: "telegram_source_missing_account".to_string(),
            source_ids: vec![row.id],
            detail: format!("Telegram source {} has no account_id", row.id),
        })?;

    let source_subtype = derive_source_subtype(row)?;
    let source_subtype_text = source_subtype.as_str().to_string();
    let peer_id = canonical_telegram_external_id(&row.external_id).map_err(|_| {
        SourceIdentityRepairDiagnostic {
            code: "malformed_telegram_external_id".to_string(),
            source_ids: vec![row.id],
            detail: format!("Telegram source {} has malformed external_id", row.id),
        }
    })?;

    Ok(TelegramRequiredIdentity {
        source_id: row.id,
        account_id,
        peer_kind: TelegramPeerKind::from_source_subtype(source_subtype),
        source_subtype,
        source_subtype_text,
        peer_id,
    })
}

fn candidate_from_required_and_legacy_metadata(
    row: &TelegramSourceRepairRow,
    required: TelegramRequiredIdentity,
) -> TelegramRepairCandidate {
    let metadata = decode_source_metadata(row.metadata_zstd.as_deref()).ok();
    let identity = metadata
        .as_ref()
        .and_then(|metadata| metadata.peer_identity.as_ref());
    let strategy = match identity.map(|identity| identity.strategy) {
        Some(SourcePeerResolutionStrategy::Username) => TelegramResolutionStrategy::Username,
        Some(SourcePeerResolutionStrategy::Dialog) => TelegramResolutionStrategy::Dialog,
        None => TelegramResolutionStrategy::Unknown,
    };

    TelegramRepairCandidate {
        source_id: required.source_id,
        account_id: required.account_id,
        peer_kind: required.peer_kind,
        source_subtype: required.source_subtype,
        source_subtype_text: required.source_subtype_text,
        peer_id: required.peer_id,
        resolution_strategy: strategy,
        username: normalize_telegram_username(
            identity.and_then(|identity| identity.username.as_deref()),
        ),
        access_hash: identity.and_then(|identity| identity.access_hash),
        avatar_cache_key: metadata.and_then(|metadata| metadata.avatar_cache_key),
    }
}
```

Add this typed-first helper. It must return without decoding legacy metadata when typed identity is valid.

```rust
fn candidate_from_row_and_existing_projection(
    row: &TelegramSourceRepairRow,
    existing: Option<&ExistingTelegramSourceProjection>,
) -> Result<TelegramRepairCandidate, SourceIdentityRepairDiagnostic> {
    let required = required_identity_from_row(row)?;
    let Some(existing) = existing else {
        return Ok(candidate_from_required_and_legacy_metadata(row, required));
    };

    let expected_peer_kind = required.peer_kind.as_str();
    if existing.account_id != required.account_id
        || existing.source_subtype != required.source_subtype_text
        || existing.peer_kind != expected_peer_kind
        || existing.peer_id != required.peer_id
    {
        return Ok(candidate_from_required_and_legacy_metadata(row, required));
    }

    let resolution_strategy =
        TelegramResolutionStrategy::parse(&existing.resolution_strategy).map_err(|_| {
            SourceIdentityRepairDiagnostic {
                code: "telegram_projection_drift_conflict".to_string(),
                source_ids: vec![row.id],
                detail: format!(
                    "Existing Telegram typed projection for source {} has invalid resolution_strategy",
                    row.id
                ),
            }
        })?;

    Ok(TelegramRepairCandidate {
        source_id: row.id,
        account_id: required.account_id,
        source_subtype: required.source_subtype,
        source_subtype_text: required.source_subtype_text,
        peer_kind: required.peer_kind,
        peer_id: required.peer_id,
        resolution_strategy,
        username: existing.username.clone(),
        access_hash: existing.access_hash,
        avatar_cache_key: existing.avatar_cache_key.clone(),
    })
}
```

Change candidate construction in `repair_source_identity` to:

```rust
let existing_projections_by_source_id: BTreeMap<i64, ExistingTelegramSourceProjection> =
    existing_projections
        .iter()
        .cloned()
        .map(|projection| (projection.source_id, projection))
        .collect();

for row in rows {
    match candidate_from_row_and_existing_projection(
        &row,
        existing_projections_by_source_id.get(&row.id),
    ) {
        Ok(candidate) => candidates.push(candidate),
        Err(diagnostic) => report.fatal_errors.push(diagnostic),
    }
}
```

Keep duplicate and projection-drift checks after candidates are built.

- [x] **Step 6: Adjust candidate order if needed**

If the compiler reports `existing_projections` is used before it is defined, move the existing projection query above candidate construction. The final order in `repair_source_identity` should be:

```rust
let rows: Vec<TelegramSourceRepairRow> = sqlx::query_as(/* sources query */)
    .fetch_all(&mut *tx)
    .await
    .map_err(AppError::database)?;

let existing_projections: Vec<ExistingTelegramSourceProjection> = sqlx::query_as(/* typed query */)
    .fetch_all(&mut *tx)
    .await
    .map_err(AppError::database)?;

let existing_projections_by_source_id: BTreeMap<i64, ExistingTelegramSourceProjection> =
    existing_projections
        .iter()
        .cloned()
        .map(|projection| (projection.source_id, projection))
        .collect();

let mut report = SourceIdentityRepairReport::default();
let mut candidates = Vec::new();
for row in rows {
    match candidate_from_row_and_existing_projection(
        &row,
        existing_projections_by_source_id.get(&row.id),
    ) {
        Ok(candidate) => candidates.push(candidate),
        Err(diagnostic) => report.fatal_errors.push(diagnostic),
    }
}
```

- [x] **Step 7: Run GREEN repair tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml repair_skips_malformed_metadata_when_typed_identity_is_valid
cargo test --manifest-path src-tauri/Cargo.toml repair_ignores_optional_enrichment_gaps_when_typed_identity_is_valid
cargo test --manifest-path src-tauri/Cargo.toml repair_creates_minimal_typed_identity_when_legacy_metadata_is_missing_or_malformed
cargo test --manifest-path src-tauri/Cargo.toml repair_fails_when_canonical_identity_is_invalid_even_with_legacy_peer_metadata
```

Expected: all four pass.

- [x] **Step 8: Run full repair module tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::identity_repair
```

Expected: all identity repair tests pass.

- [x] **Step 9: Commit**

Run:

```powershell
git status --short
git add src-tauri/src/sources/identity_repair.rs docs/superpowers/plans/2026-05-17-telegram-metadata-legacy-cleanup.md
git commit -m "feat: prefer typed telegram identity during repair"
```

## Task 3: Prove Runtime Paths Do Not Recreate Or Decode Telegram Blobs

**Files:**
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/sources/topics.rs`
- Test: same files

- [x] **Step 1: Add sync tests for null and legacy metadata preservation**

In `sync.rs` tests, add:

```rust
#[tokio::test]
async fn finalize_sync_preserves_existing_legacy_metadata_blob() {
    let pool = memory_pool_with_sources().await;
    let legacy_blob = crate::compression::compress_json_bytes(
        br#"{"peer_identity":{"strategy":"username","username":"legacy"}}"#,
    )
    .expect("compress legacy metadata");
    sqlx::query(
        r#"
        INSERT INTO sources (
            id, source_type, source_subtype, account_id, external_id, title,
            metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
        )
        VALUES (1, ?, ?, 1, '12345', 'Example', ?, 5, 10, 1, 1, 20)
        "#,
    )
    .bind(TELEGRAM_SOURCE_TYPE)
    .bind(TELEGRAM_KIND_CHANNEL)
    .bind(&legacy_blob)
    .execute(&pool)
    .await
    .expect("insert source");
    sqlx::query(
        r#"
        INSERT INTO telegram_sources (
            source_id, account_id, source_subtype, peer_kind, peer_id,
            resolution_strategy, username, access_hash, avatar_cache_key
        )
        VALUES (1, 1, 'channel', 'channel', 12345, 'username', 'before', 77, 'old.jpg')
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert typed identity");

    let source = load_source(&pool, 1).await.expect("load source");
    finalize_sync(&pool, &source, 5, 9, Some("new.jpg".to_string()))
        .await
        .expect("finalize sync");

    let row: (Option<Vec<u8>>, Option<String>) = sqlx::query_as(
        r#"
        SELECT s.metadata_zstd, ts.avatar_cache_key
        FROM sources s
        JOIN telegram_sources ts ON ts.source_id = s.id
        WHERE s.id = 1
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("load row");

    assert_eq!(row.0.as_deref(), Some(legacy_blob.as_slice()));
    assert_eq!(row.1.as_deref(), Some("new.jpg"));
}
```

The existing `finalize_sync_updates_source_state_and_typed_avatar_cache` already proves a `NULL` blob stays `NULL`; keep it.

- [x] **Step 2: Add Takeout typed identity test with corrupt blob**

In `takeout_import/mod.rs`, update or add a test:

```rust
#[tokio::test]
async fn takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists() {
    let pool = memory_pool_with_sources().await;
    sqlx::query(
        r#"
        INSERT INTO sources (
            id, source_type, source_subtype, account_id,
            external_id, title, metadata_zstd, last_sync_state, is_active, is_member,
            created_at
        )
        VALUES (?, 'telegram', 'supergroup', ?, ?, ?, x'00', NULL, 1, 1, ?)
        "#,
    )
    .bind(7_i64)
    .bind(42_i64)
    .bind("12345")
    .bind("Forum source")
    .bind(1_i64)
    .execute(&pool)
    .await
    .expect("insert source");
    sqlx::query(
        r#"
        INSERT INTO telegram_sources (
            source_id, account_id, source_subtype, peer_kind, peer_id,
            resolution_strategy, username, access_hash, avatar_cache_key,
            identity_refreshed_at, created_at, updated_at
        )
        VALUES (?, ?, 'supergroup', 'channel', ?, 'legacy_metadata', NULL, ?, NULL, ?, ?, ?)
        "#,
    )
    .bind(7_i64)
    .bind(42_i64)
    .bind(12345_i64)
    .bind(98765_i64)
    .bind(1_i64)
    .bind(1_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .expect("insert typed identity");

    let source_subtype = load_takeout_source_subtype(&pool, 7)
        .await
        .expect("load takeout source subtype");

    assert_eq!(source_subtype, TELEGRAM_KIND_SUPERGROUP);
}
```

- [x] **Step 3: Add forum topic typed identity test with corrupt blob**

In `topics.rs`, add a source with `metadata_zstd = x'00'` to the existing forum gate test or add:

```rust
#[tokio::test]
async fn forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists() {
    let pool = memory_pool_with_source_items_and_topics().await;
    sqlx::query(
        r#"
        INSERT INTO sources (
            id, source_type, source_subtype, account_id, external_id,
            title, metadata_zstd, last_sync_state, is_active, is_member,
            created_at
        )
        VALUES (11, 'telegram', 'supergroup', 42, '11', 'source 11', x'00', NULL, 1, 1, 1)
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert source");
    sqlx::query(
        r#"
        INSERT INTO telegram_sources (
            source_id, account_id, source_subtype, peer_kind, peer_id,
            resolution_strategy, username, access_hash, avatar_cache_key,
            identity_refreshed_at, created_at, updated_at
        )
        VALUES (11, 42, 'supergroup', 'channel', 11, 'legacy_metadata', NULL, 1011, NULL, 1, 1, 1)
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert typed identity");

    assert!(source_supports_forum_topics(&pool, 11)
        .await
        .expect("load typed identity"));
}
```

- [x] **Step 4: Run focused runtime tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml finalize_sync_preserves_existing_legacy_metadata_blob
cargo test --manifest-path src-tauri/Cargo.toml takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists
cargo test --manifest-path src-tauri/Cargo.toml forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists
```

Expected: all pass. These may already pass before implementation; if so, record that they are characterization tests rather than RED tests.

- [x] **Step 5: Run runtime module tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::sync
cargo test --manifest-path src-tauri/Cargo.toml takeout_import
cargo test --manifest-path src-tauri/Cargo.toml sources::topics
```

Expected: all selected module tests pass.

- [x] **Step 6: Commit**

Run:

```powershell
git status --short
git add src-tauri/src/sources/sync.rs src-tauri/src/takeout_import/mod.rs src-tauri/src/sources/topics.rs docs/superpowers/plans/2026-05-17-telegram-metadata-legacy-cleanup.md
git commit -m "test: cover telegram metadata runtime boundaries"
```

## Task 4: Documentation And Containment

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-17-telegram-metadata-legacy-cleanup.md`

- [ ] **Step 1: Update database schema docs**

In `docs/database-schema.md`, update the `sources` notes and `telegram_sources` notes so they say:

```markdown
- new Telegram source rows keep `metadata_zstd` `NULL`; old Telegram blobs may
  remain in existing databases as legacy repair input until a separate cleanup
  decision;
- normal Telegram sync, Takeout, forum topic refresh, source list display, and
  source resolution use typed identity and display cache fields in
  `telegram_sources`, not Telegram source metadata blobs;
- YouTube source rows still keep video/playlist metadata in `metadata_zstd`.
```

- [ ] **Step 2: Update backlog**

In `docs/backlog.md`, replace:

```markdown
- [ ] move remaining Telegram display/avatar metadata out of `sources.metadata_zstd`
```

with:

```markdown
- [ ] optionally clear old Telegram `sources.metadata_zstd` blobs after successful typed repair
```

Keep the YouTube typed metadata item unchanged.

- [ ] **Step 3: Run containment scans**

Run:

```powershell
rg -n "encode_source_metadata|source_metadata_for_added_source|decode_source_metadata|SourceMetadata" src-tauri\src\sources src-tauri\src\takeout_import src-tauri\src\notebooklm_export
rg -n "metadata_zstd = excluded.metadata_zstd|SET metadata_zstd|metadata_zstd," src-tauri\src\sources src-tauri\src\takeout_import
```

Expected:

- `encode_source_metadata` remains only in `peer_resolution.rs` tests or is removed if no tests need it.
- `decode_source_metadata` remains only in `identity_repair.rs` and `peer_resolution.rs` legacy decode tests.
- `source_metadata_for_added_source` is not used by `store.rs`; if no production/test use remains, remove the function and its tests.
- Telegram add/upsert no longer has `metadata_zstd = excluded.metadata_zstd`.
- YouTube source upserts still have `metadata_zstd = excluded.metadata_zstd`.

- [ ] **Step 4: Remove unused legacy encode helpers if scan proves they are dead**

If `source_metadata_for_added_source` is unused after Task 1, remove it from `peer_resolution.rs`.

If `SourceMetadata` and `encode_source_metadata` are still used by decode/roundtrip tests or repair, keep them. If `encode_source_metadata` is used only for a roundtrip test that can be replaced by `compress_json_bytes`, remove `encode_source_metadata` and update the test:

```rust
let encoded = compress_json_bytes(
    br#"{"peer_identity":{"strategy":"dialog","username":"example","access_hash":42},"avatar_cache_key":"1_channel_42.jpg"}"#,
)
.expect("encode");
let decoded = decode_source_metadata(Some(&encoded)).expect("decode");
```

- [ ] **Step 5: Run docs and containment checks**

Run:

```powershell
rg -n "move remaining Telegram display/avatar metadata out of `sources.metadata_zstd`" docs
rg -n "new Telegram source rows keep|old Telegram blobs|optionally clear old Telegram" docs\database-schema.md docs\backlog.md
git diff --check
```

Expected:

- old backlog wording is absent from current docs;
- new Telegram legacy-blob wording appears in current docs;
- `git diff --check` exits 0, ignoring CRLF warnings if there are no whitespace errors.

- [ ] **Step 6: Commit**

Run:

```powershell
git status --short
git add docs/database-schema.md docs/backlog.md docs/superpowers/plans/2026-05-17-telegram-metadata-legacy-cleanup.md
git commit -m "docs: document telegram metadata legacy boundary"
```

If Step 4 removed code, also add those Rust files and use:

```powershell
git add src-tauri/src/sources/peer_resolution.rs docs/database-schema.md docs/backlog.md docs/superpowers/plans/2026-05-17-telegram-metadata-legacy-cleanup.md
git commit -m "refactor: contain telegram metadata compatibility helpers"
```

## Task 5: Final Verification

**Files:**
- Verify all changed Rust/docs files
- Modify: `docs/superpowers/plans/2026-05-17-telegram-metadata-legacy-cleanup.md`

- [ ] **Step 1: Run full Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
test result: ok
```

- [ ] **Step 2: Run Rust formatting check**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

Expected: exit code 0.

- [ ] **Step 3: Run frontend checks only if frontend files changed**

If this plan touches only Rust and docs, skip this step and record that no frontend files changed.

If frontend files changed unexpectedly, run:

```powershell
npm.cmd test
npm.cmd run check
```

Expected:

```text
Test Files ... passed
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 4: Run final containment scans**

Run:

```powershell
rg -n "encode_source_metadata|source_metadata_for_added_source|decode_source_metadata|SourceMetadata" src-tauri\src\sources src-tauri\src\takeout_import src-tauri\src\notebooklm_export
rg -n "metadata_zstd = excluded.metadata_zstd|SET metadata_zstd|metadata_zstd," src-tauri\src\sources src-tauri\src\takeout_import
rg -n "telegram metadata|old Telegram blobs|sources.metadata_zstd" docs\database-schema.md docs\backlog.md docs\superpowers\specs\2026-05-17-telegram-metadata-legacy-cleanup-design.md
```

Expected:

- normal Telegram add/sync/list/Takeout paths do not call `decode_source_metadata` or `encode_source_metadata`;
- repair and legacy tests may still mention legacy metadata helpers;
- YouTube metadata writes remain intact;
- current docs describe new Telegram rows as `NULL` and old blobs as legacy input.

- [ ] **Step 5: Run whitespace check**

Run:

```powershell
git diff --check
```

Expected: exit code 0, with no whitespace errors. CRLF warnings are acceptable if exit code is 0 and there are no whitespace errors.

- [ ] **Step 6: Commit final plan bookkeeping if needed**

If only the plan checklist changed after verification, commit it:

```powershell
git status --short
git add docs/superpowers/plans/2026-05-17-telegram-metadata-legacy-cleanup.md
git commit -m "docs: mark telegram metadata cleanup verification complete"
```

If no files changed after verification, do not create a commit.

- [ ] **Step 7: Show final status**

Run:

```powershell
git status --short --branch
git --no-pager log -5 --oneline --decorate
```

Expected:

- working tree clean;
- latest commits include the Telegram metadata cleanup implementation and verification commits.
