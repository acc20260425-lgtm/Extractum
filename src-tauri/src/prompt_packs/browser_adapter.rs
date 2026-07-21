use std::sync::Arc;

use extractum_gemini_browser::{
    GeminiBrowserProviderConfig, GeminiBrowserProviderStatus, GeminiBrowserRunResult,
};
use tauri::{AppHandle, Manager};

use extractum_prompt_packs::{
    PromptPackBrowserCancelRequest, PromptPackBrowserExecutor, PromptPackBrowserFuture,
    PromptPackBrowserRunRequest, PromptPackBrowserStatusRequest,
};

trait BrowserAdapterBackend: Send + Sync + 'static {
    fn provider_status(
        &self,
        provider_config: Option<GeminiBrowserProviderConfig>,
    ) -> PromptPackBrowserFuture<'_, GeminiBrowserProviderStatus>;

    #[allow(clippy::too_many_arguments)]
    fn send_single_prompt(
        &self,
        run_id: String,
        prompt: String,
        source: Option<String>,
        artifact_mode: Option<String>,
        provider_config: Option<GeminiBrowserProviderConfig>,
    ) -> PromptPackBrowserFuture<'_, GeminiBrowserRunResult>;

    fn cancel(&self, run_id: String) -> PromptPackBrowserFuture<'_, ()>;
}

#[derive(Clone)]
struct TauriBrowserAdapterBackend {
    handle: AppHandle,
}

impl BrowserAdapterBackend for TauriBrowserAdapterBackend {
    fn provider_status(
        &self,
        provider_config: Option<GeminiBrowserProviderConfig>,
    ) -> PromptPackBrowserFuture<'_, GeminiBrowserProviderStatus> {
        let handle = self.handle.clone();
        Box::pin(async move {
            let state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
            crate::gemini_browser::provider_status(&handle, state.inner(), provider_config).await
        })
    }

    fn send_single_prompt(
        &self,
        run_id: String,
        prompt: String,
        source: Option<String>,
        artifact_mode: Option<String>,
        provider_config: Option<GeminiBrowserProviderConfig>,
    ) -> PromptPackBrowserFuture<'_, GeminiBrowserRunResult> {
        let handle = self.handle.clone();
        Box::pin(async move {
            let state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
            crate::gemini_browser::send_single_prompt(
                &handle,
                state.inner(),
                run_id,
                prompt,
                source,
                artifact_mode,
                provider_config,
            )
            .await
        })
    }

    fn cancel(&self, run_id: String) -> PromptPackBrowserFuture<'_, ()> {
        let handle = self.handle.clone();
        Box::pin(
            async move { crate::gemini_browser::cancel_gemini_browser_job(&handle, &run_id).await },
        )
    }
}

#[derive(Clone)]
pub(crate) struct TauriGeminiBrowserPort {
    handle: AppHandle,
}

impl TauriGeminiBrowserPort {
    pub(crate) fn new(handle: AppHandle) -> Self {
        Self { handle }
    }
}

fn delegate_status(
    backend: Arc<dyn BrowserAdapterBackend>,
    request: PromptPackBrowserStatusRequest,
) -> PromptPackBrowserFuture<'static, GeminiBrowserProviderStatus> {
    Box::pin(async move {
        backend
            .provider_status(request.provider_config().cloned())
            .await
    })
}

fn delegate_submit(
    backend: Arc<dyn BrowserAdapterBackend>,
    request: PromptPackBrowserRunRequest,
) -> PromptPackBrowserFuture<'static, GeminiBrowserRunResult> {
    Box::pin(async move {
        backend
            .send_single_prompt(
                request.run_id().to_string(),
                request.prompt().to_string(),
                Some(request.source().to_string()),
                Some(request.artifact_mode().to_string()),
                request.provider_config().cloned(),
            )
            .await
    })
}

fn delegate_cancel(
    backend: Arc<dyn BrowserAdapterBackend>,
    request: PromptPackBrowserCancelRequest,
) -> PromptPackBrowserFuture<'static, ()> {
    Box::pin(async move { backend.cancel(request.run_id().to_string()).await })
}

impl PromptPackBrowserExecutor for TauriGeminiBrowserPort {
    fn read_status(
        &self,
        request: PromptPackBrowserStatusRequest,
    ) -> PromptPackBrowserFuture<'_, GeminiBrowserProviderStatus> {
        delegate_status(
            Arc::new(TauriBrowserAdapterBackend {
                handle: self.handle.clone(),
            }),
            request,
        )
    }

    fn submit(
        &self,
        request: PromptPackBrowserRunRequest,
    ) -> PromptPackBrowserFuture<'_, GeminiBrowserRunResult> {
        delegate_submit(
            Arc::new(TauriBrowserAdapterBackend {
                handle: self.handle.clone(),
            }),
            request,
        )
    }

    fn cancel(&self, request: PromptPackBrowserCancelRequest) -> PromptPackBrowserFuture<'_, ()> {
        delegate_cancel(
            Arc::new(TauriBrowserAdapterBackend {
                handle: self.handle.clone(),
            }),
            request,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use extractum_gemini_browser::{
        GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs,
        GeminiBrowserDebugErrorStage, GeminiBrowserManualAction, GeminiBrowserProviderConfig,
        GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
        GeminiBrowserRunDebugSummary, GeminiBrowserRunResult, GeminiBrowserRunStatus,
    };

    use super::{delegate_cancel, delegate_status, delegate_submit, BrowserAdapterBackend};
    use extractum_prompt_packs::{
        PromptPackBrowserCancelRequest, PromptPackBrowserFuture, PromptPackBrowserRunRequest,
        PromptPackBrowserStatusRequest,
    };

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum BrowserCall {
        Status(Option<GeminiBrowserProviderConfig>),
        Submit {
            run_id: String,
            prompt: String,
            source: Option<String>,
            artifact_mode: Option<String>,
            provider_config: Option<GeminiBrowserProviderConfig>,
        },
        Cancel(String),
    }

    struct RecordingBrowserBackend {
        calls: Mutex<Vec<BrowserCall>>,
        status: GeminiBrowserProviderStatus,
        result: GeminiBrowserRunResult,
    }

    impl BrowserAdapterBackend for RecordingBrowserBackend {
        fn provider_status(
            &self,
            provider_config: Option<GeminiBrowserProviderConfig>,
        ) -> PromptPackBrowserFuture<'_, GeminiBrowserProviderStatus> {
            self.calls
                .lock()
                .expect("calls")
                .push(BrowserCall::Status(provider_config));
            let status = self.status.clone();
            Box::pin(async move { Ok(status) })
        }

        fn send_single_prompt(
            &self,
            run_id: String,
            prompt: String,
            source: Option<String>,
            artifact_mode: Option<String>,
            provider_config: Option<GeminiBrowserProviderConfig>,
        ) -> PromptPackBrowserFuture<'_, GeminiBrowserRunResult> {
            self.calls.lock().expect("calls").push(BrowserCall::Submit {
                run_id,
                prompt,
                source,
                artifact_mode,
                provider_config,
            });
            let result = self.result.clone();
            Box::pin(async move { Ok(result) })
        }

        fn cancel(&self, run_id: String) -> PromptPackBrowserFuture<'_, ()> {
            self.calls
                .lock()
                .expect("calls")
                .push(BrowserCall::Cancel(run_id));
            Box::pin(async { Ok(()) })
        }
    }

    #[tokio::test]
    async fn browser_port_delegates_readiness_submission_and_cancellation_without_narrowing_result()
    {
        let provider_config = GeminiBrowserProviderConfig {
            mode: GeminiBrowserProviderMode::CdpAttach,
            cdp_endpoint: Some("http://127.0.0.1:9222".to_string()),
        };
        let status = GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::NeedsManualAction,
            manual_action: Some(GeminiBrowserManualAction::Captcha),
            active_run_id: Some("active-browser-run".to_string()),
            queue_depth: 3,
            browser_profile_dir: "browser-profile".to_string(),
            latest_message: Some("Solve captcha".to_string()),
        };
        let result = GeminiBrowserRunResult {
            run_id: "prompt-pack-42-stage-7".to_string(),
            status: GeminiBrowserRunStatus::Ok,
            text: Some("complete Browser answer".to_string()),
            message: Some("Browser run completed".to_string()),
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs {
                run_dir: Some("runs/prompt-pack-42-stage-7".to_string()),
                html: Some("answer.html".to_string()),
                screenshot: Some("answer.png".to_string()),
                telemetry: Some("telemetry.json".to_string()),
                answer_extraction: Some("answer-extraction.json".to_string()),
                artifact_write_error: None,
            },
            elapsed_ms: 1_234,
            debug_summary: Some(GeminiBrowserRunDebugSummary {
                mode: GeminiBrowserProviderMode::CdpAttach,
                composer_found: true,
                send_button_found: true,
                generation_busy_observed: true,
                answer_found: true,
                answer_selector: Some("[data-answer]".to_string()),
                waited_for_send_ms: 12,
                waited_for_answer_ms: 345,
                answer_stable_ms: 678,
                answer_completion_reason: GeminiBrowserAnswerCompletionReason::Stable,
                final_text_length: 23,
                error_stage: Some(GeminiBrowserDebugErrorStage::Artifacts),
                extraction: None,
            }),
        };
        let backend = Arc::new(RecordingBrowserBackend {
            calls: Mutex::new(Vec::new()),
            status: status.clone(),
            result: result.clone(),
        });
        let actual_status = delegate_status(
            backend.clone(),
            PromptPackBrowserStatusRequest::new(Some(provider_config.clone())),
        )
        .await
        .expect("read status");
        let actual_result = delegate_submit(
            backend.clone(),
            PromptPackBrowserRunRequest::new(
                result.run_id.clone(),
                "full prompt".to_string(),
                "prompt_pack:test".to_string(),
                "reduced".to_string(),
                Some(provider_config.clone()),
            ),
        )
        .await
        .expect("submit");
        delegate_cancel(
            backend.clone(),
            PromptPackBrowserCancelRequest::new(result.run_id.clone()),
        )
        .await
        .expect("cancel");

        assert_eq!(actual_status, status);
        assert_eq!(actual_result, result);
        assert_eq!(
            *backend.calls.lock().expect("calls"),
            vec![
                BrowserCall::Status(Some(provider_config.clone())),
                BrowserCall::Submit {
                    run_id: "prompt-pack-42-stage-7".to_string(),
                    prompt: "full prompt".to_string(),
                    source: Some("prompt_pack:test".to_string()),
                    artifact_mode: Some("reduced".to_string()),
                    provider_config: Some(provider_config),
                },
                BrowserCall::Cancel("prompt-pack-42-stage-7".to_string()),
            ]
        );
    }
}
