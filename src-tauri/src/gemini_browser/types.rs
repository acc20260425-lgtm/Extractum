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
}
