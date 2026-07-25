use std::sync::Arc;

use extractum_analysis::{
    execute_analysis_report, finalize_analysis_report_execution, AnalysisEventSink,
    AnalysisExecutionError, AnalysisScopeKind, AnalysisState,
};
pub(crate) use extractum_analysis::{
    prepare_analysis_report, prepare_analysis_report_execution, StartAnalysisReportRequest,
};
use extractum_core::error::AppResult;
use extractum_llm::{resolve_effective_model, resolve_model_input_token_limit, LlmSchedulerState};
use tauri::{AppHandle, Manager};

use crate::db::get_pool;
use crate::llm::resolve_profile_for_backend;

use super::corpus::{
    resolve_analysis_sources, AnalysisSourceResolutionError, AppAnalysisCorpusReader,
};
use super::events::TauriAnalysisEventSink;

#[path = "report/lifecycle.rs"]
mod lifecycle;

pub use self::lifecycle::cleanup_interrupted_analysis_runs;
pub(crate) use self::lifecycle::request_analysis_run_cancel;

pub(crate) async fn start_analysis_report_run(
    handle: AppHandle,
    state: &AnalysisState,
    request: StartAnalysisReportRequest,
) -> AppResult<i64> {
    let pool = get_pool(&handle).await?;
    let preparation = prepare_analysis_report(&pool, request).await?;
    let resolved_profile =
        resolve_profile_for_backend(&handle, preparation.requested_profile_id()).await?;
    let effective_model = resolve_effective_model(&resolved_profile, preparation.model_override())?;
    let model_input_token_limit =
        resolve_model_input_token_limit(&resolved_profile, &effective_model).await;
    let preparation = preparation.resolve_youtube_corpus_mode()?;
    let (source_id, source_group_id, project_id) = match preparation.scope_kind() {
        AnalysisScopeKind::SingleSource => (Some(preparation.scope_id()), None, None),
        AnalysisScopeKind::SourceGroup => (None, Some(preparation.scope_id()), None),
        AnalysisScopeKind::Project => (None, None, Some(preparation.scope_id())),
    };
    let resolved_sources = resolve_analysis_sources(&pool, source_id, source_group_id, project_id)
        .await
        .map_err(AnalysisSourceResolutionError::into_app_error)?;
    let resolved_scope = resolved_sources.into_scope();
    let reader = AppAnalysisCorpusReader::new(pool.clone());
    let ticket = prepare_analysis_report_execution(
        &pool,
        state,
        &reader,
        preparation,
        resolved_scope,
        resolved_profile,
        effective_model,
        model_input_token_limit,
    )
    .await?;
    let run_id = ticket.run_id();
    let app_handle = handle.clone();
    tokio::spawn(async move {
        let state = app_handle.state::<AnalysisState>();
        let scheduler = app_handle.state::<Arc<LlmSchedulerState>>().inner().clone();
        let sink: Arc<dyn AnalysisEventSink> =
            Arc::new(TauriAnalysisEventSink::new(app_handle.clone()));
        let outcome = match get_pool(&app_handle).await {
            Ok(execution_pool) => {
                let reader = AppAnalysisCorpusReader::new(execution_pool.clone());
                execute_analysis_report(
                    &execution_pool,
                    state.inner(),
                    scheduler,
                    &reader,
                    sink.clone(),
                    ticket,
                )
                .await
            }
            Err(error) => Err(AnalysisExecutionError::Failed(error.to_string())),
        };
        let terminal_pool = get_pool(&app_handle).await.ok();
        finalize_analysis_report_execution(
            terminal_pool.as_ref(),
            state.inner(),
            sink.as_ref(),
            run_id,
            outcome,
        )
        .await;
    });

    Ok(run_id)
}

#[cfg(test)]
mod tests;
