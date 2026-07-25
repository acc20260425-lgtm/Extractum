use extractum_analysis::{
    mark_interrupted_analysis_runs,
    request_analysis_run_cancel as request_analysis_run_cancel_in_pool, AnalysisState,
};
use extractum_core::error::AppResult;
use extractum_llm::LlmSchedulerState;
use tauri::AppHandle;

use crate::db::get_pool;

use super::super::events::TauriAnalysisEventSink;

pub async fn cleanup_interrupted_analysis_runs(handle: AppHandle) {
    if let Ok(pool) = get_pool(&handle).await {
        let _ = mark_interrupted_analysis_runs(&pool).await;
    }
}

pub(crate) async fn request_analysis_run_cancel(
    handle: &AppHandle,
    state: &AnalysisState,
    scheduler: &LlmSchedulerState,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(handle).await?;
    let sink = TauriAnalysisEventSink::new(handle.clone());
    request_analysis_run_cancel_in_pool(&pool, state, scheduler, &sink, run_id).await
}
