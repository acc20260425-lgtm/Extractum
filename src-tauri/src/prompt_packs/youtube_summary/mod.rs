use sqlx::SqlitePool;

pub(crate) mod entities;
#[cfg(test)]
mod entities_tests;
pub(crate) mod execution;
pub(crate) mod execution_result;
#[cfg(test)]
mod execution_tests;
#[cfg(test)]
mod facade_tests;
pub(crate) mod outputs;
#[cfg(test)]
mod outputs_tests;
pub(crate) mod preflight;
#[cfg(test)]
mod preflight_tests;
pub(crate) mod progress;
mod result_validation;
pub(crate) mod snapshots;
#[cfg(test)]
mod snapshots_tests;
pub(crate) mod sources;
pub(crate) mod store;
pub(crate) mod synthesis_execution;
pub(crate) mod synthesis_input;
#[cfg(test)]
mod synthesis_input_tests;
pub(crate) mod tail_stages;
#[cfg(test)]
pub(crate) mod test_support;
pub(crate) mod transcript_execution;
mod types;

use super::dto::{
    PreflightYoutubeSummaryRunRequest, StartYoutubeSummaryRunOutcomeDto,
    StartYoutubeSummaryRunRequest,
};
use crate::error::{AppError, AppResult};
pub(crate) use execution::execute_youtube_summary_run_with_stage_executor;
pub(crate) use preflight::preflight_youtube_summary_in_pool;
pub(crate) use snapshots::create_youtube_summary_run_skeleton_in_pool;
use store::{load_run_by_client_request_id, load_run_summary};
pub use types::ModelBudget;
pub(crate) use types::{
    LlmCompletion, SynthesisStageExecutionRequest, TranscriptAnalysisStageExecutionRequest,
    YoutubeSummaryRunExecutionOutcome, YoutubeSummaryStageExecutionError,
    YoutubeSummaryStageExecutionRequest, SYNTHESIS_STAGE_NAME,
};

pub(crate) async fn start_youtube_summary_run_in_pool(
    pool: &SqlitePool,
    request: StartYoutubeSummaryRunRequest,
) -> AppResult<StartYoutubeSummaryRunOutcomeDto> {
    if request.client_request_id.trim().is_empty() {
        return Err(AppError::validation("client_request_id cannot be empty"));
    }

    if let Some(run) = load_run_by_client_request_id(pool, &request.client_request_id).await? {
        return Ok(StartYoutubeSummaryRunOutcomeDto::Started { run });
    }

    let preflight_request = PreflightYoutubeSummaryRunRequest {
        project_id: request.project_id,
        source_ids: request.source_ids.clone(),
        profile_id: request.profile_id.clone(),
        model_override: request.model_override.clone(),
        runtime_provider: request.runtime_provider,
        browser_provider_config: request.browser_provider_config.clone(),
        output_language: request.output_language.clone(),
        control_preset: request.control_preset.clone(),
        evidence_mode: request.evidence_mode.clone(),
        include_comments: request.include_comments,
    };
    let preflight = preflight_youtube_summary_in_pool(
        pool,
        preflight_request,
        ModelBudget {
            input_token_limit: Some(32_000),
        },
    )
    .await?;

    if preflight.included_videos.is_empty() || !preflight.blocking_failures.is_empty() {
        return Ok(StartYoutubeSummaryRunOutcomeDto::Blocked { preflight });
    }

    let run_id = create_youtube_summary_run_skeleton_in_pool(pool, request, 0).await?;
    let run = load_run_summary(pool, run_id).await?;
    Ok(StartYoutubeSummaryRunOutcomeDto::Started { run })
}

pub(crate) fn now_string() -> String {
    crate::time::now_rfc3339_utc()
}

pub(crate) fn estimate_tokens(text: &str) -> i64 {
    ((text.chars().count() as f64) / 4.0).ceil() as i64
}
