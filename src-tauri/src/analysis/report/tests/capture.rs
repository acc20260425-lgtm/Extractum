use super::super::capture_report_corpus;
use crate::analysis::corpus::{CorpusLoadRequest, YoutubeCorpusMode};

#[tokio::test]
async fn capture_report_corpus_returns_reloaded_snapshot_before_provider_phases() {
    let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;
    crate::sources::test_support::create_analysis_documents_table(&pool).await;
    crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
    sqlx::query(
        "CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY,
            scope_label_snapshot TEXT,
            snapshot_captured_at TEXT,
            snapshot_error TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("create runs");
    sqlx::query(
        "CREATE TABLE analysis_run_messages (
            run_id INTEGER NOT NULL,
            item_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            external_id TEXT NOT NULL,
            author TEXT,
            published_at INTEGER NOT NULL,
            ref TEXT NOT NULL,
            content_zstd BLOB NOT NULL,
            item_kind TEXT,
            source_type TEXT,
            source_subtype TEXT,
            metadata_zstd BLOB,
            PRIMARY KEY (run_id, ref)
        )",
    )
    .execute(&pool)
    .await
    .expect("create run messages");
    sqlx::query("INSERT INTO analysis_runs (id) VALUES (1)")
        .execute(&pool)
        .await
        .expect("insert run");
    sqlx::query(
        "CREATE TABLE youtube_transcript_segments (
            item_id INTEGER NOT NULL,
            source_id INTEGER NOT NULL,
            segment_index INTEGER NOT NULL,
            start_ms INTEGER NOT NULL,
            end_ms INTEGER,
            text TEXT NOT NULL,
            chapter_index INTEGER,
            caption_language TEXT,
            caption_track_kind TEXT,
            is_auto_generated INTEGER NOT NULL DEFAULT 0,
            metadata_zstd BLOB,
            UNIQUE(item_id, segment_index)
        )",
    )
    .execute(&pool)
    .await
    .expect("create youtube transcript segments");
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
         VALUES (2, 'telegram', 'channel', 'tg2', 'Telegram 2', 1, 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("insert source");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, ingested_at, content_kind, has_media, content_zstd)
         VALUES (10, 2, '10', 'telegram_message', 'Alice', 100, 100, 'text_only', 0, ?)",
    )
    .bind(crate::compression::compress_text("captured text").expect("compress"))
    .execute(&pool)
    .await
    .expect("insert item");
    crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 2)
        .await
        .expect("rebuild docs");

    let request = CorpusLoadRequest {
        source_type: "telegram".to_string(),
        source_ids: vec![2],
        period_from: 1,
        period_to: 1_000,
        youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
        include_migrated_history: false,
    };

    let captured = capture_report_corpus(&pool, 1, "Frozen source", &request)
        .await
        .expect("capture report corpus");

    sqlx::query("DELETE FROM analysis_documents WHERE source_id = 2")
        .execute(&pool)
        .await
        .expect("delete live docs after capture");

    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].content, "captured text");
}
