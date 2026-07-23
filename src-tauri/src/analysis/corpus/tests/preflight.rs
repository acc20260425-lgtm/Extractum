use super::super::super::corpus::{
    load_app_corpus_messages, preflight_analysis_corpus,
    AnalysisRunPreflightLimits as AppAnalysisRunPreflightLimits, AppAnalysisCorpusReader,
    YoutubeCorpusMode,
};
use super::harness::{
    corpus_request, insert_youtube_transcript_segment,
    insert_youtube_video_source_with_typed_metadata, rebuild_documents_for_sources, snapshot_pool,
};
use extractum_core::compression::compress_text;
include!("preflight_portable.rs");
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

    let reader = AppAnalysisCorpusReader::new(pool.clone());
    let preflight = preflight_analysis_corpus(
        &reader,
        &corpus_request(
            "telegram",
            vec![2, 4],
            YoutubeCorpusMode::TranscriptDescription,
        ),
        16_000,
        AppAnalysisRunPreflightLimits::default(),
    )
    .await
    .expect("preflight");

    assert_eq!(preflight.source_ids(), &[2, 4]);
    assert_eq!(preflight.message_count(), 2);
    assert_eq!(preflight.estimated_chunks(), 1);
    assert!(preflight.estimated_input_chars() > 0);
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

    let reader = AppAnalysisCorpusReader::new(pool.clone());
    let preflight = preflight_analysis_corpus(
        &reader,
        &corpus_request(
            "telegram",
            vec![2],
            YoutubeCorpusMode::TranscriptDescription,
        ),
        16_000,
        AppAnalysisRunPreflightLimits::default(),
    )
    .await
    .expect("preflight");

    assert_eq!(preflight.message_count(), 0);
    assert_eq!(preflight.estimated_chunks(), 0);
    assert_eq!(preflight.estimated_input_chars(), 0);
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
        let corpus = load_app_corpus_messages(&pool, &request)
            .await
            .expect("load corpus");
        let reader = AppAnalysisCorpusReader::new(pool.clone());
        let preflight = preflight_analysis_corpus(
            &reader,
            &request,
            16_000,
            AppAnalysisRunPreflightLimits::default(),
        )
        .await
        .expect("preflight");

        assert_eq!(preflight.message_count(), corpus.len(), "mode {mode:?}");
    }
}
