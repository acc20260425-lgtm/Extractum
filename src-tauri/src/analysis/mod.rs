mod chat;
mod corpus;
mod groups;
mod models;
mod report;
mod store;
mod templates;
mod trace;

use std::collections::HashSet;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use self::corpus::{
    list_run_snapshot_messages_page, load_trace_resolution_messages, ListRunSnapshotMessagesRequest,
};
use self::models::{
    AnalysisChatEvent, AnalysisChatTurn, AnalysisRunDetail, AnalysisRunEvent,
    AnalysisRunMessageCursor, AnalysisRunMessagesPage, AnalysisRunRow, AnalysisRunSummary,
    AnalysisSourceOption, AnalysisTraceData, AnalysisTraceRef,
};
use self::store::{delete_saved_run, fetch_run_row, map_run_detail, map_run_summary};
use self::trace::{build_trace_refs, decode_trace_data, normalize_ref};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};

pub use self::chat::{
    ask_analysis_run_question, clear_analysis_chat_messages, list_analysis_chat_messages,
};
pub use self::groups::{
    create_analysis_source_group, delete_analysis_source_group, list_analysis_source_groups,
    update_analysis_source_group,
};
pub use self::report::{
    cancel_analysis_run, cleanup_interrupted_analysis_runs, start_analysis_report,
};
pub use self::templates::{
    create_analysis_prompt_template, delete_analysis_prompt_template,
    list_analysis_prompt_templates, update_analysis_prompt_template,
};

const TEMPLATE_KIND_REPORT: &str = "report";
const TEMPLATE_KIND_CHAT: &str = "chat";
const DEFAULT_REPORT_TEMPLATE_NAME: &str = "Default report";
const ANALYSIS_RUN_EVENT: &str = "analysis://run";
const ANALYSIS_CHAT_EVENT: &str = "analysis://chat";
const ANALYSIS_RUN_TYPE_REPORT: &str = "report";
const ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE: &str = "single_source";
const ANALYSIS_SCOPE_TYPE_SOURCE_GROUP: &str = "source_group";
const ANALYSIS_STATUS_QUEUED: &str = "queued";
const ANALYSIS_STATUS_RUNNING: &str = "running";
const ANALYSIS_STATUS_COMPLETED: &str = "completed";
const ANALYSIS_STATUS_FAILED: &str = "failed";
const ANALYSIS_STATUS_CANCELLED: &str = "cancelled";
const ANALYSIS_CHUNK_TARGET_CHARS: usize = 16_000;

pub struct AnalysisState {
    active_report_runs: Mutex<HashSet<i64>>,
    cancelled_report_runs: Mutex<HashSet<i64>>,
}

impl AnalysisState {
    pub fn new() -> Self {
        Self {
            active_report_runs: Mutex::new(HashSet::new()),
            cancelled_report_runs: Mutex::new(HashSet::new()),
        }
    }

    async fn insert_active_report_run(&self, run_id: i64) {
        self.active_report_runs.lock().await.insert(run_id);
        self.cancelled_report_runs.lock().await.remove(&run_id);
    }

    async fn remove_active_report_run(&self, run_id: i64) {
        self.active_report_runs.lock().await.remove(&run_id);
        self.cancelled_report_runs.lock().await.remove(&run_id);
    }

    async fn active_report_run_ids(&self) -> HashSet<i64> {
        self.active_report_runs.lock().await.clone()
    }

    async fn request_report_run_cancel(&self, run_id: i64) -> bool {
        let active_runs = self.active_report_runs.lock().await;
        if !active_runs.contains(&run_id) {
            return false;
        }
        drop(active_runs);
        self.cancelled_report_runs.lock().await.insert(run_id);
        true
    }

    async fn is_report_run_cancelled(&self, run_id: i64) -> bool {
        self.cancelled_report_runs.lock().await.contains(&run_id)
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn emit_analysis_event(handle: &AppHandle, event: &AnalysisRunEvent) {
    let _ = handle.emit(ANALYSIS_RUN_EVENT, event);
}

fn emit_analysis_chat_event(handle: &AppHandle, event: &AnalysisChatEvent) {
    let _ = handle.emit(ANALYSIS_CHAT_EVENT, event);
}

fn validate_chat_turns(history: &[AnalysisChatTurn]) -> AppResult<()> {
    for turn in history {
        match turn.role.as_str() {
            "user" | "assistant" => {}
            other => {
                return Err(AppError::validation(format!(
                    "Unsupported chat turn role '{other}'"
                )))
            }
        }
        if turn.content.trim().is_empty() {
            return Err(AppError::validation("Chat turns cannot be empty"));
        }
    }

    Ok(())
}

fn validate_chat_role(role: &str) -> AppResult<()> {
    match role {
        "user" | "assistant" => Ok(()),
        other => Err(AppError::validation(format!(
            "Unsupported chat role '{other}'"
        ))),
    }
}

fn default_report_template_body() -> &'static str {
    r#"Create a grounded report over the provided source documents.

Focus on:
- the main topics and recurring themes
- the most notable claims, updates, and shifts
- supporting examples from the source material

Always keep the report concise, readable, and useful for later follow-up analysis."#
}

#[tauri::command]
pub async fn list_analysis_sources(handle: AppHandle) -> AppResult<Vec<AnalysisSourceOption>> {
    let pool = get_pool(&handle).await?;
    sqlx::query_as(
        r#"
        SELECT
            sources.id,
            sources.account_id,
            sources.source_type,
            sources.title,
            COUNT(items.content_zstd) AS item_count,
            sources.last_synced_at
        FROM sources
        LEFT JOIN items ON items.source_id = sources.id
        GROUP BY sources.id, sources.account_id, sources.source_type, sources.title, sources.last_synced_at
        ORDER BY sources.created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(AppError::database)
}

#[tauri::command]
pub async fn list_analysis_runs(
    handle: AppHandle,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    limit: Option<i64>,
) -> AppResult<Vec<AnalysisRunSummary>> {
    let pool = get_pool(&handle).await?;
    let limit = limit.unwrap_or(20).clamp(1, 100);

    if source_id.is_some() && source_group_id.is_some() {
        return Err(AppError::validation(
            "Pass either source_id or source_group_id, not both",
        ));
    }

    let rows: Vec<AnalysisRunRow> = if let Some(source_id) = source_id {
        sqlx::query_as(
            r#"
            SELECT
                runs.id,
                runs.run_type,
                runs.scope_type,
                runs.source_id,
                sources.title AS source_title,
                runs.source_group_id,
                groups.name AS source_group_name,
                runs.period_from,
                runs.period_to,
                runs.output_language,
                runs.prompt_template_id,
                templates.name AS prompt_template_name,
                runs.prompt_template_version,
                runs.provider_profile,
                runs.provider,
                runs.model,
                runs.youtube_corpus_mode,
                runs.status,
                runs.result_markdown,
                runs.trace_data_zstd,
                runs.scope_label_snapshot,
                runs.error,
                runs.created_at,
                runs.completed_at
            FROM analysis_runs runs
            LEFT JOIN sources ON sources.id = runs.source_id
            LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
            LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
            WHERE runs.source_id = ?
            ORDER BY runs.created_at DESC
            LIMIT ?
            "#,
        )
        .bind(source_id)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(AppError::database)?
    } else if let Some(source_group_id) = source_group_id {
        sqlx::query_as(
            r#"
            SELECT
                runs.id,
                runs.run_type,
                runs.scope_type,
                runs.source_id,
                sources.title AS source_title,
                runs.source_group_id,
                groups.name AS source_group_name,
                runs.period_from,
                runs.period_to,
                runs.output_language,
                runs.prompt_template_id,
                templates.name AS prompt_template_name,
                runs.prompt_template_version,
                runs.provider_profile,
                runs.provider,
                runs.model,
                runs.youtube_corpus_mode,
                runs.status,
                runs.result_markdown,
                runs.trace_data_zstd,
                runs.scope_label_snapshot,
                runs.error,
                runs.created_at,
                runs.completed_at
            FROM analysis_runs runs
            LEFT JOIN sources ON sources.id = runs.source_id
            LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
            LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
            WHERE runs.source_group_id = ?
            ORDER BY runs.created_at DESC
            LIMIT ?
            "#,
        )
        .bind(source_group_id)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(AppError::database)?
    } else {
        sqlx::query_as(
            r#"
            SELECT
                runs.id,
                runs.run_type,
                runs.scope_type,
                runs.source_id,
                sources.title AS source_title,
                runs.source_group_id,
                groups.name AS source_group_name,
                runs.period_from,
                runs.period_to,
                runs.output_language,
                runs.prompt_template_id,
                templates.name AS prompt_template_name,
                runs.prompt_template_version,
                runs.provider_profile,
                runs.provider,
                runs.model,
                runs.youtube_corpus_mode,
                runs.status,
                runs.result_markdown,
                runs.trace_data_zstd,
                runs.scope_label_snapshot,
                runs.error,
                runs.created_at,
                runs.completed_at
            FROM analysis_runs runs
            LEFT JOIN sources ON sources.id = runs.source_id
            LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
            LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
            ORDER BY runs.created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(AppError::database)?
    };

    Ok(rows.into_iter().map(map_run_summary).collect())
}

#[tauri::command]
pub async fn list_active_analysis_runs(
    handle: AppHandle,
    state: tauri::State<'_, AnalysisState>,
) -> AppResult<Vec<AnalysisRunSummary>> {
    let pool = get_pool(&handle).await?;
    let active_ids = state.active_report_run_ids().await;
    let mut active_runs = Vec::new();
    let mut stale_ids = Vec::new();

    for run_id in active_ids {
        match fetch_run_row(&pool, run_id).await? {
            Some(row)
                if row.status == ANALYSIS_STATUS_QUEUED
                    || row.status == ANALYSIS_STATUS_RUNNING =>
            {
                active_runs.push(map_run_summary(row));
            }
            _ => stale_ids.push(run_id),
        }
    }

    for run_id in stale_ids {
        state.remove_active_report_run(run_id).await;
    }

    active_runs.sort_by_key(|run| std::cmp::Reverse(run.created_at));
    Ok(active_runs)
}

#[tauri::command]
pub async fn get_analysis_run(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<Option<AnalysisRunDetail>> {
    let pool = get_pool(&handle).await?;
    Ok(fetch_run_row(&pool, run_id).await?.map(map_run_detail))
}

#[tauri::command]
pub async fn list_analysis_run_messages(
    handle: AppHandle,
    run_id: i64,
    after: Option<AnalysisRunMessageCursor>,
    limit: Option<i64>,
    source_id: Option<i64>,
) -> AppResult<AnalysisRunMessagesPage> {
    let pool = get_pool(&handle).await?;
    let exists =
        sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM analysis_runs WHERE id = ?)")
            .bind(run_id)
            .fetch_one(&pool)
            .await
            .map_err(AppError::database)?;

    if exists == 0 {
        return Err(AppError::not_found(format!(
            "Analysis run {run_id} not found"
        )));
    }

    let limit = limit.unwrap_or(100).clamp(1, 500) as usize;
    list_run_snapshot_messages_page(
        &pool,
        ListRunSnapshotMessagesRequest {
            run_id,
            after,
            limit,
            source_id,
        },
    )
    .await
    .map_err(AppError::database)
}

#[tauri::command]
pub async fn get_analysis_run_trace(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<AnalysisTraceData> {
    let pool = get_pool(&handle).await?;
    let row = fetch_run_row(&pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;

    Ok(decode_trace_data(row.trace_data_zstd.as_deref())?)
}

#[tauri::command]
pub async fn delete_analysis_run(
    handle: AppHandle,
    state: tauri::State<'_, AnalysisState>,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    let row = fetch_run_row(&pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;

    if row.status == ANALYSIS_STATUS_QUEUED || row.status == ANALYSIS_STATUS_RUNNING {
        return Err(AppError::conflict(
            "Queued or running analysis runs cannot be deleted",
        ));
    }

    delete_saved_run(&pool, run_id).await?;
    state.remove_active_report_run(run_id).await;
    Ok(())
}

#[tauri::command]
pub async fn resolve_analysis_trace_refs(
    handle: AppHandle,
    run_id: i64,
    refs: Vec<String>,
) -> AppResult<Vec<AnalysisTraceRef>> {
    let mut normalized_refs = refs
        .into_iter()
        .filter_map(|reference| normalize_ref(&reference))
        .collect::<Vec<_>>();
    normalized_refs.sort();
    normalized_refs.dedup();

    if normalized_refs.is_empty() {
        return Ok(Vec::new());
    }

    let pool = get_pool(&handle).await?;
    let run = get_analysis_run(handle.clone(), run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;

    let corpus = load_trace_resolution_messages(&pool, &run).await?;
    Ok(build_trace_refs(&normalized_refs, &corpus))
}

#[cfg(test)]
mod tests {
    use super::groups::normalize_source_group_input;
    use super::store::ensure_builtin_report_template;
    use super::templates::validate_template_kind;
    use super::trace::compress_trace_data;
    use super::{
        decode_trace_data, validate_chat_role, AnalysisChatTurn, AnalysisTraceData,
        AnalysisTraceRef, TEMPLATE_KIND_REPORT,
    };
    use crate::error::AppErrorKind;

    async fn memory_pool() -> sqlx::SqlitePool {
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
        sqlx::query(
            r#"
            CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_type TEXT NOT NULL,
                scope_type TEXT NOT NULL,
                source_id INTEGER,
                source_group_id INTEGER,
                period_from INTEGER NOT NULL,
                period_to INTEGER NOT NULL,
                output_language TEXT NOT NULL,
                prompt_template_id INTEGER,
                prompt_template_version INTEGER NOT NULL,
                provider_profile TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
                status TEXT NOT NULL,
                result_markdown TEXT,
                trace_data_zstd BLOB,
                scope_label_snapshot TEXT,
                error TEXT,
                created_at INTEGER NOT NULL,
                completed_at INTEGER
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create runs");
        pool
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

    #[test]
    fn trace_data_roundtrips_through_zstd() {
        let trace = AnalysisTraceData {
            refs: vec![AnalysisTraceRef {
                r#ref: "s12-m845".to_string(),
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
        let decoded = decode_trace_data(Some(&compressed)).expect("decode");
        assert_eq!(decoded, trace);
    }

    #[test]
    fn source_group_input_is_trimmed_and_deduplicated() {
        let (name, source_ids) =
            normalize_source_group_input("  Core sources  ", vec![4, 2, 4, -1, 2])
                .expect("normalize source group");

        assert_eq!(name, "Core sources");
        assert_eq!(source_ids, vec![2, 4]);
    }

    #[test]
    fn template_kind_validation_returns_typed_error() {
        let error = validate_template_kind("summary").expect_err("reject unsupported kind");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "Unsupported template kind 'summary'");
    }

    #[test]
    fn source_group_input_validation_returns_typed_error() {
        let error = normalize_source_group_input("  ", vec![1]).expect_err("reject empty name");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(error.message, "Source group name cannot be empty");
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
}
