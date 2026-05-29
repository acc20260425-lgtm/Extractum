use tauri::AppHandle;

use crate::error::AppResult;
use crate::llm::LlmSchedulerState;

use super::report::{self, StartAnalysisReportRequest};
use super::AnalysisState;

#[tauri::command]
pub async fn cancel_analysis_run(
    handle: AppHandle,
    state: tauri::State<'_, AnalysisState>,
    scheduler: tauri::State<'_, LlmSchedulerState>,
    run_id: i64,
) -> AppResult<()> {
    report::request_analysis_run_cancel(&handle, state.inner(), scheduler.inner(), run_id).await
}

#[tauri::command]
#[expect(
    clippy::too_many_arguments,
    reason = "Tauri command signature is the frontend IPC contract; inputs are normalized into StartAnalysisReportRequest immediately."
)]
pub async fn start_analysis_report(
    handle: AppHandle,
    state: tauri::State<'_, AnalysisState>,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template_id: i64,
    model_override: Option<String>,
    profile_id: Option<String>,
    youtube_corpus_mode: Option<String>,
    include_migrated_history: bool,
) -> AppResult<i64> {
    report::start_analysis_report_run(
        handle,
        state.inner(),
        StartAnalysisReportRequest {
            source_id,
            source_group_id,
            period_from,
            period_to,
            output_language,
            prompt_template_id,
            model_override,
            profile_id,
            youtube_corpus_mode,
            include_migrated_history,
        },
    )
    .await
}
