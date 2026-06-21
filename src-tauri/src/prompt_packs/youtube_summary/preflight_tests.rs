use super::test_support::*;
use super::{preflight_youtube_summary_in_pool, ModelBudget};

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
    request.runtime_provider = crate::prompt_packs::dto::PromptPackRuntimeProvider::GeminiBrowser;
    request.model_override = None;

    let response = preflight_youtube_summary_in_pool(
        &pool,
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
