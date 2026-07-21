use super::{create_youtube_summary_run_skeleton_in_pool, LlmCompletion};
use crate::dto::{
    PreflightYoutubeSummaryRunRequest, PromptPackRuntimeProvider, StartYoutubeSummaryRunRequest,
};
use crate::seed::seed_builtin_prompt_packs_in_pool;
use crate::source_port::{
    CommentBodyReadRequest, CommentCandidateReadRequest, PromptPackCommentCandidate,
    PromptPackPlaylistItemRecord, PromptPackPortFuture, PromptPackSourceReader,
    PromptPackSourceRecord, PromptPackTranscriptSegment, PromptPackYoutubeVideoRecord,
    YoutubeVideoReadRequest,
};
use crate::test_schema::prompt_pack_test_pool;
use extractum_core::compression::{compress_text, decompress_text};
use extractum_core::error::AppError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct TestPromptPackSourceReader {
    pool: sqlx::SqlitePool,
}

impl TestPromptPackSourceReader {
    pub(crate) fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }
}

impl PromptPackSourceReader for TestPromptPackSourceReader {
    fn load_source(
        &self,
        source_id: i64,
    ) -> PromptPackPortFuture<'_, Option<PromptPackSourceRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, (i64, String, Option<String>, Option<String>)>(
                "SELECT id, source_type, source_subtype, title FROM sources WHERE id = ?",
            )
            .bind(source_id)
            .fetch_optional(&pool)
            .await
            .map(|row| {
                row.map(|(id, source_type, source_subtype, title)| {
                    PromptPackSourceRecord::new(id, source_type, source_subtype, title)
                })
            })
            .map_err(AppError::database)
        })
    }

    fn load_video(
        &self,
        request: YoutubeVideoReadRequest,
    ) -> PromptPackPortFuture<'_, Option<PromptPackYoutubeVideoRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<
                _,
                (
                    i64,
                    String,
                    String,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                ),
            >(
                "SELECT yvs.source_id, yvs.video_id, yvs.canonical_url, yvs.title,
                        yvs.channel_title, yvs.published_at, yvs.description
                 FROM youtube_video_sources yvs
                 JOIN sources ON sources.id = yvs.source_id
                 WHERE yvs.source_id = ?",
            )
            .bind(request.source_id())
            .fetch_optional(&pool)
            .await
            .map(|row| {
                row.map(
                    |(
                        source_id,
                        video_id,
                        canonical_url,
                        title,
                        channel_title,
                        published_at,
                        description,
                    )| {
                        PromptPackYoutubeVideoRecord::new(
                            source_id,
                            video_id,
                            canonical_url,
                            title,
                            channel_title,
                            published_at,
                            description,
                        )
                    },
                )
            })
            .map_err(AppError::database)
        })
    }

    fn load_playlist_items(
        &self,
        playlist_source_id: i64,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackPlaylistItemRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, (Option<i64>, String, Option<String>)>(
                "SELECT video_source_id, video_id, title_snapshot
                 FROM youtube_playlist_items
                 WHERE playlist_source_id = ? AND is_removed_from_playlist = 0
                 ORDER BY position ASC, id ASC",
            )
            .bind(playlist_source_id)
            .fetch_all(&pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(video_source_id, video_id, title)| {
                        PromptPackPlaylistItemRecord::new(video_source_id, video_id, title)
                    })
                    .collect()
            })
            .map_err(AppError::database)
        })
    }

    fn load_transcript_segments(
        &self,
        source_id: i64,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackTranscriptSegment>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, (i64, i64, String)>(
                "SELECT start_ms, end_ms, text
                 FROM youtube_transcript_segments
                 WHERE source_id = ?
                 ORDER BY segment_index ASC, id ASC",
            )
            .bind(source_id)
            .fetch_all(&pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(start_ms, end_ms, text)| {
                        PromptPackTranscriptSegment::new(start_ms, end_ms, text)
                    })
                    .collect()
            })
            .map_err(AppError::database)
        })
    }

    fn select_comment_candidates(
        &self,
        request: CommentCandidateReadRequest,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackCommentCandidate>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let rows = sqlx::query_as::<_, (Option<String>, Option<Vec<u8>>)>(
                "SELECT external_id, content_zstd
                 FROM items
                 WHERE source_id = ? AND item_kind = 'youtube_comment'
                 ORDER BY published_at IS NULL ASC, published_at ASC, external_id ASC, id ASC
                 LIMIT ?",
            )
            .bind(request.source_id())
            .bind(request.limit())
            .fetch_all(&pool)
            .await
            .map_err(AppError::database)?;

            Ok(rows
                .into_iter()
                .map(|(external_id, content_zstd)| {
                    let body = content_zstd
                        .as_deref()
                        .and_then(|bytes| decompress_text(bytes).ok())
                        .unwrap_or_default();
                    PromptPackCommentCandidate::new(external_id, body)
                })
                .collect())
        })
    }

    fn load_comment_body(
        &self,
        request: CommentBodyReadRequest,
    ) -> PromptPackPortFuture<'_, String> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let Some(external_id) = request.external_id() else {
                return Ok(String::new());
            };
            let bytes = sqlx::query_scalar::<_, Vec<u8>>(
                "SELECT content_zstd FROM items
                 WHERE source_id = ? AND item_kind = 'youtube_comment' AND external_id = ?
                 LIMIT 1",
            )
            .bind(request.source_id())
            .bind(external_id)
            .fetch_optional(&pool)
            .await
            .map_err(AppError::database)?;
            Ok(bytes
                .as_deref()
                .and_then(|bytes| decompress_text(bytes).ok())
                .unwrap_or_default())
        })
    }
}

pub(crate) async fn migrated_pool() -> sqlx::SqlitePool {
    prompt_pack_test_pool().await
}

pub(crate) fn request_for_video(source_id: i64) -> PreflightYoutubeSummaryRunRequest {
    PreflightYoutubeSummaryRunRequest::new(
        None,
        vec![source_id],
        None,
        Some("test-model".to_string()),
        PromptPackRuntimeProvider::Api,
        None,
        "en".to_string(),
        "standard".to_string(),
        "standard".to_string(),
        false,
    )
}

pub(crate) fn start_request(
    client_request_id: &str,
    source_ids: Vec<i64>,
) -> StartYoutubeSummaryRunRequest {
    StartYoutubeSummaryRunRequest::new(
        client_request_id.to_string(),
        None,
        source_ids,
        None,
        Some("test-model".to_string()),
        PromptPackRuntimeProvider::Api,
        None,
        "en".to_string(),
        "standard".to_string(),
        "standard".to_string(),
        false,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SourceReadCall {
    LoadSource(i64),
    LoadVideo(i64),
    LoadPlaylistItems(i64),
    LoadTranscriptSegments(i64),
    SelectCommentCandidates {
        source_id: i64,
        limit: i64,
    },
    LoadCommentBody {
        source_id: i64,
        external_id: Option<String>,
    },
}

#[derive(Clone)]
pub(crate) struct ScriptedPromptPackSourceReader {
    calls: Arc<Mutex<Vec<SourceReadCall>>>,
    sources: HashMap<i64, PromptPackSourceRecord>,
    videos: HashMap<i64, PromptPackYoutubeVideoRecord>,
    playlist_items: HashMap<i64, Vec<PromptPackPlaylistItemRecord>>,
    transcripts: HashMap<i64, Vec<PromptPackTranscriptSegment>>,
    comment_candidates: HashMap<i64, Vec<PromptPackCommentCandidate>>,
    comment_bodies: HashMap<(i64, Option<String>), String>,
}

impl ScriptedPromptPackSourceReader {
    pub(crate) fn ready_video(
        source_id: i64,
        transcript_segments: Vec<PromptPackTranscriptSegment>,
    ) -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            sources: HashMap::from([(
                source_id,
                PromptPackSourceRecord::new(
                    source_id,
                    "youtube".to_string(),
                    Some("video".to_string()),
                    Some(format!("Source {source_id}")),
                ),
            )]),
            videos: HashMap::from([(
                source_id,
                PromptPackYoutubeVideoRecord::new(
                    source_id,
                    format!("video-{source_id}"),
                    format!("https://www.youtube.com/watch?v=video-{source_id}"),
                    Some(format!("Video {source_id}")),
                    Some("Scripted channel".to_string()),
                    Some("2026-07-20T10:00:00Z".to_string()),
                    Some("Scripted description".to_string()),
                ),
            )]),
            playlist_items: HashMap::new(),
            transcripts: HashMap::from([(source_id, transcript_segments)]),
            comment_candidates: HashMap::new(),
            comment_bodies: HashMap::new(),
        }
    }

    pub(crate) fn with_comments(
        mut self,
        source_id: i64,
        candidates: Vec<PromptPackCommentCandidate>,
        bodies: Vec<(Option<String>, String)>,
    ) -> Self {
        self.comment_candidates.insert(source_id, candidates);
        self.comment_bodies.extend(
            bodies
                .into_iter()
                .map(|(external_id, body)| ((source_id, external_id), body)),
        );
        self
    }

    pub(crate) fn calls(&self) -> Vec<SourceReadCall> {
        self.calls.lock().expect("source call log").clone()
    }
}

impl PromptPackSourceReader for ScriptedPromptPackSourceReader {
    fn load_source(
        &self,
        source_id: i64,
    ) -> PromptPackPortFuture<'_, Option<PromptPackSourceRecord>> {
        self.calls
            .lock()
            .expect("source call log")
            .push(SourceReadCall::LoadSource(source_id));
        let result = self.sources.get(&source_id).cloned();
        Box::pin(async move { Ok(result) })
    }

    fn load_video(
        &self,
        request: YoutubeVideoReadRequest,
    ) -> PromptPackPortFuture<'_, Option<PromptPackYoutubeVideoRecord>> {
        let source_id = request.source_id();
        self.calls
            .lock()
            .expect("source call log")
            .push(SourceReadCall::LoadVideo(source_id));
        let result = self.videos.get(&source_id).cloned();
        Box::pin(async move { Ok(result) })
    }

    fn load_playlist_items(
        &self,
        playlist_source_id: i64,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackPlaylistItemRecord>> {
        self.calls
            .lock()
            .expect("source call log")
            .push(SourceReadCall::LoadPlaylistItems(playlist_source_id));
        let result = self
            .playlist_items
            .get(&playlist_source_id)
            .cloned()
            .unwrap_or_default();
        Box::pin(async move { Ok(result) })
    }

    fn load_transcript_segments(
        &self,
        source_id: i64,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackTranscriptSegment>> {
        self.calls
            .lock()
            .expect("source call log")
            .push(SourceReadCall::LoadTranscriptSegments(source_id));
        let result = self
            .transcripts
            .get(&source_id)
            .cloned()
            .unwrap_or_default();
        Box::pin(async move { Ok(result) })
    }

    fn select_comment_candidates(
        &self,
        request: CommentCandidateReadRequest,
    ) -> PromptPackPortFuture<'_, Vec<PromptPackCommentCandidate>> {
        let source_id = request.source_id();
        let limit = request.limit();
        self.calls
            .lock()
            .expect("source call log")
            .push(SourceReadCall::SelectCommentCandidates { source_id, limit });
        let mut result = self
            .comment_candidates
            .get(&source_id)
            .cloned()
            .unwrap_or_default();
        result.truncate(limit.max(0) as usize);
        Box::pin(async move { Ok(result) })
    }

    fn load_comment_body(
        &self,
        request: CommentBodyReadRequest,
    ) -> PromptPackPortFuture<'_, String> {
        let source_id = request.source_id();
        let external_id = request.external_id().map(str::to_owned);
        self.calls
            .lock()
            .expect("source call log")
            .push(SourceReadCall::LoadCommentBody {
                source_id,
                external_id: external_id.clone(),
            });
        let result = self
            .comment_bodies
            .get(&(source_id, external_id))
            .cloned()
            .unwrap_or_default();
        Box::pin(async move { Ok(result) })
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

pub(crate) async fn test_pool_with_playlist_two_ready_videos() -> sqlx::SqlitePool {
    let pool = migrated_pool().await;
    seed_builtin_prompt_packs_in_pool(&pool)
        .await
        .expect("seed pack");
    insert_playlist(&pool, 701).await;
    insert_youtube_video(&pool, 901, "v-ready-1").await;
    insert_youtube_video(&pool, 902, "v-ready-2").await;
    insert_transcript(&pool, 901, "Ready transcript one").await;
    insert_transcript(&pool, 902, "Ready transcript two").await;
    insert_playlist_item(&pool, 701, Some(901), "v-ready-1", 1).await;
    insert_playlist_item(&pool, 701, Some(902), "v-ready-2", 2).await;
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

pub(crate) async fn test_pool_with_ready_video_and_comments() -> sqlx::SqlitePool {
    let pool = test_pool_with_ready_video().await;
    insert_comment(&pool, 901, "comment-1", 10, "Useful comment").await;
    insert_comment(&pool, 901, "comment-2", 20, "Second useful comment").await;
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

#[allow(dead_code)]
pub(crate) fn synthesis_json_with_string_readable_items() -> String {
    serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "synthesis_candidate": {
            "summary_text": "Combined summary",
            "cross_video_themes": [
                {
                    "theme_text": "Shared theme",
                    "source_refs": ["source_ref_1", "source_ref_2"],
                    "claim_refs": [],
                    "evidence_refs": []
                }
            ],
            "common_claims": ["Common claim"],
            "contradictions_across_videos": []
        },
        "limitations": ["Limitation"],
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
) -> extractum_core::error::AppResult<()> {
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
    .map_err(extractum_core::error::AppError::database)?;

    assert_eq!(stage_rows.len(), fixtures.len());

    for ((stage_run_id, _source_snapshot_id), fixture) in stage_rows.into_iter().zip(fixtures) {
        super::outputs::execute_transcript_analysis_stage_with_completion(
            pool,
            stage_run_id,
            LlmCompletion {
                text: transcript_analysis_json(fixture.summary, fixture.claim, fixture.evidence),
                input_tokens: Some(10),
                output_tokens: Some(10),
                latency_ms: 5,
            },
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

pub(crate) fn fake_completion_with_malformed_intermediate_candidates_json() -> LlmCompletion {
    LlmCompletion {
        text: serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/transcript_analysis",
            "video_candidate": {
                "summary_text": "Summary",
                "segment_candidates": [],
                "key_point_candidates": [],
                "quote_candidates": { "not": "an array" },
                "action_item_candidates": [],
                "open_question_candidates": []
            },
            "claim_candidates": [
                {
                    "text": "Claim",
                    "material_refs": ["m_source_ref_1_transcript"]
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
