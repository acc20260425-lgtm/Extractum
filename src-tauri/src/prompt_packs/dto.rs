#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightYoutubeSummaryRunRequest {
    pub project_id: Option<i64>,
    pub source_ids: Vec<i64>,
    pub profile_id: Option<String>,
    pub model_override: Option<String>,
    pub output_language: String,
    pub control_preset: String,
    pub evidence_mode: String,
    pub include_comments: bool,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartYoutubeSummaryRunRequest {
    pub client_request_id: String,
    pub project_id: Option<i64>,
    pub source_ids: Vec<i64>,
    pub profile_id: Option<String>,
    pub model_override: Option<String>,
    pub output_language: String,
    pub control_preset: String,
    pub evidence_mode: String,
    pub include_comments: bool,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSummaryPreflightResponse {
    pub pack_id: String,
    pub pack_version: String,
    pub included_videos: Vec<YoutubeSummaryPreflightVideo>,
    pub skipped_videos: Vec<YoutubeSummaryPreflightSkippedVideo>,
    pub blocking_failures: Vec<YoutubeSummaryPreflightFailure>,
    pub estimated_input_tokens: i64,
    pub selected_model_input_limit: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSummaryPreflightVideo {
    pub source_id: i64,
    pub video_id: String,
    pub title: String,
    pub estimated_input_tokens: i64,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSummaryPreflightSkippedVideo {
    pub source_id: Option<i64>,
    pub video_id: Option<String>,
    pub title: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSummaryPreflightFailure {
    pub source_id: Option<i64>,
    pub reason: String,
    pub message: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackRunEvent {
    pub run_id: i64,
    pub request_id: String,
    pub kind: String,
    pub run_status: String,
    pub phase: String,
    pub stage_run_id: Option<i64>,
    pub stage_name: Option<String>,
    pub source_snapshot_id: Option<i64>,
    pub queue_position: Option<i64>,
    pub progress_current: Option<i64>,
    pub progress_total: Option<i64>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPromptPackRunsRequest {
    pub project_id: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackRunSummaryDto {
    pub run_id: i64,
    pub project_id: Option<i64>,
    pub run_label: Option<String>,
    pub pack_id: String,
    pub pack_version: String,
    pub run_status: String,
    pub result_status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub latest_message: Option<String>,
    pub progress_current: Option<i64>,
    pub progress_total: Option<i64>,
    pub queue_position: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackStageRunDto {
    pub stage_run_id: i64,
    pub run_id: i64,
    pub source_snapshot_id: Option<i64>,
    pub stage_name: String,
    pub stage_order: i64,
    pub stage_status: String,
    pub latest_message: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum StartYoutubeSummaryRunOutcomeDto {
    Started {
        run: PromptPackRunSummaryDto,
    },
    Blocked {
        preflight: YoutubeSummaryPreflightResponse,
    },
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackResultDto {
    pub run_id: i64,
    pub result_status: String,
    pub canonical: serde_json::Value,
    pub storage_warning: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackStageArtifactSummaryDto {
    pub stage_run_id: i64,
    pub artifact_kind: String,
    pub attempt_number: i64,
    pub artifact_index: i64,
    pub content_type: String,
    pub content_hash: String,
    pub created_at: String,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackStageArtifactDto {
    pub stage_run_id: i64,
    pub artifact_kind: String,
    pub attempt_number: i64,
    pub artifact_index: i64,
    pub content_type: String,
    pub content: serde_json::Value,
    pub created_at: String,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackValidationFindingDto {
    pub run_id: i64,
    pub stage_run_id: Option<i64>,
    pub severity: String,
    pub code: String,
    pub message: String,
    pub object_path: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackAuditEventDto {
    pub run_id: i64,
    pub event_kind: String,
    pub message: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub created_at: String,
}

impl StartYoutubeSummaryRunOutcomeDto {
    #[cfg(test)]
    pub fn expect_started(self, context: &str) -> PromptPackRunSummaryDto {
        match self {
            StartYoutubeSummaryRunOutcomeDto::Started { run } => run,
            StartYoutubeSummaryRunOutcomeDto::Blocked { .. } => {
                panic!("{context}: expected started outcome")
            }
        }
    }

    #[cfg(test)]
    pub fn expect_blocked(self, context: &str) -> YoutubeSummaryPreflightResponse {
        match self {
            StartYoutubeSummaryRunOutcomeDto::Blocked { preflight } => preflight,
            StartYoutubeSummaryRunOutcomeDto::Started { .. } => {
                panic!("{context}: expected blocked outcome")
            }
        }
    }
}
