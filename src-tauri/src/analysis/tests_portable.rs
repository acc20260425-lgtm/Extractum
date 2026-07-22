use super::groups::normalize_source_group_input;
use super::store::ensure_builtin_report_template;
use super::templates::validate_template_kind;
use super::trace::compress_trace_data;
use super::{
    decode_trace_data, validate_chat_role, AnalysisChatTurn, AnalysisTraceData, AnalysisTraceRef,
    TEMPLATE_KIND_REPORT,
};
use extractum_core::error::AppErrorKind;

async fn memory_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query(
        r#"
        CREATE TABLE analysis_prompt_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            template_kind TEXT NOT NULL,
            body TEXT NOT NULL,
            version INTEGER NOT NULL DEFAULT 1,
            is_builtin BOOLEAN NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create templates");
    sqlx::query("CREATE TABLE sources (id INTEGER PRIMARY KEY, title TEXT)")
        .execute(&pool)
        .await
        .expect("create sources");
    sqlx::query("CREATE TABLE analysis_source_groups (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .expect("create source groups");
    sqlx::query("CREATE TABLE projects (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
        .execute(&pool)
        .await
        .expect("create projects");
    sqlx::query(
        r#"
        CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_type TEXT NOT NULL,
            scope_type TEXT NOT NULL,
            source_id INTEGER,
            source_group_id INTEGER,
            project_id INTEGER,
            period_from INTEGER NOT NULL,
            period_to INTEGER NOT NULL,
            output_language TEXT NOT NULL,
            prompt_template_id INTEGER,
            prompt_template_version INTEGER NOT NULL,
            provider_profile TEXT NOT NULL,
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
            telegram_history_scope TEXT,
            status TEXT NOT NULL,
            result_markdown TEXT,
            trace_data_zstd BLOB,
            scope_label_snapshot TEXT,
            snapshot_captured_at TEXT,
            snapshot_error TEXT,
            error TEXT,
            created_at INTEGER NOT NULL,
            completed_at INTEGER
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create runs");
    sqlx::query("CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL, ref TEXT NOT NULL)")
        .execute(&pool)
        .await
        .expect("create run messages");
    pool
}

#[tokio::test]
async fn builtin_template_is_seeded_once() {
    let pool = memory_pool().await;
    ensure_builtin_report_template(&pool)
        .await
        .expect("seed builtin");
    ensure_builtin_report_template(&pool)
        .await
        .expect("seed builtin twice");

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM analysis_prompt_templates WHERE template_kind = ?",
    )
    .bind(TEMPLATE_KIND_REPORT)
    .fetch_one(&pool)
    .await
    .expect("count report templates");
    let body = sqlx::query_scalar::<_, String>(
        "SELECT body FROM analysis_prompt_templates WHERE template_kind = ?",
    )
    .bind(TEMPLATE_KIND_REPORT)
    .fetch_one(&pool)
    .await
    .expect("load report template body");

    assert_eq!(count, 1);
    assert!(body.contains("source documents"));
    assert!(!body.contains("Telegram messages"));
}

#[tokio::test]
async fn completed_run_without_snapshot_marker_is_capture_failed() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, source_id, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model, status,
            result_markdown, created_at, completed_at
         )
         VALUES (1, 'report', 'single_source', 2, 1, 2, 'English', 1, 'default', 'gemini', 'model', 'completed', 'Saved report', 1, 2)",
    )
    .execute(&pool)
    .await
    .expect("insert run");

    let detail = super::store::fetch_run_row(&pool, 1)
        .await
        .expect("fetch run")
        .map(super::store::map_run_detail)
        .expect("run exists");

    assert_eq!(
        detail.snapshot_state,
        Some(super::models::AnalysisSnapshotState::CaptureFailed)
    );
}

#[test]
fn trace_data_roundtrips_through_zstd() {
    let trace = AnalysisTraceData {
        refs: vec![AnalysisTraceRef {
            r#ref: "s12-i321".to_string(),
            item_id: 321,
            source_id: 12,
            external_id: "845".to_string(),
            published_at: 1_710_000_000,
            excerpt: "Example quote".to_string(),
            youtube_url: None,
            youtube_timestamp_seconds: None,
            youtube_display_label: None,
            is_synthetic: false,
        }],
    };

    let compressed = compress_trace_data(&trace).expect("compress");
    let decoded = decode_trace_data(Some(&compressed)).expect("decode");
    assert_eq!(decoded, trace);
}

#[test]
fn source_group_input_is_trimmed_and_deduplicated() {
    let (name, source_ids) =
        normalize_source_group_input("  Core sources  ", vec![4, 2, 4, -1, 2])
            .expect("normalize source group");

    assert_eq!(name, "Core sources");
    assert_eq!(source_ids, vec![2, 4]);
}

#[test]
fn template_kind_validation_returns_typed_error() {
    let error = validate_template_kind("summary").expect_err("reject unsupported kind");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(error.message, "Unsupported template kind 'summary'");
}

#[test]
fn source_group_input_validation_returns_typed_error() {
    let error = normalize_source_group_input("  ", vec![1]).expect_err("reject empty name");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(error.message, "Source group name cannot be empty");
}

#[test]
fn chat_role_validation_returns_typed_error() {
    let error = validate_chat_role("system").expect_err("reject unsupported role");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(error.message, "Unsupported chat role 'system'");
}

#[test]
fn chat_turn_validation_returns_typed_error() {
    let history = vec![AnalysisChatTurn {
        role: "user".to_string(),
        content: "   ".to_string(),
    }];
    let error = super::validate_chat_turns(&history).expect_err("reject empty chat turn");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(error.message, "Chat turns cannot be empty");
}
