#![allow(dead_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::Serialize;
use tauri::Manager;
use tokio::sync::RwLock;

use crate::error::{AppError, AppErrorKind, AppResult};

use super::identity::{
    canonical_telegram_external_id, normalize_telegram_username, TelegramPeerKind,
    TelegramResolutionStrategy,
};
use super::peer_resolution::{decode_source_metadata, SourcePeerResolutionStrategy};
use super::types::TelegramSourceKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceIdentityRepairMode {
    DryRun,
    Apply,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub(crate) struct SourceIdentityRepairReport {
    pub repaired_sources: Vec<i64>,
    pub repair_notes: Vec<SourceIdentityRepairNotePreview>,
    pub fatal_errors: Vec<SourceIdentityRepairDiagnostic>,
    pub canonical_index_created: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct SourceIdentityRepairNotePreview {
    pub source_id: i64,
    pub issue_code: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct SourceIdentityRepairDiagnostic {
    pub code: String,
    pub source_ids: Vec<i64>,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourceIdentityRepairStatus {
    Pending,
    Running,
    Ready,
    Failed { error: crate::error::AppError },
}

#[derive(Clone)]
pub(crate) struct SourceIdentityRepairState {
    status: Arc<RwLock<SourceIdentityRepairStatus>>,
}

impl SourceIdentityRepairState {
    pub(crate) fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(SourceIdentityRepairStatus::Pending)),
        }
    }

    pub(crate) async fn status(&self) -> SourceIdentityRepairStatus {
        self.status.read().await.clone()
    }

    async fn set_status(&self, status: SourceIdentityRepairStatus) {
        *self.status.write().await = status;
    }
}

pub(crate) async fn run_startup_source_identity_repair(handle: tauri::AppHandle) {
    let state = handle.state::<SourceIdentityRepairState>().inner().clone();
    state.set_status(SourceIdentityRepairStatus::Running).await;

    let result = async {
        let pool = crate::db::get_pool(&handle).await?;
        repair_source_identity(&pool, SourceIdentityRepairMode::Apply).await
    }
    .await;

    match result {
        Ok(_) => state.set_status(SourceIdentityRepairStatus::Ready).await,
        Err(error) => {
            state
                .set_status(SourceIdentityRepairStatus::Failed { error })
                .await
        }
    }
}

pub(crate) async fn require_source_identity_ready(
    state: &SourceIdentityRepairState,
) -> AppResult<()> {
    match state.status().await {
        SourceIdentityRepairStatus::Ready => Ok(()),
        SourceIdentityRepairStatus::Failed { error } => Err(error),
        SourceIdentityRepairStatus::Pending | SourceIdentityRepairStatus::Running => Err(
            AppError::conflict("Source identity repair is still running"),
        ),
    }
}

#[tauri::command]
pub(crate) async fn get_source_identity_repair_status(
    state: tauri::State<'_, SourceIdentityRepairState>,
) -> AppResult<SourceIdentityRepairStatus> {
    Ok(state.status().await)
}

#[tauri::command]
pub(crate) async fn preview_source_identity_repair(
    handle: tauri::AppHandle,
) -> AppResult<SourceIdentityRepairReport> {
    let pool = crate::db::get_pool(&handle).await?;
    repair_source_identity(&pool, SourceIdentityRepairMode::DryRun).await
}

#[derive(sqlx::FromRow)]
struct TelegramSourceRepairRow {
    id: i64,
    source_subtype: Option<String>,
    telegram_source_kind: Option<String>,
    account_id: Option<i64>,
    external_id: String,
    metadata_zstd: Option<Vec<u8>>,
}

#[derive(sqlx::FromRow)]
struct ExistingTelegramSourceProjection {
    source_id: i64,
    account_id: i64,
    peer_kind: String,
    peer_id: i64,
}

#[derive(Clone)]
struct TelegramRepairCandidate {
    source_id: i64,
    account_id: i64,
    source_subtype: TelegramSourceKind,
    source_subtype_text: String,
    peer_kind: TelegramPeerKind,
    peer_id: i64,
    resolution_strategy: TelegramResolutionStrategy,
    username: Option<String>,
    access_hash: Option<i64>,
    avatar_cache_key: Option<String>,
}

pub(crate) async fn repair_source_identity(
    pool: &sqlx::SqlitePool,
    mode: SourceIdentityRepairMode,
) -> AppResult<SourceIdentityRepairReport> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    let rows: Vec<TelegramSourceRepairRow> = sqlx::query_as(
        r#"
        SELECT id, source_subtype, telegram_source_kind, account_id, external_id, metadata_zstd
        FROM sources
        WHERE source_type = 'telegram'
        ORDER BY id
        "#,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(AppError::database)?;

    let mut report = SourceIdentityRepairReport::default();
    let mut candidates = Vec::new();

    for row in rows {
        match candidate_from_row(&row) {
            Ok(candidate) => candidates.push(candidate),
            Err(diagnostic) => report.fatal_errors.push(diagnostic),
        }
    }

    let existing_projections: Vec<ExistingTelegramSourceProjection> = sqlx::query_as(
        r#"
        SELECT source_id, account_id, peer_kind, peer_id
        FROM telegram_sources
        ORDER BY source_id
        "#,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(AppError::database)?;

    report
        .fatal_errors
        .extend(duplicate_canonical_identity_errors(&candidates));
    report
        .fatal_errors
        .extend(duplicate_typed_peer_identity_errors(&candidates));
    report.fatal_errors.extend(projection_drift_conflict_errors(
        &candidates,
        &existing_projections,
    ));

    if !report.fatal_errors.is_empty() {
        tx.rollback().await.map_err(AppError::database)?;
        if mode == SourceIdentityRepairMode::DryRun {
            return Ok(report);
        }
        return Err(repair_failed_error(&report));
    }

    for candidate in candidates {
        report.repaired_sources.push(candidate.source_id);
        if mode == SourceIdentityRepairMode::Apply {
            upsert_telegram_source_identity(&mut tx, &candidate).await?;
            sqlx::query(
                "UPDATE sources SET source_subtype = ?, telegram_source_kind = ? WHERE id = ?",
            )
            .bind(&candidate.source_subtype_text)
            .bind(&candidate.source_subtype_text)
            .bind(candidate.source_id)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
        }
    }

    if mode == SourceIdentityRepairMode::Apply {
        sqlx::query(
            r#"
            CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_telegram_identity
                ON sources(account_id, source_type, source_subtype, external_id)
                WHERE source_type = 'telegram'
            "#,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
        report.canonical_index_created = true;
        tx.commit().await.map_err(AppError::database)?;
    } else {
        tx.rollback().await.map_err(AppError::database)?;
    }

    Ok(report)
}

fn candidate_from_row(
    row: &TelegramSourceRepairRow,
) -> Result<TelegramRepairCandidate, SourceIdentityRepairDiagnostic> {
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
    let metadata = decode_source_metadata(row.metadata_zstd.as_deref()).map_err(|_| {
        SourceIdentityRepairDiagnostic {
            code: "malformed_telegram_metadata".to_string(),
            source_ids: vec![row.id],
            detail: format!("Telegram source {} has malformed legacy metadata", row.id),
        }
    })?;
    let identity = metadata.peer_identity.as_ref();
    let strategy = match identity.map(|identity| identity.strategy) {
        Some(SourcePeerResolutionStrategy::Username) => TelegramResolutionStrategy::Username,
        Some(SourcePeerResolutionStrategy::Dialog) => TelegramResolutionStrategy::Dialog,
        None => TelegramResolutionStrategy::Unknown,
    };

    Ok(TelegramRepairCandidate {
        source_id: row.id,
        account_id,
        peer_kind: TelegramPeerKind::from_source_subtype(source_subtype),
        source_subtype,
        source_subtype_text,
        peer_id,
        resolution_strategy: strategy,
        username: normalize_telegram_username(
            identity.and_then(|identity| identity.username.as_deref()),
        ),
        access_hash: identity.and_then(|identity| identity.access_hash),
        avatar_cache_key: metadata.avatar_cache_key,
    })
}

fn derive_source_subtype(
    row: &TelegramSourceRepairRow,
) -> Result<TelegramSourceKind, SourceIdentityRepairDiagnostic> {
    let canonical = row
        .source_subtype
        .as_deref()
        .and_then(|value| TelegramSourceKind::from_source_subtype(value).ok());
    let legacy = row
        .telegram_source_kind
        .as_deref()
        .and_then(|value| TelegramSourceKind::from_source_subtype(value).ok());

    match (canonical, legacy) {
        (Some(canonical), Some(legacy)) if canonical != legacy => {
            Err(SourceIdentityRepairDiagnostic {
                code: "telegram_subtype_legacy_kind_conflict".to_string(),
                source_ids: vec![row.id],
                detail: format!(
                    "Telegram source {} has conflicting source_subtype and legacy mirror",
                    row.id
                ),
            })
        }
        (Some(canonical), _) => Ok(canonical),
        (None, Some(legacy)) => Ok(legacy),
        (None, None) => Err(SourceIdentityRepairDiagnostic {
            code: "unsupported_telegram_source_subtype".to_string(),
            source_ids: vec![row.id],
            detail: format!("Telegram source {} has no supported subtype", row.id),
        }),
    }
}

fn duplicate_canonical_identity_errors(
    candidates: &[TelegramRepairCandidate],
) -> Vec<SourceIdentityRepairDiagnostic> {
    let mut groups: BTreeMap<(i64, String, String), Vec<i64>> = BTreeMap::new();
    for candidate in candidates {
        groups
            .entry((
                candidate.account_id,
                candidate.source_subtype_text.clone(),
                candidate.peer_id.to_string(),
            ))
            .or_default()
            .push(candidate.source_id);
    }

    groups
        .into_iter()
        .filter_map(|((account_id, subtype, external_id), source_ids)| {
            if source_ids.len() < 2 {
                return None;
            }
            Some(SourceIdentityRepairDiagnostic {
                code: "duplicate_canonical_telegram_identity".to_string(),
                detail: format!(
                    "Duplicate Telegram identity account_id={account_id}, source_subtype={subtype}, external_id={external_id}"
                ),
                source_ids,
            })
        })
        .collect()
}

fn duplicate_typed_peer_identity_errors(
    candidates: &[TelegramRepairCandidate],
) -> Vec<SourceIdentityRepairDiagnostic> {
    let mut groups: BTreeMap<(i64, String, i64), Vec<i64>> = BTreeMap::new();
    for candidate in candidates {
        groups
            .entry((
                candidate.account_id,
                candidate.peer_kind.as_str().to_string(),
                candidate.peer_id,
            ))
            .or_default()
            .push(candidate.source_id);
    }

    groups
        .into_iter()
        .filter_map(|((account_id, peer_kind, peer_id), source_ids)| {
            if source_ids.len() < 2 {
                return None;
            }
            Some(SourceIdentityRepairDiagnostic {
                code: "duplicate_typed_telegram_peer_identity".to_string(),
                detail: format!(
                    "Duplicate Telegram peer identity account_id={account_id}, peer_kind={peer_kind}, peer_id={peer_id}"
                ),
                source_ids,
            })
        })
        .collect()
}

fn projection_drift_conflict_errors(
    candidates: &[TelegramRepairCandidate],
    existing_projections: &[ExistingTelegramSourceProjection],
) -> Vec<SourceIdentityRepairDiagnostic> {
    let mut candidates_by_source_id = BTreeMap::new();
    let mut candidate_source_ids_by_peer = BTreeMap::new();
    for candidate in candidates {
        let peer_key = (
            candidate.account_id,
            candidate.peer_kind.as_str().to_string(),
            candidate.peer_id,
        );
        candidates_by_source_id.insert(candidate.source_id, peer_key.clone());
        candidate_source_ids_by_peer.insert(peer_key, candidate.source_id);
    }

    existing_projections
        .iter()
        .filter_map(|projection| {
            let current_peer_key = (
                projection.account_id,
                projection.peer_kind.clone(),
                projection.peer_id,
            );
            let expected_peer_key = candidates_by_source_id.get(&projection.source_id)?;
            if expected_peer_key == &current_peer_key {
                return None;
            }

            let conflicting_source_id = candidate_source_ids_by_peer.get(&current_peer_key)?;
            if *conflicting_source_id == projection.source_id {
                return None;
            }

            Some(SourceIdentityRepairDiagnostic {
                code: "telegram_projection_drift_conflict".to_string(),
                source_ids: vec![projection.source_id, *conflicting_source_id],
                detail: format!(
                    "Existing Telegram typed projection for source {} conflicts with source {}",
                    projection.source_id, conflicting_source_id
                ),
            })
        })
        .collect()
}

async fn upsert_telegram_source_identity(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    candidate: &TelegramRepairCandidate,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO telegram_sources (
            source_id, account_id, source_subtype, peer_kind, peer_id,
            resolution_strategy, username, access_hash, avatar_cache_key, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, strftime('%s','now'))
        ON CONFLICT(source_id) DO UPDATE SET
            account_id = excluded.account_id,
            source_subtype = excluded.source_subtype,
            peer_kind = excluded.peer_kind,
            peer_id = excluded.peer_id,
            resolution_strategy = excluded.resolution_strategy,
            username = excluded.username,
            access_hash = excluded.access_hash,
            avatar_cache_key = excluded.avatar_cache_key,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(candidate.source_id)
    .bind(candidate.account_id)
    .bind(&candidate.source_subtype_text)
    .bind(candidate.peer_kind.as_str())
    .bind(candidate.peer_id)
    .bind(candidate.resolution_strategy.as_str())
    .bind(candidate.username.as_deref())
    .bind(candidate.access_hash)
    .bind(candidate.avatar_cache_key.as_deref())
    .execute(&mut **tx)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn repair_failed_error(report: &SourceIdentityRepairReport) -> AppError {
    let details = report
        .fatal_errors
        .iter()
        .map(|diagnostic| {
            format!(
                "{}: sources {:?}: {}",
                diagnostic.code, diagnostic.source_ids, diagnostic.detail
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    AppError::new(
        AppErrorKind::Validation,
        format!("Source identity repair failed: {details}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_json_bytes;
    use crate::migrations::build_migrations;
    use crate::sources::store::{upsert_youtube_playlist_source, upsert_youtube_video_source};
    use crate::sources::test_support::memory_pool_with_sources;
    use crate::youtube::dto::{
        YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata,
    };
    use serde_json::json;

    async fn insert_telegram_source(
        pool: &sqlx::SqlitePool,
        id: i64,
        subtype: Option<&str>,
        legacy_kind: Option<&str>,
        account_id: Option<i64>,
        external_id: &str,
        metadata_json: Option<&[u8]>,
    ) {
        let metadata_zstd = metadata_json
            .map(compress_json_bytes)
            .transpose()
            .expect("compress metadata");
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, telegram_source_kind, account_id,
                external_id, title, metadata_zstd, is_active, is_member, created_at
            )
            VALUES (?, 'telegram', ?, ?, ?, ?, 'source', ?, 1, 1, 100)
            "#,
        )
        .bind(id)
        .bind(subtype)
        .bind(legacy_kind)
        .bind(account_id)
        .bind(external_id)
        .bind(metadata_zstd)
        .execute(pool)
        .await
        .expect("insert source");
    }

    async fn memory_pool_with_migrations_through(version: i64) -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        for migration in build_migrations()
            .into_iter()
            .filter(|migration| migration.version <= version)
        {
            sqlx::raw_sql(migration.sql)
                .execute(&pool)
                .await
                .unwrap_or_else(|error| panic!("apply migration {}: {error}", migration.version));
        }

        pool
    }

    async fn insert_existing_typed_projection(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        source_subtype: &str,
        peer_kind: &str,
        peer_id: i64,
    ) {
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy, username, access_hash, avatar_cache_key
            )
            VALUES (?, 1, ?, ?, ?, 'unknown', NULL, NULL, NULL)
            "#,
        )
        .bind(source_id)
        .bind(source_subtype)
        .bind(peer_kind)
        .bind(peer_id)
        .execute(pool)
        .await
        .expect("insert existing typed projection");
    }

    async fn insert_test_account(pool: &sqlx::SqlitePool, id: i64) {
        sqlx::query(
            r#"
            INSERT INTO accounts (id, label, api_id, api_hash, phone, created_at)
            VALUES (?, 'Test account', 12345, 'hash', NULL, 100)
            "#,
        )
        .bind(id)
        .execute(pool)
        .await
        .expect("insert account");
    }

    fn youtube_video_metadata() -> YoutubeVideoMetadata {
        YoutubeVideoMetadata {
            video_id: "dQw4w9WgXcQ".to_string(),
            canonical_url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            title: Some("Demo video".to_string()),
            channel_title: Some("Demo channel".to_string()),
            channel_id: Some("channel-1".to_string()),
            channel_handle: Some("@demo".to_string()),
            channel_url: Some("https://www.youtube.com/@demo".to_string()),
            author_display: Some("Demo channel".to_string()),
            published_at: Some("2009-10-25".to_string()),
            duration_seconds: Some(213),
            description: Some("Demo description".to_string()),
            thumbnail_url: None,
            tags: Vec::new(),
            chapters: Vec::new(),
            view_count: Some(1),
            like_count: Some(1),
            comment_count: Some(1),
            category: Some("Music".to_string()),
            video_form: YoutubeVideoForm::Regular,
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: json!({ "id": "dQw4w9WgXcQ" }),
        }
    }

    fn youtube_playlist_metadata() -> YoutubePlaylistMetadata {
        YoutubePlaylistMetadata {
            playlist_id: "PLdemo".to_string(),
            canonical_url: "https://www.youtube.com/playlist?list=PLdemo".to_string(),
            title: Some("Demo playlist".to_string()),
            channel_title: Some("Demo channel".to_string()),
            channel_id: Some("channel-1".to_string()),
            channel_handle: Some("@demo".to_string()),
            channel_url: Some("https://www.youtube.com/@demo".to_string()),
            thumbnail_url: None,
            video_count: Some(0),
            items: Vec::new(),
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: json!({ "id": "PLdemo" }),
        }
    }

    #[tokio::test]
    async fn dry_run_reports_repair_without_writing_typed_rows() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            Some(br#"{"peer_identity":{"strategy":"username","username":"Example","access_hash":77},"avatar_cache_key":"1_channel_12345.jpg"}"#),
        )
        .await;

        let report = repair_source_identity(&pool, SourceIdentityRepairMode::DryRun)
            .await
            .expect("dry run succeeds");

        assert_eq!(report.repaired_sources, vec![101]);
        assert!(report.fatal_errors.is_empty());

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_sources")
            .fetch_one(&pool)
            .await
            .expect("count typed rows");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn apply_repair_creates_typed_identity_and_keeps_source_id() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            Some(br#"{"peer_identity":{"strategy":"username","username":"Example","access_hash":77},"avatar_cache_key":"1_channel_12345.jpg"}"#),
        )
        .await;

        let report = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect("repair succeeds");

        assert_eq!(report.repaired_sources, vec![101]);

        let row: (i64, String, String, i64, String, Option<i64>, Option<String>) =
            sqlx::query_as(
                "SELECT source_id, source_subtype, peer_kind, peer_id, username, access_hash, avatar_cache_key FROM telegram_sources WHERE source_id = 101",
            )
            .fetch_one(&pool)
            .await
            .expect("typed row");

        assert_eq!(row.0, 101);
        assert_eq!(row.1, "channel");
        assert_eq!(row.2, "channel");
        assert_eq!(row.3, 12345);
        assert_eq!(row.4, "example");
        assert_eq!(row.5, Some(77));
        assert_eq!(row.6.as_deref(), Some("1_channel_12345.jpg"));
    }

    #[tokio::test]
    async fn v17_upgrade_repair_preserves_source_id_and_creates_canonical_index() {
        let pool = memory_pool_with_migrations_through(17).await;
        insert_test_account(&pool, 1).await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            Some(br#"{"peer_identity":{"strategy":"username","username":"Example","access_hash":77}}"#),
        )
        .await;
        let migration_18 = build_migrations()
            .into_iter()
            .find(|migration| migration.version == 18)
            .expect("migration 18");
        sqlx::raw_sql(migration_18.sql)
            .execute(&pool)
            .await
            .expect("apply migration 18");

        repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect("repair succeeds");

        let source_id_after_repair: i64 =
            sqlx::query_scalar("SELECT id FROM sources WHERE external_id = '12345'")
                .fetch_one(&pool)
                .await
                .expect("load source id");
        let typed_source_id: i64 =
            sqlx::query_scalar("SELECT source_id FROM telegram_sources WHERE source_id = 101")
                .fetch_one(&pool)
                .await
                .expect("load typed source id");
        let canonical_index_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_sources_unique_telegram_identity'",
        )
        .fetch_one(&pool)
        .await
        .expect("count canonical index");

        assert_eq!(source_id_after_repair, 101);
        assert_eq!(typed_source_id, 101);
        assert_eq!(canonical_index_count, 1);
    }

    #[tokio::test]
    async fn youtube_sources_are_unaffected_by_source_identity_repair() {
        let pool = memory_pool_with_migrations_through(18).await;

        let mut tx = pool.begin().await.expect("begin tx");
        let video_id = upsert_youtube_video_source(&mut tx, &youtube_video_metadata())
            .await
            .expect("upsert video before repair");
        let playlist_id = upsert_youtube_playlist_source(&mut tx, &youtube_playlist_metadata())
            .await
            .expect("upsert playlist before repair");
        tx.commit().await.expect("commit first upserts");

        repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect("repair succeeds");

        let mut tx = pool.begin().await.expect("begin tx");
        let video_id_after = upsert_youtube_video_source(&mut tx, &youtube_video_metadata())
            .await
            .expect("upsert video after repair");
        let playlist_id_after =
            upsert_youtube_playlist_source(&mut tx, &youtube_playlist_metadata())
                .await
                .expect("upsert playlist after repair");
        tx.commit().await.expect("commit second upserts");

        assert_eq!(video_id_after, video_id);
        assert_eq!(playlist_id_after, playlist_id);

        let typed_rows_for_youtube: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM telegram_sources WHERE source_id IN (?, ?)")
                .bind(video_id)
                .bind(playlist_id)
                .fetch_one(&pool)
                .await
                .expect("count typed rows for youtube sources");
        assert_eq!(typed_rows_for_youtube, 0);
    }

    #[tokio::test]
    async fn repair_updates_non_conflicting_typed_projection_drift() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            Some(br#"{"peer_identity":{"strategy":"username","username":"Example","access_hash":77}}"#),
        )
        .await;
        insert_existing_typed_projection(&pool, 101, "supergroup", "channel", 12345).await;

        repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect("repair updates projection drift");

        let row: (String, String, i64, Option<String>, Option<i64>) = sqlx::query_as(
            "SELECT source_subtype, peer_kind, peer_id, username, access_hash FROM telegram_sources WHERE source_id = 101",
        )
        .fetch_one(&pool)
        .await
        .expect("load repaired typed projection");

        assert_eq!(row.0, "channel");
        assert_eq!(row.1, "channel");
        assert_eq!(row.2, 12345);
        assert_eq!(row.3.as_deref(), Some("example"));
        assert_eq!(row.4, Some(77));
    }

    #[tokio::test]
    async fn repair_fails_on_conflicting_typed_projection_drift() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            None,
        )
        .await;
        insert_telegram_source(
            &pool,
            102,
            Some("supergroup"),
            Some("supergroup"),
            Some(1),
            "67890",
            None,
        )
        .await;
        insert_existing_typed_projection(&pool, 101, "channel", "channel", 67890).await;

        let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect_err("conflicting projection drift fails repair");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("telegram_projection_drift_conflict"));
        assert!(error.message.contains("101"));
        assert!(!error.message.contains("metadata_zstd"));
        assert!(!error.message.contains("peer_identity"));
    }

    #[tokio::test]
    async fn malformed_external_ids_fail_without_writing_typed_rows() {
        for external_id in ["+123", "-123", "00123", "123 ", "12a3"] {
            let pool = memory_pool_with_sources().await;
            insert_telegram_source(
                &pool,
                101,
                Some("channel"),
                Some("channel"),
                Some(1),
                external_id,
                None,
            )
            .await;

            let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
                .await
                .expect_err("malformed id fails repair");
            assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
            assert!(error.message.contains("malformed_telegram_external_id"));

            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_sources")
                .fetch_one(&pool)
                .await
                .expect("count typed rows");
            assert_eq!(count, 0);
        }
    }

    #[tokio::test]
    async fn missing_account_id_is_fatal() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            None,
            "12345",
            None,
        )
        .await;

        let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect_err("missing account fails repair");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("telegram_source_missing_account"));
    }

    #[tokio::test]
    async fn source_subtype_and_legacy_kind_conflict_is_fatal() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("supergroup"),
            Some(1),
            "12345",
            None,
        )
        .await;

        let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect_err("conflict fails repair");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error
            .message
            .contains("telegram_subtype_legacy_kind_conflict"));
    }

    #[tokio::test]
    async fn duplicate_canonical_identity_reports_conflicting_source_ids() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            None,
        )
        .await;
        insert_telegram_source(
            &pool,
            102,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            None,
        )
        .await;

        let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect_err("duplicate canonical identity fails repair");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error
            .message
            .contains("duplicate_canonical_telegram_identity"));
        assert!(error.message.contains("101"));
        assert!(error.message.contains("102"));
    }

    #[tokio::test]
    async fn duplicate_typed_peer_identity_reports_conflicting_source_ids() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            None,
        )
        .await;
        insert_telegram_source(
            &pool,
            102,
            Some("supergroup"),
            Some("supergroup"),
            Some(1),
            "12345",
            None,
        )
        .await;

        let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect_err("duplicate peer identity fails repair");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error
            .message
            .contains("duplicate_typed_telegram_peer_identity"));
        assert!(error.message.contains("101"));
        assert!(error.message.contains("102"));
    }

    #[tokio::test]
    async fn apply_repair_is_idempotent() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("group"),
            Some("group"),
            Some(1),
            "12345",
            None,
        )
        .await;

        repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect("first repair");
        repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect("second repair");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_sources")
            .fetch_one(&pool)
            .await
            .expect("count typed rows");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn fatal_repair_rolls_back_and_does_not_create_canonical_index() {
        let pool = memory_pool_with_sources().await;
        insert_telegram_source(
            &pool,
            101,
            Some("channel"),
            Some("channel"),
            Some(1),
            "12345",
            None,
        )
        .await;
        insert_telegram_source(
            &pool,
            102,
            Some("channel"),
            Some("channel"),
            None,
            "67890",
            None,
        )
        .await;

        repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
            .await
            .expect_err("repair fails");

        let typed_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_sources")
            .fetch_one(&pool)
            .await
            .expect("count typed rows");
        assert_eq!(typed_count, 0);

        let index_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_sources_unique_telegram_identity'",
        )
        .fetch_one(&pool)
        .await
        .expect("count canonical index");
        assert_eq!(index_count, 0);
    }

    #[tokio::test]
    async fn source_identity_gate_blocks_while_running() {
        let state = SourceIdentityRepairState::new();
        state.set_status(SourceIdentityRepairStatus::Running).await;

        let error = require_source_identity_ready(&state)
            .await
            .expect_err("running gate blocks");
        assert_eq!(error.kind, crate::error::AppErrorKind::Conflict);
    }

    #[tokio::test]
    async fn source_identity_gate_returns_startup_failure() {
        let state = SourceIdentityRepairState::new();
        state
            .set_status(SourceIdentityRepairStatus::Failed {
                error: AppError::validation("Source identity repair failed: example"),
            })
            .await;

        let error = require_source_identity_ready(&state)
            .await
            .expect_err("failed gate blocks");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("Source identity repair failed"));
    }
}
