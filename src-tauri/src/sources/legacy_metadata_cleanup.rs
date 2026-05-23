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
