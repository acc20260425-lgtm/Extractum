use super::harness::{rebuild_documents_for_sources, sample_corpus, sample_run, snapshot_pool};
use crate::analysis::corpus::{
    ListRunSnapshotMessagesRequest, list_run_snapshot_messages_page, load_run_corpus_messages,
    load_run_snapshot_messages, load_trace_resolution_messages,
};
use crate::analysis::models::AnalysisRunMessageCursor;
use crate::analysis::store::persist_run_snapshot;
use crate::compression::compress_text;
use crate::error::AppErrorKind;
#[tokio::test]
async fn run_snapshot_roundtrips_frozen_corpus() {
    let pool = snapshot_pool().await;
    sqlx::query(
        r#"
        INSERT INTO analysis_runs (
            id,
            run_type,
            scope_type,
            source_group_id,
            period_from,
            period_to,
            output_language,
            prompt_template_version,
            provider_profile,
            provider,
            model,
            status,
            created_at
        )
        VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
        "#,
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");

    let corpus = sample_corpus();
    persist_run_snapshot(&pool, 1, "Frozen group", &corpus)
        .await
        .expect("persist snapshot");

    let loaded = load_run_snapshot_messages(&pool, 1)
        .await
        .expect("load snapshot");

    assert_eq!(loaded.len(), corpus.len());
    assert_eq!(loaded[0].r#ref, "s2-i11");
    assert_eq!(loaded[1].content, "Second frozen message");
}

#[tokio::test]
async fn list_run_snapshot_messages_page_reads_saved_snapshot_only() {
    let pool = snapshot_pool().await;
    sqlx::query(
        r#"
        INSERT INTO analysis_runs (
            id, run_type, scope_type, source_group_id, period_from, period_to,
            output_language, prompt_template_version, provider_profile, provider,
            model, status, created_at
        )
        VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
        "#,
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");

    persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
        .await
        .expect("persist snapshot");

    let page = list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id: 1,
            after: None,
            limit: 1,
            source_id: None,
            around_ref: None,
        },
    )
    .await
    .expect("load first page");

    assert_eq!(page.messages.len(), 1);
    assert_eq!(page.messages[0].content, "First frozen message");
    assert_eq!(page.messages[0].source_type.as_deref(), Some("youtube"));
    assert_eq!(
        page.messages[0]
            .metadata_json
            .as_ref()
            .and_then(|value| value.get("video_id"))
            .and_then(|value| value.as_str()),
        Some("video2")
    );
    assert!(page.has_more);

    let second_page = list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id: 1,
            after: page.next_cursor,
            limit: 1,
            source_id: None,
            around_ref: None,
        },
    )
    .await
    .expect("load second page");

    assert_eq!(second_page.messages.len(), 1);
    assert_eq!(second_page.messages[0].content, "Second frozen message");
    assert!(!second_page.has_more);
    assert_eq!(second_page.next_cursor, None);

    let filtered_page = list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id: 1,
            after: None,
            limit: 25,
            source_id: Some(4),
            around_ref: None,
        },
    )
    .await
    .expect("load source-filtered page");

    assert_eq!(filtered_page.messages.len(), 1);
    assert_eq!(filtered_page.messages[0].source_id, 4);
    assert_eq!(filtered_page.messages[0].content, "Second frozen message");
}

#[tokio::test]
async fn list_run_snapshot_messages_page_returns_typed_internal_for_corrupt_snapshot_content() {
    let pool = snapshot_pool().await;
    sqlx::query(
        r#"
        INSERT INTO analysis_runs (
            id, run_type, scope_type, source_group_id, period_from, period_to,
            output_language, prompt_template_version, provider_profile, provider,
            model, status, created_at
        )
        VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
        "#,
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");
    persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
        .await
        .expect("persist snapshot");
    sqlx::query("UPDATE analysis_run_messages SET content_zstd = x'00' WHERE run_id = 1")
        .execute(&pool)
        .await
        .expect("corrupt snapshot content");

    let error = match list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id: 1,
            after: None,
            limit: 25,
            source_id: None,
            around_ref: None,
        },
    )
    .await
    {
        Ok(_) => panic!("corrupt snapshot content should fail"),
        Err(error) => error,
    };

    assert_eq!(error.kind, AppErrorKind::Internal);
    assert!(!error.message.starts_with("Database error:"));
    assert!(!error.message.trim().is_empty());
}

#[tokio::test]
async fn list_run_snapshot_messages_page_starts_at_around_ref() {
    let pool = snapshot_pool().await;
    sqlx::query(
        r#"
        INSERT INTO analysis_runs (
            id, run_type, scope_type, source_group_id, period_from, period_to,
            output_language, prompt_template_version, provider_profile, provider,
            model, status, created_at
        )
        VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
        "#,
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");

    persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
        .await
        .expect("persist snapshot");

    let page = list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id: 1,
            after: None,
            limit: 10,
            source_id: None,
            around_ref: Some("s4-i12".to_string()),
        },
    )
    .await
    .expect("load around ref");

    assert_eq!(
        page.messages
            .iter()
            .map(|message| message.r#ref.as_str())
            .collect::<Vec<_>>(),
        vec!["s4-i12"]
    );
}

#[tokio::test]
async fn list_run_snapshot_messages_page_does_not_fall_back_to_live_source() {
    let pool = snapshot_pool().await;
    sqlx::query(
        r#"
        INSERT INTO analysis_runs (
            id, run_type, scope_type, source_id, period_from, period_to,
            output_language, prompt_template_version, provider_profile, provider,
            model, status, created_at
        )
        VALUES (1, 'report', 'single_source', 2, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
        "#,
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");

    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind("telegram_message")
    .bind("Alice")
    .bind(1_710_000_000_i64)
    .bind(compress_text("Live source message").expect("compress live message"))
    .execute(&pool)
    .await
    .expect("insert live item");

    let page = list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id: 1,
            after: None,
            limit: 25,
            source_id: None,
            around_ref: None,
        },
    )
    .await
    .expect("load snapshot-only page");

    assert_eq!(page.messages, Vec::new());
    assert_eq!(page.next_cursor, None);
    assert!(!page.has_more);
}

#[tokio::test]
async fn trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot() {
    let pool = snapshot_pool().await;
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind("telegram_message")
    .bind("Alice")
    .bind(1_710_000_000_i64)
    .bind(compress_text("Live source text").expect("compress live text"))
    .execute(&pool)
    .await
    .expect("insert live item");

    let messages = load_trace_resolution_messages(&pool, &sample_run())
        .await
        .expect("load trace resolution messages");

    assert!(messages.is_empty());
}

#[test]
fn run_message_cursor_uses_ref_and_published_at() {
    let cursor = AnalysisRunMessageCursor {
        published_at: 1_710_000_000,
        r#ref: "s7-i1".to_string(),
    };

    assert_eq!(cursor.published_at, 1_710_000_000);
    assert_eq!(cursor.r#ref, "s7-i1");
}

#[tokio::test]
async fn load_run_corpus_messages_uses_snapshot_when_available() {
    let pool = snapshot_pool().await;
    sqlx::query(
        r#"
        INSERT INTO analysis_runs (
            id,
            run_type,
            scope_type,
            source_group_id,
            period_from,
            period_to,
            output_language,
            prompt_template_version,
            provider_profile,
            provider,
            model,
            status,
            created_at
        )
        VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
        "#,
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");
    persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
        .await
        .expect("persist snapshot");

    let corpus = load_run_corpus_messages(&pool, &sample_run())
        .await
        .expect("load run corpus");

    assert_eq!(corpus.len(), 2);
    assert_eq!(corpus[0].external_id, "100");
    assert_eq!(corpus[0].item_kind.as_deref(), Some("youtube_transcript"));
    assert_eq!(corpus[0].source_type.as_deref(), Some("youtube"));
    assert_eq!(corpus[0].source_subtype.as_deref(), Some("video"));
    assert!(corpus[0].metadata_zstd.is_some());
    assert_eq!(corpus[1].r#ref, "s4-i12");
}

#[tokio::test]
async fn load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows()
{
    let pool = snapshot_pool().await;
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, source_id, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model, status, created_at
         )
         VALUES (1, 'report', 'single_source', 2, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)",
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
         VALUES (11, 2, '100', 'telegram_message', 'Alice', ?, ?)",
    )
    .bind(1_710_000_000_i64)
    .bind(compress_text("live drift").expect("compress"))
    .execute(&pool)
    .await
    .expect("insert live item");
    rebuild_documents_for_sources(&pool, &[2]).await;

    let mut run = sample_run();
    run.id = 1;
    run.scope_type = crate::analysis::ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE.to_string();
    run.source_id = Some(2);
    run.source_group_id = None;
    run.snapshot_state = Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed);
    run.snapshot_captured_at = None;
    run.snapshot_error = None;
    run.snapshot_message_count = 0;

    let corpus = load_run_corpus_messages(&pool, &run)
        .await
        .expect("load snapshot-only corpus");

    assert!(corpus.is_empty());
}

#[tokio::test]
async fn captured_marker_with_missing_rows_returns_corrupt_snapshot_error() {
    let pool = snapshot_pool().await;
    let mut run = sample_run();
    run.scope_type = crate::analysis::ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE.to_string();
    run.source_id = Some(2);
    run.source_group_id = None;
    run.snapshot_state = Some(crate::analysis::models::AnalysisSnapshotState::Captured);
    run.snapshot_captured_at = Some("2026-05-18T10:00:00Z".to_string());
    run.snapshot_error = None;
    run.snapshot_message_count = 0;

    let error = match load_run_corpus_messages(&pool, &run).await {
        Ok(_) => panic!("captured marker without rows should fail defensively"),
        Err(error) => error,
    };

    assert!(error.message.contains("snapshot is unavailable"));
}

#[tokio::test]
async fn source_group_membership_drift_after_capture_does_not_change_saved_run_corpus() {
    let pool = snapshot_pool().await;
    sqlx::query("INSERT INTO analysis_source_groups (id, name, source_type, created_at, updated_at) VALUES (9, 'Group', 'telegram', 1, 1)")
        .execute(&pool)
        .await
        .expect("insert group");
    sqlx::query("INSERT INTO analysis_source_group_members (group_id, source_id, created_at) VALUES (9, 2, 1), (9, 4, 1)")
        .execute(&pool)
        .await
        .expect("insert original members");
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, source_group_id, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model, status, snapshot_captured_at, created_at
         )
         VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', '2026-05-18T10:00:00Z', ?)",
    )
    .bind(1_700_000_000_i64)
    .bind(1_800_000_000_i64)
    .bind(1_710_000_500_i64)
    .execute(&pool)
    .await
    .expect("insert run");
    persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
        .await
        .expect("persist snapshot");
    sqlx::query(
        "DELETE FROM analysis_source_group_members WHERE group_id = 9 AND source_id = 4",
    )
    .execute(&pool)
    .await
    .expect("remove member after capture");

    let mut run = sample_run();
    run.id = 1;
    run.source_group_id = Some(9);
    run.snapshot_state = Some(crate::analysis::models::AnalysisSnapshotState::Captured);
    run.snapshot_captured_at = Some("2026-05-18T10:00:00Z".to_string());
    run.snapshot_message_count = 2;

    let corpus = load_run_corpus_messages(&pool, &run)
        .await
        .expect("load saved corpus");

    assert_eq!(corpus.len(), 2);
    assert_eq!(
        corpus
            .iter()
            .map(|message| message.source_id)
            .collect::<Vec<_>>(),
        vec![2, 4]
    );
}
