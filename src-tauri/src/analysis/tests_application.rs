use super::store::{
    get_analysis_run_in_pool, list_active_analysis_runs_in_pool, list_analysis_runs_in_pool,
    resolve_legacy_analysis_chat_run_in_pool,
};
use crate::error::AppError;
use extractum_analysis::AnalysisRunListFilters;
use serde_json::json;

async fn application_read_pool() -> sqlx::SqlitePool {
    application_read_pool_with_max_connections(5).await
}

async fn application_read_pool_with_max_connections(max_connections: u32) -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect("sqlite::memory:")
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

#[test]
fn analysis_command_event_and_app_error_wire_contracts_are_exact() {
    #[derive(Debug)]
    struct WireParameter {
        rust_name: &'static str,
        rust_type: &'static str,
        wire_name: &'static str,
    }

    #[derive(Debug)]
    struct CommandWireContract {
        name: &'static str,
        parameters: Vec<WireParameter>,
    }

    fn lower_camel_case(value: &str) -> String {
        let mut parts = value.split('_');
        let mut result = parts.next().unwrap_or_default().to_string();
        for part in parts {
            let mut characters = part.chars();
            if let Some(first) = characters.next() {
                result.extend(first.to_uppercase());
                result.extend(characters);
            }
        }
        result
    }

    macro_rules! collect_wire_contract {
        ($contracts:ident, $(#[$attribute:meta])* $command:ident => ($implementation:path; (($($parameter:ident : $parameter_type:ty),* $(,)?) -> $result:ty; [$($wire:literal),* $(,)?]))) => {
            $(#[$attribute])*
            {
                let _signature_witness = |$($parameter: $parameter_type),*| {
                    fn assert_output<R, F: std::future::Future<Output = R>>(_: F) {}
                    assert_output::<$result, _>(crate::$command($($parameter),*));
                };
                let rust_parameters = [
                    $((stringify!($parameter), stringify!($parameter_type))),*
                ]
                .into_iter()
                .filter(|(name, _)| !matches!(*name, "handle" | "state" | "scheduler" | "repair_state"))
                .collect::<Vec<_>>();
                let wire_names = [$($wire),*];
                assert_eq!(rust_parameters.len(), wire_names.len(), "{} wire parameter count", stringify!($command));
                let parameters = rust_parameters
                    .into_iter()
                    .zip(wire_names)
                    .map(|((rust_name, rust_type), wire_name)| {
                        assert_eq!(lower_camel_case(rust_name), wire_name, "{} parameter", stringify!($command));
                        WireParameter { rust_name, rust_type, wire_name }
                    })
                    .collect();
                $contracts.push(CommandWireContract { name: stringify!($command), parameters });
            }
        };
        ($contracts:ident, $(#[$attribute:meta])* $command:ident) => {};
    }

    macro_rules! analysis_wire_contracts {
        ([$($before:ident,)*], [$($(#[$after_attribute:meta])* $after:ident $(=> ($implementation:path; (($($parameter:ident : $parameter_type:ty),* $(,)?) -> $result:ty; [$($wire:literal),* $(,)?])))?),* $(,)?]; $($telegram:ident),* $(,)?) => {{
            let mut contracts = Vec::new();
            $(collect_wire_contract!(contracts, $(#[$after_attribute])* $after $(=> ($implementation; (($($parameter : $parameter_type),*) -> $result; [$($wire),*])))?);)*
            contracts
        }};
    }

    let contracts = crate::application_command_inventory!(analysis_wire_contracts);
    assert_eq!(contracts.len(), 27);
    assert_eq!(
        contracts
            .iter()
            .map(|contract| contract.name)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        27,
        "the production command declaration must contain 27 unique command identities"
    );
    assert!(contracts
        .iter()
        .all(
            |contract| contract
                .parameters
                .iter()
                .all(|parameter| !parameter.rust_name.is_empty()
                    && !parameter.rust_type.is_empty()
                    && !parameter.wire_name.is_empty())
        ));

    assert_eq!(super::ANALYSIS_RUN_EVENT, "analysis://run");
    assert_eq!(super::ANALYSIS_CHAT_EVENT, "analysis://chat");
    analysis_wire_values_serialize_to_exact_json_objects();

    use extractum_analysis::{AnalysisChatEvent, AnalysisChunkSummaryEvent, AnalysisRunEvent};
    let run_lifecycle = [
        "queued",
        "started",
        "progress",
        "delta",
        "completed",
        "failed",
        "cancelled",
    ];
    let serialized_run_kinds = run_lifecycle
        .into_iter()
        .map(|kind| {
            let event = AnalysisRunEvent {
                run_id: 1,
                request_id: None,
                kind: kind.to_string(),
                phase: "persist".to_string(),
                queue_position: None,
                message: None,
                progress_current: None,
                progress_total: None,
                delta: None,
                chunk_summary: None,
                error: None,
            };
            serde_json::to_value(event).expect("serialize every run lifecycle kind")["kind"]
                .as_str()
                .expect("run kind string")
                .to_string()
        })
        .collect::<Vec<_>>();
    assert_eq!(serialized_run_kinds, run_lifecycle);
    assert_eq!(
        serde_json::to_string(&AnalysisRunEvent {
            run_id: 1,
            request_id: None,
            kind: "queued".to_string(),
            phase: "persist".to_string(),
            queue_position: None,
            message: None,
            progress_current: None,
            progress_total: None,
            delta: None,
            chunk_summary: None,
            error: None,
        })
        .expect("serialize exact run-event layout"),
        r#"{"run_id":1,"request_id":null,"kind":"queued","phase":"persist","queue_position":null,"message":null,"progress_current":null,"progress_total":null,"delta":null,"chunk_summary":null,"error":null}"#
    );

    let chat_lifecycle = ["queued", "started", "delta", "completed"];
    let serialized_chat_kinds = chat_lifecycle
        .into_iter()
        .map(|kind| {
            let event = AnalysisChatEvent {
                request_id: "request-1".to_string(),
                run_id: 1,
                kind: kind.to_string(),
                queue_position: None,
                delta: None,
                message: None,
                error: None,
            };
            serde_json::to_value(event).expect("serialize every chat lifecycle kind")["kind"]
                .as_str()
                .expect("chat kind string")
                .to_string()
        })
        .collect::<Vec<_>>();
    assert_eq!(serialized_chat_kinds, chat_lifecycle);
    assert_eq!(
        serde_json::to_string(&AnalysisChatEvent {
            request_id: "request-1".to_string(),
            run_id: 1,
            kind: "queued".to_string(),
            queue_position: None,
            delta: None,
            message: None,
            error: None,
        })
        .expect("serialize exact chat-event layout"),
        r#"{"request_id":"request-1","run_id":1,"kind":"queued","queue_position":null,"delta":null,"message":null,"error":null}"#
    );

    let chunk = AnalysisChunkSummaryEvent {
        index: 1,
        total: 2,
        message_count: 3,
        summary: "summary".to_string(),
        topics: vec!["topic".to_string()],
        notable_points: vec!["point".to_string()],
        candidate_refs: vec!["source:1:item:1".to_string()],
    };
    assert_eq!(
        serde_json::to_string(&chunk).expect("serialize exact chunk layout"),
        r#"{"index":1,"total":2,"message_count":3,"summary":"summary","topics":["topic"],"notable_points":["point"],"candidate_refs":["source:1:item:1"]}"#
    );

    use crate::error::AppErrorKind;
    fn assert_exhaustive_error_kind(kind: AppErrorKind) {
        match kind {
            AppErrorKind::Validation
            | AppErrorKind::NotFound
            | AppErrorKind::Auth
            | AppErrorKind::Network
            | AppErrorKind::Conflict
            | AppErrorKind::Internal => {}
        }
    }
    let error_kinds = [
        AppErrorKind::Validation,
        AppErrorKind::NotFound,
        AppErrorKind::Auth,
        AppErrorKind::Network,
        AppErrorKind::Conflict,
        AppErrorKind::Internal,
    ];
    let _ = error_kinds.map(assert_exhaustive_error_kind);
    assert_eq!(error_kinds.map(|kind| kind as usize), [0, 1, 2, 3, 4, 5]);
    assert_eq!(
        serde_json::to_string(&error_kinds).expect("serialize exact error-kind wire values"),
        r#"["validation","not_found","auth","network","conflict","internal"]"#
    );
    assert_eq!(
        serde_json::to_string(&AppError::conflict("wire failure"))
            .expect("serialize exact AppError fields"),
        r#"{"kind":"conflict","message":"wire failure"}"#
    );
}

#[tokio::test]
async fn analysis_coordinators_share_one_app_owned_transaction() {
    use crate::analysis::store::{
        get_analysis_source_group_response_in_pool, list_analysis_source_groups_in_pool,
    };
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::notebooklm_export::load_export_source_group_in_pool;
    use crate::projects::{delete_project_in_pool, list_research_projects_in_pool};
    use std::time::Duration;

    let pool = application_read_pool_with_max_connections(1).await;
    sqlx::query("INSERT INTO sources (id, title) VALUES (7, 'Live source')")
        .execute(&pool)
        .await
        .expect("insert legacy-label source");
    insert_run(
        &pool,
        41,
        "single_source",
        Some(7),
        None,
        None,
        "running",
        100,
        None,
    )
    .await;

    let filters = AnalysisRunListFilters::for_analysis(
        None, None, 20, None, None, None, None, None, None, None,
    )
    .expect("construct filters");
    let listed = tokio::time::timeout(
        Duration::from_secs(2),
        list_analysis_runs_in_pool(&pool, filters),
    )
    .await
    .expect("list coordinator must not reacquire its sole pool connection")
    .expect("list through coordinator transaction");
    let active_ids = std::collections::HashSet::from([41]);
    let active = tokio::time::timeout(
        Duration::from_secs(2),
        list_active_analysis_runs_in_pool(&pool, &active_ids),
    )
    .await
    .expect("active coordinator must not reacquire its sole pool connection")
    .expect("list active through coordinator transaction");
    let detail = tokio::time::timeout(Duration::from_secs(2), get_analysis_run_in_pool(&pool, 41))
        .await
        .expect("detail coordinator must not reacquire its sole pool connection")
        .expect("get through coordinator transaction")
        .expect("run detail");
    let chat = tokio::time::timeout(
        Duration::from_secs(2),
        resolve_legacy_analysis_chat_run_in_pool(&pool, 41),
    )
    .await
    .expect("legacy chat coordinator must not reacquire its sole pool connection")
    .expect("resolve chat through coordinator transaction");

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, 41);
    assert_eq!(listed[0].scope_label, "Live source");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, 41);
    assert_eq!(detail.id, 41);
    assert!(chat.needs_legacy_foreign_label());

    let mut transaction = pool
        .begin()
        .await
        .expect("coordinators release transaction");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analysis_runs")
        .fetch_one(&mut *transaction)
        .await
        .expect("read after coordinators");
    transaction
        .rollback()
        .await
        .expect("rollback witness transaction");
    assert_eq!(count, 1);

    let application_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect full application pool");
    apply_all_migrations_for_test_pool(&application_pool)
        .await
        .expect("apply application migrations");
    sqlx::query("INSERT INTO sources (id, source_type, source_subtype, external_id, title, created_at) VALUES (7, 'youtube', 'video', 'video-7', 'Video seven', 1)")
        .execute(&application_pool).await.expect("insert export source");
    sqlx::query("INSERT INTO analysis_source_groups (id, name, created_at, updated_at, source_type) VALUES (3, 'Research group', 1, 1, 'youtube')")
        .execute(&application_pool).await.expect("insert source group");
    sqlx::query("INSERT INTO analysis_source_group_members (group_id, source_id, created_at) VALUES (3, 7, 1)")
        .execute(&application_pool).await.expect("insert group member");
    sqlx::query("INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (5, 'Rollback project', NULL, 1, 1)")
        .execute(&application_pool).await.expect("insert project");
    sqlx::query("INSERT INTO analysis_runs (id, run_type, scope_type, project_id, period_from, period_to, output_language, prompt_template_version, provider_profile, provider, model, status, created_at) VALUES (51, 'report', 'project', 5, 0, 1, 'English', 1, 'default', 'gemini', 'gemini-2.5-flash', 'completed', 1)")
        .execute(&application_pool).await.expect("insert project run");

    let groups = tokio::time::timeout(
        Duration::from_secs(2),
        list_analysis_source_groups_in_pool(&application_pool),
    )
    .await
    .expect("group-list coordinator must not reacquire its sole pool connection")
    .expect("list groups through coordinator transaction");
    assert_eq!(groups.len(), 1);
    let group = tokio::time::timeout(
        Duration::from_secs(2),
        get_analysis_source_group_response_in_pool(&application_pool, 3),
    )
    .await
    .expect("group-detail coordinator must not reacquire its sole pool connection")
    .expect("get group through coordinator transaction")
    .expect("group fixture");
    assert_eq!(group.members.len(), 1);
    let export = tokio::time::timeout(
        Duration::from_secs(2),
        load_export_source_group_in_pool(&application_pool, 3),
    )
    .await
    .expect("export coordinator must not reacquire its sole pool connection")
    .expect("load export group through coordinator transaction");
    assert_eq!(export.members.len(), 1);
    let projects = tokio::time::timeout(
        Duration::from_secs(2),
        list_research_projects_in_pool(&application_pool),
    )
    .await
    .expect("project-list coordinator must not reacquire its sole pool connection")
    .expect("list projects through coordinator transaction");
    assert_eq!(projects.len(), 1);

    sqlx::query("CREATE TRIGGER reject_project_delete BEFORE DELETE ON projects BEGIN SELECT RAISE(ABORT, 'injected delete failure'); END")
        .execute(&application_pool).await.expect("install rollback failure injection");
    assert!(tokio::time::timeout(
        Duration::from_secs(2),
        delete_project_in_pool(&application_pool, 5),
    )
    .await
    .expect("project-delete coordinator must not reacquire its sole pool connection")
    .is_err());
    let mut transaction = application_pool
        .begin()
        .await
        .expect("all application coordinators release their transaction");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects")
            .fetch_one(&mut *transaction)
            .await
            .expect("read after every coordinator"),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM analysis_runs WHERE project_id = 5")
            .fetch_one(&mut *transaction)
            .await
            .expect("rollback preserves participant deletion"),
        1
    );
    transaction
        .rollback()
        .await
        .expect("rollback final witness transaction");
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
