use super::super::{
    list_analysis_run_summaries, map_run_detail, map_run_summary, resolve_run_scope_label,
    AnalysisRunListFilters,
};
use crate::analysis::models::{AnalysisRunDetail, AnalysisRunRow};
use crate::error::AppErrorKind;

fn sample_run_row() -> AnalysisRunRow {
    AnalysisRunRow {
        id: 1,
        run_type: "report".to_string(),
        scope_type: "source_group".to_string(),
        source_id: None,
        source_title: None,
        source_group_id: Some(9),
        source_group_name: Some("Live group".to_string()),
        project_id: None,
        project_name: None,
        period_from: 1_700_000_000,
        period_to: 1_800_000_000,
        output_language: "English".to_string(),
        prompt_template_id: Some(1),
        prompt_template_name: Some("Default".to_string()),
        prompt_template_version: 1,
        provider_profile: "default".to_string(),
        provider: "gemini".to_string(),
        model: "gemini-2.5-flash".to_string(),
        youtube_corpus_mode: "transcript_description_comments".to_string(),
        telegram_history_scope: crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT.to_string(),
        status: "completed".to_string(),
        result_markdown: Some("Saved report".to_string()),
        trace_data_zstd: Some(vec![1, 2, 3]),
        scope_label_snapshot: Some("Frozen group".to_string()),
        snapshot_captured_at: Some("2026-05-18T10:00:00Z".to_string()),
        snapshot_error: None,
        snapshot_message_count: 2,
        error: None,
        created_at: 1_710_000_500,
        completed_at: Some(1_710_000_600),
    }
}

fn sample_run() -> AnalysisRunDetail {
    map_run_detail(sample_run_row())
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
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL
            )
            "#,
    )
    .execute(&pool)
    .await
    .expect("create sources");

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
            CREATE TABLE projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )
            "#,
    )
    .execute(&pool)
    .await
    .expect("create projects");

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

    sqlx::query("INSERT INTO sources (id, title) VALUES (1, 'Alpha Source'), (2, 'Beta Source')")
        .execute(&pool)
        .await
        .expect("insert sources");
    sqlx::query("INSERT INTO analysis_source_groups (id, name) VALUES (10, 'Research Group')")
        .execute(&pool)
        .await
        .expect("insert group");
    sqlx::query("INSERT INTO projects (id, name) VALUES (7, 'Alpha Project'), (8, 'Beta Project')")
        .execute(&pool)
        .await
        .expect("insert projects");
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

    let runs = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            source_id: None,
            source_group_id: None,
            limit: 1,
            query: Some("nebula".to_string()),
            ..AnalysisRunListFilters::default()
        },
    )
    .await
    .expect("list runs");

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

    let runs = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            source_id: Some(1),
            source_group_id: None,
            limit: 50,
            provider: Some("GEM".to_string()),
            model: Some("pro".to_string()),
            ..AnalysisRunListFilters::default()
        },
    )
    .await
    .expect("list runs");

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

    let runs = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            source_id: None,
            source_group_id: Some(10),
            limit: 50,
            template: Some("incident".to_string()),
            ..AnalysisRunListFilters::default()
        },
    )
    .await
    .expect("list runs");

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
    assert_eq!(runs[0].source_group_name.as_deref(), Some("Research Group"));
}

#[tokio::test]
async fn list_analysis_run_summaries_filters_project_runs() {
    let pool = run_list_pool().await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 1,
            source_id: None,
            source_group_id: None,
            project_id: Some(7),
            scope_label_snapshot: "Alpha Project",
            created_at: 300,
            ..RunListFixture::completed(1, 300, "Alpha Project")
        },
    )
    .await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 2,
            source_id: None,
            source_group_id: None,
            project_id: Some(8),
            scope_label_snapshot: "Beta Project",
            created_at: 200,
            ..RunListFixture::completed(2, 200, "Beta Project")
        },
    )
    .await;

    let runs = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            project_id: Some(7),
            limit: 50,
            ..AnalysisRunListFilters::default()
        },
    )
    .await
    .expect("list project runs");

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
    assert_eq!(runs[0].project_name.as_deref(), Some("Alpha Project"));
}

#[tokio::test]
async fn list_analysis_run_summaries_rejects_both_scope_ids() {
    let pool = run_list_pool().await;

    let error = match list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            source_id: Some(1),
            source_group_id: Some(10),
            limit: 50,
            ..AnalysisRunListFilters::default()
        },
    )
    .await
    {
        Ok(_) => panic!("both scope ids should fail"),
        Err(error) => error,
    };

    assert_eq!(error.kind, AppErrorKind::Validation);
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

    let completed = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            limit: 50,
            status: Some("completed".to_string()),
            date_from: Some("2024-01-02".to_string()),
            date_to: Some("2024-01-02".to_string()),
            ..AnalysisRunListFilters::default()
        },
    )
    .await
    .expect("list completed");
    assert_eq!(
        completed.iter().map(|run| run.id).collect::<Vec<_>>(),
        vec![1],
    );

    let active = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            limit: 50,
            status: Some("queued_running".to_string()),
            date_from: Some("invalid".to_string()),
            date_to: Some("2024-01-04".to_string()),
            ..AnalysisRunListFilters::default()
        },
    )
    .await
    .expect("list active");
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

    let runs = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            limit: 50,
            query: Some("100%_literal".to_string()),
            ..AnalysisRunListFilters::default()
        },
    )
    .await
    .expect("list literal percent underscore");

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
}

#[tokio::test]
async fn list_analysis_run_summaries_matches_all_query_terms_across_any_field() {
    let pool = run_list_pool().await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 1,
            source_id: Some(1),
            source_group_id: None,
            provider_profile: "research-profile",
            error: Some("quota exhausted"),
            created_at: 300,
            ..RunListFixture::completed(1, 300, "Plain label")
        },
    )
    .await;
    insert_run_list_fixture(
        &pool,
        RunListFixture {
            id: 2,
            source_id: Some(2),
            provider_profile: "research-profile",
            error: Some("different failure"),
            created_at: 200,
            ..RunListFixture::completed(2, 200, "Plain label")
        },
    )
    .await;

    let runs = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters {
            limit: 50,
            query: Some("alpha quota".to_string()),
            ..AnalysisRunListFilters::default()
        },
    )
    .await
    .expect("list terms");

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
}

#[test]
fn resolve_run_scope_label_prefers_frozen_value() {
    let run = sample_run();
    assert_eq!(resolve_run_scope_label(&run), "Frozen group");
}

#[test]
fn map_run_summary_exposes_frozen_scope_label() {
    let summary = map_run_summary(sample_run_row());
    assert_eq!(summary.scope_label, "Frozen group");
}

#[test]
fn map_run_summary_exposes_captured_snapshot_state() {
    let summary = map_run_summary(sample_run_row());

    assert_eq!(
        summary.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::Captured)
    );
    assert_eq!(
        summary.snapshot_captured_at.as_deref(),
        Some("2026-05-18T10:00:00Z")
    );
    assert_eq!(summary.snapshot_error, None);
}

#[test]
fn completed_run_without_capture_marker_is_capture_failed() {
    let mut row = sample_run_row();
    row.snapshot_captured_at = None;
    row.snapshot_error = None;
    row.snapshot_message_count = 0;
    row.status = crate::analysis::ANALYSIS_STATUS_COMPLETED.to_string();

    let detail = map_run_detail(row);

    assert_eq!(
        detail.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
    );
    assert_eq!(detail.snapshot_captured_at, None);
    assert_eq!(detail.snapshot_error, None);
}

#[test]
fn map_run_summary_exposes_capture_failed_snapshot_state() {
    let mut row = sample_run_row();
    row.snapshot_captured_at = None;
    row.snapshot_error = Some("Snapshot capture failed".to_string());
    row.snapshot_message_count = 0;
    row.status = crate::analysis::ANALYSIS_STATUS_FAILED.to_string();

    let summary = map_run_summary(row);

    assert_eq!(
        summary.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
    );
    assert_eq!(
        summary.snapshot_error.as_deref(),
        Some("Snapshot capture failed")
    );
}

#[test]
fn map_run_summary_exposes_null_snapshot_state_for_active_runs_before_capture() {
    let mut row = sample_run_row();
    row.snapshot_captured_at = None;
    row.snapshot_error = None;
    row.snapshot_message_count = 0;
    row.status = crate::analysis::ANALYSIS_STATUS_RUNNING.to_string();

    let summary = map_run_summary(row);

    assert_eq!(summary.snapshot_state, None);
}

#[test]
fn failed_terminal_run_without_capture_marker_is_capture_failed() {
    let mut row = sample_run_row();
    row.snapshot_captured_at = None;
    row.snapshot_error = None;
    row.snapshot_message_count = 0;
    row.status = crate::analysis::ANALYSIS_STATUS_CANCELLED.to_string();

    let summary = map_run_summary(row);

    assert_eq!(
        summary.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
    );
}

#[test]
fn map_run_summary_exposes_youtube_corpus_mode() {
    let summary = map_run_summary(sample_run_row());
    assert_eq!(
        summary.youtube_corpus_mode,
        "transcript_description_comments"
    );
}

#[test]
fn map_run_detail_exposes_youtube_corpus_mode() {
    let detail = map_run_detail(sample_run_row());
    assert_eq!(
        detail.youtube_corpus_mode,
        "transcript_description_comments"
    );
}
