use super::harness::{
    corpus_request, insert_youtube_transcript_segment,
    insert_youtube_video_source_with_typed_metadata, rebuild_documents_for_sources, snapshot_pool,
};
use crate::analysis::corpus::{
    AnalysisRunPreflight, AnalysisRunPreflightLimits, YoutubeCorpusMode,
    estimate_message_input_chars, estimate_preflight_chunk_count, load_corpus_messages,
    model_limit_preflight_error, preflight_analysis_run, preflight_limit_error,
};
use crate::analysis::models::CorpusMessage;
use crate::compression::compress_text;
#[test]
fn estimated_message_chars_match_report_chunk_accounting() {
    let message = CorpusMessage {
        item_id: 11,
        source_id: 2,
        external_id: "100".to_string(),
        published_at: 1_710_000_000,
        author: Some("Alice".to_string()),
        content: "First live document".to_string(),
        r#ref: "s2-i11".to_string(),
        item_kind: Some("telegram_message".to_string()),
        source_type: Some("telegram".to_string()),
        source_subtype: Some("channel".to_string()),
        metadata_zstd: None,
    };

    assert_eq!(
        estimate_message_input_chars(
            &message.content,
            &message.r#ref,
            message.author.as_deref()
        ),
        message.content.len() + message.r#ref.len() + "Alice".len() + 64
    );

#[test]
fn estimated_chunk_count_matches_chunk_boundary_behavior() {
    assert_eq!(estimate_preflight_chunk_count(&[], 16_000), 0);
    assert_eq!(estimate_preflight_chunk_count(&[8_000, 7_000], 16_000), 1);
    assert_eq!(estimate_preflight_chunk_count(&[8_000, 9_000], 16_000), 2);
    assert_eq!(estimate_preflight_chunk_count(&[20_000], 16_000), 1);
}

#[test]
fn default_preflight_limits_are_conservative() {
    let limits = AnalysisRunPreflightLimits::default();

    assert_eq!(limits.max_messages_per_run, 10_000);
    assert_eq!(limits.max_chunks_per_run, 80);
    assert_eq!(limits.max_estimated_input_chars_per_run, 1_500_000);
    assert_eq!(limits.max_background_requests_per_run, 80);
}

#[test]
fn preflight_limit_error_reports_all_scale_dimensions() {
    let preflight = AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 73_102,
        estimated_input_chars: 6_200_000,
        estimated_chunks: 381,
        limits: AnalysisRunPreflightLimits::default(),
    };

    let error = preflight_limit_error(&preflight).expect("limit error");

    assert!(error.contains("73102 documents"));
    assert!(error.contains("381 estimated chunks"));
    assert!(error.contains("6200000 estimated input characters"));
    assert!(error.contains("Narrow the period or choose a smaller source scope"));
}

#[test]
fn preflight_limit_error_allows_runs_within_limits() {
    let preflight = AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 1_000,
        estimated_input_chars: 100_000,
        estimated_chunks: 10,
        limits: AnalysisRunPreflightLimits::default(),
    };

    assert_eq!(preflight_limit_error(&preflight), None);
}

#[test]
fn model_limit_preflight_allows_unknown_or_fitting_limits() {
    let preflight = AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 1_000,
        estimated_input_chars: 120_000,
        estimated_chunks: 3,
        limits: AnalysisRunPreflightLimits::default(),
    };

    assert_eq!(model_limit_preflight_error(&preflight, None), None);
    assert_eq!(model_limit_preflight_error(&preflight, Some(40_000)), None);
}

#[test]
fn model_limit_preflight_reports_oversized_chunks() {
    let preflight = AnalysisRunPreflight {
        source_ids: vec![1],
        message_count: 1_000,
        estimated_input_chars: 120_001,
        estimated_chunks: 3,
        limits: AnalysisRunPreflightLimits::default(),
    };

    let error =
        model_limit_preflight_error(&preflight, Some(40_000)).expect("model limit error");

    assert!(error.contains("40001 estimated input characters per chunk"));
    assert!(error.contains("model input limit 40000"));
    assert!(error.contains("Choose a model with a larger context window"));
}

#[tokio::test]
async fn preflight_counts_eligible_text_messages_for_sources() {
    let pool = snapshot_pool().await;
    let first_content = compress_text("First live document").expect("compress first");
    let second_content = compress_text("Second live document").expect("compress second");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind("Alice")
    .bind(1_710_000_000_i64)
    .bind(first_content)
    .execute(&pool)
    .await
    .expect("insert first item");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(12_i64)
    .bind(4_i64)
    .bind("101")
    .bind(Option::<String>::None)
    .bind(1_710_000_100_i64)
    .bind(second_content)
    .execute(&pool)
    .await
    .expect("insert second item");
    rebuild_documents_for_sources(&pool, &[2, 4]).await;

    let preflight = preflight_analysis_run(
        &pool,
        &corpus_request(
            "telegram",
            vec![2, 4],
            YoutubeCorpusMode::TranscriptDescription,
        ),
        16_000,
        AnalysisRunPreflightLimits::default(),
    )
    .await
    .expect("preflight");

    assert_eq!(preflight.source_ids, vec![2, 4]);
    assert_eq!(preflight.message_count, 2);
    assert_eq!(preflight.estimated_chunks, 1);
    assert!(preflight.estimated_input_chars > 0);
}

#[tokio::test]
async fn preflight_ignores_media_only_items_without_text_content() {
    let pool = snapshot_pool().await;
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, NULL)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind("Alice")
    .bind(1_710_000_000_i64)
    .execute(&pool)
    .await
    .expect("insert media-only item");
    rebuild_documents_for_sources(&pool, &[2]).await;

    let preflight = preflight_analysis_run(
        &pool,
        &corpus_request(
            "telegram",
            vec![2],
            YoutubeCorpusMode::TranscriptDescription,
        ),
        16_000,
        AnalysisRunPreflightLimits::default(),
    )
    .await
    .expect("preflight");

    assert_eq!(preflight.message_count, 0);
    assert_eq!(preflight.estimated_chunks, 0);
    assert_eq!(preflight.estimated_input_chars, 0);
}

#[tokio::test]
async fn preflight_count_matches_loader_for_youtube_corpus_modes() {
    let pool = snapshot_pool().await;
    insert_youtube_video_source_with_typed_metadata(
        &pool,
        20,
        "video1",
        "Video 1",
        Some("Description body"),
        Some("2026-05-01"),
    )
    .await;
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?, ?), (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(21_i64)
    .bind(20_i64)
    .bind("transcript:v1:en:manual")
    .bind("youtube_transcript")
    .bind("Channel")
    .bind(1_710_000_000_i64)
    .bind(compress_text("Transcript text").expect("compress transcript"))
    .bind(22_i64)
    .bind(20_i64)
    .bind("comment:c1")
    .bind("youtube_comment")
    .bind("Commenter")
    .bind(1_710_000_001_i64)
    .bind(compress_text("Comment text").expect("compress comment"))
    .execute(&pool)
    .await
    .expect("insert youtube items");
    insert_youtube_transcript_segment(&pool, 21, 20, 754_000, "Transcript text").await;
    rebuild_documents_for_sources(&pool, &[20]).await;

    for mode in [
        YoutubeCorpusMode::TranscriptOnly,
        YoutubeCorpusMode::TranscriptDescription,
        YoutubeCorpusMode::TranscriptDescriptionComments,
    ] {
        let request = corpus_request("youtube", vec![20], mode);
        let corpus = load_corpus_messages(&pool, &request)
            .await
            .expect("load corpus");
        let preflight = preflight_analysis_run(
            &pool,
            &request,
            16_000,
            AnalysisRunPreflightLimits::default(),
        )
        .await
        .expect("preflight");

        assert_eq!(preflight.message_count, corpus.len(), "mode {mode:?}");
    }
}
