mod chat;
mod corpus;
mod events;
#[cfg(dev)]
mod fixtures;
mod groups;
pub(crate) mod report;
mod report_commands;
pub(crate) mod store;
mod templates;

use extractum_analysis::{
    delete_analysis_run as delete_analysis_run_in_pool,
    get_analysis_run_trace as get_analysis_run_trace_in_pool,
    list_analysis_run_messages as list_analysis_run_messages_in_pool,
    resolve_analysis_trace_refs as resolve_analysis_trace_refs_in_pool, AnalysisRunDetail,
    AnalysisRunListFilters, AnalysisRunMessageCursor, AnalysisRunMessagesPage, AnalysisRunSummary,
    AnalysisSourceOption, AnalysisTraceData, AnalysisTraceRef,
};
use tauri::AppHandle;

use self::store::{
    get_analysis_run_in_pool, list_active_analysis_runs_in_pool, list_analysis_runs_in_pool,
};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::sources::{require_source_identity_ready, SourceIdentityRepairState};

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
    seed_analysis_redesign_fixtures, AnalysisRedesignFixtureSummary,
};
#[allow(unused_imports)]
pub(crate) use self::groups::load_analysis_source_group_for_enrichment;
pub use self::groups::{
    create_analysis_source_group, delete_analysis_source_group, list_analysis_source_groups,
    update_analysis_source_group,
};
pub use self::report::cleanup_interrupted_analysis_runs;
pub use self::report_commands::{cancel_analysis_run, start_analysis_report};
pub use self::templates::{
    create_analysis_prompt_template, delete_analysis_prompt_template,
    list_analysis_prompt_templates, update_analysis_prompt_template,
};
pub use extractum_analysis::AnalysisState;
#[allow(unused_imports)]
pub(crate) use extractum_analysis::{
    analysis_run_ids_depending_on_sources, delete_project_analysis_runs,
    load_analysis_run_diagnostics,
};

pub(crate) const ANALYSIS_RUN_EVENT: &str = "analysis://run";
pub(crate) const ANALYSIS_CHAT_EVENT: &str = "analysis://chat";

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

// Preserve the public analysis-filter command contract in Wave 0.
#[allow(clippy::too_many_arguments)]
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

pub async fn get_analysis_run(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<Option<AnalysisRunDetail>> {
    let pool = get_pool(&handle).await?;
    get_analysis_run_in_pool(&pool, run_id).await
}

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

pub async fn get_analysis_run_trace(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<AnalysisTraceData> {
    let pool = get_pool(&handle).await?;
    get_analysis_run_trace_in_pool(&pool, run_id).await
}

pub async fn delete_analysis_run(
    handle: AppHandle,
    state: tauri::State<'_, AnalysisState>,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    delete_analysis_run_in_pool(&pool, state.inner(), run_id).await
}

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
mod tests_application;
