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
    pub artifact_write_error: Option<String>,
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
pub struct GeminiBrowserRunEvent {
    pub run_id: String,
    pub status: GeminiBrowserRunStatus,
    pub message: Option<String>,
    pub queue_position: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GeminiBrowserSidecarCommand {
    Status {
        browser_profile_dir: String,
    },
    OpenBrowser {
        browser_profile_dir: String,
    },
    SendSingle {
        request: GeminiBrowserRunRequest,
        browser_profile_dir: String,
        artifact_dir: String,
    },
    Resume {
        run_id: Option<String>,
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
    }

    #[test]
    fn sidecar_command_serializes_with_snake_case_tag() {
        let command = GeminiBrowserSidecarEnvelope {
            id: "cmd-1".to_string(),
            command: GeminiBrowserSidecarCommand::OpenBrowser {
                browser_profile_dir: "C:/Extractum/gemini-browser/profile".to_string(),
            },
        };

        let json = serde_json::to_value(command).expect("serialize command");
        assert_eq!(json["command"]["type"], "open_browser");
        assert_eq!(
            json["command"]["browser_profile_dir"],
            "C:/Extractum/gemini-browser/profile"
        );
    }
}
