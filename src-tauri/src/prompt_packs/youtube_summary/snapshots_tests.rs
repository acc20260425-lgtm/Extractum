use super::test_support::{
    test_pool_with_comments_out_of_order as app_test_pool_with_comments_out_of_order,
    test_pool_with_ready_video as app_test_pool_with_ready_video,
};
use crate::prompt_packs::source_adapter::AppPromptPackSourceReader;
use extractum_prompt_packs::{
    CommentCandidateReadRequest, PromptPackSourceReader as _, PromptPackTranscriptSegment,
};

#[tokio::test]
async fn transcript_text_for_source_uses_segment_renderer() {
    let pool = app_test_pool_with_ready_video().await;
    let item_id: i64 = sqlx::query_scalar(
        "SELECT id FROM items WHERE source_id = 901 AND item_kind = 'youtube_transcript'",
    )
    .fetch_one(&pool)
    .await
    .expect("transcript item");
    for (segment_index, start_ms, end_ms, text) in [
        (2_i64, 2_000_i64, 3_000_i64, "segment-two"),
        (1_i64, 1_000_i64, 2_000_i64, "segment-one"),
    ] {
        sqlx::query(
            "INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text
             ) VALUES (?, 901, ?, ?, ?, ?)",
        )
        .bind(item_id)
        .bind(segment_index)
        .bind(start_ms)
        .bind(end_ms)
        .bind(text)
        .execute(&pool)
        .await
        .expect("insert transcript segment");
    }
    let source = AppPromptPackSourceReader::new(pool);

    let segments = source
        .load_transcript_segments(901)
        .await
        .expect("segments");
    assert_eq!(
        segments,
        vec![
            PromptPackTranscriptSegment::new(0, 1_000, "Ready transcript".to_string()),
            PromptPackTranscriptSegment::new(1_000, 2_000, "segment-one".to_string()),
            PromptPackTranscriptSegment::new(2_000, 3_000, "segment-two".to_string()),
        ]
    );
    let rendered = segments
        .iter()
        .map(|segment| segment.text())
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(rendered, "Ready transcript\nsegment-one\nsegment-two");
}

#[tokio::test]
async fn comment_snapshot_selection_is_deterministic_when_enabled() {
    let pool = app_test_pool_with_comments_out_of_order().await;
    let source = AppPromptPackSourceReader::new(pool);

    let first = source
        .select_comment_candidates(CommentCandidateReadRequest::new(901, 100))
        .await
        .expect("first freeze");
    let second = source
        .select_comment_candidates(CommentCandidateReadRequest::new(901, 100))
        .await
        .expect("second freeze");

    assert_eq!(first, second);
    assert_eq!(first[0].external_id(), Some("comment-oldest"));
}
