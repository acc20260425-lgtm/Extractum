use super::test_support::TestPromptPackSourceReader as AppPromptPackSourceReader;
use super::test_support::*;
use super::{model_budget_for_runtime, preflight_youtube_summary, ModelBudget};

#[tokio::test]
async fn preflight_explicit_video_without_transcript_is_blocking_failure() {
    let pool = test_pool_with_youtube_video_without_transcript().await;
    let source = AppPromptPackSourceReader::new(pool.clone());

    let response = preflight_youtube_summary(
        &source,
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
    let source = AppPromptPackSourceReader::new(pool.clone());

    let response = preflight_youtube_summary(
        &source,
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

#[tokio::test]
async fn browser_runtime_preflight_does_not_apply_api_input_limit() {
    let pool = test_pool_with_ready_video().await;
    sqlx::query("UPDATE youtube_transcript_segments SET text = ? WHERE source_id = ?")
        .bind("x".repeat(160_000))
        .bind(901_i64)
        .execute(&pool)
        .await
        .expect("update long transcript");
    let mut request = request_for_video(901);
    request.runtime_provider = crate::dto::PromptPackRuntimeProvider::GeminiBrowser;
    request.model_override = None;
    let source = AppPromptPackSourceReader::new(pool.clone());

    let response = preflight_youtube_summary(
        &source,
        request,
        ModelBudget {
            input_token_limit: None,
        },
    )
    .await
    .expect("browser preflight");

    assert_eq!(response.included_videos.len(), 1);
    assert_eq!(response.selected_model_input_limit, None);
}

#[tokio::test]
async fn api_runtime_preflight_uses_fixed_32000_input_limit() {
    let pool = test_pool_with_ready_video().await;
    sqlx::query("UPDATE youtube_transcript_segments SET text = ? WHERE source_id = ?")
        .bind("x".repeat(160_000))
        .bind(901_i64)
        .execute(&pool)
        .await
        .expect("update long transcript");
    let source = AppPromptPackSourceReader::new(pool.clone());

    let response = preflight_youtube_summary(
        &source,
        request_for_video(901),
        model_budget_for_runtime(crate::dto::PromptPackRuntimeProvider::Api),
    )
    .await
    .expect("api preflight");

    assert_eq!(response.selected_model_input_limit, Some(32_000));
    assert!(response.included_videos.is_empty());
    assert_eq!(response.blocking_failures.len(), 1);
    assert_eq!(
        response.blocking_failures[0].reason,
        "input_budget_exceeded"
    );
}

#[tokio::test]
async fn preflight_gem_analysis_allows_exactly_one_included_video() {
    let pool = test_pool_with_ready_video().await;
    let mut request = request_for_video(901);
    request.control_preset = "gem_analysis".to_string();
    let source = AppPromptPackSourceReader::new(pool.clone());

    let response = preflight_youtube_summary(
        &source,
        request,
        ModelBudget {
            input_token_limit: Some(32_000),
        },
    )
    .await
    .expect("preflight");

    assert_eq!(response.included_videos.len(), 1);
    assert!(response.blocking_failures.is_empty());
}

#[tokio::test]
async fn preflight_gem_analysis_blocks_multiple_included_videos() {
    let pool = test_pool_with_playlist_two_ready_videos().await;
    let mut request = request_for_playlist(701);
    request.control_preset = "gem_analysis".to_string();
    let source = AppPromptPackSourceReader::new(pool.clone());

    let response = preflight_youtube_summary(
        &source,
        request,
        ModelBudget {
            input_token_limit: Some(32_000),
        },
    )
    .await
    .expect("preflight");

    assert_eq!(
        response.blocking_failures[0].reason,
        "gem_analysis_requires_single_video"
    );
    assert!(response.blocking_failures[0]
        .message
        .as_deref()
        .unwrap_or_default()
        .contains("exactly one YouTube video"));
}
