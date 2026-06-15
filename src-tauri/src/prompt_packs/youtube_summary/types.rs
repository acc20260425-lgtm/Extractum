use crate::error::AppError;
use crate::prompt_packs::json_repair::JsonRepairStageExecutionRequest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelBudget {
    pub input_token_limit: Option<i64>,
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
