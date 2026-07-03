use super::super::{delete_saved_run, set_run_status};
use crate::analysis::models::AnalysisPromptTemplate;
use crate::error::AppErrorKind;

async fn status_update_pool() -> sqlx::SqlitePool {
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
    sqlx::query("INSERT INTO analysis_runs (id, status) VALUES (1, 'running')")
        .execute(&pool)
        .await
        .expect("insert run");
    pool
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
    sqlx::query("CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL, ref TEXT NOT NULL)")
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
async fn provider_failure_status_update_does_not_write_snapshot_error() {
    let pool = status_update_pool().await;
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
    let pool = status_update_pool().await;
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

#[tokio::test]
async fn insert_analysis_run_persists_youtube_corpus_mode() {
    use super::super::{insert_analysis_run, AnalysisRunInsert};
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
    use super::super::{
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
    use super::super::{
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
    sqlx::query("CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL, ref TEXT NOT NULL)")
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
    let chat_messages = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM analysis_chat_messages")
        .fetch_one(&pool)
        .await
        .expect("count chat messages");
    let saved_messages = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM analysis_run_messages")
        .fetch_one(&pool)
        .await
        .expect("count saved messages");

    assert_eq!(runs, 0);
    assert_eq!(chat_messages, 0);
    assert_eq!(saved_messages, 0);
}
