mod models;

use std::collections::{BTreeMap, HashMap, HashSet};

use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::sources::{require_source_identity_ready, SourceIdentityRepairState};
use crate::youtube::jobs::{SourceJobRecord, SourceJobState, SourceJobStatus};

use models::LibrarySourceRow;
pub(crate) use models::{
    LibraryCatalogCapabilities, LibraryCatalogDisabledReasons, LibraryCatalogFilterCount,
    LibraryCatalogRecord, LibraryCatalogResponse, LibraryCatalogStatus,
};
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

#[tauri::command]
pub(crate) async fn list_library_catalog(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    source_jobs: tauri::State<'_, SourceJobState>,
) -> AppResult<LibraryCatalogResponse> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    query_library_catalog(&pool, source_jobs.inner()).await
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

pub(crate) async fn query_library_catalog(
    pool: &sqlx::SqlitePool,
    source_jobs: &SourceJobState,
) -> AppResult<LibraryCatalogResponse> {
    let sources = query_library_sources(pool).await?;
    let source_ids = sources
        .iter()
        .map(|source| source.source_id)
        .collect::<Vec<_>>();
    let jobs = source_jobs.catalog_jobs_for_sources(&source_ids).await;
    let latest_jobs = latest_catalog_jobs_by_source(&source_ids, jobs);
    let filter_counts = build_catalog_filter_counts(&sources);
    let sources = sources
        .into_iter()
        .map(|source| {
            let latest_job = latest_jobs.get(&source.source_id).cloned();
            catalog_record_for_source(source, latest_job)
        })
        .collect();

    Ok(LibraryCatalogResponse {
        sources,
        filter_counts,
    })
}

const LIBRARY_SOURCES_SQL: &str = r#"
    WITH item_counts AS (
        SELECT source_id, COUNT(content_zstd) AS item_count
        FROM items
        GROUP BY source_id
    ),
    project_counts AS (
        SELECT source_id, COUNT(DISTINCT project_id) AS project_count
        FROM project_sources
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

pub(crate) const YOUTUBE_CHANNEL_DISABLED_REASON: &str =
    "YouTube channel sources are not supported by the current backend.";
const SOURCE_EDIT_DISABLED_REASON: &str = "Source editing is not available yet.";
const SOURCE_SYNCING_DISABLED_REASON: &str = "Source is syncing.";

fn latest_catalog_jobs_by_source(
    source_ids: &[i64],
    jobs: Vec<SourceJobRecord>,
) -> HashMap<i64, SourceJobRecord> {
    let source_ids = source_ids.iter().copied().collect::<HashSet<_>>();
    let mut latest = HashMap::<i64, SourceJobRecord>::new();

    for job in jobs {
        let mut matched_source_ids = Vec::new();
        if source_ids.contains(&job.source_id) {
            matched_source_ids.push(job.source_id);
        }
        if let Some(related_source_id) = job.related_source_id {
            if source_ids.contains(&related_source_id)
                && !matched_source_ids.contains(&related_source_id)
            {
                matched_source_ids.push(related_source_id);
            }
        }

        for source_id in matched_source_ids {
            let replace = latest.get(&source_id).is_none_or(|current| {
                job.started_at > current.started_at
                    || (job.started_at == current.started_at && job.job_id > current.job_id)
            });
            if replace {
                latest.insert(source_id, job.clone());
            }
        }
    }

    latest
}

fn catalog_record_for_source(
    source: LibrarySourceRecord,
    latest_job: Option<SourceJobRecord>,
) -> LibraryCatalogRecord {
    let (status, status_detail) = catalog_status_for_source(&source, latest_job.as_ref());
    let disabled_reasons = catalog_disabled_reasons(&source, status);
    let capabilities = LibraryCatalogCapabilities {
        can_refresh_source: disabled_reasons.refresh_source.is_none(),
        can_delete: disabled_reasons.delete.is_none(),
        can_edit: disabled_reasons.edit.is_none(),
        can_connect_to_project: disabled_reasons.connect_to_project.is_none(),
    };

    LibraryCatalogRecord {
        source,
        latest_job,
        status,
        status_detail,
        capabilities,
        disabled_reasons,
    }
}

fn catalog_status_for_source(
    source: &LibrarySourceRecord,
    latest_job: Option<&SourceJobRecord>,
) -> (LibraryCatalogStatus, Option<String>) {
    if is_unsupported_youtube_channel(source) {
        return (
            LibraryCatalogStatus::Unavailable,
            Some(YOUTUBE_CHANNEL_DISABLED_REASON.to_string()),
        );
    }

    if let Some(job) = latest_job {
        return match job.status {
            SourceJobStatus::Queued | SourceJobStatus::Running => (
                LibraryCatalogStatus::Syncing,
                job.message
                    .clone()
                    .or_else(|| Some(SOURCE_SYNCING_DISABLED_REASON.to_string())),
            ),
            SourceJobStatus::Failed => (
                LibraryCatalogStatus::Error,
                job.error.clone().or_else(|| job.message.clone()),
            ),
            _ => (LibraryCatalogStatus::Active, None),
        };
    }

    (LibraryCatalogStatus::Active, None)
}

fn catalog_disabled_reasons(
    source: &LibrarySourceRecord,
    status: LibraryCatalogStatus,
) -> LibraryCatalogDisabledReasons {
    let unsupported_reason =
        is_unsupported_youtube_channel(source).then(|| YOUTUBE_CHANNEL_DISABLED_REASON.to_string());
    let refresh_source = if unsupported_reason.is_some() {
        unsupported_reason.clone()
    } else if status == LibraryCatalogStatus::Syncing {
        Some(SOURCE_SYNCING_DISABLED_REASON.to_string())
    } else {
        None
    };
    let delete = (source.project_count > 0).then(|| {
        format!(
            "Source {} is used by {} project(s). Remove it from projects first.",
            source.source_id, source.project_count
        )
    });
    let connect_to_project = unsupported_reason;

    LibraryCatalogDisabledReasons {
        refresh_source,
        delete,
        edit: Some(SOURCE_EDIT_DISABLED_REASON.to_string()),
        connect_to_project,
    }
}

fn build_catalog_filter_counts(sources: &[LibrarySourceRecord]) -> Vec<LibraryCatalogFilterCount> {
    let mut counts = BTreeMap::<(String, Option<String>), i64>::new();
    for source in sources {
        *counts
            .entry((source.provider.clone(), source.source_subtype.clone()))
            .or_insert(0) += 1;
    }

    let stable_rows = [
        ("youtube", Some("video"), false, None),
        ("youtube", Some("playlist"), false, None),
        (
            "youtube",
            Some("channel"),
            true,
            Some(YOUTUBE_CHANNEL_DISABLED_REASON),
        ),
        ("telegram", Some("channel"), false, None),
        ("telegram", Some("supergroup"), false, None),
        ("telegram", Some("group"), false, None),
    ];

    let mut rows = stable_rows
        .iter()
        .map(|(provider, subtype, disabled, disabled_reason)| {
            let subtype = subtype.map(str::to_string);
            LibraryCatalogFilterCount {
                provider: (*provider).to_string(),
                source_subtype: subtype.clone(),
                count: counts
                    .get(&((*provider).to_string(), subtype))
                    .copied()
                    .unwrap_or(0),
                disabled: *disabled,
                disabled_reason: disabled_reason.map(|reason| (*reason).to_string()),
            }
        })
        .collect::<Vec<_>>();

    for ((provider, subtype), count) in counts {
        if stable_rows
            .iter()
            .any(|(stable_provider, stable_subtype, _, _)| {
                *stable_provider == provider.as_str()
                    && stable_subtype.map(str::to_string) == subtype
            })
        {
            continue;
        }
        rows.push(LibraryCatalogFilterCount {
            provider,
            source_subtype: subtype,
            count,
            disabled: false,
            disabled_reason: None,
        });
    }

    rows
}

fn is_unsupported_youtube_channel(source: &LibrarySourceRecord) -> bool {
    source.provider == "youtube" && source.source_subtype.as_deref() == Some("channel")
}

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
    use crate::youtube::jobs::{
        SourceJobState, SourceJobStatus, SourceJobType, YoutubeSyncOptions,
    };

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
            CREATE TABLE projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
            r#"
            CREATE TABLE project_sources (
                project_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                added_at INTEGER NOT NULL
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
        sqlx::query("INSERT INTO projects (id, name, created_at, updated_at) VALUES (10, 'Project A', 1, 1), (11, 'Project B', 1, 1)")
            .execute(&pool)
            .await
            .expect("insert projects");
        sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (10, 1, 1), (11, 1, 1), (10, 3, 1)")
            .execute(&pool)
            .await
            .expect("insert project sources");
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

    #[tokio::test]
    async fn list_library_catalog_returns_status_capabilities_and_filter_counts() {
        let pool = memory_pool().await;
        insert_source(
            &pool,
            1,
            "youtube",
            Some("video"),
            None,
            "vid-1",
            "Video fallback",
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
            "Playlist fallback",
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
        insert_source(
            &pool,
            4,
            "youtube",
            Some("channel"),
            None,
            "chan-1",
            "Unsupported channel",
            103,
            None,
        )
        .await;

        sqlx::query("INSERT INTO items (id, source_id, content_zstd) VALUES (1, 1, X'01'), (2, 2, X'02'), (3, 3, X'03')")
            .execute(&pool)
            .await
            .expect("insert items");
        sqlx::query("INSERT INTO projects (id, name, created_at, updated_at) VALUES (10, 'Project A', 1, 1), (11, 'Project B', 1, 1)")
            .execute(&pool)
            .await
            .expect("insert projects");
        sqlx::query("INSERT INTO project_sources (project_id, source_id, added_at) VALUES (10, 1, 1), (11, 1, 1)")
            .execute(&pool)
            .await
            .expect("insert project sources");
        sqlx::query(
            r#"
            INSERT INTO youtube_video_sources (
                source_id, video_id, canonical_url, title, channel_title,
                duration_seconds, video_form, availability_status
            )
            VALUES (1, 'vid-1', 'https://youtu.be/vid-1', 'Video title', 'Channel A', 321, 'short', 'available')
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

        let source_jobs = SourceJobState::new();
        let options = YoutubeSyncOptions {
            metadata: true,
            transcripts: false,
            comments: false,
        };
        let failed = source_jobs
            .create_job(
                1,
                SourceJobType::YoutubeVideoTranscriptSync,
                None,
                options.clone(),
            )
            .await
            .expect("create failed source job");
        source_jobs
            .finish_job(&failed.job_id, |job| {
                job.status = SourceJobStatus::Failed;
                job.started_at = 20;
                job.error = Some("Transcript quota exceeded".to_string());
            })
            .await
            .expect("finish failed source job");
        let running = source_jobs
            .create_job(2, SourceJobType::YoutubePlaylistFullSync, None, options)
            .await
            .expect("create running source job");
        source_jobs
            .update_job(&running.job_id, |job| {
                job.status = SourceJobStatus::Running;
                job.started_at = 30;
                job.message = Some("Syncing playlist.".to_string());
            })
            .await
            .expect("update running source job");

        let catalog = query_library_catalog(&pool, &source_jobs)
            .await
            .expect("query library catalog");

        let video = catalog
            .sources
            .iter()
            .find(|record| record.source.source_id == 1)
            .expect("video catalog record");
        assert_eq!(video.status, LibraryCatalogStatus::Error);
        assert_eq!(
            video.status_detail.as_deref(),
            Some("Transcript quota exceeded")
        );
        assert_eq!(
            video.latest_job.as_ref().map(|job| job.job_id.as_str()),
            Some(failed.job_id.as_str())
        );
        assert!(!video.capabilities.can_delete);
        assert_eq!(
            video.disabled_reasons.delete.as_deref(),
            Some("Source 1 is used by 2 project(s). Remove it from projects first.")
        );
        assert!(!video.capabilities.can_edit);
        assert_eq!(
            video.disabled_reasons.edit.as_deref(),
            Some("Source editing is not available yet.")
        );

        let playlist = catalog
            .sources
            .iter()
            .find(|record| record.source.source_id == 2)
            .expect("playlist catalog record");
        assert_eq!(playlist.status, LibraryCatalogStatus::Syncing);
        assert_eq!(playlist.status_detail.as_deref(), Some("Syncing playlist."));
        assert!(!playlist.capabilities.can_refresh_source);
        assert_eq!(
            playlist.disabled_reasons.refresh_source.as_deref(),
            Some("Source is syncing.")
        );

        let telegram = catalog
            .sources
            .iter()
            .find(|record| record.source.source_id == 3)
            .expect("telegram catalog record");
        assert_eq!(telegram.status, LibraryCatalogStatus::Active);
        assert!(telegram.latest_job.is_none());
        assert!(telegram.capabilities.can_connect_to_project);

        let channel = catalog
            .sources
            .iter()
            .find(|record| record.source.source_id == 4)
            .expect("youtube channel catalog record");
        assert_eq!(channel.status, LibraryCatalogStatus::Unavailable);
        assert_eq!(
            channel.status_detail.as_deref(),
            Some(YOUTUBE_CHANNEL_DISABLED_REASON)
        );
        assert!(!channel.capabilities.can_refresh_source);
        assert!(!channel.capabilities.can_connect_to_project);
        assert_eq!(
            channel.disabled_reasons.connect_to_project.as_deref(),
            Some(YOUTUBE_CHANNEL_DISABLED_REASON)
        );

        let youtube_video_count = catalog
            .filter_counts
            .iter()
            .find(|count| {
                count.provider == "youtube" && count.source_subtype.as_deref() == Some("video")
            })
            .expect("youtube video count");
        assert_eq!(youtube_video_count.count, 1);
        assert!(!youtube_video_count.disabled);

        let youtube_channel_count = catalog
            .filter_counts
            .iter()
            .find(|count| {
                count.provider == "youtube" && count.source_subtype.as_deref() == Some("channel")
            })
            .expect("youtube channel count");
        assert_eq!(youtube_channel_count.count, 1);
        assert!(youtube_channel_count.disabled);
        assert_eq!(
            youtube_channel_count.disabled_reason.as_deref(),
            Some(YOUTUBE_CHANNEL_DISABLED_REASON)
        );
    }
}
