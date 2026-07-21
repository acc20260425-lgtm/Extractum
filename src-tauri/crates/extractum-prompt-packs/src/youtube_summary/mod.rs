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
pub(crate) mod gem_analysis;
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

use super::dto::PromptPackRuntimeProvider;
#[cfg(test)]
use super::dto::{
    PreflightYoutubeSummaryRunRequest, StartYoutubeSummaryRunOutcomeDto,
    StartYoutubeSummaryRunRequest, YoutubeSummaryPreflightFailure,
};
#[cfg(test)]
use super::source_port::PromptPackSourceReader;
use super::source_port::PromptPackTranscriptSegment;
#[cfg(test)]
pub(crate) use execution::execute_youtube_summary_run_with_stage_executor;
pub(crate) use execution::{
    execute_youtube_summary_run_with_stage_executor_with_options, YoutubeSummaryExecutionOptions,
};
#[cfg(test)]
use extractum_core::error::AppError;
use extractum_core::error::AppResult;
pub(crate) use preflight::preflight_youtube_summary;
pub(crate) use snapshots::create_youtube_summary_run_skeleton_with_source;
use store::load_run_by_client_request_id;
#[cfg(test)]
use store::load_run_summary;
#[cfg(test)]
use test_support::TestPromptPackSourceReader as AppPromptPackSourceReader;
#[allow(unused_imports)]
pub(crate) use types::GemAnalysisInputBudget;
pub use types::ModelBudget;
pub use types::YoutubeSummaryRunExecutionOutcome;
pub(crate) use types::{
    GemAnalysisPart, GemAnalysisPartRepairRequest, GemAnalysisPartStageExecutionRequest,
    LlmCompletion, SynthesisStageExecutionRequest, TranscriptAnalysisStageExecutionRequest,
    YoutubeSummaryStageExecutionError, YoutubeSummaryStageExecutionRequest, SYNTHESIS_STAGE_NAME,
};

#[cfg(test)]
pub(crate) async fn start_youtube_summary_run_in_pool(
    pool: &SqlitePool,
    request: StartYoutubeSummaryRunRequest,
) -> AppResult<StartYoutubeSummaryRunOutcomeDto> {
    let source = AppPromptPackSourceReader::new(pool.clone());
    start_youtube_summary_run_with_source(pool, &source, request).await
}

#[cfg(test)]
pub(crate) async fn start_youtube_summary_run_with_source(
    pool: &SqlitePool,
    source: &dyn PromptPackSourceReader,
    request: StartYoutubeSummaryRunRequest,
) -> AppResult<StartYoutubeSummaryRunOutcomeDto> {
    start_youtube_summary_run_with_preflight_failures_and_source(pool, source, request, Vec::new())
        .await
}

pub(crate) async fn load_youtube_summary_run_by_client_request_id_in_pool(
    pool: &SqlitePool,
    client_request_id: &str,
) -> AppResult<Option<super::dto::PromptPackRunSummaryDto>> {
    load_run_by_client_request_id(pool, client_request_id).await
}

#[cfg(test)]
pub(crate) async fn start_youtube_summary_run_with_preflight_failures_in_pool(
    pool: &SqlitePool,
    request: StartYoutubeSummaryRunRequest,
    extra_blocking_failures: Vec<YoutubeSummaryPreflightFailure>,
) -> AppResult<StartYoutubeSummaryRunOutcomeDto> {
    let source = AppPromptPackSourceReader::new(pool.clone());
    start_youtube_summary_run_with_preflight_failures_and_source(
        pool,
        &source,
        request,
        extra_blocking_failures,
    )
    .await
}

#[cfg(test)]
async fn start_youtube_summary_run_with_preflight_failures_and_source(
    pool: &SqlitePool,
    source: &dyn PromptPackSourceReader,
    request: StartYoutubeSummaryRunRequest,
    extra_blocking_failures: Vec<YoutubeSummaryPreflightFailure>,
) -> AppResult<StartYoutubeSummaryRunOutcomeDto> {
    if request.client_request_id().trim().is_empty() {
        return Err(AppError::validation("client_request_id cannot be empty"));
    }

    if let Some(run) = load_run_by_client_request_id(pool, request.client_request_id()).await? {
        return Ok(StartYoutubeSummaryRunOutcomeDto::Started { run });
    }

    let preflight_request = PreflightYoutubeSummaryRunRequest::new(
        request.project_id,
        request.source_ids.clone(),
        request.profile_id().map(str::to_owned),
        request.model_override().map(str::to_owned),
        request.runtime_provider(),
        request.browser_provider_config.clone(),
        request.output_language.clone(),
        request.control_preset.clone(),
        request.evidence_mode.clone(),
        request.include_comments,
    );
    let mut preflight = preflight_youtube_summary(
        source,
        preflight_request,
        model_budget_for_runtime(request.runtime_provider()),
    )
    .await?;
    preflight.blocking_failures.extend(extra_blocking_failures);

    if preflight.included_videos.is_empty() || !preflight.blocking_failures.is_empty() {
        return Ok(StartYoutubeSummaryRunOutcomeDto::Blocked { preflight });
    }

    let run_id = create_youtube_summary_run_skeleton_with_source(pool, source, request, 0).await?;
    let run = load_run_summary(pool, run_id).await?;
    Ok(StartYoutubeSummaryRunOutcomeDto::Started { run })
}

#[cfg(test)]
pub(crate) async fn preflight_youtube_summary_in_pool(
    pool: &SqlitePool,
    request: PreflightYoutubeSummaryRunRequest,
    model_budget: ModelBudget,
) -> AppResult<super::dto::YoutubeSummaryPreflightResponse> {
    let source = AppPromptPackSourceReader::new(pool.clone());
    preflight_youtube_summary(&source, request, model_budget).await
}

#[cfg(test)]
pub(crate) async fn create_youtube_summary_run_skeleton_in_pool(
    pool: &SqlitePool,
    request: StartYoutubeSummaryRunRequest,
    pack_version_id_hint: i64,
) -> AppResult<i64> {
    let source = AppPromptPackSourceReader::new(pool.clone());
    create_youtube_summary_run_skeleton_with_source(pool, &source, request, pack_version_id_hint)
        .await
}

pub(crate) fn model_budget_for_runtime(runtime_provider: PromptPackRuntimeProvider) -> ModelBudget {
    match runtime_provider {
        PromptPackRuntimeProvider::Api => ModelBudget {
            input_token_limit: Some(32_000),
        },
        PromptPackRuntimeProvider::GeminiBrowser => ModelBudget {
            input_token_limit: None,
        },
    }
}

pub(crate) fn now_string() -> String {
    extractum_core::time::now_rfc3339_utc()
}

pub(crate) fn estimate_tokens(text: &str) -> i64 {
    ((text.chars().count() as f64) / 4.0).ceil() as i64
}

pub(crate) fn render_transcript_snapshot_text(segments: &[PromptPackTranscriptSegment]) -> String {
    segments
        .iter()
        .map(PromptPackTranscriptSegment::text)
        .collect::<Vec<_>>()
        .join("\n")
}
