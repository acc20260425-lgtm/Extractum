use tauri::AppHandle;
use tokio::time::{Duration, Instant};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::telegram::TelegramState;
use crate::youtube::dto::{YoutubePlaylistMetadata, YoutubeVideoMetadata};
use crate::youtube::source_metadata::{
    upsert_playlist_source_metadata, upsert_video_source_metadata, YoutubePlaylistSourceColumns,
    YoutubeVideoSourceColumns,
};

use super::avatar::{
    cache_source_avatar, peer_photo_data_url_with_timeout, read_source_avatar_data_url,
    TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS,
};
use super::identity::{
    canonical_telegram_external_id, normalize_telegram_username, TelegramPeerKind,
    TelegramResolutionStrategy,
};
use super::identity_repair::{require_source_identity_ready, SourceIdentityRepairState};
use super::peer_resolution::{
    add_source_resolution_strategy, resolve_telegram_source, telegram_source_info_from_peer,
    ResolvedTelegramSource, SourcePeerResolutionStrategy,
};
use super::types::{
    now_secs, SourceRecord, SourceRecordRow, SourceSyncTarget, SourceType, TelegramSourceInfo,
    TelegramSourceKind, TELEGRAM_SOURCE_TYPE,
};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTelegramSourceRequest {
    pub account_id: i64,
    pub source_ref: String,
    pub expected_subtype: Option<TelegramSourceKind>,
}

#[tauri::command]
pub async fn delete_source(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    ingest_locks: tauri::State<'_, SourceIngestLocks>,
    source_id: i64,
) -> AppResult<()> {
    require_source_identity_ready(repair_state.inner()).await?;
    let _ingest_guard = ingest_locks
        .try_acquire(source_id, SourceIngestKind::Delete)
        .await?;
    let pool = get_pool(&handle).await?;
    let result = sqlx::query("DELETE FROM sources WHERE id = ?")
        .bind(source_id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!("Source {source_id} not found")));
    }

    Ok(())
}

#[tauri::command]
pub async fn list_telegram_sources(
    state: tauri::State<'_, TelegramState>,
    account_id: i64,
) -> AppResult<Vec<TelegramSourceInfo>> {
    let client = {
        let accounts = state.accounts.lock().await;
        crate::telegram::get_client(&accounts, account_id)
            .await?
            .clone()
    };

    let mut sources = Vec::new();
    let mut dialogs = client.iter_dialogs();
    let photo_budget_started_at = Instant::now();
    let photo_budget = Duration::from_millis(TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS);
    while let Some(dialog) = dialogs
        .next()
        .await
        .map_err(|e| AppError::network(e.to_string()))?
    {
        if let Some(mut source) = telegram_source_info_from_peer(dialog.peer()) {
            if photo_budget_started_at.elapsed() < photo_budget {
                source.photo_data_url =
                    peer_photo_data_url_with_timeout(&client, dialog.peer()).await;
            }
            sources.push(source);
        }
    }
    Ok(sources)
}

pub(crate) async fn load_source(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<SourceSyncTarget> {
    sqlx::query_as(
        "SELECT id, source_type, source_subtype, account_id, external_id, title, metadata_zstd, last_sync_state FROM sources WHERE id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?
    .ok_or_else(|| AppError::not_found(format!("Source {source_id} not found")))
}

pub(crate) async fn load_source_record(
    handle: &AppHandle,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<SourceRecord> {
    let row: SourceRecordRow = sqlx::query_as(
        r#"
        SELECT s.id, s.source_type, s.source_subtype, s.account_id, s.external_id,
               s.title, s.metadata_zstd,
               s.last_sync_state, s.last_synced_at, s.is_active, s.is_member, s.created_at,
               ts.username AS telegram_username,
               ts.avatar_cache_key AS telegram_avatar_cache_key
        FROM sources s
        LEFT JOIN telegram_sources ts ON ts.source_id = s.id
        WHERE s.id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?
    .ok_or_else(|| AppError::not_found(format!("Source {source_id} not found")))?;

    source_record_from_row(handle, row)
}

pub(crate) async fn upsert_youtube_video_source(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    metadata: &YoutubeVideoMetadata,
) -> AppResult<i64> {
    let _validated = YoutubeVideoSourceColumns::try_from_metadata(metadata)?;
    let now = now_secs();

    let source_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO sources (
            source_type,
            source_subtype,
            account_id,
            external_id,
            title,
            metadata_zstd,
            is_active,
            is_member,
            created_at
        )
        VALUES ('youtube', 'video', NULL, ?, ?, NULL, 1, 0, ?)
        ON CONFLICT(source_type, source_subtype, external_id)
        WHERE source_type = 'youtube' AND source_subtype = 'video'
        DO UPDATE SET
            title = excluded.title,
            metadata_zstd = NULL,
            is_active = 1
        RETURNING id
        "#,
    )
    .bind(&metadata.video_id)
    .bind(&metadata.title)
    .bind(now)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    upsert_video_source_metadata(tx, source_id, metadata).await?;
    Ok(source_id)
}

pub(crate) async fn upsert_youtube_playlist_source(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    metadata: &YoutubePlaylistMetadata,
) -> AppResult<i64> {
    let _validated = YoutubePlaylistSourceColumns::try_from_metadata(metadata)?;
    let now = now_secs();

    let source_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO sources (
            source_type,
            source_subtype,
            account_id,
            external_id,
            title,
            metadata_zstd,
            is_active,
            is_member,
            created_at
        )
        VALUES ('youtube', 'playlist', NULL, ?, ?, NULL, 1, 0, ?)
        ON CONFLICT(source_type, source_subtype, external_id)
        WHERE source_type = 'youtube' AND source_subtype = 'playlist'
        DO UPDATE SET
            title = excluded.title,
            metadata_zstd = NULL,
            is_active = 1
        RETURNING id
        "#,
    )
    .bind(&metadata.playlist_id)
    .bind(&metadata.title)
    .bind(now)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)?;

    upsert_playlist_source_metadata(tx, source_id, metadata).await?;
    Ok(source_id)
}

#[tauri::command]
pub async fn add_telegram_source(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, TelegramState>,
    request: AddTelegramSourceRequest,
) -> AppResult<SourceRecord> {
    require_source_identity_ready(repair_state.inner()).await?;
    let client = {
        let accounts = state.accounts.lock().await;
        crate::telegram::get_client(&accounts, request.account_id)
            .await?
            .clone()
    };

    let expected_subtype = request.expected_subtype.map(TelegramSourceKind::as_str);
    let resolved = resolve_telegram_source(&client, &request.source_ref, expected_subtype).await?;
    let avatar_cache_key = if let Some(bytes) = resolved.avatar_bytes.as_deref() {
        cache_source_avatar(
            &handle,
            request.account_id,
            &resolved.source_subtype,
            &resolved.external_id,
            bytes,
        )?
    } else {
        None
    };
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
}

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

#[tauri::command]
pub async fn list_sources(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    account_id: Option<i64>,
) -> AppResult<Vec<SourceRecord>> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    let rows: Vec<SourceRecordRow> = if let Some(aid) = account_id {
        sqlx::query_as(
            r#"
            SELECT s.id, s.source_type, s.source_subtype, s.account_id, s.external_id,
                   s.title, s.metadata_zstd,
                   s.last_sync_state, s.last_synced_at, s.is_active, s.is_member, s.created_at,
                   ts.username AS telegram_username,
                   ts.avatar_cache_key AS telegram_avatar_cache_key
            FROM sources s
            LEFT JOIN telegram_sources ts ON ts.source_id = s.id
            WHERE s.account_id = ?
            ORDER BY s.created_at DESC
            "#,
        )
        .bind(aid)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?
    } else {
        sqlx::query_as(
            r#"
            SELECT s.id, s.source_type, s.source_subtype, s.account_id, s.external_id,
                   s.title, s.metadata_zstd,
                   s.last_sync_state, s.last_synced_at, s.is_active, s.is_member, s.created_at,
                   ts.username AS telegram_username,
                   ts.avatar_cache_key AS telegram_avatar_cache_key
            FROM sources s
            LEFT JOIN telegram_sources ts ON ts.source_id = s.id
            ORDER BY s.created_at DESC
            "#,
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?
    };

    rows.into_iter()
        .map(|row| source_record_from_row(&handle, row))
        .collect()
}

fn source_record_from_row_parts(
    row: SourceRecordRow,
    telegram_username: Option<String>,
    avatar_data_url: Option<String>,
) -> SourceRecord {
    let source_subtype = row.source_subtype.unwrap_or_else(|| "unknown".to_string());

    SourceRecord {
        id: row.id,
        source_type: row.source_type,
        source_subtype,
        account_id: row.account_id,
        external_id: row.external_id,
        title: row.title,
        last_sync_state: row.last_sync_state,
        last_synced_at: row.last_synced_at,
        is_member: row.is_member,
        is_active: row.is_active,
        created_at: row.created_at,
        telegram_username,
        avatar_data_url,
    }
}

fn source_record_from_row(handle: &AppHandle, row: SourceRecordRow) -> AppResult<SourceRecord> {
    let telegram_username = if row.source_type == TELEGRAM_SOURCE_TYPE {
        row.telegram_username.clone()
    } else {
        None
    };
    let avatar_cache_key = source_avatar_cache_key_from_row(&row)?;
    let avatar_data_url = avatar_cache_key
        .as_deref()
        .and_then(|cache_key| read_source_avatar_data_url(handle, cache_key));

    Ok(source_record_from_row_parts(
        row,
        telegram_username,
        avatar_data_url,
    ))
}

fn source_avatar_cache_key_from_row(row: &SourceRecordRow) -> AppResult<Option<String>> {
    if row.source_type != TELEGRAM_SOURCE_TYPE {
        return Ok(None);
    }

    Ok(row.telegram_avatar_cache_key.clone())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppErrorKind;
    use crate::sources::test_support::{
        create_canonical_telegram_identity_index, create_youtube_typed_source_tables,
        memory_pool_with_sources,
    };
    use crate::sources::types::TELEGRAM_KIND_CHANNEL;
    use crate::youtube::dto::{
        YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata,
    };
    use serde_json::json;

    #[test]
    fn source_record_parts_allow_non_telegram_source() {
        let record = source_record_from_row_parts(
            SourceRecordRow {
                id: 10,
                source_type: "youtube".to_string(),
                source_subtype: Some("video".to_string()),
                account_id: None,
                external_id: "dQw4w9WgXcQ".to_string(),
                title: Some("Demo video".to_string()),
                metadata_zstd: None,
                last_sync_state: None,
                last_synced_at: None,
                is_active: true,
                is_member: false,
                created_at: 1_700_500,
                telegram_username: None,
                telegram_avatar_cache_key: None,
            },
            None,
            None,
        );

        assert_eq!(record.source_type, "youtube");
        assert_eq!(record.source_subtype, "video");
        assert_eq!(record.account_id, None);
    }

    #[test]
    fn source_record_parts_emit_only_source_subtype() {
        let record = source_record_from_row_parts(
            SourceRecordRow {
                id: 1,
                source_type: TELEGRAM_SOURCE_TYPE.to_string(),
                source_subtype: Some("supergroup".to_string()),
                account_id: Some(1),
                external_id: "12345".to_string(),
                title: Some("source".to_string()),
                metadata_zstd: None,
                last_sync_state: None,
                last_synced_at: None,
                is_active: true,
                is_member: true,
                created_at: 100,
                telegram_username: Some("example".to_string()),
                telegram_avatar_cache_key: None,
            },
            Some("example".to_string()),
            None,
        );

        let json = serde_json::to_value(&record).expect("serialize source record");
        assert_eq!(json["source_subtype"], "supergroup");
        let legacy_key = ["telegram", "source", "kind"].join("_");
        assert!(json.get(&legacy_key).is_none());
    }

    #[test]
    fn avatar_cache_key_skips_non_telegram_metadata() {
        let metadata_zstd = crate::compression::compress_json_bytes(
            br#"{"youtube":{"video_id":"abc123","title":"Demo"}}"#,
        )
        .expect("compress youtube metadata");

        let row = SourceRecordRow {
            id: 10,
            source_type: "youtube".to_string(),
            source_subtype: Some("video".to_string()),
            account_id: None,
            external_id: "abc123".to_string(),
            title: Some("Demo".to_string()),
            metadata_zstd: Some(metadata_zstd),
            last_sync_state: None,
            last_synced_at: None,
            is_active: true,
            is_member: false,
            created_at: 1,
            telegram_username: None,
            telegram_avatar_cache_key: None,
        };

        assert_eq!(source_avatar_cache_key_from_row(&row).unwrap(), None);
    }

    #[tokio::test]
    async fn load_source_returns_not_found_for_missing_source() {
        let pool = memory_pool_with_sources().await;
        let error = match load_source(&pool, 999).await {
            Ok(_) => panic!("expected missing source error"),
            Err(error) => error,
        };

        assert_eq!(error.kind, AppErrorKind::NotFound);
    }

    #[tokio::test]
    async fn telegram_source_upsert_inserts_null_metadata() {
        let pool = memory_pool_with_sources().await;
        create_canonical_telegram_identity_index(&pool).await;
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
        create_canonical_telegram_identity_index(&pool).await;
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

    #[tokio::test]
    async fn telegram_source_upsert_writes_required_identity_and_available_optional_fields() {
        let pool = memory_pool_with_sources().await;
        create_canonical_telegram_identity_index(&pool).await;
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

        let row: (
            i64,
            String,
            String,
            i64,
            String,
            Option<String>,
            Option<i64>,
            Option<String>,
        ) = sqlx::query_as(
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
        create_canonical_telegram_identity_index(&pool).await;
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
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sources WHERE external_id = '00123'")
                .fetch_one(&pool)
                .await
                .expect("count source rows");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn upsert_youtube_video_source_handles_legacy_not_null_telegram_kind() {
        let pool = legacy_not_null_telegram_kind_pool().await;
        let mut tx = pool.begin().await.expect("begin tx");

        let source_id = upsert_youtube_video_source(&mut tx, &youtube_video_metadata())
            .await
            .expect("upsert youtube video");
        tx.commit().await.expect("commit");

        let legacy_kind_column = legacy_source_kind_column();
        let select_sql = format!(
            "SELECT source_type, source_subtype, {legacy_kind_column}, external_id FROM sources WHERE id = ?"
        );
        let row: (String, String, Option<String>, String) = sqlx::query_as(&select_sql)
            .bind(source_id)
            .fetch_one(&pool)
            .await
            .expect("load source");

        assert_eq!(row.0, "youtube");
        assert_eq!(row.1, "video");
        assert_eq!(row.2.as_deref(), Some("channel"));
        assert_eq!(row.3, "dQw4w9WgXcQ");
    }

    #[tokio::test]
    async fn upsert_youtube_playlist_source_handles_legacy_not_null_telegram_kind() {
        let pool = legacy_not_null_telegram_kind_pool().await;
        let mut tx = pool.begin().await.expect("begin tx");

        let source_id = upsert_youtube_playlist_source(&mut tx, &youtube_playlist_metadata())
            .await
            .expect("upsert youtube playlist");
        tx.commit().await.expect("commit");

        let legacy_kind_column = legacy_source_kind_column();
        let select_sql = format!(
            "SELECT source_type, source_subtype, {legacy_kind_column}, external_id FROM sources WHERE id = ?"
        );
        let row: (String, String, Option<String>, String) = sqlx::query_as(&select_sql)
            .bind(source_id)
            .fetch_one(&pool)
            .await
            .expect("load source");

        assert_eq!(row.0, "youtube");
        assert_eq!(row.1, "playlist");
        assert_eq!(row.2.as_deref(), Some("channel"));
        assert_eq!(row.3, "PLdemo");
    }

    #[tokio::test]
    async fn upsert_youtube_video_source_writes_typed_row_and_null_source_metadata() {
        let pool = memory_pool_with_sources().await;
        create_youtube_typed_source_tables(&pool).await;
        create_youtube_unique_indexes(&pool).await;
        let mut tx = pool.begin().await.expect("begin tx");

        let source_id = upsert_youtube_video_source(&mut tx, &youtube_video_metadata())
            .await
            .expect("upsert youtube video");
        tx.commit().await.expect("commit");

        let source_metadata: Option<Vec<u8>> =
            sqlx::query_scalar("SELECT metadata_zstd FROM sources WHERE id = ?")
                .bind(source_id)
                .fetch_one(&pool)
                .await
                .expect("load source metadata");
        assert_eq!(source_metadata, None);

        let typed: (String, Option<String>, String, String, Option<Vec<u8>>) = sqlx::query_as(
            "SELECT video_id, title, canonical_url, availability_status, raw_metadata_zstd FROM youtube_video_sources WHERE source_id = ?",
        )
        .bind(source_id)
        .fetch_one(&pool)
        .await
        .expect("load typed video source");
        assert_eq!(typed.0, "dQw4w9WgXcQ");
        assert_eq!(typed.1.as_deref(), Some("Demo video"));
        assert_eq!(typed.2, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert_eq!(typed.3, "available");
        assert!(typed.4.is_some());
    }

    #[tokio::test]
    async fn upsert_youtube_playlist_source_writes_typed_row_and_null_source_metadata() {
        let pool = memory_pool_with_sources().await;
        create_youtube_typed_source_tables(&pool).await;
        create_youtube_unique_indexes(&pool).await;
        let mut tx = pool.begin().await.expect("begin tx");

        let source_id = upsert_youtube_playlist_source(&mut tx, &youtube_playlist_metadata())
            .await
            .expect("upsert youtube playlist");
        tx.commit().await.expect("commit");

        let source_metadata: Option<Vec<u8>> =
            sqlx::query_scalar("SELECT metadata_zstd FROM sources WHERE id = ?")
                .bind(source_id)
                .fetch_one(&pool)
                .await
                .expect("load source metadata");
        assert_eq!(source_metadata, None);

        let typed: (String, Option<String>, String, i64) = sqlx::query_as(
            "SELECT playlist_id, title, canonical_url, video_count FROM youtube_playlist_sources WHERE source_id = ?",
        )
        .bind(source_id)
        .fetch_one(&pool)
        .await
        .expect("load typed playlist source");
        assert_eq!(typed.0, "PLdemo");
        assert_eq!(typed.1.as_deref(), Some("Demo playlist"));
        assert_eq!(typed.2, "https://www.youtube.com/playlist?list=PLdemo");
        assert_eq!(typed.3, 0);
    }

    #[tokio::test]
    async fn upsert_youtube_video_source_conflict_clears_existing_legacy_blob() {
        let pool = memory_pool_with_sources().await;
        create_youtube_typed_source_tables(&pool).await;
        create_youtube_unique_indexes(&pool).await;
        let legacy_blob = crate::compression::compress_json_bytes(br#"{"legacy":true}"#)
            .expect("compress legacy");
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (77, 'youtube', 'video', 'dQw4w9WgXcQ', 'Old', ?, 1, 0, 1)",
        )
        .bind(legacy_blob)
        .execute(&pool)
        .await
        .expect("insert legacy source");
        let mut tx = pool.begin().await.expect("begin tx");

        let source_id = upsert_youtube_video_source(&mut tx, &youtube_video_metadata())
            .await
            .expect("upsert existing youtube video");
        tx.commit().await.expect("commit");

        assert_eq!(source_id, 77);
        let source_metadata: Option<Vec<u8>> =
            sqlx::query_scalar("SELECT metadata_zstd FROM sources WHERE id = 77")
                .fetch_one(&pool)
                .await
                .expect("load source metadata");
        assert_eq!(source_metadata, None);
    }

    #[tokio::test]
    async fn upsert_youtube_video_source_rejects_invalid_canonical_url_without_source_row() {
        let pool = memory_pool_with_sources().await;
        create_youtube_typed_source_tables(&pool).await;
        create_youtube_unique_indexes(&pool).await;
        let mut metadata = youtube_video_metadata();
        metadata.canonical_url = "https://example.com/watch?v=dQw4w9WgXcQ".to_string();
        let mut tx = pool.begin().await.expect("begin tx");

        let error = upsert_youtube_video_source(&mut tx, &metadata)
            .await
            .expect_err("invalid metadata rejected");
        tx.rollback().await.expect("rollback");

        assert_eq!(error.kind, AppErrorKind::Validation);
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sources WHERE external_id = 'dQw4w9WgXcQ'")
                .fetch_one(&pool)
                .await
                .expect("count source rows");
        assert_eq!(count, 0);
    }

    async fn create_youtube_unique_indexes(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "CREATE UNIQUE INDEX idx_sources_unique_youtube_video ON sources(source_type, source_subtype, external_id) WHERE source_type = 'youtube' AND source_subtype = 'video'",
        )
        .execute(pool)
        .await
        .expect("create video index");
        sqlx::query(
            "CREATE UNIQUE INDEX idx_sources_unique_youtube_playlist ON sources(source_type, source_subtype, external_id) WHERE source_type = 'youtube' AND source_subtype = 'playlist'",
        )
        .execute(pool)
        .await
        .expect("create playlist index");
    }

    async fn legacy_not_null_telegram_kind_pool() -> sqlx::SqlitePool {
        let pool = crate::sources::test_support::memory_pool().await;
        let legacy_kind_column = legacy_source_kind_column();
        let create_sources_sql = format!(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY,
                source_type TEXT NOT NULL,
                source_subtype TEXT,
                {legacy_kind_column} TEXT NOT NULL DEFAULT 'channel',
                account_id INTEGER,
                external_id TEXT NOT NULL,
                title TEXT,
                metadata_zstd BLOB,
                last_sync_state INTEGER,
                last_synced_at INTEGER,
                is_active INTEGER NOT NULL DEFAULT 1,
                is_member INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL
            )
            "#,
        );
        sqlx::query(&create_sources_sql)
            .execute(&pool)
            .await
            .expect("create legacy sources");
        sqlx::query(
            r#"
            CREATE UNIQUE INDEX idx_sources_unique_youtube_video
            ON sources(source_type, source_subtype, external_id)
            WHERE source_type = 'youtube' AND source_subtype = 'video'
            "#,
        )
        .execute(&pool)
        .await
        .expect("create video index");
        sqlx::query(
            r#"
            CREATE UNIQUE INDEX idx_sources_unique_youtube_playlist
            ON sources(source_type, source_subtype, external_id)
            WHERE source_type = 'youtube' AND source_subtype = 'playlist'
            "#,
        )
        .execute(&pool)
        .await
        .expect("create playlist index");
        create_youtube_typed_source_tables(&pool).await;
        pool
    }

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

    fn legacy_source_kind_column() -> String {
        ["telegram", "source", "kind"].join("_")
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
}
