use super::{estimate_tokens, render_transcript_snapshot_text, ModelBudget};
use crate::error::AppResult;
use crate::prompt_packs::dto::{
    PreflightYoutubeSummaryRunRequest, YoutubeSummaryPreflightFailure,
    YoutubeSummaryPreflightResponse, YoutubeSummaryPreflightSkippedVideo,
    YoutubeSummaryPreflightVideo,
};
use crate::prompt_packs::source_port::{PromptPackSourceReader, YoutubeVideoReadRequest};

struct VideoCandidate {
    source_id: i64,
    video_id: String,
    title: String,
    description: Option<String>,
    is_playlist_child: bool,
}

enum PlaylistCandidate {
    Linked(VideoCandidate),
    Unlinked {
        video_id: String,
        title: Option<String>,
    },
}

pub(crate) async fn preflight_youtube_summary(
    source: &dyn PromptPackSourceReader,
    request: PreflightYoutubeSummaryRunRequest,
    model_budget: ModelBudget,
) -> AppResult<YoutubeSummaryPreflightResponse> {
    let mut included_videos = Vec::new();
    let mut skipped_videos = Vec::new();
    let mut blocking_failures = Vec::new();
    let mut estimated_input_tokens = 0;

    for source_id in request.source_ids {
        let Some(source_record) = source.load_source(source_id).await? else {
            blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(source_id),
                reason: "source_not_found".to_string(),
                message: Some("Source was not found".to_string()),
            });
            continue;
        };

        if source_record.source_type() != "youtube" {
            blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(source_record.id()),
                reason: "unsupported_source_type".to_string(),
                message: Some("Only YouTube sources can be summarized".to_string()),
            });
            continue;
        }

        match source_record.source_subtype() {
            Some("video") => {
                if let Some(video) = load_video_candidate(source, source_record.id(), false).await?
                {
                    classify_video(
                        source,
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
                        source_id: Some(source_record.id()),
                        reason: "missing_video_metadata".to_string(),
                        message: Some("YouTube video metadata is missing".to_string()),
                    });
                }
            }
            Some("playlist") => {
                let children = load_playlist_candidates(source, source_record.id()).await?;
                if children.is_empty() {
                    skipped_videos.push(YoutubeSummaryPreflightSkippedVideo {
                        source_id: Some(source_record.id()),
                        video_id: None,
                        title: source_record.title().map(str::to_owned),
                        reason: "empty_playlist".to_string(),
                    });
                }
                for child in children {
                    match child {
                        PlaylistCandidate::Linked(video) => {
                            classify_video(
                                source,
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
                source_id: Some(source_record.id()),
                reason: "unsupported_source_subtype".to_string(),
                message: Some("Only YouTube video and playlist sources are supported".to_string()),
            }),
        }
    }

    if request.control_preset == "gem_analysis" && included_videos.len() != 1 {
        blocking_failures.push(YoutubeSummaryPreflightFailure {
            source_id: None,
            reason: "gem_analysis_requires_single_video".to_string(),
            message: Some("Gem analysis supports exactly one YouTube video.".to_string()),
        });
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
    source: &dyn PromptPackSourceReader,
    video: VideoCandidate,
    model_budget: ModelBudget,
    included_videos: &mut Vec<YoutubeSummaryPreflightVideo>,
    skipped_videos: &mut Vec<YoutubeSummaryPreflightSkippedVideo>,
    blocking_failures: &mut Vec<YoutubeSummaryPreflightFailure>,
    estimated_input_tokens: &mut i64,
) -> AppResult<()> {
    let transcript_segments = source.load_transcript_segments(video.source_id).await?;
    let transcript_text = render_transcript_snapshot_text(&transcript_segments);
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

async fn load_video_candidate(
    source: &dyn PromptPackSourceReader,
    source_id: i64,
    is_playlist_child: bool,
) -> AppResult<Option<VideoCandidate>> {
    Ok(source
        .load_video(YoutubeVideoReadRequest::new(source_id))
        .await?
        .map(|video| VideoCandidate {
            source_id: video.source_id(),
            video_id: video.video_id().to_string(),
            title: video
                .title()
                .map(str::to_owned)
                .unwrap_or_else(|| video.video_id().to_string()),
            description: video.description().map(str::to_owned),
            is_playlist_child,
        }))
}

async fn load_playlist_candidates(
    source: &dyn PromptPackSourceReader,
    playlist_source_id: i64,
) -> AppResult<Vec<PlaylistCandidate>> {
    let rows = source.load_playlist_items(playlist_source_id).await?;
    let mut candidates = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(source_id) = row.video_source_id() {
            if let Some(video) = load_video_candidate(source, source_id, true).await? {
                candidates.push(PlaylistCandidate::Linked(video));
            } else {
                candidates.push(PlaylistCandidate::Unlinked {
                    video_id: row.video_id().to_string(),
                    title: row.title().map(str::to_owned),
                });
            }
        } else {
            candidates.push(PlaylistCandidate::Unlinked {
                video_id: row.video_id().to_string(),
                title: row.title().map(str::to_owned),
            });
        }
    }
    Ok(candidates)
}
