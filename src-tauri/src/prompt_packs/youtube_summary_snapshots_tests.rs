use super::seed::seed_builtin_prompt_packs_in_pool;
use super::youtube_summary::start_youtube_summary_run_in_pool;
use super::youtube_summary_snapshots::{
    create_youtube_summary_run_skeleton_in_pool, freeze_comment_material_refs, test_comment_policy,
};
use super::youtube_summary_test_support::*;

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
