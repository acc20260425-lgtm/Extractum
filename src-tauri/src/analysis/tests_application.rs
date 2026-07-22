use super::chat::load_chat_run_and_scope_label;
use super::store::{
    fetch_run_row, list_analysis_run_summaries, map_run_detail, map_run_summary,
    AnalysisRunListFilters,
};
use crate::error::AppError;
use serde_json::json;

async fn application_read_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    for statement in [
        "CREATE TABLE sources (id INTEGER PRIMARY KEY, title TEXT)",
        "CREATE TABLE analysis_source_groups (id INTEGER PRIMARY KEY, name TEXT)",
        "CREATE TABLE projects (id INTEGER PRIMARY KEY, name TEXT)",
        "CREATE TABLE analysis_prompt_templates (id INTEGER PRIMARY KEY, name TEXT)",
        r#"CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY,
            run_type TEXT NOT NULL DEFAULT 'report',
            scope_type TEXT NOT NULL,
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
            status TEXT NOT NULL,
            result_markdown TEXT,
            trace_data_zstd BLOB,
            scope_label_snapshot TEXT,
            snapshot_captured_at TEXT,
            snapshot_error TEXT,
            error TEXT,
            created_at INTEGER NOT NULL,
            completed_at INTEGER
        )"#,
        "CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL, ref TEXT NOT NULL)",
    ] {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("create application read schema");
    }
    pool
}

async fn insert_run(
    pool: &sqlx::SqlitePool,
    id: i64,
    scope_type: &str,
    source_id: Option<i64>,
    project_id: Option<i64>,
    snapshot: Option<&str>,
    status: &str,
    created_at: i64,
    error: Option<&str>,
) {
    sqlx::query(
        r#"INSERT INTO analysis_runs (
            id, scope_type, source_id, project_id, status, scope_label_snapshot,
            error, result_markdown, created_at, completed_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, 'Saved report', ?, ?)"#,
    )
    .bind(id)
    .bind(scope_type)
    .bind(source_id)
    .bind(project_id)
    .bind(status)
    .bind(snapshot)
    .bind(error)
    .bind(created_at)
    .bind((status == "completed").then_some(created_at + 1))
    .execute(pool)
    .await
    .expect("insert application run");
}

#[tokio::test]
async fn run_reads_preserve_deleted_blank_and_snapshot_scope_labels() {
    let pool = application_read_pool().await;
    sqlx::query("INSERT INTO sources (id, title) VALUES (1, 'Live source'), (2, '   ')")
        .execute(&pool)
        .await
        .expect("insert sources");
    sqlx::query("INSERT INTO projects (id, name) VALUES (7, 'Deleted project')")
        .execute(&pool)
        .await
        .expect("insert project");

    insert_run(
        &pool,
        1,
        "single_source",
        Some(1),
        None,
        Some("Frozen source"),
        "completed",
        400,
        None,
    )
    .await;
    insert_run(
        &pool,
        2,
        "single_source",
        Some(2),
        None,
        None,
        "running",
        300,
        None,
    )
    .await;
    insert_run(
        &pool,
        3,
        "single_source",
        Some(99),
        None,
        None,
        "completed",
        200,
        None,
    )
    .await;
    insert_run(
        &pool,
        4,
        "project",
        None,
        Some(7),
        None,
        "completed",
        100,
        None,
    )
    .await;
    sqlx::query("DELETE FROM projects WHERE id = 7")
        .execute(&pool)
        .await
        .expect("delete project");

    let listed = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters::for_analysis(
            None, None, 20, None, None, None, None, None, None, None,
        )
        .expect("construct run-list filters"),
    )
    .await
    .expect("list runs");
    let labels = listed
        .iter()
        .map(|run| (run.id, run.scope_label.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        vec![
            (1, "Frozen source"),
            (2, "   "),
            (3, "Source 99"),
            (4, "Project 7"),
        ]
    );

    let active = fetch_run_row(&pool, 2)
        .await
        .expect("fetch active")
        .map(map_run_summary)
        .expect("active exists");
    assert_eq!(active.status, "running");
    assert_eq!(active.scope_label, "   ");

    let detail = fetch_run_row(&pool, 1)
        .await
        .expect("get run")
        .map(map_run_detail)
        .expect("run exists");
    assert_eq!(detail.scope_label, "Frozen source");
    assert_eq!(detail.source_title.as_deref(), Some("Live source"));
}

#[tokio::test]
async fn analysis_run_search_escapes_percent_underscore_and_backslash_before_limit() {
    let pool = application_read_pool().await;
    sqlx::query(
        "INSERT INTO sources (id, title) VALUES (1, 'Foreign Alpha'), (2, 'Foreign Alpha')",
    )
    .execute(&pool)
    .await
    .expect("insert sources");
    insert_run(
        &pool,
        1,
        "single_source",
        Some(2),
        None,
        Some("100xxliteral\\path"),
        "completed",
        500,
        Some("quota"),
    )
    .await;
    insert_run(
        &pool,
        2,
        "single_source",
        Some(1),
        None,
        Some("100%_literal\\path"),
        "completed",
        400,
        Some("quota exhausted"),
    )
    .await;
    insert_run(
        &pool,
        3,
        "single_source",
        Some(1),
        None,
        Some("100%_literal\\path"),
        "completed",
        300,
        Some("different failure"),
    )
    .await;

    let runs = list_analysis_run_summaries(
        &pool,
        AnalysisRunListFilters::for_analysis(
            None,
            None,
            1,
            Some("alpha 100%_literal\\path quota".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("construct search filters"),
    )
    .await
    .expect("search literal LIKE characters");

    assert_eq!(runs.iter().map(|run| run.id).collect::<Vec<_>>(), vec![2]);
}

#[tokio::test]
async fn chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot() {
    let pool = application_read_pool().await;
    sqlx::query("INSERT INTO sources (id, title) VALUES (1, 'Live label A')")
        .execute(&pool)
        .await
        .expect("insert source");
    insert_run(
        &pool,
        1,
        "single_source",
        Some(1),
        None,
        Some("Frozen label"),
        "completed",
        1,
        None,
    )
    .await;

    let read_label = async || {
        let (run, scope_label) = load_chat_run_and_scope_label(&pool, 1)
            .await
            .expect("read chat run context");
        assert_eq!(run.id, 1);
        scope_label
    };
    assert_eq!(read_label().await, "Frozen label");
    sqlx::query("UPDATE sources SET title = 'Live label B' WHERE id = 1")
        .execute(&pool)
        .await
        .expect("change live label");
    assert_eq!(read_label().await, "Frozen label");

    sqlx::query("UPDATE analysis_runs SET scope_label_snapshot = NULL WHERE id = 1")
        .execute(&pool)
        .await
        .expect("clear snapshot label");
    assert_eq!(read_label().await, "Live label B");
    sqlx::query("UPDATE analysis_runs SET scope_label_snapshot = '   ' WHERE id = 1")
        .execute(&pool)
        .await
        .expect("blank snapshot label");
    sqlx::query("UPDATE sources SET title = 'Live label C' WHERE id = 1")
        .execute(&pool)
        .await
        .expect("change live label again");
    assert_eq!(read_label().await, "Live label C");
}

#[test]
fn analysis_wire_values_serialize_to_exact_json_objects() {
    use super::models::{AnalysisChatEvent, AnalysisChunkSummaryEvent, AnalysisRunEvent};

    let run_event = AnalysisRunEvent {
        run_id: 41,
        request_id: Some("request-41".to_string()),
        kind: "progress".to_string(),
        phase: "map".to_string(),
        queue_position: Some(2),
        message: Some("Summarizing".to_string()),
        progress_current: Some(3),
        progress_total: Some(5),
        delta: Some("partial".to_string()),
        chunk_summary: Some(AnalysisChunkSummaryEvent {
            index: 3,
            total: 5,
            message_count: 8,
            summary: "Chunk summary".to_string(),
            topics: vec!["topic-a".to_string()],
            notable_points: vec!["point-a".to_string()],
            candidate_refs: vec!["source:1:item:2".to_string()],
        }),
        error: None,
    };
    assert_eq!(
        serde_json::to_value(run_event).expect("serialize run event"),
        json!({
            "run_id": 41,
            "request_id": "request-41",
            "kind": "progress",
            "phase": "map",
            "queue_position": 2,
            "message": "Summarizing",
            "progress_current": 3,
            "progress_total": 5,
            "delta": "partial",
            "chunk_summary": {
                "index": 3,
                "total": 5,
                "message_count": 8,
                "summary": "Chunk summary",
                "topics": ["topic-a"],
                "notable_points": ["point-a"],
                "candidate_refs": ["source:1:item:2"]
            },
            "error": null
        })
    );

    let chat_event = AnalysisChatEvent {
        request_id: "chat-7".to_string(),
        run_id: 41,
        kind: "completed".to_string(),
        queue_position: None,
        delta: None,
        message: Some("Answer".to_string()),
        error: None,
    };
    assert_eq!(
        serde_json::to_value(chat_event).expect("serialize chat event"),
        json!({
            "request_id": "chat-7",
            "run_id": 41,
            "kind": "completed",
            "queue_position": null,
            "delta": null,
            "message": "Answer",
            "error": null
        })
    );

    assert_eq!(
        serde_json::to_value(AppError::conflict("wire failure")).expect("serialize app error"),
        json!({"kind": "conflict", "message": "wire failure"})
    );
}

fn ordered(source: &str, markers: &[&str]) {
    let mut after = 0;
    for marker in markers {
        let offset = source[after..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing ordered marker: {marker}"));
        after += offset + marker.len();
    }
}

#[test]
fn chat_profile_resolution_failure_is_async_after_request_id() {
    let source = include_str!("chat.rs");
    let start = source
        .find("pub async fn ask_analysis_run_question")
        .expect("chat command");
    let command = &source[start..];
    ordered(
        command,
        &[
            "let request = build_chat_request",
            "let request_id = request.request_id.clone()",
            "tokio::spawn(async move",
            "resolve_profile_for_backend",
            "ChatEvent::new(emitted_request_id.clone(), run_id, \"failed\")",
            "Ok(request_id)",
        ],
    );
}

#[test]
fn report_start_preserves_acceptance_order_and_two_corpus_reads() {
    let source = include_str!("report.rs");
    let start = source
        .find("pub(crate) async fn start_analysis_report_run")
        .expect("report start");
    let command = &source[start..];
    ordered(
        command,
        &[
            "if period_from > period_to",
            "output_language.trim()",
            "selected_count",
            "get_pool(&handle)",
            "fetch_prompt_template",
            "resolve_profile_for_backend",
            "resolve_effective_model",
            "resolve_analysis_sources",
            "preflight_analysis_run",
            "find_active_duplicate_run",
            "insert_analysis_run",
            "insert_active_report_run",
            "tokio::spawn(async move",
            "await_report_terminal_and_cleanup",
            "run_report_pipeline",
            "Ok(run_id)",
        ],
    );
    let terminal_cleanup = &source[source
        .find("async fn await_report_terminal_and_cleanup")
        .expect("terminal cleanup helper")..start];
    ordered(
        terminal_cleanup,
        &[
            "let result = terminal.await",
            "remove_active_report_run",
            "result",
        ],
    );
    let pipeline = &source[source
        .find("async fn run_report_pipeline")
        .expect("pipeline")..start];
    assert_eq!(command.matches("preflight_analysis_run(").count(), 1);
    assert_eq!(pipeline.matches("capture_report_corpus(").count(), 1);
    assert!(
        command.find("preflight_analysis_run(").unwrap()
            < command.find("tokio::spawn(async move").unwrap()
    );
    assert!(pipeline.contains("capture_report_corpus(&pool"));
}
