use sqlx::{Pool, Sqlite};

use super::corpus::YoutubeCorpusMode;
use super::models::AnalysisPromptTemplate;
use super::{now_secs, ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING};
use crate::error::{AppError, AppResult};

mod read_model;
mod setup;
mod snapshot;

pub(crate) use self::read_model::{
    fetch_run_row, list_analysis_run_summaries, map_run_detail, map_run_summary,
    resolve_run_scope_label, AnalysisRunListFilters,
};
pub(crate) use self::setup::{
    ensure_builtin_report_template, ensure_sources_exist, fetch_prompt_template, fetch_source_group,
};
#[allow(unused_imports)]
pub(crate) use self::snapshot::{
    capture_run_snapshot, mark_run_capture_failed, persist_run_snapshot, sanitize_provider_error,
    sanitize_snapshot_error,
};

pub(crate) struct DuplicateRunLookup<'a> {
    pub(crate) scope_type: &'a str,
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: &'a str,
    pub(crate) prompt_template_id: i64,
    pub(crate) provider_profile: &'a str,
    pub(crate) model: &'a str,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
    pub(crate) telegram_history_scope: &'a str,
}

pub(crate) async fn find_active_duplicate_run(
    pool: &Pool<Sqlite>,
    lookup: &DuplicateRunLookup<'_>,
) -> AppResult<Option<i64>> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM analysis_runs
        WHERE run_type = ?
          AND scope_type = ?
          AND (source_id = ? OR (source_id IS NULL AND ? IS NULL))
          AND (source_group_id = ? OR (source_group_id IS NULL AND ? IS NULL))
          AND (project_id = ? OR (project_id IS NULL AND ? IS NULL))
          AND period_from = ?
          AND period_to = ?
          AND output_language = ?
          AND prompt_template_id = ?
          AND provider_profile = ?
          AND model = ?
          AND youtube_corpus_mode = ?
          AND COALESCE(telegram_history_scope, 'current') = ?
          AND status IN (?, ?)
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(lookup.scope_type)
    .bind(lookup.source_id)
    .bind(lookup.source_id)
    .bind(lookup.source_group_id)
    .bind(lookup.source_group_id)
    .bind(lookup.project_id)
    .bind(lookup.project_id)
    .bind(lookup.period_from)
    .bind(lookup.period_to)
    .bind(lookup.output_language)
    .bind(lookup.prompt_template_id)
    .bind(lookup.provider_profile)
    .bind(lookup.model)
    .bind(lookup.youtube_corpus_mode.as_wire())
    .bind(lookup.telegram_history_scope)
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(ANALYSIS_STATUS_RUNNING)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) struct AnalysisRunInsert<'a> {
    pub(crate) scope_type: &'a str,
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: &'a str,
    pub(crate) prompt_template: &'a AnalysisPromptTemplate,
    pub(crate) provider_profile: &'a str,
    pub(crate) provider: &'a str,
    pub(crate) model: &'a str,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
    pub(crate) telegram_history_scope: &'a str,
    pub(crate) scope_label_snapshot: Option<&'a str>,
}

pub(crate) async fn insert_analysis_run(
    pool: &Pool<Sqlite>,
    insert: &AnalysisRunInsert<'_>,
) -> AppResult<i64> {
    let created_at = now_secs();
    sqlx::query_scalar(
        r#"
        INSERT INTO analysis_runs (
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
            scope_label_snapshot,
            created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(insert.scope_type)
    .bind(insert.source_id)
    .bind(insert.source_group_id)
    .bind(insert.project_id)
    .bind(insert.period_from)
    .bind(insert.period_to)
    .bind(insert.output_language)
    .bind(insert.prompt_template.id)
    .bind(insert.prompt_template.version)
    .bind(insert.provider_profile)
    .bind(insert.provider)
    .bind(insert.model)
    .bind(insert.youtube_corpus_mode.as_wire())
    .bind(insert.telegram_history_scope)
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(insert.scope_label_snapshot)
    .bind(created_at)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn set_run_status(
    pool: &Pool<Sqlite>,
    run_id: i64,
    status: &str,
    result_markdown: Option<&str>,
    trace_data_zstd: Option<&[u8]>,
    error: Option<&str>,
    completed_at: Option<i64>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET
            status = ?,
            result_markdown = COALESCE(?, result_markdown),
            trace_data_zstd = COALESCE(?, trace_data_zstd),
            error = ?,
            completed_at = ?
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(result_markdown)
    .bind(trace_data_zstd)
    .bind(error)
    .bind(completed_at)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn delete_saved_run(pool: &Pool<Sqlite>, run_id: i64) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    sqlx::query("DELETE FROM analysis_chat_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    sqlx::query("DELETE FROM analysis_run_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    let deleted = sqlx::query("DELETE FROM analysis_runs WHERE id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?
        .rows_affected();

    if deleted == 0 {
        return Err(AppError::not_found(format!(
            "Analysis run {run_id} not found"
        )));
    }

    tx.commit().await.map_err(AppError::database)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        capture_run_snapshot, delete_saved_run, ensure_sources_exist, fetch_prompt_template,
        list_analysis_run_summaries, map_run_detail, map_run_summary, mark_run_capture_failed,
        resolve_run_scope_label, sanitize_provider_error, sanitize_snapshot_error, set_run_status,
        AnalysisRunListFilters,
    };
    use crate::analysis::models::{
        AnalysisPromptTemplate, AnalysisRunDetail, AnalysisRunRow, CorpusMessage,
    };
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
            telegram_history_scope: crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT
                .to_string(),
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

        sqlx::query(
            "INSERT INTO sources (id, title) VALUES (1, 'Alpha Source'), (2, 'Beta Source')",
        )
        .execute(&pool)
        .await
        .expect("insert sources");
        sqlx::query("INSERT INTO analysis_source_groups (id, name) VALUES (10, 'Research Group')")
            .execute(&pool)
            .await
            .expect("insert group");
        sqlx::query(
            "INSERT INTO projects (id, name) VALUES (7, 'Alpha Project'), (8, 'Beta Project')",
        )
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

    async fn template_store_pool() -> sqlx::SqlitePool {
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
        pool
    }

    async fn source_store_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query("CREATE TABLE sources (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("create sources");
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
        let second =
            capture_run_snapshot(&pool, 1, "Frozen scope", &[strict_snapshot_message("b")])
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
    async fn ensure_sources_exist_returns_typed_not_found_error() {
        let pool = source_store_pool().await;

        let error = ensure_sources_exist(&pool, &[7])
            .await
            .expect_err("missing source should fail");

        assert_eq!(error.kind, AppErrorKind::NotFound);
        assert_eq!(error.message, "Source 7 not found");
    }

    #[tokio::test]
    async fn fetch_prompt_template_returns_typed_not_found_error() {
        let pool = template_store_pool().await;

        let error = match fetch_prompt_template(&pool, 99).await {
            Ok(_) => panic!("missing prompt template should fail"),
            Err(error) => error,
        };

        assert_eq!(error.kind, AppErrorKind::NotFound);
        assert_eq!(error.message, "Analysis prompt template 99 not found");
    }

    #[tokio::test]
    async fn delete_saved_run_returns_typed_not_found_error() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query("CREATE TABLE analysis_runs (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("create runs");
        sqlx::query(
            "CREATE TABLE analysis_chat_messages (id INTEGER PRIMARY KEY, run_id INTEGER NOT NULL)",
        )
        .execute(&pool)
        .await
        .expect("create chat messages");
        sqlx::query(
            "CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL, ref TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .expect("create run messages");

        let error = delete_saved_run(&pool, 42)
            .await
            .expect_err("missing run should fail");

        assert_eq!(error.kind, AppErrorKind::NotFound);
        assert_eq!(error.message, "Analysis run 42 not found");
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

    #[tokio::test]
    async fn provider_failure_status_update_does_not_write_snapshot_error() {
        let pool = snapshot_store_pool().await;
        sqlx::query(
            "UPDATE analysis_runs SET snapshot_captured_at = '2026-05-18T10:00:00Z' WHERE id = 1",
        )
        .execute(&pool)
        .await
        .expect("mark captured");

        set_run_status(
            &pool,
            1,
            crate::analysis::ANALYSIS_STATUS_FAILED,
            None,
            None,
            Some("Provider network failed"),
            Some(1_710_000_500),
        )
        .await
        .expect("mark provider failed");

        let snapshot_error: Option<String> =
            sqlx::query_scalar("SELECT snapshot_error FROM analysis_runs WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("load snapshot_error");
        assert_eq!(snapshot_error, None);
    }

    #[tokio::test]
    async fn cancellation_after_capture_does_not_write_snapshot_error() {
        let pool = snapshot_store_pool().await;
        sqlx::query(
            "UPDATE analysis_runs SET snapshot_captured_at = '2026-05-18T10:00:00Z' WHERE id = 1",
        )
        .execute(&pool)
        .await
        .expect("mark captured");

        set_run_status(
            &pool,
            1,
            crate::analysis::ANALYSIS_STATUS_CANCELLED,
            None,
            None,
            Some("Analysis run cancelled."),
            Some(1_710_000_500),
        )
        .await
        .expect("mark cancelled");

        let snapshot_error: Option<String> =
            sqlx::query_scalar("SELECT snapshot_error FROM analysis_runs WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("load snapshot_error");
        assert_eq!(snapshot_error, None);
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

    #[tokio::test]
    async fn insert_analysis_run_persists_youtube_corpus_mode() {
        use super::{insert_analysis_run, AnalysisRunInsert};
        use crate::analysis::corpus::YoutubeCorpusMode;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
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

        let template = AnalysisPromptTemplate {
            id: 5,
            name: "Report".to_string(),
            template_kind: "report".to_string(),
            body: "Body".to_string(),
            version: 3,
            is_builtin: false,
            created_at: 1,
            updated_at: 1,
        };

        let run_id = insert_analysis_run(
            &pool,
            &AnalysisRunInsert {
                scope_type: "single_source",
                source_id: Some(7),
                source_group_id: None,
                project_id: None,
                period_from: 10,
                period_to: 20,
                output_language: "English",
                prompt_template: &template,
                provider_profile: "default",
                provider: "gemini",
                model: "gemini-2.5-flash",
                youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescriptionComments,
                telegram_history_scope: crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT,
                scope_label_snapshot: None,
            },
        )
        .await
        .expect("insert run");

        let mode = sqlx::query_scalar::<_, String>(
            "SELECT youtube_corpus_mode FROM analysis_runs WHERE id = ?",
        )
        .bind(run_id)
        .fetch_one(&pool)
        .await
        .expect("load mode");

        assert_eq!(mode, "transcript_description_comments");
    }

    #[tokio::test]
    async fn duplicate_lookup_matches_telegram_history_scope() {
        use super::{
            find_active_duplicate_run, insert_analysis_run, AnalysisRunInsert, DuplicateRunLookup,
        };
        use crate::analysis::corpus::YoutubeCorpusMode;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
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

        let template = AnalysisPromptTemplate {
            id: 5,
            name: "Report".to_string(),
            template_kind: "report".to_string(),
            body: "Body".to_string(),
            version: 3,
            is_builtin: false,
            created_at: 1,
            updated_at: 1,
        };

        let current_run_id = insert_analysis_run(
            &pool,
            &AnalysisRunInsert {
                scope_type: "single_source",
                source_id: Some(7),
                source_group_id: None,
                project_id: None,
                period_from: 10,
                period_to: 20,
                output_language: "English",
                prompt_template: &template,
                provider_profile: "default",
                provider: "gemini",
                model: "gemini-2.5-flash",
                youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
                telegram_history_scope: crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT,
                scope_label_snapshot: None,
            },
        )
        .await
        .expect("insert current run");
        sqlx::query("UPDATE analysis_runs SET created_at = 1 WHERE id = ?")
            .bind(current_run_id)
            .execute(&pool)
            .await
            .expect("stabilize current created_at");

        let migrated_run_id = insert_analysis_run(
            &pool,
            &AnalysisRunInsert {
                scope_type: "single_source",
                source_id: Some(7),
                source_group_id: None,
                project_id: None,
                period_from: 10,
                period_to: 20,
                output_language: "English",
                prompt_template: &template,
                provider_profile: "default",
                provider: "gemini",
                model: "gemini-2.5-flash",
                youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
                telegram_history_scope:
                    crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED,
                scope_label_snapshot: None,
            },
        )
        .await
        .expect("insert migrated run");
        sqlx::query("UPDATE analysis_runs SET created_at = 2 WHERE id = ?")
            .bind(migrated_run_id)
            .execute(&pool)
            .await
            .expect("stabilize migrated created_at");

        let lookup = |telegram_history_scope| DuplicateRunLookup {
            scope_type: "single_source",
            source_id: Some(7),
            source_group_id: None,
            project_id: None,
            period_from: 10,
            period_to: 20,
            output_language: "English",
            prompt_template_id: 5,
            provider_profile: "default",
            model: "gemini-2.5-flash",
            youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
            telegram_history_scope,
        };

        let current_duplicate = find_active_duplicate_run(
            &pool,
            &lookup(crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT),
        )
        .await
        .expect("current duplicate lookup");
        let current_plus_migrated_duplicate = find_active_duplicate_run(
            &pool,
            &lookup(crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED),
        )
        .await
        .expect("migrated duplicate lookup");

        assert_eq!(current_duplicate, Some(current_run_id));
        assert_eq!(current_plus_migrated_duplicate, Some(migrated_run_id));
    }

    #[tokio::test]
    async fn duplicate_lookup_keeps_project_and_source_group_scopes_separate() {
        use super::{
            find_active_duplicate_run, insert_analysis_run, AnalysisRunInsert, DuplicateRunLookup,
        };
        use crate::analysis::corpus::YoutubeCorpusMode;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
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

        let template = AnalysisPromptTemplate {
            id: 5,
            name: "Report".to_string(),
            template_kind: "report".to_string(),
            body: "Body".to_string(),
            version: 3,
            is_builtin: false,
            created_at: 1,
            updated_at: 1,
        };

        let group_run_id = insert_analysis_run(
            &pool,
            &AnalysisRunInsert {
                scope_type: "source_group",
                source_id: None,
                source_group_id: Some(7),
                project_id: None,
                period_from: 10,
                period_to: 20,
                output_language: "English",
                prompt_template: &template,
                provider_profile: "default",
                provider: "gemini",
                model: "gemini-2.5-flash",
                youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
                telegram_history_scope: crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT,
                scope_label_snapshot: Some("Group"),
            },
        )
        .await
        .expect("insert group run");
        let project_run_id = insert_analysis_run(
            &pool,
            &AnalysisRunInsert {
                scope_type: "project",
                source_id: None,
                source_group_id: None,
                project_id: Some(7),
                period_from: 10,
                period_to: 20,
                output_language: "English",
                prompt_template: &template,
                provider_profile: "default",
                provider: "gemini",
                model: "gemini-2.5-flash",
                youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
                telegram_history_scope: crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT,
                scope_label_snapshot: Some("Project"),
            },
        )
        .await
        .expect("insert project run");

        let project_duplicate = find_active_duplicate_run(
            &pool,
            &DuplicateRunLookup {
                scope_type: "project",
                source_id: None,
                source_group_id: None,
                project_id: Some(7),
                period_from: 10,
                period_to: 20,
                output_language: "English",
                prompt_template_id: 5,
                provider_profile: "default",
                model: "gemini-2.5-flash",
                youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
                telegram_history_scope: crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT,
            },
        )
        .await
        .expect("project duplicate lookup");

        assert_eq!(project_duplicate, Some(project_run_id));
        assert_ne!(project_duplicate, Some(group_run_id));
    }

    #[tokio::test]
    async fn delete_saved_run_removes_run_and_saved_children() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            "CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY,
                snapshot_captured_at TEXT,
                snapshot_error TEXT
            )",
        )
        .execute(&pool)
        .await
        .expect("create runs");
        sqlx::query(
            "CREATE TABLE analysis_chat_messages (id INTEGER PRIMARY KEY, run_id INTEGER NOT NULL)",
        )
        .execute(&pool)
        .await
        .expect("create chat messages");
        sqlx::query(
            "CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL, ref TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .expect("create run messages");

        sqlx::query("INSERT INTO analysis_runs (id) VALUES (42)")
            .execute(&pool)
            .await
            .expect("insert run");
        sqlx::query("INSERT INTO analysis_chat_messages (run_id) VALUES (42)")
            .execute(&pool)
            .await
            .expect("insert chat");
        sqlx::query("INSERT INTO analysis_run_messages (run_id, ref) VALUES (42, 's1-i1')")
            .execute(&pool)
            .await
            .expect("insert saved corpus");

        delete_saved_run(&pool, 42).await.expect("delete saved run");

        let runs = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM analysis_runs")
            .fetch_one(&pool)
            .await
            .expect("count runs");
        let chat_messages =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM analysis_chat_messages")
                .fetch_one(&pool)
                .await
                .expect("count chat messages");
        let saved_messages =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM analysis_run_messages")
                .fetch_one(&pool)
                .await
                .expect("count saved messages");

        assert_eq!(runs, 0);
        assert_eq!(chat_messages, 0);
        assert_eq!(saved_messages, 0);
    }
}
