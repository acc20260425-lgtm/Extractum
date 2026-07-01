use super::harness::{
    corpus_request, decode_message_metadata_for_test, insert_youtube_transcript_segment,
    insert_youtube_video_source, insert_youtube_video_source_with_typed_metadata,
    rebuild_documents_for_sources, seed_analysis_source, seed_telegram_item, snapshot_pool,
};
use crate::analysis::corpus::{
    AnalysisRunPreflightLimits, CorpusLoadRequest, YoutubeCorpusMode, live_corpus_ref,
    load_corpus_messages, preflight_analysis_run,
};
use crate::compression::compress_text;
use crate::error::AppErrorKind;
#[tokio::test]
async fn default_analysis_corpus_excludes_migrated_history_documents() {
    let pool = snapshot_pool().await;
    crate::sources::test_support::create_telegram_messages_table(&pool).await;
    crate::sources::test_support::create_analysis_documents_table(&pool).await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
         VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
    )
    .execute(&pool)
    .await
    .expect("seed source");
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at, content_zstd
         ) VALUES
            (1, 1, '10', 'telegram_message', 'Ada', 1700000010, ?),
            (2, 1, '10', 'telegram_message', 'Ada', 1700000009, ?)",
    )
    .bind(compress_text("current").expect("compress current"))
    .bind(compress_text("migrated").expect("compress migrated"))
    .execute(&pool)
    .await
    .expect("seed items");
    sqlx::query(
        "INSERT INTO telegram_messages (
            item_id, source_id, history_peer_kind, history_peer_id,
            telegram_message_id, migration_domain, is_migrated_history
         ) VALUES (1, 1, 'channel', 12345, 10, NULL, 0),
                  (2, 1, 'chat', 777, 10, 'migrated_from_chat', 1)",
    )
    .execute(&pool)
    .await
    .expect("seed telegram rows");

    crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 1)
        .await
        .expect("rebuild docs");

    let document_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM analysis_documents d
         JOIN telegram_messages tm ON tm.item_id = d.item_id
         WHERE d.source_id = 1 AND tm.is_migrated_history = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("count migrated docs");

    assert_eq!(document_count, 0);
}

#[tokio::test]
async fn opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight() {
    let pool = snapshot_pool().await;
    seed_analysis_source(&pool, 1, "telegram", "supergroup").await;
    seed_telegram_item(&pool, 10, 1, "10", 100, "current", false).await;
    seed_telegram_item(&pool, 11, 1, "11", 90, "migrated", true).await;
    crate::sources::test_support::create_analysis_documents_table(&pool).await;
    crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 1)
        .await
        .expect("rebuild docs");

    let request = CorpusLoadRequest {
        source_type: "telegram".to_string(),
        source_ids: vec![1],
        period_from: 1,
        period_to: 200,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
        include_migrated_history: true,
    };

    let corpus = load_corpus_messages(&pool, &request)
        .await
        .expect("load corpus");
    assert_eq!(
        corpus
            .iter()
            .map(|message| message.item_id)
            .collect::<Vec<_>>(),
        vec![11, 10]
    );

    let migrated_metadata = decode_message_metadata_for_test(&corpus[0]);
    assert_eq!(migrated_metadata["history_scope"], "migrated");
    assert_eq!(migrated_metadata["migration_domain"], "migrated_from_chat");

    let preflight = preflight_analysis_run(
        &pool,
        &request,
        16000,
        AnalysisRunPreflightLimits::default(),
    )
    .await
    .expect("preflight");
    assert_eq!(preflight.message_count, 2);
}

#[tokio::test]
async fn source_group_opt_in_includes_only_members_with_migrated_rows() {
    let pool = snapshot_pool().await;
    seed_analysis_source(&pool, 1, "telegram", "supergroup").await;
    seed_analysis_source(&pool, 2, "telegram", "supergroup").await;
    seed_telegram_item(&pool, 10, 1, "10", 100, "current one", false).await;
    seed_telegram_item(&pool, 11, 1, "11", 90, "migrated one", true).await;
    seed_telegram_item(&pool, 20, 2, "20", 95, "current two", false).await;
    crate::sources::test_support::create_analysis_documents_table(&pool).await;
    crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 1)
        .await
        .expect("rebuild source 1");
    crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 2)
        .await
        .expect("rebuild source 2");

    let request = CorpusLoadRequest {
        source_type: "telegram".to_string(),
        source_ids: vec![1, 2],
        period_from: 1,
        period_to: 200,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
        include_migrated_history: true,
    };

    let corpus = load_corpus_messages(&pool, &request)
        .await
        .expect("load corpus");

    assert_eq!(
        corpus
            .iter()
            .map(|message| message.item_id)
            .collect::<Vec<_>>(),
        vec![11, 20, 10]
    );
}

#[tokio::test]
async fn explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus() {
    let pool = snapshot_pool().await;
    seed_analysis_source(&pool, 1, "telegram", "supergroup").await;
    seed_telegram_item(&pool, 10, 1, "10", 100, "current only", false).await;
    crate::sources::test_support::create_analysis_documents_table(&pool).await;
    crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 1)
        .await
        .expect("rebuild docs");

    let request = CorpusLoadRequest {
        source_type: "telegram".to_string(),
        source_ids: vec![1],
        period_from: 1,
        period_to: 200,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
        include_migrated_history: true,
    };

    let corpus = load_corpus_messages(&pool, &request)
        .await
        .expect("load corpus");
    assert_eq!(
        corpus
            .iter()
            .map(|message| message.item_id)
            .collect::<Vec<_>>(),
        vec![10]
    );

    let preflight = preflight_analysis_run(
        &pool,
        &request,
        16000,
        AnalysisRunPreflightLimits::default(),
    )
    .await
    .expect("preflight");
    assert_eq!(preflight.message_count, 1);
}

#[tokio::test]
async fn youtube_description_rows_use_typed_metadata_with_corrupt_source_blob() {
    let pool = snapshot_pool().await;
    insert_youtube_video_source_with_typed_metadata(
        &pool,
        401,
        "video401",
        "Typed title",
        Some("Typed description"),
        Some("2026-05-17"),
    )
    .await;
    sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = 401")
        .execute(&pool)
        .await
        .expect("corrupt source blob");
    rebuild_documents_for_sources(&pool, &[401]).await;

    let request = CorpusLoadRequest {
        source_type: "youtube".to_string(),
        source_ids: vec![401],
        period_from: 1,
        period_to: i64::MAX,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
        include_migrated_history: false,
    };
    let messages = load_corpus_messages(&pool, &request)
        .await
        .expect("load descriptions");

    assert_eq!(messages.len(), 1);
    assert!(messages[0].content.contains("Typed description"));
    assert!(messages[0]
        .content
        .contains("URL: https://www.youtube.com/watch?v=video401"));
}

#[tokio::test]
async fn youtube_description_missing_typed_metadata_skips_without_decoding_source_blob() {
    let pool = snapshot_pool().await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, metadata_zstd) VALUES (402, 'youtube', 'video', 'video402', 'Generic title', x'00')",
    )
    .execute(&pool)
    .await
    .expect("insert source");
    rebuild_documents_for_sources(&pool, &[402]).await;

    let request = CorpusLoadRequest {
        source_type: "youtube".to_string(),
        source_ids: vec![402],
        period_from: 1,
        period_to: i64::MAX,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
        include_migrated_history: false,
    };
    let messages = load_corpus_messages(&pool, &request)
        .await
        .expect("load descriptions");

    assert!(messages.is_empty());
}

#[tokio::test]
async fn youtube_transcript_segment_evidence_uses_typed_source_context() {
    let pool = snapshot_pool().await;
    insert_youtube_video_source_with_typed_metadata(
        &pool,
        403,
        "video403",
        "Typed title",
        None,
        Some("2026-05-17"),
    )
    .await;
    sqlx::query("UPDATE sources SET title = 'Generic title' WHERE id = 403")
        .execute(&pool)
        .await
        .expect("set generic source title");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at)
         VALUES (9001, 403, 'transcript:video403:en:manual', 'youtube_transcript', 'Channel', 1)",
    )
    .execute(&pool)
    .await
    .expect("insert transcript item");
    insert_youtube_transcript_segment(&pool, 9001, 403, 12_000, "segment text").await;
    sqlx::query("UPDATE sources SET metadata_zstd = x'00' WHERE id = 403")
        .execute(&pool)
        .await
        .expect("corrupt source blob");
    rebuild_documents_for_sources(&pool, &[403]).await;

    let request = CorpusLoadRequest {
        source_type: "youtube".to_string(),
        source_ids: vec![403],
        period_from: 1,
        period_to: i64::MAX,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptOnly,
        include_migrated_history: false,
    };
    let messages = load_corpus_messages(&pool, &request)
        .await
        .expect("load transcript segments");

    let metadata_json = decode_message_metadata_for_test(&messages[0]);
    assert_eq!(metadata_json["video_id"], "video403");
    assert_eq!(
        metadata_json["canonical_url"],
        "https://www.youtube.com/watch?v=video403"
    );
    assert_eq!(metadata_json["title"], "Typed title");
    assert_eq!(metadata_json["segment_start_ms"], 12_000);
}

#[tokio::test]
async fn live_corpus_refs_use_local_item_ids() {
    let pool = snapshot_pool().await;
    let first_content = compress_text("First live document").expect("compress first");
    let second_content = compress_text("Second live document").expect("compress second");
    sqlx::query(
        r#"
        INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
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
        r#"
        INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
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

    let request = corpus_request(
        "telegram",
        vec![2, 4],
        YoutubeCorpusMode::TranscriptDescription,
    );
    let corpus = load_corpus_messages(&pool, &request)
        .await
        .expect("load live corpus");

    assert_eq!(corpus.len(), 2);
    assert_eq!(corpus[0].r#ref, "s2-i11");
    assert_eq!(corpus[1].r#ref, "s4-i12");
}

#[tokio::test]
async fn preflight_ref_format_matches_corpus_loader_ref_format() {
    let pool = snapshot_pool().await;
    let content = compress_text("Test message").expect("compress");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind(Option::<String>::None)
    .bind(1_710_000_000_i64)
    .bind(content)
    .execute(&pool)
    .await
    .expect("insert item");
    rebuild_documents_for_sources(&pool, &[2]).await;

    let corpus = load_corpus_messages(
        &pool,
        &corpus_request(
            "telegram",
            vec![2],
            YoutubeCorpusMode::TranscriptDescription,
        ),
    )
    .await
    .expect("load corpus");

    assert_eq!(
        corpus[0].r#ref,
        live_corpus_ref(corpus[0].source_id, corpus[0].item_id)
    );
}

#[tokio::test]
async fn load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content() {
    let pool = snapshot_pool().await;
    let content = compress_text("Corrupt me").expect("compress");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind(Option::<String>::None)
    .bind(1_710_000_000_i64)
    .bind(content)
    .execute(&pool)
    .await
    .expect("insert item");
    rebuild_documents_for_sources(&pool, &[2]).await;
    sqlx::query("UPDATE analysis_documents SET content_zstd = x'00' WHERE source_id = 2")
        .execute(&pool)
        .await
        .expect("corrupt live document content");

    let error = match load_corpus_messages(
        &pool,
        &corpus_request(
            "telegram",
            vec![2],
            YoutubeCorpusMode::TranscriptDescription,
        ),
    )
    .await
    {
        Ok(_) => panic!("corrupt live document content should fail"),
        Err(error) => error,
    };

    assert_eq!(error.kind, AppErrorKind::Internal);
    assert!(!error.message.starts_with("Database error:"));
    assert!(!error.message.is_empty());
}

#[tokio::test]
async fn load_corpus_messages_orders_transcript_segments_by_document_order_not_ref() {
    let pool = snapshot_pool().await;
    insert_youtube_video_source(&pool, 20).await;
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
         VALUES (?, ?, ?, 'youtube_transcript', 'Channel', ?, ?)",
    )
    .bind(21_i64)
    .bind(20_i64)
    .bind("transcript:v1:en:manual")
    .bind(1_710_000_000_i64)
    .bind(compress_text("full transcript").expect("compress"))
    .execute(&pool)
    .await
    .expect("insert transcript item");
    insert_youtube_transcript_segment(&pool, 21, 20, 900, "early").await;
    sqlx::query(
        "INSERT INTO youtube_transcript_segments (
            item_id, source_id, segment_index, start_ms, end_ms, text,
            caption_language, caption_track_kind, is_auto_generated
         ) VALUES (21, 20, 1, 10000, 11000, 'late', 'en', 'manual', 0)",
    )
    .execute(&pool)
    .await
    .expect("insert late segment");
    rebuild_documents_for_sources(&pool, &[20]).await;

    let corpus = load_corpus_messages(
        &pool,
        &corpus_request("youtube", vec![20], YoutubeCorpusMode::TranscriptOnly),
    )
    .await
    .expect("load corpus");

    assert_eq!(
        corpus
            .iter()
            .map(|message| message.r#ref.as_str())
            .collect::<Vec<_>>(),
        vec!["s20-i21@900ms", "s20-i21@10000ms"]
    );
}

#[test]
fn youtube_corpus_mode_parses_wire_values_and_defaults() {
    assert_eq!(
        YoutubeCorpusMode::from_wire(None).expect("default mode"),
        YoutubeCorpusMode::TranscriptDescription
    );
    assert_eq!(
        YoutubeCorpusMode::from_wire(Some("transcript_only")).expect("transcript only"),
        YoutubeCorpusMode::TranscriptOnly
    );
    assert_eq!(
        YoutubeCorpusMode::from_wire(Some("transcript_description_comments"))
            .expect("comments mode"),
        YoutubeCorpusMode::TranscriptDescriptionComments
    );
    assert!(YoutubeCorpusMode::from_wire(Some("all_text")).is_err());
    assert_eq!(
        YoutubeCorpusMode::TranscriptOnly.as_wire(),
        "transcript_only"
    );
    assert_eq!(
        YoutubeCorpusMode::TranscriptDescription.as_wire(),
        "transcript_description"
    );
    assert_eq!(
        YoutubeCorpusMode::TranscriptDescriptionComments.as_wire(),
        "transcript_description_comments"
    );
}

#[tokio::test]
async fn load_corpus_messages_filters_telegram_to_telegram_message() {
    let pool = snapshot_pool().await;
    insert_youtube_video_source(&pool, 20).await;
    let telegram_text = compress_text("Telegram message").expect("compress telegram");
    let youtube_text = compress_text("YouTube comment").expect("compress youtube");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?, ?), (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind("telegram_message")
    .bind("Alice")
    .bind(1_710_000_000_i64)
    .bind(telegram_text)
    .bind(12_i64)
    .bind(20_i64)
    .bind("comment:c1")
    .bind("youtube_comment")
    .bind("Bob")
    .bind(1_710_000_001_i64)
    .bind(youtube_text)
    .execute(&pool)
    .await
    .expect("insert mixed items");
    rebuild_documents_for_sources(&pool, &[2, 20]).await;

    let corpus = load_corpus_messages(
        &pool,
        &corpus_request(
            "telegram",
            vec![2, 20],
            YoutubeCorpusMode::TranscriptDescription,
        ),
    )
    .await
    .expect("load telegram corpus");

    assert_eq!(corpus.len(), 1);
    assert_eq!(corpus[0].external_id, "100");
    assert_eq!(corpus[0].content, "Telegram message");
}

#[tokio::test]
async fn load_corpus_messages_filters_youtube_transcript_only_to_transcripts() {
    let pool = snapshot_pool().await;
    insert_youtube_video_source(&pool, 20).await;
    let transcript = compress_text("Transcript text").expect("compress transcript");
    let comment = compress_text("Comment text").expect("compress comment");
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
    .bind(transcript)
    .bind(22_i64)
    .bind(20_i64)
    .bind("comment:c1")
    .bind("youtube_comment")
    .bind("Commenter")
    .bind(1_710_000_001_i64)
    .bind(comment)
    .execute(&pool)
    .await
    .expect("insert youtube items");
    insert_youtube_transcript_segment(&pool, 21, 20, 754_000, "Transcript text").await;
    rebuild_documents_for_sources(&pool, &[20]).await;

    let corpus = load_corpus_messages(
        &pool,
        &corpus_request("youtube", vec![20], YoutubeCorpusMode::TranscriptOnly),
    )
    .await
    .expect("load youtube transcript-only corpus");

    assert_eq!(corpus.len(), 1);
    assert_eq!(corpus[0].external_id, "transcript:v1:en:manual");
    assert_eq!(corpus[0].r#ref, "s20-i21@754000ms");
}

#[tokio::test]
async fn load_corpus_messages_includes_youtube_comment_only_in_comments_mode() {
    let pool = snapshot_pool().await;
    insert_youtube_video_source(&pool, 20).await;
    let transcript = compress_text("Transcript text").expect("compress transcript");
    let comment = compress_text("Comment text").expect("compress comment");
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
    .bind(transcript)
    .bind(22_i64)
    .bind(20_i64)
    .bind("comment:c1")
    .bind("youtube_comment")
    .bind("Commenter")
    .bind(1_710_000_001_i64)
    .bind(comment)
    .execute(&pool)
    .await
    .expect("insert youtube items");
    insert_youtube_transcript_segment(&pool, 21, 20, 754_000, "Transcript text").await;
    rebuild_documents_for_sources(&pool, &[20]).await;

    let without_comments = load_corpus_messages(
        &pool,
        &corpus_request(
            "youtube",
            vec![20],
            YoutubeCorpusMode::TranscriptDescription,
        ),
    )
    .await
    .expect("load youtube transcript+description corpus");
    let with_comments = load_corpus_messages(
        &pool,
        &corpus_request(
            "youtube",
            vec![20],
            YoutubeCorpusMode::TranscriptDescriptionComments,
        ),
    )
    .await
    .expect("load youtube comments corpus");

    assert_eq!(without_comments.len(), 1);
    assert_eq!(with_comments.len(), 2);
    assert!(with_comments
        .iter()
        .any(|message| message.external_id == "comment:c1"));
}

#[tokio::test]
async fn description_mode_creates_synthetic_description_message() {
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
    rebuild_documents_for_sources(&pool, &[20]).await;

    let corpus = load_corpus_messages(
        &pool,
        &corpus_request(
            "youtube",
            vec![20],
            YoutubeCorpusMode::TranscriptDescription,
        ),
    )
    .await
    .expect("load youtube corpus");

    assert_eq!(corpus.len(), 1);
    assert_eq!(corpus[0].item_id, 0);
    assert_eq!(corpus[0].external_id, "description:video1");
    assert_eq!(corpus[0].r#ref, "s20-i0");
    assert!(corpus[0].content.contains("YouTube video description"));
    assert!(corpus[0].content.contains("Description body"));
}
