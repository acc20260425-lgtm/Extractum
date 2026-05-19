use std::collections::HashMap;

use serde::Serialize;
use sqlx::{QueryBuilder, Row};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::sources::{require_source_identity_ready, SourceIdentityRepairState};
use crate::sql_helpers::push_i64_bind_list;
use crate::time::ymd_to_unix_midnight;
use crate::youtube::source_metadata::{
    load_playlist_source_metadata_map, load_video_source_metadata_map,
    YoutubePlaylistSourceMetadata, YoutubeVideoSourceMetadata,
};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum YoutubeContentSyncState {
    NotSynced,
    Synced,
    Unavailable,
    Failed,
    Unknown,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeContentStatusDto {
    pub state: YoutubeContentSyncState,
    pub item_count: i64,
    pub segment_count: i64,
    pub last_synced_at: Option<i64>,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSourceSummaryDto {
    pub source_id: i64,
    pub source_subtype: String,
    pub title: Option<String>,
    pub channel_title: Option<String>,
    pub channel_handle: Option<String>,
    pub canonical_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i64>,
    pub published_at: Option<i64>,
    pub availability_status: Option<String>,
    pub video_count: Option<i64>,
    pub linked_video_count: Option<i64>,
    pub unavailable_count: Option<i64>,
    pub captions: YoutubeContentStatusDto,
    pub comments: YoutubeContentStatusDto,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubePlaylistMembershipDto {
    pub playlist_source_id: i64,
    pub playlist_title: Option<String>,
    pub position: Option<i64>,
    pub availability_status: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeVideoDetailDto {
    pub summary: YoutubeSourceSummaryDto,
    pub playlist_memberships: Vec<YoutubePlaylistMembershipDto>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubePlaylistItemDetailDto {
    pub position: Option<i64>,
    pub video_id: String,
    pub video_source_id: Option<i64>,
    pub title: Option<String>,
    pub canonical_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i64>,
    pub published_at: Option<i64>,
    pub availability_status: String,
    pub is_removed_from_playlist: bool,
    pub captions: YoutubeContentStatusDto,
    pub comments: YoutubeContentStatusDto,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubePlaylistDetailDto {
    pub summary: YoutubeSourceSummaryDto,
    pub items: Vec<YoutubePlaylistItemDetailDto>,
}

#[tauri::command]
pub async fn list_youtube_source_summaries(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    source_ids: Vec<i64>,
) -> AppResult<Vec<YoutubeSourceSummaryDto>> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    list_youtube_source_summaries_from_pool(&pool, source_ids).await
}

#[tauri::command]
pub async fn get_youtube_video_detail(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    source_id: i64,
) -> AppResult<YoutubeVideoDetailDto> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    get_youtube_video_detail_from_pool(&pool, source_id).await
}

#[tauri::command]
pub async fn get_youtube_playlist_detail(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    source_id: i64,
) -> AppResult<YoutubePlaylistDetailDto> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    get_youtube_playlist_detail_from_pool(&pool, source_id).await
}

pub(crate) async fn list_youtube_source_summaries_from_pool(
    pool: &sqlx::SqlitePool,
    source_ids: Vec<i64>,
) -> AppResult<Vec<YoutubeSourceSummaryDto>> {
    if source_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = load_source_rows(pool, &source_ids).await?;
    let source_ids_from_rows = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let video_metadata = load_video_source_metadata_map(pool, &source_ids_from_rows).await?;
    let playlist_metadata = load_playlist_source_metadata_map(pool, &source_ids_from_rows).await?;
    let source_caption_counts =
        load_direct_content_counts(pool, &source_ids_from_rows, "youtube_transcript").await?;
    let source_comment_counts =
        load_direct_content_counts(pool, &source_ids_from_rows, "youtube_comment").await?;
    let playlist_caption_counts =
        load_playlist_content_counts(pool, &source_ids_from_rows, "youtube_transcript").await?;
    let playlist_comment_counts =
        load_playlist_content_counts(pool, &source_ids_from_rows, "youtube_comment").await?;
    let playlist_counts = load_playlist_counts(pool, &source_ids_from_rows).await?;

    let mut summaries = HashMap::new();
    for row in rows {
        let typed_video = video_metadata.get(&row.id);
        let typed_playlist = playlist_metadata.get(&row.id);
        let playlist_stats = playlist_counts.get(&row.id);
        let captions_counts = if row.source_subtype.as_deref() == Some("playlist") {
            playlist_caption_counts.get(&row.id)
        } else {
            source_caption_counts.get(&row.id)
        };
        let comments_counts = if row.source_subtype.as_deref() == Some("playlist") {
            playlist_comment_counts.get(&row.id)
        } else {
            source_comment_counts.get(&row.id)
        };
        summaries.insert(
            row.id,
            summary_from_row(
                row,
                captions_counts.copied().unwrap_or_default(),
                comments_counts.copied().unwrap_or_default(),
                playlist_stats.copied(),
                typed_video,
                typed_playlist,
            ),
        );
    }

    Ok(source_ids
        .into_iter()
        .filter_map(|source_id| summaries.remove(&source_id))
        .collect())
}

pub(crate) async fn get_youtube_video_detail_from_pool(
    pool: &sqlx::SqlitePool,
    source_id: i64,
) -> AppResult<YoutubeVideoDetailDto> {
    let mut summaries = list_youtube_source_summaries_from_pool(pool, vec![source_id]).await?;
    let summary = summaries
        .pop()
        .ok_or_else(|| AppError::not_found(format!("YouTube source {source_id} not found")))?;
    if summary.source_subtype != "video" {
        return Err(AppError::validation(format!(
            "Source {source_id} is not a YouTube video source"
        )));
    }
    let typed = load_video_source_metadata_map(pool, &[source_id]).await?;
    if !typed.contains_key(&source_id) {
        return Err(AppError::validation(format!(
            "Source {source_id} has missing or invalid typed YouTube video metadata"
        )));
    }

    let playlist_memberships = sqlx::query_as::<_, PlaylistMembershipRow>(
        r#"
        SELECT
            youtube_playlist_items.playlist_source_id,
            sources.title AS playlist_title,
            youtube_playlist_items.position,
            youtube_playlist_items.availability_status
        FROM youtube_playlist_items
        LEFT JOIN sources ON sources.id = youtube_playlist_items.playlist_source_id
        WHERE youtube_playlist_items.video_source_id = ?
        ORDER BY youtube_playlist_items.position IS NULL,
                 youtube_playlist_items.position,
                 youtube_playlist_items.video_id
        "#,
    )
    .bind(source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?
    .into_iter()
    .map(|row| YoutubePlaylistMembershipDto {
        playlist_source_id: row.playlist_source_id,
        playlist_title: row.playlist_title,
        position: row.position,
        availability_status: row.availability_status,
    })
    .collect();

    Ok(YoutubeVideoDetailDto {
        summary,
        playlist_memberships,
    })
}

pub(crate) async fn get_youtube_playlist_detail_from_pool(
    pool: &sqlx::SqlitePool,
    source_id: i64,
) -> AppResult<YoutubePlaylistDetailDto> {
    let mut summaries = list_youtube_source_summaries_from_pool(pool, vec![source_id]).await?;
    let summary = summaries
        .pop()
        .ok_or_else(|| AppError::not_found(format!("YouTube source {source_id} not found")))?;
    if summary.source_subtype != "playlist" {
        return Err(AppError::validation(format!(
            "Source {source_id} is not a YouTube playlist source"
        )));
    }
    let typed = load_playlist_source_metadata_map(pool, &[source_id]).await?;
    if !typed.contains_key(&source_id) {
        return Err(AppError::validation(format!(
            "Source {source_id} has missing or invalid typed YouTube playlist metadata"
        )));
    }

    let rows = sqlx::query_as::<_, PlaylistItemRow>(
        r#"
        SELECT
            youtube_playlist_items.position,
            youtube_playlist_items.video_id,
            youtube_playlist_items.video_source_id,
            youtube_playlist_items.title_snapshot,
            youtube_playlist_items.url,
            youtube_playlist_items.thumbnail_url,
            youtube_playlist_items.availability_status,
            youtube_playlist_items.is_removed_from_playlist,
            sources.title AS video_source_title,
            yvs.title AS typed_video_title,
            yvs.canonical_url AS typed_video_canonical_url,
            yvs.thumbnail_url AS typed_video_thumbnail_url,
            yvs.duration_seconds AS typed_video_duration_seconds,
            yvs.published_at AS typed_video_published_at
        FROM youtube_playlist_items
        LEFT JOIN sources ON sources.id = youtube_playlist_items.video_source_id
        LEFT JOIN youtube_video_sources yvs ON yvs.source_id = youtube_playlist_items.video_source_id
        WHERE youtube_playlist_items.playlist_source_id = ?
        ORDER BY youtube_playlist_items.position IS NULL,
                 youtube_playlist_items.position,
                 youtube_playlist_items.video_id
        "#,
    )
    .bind(source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let linked_source_ids = rows
        .iter()
        .filter_map(|row| row.video_source_id)
        .collect::<Vec<_>>();
    let caption_counts =
        load_direct_content_counts(pool, &linked_source_ids, "youtube_transcript").await?;
    let comment_counts =
        load_direct_content_counts(pool, &linked_source_ids, "youtube_comment").await?;

    let items = rows
        .into_iter()
        .map(|row| {
            let title = row
                .typed_video_title
                .or(row.video_source_title)
                .or(row.title_snapshot);
            let canonical_url = row.typed_video_canonical_url.or(row.url);
            let thumbnail_url = row.typed_video_thumbnail_url.or(row.thumbnail_url);
            let duration_seconds = row.typed_video_duration_seconds;
            let published_at = row
                .typed_video_published_at
                .as_deref()
                .and_then(ymd_to_unix_midnight);
            let availability_status = row.availability_status;
            let captions = row
                .video_source_id
                .and_then(|source_id| caption_counts.get(&source_id).copied())
                .unwrap_or_default();
            let comments = row
                .video_source_id
                .and_then(|source_id| comment_counts.get(&source_id).copied())
                .unwrap_or_default();

            YoutubePlaylistItemDetailDto {
                position: row.position,
                video_id: row.video_id,
                video_source_id: row.video_source_id,
                title,
                canonical_url,
                thumbnail_url,
                duration_seconds,
                published_at,
                availability_status: availability_status.clone(),
                is_removed_from_playlist: row.is_removed_from_playlist,
                captions: caption_status(captions, Some(availability_status.as_str())),
                comments: comment_status(comments),
            }
        })
        .collect();

    Ok(YoutubePlaylistDetailDto { summary, items })
}

#[derive(Clone, Debug, sqlx::FromRow)]
struct SourceSummaryRow {
    id: i64,
    source_subtype: Option<String>,
    external_id: String,
    title: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
struct ContentCounts {
    item_count: i64,
    segment_count: i64,
    last_synced_at: Option<i64>,
}

#[derive(Clone, Copy, Debug, Default)]
struct PlaylistCounts {
    total_count: i64,
    linked_count: i64,
    unavailable_count: i64,
}

#[derive(Clone, Debug, sqlx::FromRow)]
struct PlaylistMembershipRow {
    playlist_source_id: i64,
    playlist_title: Option<String>,
    position: Option<i64>,
    availability_status: String,
}

#[derive(Clone, Debug, sqlx::FromRow)]
struct PlaylistItemRow {
    position: Option<i64>,
    video_id: String,
    video_source_id: Option<i64>,
    title_snapshot: Option<String>,
    url: Option<String>,
    thumbnail_url: Option<String>,
    availability_status: String,
    is_removed_from_playlist: bool,
    video_source_title: Option<String>,
    typed_video_title: Option<String>,
    typed_video_canonical_url: Option<String>,
    typed_video_thumbnail_url: Option<String>,
    typed_video_duration_seconds: Option<i64>,
    typed_video_published_at: Option<String>,
}

async fn load_source_rows(
    pool: &sqlx::SqlitePool,
    source_ids: &[i64],
) -> AppResult<Vec<SourceSummaryRow>> {
    let mut query = QueryBuilder::new(
        r#"
        SELECT id, source_subtype, external_id, title
        FROM sources
        WHERE source_type = 'youtube' AND id IN (
        "#,
    );
    push_i64_bind_list(&mut query, source_ids);
    query.push(")");

    query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)
}

async fn load_direct_content_counts(
    pool: &sqlx::SqlitePool,
    source_ids: &[i64],
    item_kind: &str,
) -> AppResult<HashMap<i64, ContentCounts>> {
    if source_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut query = QueryBuilder::new(
        r#"
        SELECT
            items.source_id AS source_id,
            COUNT(DISTINCT items.id) AS item_count,
            COUNT(youtube_transcript_segments.id) AS segment_count,
            MAX(items.ingested_at) AS last_synced_at
        FROM items
        LEFT JOIN youtube_transcript_segments
            ON youtube_transcript_segments.item_id = items.id
        WHERE items.item_kind =
        "#,
    );
    query.push_bind(item_kind);
    query.push(" AND items.source_id IN (");
    push_i64_bind_list(&mut query, source_ids);
    query.push(") GROUP BY items.source_id");

    let rows = query
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;
    rows_to_content_counts(rows)
}

async fn load_playlist_content_counts(
    pool: &sqlx::SqlitePool,
    source_ids: &[i64],
    item_kind: &str,
) -> AppResult<HashMap<i64, ContentCounts>> {
    if source_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut query = QueryBuilder::new(
        r#"
        SELECT
            youtube_playlist_items.playlist_source_id AS source_id,
            COUNT(DISTINCT items.id) AS item_count,
            COUNT(youtube_transcript_segments.id) AS segment_count,
            MAX(items.ingested_at) AS last_synced_at
        FROM youtube_playlist_items
        JOIN items
            ON items.source_id = youtube_playlist_items.video_source_id
           AND items.item_kind =
        "#,
    );
    query.push_bind(item_kind);
    query.push(
        r#"
        LEFT JOIN youtube_transcript_segments
            ON youtube_transcript_segments.item_id = items.id
        WHERE youtube_playlist_items.is_removed_from_playlist = 0
          AND youtube_playlist_items.video_source_id IS NOT NULL
          AND youtube_playlist_items.playlist_source_id IN (
        "#,
    );
    push_i64_bind_list(&mut query, source_ids);
    query.push(") GROUP BY youtube_playlist_items.playlist_source_id");

    let rows = query
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;
    rows_to_content_counts(rows)
}

async fn load_playlist_counts(
    pool: &sqlx::SqlitePool,
    source_ids: &[i64],
) -> AppResult<HashMap<i64, PlaylistCounts>> {
    if source_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut query = QueryBuilder::new(
        r#"
        SELECT
            playlist_source_id,
            COUNT(*) AS total_count,
            SUM(CASE WHEN video_source_id IS NOT NULL AND is_removed_from_playlist = 0 THEN 1 ELSE 0 END) AS linked_count,
            SUM(
                CASE
                    WHEN is_removed_from_playlist = 1
                      OR availability_status NOT IN (
                        'available',
                        'live_now',
                        'live_ended_transcript_pending'
                      )
                    THEN 1
                    ELSE 0
                END
            ) AS unavailable_count
        FROM youtube_playlist_items
        WHERE playlist_source_id IN (
        "#,
    );
    push_i64_bind_list(&mut query, source_ids);
    query.push(") GROUP BY playlist_source_id");

    let rows = query
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;
    let mut counts = HashMap::new();
    for row in rows {
        counts.insert(
            row.try_get::<i64, _>("playlist_source_id")
                .map_err(AppError::database)?,
            PlaylistCounts {
                total_count: row
                    .try_get::<i64, _>("total_count")
                    .map_err(AppError::database)?,
                linked_count: row
                    .try_get::<i64, _>("linked_count")
                    .map_err(AppError::database)?,
                unavailable_count: row
                    .try_get::<i64, _>("unavailable_count")
                    .map_err(AppError::database)?,
            },
        );
    }
    Ok(counts)
}

fn rows_to_content_counts(
    rows: Vec<sqlx::sqlite::SqliteRow>,
) -> AppResult<HashMap<i64, ContentCounts>> {
    let mut counts = HashMap::new();
    for row in rows {
        counts.insert(
            row.try_get::<i64, _>("source_id")
                .map_err(AppError::database)?,
            ContentCounts {
                item_count: row
                    .try_get::<i64, _>("item_count")
                    .map_err(AppError::database)?,
                segment_count: row
                    .try_get::<i64, _>("segment_count")
                    .map_err(AppError::database)?,
                last_synced_at: row
                    .try_get::<Option<i64>, _>("last_synced_at")
                    .map_err(AppError::database)?,
            },
        );
    }
    Ok(counts)
}

fn summary_from_row(
    row: SourceSummaryRow,
    caption_counts: ContentCounts,
    comment_counts: ContentCounts,
    playlist_counts: Option<PlaylistCounts>,
    video_metadata: Option<&YoutubeVideoSourceMetadata>,
    playlist_metadata: Option<&YoutubePlaylistSourceMetadata>,
) -> YoutubeSourceSummaryDto {
    let source_subtype = row.source_subtype.unwrap_or_default();
    match source_subtype.as_str() {
        "playlist" => {
            let availability_status =
                playlist_metadata.map(|metadata| metadata.availability_status.clone());
            YoutubeSourceSummaryDto {
                source_id: row.id,
                source_subtype,
                title: playlist_metadata
                    .and_then(|metadata| metadata.title.clone())
                    .or(row.title),
                channel_title: playlist_metadata
                    .and_then(|metadata| metadata.channel_title.clone()),
                channel_handle: playlist_metadata
                    .and_then(|metadata| metadata.channel_handle.clone()),
                canonical_url: playlist_metadata
                    .map(|metadata| metadata.canonical_url.clone())
                    .or_else(|| {
                        Some(format!(
                            "https://www.youtube.com/playlist?list={}",
                            row.external_id
                        ))
                    }),
                thumbnail_url: playlist_metadata
                    .and_then(|metadata| metadata.thumbnail_url.clone()),
                duration_seconds: None,
                published_at: None,
                availability_status,
                video_count: playlist_metadata
                    .and_then(|metadata| metadata.video_count)
                    .or_else(|| playlist_counts.map(|counts| counts.total_count)),
                linked_video_count: playlist_counts.map(|counts| counts.linked_count),
                unavailable_count: playlist_counts.map(|counts| counts.unavailable_count),
                captions: caption_status(caption_counts, None),
                comments: comment_status(comment_counts),
            }
        }
        _ => {
            let availability_status =
                video_metadata.map(|metadata| metadata.availability_status.clone());
            YoutubeSourceSummaryDto {
                source_id: row.id,
                source_subtype,
                title: video_metadata
                    .and_then(|metadata| metadata.title.clone())
                    .or(row.title),
                channel_title: video_metadata.and_then(|metadata| metadata.channel_title.clone()),
                channel_handle: video_metadata.and_then(|metadata| metadata.channel_handle.clone()),
                canonical_url: video_metadata
                    .map(|metadata| metadata.canonical_url.clone())
                    .or_else(|| {
                        Some(format!(
                            "https://www.youtube.com/watch?v={}",
                            row.external_id
                        ))
                    }),
                thumbnail_url: video_metadata.and_then(|metadata| metadata.thumbnail_url.clone()),
                duration_seconds: video_metadata.and_then(|metadata| metadata.duration_seconds),
                published_at: video_metadata
                    .and_then(|metadata| metadata.published_at.as_deref())
                    .and_then(ymd_to_unix_midnight),
                availability_status: availability_status.clone(),
                video_count: None,
                linked_video_count: None,
                unavailable_count: None,
                captions: caption_status(caption_counts, availability_status.as_deref()),
                comments: comment_status(comment_counts),
            }
        }
    }
}

fn caption_status(
    counts: ContentCounts,
    availability_status: Option<&str>,
) -> YoutubeContentStatusDto {
    let state = if counts.item_count > 0 {
        YoutubeContentSyncState::Synced
    } else if availability_status.is_some_and(captions_unavailable_for_status) {
        YoutubeContentSyncState::Unavailable
    } else {
        YoutubeContentSyncState::NotSynced
    };
    let label = match state {
        YoutubeContentSyncState::Synced => "Captions synced",
        YoutubeContentSyncState::Unavailable => "Captions unavailable",
        YoutubeContentSyncState::Failed => "Captions sync failed",
        YoutubeContentSyncState::Unknown => "Captions status unknown",
        YoutubeContentSyncState::NotSynced => "Captions not synced",
    }
    .to_string();

    YoutubeContentStatusDto {
        state,
        item_count: counts.item_count,
        segment_count: counts.segment_count,
        last_synced_at: counts.last_synced_at,
        label,
    }
}

fn comment_status(counts: ContentCounts) -> YoutubeContentStatusDto {
    let state = if counts.item_count > 0 {
        YoutubeContentSyncState::Synced
    } else {
        YoutubeContentSyncState::NotSynced
    };
    let label = match state {
        YoutubeContentSyncState::Synced => "Comments synced",
        YoutubeContentSyncState::Unavailable => "Comments unavailable",
        YoutubeContentSyncState::Failed => "Comments sync failed",
        YoutubeContentSyncState::Unknown => "Comments status unknown",
        YoutubeContentSyncState::NotSynced => "Comments not synced",
    }
    .to_string();

    YoutubeContentStatusDto {
        state,
        item_count: counts.item_count,
        segment_count: 0,
        last_synced_at: counts.last_synced_at,
        label,
    }
}

fn captions_unavailable_for_status(status: &str) -> bool {
    matches!(
        status,
        "no_captions"
            | "private_or_auth_required"
            | "members_only"
            | "age_restricted"
            | "geo_blocked"
            | "deleted"
            | "removed_from_playlist"
            | "unavailable_unknown"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        get_youtube_playlist_detail_from_pool, get_youtube_video_detail_from_pool,
        list_youtube_source_summaries_from_pool, YoutubeContentSyncState,
    };
    use crate::error::AppErrorKind;

    #[tokio::test]
    async fn video_detail_reports_synced_transcript_comments_and_playlist_memberships() {
        let pool = youtube_detail_pool().await;
        seed_video(&pool, 10, "video01", "Demo Video").await;
        seed_playlist(&pool, 20, "PLdemo", "Demo Playlist").await;
        seed_playlist_item(&pool, 20, Some(10), "video01", Some(3), "available", false).await;
        seed_transcript(&pool, 10, 100, 1_800_000_000, 2).await;
        seed_comment(&pool, 10, "comment01", 1_800_000_100).await;

        let detail = get_youtube_video_detail_from_pool(&pool, 10)
            .await
            .expect("load video detail");

        assert_eq!(detail.summary.source_id, 10);
        assert_eq!(detail.summary.title.as_deref(), Some("Demo Video"));
        assert_eq!(
            detail.summary.captions.state,
            YoutubeContentSyncState::Synced
        );
        assert_eq!(detail.summary.captions.item_count, 1);
        assert_eq!(detail.summary.captions.segment_count, 2);
        assert_eq!(
            detail.summary.comments.state,
            YoutubeContentSyncState::Synced
        );
        assert_eq!(detail.summary.comments.item_count, 1);
        assert_eq!(detail.playlist_memberships.len(), 1);
        assert_eq!(detail.playlist_memberships[0].playlist_source_id, 20);
        assert_eq!(detail.playlist_memberships[0].position, Some(3));
    }

    #[tokio::test]
    async fn playlist_detail_reports_ordered_items_and_summary_counts() {
        let pool = youtube_detail_pool().await;
        seed_playlist(&pool, 20, "PLdemo", "Demo Playlist").await;
        seed_video(&pool, 10, "video01", "Linked Video").await;
        seed_transcript(&pool, 10, 100, 1_800_000_000, 1).await;
        seed_playlist_item(
            &pool,
            20,
            None,
            "private01",
            Some(2),
            "private_or_auth_required",
            false,
        )
        .await;
        seed_playlist_item(&pool, 20, Some(10), "video01", Some(1), "available", false).await;
        seed_playlist_item(
            &pool,
            20,
            None,
            "removed01",
            None,
            "removed_from_playlist",
            true,
        )
        .await;

        let detail = get_youtube_playlist_detail_from_pool(&pool, 20)
            .await
            .expect("load playlist detail");

        assert_eq!(detail.summary.video_count, Some(3));
        assert_eq!(detail.summary.linked_video_count, Some(1));
        assert_eq!(detail.summary.unavailable_count, Some(2));
        assert_eq!(
            detail
                .items
                .iter()
                .map(|item| item.video_id.as_str())
                .collect::<Vec<_>>(),
            vec!["video01", "private01", "removed01"]
        );
        assert_eq!(
            detail.items[0].captions.state,
            YoutubeContentSyncState::Synced
        );
        assert_eq!(
            detail.items[1].captions.state,
            YoutubeContentSyncState::Unavailable
        );
    }

    #[tokio::test]
    async fn list_summaries_uses_source_id_order_and_marks_no_captions_unavailable() {
        let pool = youtube_detail_pool().await;
        seed_video_with_availability(&pool, 10, "video01", "Demo Video", "no_captions").await;
        seed_playlist(&pool, 20, "PLdemo", "Demo Playlist").await;

        let summaries = list_youtube_source_summaries_from_pool(&pool, vec![20, 10])
            .await
            .expect("load summaries");

        assert_eq!(
            summaries
                .iter()
                .map(|summary| summary.source_id)
                .collect::<Vec<_>>(),
            vec![20, 10]
        );
        assert_eq!(
            summaries[1].captions.state,
            YoutubeContentSyncState::Unavailable
        );
        assert_eq!(summaries[1].captions.label, "Captions unavailable");
    }

    #[tokio::test]
    async fn summaries_use_typed_video_metadata_with_corrupt_source_blob() {
        let pool = youtube_detail_pool().await;
        let source_id =
            insert_youtube_video_source(&pool, "video01", "Generic title", "available").await;
        sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = ?")
            .bind(source_id)
            .execute(&pool)
            .await
            .expect("corrupt source blob");

        let summaries = list_youtube_source_summaries_from_pool(&pool, vec![source_id])
            .await
            .expect("list summaries");

        assert_eq!(summaries[0].title.as_deref(), Some("Typed video title"));
        assert_eq!(
            summaries[0].canonical_url.as_deref(),
            Some("https://www.youtube.com/watch?v=video01")
        );
        assert_eq!(
            summaries[0].availability_status.as_deref(),
            Some("available")
        );
    }

    #[tokio::test]
    async fn source_summary_missing_typed_metadata_uses_generic_title_without_blob_decode() {
        let pool = youtube_detail_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (901, 'youtube', 'video', 'missing01', 'Generic fallback', x'00', 1, 0, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert source");

        let summaries = list_youtube_source_summaries_from_pool(&pool, vec![901])
            .await
            .expect("list summaries");

        assert_eq!(summaries[0].title.as_deref(), Some("Generic fallback"));
        assert_eq!(
            summaries[0].canonical_url.as_deref(),
            Some("https://www.youtube.com/watch?v=missing01")
        );
        assert_eq!(summaries[0].availability_status, None);
    }

    #[tokio::test]
    async fn video_detail_missing_typed_metadata_returns_controlled_error() {
        let pool = youtube_detail_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd, is_active, is_member, created_at) VALUES (902, 'youtube', 'video', 'missing02', 'Generic fallback', x'00', 1, 0, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert source");

        let error = get_youtube_video_detail_from_pool(&pool, 902)
            .await
            .expect_err("missing typed metadata rejected");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert!(error.to_string().contains("typed YouTube video metadata"));
        assert!(!error.to_string().contains("metadata_zstd"));
    }

    #[tokio::test]
    async fn playlist_detail_uses_typed_linked_video_metadata_with_corrupt_source_blob() {
        let pool = youtube_detail_pool().await;
        let playlist_id = insert_youtube_playlist_source(&pool, "PLdemo", "Generic playlist").await;
        let video_id =
            insert_youtube_video_source(&pool, "video02", "Generic linked video", "available")
                .await;
        sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = ?")
            .bind(video_id)
            .execute(&pool)
            .await
            .expect("corrupt source blob");
        insert_playlist_item(
            &pool,
            playlist_id,
            Some(video_id),
            "video02",
            "Snapshot title",
        )
        .await;

        let detail = get_youtube_playlist_detail_from_pool(&pool, playlist_id)
            .await
            .expect("playlist detail");

        assert_eq!(detail.items[0].title.as_deref(), Some("Typed video title"));
        assert_eq!(
            detail.items[0].canonical_url.as_deref(),
            Some("https://www.youtube.com/watch?v=video02")
        );
    }

    async fn youtube_detail_pool() -> sqlx::SqlitePool {
        let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;
        crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
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
        sqlx::query(
            r#"
            CREATE TABLE youtube_transcript_segments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                segment_index INTEGER NOT NULL,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER,
                text TEXT NOT NULL,
                chapter_index INTEGER,
                caption_language TEXT,
                caption_track_kind TEXT,
                is_auto_generated INTEGER NOT NULL DEFAULT 0,
                metadata_zstd BLOB,
                UNIQUE(item_id, segment_index)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create youtube_transcript_segments");
        pool
    }

    async fn seed_video(pool: &sqlx::SqlitePool, id: i64, video_id: &str, title: &str) {
        seed_video_with_availability(pool, id, video_id, title, "available").await;
    }

    async fn seed_video_with_availability(
        pool: &sqlx::SqlitePool,
        id: i64,
        video_id: &str,
        title: &str,
        availability: &str,
    ) {
        let metadata_zstd = youtube_video_metadata_zstd(video_id, title, availability);
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id,
                title, metadata_zstd, is_active, is_member, created_at
            )
            VALUES (?, 'youtube', 'video', NULL, ?, ?, ?, 1, 0, 1)
            "#,
        )
        .bind(id)
        .bind(video_id)
        .bind(title)
        .bind(metadata_zstd)
        .execute(pool)
        .await
        .expect("seed video");
        insert_typed_video_source(pool, id, video_id, title, availability).await;
    }

    async fn seed_playlist(pool: &sqlx::SqlitePool, id: i64, playlist_id: &str, title: &str) {
        let metadata_zstd = youtube_playlist_metadata_zstd(playlist_id, title);
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id,
                title, metadata_zstd, is_active, is_member, created_at
            )
            VALUES (?, 'youtube', 'playlist', NULL, ?, ?, ?, 1, 0, 1)
            "#,
        )
        .bind(id)
        .bind(playlist_id)
        .bind(title)
        .bind(metadata_zstd)
        .execute(pool)
        .await
        .expect("seed playlist");
        insert_typed_playlist_source(pool, id, playlist_id, title, Some(3)).await;
    }

    async fn insert_youtube_video_source(
        pool: &sqlx::SqlitePool,
        video_id: &str,
        generic_title: &str,
        availability: &str,
    ) -> i64 {
        let metadata_zstd = youtube_video_metadata_zstd(video_id, generic_title, availability);
        let source_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO sources (
                source_type, source_subtype, account_id, external_id, title,
                metadata_zstd, is_active, is_member, created_at
            )
            VALUES ('youtube', 'video', NULL, ?, ?, ?, 1, 0, 1)
            RETURNING id
            "#,
        )
        .bind(video_id)
        .bind(generic_title)
        .bind(metadata_zstd)
        .fetch_one(pool)
        .await
        .expect("insert youtube video source");
        insert_typed_video_source(pool, source_id, video_id, "Typed video title", availability)
            .await;
        source_id
    }

    async fn insert_youtube_playlist_source(
        pool: &sqlx::SqlitePool,
        playlist_id: &str,
        generic_title: &str,
    ) -> i64 {
        let metadata_zstd = youtube_playlist_metadata_zstd(playlist_id, generic_title);
        let source_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO sources (
                source_type, source_subtype, account_id, external_id, title,
                metadata_zstd, is_active, is_member, created_at
            )
            VALUES ('youtube', 'playlist', NULL, ?, ?, ?, 1, 0, 1)
            RETURNING id
            "#,
        )
        .bind(playlist_id)
        .bind(generic_title)
        .bind(metadata_zstd)
        .fetch_one(pool)
        .await
        .expect("insert youtube playlist source");
        insert_typed_playlist_source(pool, source_id, playlist_id, "Typed playlist title", None)
            .await;
        source_id
    }

    async fn insert_typed_video_source(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        video_id: &str,
        title: &str,
        availability: &str,
    ) {
        sqlx::query(
            r#"
            INSERT INTO youtube_video_sources (
                source_id, video_id, canonical_url, title, channel_title,
                channel_handle, author_display, published_at, duration_seconds,
                description, thumbnail_url, video_form, availability_status
            )
            VALUES (?, ?, ?, ?, 'Demo Channel', '@demo', 'Demo Channel', '2026-05-01',
                    120, 'Demo description', ?, 'regular', ?)
            "#,
        )
        .bind(source_id)
        .bind(video_id)
        .bind(format!("https://www.youtube.com/watch?v={video_id}"))
        .bind(title)
        .bind(format!(
            "https://img.youtube.com/vi/{video_id}/hqdefault.jpg"
        ))
        .bind(availability)
        .execute(pool)
        .await
        .expect("insert typed video source");
    }

    async fn insert_typed_playlist_source(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        playlist_id: &str,
        title: &str,
        video_count: Option<i64>,
    ) {
        sqlx::query(
            r#"
            INSERT INTO youtube_playlist_sources (
                source_id, playlist_id, canonical_url, title, channel_title,
                channel_handle, thumbnail_url, video_count, availability_status
            )
            VALUES (?, ?, ?, ?, 'Demo Channel', '@demo',
                    'https://img.youtube.com/playlist.jpg', ?, 'available')
            "#,
        )
        .bind(source_id)
        .bind(playlist_id)
        .bind(format!(
            "https://www.youtube.com/playlist?list={playlist_id}"
        ))
        .bind(title)
        .bind(video_count)
        .execute(pool)
        .await
        .expect("insert typed playlist source");
    }

    async fn insert_playlist_item(
        pool: &sqlx::SqlitePool,
        playlist_source_id: i64,
        video_source_id: Option<i64>,
        video_id: &str,
        title_snapshot: &str,
    ) {
        sqlx::query(
            r#"
            INSERT INTO youtube_playlist_items (
                playlist_source_id, video_source_id, video_id, position, title_snapshot, url,
                thumbnail_url, availability_status, is_removed_from_playlist, last_seen_at,
                created_at, updated_at
            )
            VALUES (?, ?, ?, 1, ?, ?, ?, 'available', 0, 1, 1, 1)
            "#,
        )
        .bind(playlist_source_id)
        .bind(video_source_id)
        .bind(video_id)
        .bind(title_snapshot)
        .bind(format!("https://www.youtube.com/watch?v={video_id}"))
        .bind(format!(
            "https://img.youtube.com/vi/{video_id}/hqdefault.jpg"
        ))
        .execute(pool)
        .await
        .expect("insert playlist item");
    }

    async fn seed_playlist_item(
        pool: &sqlx::SqlitePool,
        playlist_source_id: i64,
        video_source_id: Option<i64>,
        video_id: &str,
        position: Option<i64>,
        availability_status: &str,
        is_removed_from_playlist: bool,
    ) {
        sqlx::query(
            r#"
            INSERT INTO youtube_playlist_items (
                playlist_source_id, video_source_id, video_id, position, title_snapshot, url,
                thumbnail_url, availability_status, is_removed_from_playlist, last_seen_at,
                created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, 1, 1)
            "#,
        )
        .bind(playlist_source_id)
        .bind(video_source_id)
        .bind(video_id)
        .bind(position)
        .bind(format!("Playlist item {video_id}"))
        .bind(format!("https://www.youtube.com/watch?v={video_id}"))
        .bind(format!(
            "https://img.youtube.com/vi/{video_id}/hqdefault.jpg"
        ))
        .bind(availability_status)
        .bind(is_removed_from_playlist)
        .execute(pool)
        .await
        .expect("seed playlist item");
    }

    async fn seed_transcript(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        item_id: i64,
        ingested_at: i64,
        segment_count: i64,
    ) {
        sqlx::query(
            r#"
            INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_kind, has_media
            )
            VALUES (?, ?, ?, 'youtube_transcript', 'Demo Channel', 1, ?, 'text_only', 0)
            "#,
        )
        .bind(item_id)
        .bind(source_id)
        .bind(format!("transcript:{source_id}:en:manual"))
        .bind(ingested_at)
        .execute(pool)
        .await
        .expect("seed transcript item");

        for index in 0..segment_count {
            sqlx::query(
                r#"
                INSERT INTO youtube_transcript_segments (
                    item_id, source_id, segment_index, start_ms, end_ms, text
                )
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(item_id)
            .bind(source_id)
            .bind(index)
            .bind(index * 1_000)
            .bind(index * 1_000 + 900)
            .bind(format!("Segment {index}"))
            .execute(pool)
            .await
            .expect("seed transcript segment");
        }
    }

    async fn seed_comment(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        comment_id: &str,
        ingested_at: i64,
    ) {
        sqlx::query(
            r#"
            INSERT INTO items (
                source_id, external_id, item_kind, author, published_at, ingested_at,
                content_kind, has_media
            )
            VALUES (?, ?, 'youtube_comment', 'Alice', 1, ?, 'text_only', 0)
            "#,
        )
        .bind(source_id)
        .bind(comment_id)
        .bind(ingested_at)
        .execute(pool)
        .await
        .expect("seed comment");
    }

    fn youtube_video_metadata_zstd(video_id: &str, title: &str, availability: &str) -> Vec<u8> {
        let metadata = serde_json::json!({
            "video_id": video_id,
            "canonical_url": format!("https://www.youtube.com/watch?v={video_id}"),
            "title": title,
            "channel_title": "Demo Channel",
            "channel_id": "UCdemo",
            "channel_handle": "@demo",
            "channel_url": "https://www.youtube.com/@demo",
            "author_display": "Demo Channel",
            "published_at": "2026-05-01",
            "duration_seconds": 120,
            "description": "Demo description",
            "thumbnail_url": format!("https://img.youtube.com/vi/{video_id}/hqdefault.jpg"),
            "tags": [],
            "chapters": [],
            "view_count": 10,
            "like_count": 2,
            "comment_count": 1,
            "category": "Education",
            "video_form": "regular",
            "availability_status": availability,
            "raw_metadata_json": { "id": video_id }
        });
        crate::compression::compress_json_bytes(
            &serde_json::to_vec(&metadata).expect("serialize video metadata"),
        )
        .expect("compress video metadata")
    }

    fn youtube_playlist_metadata_zstd(playlist_id: &str, title: &str) -> Vec<u8> {
        let metadata = serde_json::json!({
            "playlist_id": playlist_id,
            "canonical_url": format!("https://www.youtube.com/playlist?list={playlist_id}"),
            "title": title,
            "channel_title": "Demo Channel",
            "channel_id": "UCdemo",
            "channel_handle": "@demo",
            "channel_url": "https://www.youtube.com/@demo",
            "thumbnail_url": "https://img.youtube.com/playlist.jpg",
            "video_count": 3,
            "items": [],
            "availability_status": "available",
            "raw_metadata_json": { "id": playlist_id }
        });
        crate::compression::compress_json_bytes(
            &serde_json::to_vec(&metadata).expect("serialize playlist metadata"),
        )
        .expect("compress playlist metadata")
    }
}
