use super::super::read_model::{
    prepare_analysis_run_detail, prepare_analysis_run_summaries, AnalysisRunListFilters,
};
use super::super::super::models::{
    AnalysisForeignLabelMatch, AnalysisForeignLabels, AnalysisSnapshotState, AnalysisSourceLabel,
};
use extractum_core::error::AppErrorKind;

fn empty_foreign_labels() -> AnalysisForeignLabels {
    AnalysisForeignLabels::new(Vec::new(), Vec::new()).expect("construct empty foreign labels")
}

fn source_labels(id: i64, title: &str) -> AnalysisForeignLabels {
    AnalysisForeignLabels::new(
        vec![AnalysisSourceLabel::new(id, Some(title.to_string()))
            .expect("construct source label")],
        Vec::new(),
    )
    .expect("construct source labels")
}

async fn finish_run_summaries(
    pool: &sqlx::SqlitePool,
    filters: AnalysisRunListFilters,
    matches: Vec<AnalysisForeignLabelMatch>,
    labels: AnalysisForeignLabels,
) -> Vec<super::super::super::models::AnalysisRunSummary> {
    let mut connection = pool.acquire().await.expect("acquire run-list connection");
    prepare_analysis_run_summaries(&mut connection, filters, matches)
        .await
        .expect("prepare run summaries")
        .finish(labels)
        .expect("finish run summaries")
}

async fn finish_run_detail(
    pool: &sqlx::SqlitePool,
    labels: AnalysisForeignLabels,
) -> super::super::super::models::AnalysisRunDetail {
    let mut connection = pool.acquire().await.expect("acquire run-detail connection");
    prepare_analysis_run_detail(&mut connection, 1)
        .await
        .expect("prepare run detail")
        .finish(labels)
        .expect("finish run detail")
        .expect("run exists")
}

#[derive(Clone)]
struct RunListFixture {
    id: i64,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    project_id: Option<i64>,
    scope_label_snapshot: &'static str,
    prompt_template_id: Option<i64>,
    provider_profile: &'static str,
    provider: &'static str,
    model: &'static str,
    status: &'static str,
    error: Option<&'static str>,
    created_at: i64,
}

impl RunListFixture {
    fn completed(id: i64, created_at: i64, label: &'static str) -> Self {
        Self {
            id,
            source_id: Some(1),
            source_group_id: None,
            project_id: None,
            scope_label_snapshot: label,
            prompt_template_id: Some(1),
            provider_profile: "default",
            provider: "gemini",
            model: "gemini-2.5-flash",
            status: "completed",
            error: None,
            created_at,
        }
    }
}

async fn run_list_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    sqlx::query(
        r#"
            CREATE TABLE analysis_source_groups (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )
            "#,
    )
    .execute(&pool)
    .await
    .expect("create groups");

    sqlx::query(
        r#"
            CREATE TABLE analysis_prompt_templates (
                id INTEGER PRIMARY KEY,
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

    sqlx::query(
        r#"
            CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY,
                run_type TEXT NOT NULL DEFAULT 'report',
                scope_type TEXT NOT NULL DEFAULT 'single_source',
                source_id INTEGER,
                source_group_id INTEGER,
                project_id INTEGER,
                period_from INTEGER NOT NULL DEFAULT 0,
                period_to INTEGER NOT NULL DEFAULT 0,
                output_language TEXT NOT NULL DEFAULT 'English',
                prompt_template_id INTEGER,
                prompt_template_version INTEGER NOT NULL DEFAULT 1,
                provider_profile TEXT NOT NULL DEFAULT 'default',
                provider TEXT NOT NULL DEFAULT 'gemini',
                model TEXT NOT NULL DEFAULT 'gemini-2.5-flash',
                youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
                telegram_history_scope TEXT NOT NULL DEFAULT 'current',
                status TEXT NOT NULL DEFAULT 'completed',
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

    sqlx::query(
        r#"
            CREATE TABLE analysis_run_messages (
                run_id INTEGER NOT NULL,
                ref TEXT NOT NULL
            )
            "#,
    )
    .execute(&pool)
    .await
    .expect("create run messages");

    sqlx::query("INSERT INTO analysis_source_groups (id, name) VALUES (10, 'Research Group')")
        .execute(&pool)
        .await
        .expect("insert group");
    sqlx::query(
            "INSERT INTO analysis_prompt_templates (id, name, template_kind, body, version, is_builtin, created_at, updated_at) VALUES (1, 'Weekly Digest', 'report', 'body', 1, 0, 1, 1), (2, 'Incident Review', 'report', 'body', 1, 0, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert templates");

    pool
}

async fn insert_run_list_fixture(pool: &sqlx::SqlitePool, fixture: RunListFixture) {
    sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id,
                run_type,
                scope_type,
                source_id,
                source_group_id,
                project_id,
                period_from,
                period_to,
                output_language,
                prompt_template_id,
                prompt_template_version,
                provider_profile,
                provider,
                model,
                youtube_corpus_mode,
                telegram_history_scope,
                status,
                result_markdown,
                trace_data_zstd,
                scope_label_snapshot,
                snapshot_captured_at,
                snapshot_error,
                error,
                created_at,
                completed_at
            )
            VALUES (?, 'report', ?, ?, ?, ?, 0, 0, 'English', ?, 1, ?, ?, ?, 'transcript_description', 'current', ?, 'Report', NULL, ?, NULL, NULL, ?, ?, ?)
            "#,
        )
        .bind(fixture.id)
        .bind(if fixture.project_id.is_some() {
            "project"
        } else if fixture.source_group_id.is_some() {
            "source_group"
        } else {
            "single_source"
        })
        .bind(fixture.source_id)
        .bind(fixture.source_group_id)
        .bind(fixture.project_id)
        .bind(fixture.prompt_template_id)
        .bind(fixture.provider_profile)
        .bind(fixture.provider)
        .bind(fixture.model)
        .bind(fixture.status)
        .bind(fixture.scope_label_snapshot)
        .bind(fixture.error)
        .bind(fixture.created_at)
        .bind(if fixture.status == "completed" {
            Some(fixture.created_at + 10)
        } else {
            None
        })
        .execute(pool)
        .await
        .expect("insert run fixture");
}

async fn insert_snapshot_run(
    pool: &sqlx::SqlitePool,
    status: &str,
    snapshot_captured_at: Option<&str>,
    snapshot_error: Option<&str>,
    snapshot_message_count: usize,
) {
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
            prompt_template_id,
            prompt_template_version,
            provider_profile,
            provider,
            model,
            youtube_corpus_mode,
            telegram_history_scope,
            status,
            result_markdown,
            trace_data_zstd,
            scope_label_snapshot,
            snapshot_captured_at,
            snapshot_error,
            created_at,
            completed_at
        )
        VALUES (
            1, 'report', 'source_group', 9, 1700000000, 1800000000, 'English',
            1, 1, 'default', 'gemini', 'gemini-2.5-flash',
            'transcript_description_comments', 'current', ?, 'Saved report',
            x'010203', 'Frozen group', ?, ?, 1710000500, 1710000600
        )
        "#,
    )
    .bind(status)
    .bind(snapshot_captured_at)
    .bind(snapshot_error)
    .execute(pool)
    .await
    .expect("insert snapshot run");

    for index in 0..snapshot_message_count {
        sqlx::query("INSERT INTO analysis_run_messages (run_id, ref) VALUES (1, ?)")
            .bind(format!("s2-i{}", index + 1))
            .execute(pool)
            .await
            .expect("insert snapshot message");
    }
}

async fn prepared_sample_summary(
    pool: &sqlx::SqlitePool,
) -> super::super::super::models::AnalysisRunSummary {
    finish_run_summaries(
        pool,
        AnalysisRunListFilters::for_analysis(
            None, None, 50, None, None, None, None, None, None, None,
        )
        .expect("construct sample filters"),
        Vec::new(),
        empty_foreign_labels(),
    )
    .await
    .into_iter()
    .next()
    .expect("sample summary")
}

#[tokio::test]
async fn list_analysis_run_summaries_applies_query_before_limit() {
    let pool = run_list_pool().await;
    insert_run_list_fixture(
        &pool,
        RunListFixture::completed(1, 300, "Newest irrelevant"),
    )
    .await;
    insert_run_list_fixture(
        &pool,
        RunListFixture::completed(2, 200, "Older target nebula"),
    )
    .await;
    insert_run_list_fixture(
        &pool,
        RunListFixture::completed(3, 100, "Oldest target nebula"),
    )
    .await;

    let runs = finish_run_summaries(
        &pool,
        AnalysisRunListFilters::for_analysis(
            None,
            None,
            1,
            Some("nebula".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("construct query filters"),
        Vec::new(),
        empty_foreign_labels(),
    )
    .await;

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![2]);
}

#[tokio::test]
async fn list_analysis_run_summaries_combines_scope_and_field_filters() {
    let pool = run_list_pool().await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 1,
            source_id: Some(1),
            provider: "gemini",
            model: "gemini-2.5-pro",
            created_at: 300,
            ..RunListFixture::completed(1, 300, "Source match")
        },
    )
    .await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 2,
            source_id: Some(2),
            provider: "openai",
            model: "gpt-5",
            created_at: 200,
            ..RunListFixture::completed(2, 200, "Other source")
        },
    )
    .await;

    let runs = finish_run_summaries(
        &pool,
        AnalysisRunListFilters::for_analysis(
            Some(1),
            None,
            50,
            None,
            None,
            Some("GEM".to_string()),
            Some("pro".to_string()),
            None,
            None,
            None,
        )
        .expect("construct scope and field filters"),
        Vec::new(),
        source_labels(1, "Alpha Source"),
    )
    .await;

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
}

#[tokio::test]
async fn list_analysis_run_summaries_filters_source_groups_and_template_names() {
    let pool = run_list_pool().await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 1,
            source_id: None,
            source_group_id: Some(10),
            scope_label_snapshot: "Research Group",
            prompt_template_id: Some(2),
            created_at: 300,
            ..RunListFixture::completed(1, 300, "Research Group")
        },
    )
    .await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 2,
            source_id: Some(1),
            source_group_id: None,
            prompt_template_id: Some(1),
            created_at: 200,
            ..RunListFixture::completed(2, 200, "Single source")
        },
    )
    .await;

    let runs = finish_run_summaries(
        &pool,
        AnalysisRunListFilters::for_analysis(
            None,
            Some(10),
            50,
            None,
            None,
            None,
            None,
            Some("incident".to_string()),
            None,
            None,
        )
        .expect("construct source-group template filters"),
        Vec::new(),
        empty_foreign_labels(),
    )
    .await;

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
    assert_eq!(runs[0].source_group_name.as_deref(), Some("Research Group"));
}

#[tokio::test]
async fn list_analysis_run_summaries_rejects_both_scope_ids() {
    let error = AnalysisRunListFilters::for_analysis(
        Some(1),
        Some(10),
        50,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect_err("both scope ids should fail");

    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(
        error.message,
        "Pass only one of source_id, source_group_id, or project_id"
    );
}

#[tokio::test]
async fn list_analysis_run_summaries_filters_status_and_dates() {
    let pool = run_list_pool().await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 1,
            status: "completed",
            created_at: 1_704_153_600,
            ..RunListFixture::completed(1, 1_704_153_600, "Jan 2")
        },
    )
    .await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 2,
            status: "failed",
            created_at: 1_704_240_000,
            ..RunListFixture::completed(2, 1_704_240_000, "Jan 3")
        },
    )
    .await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 3,
            status: "running",
            created_at: 1_704_326_400,
            ..RunListFixture::completed(3, 1_704_326_400, "Jan 4")
        },
    )
    .await;

    let completed = finish_run_summaries(
        &pool,
        AnalysisRunListFilters::for_analysis(
            None,
            None,
            50,
            None,
            Some("completed".to_string()),
            None,
            None,
            None,
            Some("2024-01-02".to_string()),
            Some("2024-01-02".to_string()),
        )
        .expect("construct completed date filters"),
        Vec::new(),
        empty_foreign_labels(),
    )
    .await;
    assert_eq!(
        completed.iter().map(|run| run.id).collect::<Vec<_>>(),
        vec![1],
    );

    let active = finish_run_summaries(
        &pool,
        AnalysisRunListFilters::for_analysis(
            None,
            None,
            50,
            None,
            Some("queued_running".to_string()),
            None,
            None,
            None,
            Some("invalid".to_string()),
            Some("2024-01-04".to_string()),
        )
        .expect("construct active date filters"),
        Vec::new(),
        empty_foreign_labels(),
    )
    .await;
    assert_eq!(active.iter().map(|run| run.id).collect::<Vec<_>>(), vec![3]);
}

#[tokio::test]
async fn list_analysis_run_summaries_escapes_literal_like_characters() {
    let pool = run_list_pool().await;
    insert_run_list_fixture(&pool, RunListFixture::completed(1, 300, "100%_literal")).await;
    insert_run_list_fixture(
        &pool,
        RunListFixture::completed(2, 200, "100 percent literal"),
    )
    .await;

    let runs = finish_run_summaries(
        &pool,
        AnalysisRunListFilters::for_analysis(
            None,
            None,
            50,
            Some("100%_literal".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("construct literal query filters"),
        Vec::new(),
        empty_foreign_labels(),
    )
    .await;

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
}

#[tokio::test]
async fn resolve_run_scope_label_prefers_frozen_value() {
    let pool = run_list_pool().await;
    insert_snapshot_run(&pool, "completed", Some("2026-05-18T10:00:00Z"), None, 2).await;
    let detail = finish_run_detail(&pool, empty_foreign_labels()).await;

    assert_eq!(detail.scope_label, "Frozen group");
}

#[tokio::test]
async fn map_run_summary_exposes_frozen_scope_label() {
    let pool = run_list_pool().await;
    insert_snapshot_run(&pool, "completed", Some("2026-05-18T10:00:00Z"), None, 2).await;
    let summary = prepared_sample_summary(&pool).await;

    assert_eq!(summary.scope_label, "Frozen group");
}

#[tokio::test]
async fn map_run_summary_exposes_captured_snapshot_state() {
    let pool = run_list_pool().await;
    insert_snapshot_run(&pool, "completed", Some("2026-05-18T10:00:00Z"), None, 2).await;
    let summary = prepared_sample_summary(&pool).await;

    assert_eq!(
        summary.snapshot_state,
        Some(AnalysisSnapshotState::Captured)
    );
    assert_eq!(
        summary.snapshot_captured_at.as_deref(),
        Some("2026-05-18T10:00:00Z")
    );
    assert_eq!(summary.snapshot_error, None);
}

#[tokio::test]
async fn completed_run_without_capture_marker_is_capture_failed() {
    let pool = run_list_pool().await;
    insert_snapshot_run(&pool, "completed", None, None, 0).await;
    let detail = finish_run_detail(&pool, empty_foreign_labels()).await;

    assert_eq!(
        detail.snapshot_state,
        Some(AnalysisSnapshotState::CaptureFailed)
    );
    assert_eq!(detail.snapshot_captured_at, None);
    assert_eq!(detail.snapshot_error, None);
}

#[tokio::test]
async fn map_run_summary_exposes_capture_failed_snapshot_state() {
    let pool = run_list_pool().await;
    insert_snapshot_run(&pool, "failed", None, Some("Snapshot capture failed"), 0).await;
    let summary = prepared_sample_summary(&pool).await;

    assert_eq!(
        summary.snapshot_state,
        Some(AnalysisSnapshotState::CaptureFailed)
    );
    assert_eq!(
        summary.snapshot_error.as_deref(),
        Some("Snapshot capture failed")
    );
}

#[tokio::test]
async fn map_run_summary_exposes_null_snapshot_state_for_active_runs_before_capture() {
    let pool = run_list_pool().await;
    insert_snapshot_run(&pool, "running", None, None, 0).await;
    let summary = prepared_sample_summary(&pool).await;

    assert_eq!(summary.snapshot_state, None);
}

#[tokio::test]
async fn failed_terminal_run_without_capture_marker_is_capture_failed() {
    let pool = run_list_pool().await;
    insert_snapshot_run(&pool, "cancelled", None, None, 0).await;
    let summary = prepared_sample_summary(&pool).await;

    assert_eq!(
        summary.snapshot_state,
        Some(AnalysisSnapshotState::CaptureFailed)
    );
}

#[tokio::test]
async fn map_run_summary_exposes_youtube_corpus_mode() {
    let pool = run_list_pool().await;
    insert_snapshot_run(&pool, "completed", Some("2026-05-18T10:00:00Z"), None, 2).await;
    let summary = prepared_sample_summary(&pool).await;

    assert_eq!(
        summary.youtube_corpus_mode,
        "transcript_description_comments"
    );
}

#[tokio::test]
async fn map_run_detail_exposes_youtube_corpus_mode() {
    let pool = run_list_pool().await;
    insert_snapshot_run(&pool, "completed", Some("2026-05-18T10:00:00Z"), None, 2).await;
    let detail = finish_run_detail(&pool, empty_foreign_labels()).await;

    assert_eq!(
        detail.youtube_corpus_mode,
        "transcript_description_comments"
    );
}

#[test]
fn analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes() {
    let analysis = AnalysisRunListFilters::for_analysis(
        Some(11),
        None,
        250,
        Some("  alpha   beta  ".to_string()),
        Some(" all ".to_string()),
        Some(" gemini ".to_string()),
        None,
        None,
        Some(" 2026-07-01 ".to_string()),
        None,
    )
    .expect("construct analysis filters");
    let project = AnalysisRunListFilters::for_project(19, 500);

    assert_eq!(
        format!("{analysis:?}"),
        concat!(
            "AnalysisRunListFilters { source_id: Some(11), source_group_id: None, ",
            "project_id: None, limit: 100, query: Some(\"alpha   beta\"), ",
            "status: Some(\"all\"), provider: Some(\"gemini\"), model: None, ",
            "template: None, date_from: Some(\"2026-07-01\"), date_to: None, ",
            "foreign_label_search_terms: [\"alpha\", \"beta\"] }"
        ),
        "RED: CP2 run list filters"
    );
    assert_eq!(
        analysis.foreign_label_search_terms(),
        &["alpha".to_string(), "beta".to_string()]
    );
    assert_eq!(
        format!("{project:?}"),
        concat!(
            "AnalysisRunListFilters { source_id: None, source_group_id: None, ",
            "project_id: Some(19), limit: 100, query: None, status: Some(\"all\"), ",
            "provider: None, model: None, template: None, date_from: None, ",
            "date_to: None, foreign_label_search_terms: [] }"
        )
    );

    let error = AnalysisRunListFilters::for_analysis(
        Some(1),
        Some(2),
        20,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect_err("multiple analysis scopes rejected");
    assert_eq!(error.kind, AppErrorKind::Validation);
    assert_eq!(
        error.message,
        "Pass only one of source_id, source_group_id, or project_id"
    );
}
