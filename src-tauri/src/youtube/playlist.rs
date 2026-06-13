use serde_json::to_vec;

use crate::compression::compress_json_bytes;
use crate::error::{AppError, AppResult};
use crate::sources::upsert_youtube_video_source;
use crate::time::now_secs;

use super::dto::{
    YoutubeAvailabilityStatus, YoutubePlaylistItemMetadata, YoutubePlaylistMetadata,
    YoutubeVideoForm, YoutubeVideoMetadata,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct UpsertPlaylistItemsOptions {
    pub(crate) materialize_video_sources: bool,
}

impl Default for UpsertPlaylistItemsOptions {
    fn default() -> Self {
        Self {
            materialize_video_sources: true,
        }
    }
}

pub(crate) async fn upsert_playlist_items(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    playlist_source_id: i64,
    metadata: &YoutubePlaylistMetadata,
) -> AppResult<()> {
    upsert_playlist_items_with_options(
        tx,
        playlist_source_id,
        metadata,
        UpsertPlaylistItemsOptions::default(),
    )
    .await
}

pub(crate) async fn upsert_playlist_items_with_options(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    playlist_source_id: i64,
    metadata: &YoutubePlaylistMetadata,
    options: UpsertPlaylistItemsOptions,
) -> AppResult<()> {
    let now = now_secs();
    let mut seen_video_ids = Vec::with_capacity(metadata.items.len());

    for item in &metadata.items {
        seen_video_ids.push(item.video_id.clone());
        let video_source_id = if !can_create_video_source(item) {
            None
        } else if options.materialize_video_sources {
            Some(upsert_youtube_video_source(tx, &video_metadata_from_playlist_item(item)).await?)
        } else {
            load_existing_youtube_video_source_id(tx, &item.video_id).await?
        };
        let metadata_zstd = encode_playlist_item_metadata(item)?;
        let availability_status = availability_status_wire(&item.availability_status);

        sqlx::query(
            r#"
            INSERT INTO youtube_playlist_items (
                playlist_source_id,
                video_source_id,
                video_id,
                position,
                title_snapshot,
                url,
                thumbnail_url,
                availability_status,
                is_removed_from_playlist,
                last_seen_at,
                metadata_zstd,
                created_at,
                updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?, ?)
            ON CONFLICT(playlist_source_id, video_id) DO UPDATE SET
                video_source_id = excluded.video_source_id,
                position = excluded.position,
                title_snapshot = excluded.title_snapshot,
                url = excluded.url,
                thumbnail_url = excluded.thumbnail_url,
                availability_status = excluded.availability_status,
                is_removed_from_playlist = 0,
                last_seen_at = excluded.last_seen_at,
                metadata_zstd = excluded.metadata_zstd,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(playlist_source_id)
        .bind(video_source_id)
        .bind(&item.video_id)
        .bind(item.position)
        .bind(&item.title_snapshot)
        .bind(&item.url)
        .bind(&item.thumbnail_url)
        .bind(availability_status)
        .bind(now)
        .bind(metadata_zstd)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
    }

    mark_missing_playlist_items_removed(tx, playlist_source_id, &seen_video_ids, now).await
}

async fn load_existing_youtube_video_source_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    video_id: &str,
) -> AppResult<Option<i64>> {
    sqlx::query_scalar(
        r#"
        SELECT yvs.source_id
        FROM youtube_video_sources yvs
        JOIN sources s ON s.id = yvs.source_id
        WHERE s.source_type = 'youtube'
          AND s.source_subtype = 'video'
          AND yvs.video_id = ?
        "#,
    )
    .bind(video_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::database)
}

async fn mark_missing_playlist_items_removed(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    playlist_source_id: i64,
    seen_video_ids: &[String],
    now: i64,
) -> AppResult<()> {
    if seen_video_ids.is_empty() {
        sqlx::query(
            r#"
            UPDATE youtube_playlist_items
            SET is_removed_from_playlist = 1,
                availability_status = 'removed_from_playlist',
                updated_at = ?
            WHERE playlist_source_id = ?
            "#,
        )
        .bind(now)
        .bind(playlist_source_id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;
        return Ok(());
    }

    let mut query = sqlx::QueryBuilder::new(
        r#"
        UPDATE youtube_playlist_items
        SET is_removed_from_playlist = 1,
            availability_status = 'removed_from_playlist',
            updated_at = 
        "#,
    );
    query.push_bind(now);
    query.push(" WHERE playlist_source_id = ");
    query.push_bind(playlist_source_id);
    query.push(" AND video_id NOT IN (");
    let mut separated = query.separated(", ");
    for video_id in seen_video_ids {
        separated.push_bind(video_id);
    }
    separated.push_unseparated(")");

    query
        .build()
        .execute(&mut **tx)
        .await
        .map_err(AppError::database)?;

    Ok(())
}

fn can_create_video_source(item: &YoutubePlaylistItemMetadata) -> bool {
    matches!(
        item.availability_status,
        YoutubeAvailabilityStatus::Available
            | YoutubeAvailabilityStatus::Upcoming
            | YoutubeAvailabilityStatus::LiveNow
            | YoutubeAvailabilityStatus::LiveEndedTranscriptPending
            | YoutubeAvailabilityStatus::NoCaptions
    )
}

fn video_metadata_from_playlist_item(item: &YoutubePlaylistItemMetadata) -> YoutubeVideoMetadata {
    YoutubeVideoMetadata {
        video_id: item.video_id.clone(),
        canonical_url: item
            .url
            .clone()
            .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", item.video_id)),
        title: item.title_snapshot.clone(),
        channel_title: None,
        channel_id: None,
        channel_handle: None,
        channel_url: None,
        author_display: None,
        published_at: None,
        duration_seconds: None,
        description: None,
        thumbnail_url: item.thumbnail_url.clone(),
        tags: Vec::new(),
        chapters: Vec::new(),
        view_count: None,
        like_count: None,
        comment_count: None,
        category: None,
        video_form: YoutubeVideoForm::Regular,
        availability_status: item.availability_status.clone(),
        raw_metadata_json: item.raw_metadata_json.clone(),
    }
}

fn encode_playlist_item_metadata(item: &YoutubePlaylistItemMetadata) -> AppResult<Vec<u8>> {
    let json = to_vec(item).map_err(|e| AppError::internal(e.to_string()))?;
    compress_json_bytes(&json).map_err(AppError::internal)
}

fn availability_status_wire(status: &YoutubeAvailabilityStatus) -> &'static str {
    match status {
        YoutubeAvailabilityStatus::Available => "available",
        YoutubeAvailabilityStatus::Upcoming => "upcoming",
        YoutubeAvailabilityStatus::LiveNow => "live_now",
        YoutubeAvailabilityStatus::LiveEndedTranscriptPending => "live_ended_transcript_pending",
        YoutubeAvailabilityStatus::NoCaptions => "no_captions",
        YoutubeAvailabilityStatus::PrivateOrAuthRequired => "private_or_auth_required",
        YoutubeAvailabilityStatus::MembersOnly => "members_only",
        YoutubeAvailabilityStatus::AgeRestricted => "age_restricted",
        YoutubeAvailabilityStatus::GeoBlocked => "geo_blocked",
        YoutubeAvailabilityStatus::Deleted => "deleted",
        YoutubeAvailabilityStatus::RemovedFromPlaylist => "removed_from_playlist",
        YoutubeAvailabilityStatus::UnavailableUnknown => "unavailable_unknown",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        upsert_playlist_items, upsert_playlist_items_with_options, UpsertPlaylistItemsOptions,
    };
    use crate::sources::{upsert_youtube_playlist_source, upsert_youtube_video_source};
    use crate::youtube::dto::{
        YoutubeAvailabilityStatus, YoutubePlaylistItemMetadata, YoutubePlaylistMetadata,
        YoutubeVideoForm, YoutubeVideoMetadata,
    };

    async fn youtube_pool() -> sqlx::SqlitePool {
        let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;
        crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
        crate::sources::test_support::create_analysis_documents_table(&pool).await;
        sqlx::query(
            r#"
            CREATE UNIQUE INDEX idx_sources_unique_youtube_video
            ON sources(source_type, source_subtype, external_id)
            WHERE source_type = 'youtube' AND source_subtype = 'video'
            "#,
        )
        .execute(&pool)
        .await
        .expect("create video source unique index");
        sqlx::query(
            r#"
            CREATE UNIQUE INDEX idx_sources_unique_youtube_playlist
            ON sources(source_type, source_subtype, external_id)
            WHERE source_type = 'youtube' AND source_subtype = 'playlist'
            "#,
        )
        .execute(&pool)
        .await
        .expect("create playlist source unique index");
        sqlx::query(
            r#"
            CREATE TABLE youtube_playlist_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                playlist_source_id INTEGER NOT NULL,
                video_source_id INTEGER,
                video_id TEXT NOT NULL,
                position INTEGER,
                title_snapshot TEXT,
                url TEXT,
                thumbnail_url TEXT,
                availability_status TEXT NOT NULL,
                is_removed_from_playlist INTEGER NOT NULL DEFAULT 0,
                last_seen_at INTEGER,
                metadata_zstd BLOB,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(playlist_source_id, video_id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create youtube_playlist_items");
        pool
    }

    fn video_metadata(video_id: &str, title: &str) -> YoutubeVideoMetadata {
        YoutubeVideoMetadata {
            video_id: video_id.to_string(),
            canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
            title: Some(title.to_string()),
            channel_title: Some("Channel".to_string()),
            channel_id: Some("UC1".to_string()),
            channel_handle: Some("@channel".to_string()),
            channel_url: Some("https://www.youtube.com/@channel".to_string()),
            author_display: Some("Channel".to_string()),
            published_at: Some("2026-05-01".to_string()),
            duration_seconds: Some(120),
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
            raw_metadata_json: json!({ "id": video_id, "title": title }),
        }
    }

    fn playlist_metadata(items: Vec<YoutubePlaylistItemMetadata>) -> YoutubePlaylistMetadata {
        YoutubePlaylistMetadata {
            playlist_id: "PLabc123".to_string(),
            canonical_url: "https://www.youtube.com/playlist?list=PLabc123".to_string(),
            title: Some("Playlist".to_string()),
            channel_title: Some("Channel".to_string()),
            channel_id: Some("UC1".to_string()),
            channel_handle: Some("@channel".to_string()),
            channel_url: Some("https://www.youtube.com/@channel".to_string()),
            thumbnail_url: None,
            video_count: Some(items.len() as i64),
            items,
            availability_status: YoutubeAvailabilityStatus::Available,
            raw_metadata_json: json!({ "id": "PLabc123", "title": "Playlist" }),
        }
    }

    fn playlist_item(
        video_id: &str,
        status: YoutubeAvailabilityStatus,
    ) -> YoutubePlaylistItemMetadata {
        YoutubePlaylistItemMetadata {
            video_id: video_id.to_string(),
            position: Some(1),
            title_snapshot: Some(format!("Video {video_id}")),
            url: Some(format!("https://www.youtube.com/watch?v={video_id}")),
            thumbnail_url: Some(format!(
                "https://img.youtube.com/vi/{video_id}/hqdefault.jpg"
            )),
            availability_status: status,
            raw_metadata_json: json!({ "id": video_id }),
        }
    }

    #[tokio::test]
    async fn upsert_playlist_items_reuses_existing_video_source_and_keeps_unavailable_null() {
        let pool = youtube_pool().await;
        let mut tx = pool.begin().await.expect("begin transaction");
        let existing_video_id =
            upsert_youtube_video_source(&mut tx, &video_metadata("video01", "Existing"))
                .await
                .expect("upsert existing video");
        upsert_youtube_video_source(&mut tx, &video_metadata("private01", "Private existing"))
            .await
            .expect("upsert existing private video source");
        let playlist_id = upsert_youtube_playlist_source(&mut tx, &playlist_metadata(Vec::new()))
            .await
            .expect("upsert playlist");

        upsert_playlist_items(
            &mut tx,
            playlist_id,
            &playlist_metadata(vec![
                playlist_item("video01", YoutubeAvailabilityStatus::Available),
                playlist_item(
                    "private01",
                    YoutubeAvailabilityStatus::PrivateOrAuthRequired,
                ),
            ]),
        )
        .await
        .expect("upsert playlist items");
        tx.commit().await.expect("commit transaction");

        let rows: Vec<(String, Option<i64>, i64)> = sqlx::query_as(
            "SELECT video_id, video_source_id, is_removed_from_playlist FROM youtube_playlist_items ORDER BY video_id",
        )
        .fetch_all(&pool)
        .await
        .expect("load playlist items");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], ("private01".to_string(), None, 0));
        assert_eq!(rows[1], ("video01".to_string(), Some(existing_video_id), 0));
    }

    #[tokio::test]
    async fn playlist_item_video_source_upsert_writes_typed_video_metadata_not_source_blob() {
        let pool = youtube_pool().await;
        let mut tx = pool.begin().await.expect("begin tx");
        let playlist_source_id = upsert_youtube_playlist_source(
            &mut tx,
            &playlist_metadata(vec![playlist_item(
                "video01",
                YoutubeAvailabilityStatus::Available,
            )]),
        )
        .await
        .expect("upsert playlist source");

        upsert_playlist_items(
            &mut tx,
            playlist_source_id,
            &playlist_metadata(vec![playlist_item(
                "video01",
                YoutubeAvailabilityStatus::Available,
            )]),
        )
        .await
        .expect("upsert playlist items");
        tx.commit().await.expect("commit");

        let row: (Option<Vec<u8>>, String) = sqlx::query_as(
            r#"
            SELECT sources.metadata_zstd, youtube_video_sources.video_id
            FROM youtube_playlist_items
            JOIN sources ON sources.id = youtube_playlist_items.video_source_id
            JOIN youtube_video_sources ON youtube_video_sources.source_id = sources.id
            WHERE youtube_playlist_items.video_id = 'video01'
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("load linked typed video");

        assert_eq!(row.0, None);
        assert_eq!(row.1, "video01");
    }

    #[tokio::test]
    async fn upsert_playlist_items_can_skip_video_source_materialization() {
        let pool = youtube_pool().await;
        let mut tx = pool.begin().await.expect("begin tx");
        let playlist_source_id = upsert_youtube_playlist_source(
            &mut tx,
            &playlist_metadata(vec![playlist_item(
                "video01",
                YoutubeAvailabilityStatus::Available,
            )]),
        )
        .await
        .expect("upsert playlist source");

        upsert_playlist_items_with_options(
            &mut tx,
            playlist_source_id,
            &playlist_metadata(vec![playlist_item(
                "video01",
                YoutubeAvailabilityStatus::Available,
            )]),
            UpsertPlaylistItemsOptions {
                materialize_video_sources: false,
            },
        )
        .await
        .expect("upsert playlist items");
        tx.commit().await.expect("commit");

        let item_video_source_id: Option<i64> = sqlx::query_scalar(
            "SELECT video_source_id FROM youtube_playlist_items WHERE video_id = 'video01'",
        )
        .fetch_one(&pool)
        .await
        .expect("load playlist item");
        let video_source_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sources WHERE source_type = 'youtube' AND source_subtype = 'video'",
        )
        .fetch_one(&pool)
        .await
        .expect("count video sources");

        assert_eq!(item_video_source_id, None);
        assert_eq!(video_source_count, 0);
    }

    #[tokio::test]
    async fn upsert_playlist_items_without_materialization_reuses_existing_video_source() {
        let pool = youtube_pool().await;
        let mut tx = pool.begin().await.expect("begin tx");
        let existing_video_id =
            upsert_youtube_video_source(&mut tx, &video_metadata("video01", "Existing"))
                .await
                .expect("upsert existing video");
        let playlist_source_id = upsert_youtube_playlist_source(
            &mut tx,
            &playlist_metadata(vec![playlist_item(
                "video01",
                YoutubeAvailabilityStatus::Available,
            )]),
        )
        .await
        .expect("upsert playlist source");

        upsert_playlist_items_with_options(
            &mut tx,
            playlist_source_id,
            &playlist_metadata(vec![playlist_item(
                "video01",
                YoutubeAvailabilityStatus::Available,
            )]),
            UpsertPlaylistItemsOptions {
                materialize_video_sources: false,
            },
        )
        .await
        .expect("upsert playlist items");
        tx.commit().await.expect("commit");

        let item_video_source_id: Option<i64> = sqlx::query_scalar(
            "SELECT video_source_id FROM youtube_playlist_items WHERE video_id = 'video01'",
        )
        .fetch_one(&pool)
        .await
        .expect("load playlist item");
        let video_source_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sources WHERE source_type = 'youtube' AND source_subtype = 'video'",
        )
        .fetch_one(&pool)
        .await
        .expect("count video sources");

        assert_eq!(item_video_source_id, Some(existing_video_id));
        assert_eq!(video_source_count, 1);
    }

    #[tokio::test]
    async fn upsert_playlist_items_marks_missing_rows_removed() {
        let pool = youtube_pool().await;
        let mut tx = pool.begin().await.expect("begin transaction");
        let playlist_id = upsert_youtube_playlist_source(
            &mut tx,
            &playlist_metadata(vec![
                playlist_item("video01", YoutubeAvailabilityStatus::Available),
                playlist_item("video02", YoutubeAvailabilityStatus::Available),
            ]),
        )
        .await
        .expect("upsert playlist");

        upsert_playlist_items(
            &mut tx,
            playlist_id,
            &playlist_metadata(vec![
                playlist_item("video01", YoutubeAvailabilityStatus::Available),
                playlist_item("video02", YoutubeAvailabilityStatus::Available),
            ]),
        )
        .await
        .expect("upsert initial playlist items");
        upsert_playlist_items(
            &mut tx,
            playlist_id,
            &playlist_metadata(vec![playlist_item(
                "video02",
                YoutubeAvailabilityStatus::Available,
            )]),
        )
        .await
        .expect("upsert updated playlist items");
        tx.commit().await.expect("commit transaction");

        let removed: i64 = sqlx::query_scalar(
            "SELECT is_removed_from_playlist FROM youtube_playlist_items WHERE video_id = 'video01'",
        )
        .fetch_one(&pool)
        .await
        .expect("load removed flag");

        assert_eq!(removed, 1);
    }
}
