use super::super::{
    capture_run_snapshot, mark_run_capture_failed, sanitize_provider_error, sanitize_snapshot_error,
};
use crate::analysis::models::CorpusMessage;

async fn snapshot_store_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query(
        r#"
            CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scope_label_snapshot TEXT,
                snapshot_captured_at TEXT,
                snapshot_error TEXT,
                status TEXT,
                result_markdown TEXT,
                trace_data_zstd BLOB,
                error TEXT,
                completed_at INTEGER
            )
            "#,
    )
    .execute(&pool)
    .await
    .expect("create runs");
    sqlx::query(
        r#"
            CREATE TABLE analysis_run_messages (
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
            )
            "#,
    )
    .execute(&pool)
    .await
    .expect("create messages");
    sqlx::query("INSERT INTO analysis_runs (id, status) VALUES (1, 'running')")
        .execute(&pool)
        .await
        .expect("insert run");
    pool
}

fn strict_snapshot_message(label: &str) -> CorpusMessage {
    CorpusMessage {
        item_id: 10,
        source_id: 2,
        external_id: label.to_string(),
        published_at: 1_710_000_000,
        author: Some("Alice".to_string()),
        content: format!("content {label}"),
        r#ref: format!("s2-i10-{label}"),
        item_kind: Some("telegram_message".to_string()),
        source_type: Some("telegram".to_string()),
        source_subtype: Some("channel".to_string()),
        metadata_zstd: None,
    }
}

#[test]
fn sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens() {
    let long = "x".repeat(600);
    let raw = format!(
        "failed at C:\\Users\\Dima\\AppData\\Local\\Extractum\\db.sqlite\n\
             see /home/dima/.config/extractum/db.sqlite and file:///tmp/secret.txt \
             https://example.test/path?token=abc#frag \
             bearer sk-live-secret api_key=secret {long}"
    );

    let sanitized = sanitize_snapshot_error("Snapshot capture failed", &raw);

    assert!(sanitized.chars().count() <= 512);
    assert!(!sanitized.contains('\n'));
    assert!(!sanitized.contains("C:\\"));
    assert!(!sanitized.contains("/home/dima"));
    assert!(!sanitized.contains("file://"));
    assert!(!sanitized.contains("?token="));
    assert!(!sanitized.contains("#frag"));
    assert!(!sanitized.to_lowercase().contains("bearer"));
    assert!(!sanitized.contains("sk-live-secret"));
    assert!(!sanitized.contains("api_key=secret"));
}

#[test]
fn sanitize_provider_error_redacts_provider_payloads() {
    let long = "x".repeat(600);
    let raw = format!(
        "OpenAI-compatible request failed with HTTP 500: \
             api_key=sk-live-secret Authorization: Bearer token-123 \
             prompt: private user prompt payload: raw provider body \
             https://llm.example.test/v1/chat/completions?api_key=secret#frag {long}"
    );

    let sanitized = sanitize_provider_error("Provider request failed", &raw);
    let lower = sanitized.to_lowercase();

    assert!(sanitized.chars().count() <= 512);
    assert!(!lower.contains("api_key"));
    assert!(!lower.contains("bearer"));
    assert!(!lower.contains("private user prompt"));
    assert!(!lower.contains("raw provider body"));
    assert!(!sanitized.contains("?api_key="));
    assert!(!sanitized.contains("#frag"));
    assert_ne!(sanitized.trim(), "");
}

#[tokio::test]
async fn capture_run_snapshot_marks_captured_after_reload_and_replaces_rows() {
    let pool = snapshot_store_pool().await;

    let first = capture_run_snapshot(&pool, 1, "Frozen scope", &[strict_snapshot_message("a")])
        .await
        .expect("capture first");
    let second = capture_run_snapshot(&pool, 1, "Frozen scope", &[strict_snapshot_message("b")])
        .await
        .expect("capture second");

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].external_id, "b");

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("count messages");
    assert_eq!(count, 1);

    let marker: Option<String> =
        sqlx::query_scalar("SELECT snapshot_captured_at FROM analysis_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("load marker");
    assert!(marker.is_some());

    let snapshot_error: Option<String> =
        sqlx::query_scalar("SELECT snapshot_error FROM analysis_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("load snapshot error");
    assert_eq!(snapshot_error, None);
}

#[tokio::test]
async fn capture_run_snapshot_rejects_missing_required_fields_without_marker() {
    let pool = snapshot_store_pool().await;
    let mut message = strict_snapshot_message("bad");
    message.item_kind = None;

    let error = match capture_run_snapshot(&pool, 1, "Frozen scope", &[message]).await {
        Ok(_) => panic!("missing item_kind should fail"),
        Err(error) => error,
    };
    assert!(error.message.contains("item_kind"));

    let marker: Option<String> =
        sqlx::query_scalar("SELECT snapshot_captured_at FROM analysis_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("load marker");
    assert_eq!(marker, None);

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("count messages");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn mark_run_capture_failed_sets_snapshot_error() {
    let pool = snapshot_store_pool().await;

    mark_run_capture_failed(
        &pool,
        1,
        "failed at C:\\Users\\Dima\\secret.sqlite?token=abc",
        1_710_000_500,
    )
    .await
    .expect("mark capture failed");

    let row: (String, Option<String>, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT status, error, snapshot_error, completed_at FROM analysis_runs WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("load run");

    assert_eq!(row.0, crate::analysis::ANALYSIS_STATUS_FAILED);
    assert_eq!(row.1, row.2);
    assert_eq!(row.3, Some(1_710_000_500));
    assert!(!row.2.unwrap().contains("C:\\"));
}
