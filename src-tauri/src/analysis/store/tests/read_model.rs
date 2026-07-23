use super::super::{
    list_analysis_runs_in_pool, AnalysisRunListFilters as AppAnalysisRunListFilters,
};

include!("read_model_portable.rs");

async fn app_run_list_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    for statement in [
        "CREATE TABLE sources (id INTEGER PRIMARY KEY, title TEXT)",
        "CREATE TABLE analysis_source_groups (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
        "CREATE TABLE projects (id INTEGER PRIMARY KEY, name TEXT)",
        "CREATE TABLE analysis_prompt_templates (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            template_kind TEXT NOT NULL,
            body TEXT NOT NULL,
            version INTEGER NOT NULL DEFAULT 1,
            is_builtin BOOLEAN NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        "CREATE TABLE analysis_runs (
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
        )",
        "CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL, ref TEXT NOT NULL)",
    ] {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("create read-model fixture table");
    }
    sqlx::query("INSERT INTO sources (id, title) VALUES (1, 'Alpha Source'), (2, 'Beta Source')")
        .execute(&pool)
        .await
        .expect("insert source labels");
    sqlx::query("INSERT INTO projects (id, name) VALUES (7, 'Alpha Project'), (8, 'Beta Project')")
        .execute(&pool)
        .await
        .expect("insert project labels");
    pool
}

async fn insert_app_run(
    pool: &sqlx::SqlitePool,
    id: i64,
    source_id: Option<i64>,
    project_id: Option<i64>,
    scope_label_snapshot: &str,
    error: Option<&str>,
    created_at: i64,
) {
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, scope_type, source_id, project_id, scope_label_snapshot, error, created_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(if project_id.is_some() {
        "project"
    } else {
        "single_source"
    })
    .bind(source_id)
    .bind(project_id)
    .bind(scope_label_snapshot)
    .bind(error)
    .bind(created_at)
    .execute(pool)
    .await
    .expect("insert app run");
}

#[tokio::test]
async fn list_analysis_run_summaries_filters_project_runs() {
    let pool = app_run_list_pool().await;
    insert_app_run(&pool, 1, None, Some(7), "Alpha Project", None, 300).await;
    insert_app_run(&pool, 2, None, Some(8), "Beta Project", None, 200).await;

    let runs = list_analysis_runs_in_pool(&pool, AppAnalysisRunListFilters::for_project(7, 50))
        .await
        .expect("list project runs");

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
    assert_eq!(runs[0].project_name.as_deref(), Some("Alpha Project"));
}

#[tokio::test]
async fn list_analysis_run_summaries_matches_all_query_terms_across_any_field() {
    let pool = app_run_list_pool().await;
    insert_app_run(
        &pool,
        1,
        Some(1),
        None,
        "Plain label",
        Some("quota exhausted"),
        300,
    )
    .await;
    insert_app_run(
        &pool,
        2,
        Some(2),
        None,
        "Plain label",
        Some("different failure"),
        200,
    )
    .await;

    let runs = list_analysis_runs_in_pool(
        &pool,
        AppAnalysisRunListFilters::for_analysis(
            None,
            None,
            50,
            Some("alpha quota".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("construct term query filters"),
    )
    .await
    .expect("list terms");

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![1]);
}
