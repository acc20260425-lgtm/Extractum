mod chat;
mod corpus;
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
mod trace;

use tauri::AppHandle;

use self::corpus::{
    list_run_snapshot_messages_page, load_trace_resolution_messages, ListRunSnapshotMessagesRequest,
};
use self::models::{
    AnalysisChatTurn, AnalysisRunDetail, AnalysisRunMessageCursor, AnalysisRunMessagesPage,
    AnalysisRunSummary, AnalysisSourceOption, AnalysisTraceData, AnalysisTraceRef,
};
use self::store::{
    delete_saved_run, fetch_run_row, list_analysis_run_summaries, map_run_detail, map_run_summary,
    AnalysisRunListFilters,
};
use self::trace::{decode_trace_data, normalize_ref, try_build_trace_refs};
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
    push_analysis_document_kind_filter, resolve_analysis_sources, AnalysisSourceResolutionError,
    AnalysisSourceResolutionErrorCode, YoutubeCorpusMode,
};
#[cfg(dev)]
pub use self::fixtures::{
    clear_analysis_redesign_fixture_active_runs, clear_analysis_redesign_fixtures,
    seed_analysis_redesign_fixtures,
};
pub use self::groups::{
    create_analysis_source_group, delete_analysis_source_group, list_analysis_source_groups,
    update_analysis_source_group,
};
pub use self::report::cleanup_interrupted_analysis_runs;
pub(crate) use self::report::resolve_analysis_telegram_history_scope;
pub use self::report_commands::{cancel_analysis_run, start_analysis_report};
pub use self::state::AnalysisState;
pub use self::templates::{
    create_analysis_prompt_template, delete_analysis_prompt_template,
    list_analysis_prompt_templates, update_analysis_prompt_template,
};

const ANALYSIS_RUN_EVENT: &str = "analysis://run";
const ANALYSIS_CHAT_EVENT: &str = "analysis://chat";

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

    list_analysis_run_summaries(&pool, filters).await
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
    around_ref: Option<String>,
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
            around_ref,
        },
    )
    .await
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
    try_build_trace_refs(&normalized_refs, &corpus)
}

#[cfg(test)]
mod tests {
    include!("tests_portable.rs");
}

#[cfg(test)]
mod tests_application;
