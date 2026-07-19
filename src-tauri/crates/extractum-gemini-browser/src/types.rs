use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserProviderStatusKind {
    NotStarted,
    Ready,
    NeedsLogin,
    NeedsManualAction,
    Running,
    Stopped,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserManualAction {
    Login,
    AccountPicker,
    Consent,
    Captcha,
    UnknownModal,
    StartChromeCdp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserProviderMode {
    Managed,
    CdpAttach,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserDebugErrorStage {
    Setup,
    Composer,
    Send,
    Answer,
    Artifacts,
    Transport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserAnswerCompletionReason {
    Stable,
    TimeoutLatest,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserCandidateRejectReason {
    Baseline,
    Composer,
    PromptContainer,
    Navigation,
    AccountOrLogin,
    Controls,
    MultiTurn,
    NotVisible,
    Empty,
    LowerScore,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserAnswerGrouping {
    AssistantTurn,
    SingleNode,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserAnswerExtractionDebug {
    pub raw_candidate_count: u64,
    pub grouped_candidate_count: u64,
    pub selected_candidate_length: u64,
    pub returned_text_length: u64,
    pub selected_grouping: GeminiBrowserAnswerGrouping,
    pub selected_candidate_rank: Option<u64>,
    pub selected_score: Option<i64>,
    pub largest_candidate_length: u64,
    pub larger_valid_candidate_available: bool,
    pub larger_rejected_candidate_count: u64,
    pub larger_rejected_reasons: Vec<GeminiBrowserCandidateRejectReason>,
    pub top_candidate_lengths: Vec<u64>,
    pub busy_visible_at_completion: bool,
    pub last_growth_elapsed_ms: Option<u64>,
    pub candidate_signature_changed_count: u64,
    pub stable_poll_count_after_last_candidate_change: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserProviderConfig {
    pub mode: GeminiBrowserProviderMode,
    #[serde(alias = "cdpEndpoint")]
    pub cdp_endpoint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserStartChromeResult {
    pub browser_profile_dir: String,
    pub cdp_endpoint: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserProviderStatus {
    pub status: GeminiBrowserProviderStatusKind,
    pub manual_action: Option<GeminiBrowserManualAction>,
    pub active_run_id: Option<String>,
    pub queue_depth: usize,
    pub browser_profile_dir: String,
    pub latest_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRunRequest {
    pub run_id: String,
    pub prompt: String,
    pub source: String,
    pub artifact_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserRunStatus {
    Queued,
    Running,
    Ok,
    Ready,
    NeedsLogin,
    NeedsManualAction,
    Blocked,
    Timeout,
    BrowserCrashed,
    Failed,
    Cancelled,
}

impl GeminiBrowserRunStatus {
    #[cfg(test)]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Ok | Self::Ready)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Ok
                | Self::Ready
                | Self::NeedsLogin
                | Self::NeedsManualAction
                | Self::Blocked
                | Self::Timeout
                | Self::BrowserCrashed
                | Self::Failed
                | Self::Cancelled
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserArtifactRefs {
    pub run_dir: Option<String>,
    pub html: Option<String>,
    pub screenshot: Option<String>,
    pub telemetry: Option<String>,
    #[serde(default)]
    pub answer_extraction: Option<String>,
    pub artifact_write_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRunDebugSummary {
    pub mode: GeminiBrowserProviderMode,
    pub composer_found: bool,
    pub send_button_found: bool,
    pub generation_busy_observed: bool,
    pub answer_found: bool,
    pub answer_selector: Option<String>,
    pub waited_for_send_ms: u64,
    pub waited_for_answer_ms: u64,
    pub answer_stable_ms: u64,
    pub answer_completion_reason: GeminiBrowserAnswerCompletionReason,
    pub final_text_length: u64,
    pub error_stage: Option<GeminiBrowserDebugErrorStage>,
    #[serde(default)]
    pub extraction: Option<GeminiBrowserAnswerExtractionDebug>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRunResult {
    pub run_id: String,
    pub status: GeminiBrowserRunStatus,
    pub text: Option<String>,
    pub message: Option<String>,
    pub manual_action: Option<GeminiBrowserManualAction>,
    pub artifacts: GeminiBrowserArtifactRefs,
    pub elapsed_ms: u64,
    #[serde(default)]
    pub debug_summary: Option<GeminiBrowserRunDebugSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRun {
    pub run_id: String,
    pub source: String,
    pub status: GeminiBrowserRunStatus,
    pub prompt_preview: String,
    pub created_at: String,
    pub updated_at: String,
    pub result: Option<GeminiBrowserRunResult>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRunLogSummary {
    pub runs: Vec<GeminiBrowserRun>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GeminiBrowserSidecarCommand {
    Status {
        browser_profile_dir: String,
        browser_config: Option<GeminiBrowserProviderConfig>,
    },
    OpenBrowser {
        browser_profile_dir: String,
        browser_config: Option<GeminiBrowserProviderConfig>,
    },
    SendSingle {
        request: GeminiBrowserRunRequest,
        browser_profile_dir: String,
        artifact_dir: String,
        browser_config: Option<GeminiBrowserProviderConfig>,
    },
    Resume {
        run_id: Option<String>,
        browser_profile_dir: String,
        browser_config: Option<GeminiBrowserProviderConfig>,
    },
    Stop,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserSidecarEnvelope {
    pub id: String,
    pub command: GeminiBrowserSidecarCommand,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GeminiBrowserSidecarResponse {
    Status { status: GeminiBrowserProviderStatus },
    RunResult { result: GeminiBrowserRunResult },
    Ack,
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_statuses_include_ready_and_ok() {
        assert!(GeminiBrowserRunStatus::Ok.is_success());
        assert!(GeminiBrowserRunStatus::Ready.is_success());
        assert!(!GeminiBrowserRunStatus::NeedsLogin.is_success());
        assert!(GeminiBrowserRunStatus::NeedsLogin.is_terminal());
        assert!(!GeminiBrowserRunStatus::Running.is_terminal());
    }

    #[test]
    fn sidecar_command_serializes_with_snake_case_tag() {
        let command = GeminiBrowserSidecarEnvelope {
            id: "cmd-1".to_string(),
            command: GeminiBrowserSidecarCommand::OpenBrowser {
                browser_profile_dir: "C:/Extractum/gemini-browser/profile".to_string(),
                browser_config: None,
            },
        };

        let json = serde_json::to_value(command).expect("serialize command");
        assert_eq!(json["command"]["type"], "open_browser");
        assert_eq!(
            json["command"]["browser_profile_dir"],
            "C:/Extractum/gemini-browser/profile"
        );
    }

    #[test]
    fn manual_action_serializes_start_chrome_cdp() {
        let value = serde_json::to_value(GeminiBrowserManualAction::StartChromeCdp)
            .expect("serialize manual action");

        assert_eq!(value, "start_chrome_cdp");
    }

    #[test]
    fn resume_command_serializes_browser_profile_dir() {
        let command = GeminiBrowserSidecarEnvelope {
            id: "cmd-resume".to_string(),
            command: GeminiBrowserSidecarCommand::Resume {
                run_id: None,
                browser_profile_dir: "C:/Extractum/gemini-browser/profile".to_string(),
                browser_config: None,
            },
        };

        let json = serde_json::to_value(command).expect("serialize command");
        assert_eq!(json["command"]["type"], "resume");
        assert_eq!(json["command"]["run_id"], serde_json::Value::Null);
        assert!(json["command"].get("browser_profile_dir").is_some());
        assert_eq!(
            json["command"]["browser_profile_dir"],
            "C:/Extractum/gemini-browser/profile"
        );
    }

    #[test]
    fn sidecar_command_serializes_browser_config() {
        let command = GeminiBrowserSidecarEnvelope {
            id: "cmd-status".to_string(),
            command: GeminiBrowserSidecarCommand::Status {
                browser_profile_dir: "C:/Extractum/gemini-browser/profile".to_string(),
                browser_config: Some(GeminiBrowserProviderConfig {
                    mode: GeminiBrowserProviderMode::CdpAttach,
                    cdp_endpoint: Some("http://127.0.0.1:9222".to_string()),
                }),
            },
        };

        let json = serde_json::to_value(command).expect("serialize command");
        assert_eq!(json["command"]["type"], "status");
        assert_eq!(json["command"]["browser_config"]["mode"], "cdp_attach");
        assert_eq!(
            json["command"]["browser_config"]["cdp_endpoint"],
            "http://127.0.0.1:9222"
        );
    }

    #[test]
    fn run_result_serializes_optional_debug_summary() {
        let result = GeminiBrowserRunResult {
            run_id: "run-1".to_string(),
            status: GeminiBrowserRunStatus::Ok,
            text: Some("answer".to_string()),
            message: None,
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs {
                run_dir: None,
                html: None,
                screenshot: None,
                telemetry: None,
                answer_extraction: Some("answer-extraction.json".to_string()),
                artifact_write_error: None,
            },
            elapsed_ms: 42,
            debug_summary: Some(GeminiBrowserRunDebugSummary {
                mode: GeminiBrowserProviderMode::CdpAttach,
                composer_found: true,
                send_button_found: true,
                generation_busy_observed: true,
                answer_found: true,
                answer_selector: Some("message-content".to_string()),
                waited_for_send_ms: 15_000,
                waited_for_answer_ms: 12_000,
                answer_stable_ms: 8_000,
                answer_completion_reason: GeminiBrowserAnswerCompletionReason::Stable,
                final_text_length: 6,
                error_stage: None,
                extraction: Some(GeminiBrowserAnswerExtractionDebug {
                    raw_candidate_count: 2,
                    grouped_candidate_count: 1,
                    selected_candidate_length: 6,
                    returned_text_length: 6,
                    selected_grouping: GeminiBrowserAnswerGrouping::AssistantTurn,
                    selected_candidate_rank: Some(1),
                    selected_score: Some(120),
                    largest_candidate_length: 6,
                    larger_valid_candidate_available: false,
                    larger_rejected_candidate_count: 1,
                    larger_rejected_reasons: vec![GeminiBrowserCandidateRejectReason::Composer],
                    top_candidate_lengths: vec![6],
                    busy_visible_at_completion: false,
                    last_growth_elapsed_ms: Some(8_000),
                    candidate_signature_changed_count: 1,
                    stable_poll_count_after_last_candidate_change: 3,
                }),
            }),
        };

        let json = serde_json::to_value(&result).expect("serialize result");
        assert_eq!(
            json["artifacts"]["answer_extraction"],
            "answer-extraction.json"
        );
        assert_eq!(json["debug_summary"]["mode"], "cdp_attach");
        assert_eq!(json["debug_summary"]["generation_busy_observed"], true);
        assert_eq!(
            json["debug_summary"]["extraction"]["selected_grouping"],
            "assistant_turn"
        );

        let decoded: GeminiBrowserRunResult =
            serde_json::from_value(json).expect("deserialize result");
        let debug_summary = decoded.debug_summary.expect("debug summary");
        assert_eq!(
            debug_summary.answer_selector,
            Some("message-content".to_string())
        );
        assert_eq!(
            debug_summary
                .extraction
                .expect("extraction")
                .larger_rejected_reasons,
            vec![GeminiBrowserCandidateRejectReason::Composer]
        );
    }
}
