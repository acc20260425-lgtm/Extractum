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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GemAnalysisPart {
    Passport,
    Comments,
    DeepRecap,
}

impl GemAnalysisPart {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Passport => "passport",
            Self::Comments => "comments",
            Self::DeepRecap => "deep_recap",
        }
    }

    pub(crate) fn slug(self) -> &'static str {
        match self {
            Self::Passport => "passport",
            Self::Comments => "comments",
            Self::DeepRecap => "deep-recap",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisInputBudget {
    pub(crate) max_input_tokens: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisPartStageExecutionRequest {
    pub run_id: i64,
    pub stage_run_id: i64,
    pub source_snapshot_id: i64,
    pub source_ref_id: String,
    pub part: GemAnalysisPart,
    pub prompt_input_json: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GemAnalysisPartRepairRequest {
    pub run_id: i64,
    pub stage_run_id: i64,
    pub source_snapshot_id: i64,
    pub source_ref_id: String,
    pub part: GemAnalysisPart,
    pub attempt_number: i64,
    pub prompt_input_json: String,
    pub raw_output: String,
    pub error_message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum YoutubeSummaryStageExecutionRequest {
    TranscriptAnalysis(TranscriptAnalysisStageExecutionRequest),
    Synthesis(SynthesisStageExecutionRequest),
    JsonRepair(JsonRepairStageExecutionRequest),
    GemAnalysisPart(GemAnalysisPartStageExecutionRequest),
    GemAnalysisPartRepair(GemAnalysisPartRepairRequest),
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
