use crate::gemini_browser::GeminiBrowserProviderConfig;

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PromptPackRuntimeProvider {
    Api,
    GeminiBrowser,
}

impl Default for PromptPackRuntimeProvider {
    fn default() -> Self {
        Self::Api
    }
}

impl PromptPackRuntimeProvider {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::GeminiBrowser => "gemini_browser",
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct PreflightYoutubeSummaryRunRequest {
    pub(crate) project_id: Option<i64>,
    pub(crate) source_ids: Vec<i64>,
    pub(crate) profile_id: Option<String>,
    pub(crate) model_override: Option<String>,
    #[serde(default)]
    pub(crate) runtime_provider: PromptPackRuntimeProvider,
    #[serde(default)]
    pub(crate) browser_provider_config: Option<GeminiBrowserProviderConfig>,
    pub(crate) output_language: String,
    pub(crate) control_preset: String,
    pub(crate) evidence_mode: String,
    pub(crate) include_comments: bool,
}

impl PreflightYoutubeSummaryRunRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: Option<i64>,
        source_ids: Vec<i64>,
        profile_id: Option<String>,
        model_override: Option<String>,
        runtime_provider: PromptPackRuntimeProvider,
        browser_provider_config: Option<GeminiBrowserProviderConfig>,
        output_language: String,
        control_preset: String,
        evidence_mode: String,
        include_comments: bool,
    ) -> Self {
        Self {
            project_id,
            source_ids,
            profile_id,
            model_override,
            runtime_provider,
            browser_provider_config,
            output_language,
            control_preset,
            evidence_mode,
            include_comments,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartYoutubeSummaryRunRequest {
    pub(crate) client_request_id: String,
    pub(crate) project_id: Option<i64>,
    pub(crate) source_ids: Vec<i64>,
    pub(crate) profile_id: Option<String>,
    pub(crate) model_override: Option<String>,
    #[serde(default)]
    pub(crate) runtime_provider: PromptPackRuntimeProvider,
    #[serde(default)]
    pub(crate) browser_provider_config: Option<GeminiBrowserProviderConfig>,
    pub(crate) output_language: String,
    pub(crate) control_preset: String,
    pub(crate) evidence_mode: String,
    pub(crate) include_comments: bool,
}

impl StartYoutubeSummaryRunRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client_request_id: String,
        project_id: Option<i64>,
        source_ids: Vec<i64>,
        profile_id: Option<String>,
        model_override: Option<String>,
        runtime_provider: PromptPackRuntimeProvider,
        browser_provider_config: Option<GeminiBrowserProviderConfig>,
        output_language: String,
        control_preset: String,
        evidence_mode: String,
        include_comments: bool,
    ) -> Self {
        Self {
            client_request_id,
            project_id,
            source_ids,
            profile_id,
            model_override,
            runtime_provider,
            browser_provider_config,
            output_language,
            control_preset,
            evidence_mode,
            include_comments,
        }
    }

    pub fn client_request_id(&self) -> &str {
        &self.client_request_id
    }

    pub fn runtime_provider(&self) -> PromptPackRuntimeProvider {
        self.runtime_provider
    }

    pub fn profile_id(&self) -> Option<&str> {
        self.profile_id.as_deref()
    }

    pub fn model_override(&self) -> Option<&str> {
        self.model_override.as_deref()
    }
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSummaryPreflightResponse {
    pub(crate) pack_id: String,
    pub(crate) pack_version: String,
    pub(crate) included_videos: Vec<YoutubeSummaryPreflightVideo>,
    pub(crate) skipped_videos: Vec<YoutubeSummaryPreflightSkippedVideo>,
    pub(crate) blocking_failures: Vec<YoutubeSummaryPreflightFailure>,
    pub(crate) estimated_input_tokens: i64,
    pub(crate) selected_model_input_limit: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSummaryPreflightVideo {
    pub(crate) source_id: i64,
    pub(crate) video_id: String,
    pub(crate) title: String,
    pub(crate) estimated_input_tokens: i64,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSummaryPreflightSkippedVideo {
    pub(crate) source_id: Option<i64>,
    pub(crate) video_id: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSummaryPreflightFailure {
    pub(crate) source_id: Option<i64>,
    pub(crate) reason: String,
    pub(crate) message: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPromptPackRunsRequest {
    pub(crate) project_id: Option<i64>,
    pub(crate) limit: Option<i64>,
}

impl ListPromptPackRunsRequest {
    pub fn new(project_id: Option<i64>, limit: Option<i64>) -> Self {
        Self { project_id, limit }
    }
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackRunSummaryDto {
    pub(crate) run_id: i64,
    pub(crate) project_id: Option<i64>,
    pub(crate) run_label: Option<String>,
    pub(crate) runtime_provider: String,
    pub(crate) pack_id: String,
    pub(crate) pack_version: String,
    pub(crate) run_status: String,
    pub(crate) result_status: String,
    pub(crate) created_at: String,
    pub(crate) started_at: Option<String>,
    pub(crate) completed_at: Option<String>,
    pub(crate) latest_message: Option<String>,
    pub(crate) progress_current: Option<i64>,
    pub(crate) progress_total: Option<i64>,
    pub(crate) queue_position: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackStageRunDto {
    pub(crate) stage_run_id: i64,
    pub(crate) run_id: i64,
    pub(crate) source_snapshot_id: Option<i64>,
    pub(crate) stage_name: String,
    pub(crate) stage_order: i64,
    pub(crate) stage_status: String,
    pub(crate) latest_message: Option<String>,
    pub(crate) browser_run_id: Option<String>,
    pub(crate) browser_run_status: Option<String>,
    pub(crate) browser_completion_reason: Option<String>,
    pub(crate) browser_provider_mode: Option<String>,
    pub(crate) browser_run_message: Option<String>,
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
    pub(crate) run_id: i64,
    pub(crate) result_status: String,
    pub(crate) canonical: serde_json::Value,
    pub(crate) storage_warning: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackStageArtifactSummaryDto {
    pub(crate) stage_run_id: i64,
    pub(crate) artifact_kind: String,
    pub(crate) attempt_number: i64,
    pub(crate) artifact_index: i64,
    pub(crate) content_type: String,
    pub(crate) content_hash: String,
    pub(crate) created_at: String,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackStageArtifactDto {
    pub(crate) stage_run_id: i64,
    pub(crate) artifact_kind: String,
    pub(crate) attempt_number: i64,
    pub(crate) artifact_index: i64,
    pub(crate) content_type: String,
    pub(crate) content: serde_json::Value,
    pub(crate) created_at: String,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackValidationFindingDto {
    pub(crate) run_id: i64,
    pub(crate) stage_run_id: Option<i64>,
    pub(crate) severity: String,
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) object_path: Option<String>,
    pub(crate) created_at: String,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPackAuditEventDto {
    pub(crate) run_id: i64,
    pub(crate) event_kind: String,
    pub(crate) message: Option<String>,
    pub(crate) payload: Option<serde_json::Value>,
    pub(crate) created_at: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt_packs::event_adapter::{PromptPackRunEvent, PROMPT_PACK_RUN_EVENT};

    #[test]
    fn crate_boundary_constructors_and_accessors_preserve_serialized_shapes() {
        let request = StartYoutubeSummaryRunRequest::new(
            "client-1".into(),
            Some(7),
            vec![11],
            Some("profile-1".into()),
            Some("model-1".into()),
            PromptPackRuntimeProvider::Api,
            None,
            "English".into(),
            "detailed_report".into(),
            "strict".into(),
            true,
        );
        assert_eq!(request.client_request_id(), "client-1");
        assert_eq!(request.runtime_provider(), PromptPackRuntimeProvider::Api);
        assert_eq!(request.profile_id(), Some("profile-1"));
        assert_eq!(request.model_override(), Some("model-1"));
    }

    fn serialized_run_summary() -> PromptPackRunSummaryDto {
        PromptPackRunSummaryDto {
            run_id: 42,
            project_id: Some(7),
            run_label: Some("Weekly review".to_string()),
            runtime_provider: "api".to_string(),
            pack_id: "youtube_summary".to_string(),
            pack_version: "1.0.0".to_string(),
            run_status: "queued".to_string(),
            result_status: "none".to_string(),
            created_at: "2026-07-20T12:00:00Z".to_string(),
            started_at: None,
            completed_at: None,
            latest_message: Some("Queued".to_string()),
            progress_current: Some(0),
            progress_total: Some(4),
            queue_position: Some(2),
        }
    }

    #[test]
    fn start_outcomes_serialize_exact_ipc_contract() {
        let started = serde_json::to_value(StartYoutubeSummaryRunOutcomeDto::Started {
            run: serialized_run_summary(),
        })
        .expect("serialize started outcome");
        assert_eq!(
            started,
            serde_json::json!({
                "kind": "started",
                "run": {
                    "runId": 42,
                    "projectId": 7,
                    "runLabel": "Weekly review",
                    "runtimeProvider": "api",
                    "packId": "youtube_summary",
                    "packVersion": "1.0.0",
                    "runStatus": "queued",
                    "resultStatus": "none",
                    "createdAt": "2026-07-20T12:00:00Z",
                    "startedAt": null,
                    "completedAt": null,
                    "latestMessage": "Queued",
                    "progressCurrent": 0,
                    "progressTotal": 4,
                    "queuePosition": 2
                }
            })
        );

        let blocked = serde_json::to_value(StartYoutubeSummaryRunOutcomeDto::Blocked {
            preflight: YoutubeSummaryPreflightResponse {
                pack_id: "youtube_summary".to_string(),
                pack_version: "1.0.0".to_string(),
                included_videos: vec![YoutubeSummaryPreflightVideo {
                    source_id: 901,
                    video_id: "video-901".to_string(),
                    title: "Ready video".to_string(),
                    estimated_input_tokens: 1_250,
                }],
                skipped_videos: vec![YoutubeSummaryPreflightSkippedVideo {
                    source_id: None,
                    video_id: Some("video-missing".to_string()),
                    title: None,
                    reason: "unlinked_playlist_item".to_string(),
                }],
                blocking_failures: vec![YoutubeSummaryPreflightFailure {
                    source_id: Some(902),
                    reason: "no_usable_transcript".to_string(),
                    message: Some(
                        "The selected YouTube video has no usable transcript".to_string(),
                    ),
                }],
                estimated_input_tokens: 1_250,
                selected_model_input_limit: Some(32_000),
            },
        })
        .expect("serialize blocked outcome");
        assert_eq!(
            blocked,
            serde_json::json!({
                "kind": "blocked",
                "preflight": {
                    "packId": "youtube_summary",
                    "packVersion": "1.0.0",
                    "includedVideos": [{
                        "sourceId": 901,
                        "videoId": "video-901",
                        "title": "Ready video",
                        "estimatedInputTokens": 1250
                    }],
                    "skippedVideos": [{
                        "sourceId": null,
                        "videoId": "video-missing",
                        "title": null,
                        "reason": "unlinked_playlist_item"
                    }],
                    "blockingFailures": [{
                        "sourceId": 902,
                        "reason": "no_usable_transcript",
                        "message": "The selected YouTube video has no usable transcript"
                    }],
                    "estimatedInputTokens": 1250,
                    "selectedModelInputLimit": 32000
                }
            })
        );
    }

    #[test]
    fn prompt_pack_run_events_serialize_exact_ipc_contract() {
        let events = vec![
            PromptPackRunEvent {
                run_id: 42,
                request_id: "run-42".to_string(),
                kind: "queued".to_string(),
                run_status: "queued".to_string(),
                phase: "snapshot".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: Some(2),
                progress_current: Some(0),
                progress_total: Some(4),
                message: Some("Queued".to_string()),
                error: None,
            },
            PromptPackRunEvent {
                run_id: 42,
                request_id: "run-42-started".to_string(),
                kind: "started".to_string(),
                run_status: "running".to_string(),
                phase: "execution".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: None,
                progress_current: Some(0),
                progress_total: Some(4),
                message: Some("Running".to_string()),
                error: None,
            },
            PromptPackRunEvent {
                run_id: 42,
                request_id: "run-42-stage-44-repair".to_string(),
                kind: "queued".to_string(),
                run_status: "running".to_string(),
                phase: "repair".to_string(),
                stage_run_id: Some(44),
                stage_name: Some("youtube_summary/transcript_analysis".to_string()),
                source_snapshot_id: Some(901),
                queue_position: Some(3),
                progress_current: None,
                progress_total: None,
                message: Some("JSON repair queued at position 3".to_string()),
                error: None,
            },
            PromptPackRunEvent {
                run_id: 42,
                request_id: "run-42-gem-comments".to_string(),
                kind: "started".to_string(),
                run_status: "running".to_string(),
                phase: "gem_analysis".to_string(),
                stage_run_id: Some(45),
                stage_name: Some("youtube_summary/gem_analysis/comments".to_string()),
                source_snapshot_id: Some(901),
                queue_position: None,
                progress_current: None,
                progress_total: None,
                message: Some("Gem analysis: analyzing comments".to_string()),
                error: None,
            },
            PromptPackRunEvent {
                run_id: 42,
                request_id: "run-42-terminal".to_string(),
                kind: "completed".to_string(),
                run_status: "complete".to_string(),
                phase: "terminal".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: None,
                progress_current: Some(4),
                progress_total: Some(4),
                message: Some("Completed".to_string()),
                error: None,
            },
            PromptPackRunEvent {
                run_id: 43,
                request_id: "run-43-terminal".to_string(),
                kind: "failed".to_string(),
                run_status: "failed".to_string(),
                phase: "terminal".to_string(),
                stage_run_id: Some(46),
                stage_name: Some("youtube_summary/synthesis".to_string()),
                source_snapshot_id: None,
                queue_position: None,
                progress_current: Some(3),
                progress_total: Some(4),
                message: Some("Provider request failed".to_string()),
                error: Some("Provider request failed".to_string()),
            },
            PromptPackRunEvent {
                run_id: 44,
                request_id: "cancel-44".to_string(),
                kind: "cancelled".to_string(),
                run_status: "cancelled".to_string(),
                phase: "terminal".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: None,
                progress_current: None,
                progress_total: None,
                message: Some("Cancelled".to_string()),
                error: None,
            },
        ];

        assert_eq!(PROMPT_PACK_RUN_EVENT, "prompt-pack-run-event");
        assert_eq!(
            serde_json::to_value(events).expect("serialize events"),
            serde_json::json!([
                {"runId":42,"requestId":"run-42","kind":"queued","runStatus":"queued","phase":"snapshot","stageRunId":null,"stageName":null,"sourceSnapshotId":null,"queuePosition":2,"progressCurrent":0,"progressTotal":4,"message":"Queued","error":null},
                {"runId":42,"requestId":"run-42-started","kind":"started","runStatus":"running","phase":"execution","stageRunId":null,"stageName":null,"sourceSnapshotId":null,"queuePosition":null,"progressCurrent":0,"progressTotal":4,"message":"Running","error":null},
                {"runId":42,"requestId":"run-42-stage-44-repair","kind":"queued","runStatus":"running","phase":"repair","stageRunId":44,"stageName":"youtube_summary/transcript_analysis","sourceSnapshotId":901,"queuePosition":3,"progressCurrent":null,"progressTotal":null,"message":"JSON repair queued at position 3","error":null},
                {"runId":42,"requestId":"run-42-gem-comments","kind":"started","runStatus":"running","phase":"gem_analysis","stageRunId":45,"stageName":"youtube_summary/gem_analysis/comments","sourceSnapshotId":901,"queuePosition":null,"progressCurrent":null,"progressTotal":null,"message":"Gem analysis: analyzing comments","error":null},
                {"runId":42,"requestId":"run-42-terminal","kind":"completed","runStatus":"complete","phase":"terminal","stageRunId":null,"stageName":null,"sourceSnapshotId":null,"queuePosition":null,"progressCurrent":4,"progressTotal":4,"message":"Completed","error":null},
                {"runId":43,"requestId":"run-43-terminal","kind":"failed","runStatus":"failed","phase":"terminal","stageRunId":46,"stageName":"youtube_summary/synthesis","sourceSnapshotId":null,"queuePosition":null,"progressCurrent":3,"progressTotal":4,"message":"Provider request failed","error":"Provider request failed"},
                {"runId":44,"requestId":"cancel-44","kind":"cancelled","runStatus":"cancelled","phase":"terminal","stageRunId":null,"stageName":null,"sourceSnapshotId":null,"queuePosition":null,"progressCurrent":null,"progressTotal":null,"message":"Cancelled","error":null}
            ])
        );
    }

    #[test]
    fn prompt_pack_errors_serialize_exact_json_contract() {
        let errors = [
            crate::error::AppError::validation("client_request_id cannot be empty"),
            crate::error::AppError::not_found("Prompt Pack run 404 not found"),
            crate::error::AppError::conflict("Active Prompt Pack runs cannot be deleted"),
            crate::error::AppError::internal("Database error: connection closed"),
        ];

        assert_eq!(
            serde_json::to_value(errors).expect("serialize errors"),
            serde_json::json!([
                {"kind":"validation","message":"client_request_id cannot be empty"},
                {"kind":"not_found","message":"Prompt Pack run 404 not found"},
                {"kind":"conflict","message":"Active Prompt Pack runs cannot be deleted"},
                {"kind":"internal","message":"Database error: connection closed"}
            ])
        );
    }

    #[test]
    fn preflight_request_defaults_to_api_runtime_provider() {
        let request: PreflightYoutubeSummaryRunRequest =
            serde_json::from_value(serde_json::json!({
                "projectId": null,
                "sourceIds": [901],
                "profileId": null,
                "modelOverride": null,
                "outputLanguage": "en",
                "controlPreset": "standard",
                "evidenceMode": "standard",
                "includeComments": false
            }))
            .expect("deserialize preflight request");

        assert_eq!(request.runtime_provider, PromptPackRuntimeProvider::Api);
        assert!(request.browser_provider_config.is_none());
    }

    #[test]
    fn start_request_accepts_gemini_browser_runtime_provider() {
        let request: StartYoutubeSummaryRunRequest = serde_json::from_value(serde_json::json!({
            "clientRequestId": "req-browser-runtime-1",
            "projectId": null,
            "sourceIds": [901],
            "profileId": null,
            "modelOverride": null,
            "outputLanguage": "en",
            "controlPreset": "standard",
            "evidenceMode": "standard",
            "includeComments": false,
            "runtimeProvider": "gemini_browser",
            "browserProviderConfig": {
                "mode": "cdp_attach",
                "cdpEndpoint": "http://127.0.0.1:9222"
            }
        }))
        .expect("deserialize start request");

        assert_eq!(
            request.runtime_provider,
            PromptPackRuntimeProvider::GeminiBrowser
        );
        let config = request.browser_provider_config.expect("browser config");
        assert_eq!(
            config.mode,
            crate::gemini_browser::GeminiBrowserProviderMode::CdpAttach
        );
        assert_eq!(
            config.cdp_endpoint.as_deref(),
            Some("http://127.0.0.1:9222")
        );
    }
}
