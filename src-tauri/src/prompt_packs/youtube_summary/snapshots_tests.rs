use super::app_test_support::{
    test_pool_with_comments_out_of_order as app_test_pool_with_comments_out_of_order,
    test_pool_with_ready_video as app_test_pool_with_ready_video,
};
use super::render_transcript_snapshot_text as app_render_transcript_snapshot_text;
use super::snapshots::{
    freeze_comment_material_refs as app_freeze_comment_material_refs,
    test_comment_policy as app_test_comment_policy,
};
use crate::prompt_packs::source_adapter::AppPromptPackSourceReader;
use crate::prompt_packs::source_port::PromptPackSourceReader as _;

#[tokio::test]
async fn transcript_text_for_source_uses_segment_renderer() {
    let pool = app_test_pool_with_ready_video().await;
    let source = AppPromptPackSourceReader::new(pool);

    let segments = source
        .load_transcript_segments(901)
        .await
        .expect("segments");
    let rendered = app_render_transcript_snapshot_text(&segments);
    let reread = source
        .load_transcript_segments(901)
        .await
        .expect("segments reread");
    let adapter_text = app_render_transcript_snapshot_text(&reread);

    assert_eq!(adapter_text, rendered);
}

#[tokio::test]
async fn comment_snapshot_selection_is_deterministic_when_enabled() {
    let pool = app_test_pool_with_comments_out_of_order().await;
    let source = AppPromptPackSourceReader::new(pool);

    let first = app_freeze_comment_material_refs(&source, 901, app_test_comment_policy())
        .await
        .expect("first freeze");
    let second = app_freeze_comment_material_refs(&source, 901, app_test_comment_policy())
        .await
        .expect("second freeze");

    assert_eq!(first, second);
    assert_eq!(first[0].external_id.as_deref(), Some("comment-oldest"));
}

include!("domain_snapshots_tests.rs");
