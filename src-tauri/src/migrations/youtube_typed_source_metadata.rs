use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppErrorKind, AppResult};
use crate::tx::{begin_immediate_on_connection, finish_connection_transaction};

pub(super) const YOUTUBE_TYPED_SOURCE_METADATA_VERSION: i64 = 20;
pub(super) const YOUTUBE_TYPED_SOURCE_METADATA_DESCRIPTION: &str =
    "add youtube typed source metadata";
pub(super) const YOUTUBE_TYPED_SOURCE_METADATA_SENTINEL_SQL: &str =
    include_str!("../../migrations/20.sql");

pub(super) async fn apply_youtube_typed_source_metadata_if_needed(db_url: &str) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_youtube_typed_source_metadata_on_connection(&mut conn).await
}

pub(super) async fn apply_youtube_typed_source_metadata_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_previous_migration_recorded(conn).await?;
    if migration_20_recorded(conn).await? {
        return Ok(());
    }

    let started_at = Instant::now();
    begin_immediate_on_connection(conn).await?;

    let result = async {
        crate::youtube::source_metadata::create_youtube_typed_source_tables(&mut *conn).await?;
        backfill_youtube_source_metadata(conn).await
    }
    .await;

    finish_connection_transaction(conn, result).await?;

    record_migration_success(
        conn,
        YOUTUBE_TYPED_SOURCE_METADATA_VERSION,
        YOUTUBE_TYPED_SOURCE_METADATA_DESCRIPTION,
        expected_migration_20_checksum(),
        started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
    )
    .await
}

async fn ensure_previous_migration_recorded(conn: &mut SqliteConnection) -> AppResult<()> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 19 AND success = 1",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if exists == 0 {
        return Err(AppError::validation(
            "YouTube typed source metadata migration 20 requires migration 19",
        ));
    }
    Ok(())
}

async fn migration_20_recorded(conn: &mut SqliteConnection) -> AppResult<bool> {
    let checksum = expected_migration_20_checksum();
    let row: Option<(Vec<u8>, bool)> =
        sqlx::query_as("SELECT checksum, success FROM _sqlx_migrations WHERE version = ?")
            .bind(YOUTUBE_TYPED_SOURCE_METADATA_VERSION)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?;

    match row {
        None => Ok(false),
        Some((applied_checksum, true)) if applied_checksum == checksum => Ok(true),
        Some((_applied_checksum, true)) => Err(AppError::internal(
            "Migration 20 checksum does not match the runner-managed YouTube typed source metadata sentinel",
        )),
        Some((_applied_checksum, false)) => Err(AppError::internal(
            "Migration 20 is marked as failed in _sqlx_migrations",
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

fn expected_migration_20_checksum() -> Vec<u8> {
    Sha384::digest(YOUTUBE_TYPED_SOURCE_METADATA_SENTINEL_SQL.as_bytes()).to_vec()
}

#[derive(sqlx::FromRow)]
struct LegacyYoutubeSourceRow {
    id: i64,
    source_subtype: String,
    external_id: String,
    metadata_zstd: Option<Vec<u8>>,
}

async fn backfill_youtube_source_metadata(conn: &mut SqliteConnection) -> AppResult<()> {
    let rows: Vec<LegacyYoutubeSourceRow> = sqlx::query_as(
        r#"
        SELECT id, source_subtype, external_id, metadata_zstd
        FROM sources
        WHERE source_type = 'youtube'
          AND source_subtype IN ('video', 'playlist')
          AND metadata_zstd IS NOT NULL
        ORDER BY id
        "#,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    for row in rows {
        let Some(bytes) = row.metadata_zstd.as_deref() else {
            continue;
        };
        match row.source_subtype.as_str() {
            "video" => {
                if let Some(metadata) =
                    crate::youtube::source_metadata::decode_legacy_video_source_metadata(bytes)
                {
                    if metadata.video_id == row.external_id
                        && insert_video_source_metadata(conn, row.id, &metadata).await?
                    {
                        clear_source_blob(conn, row.id).await?;
                    }
                }
            }
            "playlist" => {
                if let Some(metadata) =
                    crate::youtube::source_metadata::decode_legacy_playlist_source_metadata(bytes)
                {
                    if metadata.playlist_id == row.external_id
                        && insert_playlist_source_metadata(conn, row.id, &metadata).await?
                    {
                        clear_source_blob(conn, row.id).await?;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

async fn insert_video_source_metadata(
    conn: &mut SqliteConnection,
    source_id: i64,
    metadata: &crate::youtube::dto::YoutubeVideoMetadata,
) -> AppResult<bool> {
    match crate::youtube::source_metadata::insert_video_source_metadata_on_connection(
        conn, source_id, metadata,
    )
    .await
    {
        Ok(()) => Ok(true),
        Err(error) if error.kind == AppErrorKind::Validation => Ok(false),
        Err(error) => Err(error),
    }
}

async fn insert_playlist_source_metadata(
    conn: &mut SqliteConnection,
    source_id: i64,
    metadata: &crate::youtube::dto::YoutubePlaylistMetadata,
) -> AppResult<bool> {
    match crate::youtube::source_metadata::insert_playlist_source_metadata_on_connection(
        conn, source_id, metadata,
    )
    .await
    {
        Ok(()) => Ok(true),
        Err(error) if error.kind == AppErrorKind::Validation => Ok(false),
        Err(error) => Err(error),
    }
}

async fn clear_source_blob(conn: &mut SqliteConnection, source_id: i64) -> AppResult<()> {
    sqlx::query("UPDATE sources SET metadata_zstd = NULL WHERE id = ?")
        .bind(source_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_json_bytes;
    use crate::migrations::build_migrations;
    use crate::youtube::dto::{
        YoutubeAvailabilityStatus, YoutubePlaylistMetadata, YoutubeVideoForm, YoutubeVideoMetadata,
    };
    use serde_json::json;

    #[tokio::test]
    async fn migration_20_backfills_valid_video_and_playlist_metadata_and_clears_source_blobs() {
        let mut conn = memory_conn_with_history_through_19().await;
        insert_legacy_video_source(&mut conn, 101, "video01", "Video title").await;
        insert_legacy_playlist_source(&mut conn, 201, "PLdemo", "Playlist title").await;

        apply_youtube_typed_source_metadata_on_connection(&mut conn)
            .await
            .expect("apply v20");

        let video: (String, Option<String>, String, Option<Vec<u8>>) = sqlx::query_as(
            "SELECT video_id, title, canonical_url, raw_metadata_zstd FROM youtube_video_sources WHERE source_id = 101",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load video typed row");
        assert_eq!(video.0, "video01");
        assert_eq!(video.1.as_deref(), Some("Video title"));
        assert_eq!(video.2, "https://www.youtube.com/watch?v=video01");
        assert!(video.3.is_some());

        let playlist: (String, Option<String>, String) = sqlx::query_as(
            "SELECT playlist_id, title, canonical_url FROM youtube_playlist_sources WHERE source_id = 201",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load playlist typed row");
        assert_eq!(playlist.0, "PLdemo");
        assert_eq!(playlist.1.as_deref(), Some("Playlist title"));
        assert_eq!(playlist.2, "https://www.youtube.com/playlist?list=PLdemo");

        let remaining_blobs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sources WHERE id IN (101, 201) AND metadata_zstd IS NOT NULL",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count source blobs");
        assert_eq!(remaining_blobs, 0);
    }

    #[tokio::test]
    async fn migration_20_skips_corrupt_wrong_shape_and_mismatched_blobs_without_failing() {
        let mut conn = memory_conn_with_history_through_19().await;
        insert_corrupt_youtube_source(&mut conn, 301, "video", "bad-video").await;
        insert_mismatched_video_source(&mut conn, 302, "source-video", "metadata-video").await;
        insert_wrong_shape_playlist_source(&mut conn, 303, "PLshape").await;

        apply_youtube_typed_source_metadata_on_connection(&mut conn)
            .await
            .expect("apply v20");

        let video_typed_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM youtube_video_sources WHERE source_id IN (301, 302, 303)",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count video typed rows");
        assert_eq!(video_typed_count, 0);

        let playlist_typed_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM youtube_playlist_sources WHERE source_id IN (301, 302, 303)",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count playlist typed rows");
        assert_eq!(playlist_typed_count, 0);

        let inert_blob_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sources WHERE id IN (301, 302, 303) AND metadata_zstd IS NOT NULL",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count inert blobs");
        assert_eq!(inert_blob_count, 3);
    }

    #[tokio::test]
    async fn migration_20_is_idempotent_when_checksum_matches() {
        let mut conn = memory_conn_with_history_through_19().await;

        apply_youtube_typed_source_metadata_on_connection(&mut conn)
            .await
            .expect("first v20");
        apply_youtube_typed_source_metadata_on_connection(&mut conn)
            .await
            .expect("second v20");

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 20")
                .fetch_one(&mut conn)
                .await
                .expect("count v20 history");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn migration_20_sentinel_checksum_is_recorded() {
        let mut conn = memory_conn_with_history_through_19().await;

        apply_youtube_typed_source_metadata_on_connection(&mut conn)
            .await
            .expect("apply v20");

        let row: (String, bool, Vec<u8>) = sqlx::query_as(
            "SELECT description, success, checksum FROM _sqlx_migrations WHERE version = 20",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load v20 history");

        assert_eq!(row.0, YOUTUBE_TYPED_SOURCE_METADATA_DESCRIPTION);
        assert!(row.1);
        assert_eq!(row.2, expected_migration_20_checksum());
    }

    async fn memory_conn_with_history_through_19() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )
            "#,
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
            sqlx::query(
                "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)",
            )
            .bind(migration.version)
            .bind(migration.description)
            .bind(Sha384::digest(migration.sql.as_bytes()).to_vec())
            .execute(&mut conn)
            .await
            .expect("record migration");
        }

        crate::migrations::source_identity_cleanup::apply_source_identity_cleanup_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v19");

        conn
    }

    async fn insert_legacy_video_source(
        conn: &mut SqliteConnection,
        id: i64,
        video_id: &str,
        title: &str,
    ) {
        let metadata = YoutubeVideoMetadata {
            video_id: video_id.to_string(),
            canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
            title: Some(title.to_string()),
            channel_title: Some("Demo channel".to_string()),
            channel_id: Some("channel-1".to_string()),
            channel_handle: Some("@demo".to_string()),
            channel_url: Some("https://www.youtube.com/@demo".to_string()),
            author_display: Some("Demo channel".to_string()),
            published_at: Some("2026-05-17".to_string()),
            duration_seconds: Some(123),
            description: Some("Description".to_string()),
            thumbnail_url: None,
            tags: Vec::new(),
            chapters: Vec::new(),
            view_count: None,
            like_count: None,
            comment_count: None,
            category: None,
            video_form: YoutubeVideoForm::Regular,
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: json!({ "id": video_id }),
        };
        insert_legacy_source_blob(conn, id, "video", video_id, title, &metadata).await;
    }

    async fn insert_legacy_playlist_source(
        conn: &mut SqliteConnection,
        id: i64,
        playlist_id: &str,
        title: &str,
    ) {
        let metadata = YoutubePlaylistMetadata {
            playlist_id: playlist_id.to_string(),
            canonical_url: format!("https://www.youtube.com/playlist?list={playlist_id}"),
            title: Some(title.to_string()),
            channel_title: Some("Demo channel".to_string()),
            channel_id: Some("channel-1".to_string()),
            channel_handle: Some("@demo".to_string()),
            channel_url: Some("https://www.youtube.com/@demo".to_string()),
            thumbnail_url: None,
            video_count: Some(0),
            items: Vec::new(),
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: json!({ "id": playlist_id }),
        };
        insert_legacy_source_blob(conn, id, "playlist", playlist_id, title, &metadata).await;
    }

    async fn insert_corrupt_youtube_source(
        conn: &mut SqliteConnection,
        id: i64,
        source_subtype: &str,
        external_id: &str,
    ) {
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (?, 'youtube', ?, ?, 'corrupt', ?, 1, 0, 1)",
        )
        .bind(id)
        .bind(source_subtype)
        .bind(external_id)
        .bind(vec![0, 1, 2, 3, 4])
        .execute(conn)
        .await
        .expect("insert corrupt source");
    }

    async fn insert_mismatched_video_source(
        conn: &mut SqliteConnection,
        id: i64,
        source_video_id: &str,
        metadata_video_id: &str,
    ) {
        let metadata = YoutubeVideoMetadata {
            video_id: metadata_video_id.to_string(),
            canonical_url: format!("https://www.youtube.com/watch?v={metadata_video_id}"),
            title: Some("Mismatched video".to_string()),
            channel_title: None,
            channel_id: None,
            channel_handle: None,
            channel_url: None,
            author_display: None,
            published_at: None,
            duration_seconds: None,
            description: None,
            thumbnail_url: None,
            tags: Vec::new(),
            chapters: Vec::new(),
            view_count: None,
            like_count: None,
            comment_count: None,
            category: None,
            video_form: YoutubeVideoForm::Regular,
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: json!({ "id": metadata_video_id }),
        };
        insert_legacy_source_blob(
            conn,
            id,
            "video",
            source_video_id,
            "Mismatched video",
            &metadata,
        )
        .await;
    }

    async fn insert_wrong_shape_playlist_source(
        conn: &mut SqliteConnection,
        id: i64,
        playlist_id: &str,
    ) {
        let wrong_shape = json!({
            "video_id": playlist_id,
            "canonical_url": format!("https://www.youtube.com/watch?v={playlist_id}")
        });
        insert_legacy_source_blob(
            conn,
            id,
            "playlist",
            playlist_id,
            "Wrong shape",
            &wrong_shape,
        )
        .await;
    }

    async fn insert_legacy_source_blob<T: serde::Serialize>(
        conn: &mut SqliteConnection,
        id: i64,
        source_subtype: &str,
        external_id: &str,
        title: &str,
        metadata: &T,
    ) {
        let json = serde_json::to_vec(metadata).expect("serialize metadata");
        let blob = compress_json_bytes(&json).expect("compress metadata");
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (?, 'youtube', ?, ?, ?, ?, 1, 0, 1)",
        )
        .bind(id)
        .bind(source_subtype)
        .bind(external_id)
        .bind(title)
        .bind(blob)
        .execute(conn)
        .await
        .expect("insert legacy source");
    }
}
