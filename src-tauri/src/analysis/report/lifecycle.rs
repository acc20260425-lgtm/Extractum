use tauri::AppHandle;

use crate::db::get_pool;

use super::super::events::TauriAnalysisEventSink;

include!("lifecycle_portable.rs");

pub async fn cleanup_interrupted_analysis_runs(handle: AppHandle) {
    if let Ok(pool) = get_pool(&handle).await {
        let _ = super::mark_interrupted_analysis_runs(&pool).await;
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
    super::request_analysis_run_cancel_in_pool(&pool, state, scheduler, &sink, run_id).await
}
