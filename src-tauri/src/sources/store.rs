use tauri::AppHandle;
use tokio::time::{Duration, Instant};

use crate::compression::compress_json_bytes;
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::telegram::TelegramState;
use crate::youtube::dto::{YoutubePlaylistMetadata, YoutubeVideoMetadata};

use super::avatar::{
    cache_source_avatar, peer_photo_data_url_with_timeout, read_source_avatar_data_url,
    TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS,
};
use super::peer_resolution::{
    decode_source_metadata, encode_source_metadata, resolve_telegram_source,
    source_metadata_for_added_source, telegram_source_info_from_peer,
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
    pub expected_kind: Option<TelegramSourceKind>,
}

#[tauri::command]
pub async fn delete_source(
    handle: AppHandle,
    ingest_locks: tauri::State<'_, SourceIngestLocks>,
    source_id: i64,
) -> AppResult<()> {
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
        "SELECT id, source_type, source_subtype, COALESCE(telegram_source_kind, '') AS telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state FROM sources WHERE id = ?",
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
        "SELECT id, source_type, source_subtype, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at FROM sources WHERE id = ?",
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
    let metadata_zstd = encode_youtube_metadata(metadata)?;
    let now = now_secs();

    sqlx::query_scalar(
        r#"
        INSERT INTO sources (
            source_type,
            source_subtype,
            telegram_source_kind,
            account_id,
            external_id,
            title,
            metadata_zstd,
            is_active,
            is_member,
            created_at
        )
        VALUES ('youtube', 'video', '', NULL, ?, ?, ?, 1, 0, ?)
        ON CONFLICT(source_type, source_subtype, external_id)
        WHERE source_type = 'youtube' AND source_subtype = 'video'
        DO UPDATE SET
            title = excluded.title,
            metadata_zstd = excluded.metadata_zstd,
            is_active = 1
        RETURNING id
        "#,
    )
    .bind(&metadata.video_id)
    .bind(&metadata.title)
    .bind(metadata_zstd)
    .bind(now)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn upsert_youtube_playlist_source(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    metadata: &YoutubePlaylistMetadata,
) -> AppResult<i64> {
    let metadata_zstd = encode_youtube_metadata(metadata)?;
    let now = now_secs();

    sqlx::query_scalar(
        r#"
        INSERT INTO sources (
            source_type,
            source_subtype,
            telegram_source_kind,
            account_id,
            external_id,
            title,
            metadata_zstd,
            is_active,
            is_member,
            created_at
        )
        VALUES ('youtube', 'playlist', '', NULL, ?, ?, ?, 1, 0, ?)
        ON CONFLICT(source_type, source_subtype, external_id)
        WHERE source_type = 'youtube' AND source_subtype = 'playlist'
        DO UPDATE SET
            title = excluded.title,
            metadata_zstd = excluded.metadata_zstd,
            is_active = 1
        RETURNING id
        "#,
    )
    .bind(&metadata.playlist_id)
    .bind(&metadata.title)
    .bind(metadata_zstd)
    .bind(now)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::database)
}

#[tauri::command]
pub async fn add_telegram_source(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    request: AddTelegramSourceRequest,
) -> AppResult<SourceRecord> {
    let client = {
        let accounts = state.accounts.lock().await;
        crate::telegram::get_client(&accounts, request.account_id)
            .await?
            .clone()
    };

    let expected_kind = request.expected_kind.map(TelegramSourceKind::as_str);
    let resolved = resolve_telegram_source(&client, &request.source_ref, expected_kind).await?;
    let avatar_cache_key = if let Some(bytes) = resolved.avatar_bytes.as_deref() {
        cache_source_avatar(
            &handle,
            request.account_id,
            &resolved.telegram_source_kind,
            &resolved.external_id,
            bytes,
        )?
    } else {
        None
    };
    let metadata_zstd = encode_source_metadata(&source_metadata_for_added_source(
        &request.source_ref,
        expected_kind,
        &resolved,
        avatar_cache_key,
    ))?;
    let now = now_secs();

    let pool = get_pool(&handle).await?;
    let row: SourceRecordRow = sqlx::query_as(
        r#"
        INSERT INTO sources (
            source_type,
            source_subtype,
            telegram_source_kind,
            external_id,
            title,
            metadata_zstd,
            is_active,
            is_member,
            account_id,
            created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?)
        ON CONFLICT(account_id, source_type, telegram_source_kind, external_id) DO UPDATE SET
            title = excluded.title,
            source_subtype = excluded.source_subtype,
            metadata_zstd = excluded.metadata_zstd,
            is_member = excluded.is_member,
            account_id = excluded.account_id
        RETURNING
            id,
            source_type,
            source_subtype,
            telegram_source_kind,
            account_id,
            external_id,
            title,
            metadata_zstd,
            last_sync_state,
            last_synced_at,
            is_active,
            is_member,
            created_at
        "#,
    )
    .bind(SourceType::Telegram.as_str())
    .bind(&resolved.telegram_source_kind)
    .bind(&resolved.telegram_source_kind)
    .bind(&resolved.external_id)
    .bind(&resolved.title)
    .bind(metadata_zstd)
    .bind(resolved.is_member)
    .bind(request.account_id)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;
    source_record_from_row(&handle, row)
}

#[tauri::command]
pub async fn list_sources(
    handle: AppHandle,
    account_id: Option<i64>,
) -> AppResult<Vec<SourceRecord>> {
    let pool = get_pool(&handle).await?;
    let rows: Vec<SourceRecordRow> = if let Some(aid) = account_id {
        sqlx::query_as(
            "SELECT id, source_type, source_subtype, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at FROM sources WHERE account_id = ? ORDER BY created_at DESC",
        )
        .bind(aid)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?
    } else {
        sqlx::query_as(
            "SELECT id, source_type, source_subtype, telegram_source_kind, account_id, external_id, title, metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at FROM sources ORDER BY created_at DESC",
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
    let telegram_source_kind = if row.source_type == TELEGRAM_SOURCE_TYPE {
        row.telegram_source_kind
    } else {
        None
    };

    SourceRecord {
        id: row.id,
        source_type: row.source_type,
        source_subtype: row.source_subtype,
        telegram_source_kind,
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
        decode_source_metadata(row.metadata_zstd.as_deref())?.peer_username()
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

    let metadata = decode_source_metadata(row.metadata_zstd.as_deref())?;
    Ok(metadata.avatar_cache_key)
}

fn encode_youtube_metadata(metadata: &impl serde::Serialize) -> AppResult<Vec<u8>> {
    let json = serde_json::to_vec(metadata).map_err(|e| AppError::internal(e.to_string()))?;
    compress_json_bytes(&json).map_err(AppError::internal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppErrorKind;
    use crate::sources::test_support::memory_pool_with_sources;
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
                telegram_source_kind: None,
                account_id: None,
                external_id: "dQw4w9WgXcQ".to_string(),
                title: Some("Demo video".to_string()),
                metadata_zstd: None,
                last_sync_state: None,
                last_synced_at: None,
                is_active: true,
                is_member: false,
                created_at: 1_700_500,
            },
            None,
            None,
        );

        assert_eq!(record.source_type, "youtube");
        assert_eq!(record.source_subtype.as_deref(), Some("video"));
        assert_eq!(record.telegram_source_kind, None);
        assert_eq!(record.account_id, None);
    }

    #[test]
    fn source_record_parts_hides_non_telegram_compatibility_kind() {
        let record = source_record_from_row_parts(
            SourceRecordRow {
                id: 10,
                source_type: "youtube".to_string(),
                source_subtype: Some("video".to_string()),
                telegram_source_kind: Some("channel".to_string()),
                account_id: None,
                external_id: "dQw4w9WgXcQ".to_string(),
                title: Some("Demo video".to_string()),
                metadata_zstd: None,
                last_sync_state: None,
                last_synced_at: None,
                is_active: true,
                is_member: false,
                created_at: 1_700_500,
            },
            None,
            None,
        );

        assert_eq!(record.source_type, "youtube");
        assert_eq!(record.telegram_source_kind, None);
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
            telegram_source_kind: None,
            account_id: None,
            external_id: "abc123".to_string(),
            title: Some("Demo".to_string()),
            metadata_zstd: Some(metadata_zstd),
            last_sync_state: None,
            last_synced_at: None,
            is_active: true,
            is_member: false,
            created_at: 1,
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
    async fn upsert_youtube_video_source_handles_legacy_not_null_telegram_kind() {
        let pool = legacy_not_null_telegram_kind_pool().await;
        let mut tx = pool.begin().await.expect("begin tx");

        let source_id = upsert_youtube_video_source(&mut tx, &youtube_video_metadata())
            .await
            .expect("upsert youtube video");
        tx.commit().await.expect("commit");

        let row: (String, String, Option<String>, String) = sqlx::query_as(
            "SELECT source_type, source_subtype, telegram_source_kind, external_id FROM sources WHERE id = ?",
        )
        .bind(source_id)
        .fetch_one(&pool)
        .await
        .expect("load source");

        assert_eq!(row.0, "youtube");
        assert_eq!(row.1, "video");
        assert_eq!(row.2.as_deref(), Some(""));
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

        let row: (String, String, Option<String>, String) = sqlx::query_as(
            "SELECT source_type, source_subtype, telegram_source_kind, external_id FROM sources WHERE id = ?",
        )
        .bind(source_id)
        .fetch_one(&pool)
        .await
        .expect("load source");

        assert_eq!(row.0, "youtube");
        assert_eq!(row.1, "playlist");
        assert_eq!(row.2.as_deref(), Some(""));
        assert_eq!(row.3, "PLdemo");
    }

    async fn legacy_not_null_telegram_kind_pool() -> sqlx::SqlitePool {
        let pool = crate::sources::test_support::memory_pool().await;
        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY,
                source_type TEXT NOT NULL,
                source_subtype TEXT,
                telegram_source_kind TEXT NOT NULL DEFAULT 'channel',
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
        )
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
        pool
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
