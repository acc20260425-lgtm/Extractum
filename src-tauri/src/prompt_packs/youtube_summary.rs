use sqlx::SqlitePool;

use super::dto::{
    PreflightYoutubeSummaryRunRequest, YoutubeSummaryPreflightFailure,
    YoutubeSummaryPreflightResponse, YoutubeSummaryPreflightSkippedVideo,
    YoutubeSummaryPreflightVideo,
};
use crate::error::{AppError, AppResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelBudget {
    pub input_token_limit: Option<i64>,
}

#[derive(Clone, Debug)]
struct SourceRow {
    id: i64,
    source_type: String,
    source_subtype: Option<String>,
    title: Option<String>,
}

#[derive(Clone, Debug)]
struct VideoCandidate {
    source_id: i64,
    video_id: String,
    title: String,
    description: Option<String>,
    is_playlist_child: bool,
}

pub(crate) async fn preflight_youtube_summary_in_pool(
    pool: &SqlitePool,
    request: PreflightYoutubeSummaryRunRequest,
    model_budget: ModelBudget,
) -> AppResult<YoutubeSummaryPreflightResponse> {
    let mut included_videos = Vec::new();
    let mut skipped_videos = Vec::new();
    let mut blocking_failures = Vec::new();
    let mut estimated_input_tokens = 0;

    for source_id in request.source_ids {
        let Some(source) = load_source(pool, source_id).await? else {
            blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(source_id),
                reason: "source_not_found".to_string(),
                message: Some("Source was not found".to_string()),
            });
            continue;
        };

        if source.source_type != "youtube" {
            blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(source.id),
                reason: "unsupported_source_type".to_string(),
                message: Some("Only YouTube sources can be summarized".to_string()),
            });
            continue;
        }

        match source.source_subtype.as_deref() {
            Some("video") => {
                if let Some(video) = load_video_candidate(pool, source.id, false).await? {
                    classify_video(
                        pool,
                        video,
                        model_budget,
                        &mut included_videos,
                        &mut skipped_videos,
                        &mut blocking_failures,
                        &mut estimated_input_tokens,
                    )
                    .await?;
                } else {
                    blocking_failures.push(YoutubeSummaryPreflightFailure {
                        source_id: Some(source.id),
                        reason: "missing_video_metadata".to_string(),
                        message: Some("YouTube video metadata is missing".to_string()),
                    });
                }
            }
            Some("playlist") => {
                let children = load_playlist_candidates(pool, source.id).await?;
                if children.is_empty() {
                    skipped_videos.push(YoutubeSummaryPreflightSkippedVideo {
                        source_id: Some(source.id),
                        video_id: None,
                        title: source.title,
                        reason: "empty_playlist".to_string(),
                    });
                }
                for child in children {
                    match child {
                        PlaylistCandidate::Linked(video) => {
                            classify_video(
                                pool,
                                video,
                                model_budget,
                                &mut included_videos,
                                &mut skipped_videos,
                                &mut blocking_failures,
                                &mut estimated_input_tokens,
                            )
                            .await?;
                        }
                        PlaylistCandidate::Unlinked { video_id, title } => {
                            skipped_videos.push(YoutubeSummaryPreflightSkippedVideo {
                                source_id: None,
                                video_id: Some(video_id),
                                title,
                                reason: "unlinked_playlist_item".to_string(),
                            });
                        }
                    }
                }
            }
            _ => blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(source.id),
                reason: "unsupported_source_subtype".to_string(),
                message: Some("Only YouTube video and playlist sources are supported".to_string()),
            }),
        }
    }

    Ok(YoutubeSummaryPreflightResponse {
        pack_id: "youtube_summary".to_string(),
        pack_version: "1.0.0".to_string(),
        included_videos,
        skipped_videos,
        blocking_failures,
        estimated_input_tokens,
        selected_model_input_limit: model_budget.input_token_limit,
    })
}

async fn classify_video(
    pool: &SqlitePool,
    video: VideoCandidate,
    model_budget: ModelBudget,
    included_videos: &mut Vec<YoutubeSummaryPreflightVideo>,
    skipped_videos: &mut Vec<YoutubeSummaryPreflightSkippedVideo>,
    blocking_failures: &mut Vec<YoutubeSummaryPreflightFailure>,
    estimated_input_tokens: &mut i64,
) -> AppResult<()> {
    let transcript_text = transcript_text_for_source(pool, video.source_id).await?;
    if transcript_text.trim().is_empty() {
        if video.is_playlist_child {
            skipped_videos.push(YoutubeSummaryPreflightSkippedVideo {
                source_id: Some(video.source_id),
                video_id: Some(video.video_id),
                title: Some(video.title),
                reason: "no_usable_transcript".to_string(),
            });
        } else {
            blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(video.source_id),
                reason: "no_usable_transcript".to_string(),
                message: Some("The selected YouTube video has no usable transcript".to_string()),
            });
        }
        return Ok(());
    }

    let token_estimate = estimate_tokens(&transcript_text)
        + estimate_tokens(video.description.as_deref().unwrap_or(""))
        + 800;
    if model_budget
        .input_token_limit
        .is_some_and(|limit| token_estimate > limit)
    {
        if video.is_playlist_child {
            skipped_videos.push(YoutubeSummaryPreflightSkippedVideo {
                source_id: Some(video.source_id),
                video_id: Some(video.video_id),
                title: Some(video.title),
                reason: "input_budget_exceeded".to_string(),
            });
        } else {
            blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(video.source_id),
                reason: "input_budget_exceeded".to_string(),
                message: Some("The selected YouTube video exceeds the model input budget".to_string()),
            });
        }
        return Ok(());
    }

    *estimated_input_tokens += token_estimate;
    included_videos.push(YoutubeSummaryPreflightVideo {
        source_id: video.source_id,
        video_id: video.video_id,
        title: video.title,
        estimated_input_tokens: token_estimate,
    });
    Ok(())
}

fn estimate_tokens(text: &str) -> i64 {
    ((text.chars().count() as f64) / 4.0).ceil() as i64
}

async fn load_source(pool: &SqlitePool, source_id: i64) -> AppResult<Option<SourceRow>> {
    sqlx::query_as::<_, (i64, String, Option<String>, Option<String>)>(
        "SELECT id, source_type, source_subtype, title FROM sources WHERE id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(|(id, source_type, source_subtype, title)| SourceRow {
            id,
            source_type,
            source_subtype,
            title,
        })
    })
    .map_err(AppError::database)
}

async fn load_video_candidate(
    pool: &SqlitePool,
    source_id: i64,
    is_playlist_child: bool,
) -> AppResult<Option<VideoCandidate>> {
    sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT video_id, title, description FROM youtube_video_sources WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(|(video_id, title, description)| VideoCandidate {
            source_id,
            title: title.unwrap_or_else(|| video_id.clone()),
            video_id,
            description,
            is_playlist_child,
        })
    })
    .map_err(AppError::database)
}

enum PlaylistCandidate {
    Linked(VideoCandidate),
    Unlinked {
        video_id: String,
        title: Option<String>,
    },
}

async fn load_playlist_candidates(
    pool: &SqlitePool,
    playlist_source_id: i64,
) -> AppResult<Vec<PlaylistCandidate>> {
    let rows = sqlx::query_as::<_, (Option<i64>, String, Option<String>)>(
        "SELECT video_source_id, video_id, title_snapshot
         FROM youtube_playlist_items
         WHERE playlist_source_id = ? AND is_removed_from_playlist = 0
         ORDER BY position ASC, id ASC",
    )
    .bind(playlist_source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut candidates = Vec::with_capacity(rows.len());
    for (video_source_id, video_id, title) in rows {
        if let Some(source_id) = video_source_id {
            if let Some(video) = load_video_candidate(pool, source_id, true).await? {
                candidates.push(PlaylistCandidate::Linked(video));
            } else {
                candidates.push(PlaylistCandidate::Unlinked { video_id, title });
            }
        } else {
            candidates.push(PlaylistCandidate::Unlinked { video_id, title });
        }
    }
    Ok(candidates)
}

async fn transcript_text_for_source(pool: &SqlitePool, source_id: i64) -> AppResult<String> {
    let segments = sqlx::query_scalar::<_, String>(
        "SELECT text
         FROM youtube_transcript_segments
         WHERE source_id = ?
         ORDER BY segment_index ASC, id ASC",
    )
    .bind(source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    Ok(segments.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::{preflight_youtube_summary_in_pool, ModelBudget};
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::prompt_packs::dto::PreflightYoutubeSummaryRunRequest;

    async fn migrated_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        pool
    }

    fn request_for_video(source_id: i64) -> PreflightYoutubeSummaryRunRequest {
        PreflightYoutubeSummaryRunRequest {
            project_id: None,
            source_ids: vec![source_id],
            profile_id: None,
            model_override: Some("test-model".to_string()),
            output_language: "en".to_string(),
            control_preset: "standard".to_string(),
            evidence_mode: "standard".to_string(),
            include_comments: false,
        }
    }

    fn request_for_playlist(source_id: i64) -> PreflightYoutubeSummaryRunRequest {
        request_for_video(source_id)
    }

    async fn insert_youtube_video(pool: &sqlx::SqlitePool, source_id: i64, video_id: &str) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at
             )
             VALUES (?, 'youtube', 'video', ?, ?, 1, 0, 1)",
        )
        .bind(source_id)
        .bind(video_id)
        .bind(format!("Video {video_id}"))
        .execute(pool)
        .await
        .expect("insert source");

        sqlx::query(
            "INSERT INTO youtube_video_sources (
                source_id, video_id, canonical_url, title, description,
                video_form, availability_status
             )
             VALUES (?, ?, ?, ?, 'Description', 'regular', 'available')",
        )
        .bind(source_id)
        .bind(video_id)
        .bind(format!("https://www.youtube.com/watch?v={video_id}"))
        .bind(format!("Video {video_id}"))
        .execute(pool)
        .await
        .expect("insert video metadata");
    }

    async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at
             )
             VALUES (?, 'youtube', 'playlist', 'playlist-1', 'Playlist', 1, 0, 1)",
        )
        .bind(playlist_source_id)
        .execute(pool)
        .await
        .expect("insert playlist source");

        sqlx::query(
            "INSERT INTO youtube_playlist_sources (
                source_id, playlist_id, canonical_url, title, availability_status
             )
             VALUES (?, 'playlist-1', 'https://www.youtube.com/playlist?list=playlist-1', 'Playlist', 'available')",
        )
        .bind(playlist_source_id)
        .execute(pool)
        .await
        .expect("insert playlist metadata");
    }

    async fn insert_playlist_item(
        pool: &sqlx::SqlitePool,
        playlist_source_id: i64,
        video_source_id: Option<i64>,
        video_id: &str,
        position: i64,
    ) {
        sqlx::query(
            "INSERT INTO youtube_playlist_items (
                playlist_source_id, video_source_id, video_id, position,
                title_snapshot, availability_status, is_removed_from_playlist
             )
             VALUES (?, ?, ?, ?, ?, 'available', 0)",
        )
        .bind(playlist_source_id)
        .bind(video_source_id)
        .bind(video_id)
        .bind(position)
        .bind(format!("Video {video_id}"))
        .execute(pool)
        .await
        .expect("insert playlist item");
    }

    async fn insert_transcript(pool: &sqlx::SqlitePool, source_id: i64, text: &str) {
        let item_id: i64 = sqlx::query_scalar(
            "INSERT INTO items (
                source_id, external_id, published_at, ingested_at, item_kind
             )
             VALUES (?, ?, 1, 1, 'youtube_transcript')
             RETURNING id",
        )
        .bind(source_id)
        .bind(format!("item-{source_id}"))
        .fetch_one(pool)
        .await
        .expect("insert transcript item");

        sqlx::query(
            "INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text
             )
             VALUES (?, ?, 0, 0, 1000, ?)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(text)
        .execute(pool)
        .await
        .expect("insert transcript segment");
    }

    async fn test_pool_with_youtube_video_without_transcript() -> sqlx::SqlitePool {
        let pool = migrated_pool().await;
        insert_youtube_video(&pool, 901, "v-missing").await;
        pool
    }

    async fn test_pool_with_playlist_one_ready_one_missing_transcript() -> sqlx::SqlitePool {
        let pool = migrated_pool().await;
        insert_playlist(&pool, 701).await;
        insert_youtube_video(&pool, 901, "v-ready").await;
        insert_youtube_video(&pool, 902, "v-missing").await;
        insert_transcript(&pool, 901, "Ready transcript").await;
        insert_playlist_item(&pool, 701, Some(901), "v-ready", 1).await;
        insert_playlist_item(&pool, 701, Some(902), "v-missing", 2).await;
        pool
    }

    #[tokio::test]
    async fn preflight_explicit_video_without_transcript_is_blocking_failure() {
        let pool = test_pool_with_youtube_video_without_transcript().await;

        let response = preflight_youtube_summary_in_pool(
            &pool,
            request_for_video(901),
            ModelBudget {
                input_token_limit: Some(32_000),
            },
        )
        .await
        .expect("preflight");

        assert!(response.included_videos.is_empty());
        assert_eq!(response.blocking_failures[0].reason, "no_usable_transcript");
    }

    #[tokio::test]
    async fn preflight_playlist_video_without_transcript_is_skipped() {
        let pool = test_pool_with_playlist_one_ready_one_missing_transcript().await;

        let response = preflight_youtube_summary_in_pool(
            &pool,
            request_for_playlist(701),
            ModelBudget {
                input_token_limit: Some(32_000),
            },
        )
        .await
        .expect("preflight");

        assert_eq!(response.included_videos.len(), 1);
        assert_eq!(response.skipped_videos[0].reason, "no_usable_transcript");
        assert!(response.blocking_failures.is_empty());
    }
}
