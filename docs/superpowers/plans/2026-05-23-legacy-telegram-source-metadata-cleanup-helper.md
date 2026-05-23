# Legacy Telegram Source Metadata Cleanup Helper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an explicit guarded audit/clear helper for eligible legacy Telegram `sources.metadata_zstd` blobs without automatic migration or opportunistic runtime cleanup.

**Architecture:** Add a focused `sources::legacy_metadata_cleanup` module that owns eligibility evaluation, sanitized reporting, and optional clearing. Expose two Tauri commands: an audit command that never mutates and a clear command that reuses the same guard logic inside a transaction before setting eligible blobs to `NULL`.

**Tech Stack:** Rust, Tauri commands, `sqlx` SQLite, existing `SourceIdentityRepairState`, existing source identity enum parsers, Tokio tests.

---

## File Structure

- Create `src-tauri/src/sources/legacy_metadata_cleanup.rs`
  - Owns report DTOs, skip reason codes, eligibility query, audit mode, clear mode, and unit tests.
- Modify `src-tauri/src/sources/mod.rs`
  - Exports Tauri command functions from the new module.
- Modify `src-tauri/src/lib.rs`
  - Registers `audit_legacy_telegram_source_metadata` and `clear_legacy_telegram_source_metadata`.
- Modify `docs/database-schema.md`
  - Adds the implemented command names to the cleanup policy note.
- Modify `docs/backlog.md`
  - Marks the guarded helper follow-up complete when implementation is verified.

## Task 1: Add Audit DTOs And Eligibility Tests

**Files:**
- Create: `src-tauri/src/sources/legacy_metadata_cleanup.rs`

- [x] **Step 1: Write the failing tests**

Add the new file with tests first. The first red run should fail before the
eligibility query and guard logic are added.

```rust
use std::collections::BTreeMap;

use serde::Serialize;

use crate::error::{AppError, AppResult};

use super::identity::{TelegramPeerKind, TelegramResolutionStrategy};
use super::types::{TelegramSourceKind, TELEGRAM_SOURCE_TYPE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LegacyTelegramMetadataCleanupMode {
    Audit,
    Clear,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct LegacyTelegramSourceMetadataCleanupReport {
    pub(crate) dry_run: bool,
    pub(crate) candidate_count: i64,
    pub(crate) eligible_count: i64,
    pub(crate) cleared_count: i64,
    pub(crate) candidate_source_ids: Vec<i64>,
    pub(crate) eligible_source_ids: Vec<i64>,
    pub(crate) cleared_source_ids: Vec<i64>,
    pub(crate) subtype_counts: Vec<LegacyTelegramSourceMetadataSubtypeCount>,
    pub(crate) skipped: Vec<LegacyTelegramSourceMetadataSkip>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct LegacyTelegramSourceMetadataSubtypeCount {
    pub(crate) source_subtype: String,
    pub(crate) candidate_count: i64,
    pub(crate) eligible_count: i64,
    pub(crate) cleared_count: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct LegacyTelegramSourceMetadataSkip {
    pub(crate) source_id: i64,
    pub(crate) reason_code: String,
}

const SKIP_MISSING_TYPED_IDENTITY: &str = "missing_typed_identity";
const SKIP_SOURCE_SUBTYPE_MISMATCH: &str = "source_subtype_mismatch";
const SKIP_ACCOUNT_MISMATCH: &str = "account_mismatch";
const SKIP_INVALID_TYPED_IDENTITY: &str = "invalid_typed_identity";
const SKIP_UNSUPPORTED_SOURCE_SUBTYPE: &str = "unsupported_source_subtype";
const SKIP_MISSING_ACCOUNT: &str = "missing_account";

pub(crate) async fn run_legacy_telegram_source_metadata_cleanup(
    pool: &sqlx::SqlitePool,
    mode: LegacyTelegramMetadataCleanupMode,
) -> AppResult<LegacyTelegramSourceMetadataCleanupReport> {
    let _ = (pool, mode);
    Ok(LegacyTelegramSourceMetadataCleanupReport {
        dry_run: true,
        candidate_count: 0,
        eligible_count: 0,
        cleared_count: 0,
        candidate_source_ids: Vec::new(),
        eligible_source_ids: Vec::new(),
        cleared_source_ids: Vec::new(),
        subtype_counts: Vec::new(),
        skipped: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_json_bytes;
    use crate::sources::test_support::memory_pool_with_sources;

    async fn insert_account(pool: &sqlx::SqlitePool, account_id: i64) {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY,
                label TEXT NOT NULL,
                api_id INTEGER NOT NULL,
                api_hash TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("create accounts");
        sqlx::query("INSERT OR IGNORE INTO accounts (id, label, api_id, api_hash) VALUES (?, 'a', 1, '')")
            .bind(account_id)
            .execute(pool)
            .await
            .expect("insert account");
    }

    async fn insert_telegram_source(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        source_subtype: &str,
        account_id: Option<i64>,
        external_id: &str,
        has_legacy_blob: bool,
    ) {
        let metadata_zstd = if has_legacy_blob {
            Some(compress_json_bytes(br#"{"legacy":true}"#).expect("compress legacy blob"))
        } else {
            None
        };
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id,
                title, metadata_zstd, is_active, is_member, created_at
            )
            VALUES (?, 'telegram', ?, ?, ?, 'source', ?, 1, 1, 100)
            "#,
        )
        .bind(source_id)
        .bind(source_subtype)
        .bind(account_id)
        .bind(external_id)
        .bind(metadata_zstd)
        .execute(pool)
        .await
        .expect("insert telegram source");
    }

    async fn insert_typed_identity(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        account_id: i64,
        source_subtype: &str,
        peer_kind: &str,
        peer_id: i64,
        resolution_strategy: &str,
    ) {
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(source_id)
        .bind(account_id)
        .bind(source_subtype)
        .bind(peer_kind)
        .bind(peer_id)
        .bind(resolution_strategy)
        .execute(pool)
        .await
        .expect("insert typed identity");
    }

    #[tokio::test]
    async fn audit_reports_eligible_legacy_telegram_metadata_without_mutating() {
        let pool = memory_pool_with_sources().await;
        insert_account(&pool, 1).await;
        insert_telegram_source(&pool, 101, "channel", Some(1), "12345", true).await;
        insert_typed_identity(&pool, 101, 1, "channel", "channel", 12345, "dialog").await;

        let report = run_legacy_telegram_source_metadata_cleanup(
            &pool,
            LegacyTelegramMetadataCleanupMode::Audit,
        )
        .await
        .expect("audit succeeds");

        assert!(report.dry_run);
        assert_eq!(report.candidate_source_ids, vec![101]);
        assert_eq!(report.eligible_source_ids, vec![101]);
        assert!(report.cleared_source_ids.is_empty());
        assert_eq!(report.candidate_count, 1);
        assert_eq!(report.eligible_count, 1);
        assert_eq!(report.cleared_count, 0);
        assert!(report.skipped.is_empty());
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM sources WHERE id = 101 AND metadata_zstd IS NOT NULL",
            )
            .fetch_one(&pool)
            .await
            .expect("count legacy blob"),
            1
        );
    }

    #[tokio::test]
    async fn audit_skips_missing_typed_identity() {
        let pool = memory_pool_with_sources().await;
        insert_account(&pool, 1).await;
        insert_telegram_source(&pool, 101, "channel", Some(1), "12345", true).await;

        let report = run_legacy_telegram_source_metadata_cleanup(
            &pool,
            LegacyTelegramMetadataCleanupMode::Audit,
        )
        .await
        .expect("audit succeeds");

        assert_eq!(report.candidate_source_ids, vec![101]);
        assert!(report.eligible_source_ids.is_empty());
        assert_eq!(
            report.skipped,
            vec![LegacyTelegramSourceMetadataSkip {
                source_id: 101,
                reason_code: SKIP_MISSING_TYPED_IDENTITY.to_string(),
            }]
        );
    }
}
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::legacy_metadata_cleanup::tests::audit_ -- --nocapture
```

Expected: compile failure or assertion failure because the module is not wired
into `sources::mod.rs` and the stub returns an empty report.

- [x] **Step 3: Register the module privately**

Modify `src-tauri/src/sources/mod.rs`:

```rust
mod avatar;
pub(crate) mod identity;
pub(crate) mod identity_repair;
mod items;
mod legacy_metadata_cleanup;
mod peer_resolution;
mod settings;
mod store;
mod sync;
#[cfg(test)]
pub(crate) mod test_support;
mod topics;
mod types;
```

- [x] **Step 4: Run tests again and verify implementation failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::legacy_metadata_cleanup::tests::audit_ -- --nocapture
```

Expected: tests compile and fail because the stub returns an empty report.

## Task 2: Implement Audit Eligibility

**Files:**
- Modify: `src-tauri/src/sources/legacy_metadata_cleanup.rs`

- [x] **Step 1: Replace the stub with row loading and guard evaluation**

Replace `run_legacy_telegram_source_metadata_cleanup` and add the helper structs/functions below the constants:

```rust
#[derive(sqlx::FromRow)]
struct LegacyTelegramMetadataCandidateRow {
    source_id: i64,
    source_subtype: Option<String>,
    account_id: Option<i64>,
    metadata_present: i64,
    typed_source_id: Option<i64>,
    typed_account_id: Option<i64>,
    typed_source_subtype: Option<String>,
    typed_peer_kind: Option<String>,
    typed_peer_id: Option<i64>,
    typed_resolution_strategy: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EvaluatedCandidate {
    source_id: i64,
    source_subtype: String,
    eligible: bool,
    skip_reason: Option<&'static str>,
}

pub(crate) async fn run_legacy_telegram_source_metadata_cleanup(
    pool: &sqlx::SqlitePool,
    mode: LegacyTelegramMetadataCleanupMode,
) -> AppResult<LegacyTelegramSourceMetadataCleanupReport> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    let rows: Vec<LegacyTelegramMetadataCandidateRow> = sqlx::query_as(
        r#"
        SELECT
            s.id AS source_id,
            s.source_subtype,
            s.account_id,
            CASE WHEN s.metadata_zstd IS NOT NULL THEN 1 ELSE 0 END AS metadata_present,
            ts.source_id AS typed_source_id,
            ts.account_id AS typed_account_id,
            ts.source_subtype AS typed_source_subtype,
            ts.peer_kind AS typed_peer_kind,
            ts.peer_id AS typed_peer_id,
            ts.resolution_strategy AS typed_resolution_strategy
        FROM sources s
        LEFT JOIN telegram_sources ts ON ts.source_id = s.id
        WHERE s.source_type = ?
          AND s.metadata_zstd IS NOT NULL
        ORDER BY s.id
        "#,
    )
    .bind(TELEGRAM_SOURCE_TYPE)
    .fetch_all(&mut *tx)
    .await
    .map_err(AppError::database)?;

    let evaluated: Vec<EvaluatedCandidate> = rows.iter().map(evaluate_candidate).collect();
    let eligible_source_ids: Vec<i64> = evaluated
        .iter()
        .filter(|candidate| candidate.eligible)
        .map(|candidate| candidate.source_id)
        .collect();

    let mut cleared_source_ids = Vec::new();
    if matches!(mode, LegacyTelegramMetadataCleanupMode::Clear) {
        for source_id in &eligible_source_ids {
            sqlx::query(
                r#"
                UPDATE sources
                SET metadata_zstd = NULL
                WHERE id = ?
                  AND source_type = ?
                  AND metadata_zstd IS NOT NULL
                "#,
            )
            .bind(source_id)
            .bind(TELEGRAM_SOURCE_TYPE)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
            cleared_source_ids.push(*source_id);
        }
        tx.commit().await.map_err(AppError::database)?;
    } else {
        tx.rollback().await.map_err(AppError::database)?;
    }

    Ok(build_report(
        matches!(mode, LegacyTelegramMetadataCleanupMode::Audit),
        &rows,
        &evaluated,
        cleared_source_ids,
    ))
}

fn evaluate_candidate(row: &LegacyTelegramMetadataCandidateRow) -> EvaluatedCandidate {
    let source_subtype = row
        .source_subtype
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let skip_reason = candidate_skip_reason(row);
    EvaluatedCandidate {
        source_id: row.source_id,
        source_subtype,
        eligible: skip_reason.is_none(),
        skip_reason,
    }
}

fn candidate_skip_reason(row: &LegacyTelegramMetadataCandidateRow) -> Option<&'static str> {
    let Some(source_subtype) = row.source_subtype.as_deref() else {
        return Some(SKIP_UNSUPPORTED_SOURCE_SUBTYPE);
    };
    let Ok(source_kind) = TelegramSourceKind::parse(source_subtype) else {
        return Some(SKIP_UNSUPPORTED_SOURCE_SUBTYPE);
    };
    let Some(source_account_id) = row.account_id else {
        return Some(SKIP_MISSING_ACCOUNT);
    };
    if row.typed_source_id.is_none() {
        return Some(SKIP_MISSING_TYPED_IDENTITY);
    }
    if row.typed_account_id != Some(source_account_id) {
        return Some(SKIP_ACCOUNT_MISMATCH);
    }
    if row.typed_source_subtype.as_deref() != Some(source_subtype) {
        return Some(SKIP_SOURCE_SUBTYPE_MISMATCH);
    }

    let Some(peer_kind) = row.typed_peer_kind.as_deref() else {
        return Some(SKIP_INVALID_TYPED_IDENTITY);
    };
    let Ok(parsed_peer_kind) = TelegramPeerKind::parse(peer_kind) else {
        return Some(SKIP_INVALID_TYPED_IDENTITY);
    };
    if parsed_peer_kind != TelegramPeerKind::from_source_subtype(source_kind) {
        return Some(SKIP_INVALID_TYPED_IDENTITY);
    }
    match row.typed_peer_id {
        Some(peer_id) if peer_id > 0 => {}
        _ => return Some(SKIP_INVALID_TYPED_IDENTITY),
    }
    let Some(strategy) = row.typed_resolution_strategy.as_deref() else {
        return Some(SKIP_INVALID_TYPED_IDENTITY);
    };
    if TelegramResolutionStrategy::parse(strategy).is_err() {
        return Some(SKIP_INVALID_TYPED_IDENTITY);
    }

    None
}

fn build_report(
    dry_run: bool,
    rows: &[LegacyTelegramMetadataCandidateRow],
    evaluated: &[EvaluatedCandidate],
    cleared_source_ids: Vec<i64>,
) -> LegacyTelegramSourceMetadataCleanupReport {
    let candidate_source_ids = rows.iter().map(|row| row.source_id).collect::<Vec<_>>();
    let eligible_source_ids = evaluated
        .iter()
        .filter(|candidate| candidate.eligible)
        .map(|candidate| candidate.source_id)
        .collect::<Vec<_>>();
    let skipped = evaluated
        .iter()
        .filter_map(|candidate| {
            candidate
                .skip_reason
                .map(|reason| LegacyTelegramSourceMetadataSkip {
                    source_id: candidate.source_id,
                    reason_code: reason.to_string(),
                })
        })
        .collect::<Vec<_>>();

    let subtype_counts = subtype_counts(evaluated, &cleared_source_ids);

    LegacyTelegramSourceMetadataCleanupReport {
        dry_run,
        candidate_count: candidate_source_ids.len() as i64,
        eligible_count: eligible_source_ids.len() as i64,
        cleared_count: cleared_source_ids.len() as i64,
        candidate_source_ids,
        eligible_source_ids,
        cleared_source_ids,
        subtype_counts,
        skipped,
    }
}

fn subtype_counts(
    evaluated: &[EvaluatedCandidate],
    cleared_source_ids: &[i64],
) -> Vec<LegacyTelegramSourceMetadataSubtypeCount> {
    let mut counts: BTreeMap<String, (i64, i64, i64)> = BTreeMap::new();
    for candidate in evaluated {
        let entry = counts
            .entry(candidate.source_subtype.clone())
            .or_insert((0, 0, 0));
        entry.0 += 1;
        if candidate.eligible {
            entry.1 += 1;
        }
        if cleared_source_ids.contains(&candidate.source_id) {
            entry.2 += 1;
        }
    }
    counts
        .into_iter()
        .map(
            |(source_subtype, (candidate_count, eligible_count, cleared_count))| {
                LegacyTelegramSourceMetadataSubtypeCount {
                    source_subtype,
                    candidate_count,
                    eligible_count,
                    cleared_count,
                }
            },
        )
        .collect()
}
```

- [x] **Step 2: Run audit tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::legacy_metadata_cleanup::tests::audit_ -- --nocapture
```

Expected: both audit tests pass.

- [x] **Step 3: Commit**

```powershell
git add src-tauri/src/sources/legacy_metadata_cleanup.rs src-tauri/src/sources/mod.rs
git commit -m "test: add legacy telegram metadata cleanup audit"
```

## Task 3: Add Clear Mode Tests And Implementation Hardening

**Files:**
- Modify: `src-tauri/src/sources/legacy_metadata_cleanup.rs`

- [x] **Step 1: Add clear-mode and skip-reason tests**

Add these tests to the existing `tests` module:

```rust
#[tokio::test]
async fn clear_nulls_only_eligible_legacy_telegram_metadata() {
    let pool = memory_pool_with_sources().await;
    insert_account(&pool, 1).await;
    insert_telegram_source(&pool, 101, "channel", Some(1), "12345", true).await;
    insert_typed_identity(&pool, 101, 1, "channel", "channel", 12345, "dialog").await;
    insert_telegram_source(&pool, 102, "channel", Some(1), "67890", true).await;

    let report = run_legacy_telegram_source_metadata_cleanup(
        &pool,
        LegacyTelegramMetadataCleanupMode::Clear,
    )
    .await
    .expect("clear succeeds");

    assert!(!report.dry_run);
    assert_eq!(report.candidate_source_ids, vec![101, 102]);
    assert_eq!(report.eligible_source_ids, vec![101]);
    assert_eq!(report.cleared_source_ids, vec![101]);
    assert_eq!(report.cleared_count, 1);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sources WHERE id = 101 AND metadata_zstd IS NULL",
        )
        .fetch_one(&pool)
        .await
        .expect("count cleared blob"),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sources WHERE id = 102 AND metadata_zstd IS NOT NULL",
        )
        .fetch_one(&pool)
        .await
        .expect("count skipped blob"),
        1
    );
}

#[tokio::test]
async fn audit_skips_subtype_and_account_mismatches() {
    let pool = memory_pool_with_sources().await;
    insert_account(&pool, 1).await;
    insert_account(&pool, 2).await;
    insert_telegram_source(&pool, 101, "channel", Some(1), "12345", true).await;
    insert_typed_identity(&pool, 101, 1, "supergroup", "channel", 12345, "dialog").await;
    insert_telegram_source(&pool, 102, "channel", Some(1), "67890", true).await;
    insert_typed_identity(&pool, 102, 2, "channel", "channel", 67890, "dialog").await;

    let report = run_legacy_telegram_source_metadata_cleanup(
        &pool,
        LegacyTelegramMetadataCleanupMode::Audit,
    )
    .await
    .expect("audit succeeds");

    assert_eq!(
        report.skipped,
        vec![
            LegacyTelegramSourceMetadataSkip {
                source_id: 101,
                reason_code: SKIP_SOURCE_SUBTYPE_MISMATCH.to_string(),
            },
            LegacyTelegramSourceMetadataSkip {
                source_id: 102,
                reason_code: SKIP_ACCOUNT_MISMATCH.to_string(),
            },
        ]
    );
}

#[tokio::test]
async fn audit_ignores_non_telegram_and_null_metadata_rows() {
    let pool = memory_pool_with_sources().await;
    insert_account(&pool, 1).await;
    insert_telegram_source(&pool, 101, "channel", Some(1), "12345", false).await;
    insert_typed_identity(&pool, 101, 1, "channel", "channel", 12345, "dialog").await;
    sqlx::query(
        r#"
        INSERT INTO sources (
            id, source_type, source_subtype, account_id, external_id,
            title, metadata_zstd, is_active, is_member, created_at
        )
        VALUES (201, 'youtube', 'video', NULL, 'video-id', 'video', x'00', 1, 1, 100)
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert non-telegram source");

    let report = run_legacy_telegram_source_metadata_cleanup(
        &pool,
        LegacyTelegramMetadataCleanupMode::Audit,
    )
    .await
    .expect("audit succeeds");

    assert_eq!(report.candidate_count, 0);
    assert!(report.candidate_source_ids.is_empty());
    assert!(report.eligible_source_ids.is_empty());
    assert!(report.skipped.is_empty());
}
```

- [x] **Step 2: Run focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::legacy_metadata_cleanup::tests:: -- --nocapture
```

Expected: all legacy cleanup module tests pass.

- [x] **Step 3: Commit**

```powershell
git add src-tauri/src/sources/legacy_metadata_cleanup.rs
git commit -m "feat: add guarded legacy telegram metadata cleanup"
```

## Task 4: Add Repair-State-Gated Tauri Commands

**Files:**
- Modify: `src-tauri/src/sources/legacy_metadata_cleanup.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add command wrappers**

Append this code above the tests in `src-tauri/src/sources/legacy_metadata_cleanup.rs`:

```rust
#[tauri::command]
pub(crate) async fn audit_legacy_telegram_source_metadata(
    handle: tauri::AppHandle,
    repair_state: tauri::State<'_, super::identity_repair::SourceIdentityRepairState>,
) -> AppResult<LegacyTelegramSourceMetadataCleanupReport> {
    super::identity_repair::require_source_identity_ready(repair_state.inner()).await?;
    let pool = crate::db::get_pool(&handle).await?;
    run_legacy_telegram_source_metadata_cleanup(
        &pool,
        LegacyTelegramMetadataCleanupMode::Audit,
    )
    .await
}

#[tauri::command]
pub(crate) async fn clear_legacy_telegram_source_metadata(
    handle: tauri::AppHandle,
    repair_state: tauri::State<'_, super::identity_repair::SourceIdentityRepairState>,
) -> AppResult<LegacyTelegramSourceMetadataCleanupReport> {
    super::identity_repair::require_source_identity_ready(repair_state.inner()).await?;
    let pool = crate::db::get_pool(&handle).await?;
    run_legacy_telegram_source_metadata_cleanup(
        &pool,
        LegacyTelegramMetadataCleanupMode::Clear,
    )
    .await
}
```

- [ ] **Step 2: Export commands from sources module**

Add to `src-tauri/src/sources/mod.rs` near other public command exports:

```rust
pub use legacy_metadata_cleanup::{
    audit_legacy_telegram_source_metadata, clear_legacy_telegram_source_metadata,
};
```

- [ ] **Step 3: Register commands in Tauri**

Modify `src-tauri/src/lib.rs` imports:

```rust
use sources::{
    add_telegram_source, audit_legacy_telegram_source_metadata,
    clear_legacy_telegram_source_metadata, delete_source, get_sync_settings,
    list_source_forum_topics, list_source_items, list_sources, list_telegram_sources,
    save_sync_settings, sync_source,
};
```

Add both commands to `tauri::generate_handler!` after `preview_source_identity_repair`:

```rust
            preview_source_identity_repair,
            audit_legacy_telegram_source_metadata,
            clear_legacy_telegram_source_metadata,
            list_telegram_sources,
```

- [ ] **Step 4: Run compile-focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::legacy_metadata_cleanup::tests:: -- --nocapture
```

Expected: tests pass and command wrappers compile.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/sources/legacy_metadata_cleanup.rs src-tauri/src/sources/mod.rs src-tauri/src/lib.rs
git commit -m "feat: expose legacy telegram metadata cleanup commands"
```

## Task 5: Update Docs And Backlog

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/backlog.md`

- [ ] **Step 1: Update schema docs with command names**

In `docs/database-schema.md`, extend the cleanup policy bullet to name the commands:

```markdown
- legacy Telegram `sources.metadata_zstd` is no longer the runtime source of
  truth; cleanup is allowed only through the explicit
  `audit_legacy_telegram_source_metadata` and
  `clear_legacy_telegram_source_metadata` guarded operations after typed
  identity validation, not through startup, ordinary schema migration, or
  opportunistic sync/update/list/Takeout paths;
```

- [ ] **Step 2: Mark helper backlog item complete**

In `docs/backlog.md`, update the 3.2 helper item:

```markdown
- [x] implement an explicit guarded audit/dry-run/clear helper for eligible
  legacy Telegram source metadata blobs
```

- [ ] **Step 3: Run docs whitespace check**

Run:

```powershell
git diff --check
```

Expected: exit code 0. CRLF warnings are acceptable if there are no whitespace errors.

- [ ] **Step 4: Commit**

```powershell
git add docs/database-schema.md docs/backlog.md
git commit -m "docs: document legacy telegram metadata cleanup helper"
```

## Task 6: Full Verification

**Files:**
- No edits unless verification finds a defect.

- [ ] **Step 1: Run focused module tests**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::legacy_metadata_cleanup::tests:: -- --nocapture
```

Expected: all cleanup tests pass.

- [ ] **Step 2: Run source identity repair tests**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::identity_repair::tests:: -- --nocapture
```

Expected: existing source identity repair tests pass.

- [ ] **Step 3: Run full Rust tests**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all Rust tests pass.

- [ ] **Step 4: Check working tree and whitespace**

```powershell
git diff --check
git status --short --branch
```

Expected: `git diff --check` exits 0 and status is clean after all commits.

- [ ] **Step 5: Record final verification commit if docs changed**

If verification requires documentation changes, commit them:

```powershell
git add docs/backlog.md docs/database-schema.md
git commit -m "docs: record legacy metadata cleanup verification"
```

If no files changed, do not create an empty commit.
