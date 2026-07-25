use super::store::{
    get_analysis_run_in_pool, list_active_analysis_runs_in_pool, list_analysis_runs_in_pool,
    resolve_legacy_analysis_chat_run_in_pool,
};
use crate::error::AppError;
use extractum_analysis::AnalysisRunListFilters;
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

    let listed = list_analysis_runs_in_pool(
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

    let active = list_active_analysis_runs_in_pool(&pool, &std::collections::HashSet::from([2]))
        .await
        .expect("fetch active")
        .into_iter()
        .next()
        .expect("active exists");
    assert_eq!(active.status, "running");
    assert_eq!(active.scope_label, "   ");

    let detail = get_analysis_run_in_pool(&pool, 1)
        .await
        .expect("get run")
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

    let runs = list_analysis_runs_in_pool(
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
        let _chat_run = resolve_legacy_analysis_chat_run_in_pool(&pool, 1)
            .await
            .expect("read opaque chat run context");
        let run = get_analysis_run_in_pool(&pool, 1)
            .await
            .expect("read observable run detail")
            .expect("run detail exists");
        assert_eq!(run.id, 1);
        run.scope_label
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
    use extractum_analysis::{AnalysisChatEvent, AnalysisChunkSummaryEvent, AnalysisRunEvent};

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

struct FailingReportAcceptanceReader {
    calls: std::sync::atomic::AtomicUsize,
}

impl FailingReportAcceptanceReader {
    fn new() -> Self {
        Self {
            calls: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl super::corpus::AnalysisCorpusReader for FailingReportAcceptanceReader {
    fn load_corpus(
        &self,
        _request: super::corpus::AnalysisCorpusRequest,
    ) -> super::corpus::AnalysisPortFuture<'_, Vec<super::corpus::AnalysisCorpusMessage>> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async { Err(AppError::internal("preflight A failed")) })
    }
}

async fn report_acceptance_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect report acceptance pool");
    for statement in [
        r#"CREATE TABLE analysis_prompt_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            template_kind TEXT NOT NULL,
            body TEXT NOT NULL,
            version INTEGER NOT NULL DEFAULT 1,
            is_builtin BOOLEAN NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )"#,
        "CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        "CREATE TABLE analysis_runs (id INTEGER PRIMARY KEY AUTOINCREMENT)",
        "CREATE TABLE sources (
            id INTEGER PRIMARY KEY,
            source_type TEXT NOT NULL,
            source_subtype TEXT,
            title TEXT
        )",
    ] {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("create report acceptance schema");
    }
    pool
}

fn report_request_error(
    result: Result<extractum_analysis::StartAnalysisReportRequest, AppError>,
) -> AppError {
    match result {
        Ok(_) => panic!("report request unexpectedly succeeded"),
        Err(error) => error,
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
            "let request = AskAnalysisRunQuestionRequest::new(",
            "let pool = get_pool(&handle).await?",
            "resolve_legacy_analysis_chat_run_in_pool(&pool, run_id)",
            "prepare_analysis_chat(&pool, request, run)",
            "let request_id = ticket.request_id().to_string()",
            "let profile_id = ticket.profile_id().to_string()",
            "tokio::spawn(async move",
            "resolve_profile_for_backend",
            "execute_analysis_chat(",
            "get_pool(&app_handle)",
            "complete_analysis_chat(",
            "Ok(request_id)",
        ],
    );
    assert_eq!(command.matches("tokio::spawn(async move").count(), 1);
    assert!(!command.contains(".run_request("));
    assert!(!command.contains("persist_chat_exchange("));
    assert!(!command.contains(".emit("));
    assert!(!command.contains("request.run_id"));
}

#[tokio::test]
async fn report_start_preserves_acceptance_order_and_two_corpus_reads() {
    use crate::error::AppErrorKind;
    use crate::llm::{LlmProviderAccess, ProviderKind, ResolvedLlmProfile};
    use extractum_analysis::{
        prepare_analysis_report, prepare_analysis_report_execution, AnalysisSourceKind,
        AnalysisState, ResolvedAnalysisScope, StartAnalysisReportRequest,
    };

    let source = include_str!("report.rs");
    let portable_engine_source = include_str!("../../crates/extractum-analysis/src/report.rs");
    let start = source
        .find("pub(crate) async fn start_analysis_report_run")
        .expect("report start");
    let command = &source[start..];
    ordered(
        command,
        &[
            "get_pool(&handle)",
            "prepare_analysis_report(&pool, request)",
            "resolve_profile_for_backend",
            "resolve_effective_model",
            "resolve_model_input_token_limit",
            "resolve_youtube_corpus_mode()",
            "resolve_analysis_sources",
            "prepare_analysis_report_execution(",
            "let run_id = ticket.run_id()",
            "tokio::spawn(async move",
            "get_pool(&app_handle)",
            "execute_analysis_report(",
            "get_pool(&app_handle)",
            "finalize_analysis_report_execution(",
            "Ok(run_id)",
        ],
    );
    assert_eq!(command.matches("tokio::spawn(async move").count(), 1);
    let preparation = &portable_engine_source[portable_engine_source
        .find("pub async fn prepare_analysis_report_execution")
        .expect("execution preparation")
        ..portable_engine_source
            .find("pub async fn execute_analysis_report")
            .expect("report execution")];
    assert_eq!(preparation.matches("preflight_analysis_run(").count(), 1);
    let pipeline = &portable_engine_source[portable_engine_source
        .find("async fn run_report_pipeline_with_capabilities")
        .expect("portable report pipeline")..];
    assert_eq!(pipeline.matches("capture_analysis_corpus(").count(), 1);

    let command_adapter = include_str!("report_commands.rs");
    ordered(
        command_adapter,
        &[
            "StartAnalysisReportRequest::from_command",
            "start_analysis_report_run",
        ],
    );

    let invalid_period = report_request_error(StartAnalysisReportRequest::from_command(
        Some(2),
        None,
        None,
        20,
        10,
        "English".to_string(),
        1,
        None,
        None,
        None,
        false,
    ));
    assert_eq!(invalid_period.kind, AppErrorKind::Validation);
    assert_eq!(
        invalid_period.message,
        "period_from must be less than or equal to period_to"
    );
    let invalid_language = report_request_error(StartAnalysisReportRequest::from_command(
        Some(2),
        None,
        None,
        10,
        20,
        "   ".to_string(),
        1,
        None,
        None,
        None,
        false,
    ));
    assert_eq!(invalid_language.kind, AppErrorKind::Validation);
    assert_eq!(invalid_language.message, "Output language cannot be empty");
    for invalid_scope in [
        StartAnalysisReportRequest::from_command(
            None,
            None,
            None,
            10,
            20,
            "English".to_string(),
            1,
            None,
            None,
            None,
            false,
        ),
        StartAnalysisReportRequest::from_command(
            Some(2),
            Some(3),
            None,
            10,
            20,
            "English".to_string(),
            1,
            None,
            None,
            None,
            false,
        ),
    ] {
        let error = report_request_error(invalid_scope);
        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "Select exactly one analysis scope");
    }

    let missing_schema_pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect missing-schema pool");
    let missing_schema_request = StartAnalysisReportRequest::for_source(
        2,
        10,
        20,
        "English".to_string(),
        1,
        None,
        Some("prod west".to_string()),
        Some("invalid-youtube-mode".to_string()),
        false,
    )
    .expect("construct missing-schema request");
    let missing_schema_error =
        match prepare_analysis_report(&missing_schema_pool, missing_schema_request).await {
            Ok(_) => panic!("missing template schema unexpectedly succeeded"),
            Err(error) => error,
        };
    assert_eq!(missing_schema_error.kind, AppErrorKind::Internal);

    let pool = report_acceptance_pool().await;
    let missing_template_request = StartAnalysisReportRequest::for_source(
        2,
        10,
        20,
        "English".to_string(),
        99,
        None,
        Some("prod west".to_string()),
        Some("invalid-youtube-mode".to_string()),
        false,
    )
    .expect("construct missing-template request");
    let missing_template_error =
        match prepare_analysis_report(&pool, missing_template_request).await {
            Ok(_) => panic!("missing template unexpectedly succeeded"),
            Err(error) => error,
        };
    assert_eq!(missing_template_error.kind, AppErrorKind::NotFound);
    assert_eq!(
        missing_template_error.message,
        "Analysis prompt template 99 not found"
    );

    let invalid_youtube_request = StartAnalysisReportRequest::for_source(
        0,
        10,
        20,
        "English".to_string(),
        1,
        None,
        None,
        Some("invalid-youtube-mode".to_string()),
        false,
    )
    .expect("construct invalid-youtube request");
    let invalid_youtube_preparation = prepare_analysis_report(&pool, invalid_youtube_request)
        .await
        .expect("prepare invalid-youtube request");
    let invalid_youtube_error = match invalid_youtube_preparation.resolve_youtube_corpus_mode() {
        Ok(_) => panic!("invalid YouTube mode unexpectedly succeeded"),
        Err(error) => error,
    };
    assert_eq!(invalid_youtube_error.kind, AppErrorKind::Validation);
    assert_eq!(
        invalid_youtube_error.message,
        "Unsupported youtube_corpus_mode 'invalid-youtube-mode'"
    );

    let reader = FailingReportAcceptanceReader::new();
    let missing_source_request = StartAnalysisReportRequest::for_source(
        0,
        10,
        20,
        "English".to_string(),
        1,
        None,
        None,
        Some("transcript_description".to_string()),
        false,
    )
    .expect("construct missing-source request");
    let missing_source_preparation = prepare_analysis_report(&pool, missing_source_request)
        .await
        .expect("prepare missing-source request")
        .resolve_youtube_corpus_mode()
        .expect("resolve valid YouTube mode");
    let missing_source_error = match super::corpus::resolve_analysis_sources(
        &pool,
        Some(missing_source_preparation.scope_id()),
        None,
        None,
    )
    .await
    {
        Ok(_) => panic!("source zero unexpectedly resolved"),
        Err(error) => error.into_app_error(),
    };
    assert_eq!(missing_source_error.kind, AppErrorKind::NotFound);
    assert_eq!(missing_source_error.message, "Source 0 not found");
    assert_eq!(reader.call_count(), 0);

    let preflight_request = StartAnalysisReportRequest::for_source(
        2,
        10,
        20,
        "English".to_string(),
        1,
        None,
        None,
        Some("transcript_description".to_string()),
        false,
    )
    .expect("construct preflight request");
    let preflight_preparation = prepare_analysis_report(&pool, preflight_request)
        .await
        .expect("prepare preflight request")
        .resolve_youtube_corpus_mode()
        .expect("resolve preflight YouTube mode");
    let scope = ResolvedAnalysisScope::for_source(
        2,
        AnalysisSourceKind::Telegram,
        vec![2],
        "Source 2".to_string(),
    )
    .expect("construct resolved scope");
    let resolved_profile = ResolvedLlmProfile::new(
        "research".to_string(),
        "test-model".to_string(),
        LlmProviderAccess::new(
            ProviderKind::OpenAiCompatible,
            "secret-key".to_string().into(),
            "http://127.0.0.1:9/v1".to_string(),
        ),
    );
    let state = AnalysisState::new();
    let preflight_error = match prepare_analysis_report_execution(
        &pool,
        &state,
        &reader,
        preflight_preparation,
        scope,
        resolved_profile,
        "test-model".to_string(),
        Some(4_096),
    )
    .await
    {
        Ok(_) => panic!("failing preflight unexpectedly created a run"),
        Err(error) => error,
    };
    assert_eq!(preflight_error.kind, AppErrorKind::Internal);
    assert_eq!(preflight_error.message, "preflight A failed");
    assert_eq!(reader.call_count(), 1);
    let run_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analysis_runs")
        .fetch_one(&pool)
        .await
        .expect("count report acceptance runs");
    assert_eq!(run_count, 0);
    assert!(state.active_report_run_ids().await.is_empty());
}

#[tokio::test]
async fn report_profile_resolution_failure_prevents_run_creation() {
    use super::report::{prepare_analysis_report, StartAnalysisReportRequest};
    use crate::error::AppErrorKind;
    use crate::llm::resolve_profile_for_backend_in_pool;
    use crate::secret_store::tests::InMemorySecretStore;
    use crate::secret_store::SecretStoreState;
    use std::sync::Arc;

    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect report precedence pool");
    for statement in [
        r#"CREATE TABLE analysis_prompt_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            template_kind TEXT NOT NULL,
            body TEXT NOT NULL,
            version INTEGER NOT NULL DEFAULT 1,
            is_builtin BOOLEAN NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )"#,
        "CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        "CREATE TABLE analysis_runs (id INTEGER PRIMARY KEY AUTOINCREMENT)",
    ] {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("create report precedence schema");
    }
    let request = StartAnalysisReportRequest::for_source(
        0,
        10,
        20,
        "English".to_string(),
        1,
        None,
        Some("prod west".to_string()),
        Some("invalid-youtube-mode".to_string()),
        false,
    )
    .expect("construct conflicting report request");
    let state = extractum_analysis::AnalysisState::new();
    let preparation = prepare_analysis_report(&pool, request)
        .await
        .expect("template preparation precedes profile resolution");
    let secret_store = SecretStoreState::new(Arc::new(InMemorySecretStore::new()));

    let profile_error = match resolve_profile_for_backend_in_pool(
        &pool,
        &secret_store,
        preparation.requested_profile_id(),
    )
    .await
    {
        Ok(_) => panic!("invalid profile must fail before later conflicts"),
        Err(error) => error,
    };

    assert_eq!(
        profile_error.kind,
        AppErrorKind::Validation,
        "RED: CP4 profile precedence"
    );
    assert_eq!(
        profile_error.message,
        "Profile ID can only contain ASCII letters, numbers, dashes, and underscores"
    );
    let youtube_error = match preparation.resolve_youtube_corpus_mode() {
        Ok(_) => panic!("invalid YouTube mode is the later conflicting failure"),
        Err(error) => error,
    };
    assert_eq!(youtube_error.kind, AppErrorKind::Validation);
    let run_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analysis_runs")
        .fetch_one(&pool)
        .await
        .expect("count report runs");
    assert_eq!(run_count, 0);
    assert!(state.active_report_run_ids().await.is_empty());
}
