use super::test_support::{
    test_pool_with_comments_out_of_order as app_test_pool_with_comments_out_of_order,
    test_pool_with_ready_video as app_test_pool_with_ready_video,
};
use crate::prompt_packs::source_adapter::AppPromptPackSourceReader;
use extractum_prompt_packs::{CommentCandidateReadRequest, PromptPackSourceReader as _};

#[tokio::test]
async fn transcript_text_for_source_uses_segment_renderer() {
    let pool = app_test_pool_with_ready_video().await;
    let source = AppPromptPackSourceReader::new(pool);

    let segments = source
        .load_transcript_segments(901)
        .await
        .expect("segments");
    let rendered = segments
        .iter()
        .map(|segment| segment.text())
        .collect::<Vec<_>>()
        .join("\n");
    let reread = source
        .load_transcript_segments(901)
        .await
        .expect("segments reread");
    let adapter_text = reread
        .iter()
        .map(|segment| segment.text())
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(adapter_text, rendered);
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
