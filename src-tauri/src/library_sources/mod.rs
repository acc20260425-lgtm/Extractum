mod models;

use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::sources::{require_source_identity_ready, SourceIdentityRepairState};

use models::LibrarySourceRow;
pub use models::{LibrarySourceRecord, LibraryTelegramSourceDetails, LibraryYoutubeSourceDetails};

#[tauri::command]
pub async fn list_library_sources(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
) -> AppResult<Vec<LibrarySourceRecord>> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    query_library_sources(&pool).await
}

pub(crate) async fn query_library_sources(
    pool: &sqlx::SqlitePool,
) -> AppResult<Vec<LibrarySourceRecord>> {
    let rows: Vec<LibrarySourceRow> = sqlx::query_as(LIBRARY_SOURCES_SQL)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    Ok(rows.into_iter().map(map_library_source_row).collect())
}

const LIBRARY_SOURCES_SQL: &str = r#"
    WITH item_counts AS (
        SELECT source_id, COUNT(content_zstd) AS item_count
        FROM items
        GROUP BY source_id
    ),
    project_counts AS (
        SELECT source_id, COUNT(DISTINCT group_id) AS project_count
        FROM analysis_source_group_members
        GROUP BY source_id
    )
    SELECT
        s.id AS source_id,
        s.source_type AS provider,
        s.source_subtype,
        s.account_id,
        s.external_id,
        s.title AS source_title,
        s.created_at,
        s.last_synced_at,
        COALESCE(item_counts.item_count, 0) AS item_count,
        COALESCE(project_counts.project_count, 0) AS project_count,
        yvs.title AS video_title,
        yvs.canonical_url AS video_canonical_url,
        yvs.channel_title AS video_channel_title,
        yvs.duration_seconds,
        yvs.video_form,
        yvs.availability_status AS video_availability_status,
        yps.title AS playlist_title,
        yps.canonical_url AS playlist_canonical_url,
        yps.channel_title AS playlist_channel_title,
        yps.video_count AS playlist_video_count,
        yps.availability_status AS playlist_availability_status
    FROM sources s
    LEFT JOIN item_counts ON item_counts.source_id = s.id
    LEFT JOIN project_counts ON project_counts.source_id = s.id
    LEFT JOIN youtube_video_sources yvs
        ON yvs.source_id = s.id
        AND s.source_type = 'youtube'
        AND s.source_subtype = 'video'
    LEFT JOIN youtube_playlist_sources yps
        ON yps.source_id = s.id
        AND s.source_type = 'youtube'
        AND s.source_subtype = 'playlist'
    ORDER BY s.created_at DESC, s.id DESC
"#;

fn map_library_source_row(row: LibrarySourceRow) -> LibrarySourceRecord {
    let youtube = match (row.provider.as_str(), row.source_subtype.as_deref()) {
        ("youtube", Some("video"))
            if row.video_title.is_some()
                || row.video_canonical_url.is_some()
                || row.video_channel_title.is_some()
                || row.duration_seconds.is_some()
                || row.video_form.is_some()
                || row.video_availability_status.is_some() =>
        {
            Some(LibraryYoutubeSourceDetails {
                video_form: row.video_form.clone(),
                duration_seconds: row.duration_seconds,
                playlist_video_count: None,
                channel_title: row.video_channel_title.clone(),
                availability_status: row.video_availability_status.clone(),
            })
        }
        ("youtube", Some("playlist"))
            if row.playlist_title.is_some()
                || row.playlist_canonical_url.is_some()
                || row.playlist_channel_title.is_some()
                || row.playlist_video_count.is_some()
                || row.playlist_availability_status.is_some() =>
        {
            Some(LibraryYoutubeSourceDetails {
                video_form: None,
                duration_seconds: None,
                playlist_video_count: row.playlist_video_count,
                channel_title: row.playlist_channel_title.clone(),
                availability_status: row.playlist_availability_status.clone(),
            })
        }
        _ => None,
    };

    let telegram = if row.provider == "telegram" {
        Some(LibraryTelegramSourceDetails {
            account_id: row.account_id,
        })
    } else {
        None
    };

    let title = match row.source_subtype.as_deref() {
        Some("video") => row.video_title.clone().or_else(|| row.source_title.clone()),
        Some("playlist") => row
            .playlist_title
            .clone()
            .or_else(|| row.source_title.clone()),
        _ => row.source_title.clone(),
    };
    let subtitle = match row.source_subtype.as_deref() {
        Some("video") => row.video_channel_title.clone(),
        Some("playlist") => row.playlist_channel_title.clone(),
        _ => row
            .account_id
            .map(|account_id| format!("Account #{account_id}")),
    };
    let canonical_url = match row.source_subtype.as_deref() {
        Some("video") => row.video_canonical_url.clone(),
        Some("playlist") => row.playlist_canonical_url.clone(),
        _ => None,
    };

    LibrarySourceRecord {
        source_id: row.source_id,
        provider: row.provider,
        source_subtype: row.source_subtype,
        account_id: row.account_id,
        external_id: row.external_id,
        title,
        subtitle,
        canonical_url,
        created_at: row.created_at,
        last_synced_at: row.last_synced_at,
        item_count: row.item_count,
        project_count: row.project_count,
        youtube,
        telegram,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn memory_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        create_schema(&pool).await;
        pool
    }

    async fn create_schema(pool: &sqlx::SqlitePool) {
        for statement in [
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY,
                source_type TEXT NOT NULL,
                source_subtype TEXT,
                account_id INTEGER,
                external_id TEXT,
                title TEXT,
                last_synced_at INTEGER,
                created_at INTEGER NOT NULL
            )
            "#,
            r#"
            CREATE TABLE items (
                id INTEGER PRIMARY KEY,
                source_id INTEGER NOT NULL,
                content_zstd BLOB
            )
            "#,
            r#"
            CREATE TABLE analysis_source_groups (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
            r#"
            CREATE TABLE analysis_source_group_members (
                group_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
            r#"
            CREATE TABLE youtube_video_sources (
                source_id INTEGER PRIMARY KEY,
                video_id TEXT NOT NULL,
                canonical_url TEXT,
                title TEXT,
                channel_title TEXT,
                duration_seconds INTEGER,
                video_form TEXT,
                availability_status TEXT
            )
            "#,
            r#"
            CREATE TABLE youtube_playlist_sources (
                source_id INTEGER PRIMARY KEY,
                playlist_id TEXT NOT NULL,
                canonical_url TEXT,
                title TEXT,
                channel_title TEXT,
                video_count INTEGER,
                availability_status TEXT
            )
            "#,
        ] {
            sqlx::query(statement)
                .execute(pool)
                .await
                .expect("create library source test schema");
        }
    }

    async fn insert_source(
        pool: &sqlx::SqlitePool,
        id: i64,
        provider: &str,
        subtype: Option<&str>,
        account_id: Option<i64>,
        external_id: &str,
        title: &str,
        created_at: i64,
        last_synced_at: Option<i64>,
    ) {
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id,
                title, created_at, last_synced_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(provider)
        .bind(subtype)
        .bind(account_id)
        .bind(external_id)
        .bind(title)
        .bind(created_at)
        .bind(last_synced_at)
        .execute(pool)
        .await
        .expect("insert source");
    }

    #[tokio::test]
    async fn list_library_sources_returns_youtube_and_telegram_metadata() {
        let pool = memory_pool().await;
        insert_source(
            &pool,
            1,
            "youtube",
            Some("video"),
            None,
            "vid-1",
            "Fallback video",
            100,
            Some(200),
        )
        .await;
        insert_source(
            &pool,
            2,
            "youtube",
            Some("playlist"),
            None,
            "pl-1",
            "Fallback playlist",
            101,
            None,
        )
        .await;
        insert_source(
            &pool,
            3,
            "telegram",
            Some("supergroup"),
            Some(77),
            "-1007",
            "Drone Radar",
            102,
            Some(202),
        )
        .await;

        sqlx::query("INSERT INTO items (id, source_id, content_zstd) VALUES (1, 1, X'01'), (2, 1, X'02'), (3, 3, X'03')")
            .execute(&pool)
            .await
            .expect("insert items");
        sqlx::query("INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at) VALUES (10, 'Project A', 'youtube', 1, 1), (11, 'Project B', 'youtube', 1, 1)")
            .execute(&pool)
            .await
            .expect("insert groups");
        sqlx::query("INSERT INTO analysis_source_group_members (group_id, source_id, created_at) VALUES (10, 1, 1), (11, 1, 1), (10, 3, 1)")
            .execute(&pool)
            .await
            .expect("insert members");
        sqlx::query(
            r#"
            INSERT INTO youtube_video_sources (
                source_id, video_id, canonical_url, title, channel_title,
                duration_seconds, video_form, availability_status
            )
            VALUES (1, 'vid-1', 'https://youtu.be/vid-1', 'Video title', NULL, 321, 'short', 'available')
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert video metadata");
        sqlx::query(
            r#"
            INSERT INTO youtube_playlist_sources (
                source_id, playlist_id, canonical_url, title, channel_title,
                video_count, availability_status
            )
            VALUES (2, 'pl-1', 'https://www.youtube.com/playlist?list=pl-1', 'Playlist title', 'Channel B', 44, 'available')
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert playlist metadata");

        let rows = query_library_sources(&pool)
            .await
            .expect("list library sources");

        assert_eq!(
            rows.iter().map(|row| row.source_id).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );

        let video = rows
            .iter()
            .find(|row| row.source_id == 1)
            .expect("video source");
        assert_eq!(video.source_subtype.as_deref(), Some("video"));
        assert_eq!(video.title.as_deref(), Some("Video title"));
        assert_eq!(video.subtitle, None);
        assert_eq!(
            video.canonical_url.as_deref(),
            Some("https://youtu.be/vid-1")
        );
        assert_eq!(video.item_count, 2);
        assert_eq!(video.project_count, 2);
        assert_eq!(
            video.youtube,
            Some(LibraryYoutubeSourceDetails {
                video_form: Some("short".to_string()),
                duration_seconds: Some(321),
                playlist_video_count: None,
                channel_title: None,
                availability_status: Some("available".to_string()),
            })
        );

        let playlist = rows
            .iter()
            .find(|row| row.source_id == 2)
            .expect("playlist source");
        assert_eq!(playlist.source_subtype.as_deref(), Some("playlist"));
        assert_eq!(playlist.title.as_deref(), Some("Playlist title"));
        assert_eq!(playlist.subtitle.as_deref(), Some("Channel B"));
        assert_eq!(playlist.item_count, 0);
        assert_eq!(playlist.project_count, 0);
        assert_eq!(
            playlist
                .youtube
                .as_ref()
                .and_then(|details| details.playlist_video_count),
            Some(44)
        );

        let telegram = rows
            .iter()
            .find(|row| row.source_id == 3)
            .expect("telegram source");
        assert_eq!(telegram.source_subtype.as_deref(), Some("supergroup"));
        assert_eq!(telegram.subtitle.as_deref(), Some("Account #77"));
        assert_eq!(
            telegram.telegram,
            Some(LibraryTelegramSourceDetails {
                account_id: Some(77)
            })
        );
    }

    #[tokio::test]
    async fn list_library_sources_keeps_sources_with_missing_provider_details() {
        let pool = memory_pool().await;
        insert_source(
            &pool,
            5,
            "youtube",
            Some("video"),
            None,
            "missing-video",
            "Stored title",
            500,
            None,
        )
        .await;

        let rows = query_library_sources(&pool)
            .await
            .expect("list library sources");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].source_id, 5);
        assert_eq!(rows[0].title.as_deref(), Some("Stored title"));
        assert_eq!(rows[0].canonical_url, None);
        assert_eq!(rows[0].youtube, None);
    }
}
