use super::snapshots::{
    create_youtube_summary_run_skeleton_in_pool, freeze_comment_material_refs, test_comment_policy,
};
use super::sources::{
    render_transcript_snapshot_text, transcript_snapshot_segments_for_source,
    transcript_text_for_source,
};
use super::test_support::*;
use super::{
    start_youtube_summary_run_in_pool, start_youtube_summary_run_with_preflight_failures_in_pool,
};
use crate::compression::decompress_text;
use crate::gemini_browser::{GeminiBrowserProviderConfig, GeminiBrowserProviderMode};
use crate::prompt_packs::dto::{PromptPackRuntimeProvider, YoutubeSummaryPreflightFailure};
use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;

#[tokio::test]
async fn start_freezes_one_canonical_video_snapshot_with_multiple_origins() {
    let pool = test_pool_with_same_video_selected_explicitly_and_from_playlist().await;
    let request = start_request("req-freeze-1", vec![901, 701]);

    let run_id = create_youtube_summary_run_skeleton_in_pool(&pool, request, 10)
        .await
        .expect("create run");

    let snapshot_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_run_source_snapshots WHERE run_id = ?",
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("snapshot count");

    let origin_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_run_source_origins WHERE run_id = ?")
            .bind(run_id)
            .fetch_one(&pool)
            .await
            .expect("origin count");

    assert_eq!(snapshot_count, 1);
    assert_eq!(origin_count, 2);
}

#[tokio::test]
async fn start_returns_existing_run_for_duplicate_client_request_id() {
    let pool = test_pool_with_ready_video().await;
    let request = start_request("req-duplicate-start", vec![901]);

    let first = start_youtube_summary_run_in_pool(&pool, request.clone())
        .await
        .expect("first start")
        .expect_started("first start returns a run");
    let second = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("duplicate start")
        .expect_started("duplicate start returns existing run");

    assert_eq!(first.run_id, second.run_id);

    let run_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_runs WHERE client_request_id = 'req-duplicate-start'",
    )
    .fetch_one(&pool)
    .await
    .expect("run count");
    assert_eq!(run_count, 1);
}

#[tokio::test]
async fn start_with_recomputed_blocking_preflight_returns_response_without_run() {
    let pool = test_pool_with_youtube_video_without_transcript().await;
    seed_builtin_prompt_packs_in_pool(&pool)
        .await
        .expect("seed pack");
    let request = start_request("req-blocked-start", vec![901]);

    let outcome = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("start command returns structured blocking response");

    let blocking = outcome.expect_blocked("blocking response");
    assert!(blocking.included_videos.is_empty());
    assert_eq!(blocking.blocking_failures[0].reason, "no_usable_transcript");

    let run_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_runs WHERE client_request_id = 'req-blocked-start'",
    )
    .fetch_one(&pool)
    .await
    .expect("run count");
    assert_eq!(run_count, 0);
}

#[tokio::test]
async fn start_with_runtime_blocking_failure_returns_preflight_without_run() {
    let pool = test_pool_with_ready_video().await;
    let mut request = start_request("req-browser-runtime-blocked", vec![901]);
    request.runtime_provider = PromptPackRuntimeProvider::GeminiBrowser;
    request.profile_id = None;
    request.model_override = None;

    let outcome = start_youtube_summary_run_with_preflight_failures_in_pool(
        &pool,
        request,
        vec![YoutubeSummaryPreflightFailure {
            source_id: None,
            reason: "browser_provider_not_ready".to_string(),
            message: Some("Gemini Browser Provider needs login.".to_string()),
        }],
    )
    .await
    .expect("start command returns browser blocking response");

    let blocking = outcome.expect_blocked("browser blocking response");
    assert_eq!(blocking.included_videos.len(), 1);
    assert_eq!(
        blocking.blocking_failures[0].reason,
        "browser_provider_not_ready"
    );

    let run_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_runs WHERE client_request_id = 'req-browser-runtime-blocked'",
    )
    .fetch_one(&pool)
    .await
    .expect("run count");
    assert_eq!(run_count, 0);
}

#[tokio::test]
async fn duplicate_start_ignores_runtime_blocking_failure() {
    let pool = test_pool_with_ready_video().await;
    let mut request = start_request("req-browser-runtime-duplicate-blocked", vec![901]);
    request.runtime_provider = PromptPackRuntimeProvider::GeminiBrowser;
    request.profile_id = None;
    request.model_override = None;

    let first = start_youtube_summary_run_in_pool(&pool, request.clone())
        .await
        .expect("first start")
        .expect_started("first start");
    let second = start_youtube_summary_run_with_preflight_failures_in_pool(
        &pool,
        request,
        vec![YoutubeSummaryPreflightFailure {
            source_id: None,
            reason: "browser_provider_not_ready".to_string(),
            message: Some("Gemini Browser Provider needs login.".to_string()),
        }],
    )
    .await
    .expect("duplicate start")
    .expect_started("duplicate start returns existing run");

    assert_eq!(first.run_id, second.run_id);

    let run_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_runs WHERE client_request_id = 'req-browser-runtime-duplicate-blocked'",
    )
    .fetch_one(&pool)
    .await
    .expect("run count");
    assert_eq!(run_count, 1);
}

#[tokio::test]
async fn transcript_snapshot_text_is_rendered_from_structured_segments() {
    let pool = test_pool_with_ready_video().await;
    let request = start_request("req-transcript-segments", vec![901]);

    let run = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("start")
        .expect_started("started");

    let (text_zstd, metadata_json_zstd): (Vec<u8>, Vec<u8>) = sqlx::query_as(
        "SELECT text_zstd, metadata_json_zstd
         FROM prompt_pack_run_material_snapshots
         WHERE run_id = ? AND material_kind = 'transcript'",
    )
    .bind(run.run_id)
    .fetch_one(&pool)
    .await
    .expect("transcript material");

    let text = decompress_text(&text_zstd).expect("text");
    let metadata = decompress_text(&metadata_json_zstd).expect("metadata");
    let value: serde_json::Value = serde_json::from_str(&metadata).expect("metadata json");
    let segments = value["segments"].as_array().expect("segments");

    let joined = segments
        .iter()
        .map(|segment| segment["text"].as_str().expect("segment text"))
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(text, joined);
    assert_eq!(segments[0]["start_ms"], serde_json::json!(0));
    assert!(segments[0]["end_ms"].as_i64().unwrap_or_default() >= 0);
}

#[tokio::test]
async fn transcript_text_for_source_uses_segment_renderer() {
    let pool = test_pool_with_ready_video().await;

    let segments = transcript_snapshot_segments_for_source(&pool, 901)
        .await
        .expect("segments");
    let rendered = render_transcript_snapshot_text(&segments);
    let legacy_text = transcript_text_for_source(&pool, 901).await.expect("text");

    assert_eq!(legacy_text, rendered);
}

#[tokio::test]
async fn start_persists_gemini_browser_runtime_and_config_snapshot() {
    let pool = test_pool_with_ready_video().await;
    let mut request = start_request("req-browser-runtime-start", vec![901]);
    request.runtime_provider = PromptPackRuntimeProvider::GeminiBrowser;
    request.profile_id = None;
    request.model_override = None;
    request.browser_provider_config = Some(GeminiBrowserProviderConfig {
        mode: GeminiBrowserProviderMode::CdpAttach,
        cdp_endpoint: Some("http://127.0.0.1:9222".to_string()),
    });

    let run = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect("start browser runtime")
        .expect_started("browser runtime run");

    assert_eq!(run.runtime_provider, "gemini_browser");

    let (runtime_provider, browser_config_json, request_json_zstd): (
        String,
        Option<String>,
        Vec<u8>,
    ) = sqlx::query_as(
        "SELECT runtime_provider, browser_provider_config_json, request_json_zstd
         FROM prompt_pack_runs
         WHERE id = ?",
    )
    .bind(run.run_id)
    .fetch_one(&pool)
    .await
    .expect("runtime row");

    assert_eq!(runtime_provider, "gemini_browser");
    let browser_config_json = browser_config_json.expect("browser config json");
    assert!(browser_config_json.contains("\"mode\":\"cdp_attach\""));
    assert!(browser_config_json.contains("127.0.0.1:9222"));

    let request_json = decompress_text(&request_json_zstd).expect("decompress request");
    assert!(request_json.contains("\"runtimeProvider\":\"gemini_browser\""));
    assert!(request_json.contains("\"browserProviderConfig\""));
}

#[tokio::test]
async fn duplicate_client_request_id_preserves_existing_runtime_provider() {
    let pool = test_pool_with_ready_video().await;
    let mut browser_request = start_request("req-runtime-idempotent", vec![901]);
    browser_request.runtime_provider = PromptPackRuntimeProvider::GeminiBrowser;
    browser_request.profile_id = None;
    browser_request.model_override = None;

    let first = start_youtube_summary_run_in_pool(&pool, browser_request)
        .await
        .expect("first start")
        .expect_started("first start");

    let api_request = start_request("req-runtime-idempotent", vec![901]);
    let second = start_youtube_summary_run_in_pool(&pool, api_request)
        .await
        .expect("second start")
        .expect_started("second start");

    assert_eq!(first.run_id, second.run_id);
    assert_eq!(second.runtime_provider, "gemini_browser");
}

#[tokio::test]
async fn comment_snapshot_selection_is_deterministic_when_enabled() {
    let pool = test_pool_with_comments_out_of_order().await;

    let first = freeze_comment_material_refs(&pool, 901, test_comment_policy())
        .await
        .expect("first freeze");
    let second = freeze_comment_material_refs(&pool, 901, test_comment_policy())
        .await
        .expect("second freeze");

    assert_eq!(first, second);
    assert_eq!(first[0].external_id.as_deref(), Some("comment-oldest"));
}
