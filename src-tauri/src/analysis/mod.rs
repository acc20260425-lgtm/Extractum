mod chat;
mod corpus;
mod corpus_portable;
#[path = "domain_portable.rs"]
mod domain;
mod events;
#[cfg(dev)]
mod fixtures;
mod groups;
pub(crate) mod models;
pub(crate) mod report;
mod report_commands;
mod state;
pub(crate) mod store;
mod templates;
#[cfg(test)]
mod test_schema;
mod trace;

use tauri::AppHandle;

use self::corpus::list_run_snapshot_messages_page as list_analysis_run_messages_in_pool;
use self::models::{
    AnalysisChatTurn, AnalysisRunDetail, AnalysisRunMessageCursor, AnalysisRunMessagesPage,
    AnalysisRunSummary, AnalysisSourceOption, AnalysisTraceData, AnalysisTraceRef,
};
use self::store::{
    delete_analysis_run as delete_analysis_run_in_pool, get_analysis_run_in_pool,
    list_active_analysis_runs_in_pool, list_analysis_runs_in_pool, AnalysisRunListFilters,
};
use self::trace::{get_analysis_run_trace_in_pool, resolve_analysis_trace_refs_in_pool};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::sources::{require_source_identity_ready, SourceIdentityRepairState};

pub(crate) use self::domain::{
    default_report_template_body, now_secs, validate_chat_role, validate_chat_turns,
    ANALYSIS_FALLBACK_CHUNK_TARGET_CHARS, ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_SCOPE_TYPE_PROJECT,
    ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED,
    ANALYSIS_STATUS_COMPLETED, ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED,
    ANALYSIS_STATUS_RUNNING, DEFAULT_REPORT_TEMPLATE_NAME, TEMPLATE_KIND_CHAT,
    TEMPLATE_KIND_REPORT,
};

pub use self::chat::{
    ask_analysis_run_question, clear_analysis_chat_messages, list_analysis_chat_messages,
};
#[allow(unused_imports)]
pub(crate) use self::corpus::{
    resolve_analysis_sources, resolve_analysis_telegram_history_scope,
    AnalysisSourceResolutionError, AnalysisSourceResolutionErrorCode, YoutubeCorpusMode,
};
#[cfg(dev)]
pub use self::fixtures::{
    clear_analysis_redesign_fixture_active_runs, clear_analysis_redesign_fixtures,
    seed_analysis_redesign_fixtures,
};
pub(crate) use self::groups::load_analysis_source_group_for_enrichment;
pub use self::groups::{
    create_analysis_source_group, delete_analysis_source_group, list_analysis_source_groups,
    update_analysis_source_group,
};
pub use self::report::cleanup_interrupted_analysis_runs;
pub use self::report_commands::{cancel_analysis_run, start_analysis_report};
pub use self::state::AnalysisState;
pub(crate) use self::store::{
    analysis_run_ids_depending_on_sources, delete_project_analysis_runs,
    load_analysis_run_diagnostics,
};
pub use self::templates::{
    create_analysis_prompt_template, delete_analysis_prompt_template,
    list_analysis_prompt_templates, update_analysis_prompt_template,
};

const ANALYSIS_RUN_EVENT: &str = "analysis://run";
const ANALYSIS_CHAT_EVENT: &str = "analysis://chat";

fn is_analysis_trace_ref_digits(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
}

fn parse_analysis_trace_ref_millis(value: &str) -> Option<i64> {
    is_analysis_trace_ref_digits(value)
        .then(|| value.parse::<i64>().ok())
        .flatten()
}

fn has_normalizable_analysis_trace_ref(refs: &[String]) -> bool {
    refs.iter().any(|reference| {
        let candidate = reference.trim().trim_matches('[').trim_matches(']');
        let Some((source_part, item_part)) = candidate.split_once("-i") else {
            return false;
        };
        let Some(source_digits) = source_part.strip_prefix('s') else {
            return false;
        };
        if !is_analysis_trace_ref_digits(source_digits) {
            return false;
        }

        let (item_digits, timestamp_suffix) = match item_part.split_once('@') {
            Some((digits, suffix)) => (digits, Some(suffix)),
            None => (item_part, None),
        };
        if !is_analysis_trace_ref_digits(item_digits) {
            return false;
        }

        let Some(suffix) = timestamp_suffix else {
            return true;
        };
        let Some(body) = suffix.strip_suffix("ms") else {
            return false;
        };
        if let Some((start, end)) = body.split_once('-') {
            let (Some(start), Some(end)) = (
                parse_analysis_trace_ref_millis(start),
                parse_analysis_trace_ref_millis(end),
            ) else {
                return false;
            };
            end >= start
        } else {
            parse_analysis_trace_ref_millis(body).is_some()
        }
    })
}

#[tauri::command]
pub async fn list_analysis_sources(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
) -> AppResult<Vec<AnalysisSourceOption>> {
    require_source_identity_ready(repair_state.inner()).await?;
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
    query: Option<String>,
    status: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    template: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> AppResult<Vec<AnalysisRunSummary>> {
    let filters = AnalysisRunListFilters::for_analysis(
        source_id,
        source_group_id,
        limit.unwrap_or(20),
        query,
        status,
        provider,
        model,
        template,
        date_from,
        date_to,
    )?;
    let pool = get_pool(&handle).await?;

    list_analysis_runs_in_pool(&pool, filters).await
}

#[tauri::command]
pub async fn list_active_analysis_runs(
    handle: AppHandle,
    state: tauri::State<'_, AnalysisState>,
) -> AppResult<Vec<AnalysisRunSummary>> {
    let pool = get_pool(&handle).await?;
    let active_ids = state.active_report_run_ids().await;
    let active_runs = list_active_analysis_runs_in_pool(&pool, &active_ids).await?;
    let returned_ids = active_runs
        .iter()
        .map(|run| run.id)
        .collect::<std::collections::HashSet<_>>();
    let stale_ids = active_ids
        .difference(&returned_ids)
        .copied()
        .collect::<Vec<_>>();

    for run_id in stale_ids {
        state.remove_active_report_run(run_id).await;
    }

    Ok(active_runs)
}

#[tauri::command]
pub async fn get_analysis_run(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<Option<AnalysisRunDetail>> {
    let pool = get_pool(&handle).await?;
    get_analysis_run_in_pool(&pool, run_id).await
}

#[tauri::command]
pub async fn list_analysis_run_messages(
    handle: AppHandle,
    run_id: i64,
    after: Option<AnalysisRunMessageCursor>,
    limit: Option<i64>,
    source_id: Option<i64>,
    around_ref: Option<String>,
) -> AppResult<AnalysisRunMessagesPage> {
    let pool = get_pool(&handle).await?;
    list_analysis_run_messages_in_pool(&pool, run_id, after, limit, source_id, around_ref).await
}

#[tauri::command]
pub async fn get_analysis_run_trace(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<AnalysisTraceData> {
    let pool = get_pool(&handle).await?;
    get_analysis_run_trace_in_pool(&pool, run_id).await
}

#[tauri::command]
pub async fn delete_analysis_run(
    handle: AppHandle,
    state: tauri::State<'_, AnalysisState>,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    delete_analysis_run_in_pool(&pool, state.inner(), run_id).await
}

#[tauri::command]
pub async fn resolve_analysis_trace_refs(
    handle: AppHandle,
    run_id: i64,
    refs: Vec<String>,
) -> AppResult<Vec<AnalysisTraceRef>> {
    if !has_normalizable_analysis_trace_ref(&refs) {
        return Ok(Vec::new());
    }

    let pool = get_pool(&handle).await?;
    resolve_analysis_trace_refs_in_pool(&pool, run_id, refs).await
}

#[cfg(test)]
mod tests {
    include!("tests_portable.rs");
}

#[cfg(test)]
mod tests_application;
