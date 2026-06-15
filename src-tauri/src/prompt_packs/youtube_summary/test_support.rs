use super::{create_youtube_summary_run_skeleton_in_pool, now_string, LlmCompletion};
use crate::compression::compress_text;
use crate::migrations::apply_all_migrations_for_test_pool;
use crate::prompt_packs::dto::{PreflightYoutubeSummaryRunRequest, StartYoutubeSummaryRunRequest};
use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;

pub(crate) async fn migrated_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");
    pool
}

pub(crate) fn request_for_video(source_id: i64) -> PreflightYoutubeSummaryRunRequest {
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

pub(crate) fn start_request(
    client_request_id: &str,
    source_ids: Vec<i64>,
) -> StartYoutubeSummaryRunRequest {
    StartYoutubeSummaryRunRequest {
        client_request_id: client_request_id.to_string(),
        project_id: None,
        source_ids,
        profile_id: None,
        model_override: Some("test-model".to_string()),
        output_language: "en".to_string(),
        control_preset: "standard".to_string(),
        evidence_mode: "standard".to_string(),
        include_comments: false,
    }
}

pub(crate) fn request_for_playlist(source_id: i64) -> PreflightYoutubeSummaryRunRequest {
    request_for_video(source_id)
}

pub(crate) async fn insert_youtube_video(pool: &sqlx::SqlitePool, source_id: i64, video_id: &str) {
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

pub(crate) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
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

pub(crate) async fn insert_playlist_item(
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

pub(crate) async fn insert_transcript(pool: &sqlx::SqlitePool, source_id: i64, text: &str) {
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

pub(crate) async fn insert_comment(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    external_id: &str,
    published_at: i64,
    text: &str,
) {
    sqlx::query(
        "INSERT INTO items (
            source_id, external_id, author, published_at, ingested_at,
            content_zstd, content_kind, has_media, item_kind
         )
         VALUES (?, ?, 'Alice', ?, 1, ?, 'text_only', 0, 'youtube_comment')",
    )
    .bind(source_id)
    .bind(external_id)
    .bind(published_at)
    .bind(compress_text(text).expect("compress comment"))
    .execute(pool)
    .await
    .expect("insert comment");
}

pub(crate) async fn test_pool_with_youtube_video_without_transcript() -> sqlx::SqlitePool {
    let pool = migrated_pool().await;
    insert_youtube_video(&pool, 901, "v-missing").await;
    pool
}

pub(crate) async fn test_pool_with_playlist_one_ready_one_missing_transcript() -> sqlx::SqlitePool {
    let pool = migrated_pool().await;
    insert_playlist(&pool, 701).await;
    insert_youtube_video(&pool, 901, "v-ready").await;
    insert_youtube_video(&pool, 902, "v-missing").await;
    insert_transcript(&pool, 901, "Ready transcript").await;
    insert_playlist_item(&pool, 701, Some(901), "v-ready", 1).await;
    insert_playlist_item(&pool, 701, Some(902), "v-missing", 2).await;
    pool
}

pub(crate) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    let pool = migrated_pool().await;
    seed_builtin_prompt_packs_in_pool(&pool)
        .await
        .expect("seed pack");
    insert_youtube_video(&pool, 901, "v-ready").await;
    insert_transcript(&pool, 901, "Ready transcript").await;
    pool
}

pub(crate) async fn test_pool_with_same_video_selected_explicitly_and_from_playlist(
) -> sqlx::SqlitePool {
    let pool = migrated_pool().await;
    seed_builtin_prompt_packs_in_pool(&pool)
        .await
        .expect("seed pack");
    insert_playlist(&pool, 701).await;
    insert_youtube_video(&pool, 901, "v-ready").await;
    insert_transcript(&pool, 901, "Ready transcript").await;
    insert_playlist_item(&pool, 701, Some(901), "v-ready", 1).await;
    pool
}

pub(crate) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    let pool = test_pool_with_ready_video().await;
    insert_comment(&pool, 901, "comment-newer", 20, "newer").await;
    insert_comment(&pool, 901, "comment-oldest", 10, "oldest").await;
    insert_comment(&pool, 901, "comment-middle", 15, "middle").await;
    pool
}

pub(crate) async fn test_pool_with_frozen_youtube_summary_run() -> sqlx::SqlitePool {
    let pool = test_pool_with_ready_video().await;
    let run_id = create_youtube_summary_run_skeleton_in_pool(
        &pool,
        start_request("req-execute-1", vec![901]),
        1,
    )
    .await
    .expect("run skeleton");
    assert_eq!(run_id, 1);
    pool
}

pub(crate) async fn test_pool_with_two_frozen_youtube_summary_sources() -> sqlx::SqlitePool {
    let pool = migrated_pool().await;
    seed_builtin_prompt_packs_in_pool(&pool)
        .await
        .expect("seed pack");
    insert_youtube_video(&pool, 901, "v-ready-1").await;
    insert_youtube_video(&pool, 902, "v-ready-2").await;
    insert_transcript(&pool, 901, "Ready transcript one").await;
    insert_transcript(&pool, 902, "Ready transcript two").await;
    create_youtube_summary_run_skeleton_in_pool(
        &pool,
        start_request("req-execute-2", vec![901, 902]),
        1,
    )
    .await
    .expect("run skeleton");
    pool
}

pub(crate) struct TranscriptStageFixture {
    pub(crate) summary: &'static str,
    pub(crate) claim: &'static str,
    pub(crate) evidence: &'static str,
}

pub(crate) fn transcript_analysis_json(summary: &str, claim: &str, evidence: &str) -> String {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/transcript_analysis",
        "video_candidate": {
            "summary_text": summary,
            "segment_candidates": [],
            "key_point_candidates": [],
            "quote_candidates": [],
            "action_item_candidates": [],
            "open_question_candidates": []
        },
        "claim_candidates": [
            {
                "text": claim
            }
        ],
        "evidence_fragment_candidates": [
            {
                "text": evidence
            }
        ],
        "warning_candidates": []
    })
    .to_string()
}

#[allow(dead_code)]
pub(crate) fn synthesis_json(summary: &str) -> String {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "synthesis_candidate": {
            "summary_text": summary,
            "cross_video_themes": [
                {
                    "theme_text": "Shared theme",
                    "source_refs": ["source_ref_1", "source_ref_2"],
                    "claim_refs": [],
                    "evidence_refs": []
                }
            ],
            "common_claims": [],
            "contradictions_across_videos": []
        },
        "limitations": [],
        "warning_candidates": []
    })
    .to_string()
}

pub(crate) fn synthesis_json_with_backend_owned_id() -> String {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "synthesis_candidate": {
            "summary_text": "Invalid synthesis",
            "cross_video_themes": [
                {
                    "theme_id": "theme_from_provider",
                    "theme_text": "Provider must not assign backend IDs",
                    "source_refs": ["source_ref_1", "source_ref_2"],
                    "claim_refs": [],
                    "evidence_refs": []
                }
            ],
            "common_claims": [],
            "contradictions_across_videos": []
        },
        "limitations": [],
        "warning_candidates": []
    })
    .to_string()
}

pub(crate) async fn persist_succeeded_transcript_stage_fixtures(
    pool: &sqlx::SqlitePool,
    run_id: i64,
    fixtures: Vec<TranscriptStageFixture>,
) -> crate::error::AppResult<()> {
    let stage_rows = sqlx::query_as::<_, (i64, i64)>(
        "SELECT id, source_snapshot_id
         FROM prompt_pack_stage_runs
         WHERE run_id = ?
           AND stage_name = 'youtube_summary/transcript_analysis'
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(crate::error::AppError::database)?;

    assert_eq!(stage_rows.len(), fixtures.len());

    for ((stage_run_id, _source_snapshot_id), fixture) in stage_rows.into_iter().zip(fixtures) {
        sqlx::query(
            "UPDATE prompt_pack_stage_runs
             SET stage_status = 'succeeded', updated_at = ?
             WHERE id = ?",
        )
        .bind(now_string())
        .bind(stage_run_id)
        .execute(pool)
        .await
        .map_err(crate::error::AppError::database)?;

        let parsed = transcript_analysis_json(fixture.summary, fixture.claim, fixture.evidence);
        crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
            pool,
            run_id,
            stage_run_id,
            "parsed_output",
            1,
            3,
            &parsed,
        )
        .await?;
    }

    Ok(())
}

pub(crate) async fn transcript_analysis_stage_id(pool: &sqlx::SqlitePool, run_id: i64) -> i64 {
    sqlx::query_scalar(
        "SELECT id FROM prompt_pack_stage_runs
         WHERE run_id = ? AND stage_name = 'youtube_summary/transcript_analysis'
         ORDER BY id ASC LIMIT 1",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .expect("stage id")
}

pub(crate) async fn list_stage_artifact_kinds(
    pool: &sqlx::SqlitePool,
    stage_id: i64,
) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT artifact_kind FROM prompt_pack_stage_artifacts
         WHERE stage_run_id = ?
         ORDER BY attempt_number ASC, artifact_index ASC",
    )
    .bind(stage_id)
    .fetch_all(pool)
    .await
    .expect("artifact kinds")
}

pub(crate) async fn list_stage_artifact_attempts(
    pool: &sqlx::SqlitePool,
    stage_id: i64,
) -> Vec<(String, i64, i64)> {
    sqlx::query_as(
        "SELECT artifact_kind, attempt_number, artifact_index
         FROM prompt_pack_stage_artifacts
         WHERE stage_run_id = ?
         ORDER BY attempt_number ASC, artifact_index ASC",
    )
    .bind(stage_id)
    .fetch_all(pool)
    .await
    .expect("artifact attempts")
}

pub(crate) fn fake_completion_with_valid_transcript_analysis_json() -> LlmCompletion {
    fake_completion_with_valid_transcript_analysis_json_for_source("source_ref_1")
}

pub(crate) fn fake_completion_with_valid_transcript_analysis_json_for_source(
    source_ref_id: &str,
) -> LlmCompletion {
    LlmCompletion {
        text: serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/transcript_analysis",
            "video_candidate": {
                "summary_text": format!("Summary for {source_ref_id}"),
                "segment_candidates": [],
                "key_point_candidates": [],
                "quote_candidates": [],
                "action_item_candidates": [],
                "open_question_candidates": []
            },
            "claim_candidates": [
                {
                    "text": "Claim",
                    "material_refs": [format!("m_{source_ref_id}_transcript")]
                }
            ],
            "evidence_fragment_candidates": [],
            "warning_candidates": []
        })
        .to_string(),
        input_tokens: Some(10),
        output_tokens: Some(20),
        latency_ms: 5,
    }
}

pub(crate) fn malformed_completion() -> LlmCompletion {
    LlmCompletion {
        text: r#"{
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/transcript_analysis",
            "evidence_fragment_candidates":"#
            .to_string(),
        input_tokens: Some(10),
        output_tokens: Some(20),
        latency_ms: 30,
    }
}

pub(crate) fn fake_provider_failure(message: &str) -> String {
    message.to_string()
}
