use super::groups::AnalysisSourceGroupInput;
use super::models::AnalysisSourceKind;
use super::store::ensure_builtin_report_template;
use super::templates::validate_template_kind;
use super::trace::{compress_trace_data, get_analysis_run_trace_in_pool};
use super::{
    validate_chat_role, AnalysisChatTurn, AnalysisTraceData, AnalysisTraceRef, TEMPLATE_KIND_REPORT,
};
use extractum_core::error::AppErrorKind;

async fn memory_pool() -> sqlx::SqlitePool {
    super::test_schema::analysis_test_pool().await
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

    let mut connection = pool
        .acquire()
        .await
        .expect("acquire analysis test connection");
    let enrichment = super::store::prepare_analysis_run_detail(&mut *connection, 1)
        .await
        .expect("prepare run detail");
    let labels = super::models::AnalysisForeignLabels::new(Vec::new(), Vec::new())
        .expect("build empty foreign labels");
    let detail = enrichment
        .finish(labels)
        .expect("finish run detail")
        .expect("run exists");

    assert_eq!(
        detail.snapshot_state,
        Some(super::models::AnalysisSnapshotState::CaptureFailed)
    );
}

#[tokio::test]
async fn trace_data_roundtrips_through_zstd() {
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
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model, status,
            trace_data_zstd, created_at
         ) VALUES (
            1, 'report', 'single_source', 1, 2, 'English',
            1, 'default', 'gemini', 'model', 'completed', ?, 1
         )",
    )
    .bind(compressed)
    .execute(&pool)
    .await
    .expect("insert traced run");

    let decoded = get_analysis_run_trace_in_pool(&pool, 1)
        .await
        .expect("load trace");
    assert_eq!(decoded, trace);
}

#[test]
fn source_group_input_is_trimmed_and_deduplicated() {
    let input = AnalysisSourceGroupInput::new(
        "  Core sources  ".to_string(),
        AnalysisSourceKind::Telegram,
        vec![4, 2, 4, -1, 2],
    )
    .expect("normalize source group");

    assert_eq!(input.name(), "Core sources");
    assert_eq!(input.source_kind(), AnalysisSourceKind::Telegram);
    assert_eq!(input.source_ids(), &[2, 4]);
}

#[test]
fn template_kind_validation_returns_typed_error() {
    let error = validate_template_kind("summary").expect_err("reject unsupported kind");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(error.message, "Unsupported template kind 'summary'");
}

#[test]
fn source_group_input_validation_returns_typed_error() {
    let error =
        AnalysisSourceGroupInput::new("  ".to_string(), AnalysisSourceKind::Telegram, vec![1])
            .expect_err("reject empty name");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(error.message, "Source group name cannot be empty");

    let error = AnalysisSourceGroupInput::new(
        "Core sources".to_string(),
        AnalysisSourceKind::Telegram,
        vec![0, -1],
    )
    .expect_err("reject all-nonpositive source ids");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(error.message, "Select at least one source for the group");
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
