use sqlx::SqlitePool;

use super::dto::{
    PreflightYoutubeSummaryRunRequest, StartYoutubeSummaryRunOutcomeDto,
    StartYoutubeSummaryRunRequest,
};
use super::json_repair::JsonRepairStageExecutionRequest;
#[cfg(test)]
pub(crate) use super::youtube_summary_execution::execute_youtube_summary_run_with_fake_completions;
pub(crate) use super::youtube_summary_execution::execute_youtube_summary_run_with_stage_executor;
pub(crate) use super::youtube_summary_preflight::preflight_youtube_summary_in_pool;
use super::youtube_summary_run_store::{load_run_by_client_request_id, load_run_summary};
pub(crate) use super::youtube_summary_snapshots::create_youtube_summary_run_skeleton_in_pool;
#[cfg(test)]
pub(crate) use super::youtube_summary_snapshots::{
    freeze_comment_material_refs, test_comment_policy,
};
#[cfg(test)]
pub(crate) use super::youtube_summary_stage_outputs::{
    execute_synthesis_stage_with_completion, execute_transcript_analysis_stage_with_completion,
};
#[cfg(test)]
pub(crate) use super::youtube_summary_synthesis_input::build_synthesis_stage_input;
use crate::error::{AppError, AppResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelBudget {
    pub input_token_limit: Option<i64>,
}

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

#[derive(Clone, Debug)]
pub(crate) struct LlmCompletion {
    pub text: String,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub latency_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TranscriptAnalysisStageExecutionRequest {
    pub run_id: i64,
    pub stage_run_id: i64,
    pub source_snapshot_id: i64,
    pub source_ref_id: String,
    pub prompt_input_json: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum YoutubeSummaryStageExecutionRequest {
    TranscriptAnalysis(TranscriptAnalysisStageExecutionRequest),
    Synthesis(SynthesisStageExecutionRequest),
    JsonRepair(JsonRepairStageExecutionRequest),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SynthesisStageExecutionRequest {
    pub run_id: i64,
    pub stage_run_id: i64,
    pub prompt_input_json: String,
}

pub(crate) const SYNTHESIS_STAGE_NAME: &str = "youtube_summary/synthesis";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct YoutubeSummaryRunExecutionOutcome {
    pub run_id: i64,
    pub run_status: String,
    pub progress_current: i64,
    pub progress_total: i64,
    pub message: String,
}

#[derive(Debug)]
pub(crate) enum YoutubeSummaryStageExecutionError {
    Cancelled,
    Failed(AppError),
}

impl From<AppError> for YoutubeSummaryStageExecutionError {
    fn from(error: AppError) -> Self {
        Self::Failed(error)
    }
}

pub(crate) fn now_string() -> String {
    crate::time::now_rfc3339_utc()
}

pub(crate) fn estimate_tokens(text: &str) -> i64 {
    ((text.chars().count() as f64) / 4.0).ceil() as i64
}
