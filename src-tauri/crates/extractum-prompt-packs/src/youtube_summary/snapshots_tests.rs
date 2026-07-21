use super::snapshots::{
    create_youtube_summary_run_skeleton_with_source, freeze_comment_material_refs,
    CommentSelectionPolicy,
};
use super::test_support::*;
use super::{
    create_youtube_summary_run_skeleton_in_pool, start_youtube_summary_run_in_pool,
    start_youtube_summary_run_with_preflight_failures_in_pool,
    start_youtube_summary_run_with_source,
};
use crate::dto::{PromptPackRuntimeProvider, YoutubeSummaryPreflightFailure};
use crate::seed::seed_builtin_prompt_packs_in_pool;
use crate::source_port::{PromptPackCommentCandidate, PromptPackTranscriptSegment};
use extractum_core::compression::decompress_text;
use extractum_gemini_browser::{GeminiBrowserProviderConfig, GeminiBrowserProviderMode};

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
async fn empty_client_request_id_returns_before_any_database_or_source_read() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect deliberately unmigrated pool");
    let request = start_request("", vec![901]);

    let error = start_youtube_summary_run_in_pool(&pool, request)
        .await
        .expect_err("empty client request id must fail before querying the pool");

    assert_eq!(error.kind, extractum_core::error::AppErrorKind::Validation);
    assert_eq!(error.message, "client_request_id cannot be empty");
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
    let mut duplicate_request = request;
    duplicate_request.source_ids = vec![999_999];
    let second = start_youtube_summary_run_with_preflight_failures_in_pool(
        &pool,
        duplicate_request,
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

#[test]
fn snapshot_start_source_preserves_repeated_preflight_and_post_insert_fresh_reads() {
    let outer_source = include_str!("mod.rs");
    let snapshot_source = include_str!("snapshots.rs");

    let outer_start = outer_source
        .find("async fn start_youtube_summary_run_with_preflight_failures_and_source")
        .expect("outer start function");
    let outer_preflight = outer_source[outer_start..]
        .find("preflight_youtube_summary(")
        .expect("outer preflight");
    let skeleton_call = outer_source[outer_start..]
        .find("create_youtube_summary_run_skeleton_with_source(")
        .expect("skeleton call");
    assert!(outer_preflight < skeleton_call);

    let skeleton_start = snapshot_source
        .find("pub(crate) async fn create_youtube_summary_run_skeleton_with_source")
        .expect("skeleton function");
    let skeleton = &snapshot_source[skeleton_start..];
    let repeated_preflight = skeleton
        .find("preflight_youtube_summary(")
        .expect("repeated skeleton preflight");
    let run_insert = skeleton
        .find("INSERT INTO prompt_pack_runs")
        .expect("run insertion");
    let post_insert_source_read = skeleton
        .find("let Some(source_record) = source.load_source(*source_id).await?")
        .expect("post-insert source read");
    let post_insert_snapshot_read = skeleton
        .find("insert_source_snapshot(pool, source, run_id, video")
        .expect("post-insert video snapshot read");
    let post_insert_material_read = skeleton
        .find("insert_material_snapshots(")
        .expect("post-insert material read");

    assert!(repeated_preflight < run_insert);
    assert!(run_insert < post_insert_source_read);
    assert!(post_insert_source_read < post_insert_snapshot_read);
    assert!(post_insert_snapshot_read < post_insert_material_read);
}

#[test]
fn comment_snapshot_source_reads_candidates_for_estimates_then_selected_bodies_again() {
    let source = include_str!("snapshots.rs");
    let insertion_start = source
        .find("async fn insert_material_snapshots(")
        .expect("material insertion function");
    let insertion = &source[insertion_start..];
    let candidate_read = insertion
        .find("freeze_comment_material_refs(source, source_id")
        .expect("candidate comment read");
    let selected_body_read = insertion
        .find(".load_comment_body(")
        .expect("selected comment body read");

    assert!(candidate_read < selected_body_read);

    let candidate_function = source
        .find("pub(crate) async fn freeze_comment_material_refs(")
        .expect("candidate function");
    let candidate_body = &source[candidate_function..];
    assert!(candidate_body.contains(".select_comment_candidates("));
    assert!(candidate_body.contains("estimate_tokens(candidate.body())"));
    assert!(!candidate_body.contains("SELECT content_zstd FROM items"));
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
async fn gem_analysis_freezes_comments_even_when_include_comments_is_false() {
    let pool = test_pool_with_ready_video_and_comments().await;
    let mut request = start_request("req-gem-analysis-comments-default", vec![901]);
    request.control_preset = "gem_analysis".to_string();
    request.include_comments = false;

    let run_id = create_youtube_summary_run_skeleton_in_pool(&pool, request, 1)
        .await
        .expect("create run");

    let comment_materials: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM prompt_pack_run_material_snapshots
         WHERE run_id = ? AND material_kind = 'comment'",
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("comment material count");

    let include_comments: bool =
        sqlx::query_scalar("SELECT include_comments FROM prompt_pack_runs WHERE id = ?")
            .bind(run_id)
            .fetch_one(&pool)
            .await
            .expect("stored include comments");

    assert!(comment_materials > 0);
    assert!(include_comments);
}

#[tokio::test]
async fn runnable_start_uses_complete_fresh_source_read_sequence() {
    let pool = test_pool_with_ready_video().await;
    let source = ScriptedPromptPackSourceReader::ready_video(
        901,
        vec![PromptPackTranscriptSegment::new(
            0,
            1_000,
            "scripted transcript".to_string(),
        )],
    )
    .with_comments(
        901,
        vec![
            PromptPackCommentCandidate::new(
                Some("comment-1".to_string()),
                "candidate one".to_string(),
            ),
            PromptPackCommentCandidate::new(
                Some("comment-2".to_string()),
                "candidate two".to_string(),
            ),
        ],
        vec![
            (Some("comment-1".to_string()), "fresh body one".to_string()),
            (Some("comment-2".to_string()), "fresh body two".to_string()),
        ],
    );
    let mut request = start_request("req-complete-source-sequence", vec![901]);
    request.include_comments = true;

    start_youtube_summary_run_with_source(&pool, &source, request)
        .await
        .expect("start with scripted source")
        .expect_started("scripted start");

    assert_eq!(
        source.calls(),
        vec![
            SourceReadCall::LoadSource(901),
            SourceReadCall::LoadVideo(901),
            SourceReadCall::LoadTranscriptSegments(901),
            SourceReadCall::LoadSource(901),
            SourceReadCall::LoadVideo(901),
            SourceReadCall::LoadTranscriptSegments(901),
            SourceReadCall::LoadSource(901),
            SourceReadCall::LoadSource(901),
            SourceReadCall::LoadVideo(901),
            SourceReadCall::LoadTranscriptSegments(901),
            SourceReadCall::LoadVideo(901),
            SourceReadCall::SelectCommentCandidates {
                source_id: 901,
                limit: 50,
            },
            SourceReadCall::LoadCommentBody {
                source_id: 901,
                external_id: Some("comment-1".to_string()),
            },
            SourceReadCall::LoadCommentBody {
                source_id: 901,
                external_id: Some("comment-2".to_string()),
            },
            SourceReadCall::LoadSource(901),
            SourceReadCall::LoadVideo(901),
        ]
    );
}

#[tokio::test]
async fn selected_comment_body_is_reloaded_after_candidate_estimation() {
    let pool = test_pool_with_ready_video().await;
    let source = ScriptedPromptPackSourceReader::ready_video(
        901,
        vec![PromptPackTranscriptSegment::new(
            0,
            1_000,
            "scripted transcript".to_string(),
        )],
    )
    .with_comments(
        901,
        vec![PromptPackCommentCandidate::new(
            Some("comment-fresh".to_string()),
            "candidate body used only for estimate".to_string(),
        )],
        vec![(
            Some("comment-fresh".to_string()),
            "fresh body persisted later".to_string(),
        )],
    );
    let mut request = start_request("req-fresh-comment-body", vec![901]);
    request.include_comments = true;

    let run_id = create_youtube_summary_run_skeleton_with_source(&pool, &source, request, 1)
        .await
        .expect("create run with scripted comments");
    let text_zstd: Vec<u8> = sqlx::query_scalar(
        "SELECT text_zstd FROM prompt_pack_run_material_snapshots
         WHERE run_id = ? AND material_kind = 'comment'",
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("read frozen comment");
    let frozen = decompress_text(&text_zstd).expect("decompress frozen comment");

    assert_eq!(frozen, "fresh body persisted later");
    let calls = source.calls();
    let candidate_index = calls
        .iter()
        .position(|call| matches!(call, SourceReadCall::SelectCommentCandidates { .. }))
        .expect("candidate read");
    let body_index = calls
        .iter()
        .position(|call| matches!(call, SourceReadCall::LoadCommentBody { .. }))
        .expect("body read");
    assert!(candidate_index < body_index);
}

#[tokio::test]
async fn transcript_material_policy_uses_owned_segment_reader_values() {
    let pool = test_pool_with_ready_video().await;
    let source = ScriptedPromptPackSourceReader::ready_video(
        901,
        vec![
            PromptPackTranscriptSegment::new(111, 222, "owned first".to_string()),
            PromptPackTranscriptSegment::new(333, 444, "owned second".to_string()),
        ],
    );
    let request = start_request("req-owned-transcript-segments", vec![901]);

    let run_id = create_youtube_summary_run_skeleton_with_source(&pool, &source, request, 1)
        .await
        .expect("create run with owned transcript segments");
    let (text_zstd, metadata_zstd): (Vec<u8>, Vec<u8>) = sqlx::query_as(
        "SELECT text_zstd, metadata_json_zstd
         FROM prompt_pack_run_material_snapshots
         WHERE run_id = ? AND material_kind = 'transcript'",
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("read transcript snapshot");
    let text = decompress_text(&text_zstd).expect("decompress transcript");
    let metadata = decompress_text(&metadata_zstd).expect("decompress transcript metadata");
    let metadata: serde_json::Value = serde_json::from_str(&metadata).expect("metadata JSON");

    assert_eq!(text, "owned first\nowned second");
    assert_eq!(metadata["segments"][0]["start_ms"], 111);
    assert_eq!(metadata["segments"][0]["end_ms"], 222);
    assert_eq!(metadata["segments"][1]["text"], "owned second");
}

#[tokio::test]
async fn comment_material_ref_policy_preserves_order_and_token_cap() {
    let source = ScriptedPromptPackSourceReader::ready_video(901, Vec::new()).with_comments(
        901,
        vec![
            PromptPackCommentCandidate::new(
                Some("comment-first".to_string()),
                "abcdefghijklmnop".to_string(),
            ),
            PromptPackCommentCandidate::new(Some("comment-second".to_string()), "x".to_string()),
        ],
        Vec::new(),
    );

    let refs = freeze_comment_material_refs(
        &source,
        901,
        CommentSelectionPolicy {
            comment_count_cap: 50,
            comment_token_cap: 2,
        },
    )
    .await
    .expect("freeze comment refs");

    assert_eq!(refs[0].external_id.as_deref(), Some("comment-first"));
    assert_eq!(refs[0].material_ref_id, "m_comment_1");
    assert_eq!(refs[0].token_estimate, 2);
    assert_eq!(refs[1].external_id.as_deref(), Some("comment-second"));
    assert_eq!(refs[1].material_ref_id, "m_comment_2");
    assert_eq!(refs[1].token_estimate, 1);
}
