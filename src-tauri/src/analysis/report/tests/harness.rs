use crate::analysis::models::{AnalysisPromptTemplate, ChunkSummary, CorpusMessage};
use crate::llm::{ProviderKind, ResolvedLlmProfile};
use sqlx::SqlitePool;

pub(super) const SAMPLE_JSON: &str = r#"{"summary":"Brief","topics":["sync"],"notable_points":["Point"],"candidate_refs":["s1-i2"]}"#;

pub(super) fn sample_chunk_summary(label: &str) -> ChunkSummary {
    ChunkSummary {
        summary: label.to_string(),
        topics: vec![format!("{label}-topic")],
        notable_points: vec![format!("{label}-point")],
        candidate_refs: vec![format!("{label}-ref")],
    }
}

pub(super) fn sample_prompt_template() -> AnalysisPromptTemplate {
    AnalysisPromptTemplate {
        id: 7,
        name: "Report".to_string(),
        template_kind: "report".to_string(),
        body: "Write a concise report.".to_string(),
        version: 3,
        is_builtin: false,
        created_at: 1,
        updated_at: 1,
    }
}

pub(super) fn sample_corpus_message() -> CorpusMessage {
    CorpusMessage {
        item_id: 1,
        source_id: 2,
        external_id: "42".to_string(),
        published_at: 1_700_000_000,
        author: Some("analyst".to_string()),
        content: "Important update from the source".to_string(),
        r#ref: "s2-i1".to_string(),
        item_kind: Some("telegram_message".to_string()),
        source_type: Some("telegram".to_string()),
        source_subtype: Some("channel".to_string()),
        metadata_zstd: None,
    }
}

pub(super) fn sample_resolved_profile() -> ResolvedLlmProfile {
    ResolvedLlmProfile {
        profile_id: "research".to_string(),
        provider: ProviderKind::Gemini,
        default_model: "gemini-2.5-flash".to_string(),
        api_key: "secret-key".to_string().into(),
        base_url: String::new(),
    }
}

pub(super) async fn request_cancel_pool_with_runs() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    sqlx::query(
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
            prompt_template_id INTEGER NOT NULL DEFAULT 1,
            prompt_template_version INTEGER NOT NULL DEFAULT 1,
            provider_profile TEXT NOT NULL DEFAULT 'research',
            provider TEXT NOT NULL DEFAULT 'gemini',
            model TEXT NOT NULL DEFAULT 'gemini-2.5-flash',
            youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
            telegram_history_scope TEXT,
            status TEXT NOT NULL,
            result_markdown TEXT,
            trace_data_zstd BLOB,
            scope_label_snapshot TEXT,
            snapshot_captured_at TEXT,
            snapshot_error TEXT,
            error TEXT,
            created_at INTEGER NOT NULL DEFAULT 1,
            completed_at INTEGER
        )",
    )
    .execute(&pool)
    .await
    .expect("create analysis_runs");

    sqlx::query("CREATE TABLE sources (id INTEGER PRIMARY KEY, title TEXT)")
        .execute(&pool)
        .await
        .expect("create sources");
    sqlx::query("CREATE TABLE analysis_source_groups (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .expect("create groups");
    sqlx::query("CREATE TABLE projects (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .expect("create projects");
    sqlx::query("CREATE TABLE analysis_prompt_templates (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .expect("create templates");
    sqlx::query("CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL)")
        .execute(&pool)
        .await
        .expect("create run messages");

    pool
}

pub(super) async fn insert_cancel_request_run(pool: &SqlitePool, run_id: i64, status: &str) {
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, status, period_from, period_to, output_language,
            prompt_template_id, prompt_template_version, provider_profile, provider, model,
            youtube_corpus_mode, created_at
        ) VALUES (
            ?, 'report', 'single_source', ?, 1, 2, 'English', 1, 1,
            'research', 'gemini', 'gemini-2.5-flash', 'transcript_description', 1
        )",
    )
    .bind(run_id)
    .bind(status)
    .execute(pool)
    .await
    .expect("insert analysis run");
}
