use sqlx::SqlitePool;

use super::dto::{
    PreflightYoutubeSummaryRunRequest, YoutubeSummaryPreflightFailure,
    YoutubeSummaryPreflightResponse, YoutubeSummaryPreflightSkippedVideo,
    YoutubeSummaryPreflightVideo,
};
use super::youtube_summary::{estimate_tokens, ModelBudget};
use super::youtube_summary_sources::{
    load_playlist_candidates, load_source, load_video_candidate, transcript_text_for_source,
    PlaylistCandidate, VideoCandidate,
};
use crate::error::AppResult;

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
                message: Some(
                    "The selected YouTube video exceeds the model input budget".to_string(),
                ),
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
