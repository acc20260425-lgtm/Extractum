use std::future::Future;
use std::time::Instant;

use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;

use super::dto::{PromptPackRunEvent, PROMPT_PACK_RUN_EVENT};
use super::run_control::run_with_prompt_pack_run_cancellation;
use super::youtube_summary::{
    LlmCompletion as PromptPackLlmCompletion, YoutubeSummaryStageExecutionError,
};
use crate::error::{AppError, AppResult};
use crate::llm::{
    resolve_effective_model, resolve_model_output_token_limit_for_backend,
    run_llm_collect_with_profile, LlmChatRequest, LlmRequestError, LlmRequestKind,
    LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState, ResolvedLlmProfile,
};

#[derive(Clone)]
pub(super) enum RunCompletionRuntime {
    Api {
        profile: ResolvedLlmProfile,
        model_override: Option<String>,
    },
    GeminiBrowser {
        browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
    },
}

pub(super) struct CompletionModelContext {
    pub(super) profile_id: Option<String>,
    pub(super) model_override: Option<String>,
    pub(super) model_output_limit: Option<i64>,
}

pub(super) struct StageCompletionRequest {
    pub(super) llm_request: LlmChatRequest,
    pub(super) run_id: i64,
    pub(super) stage_run_id: i64,
    pub(super) source_snapshot_id: Option<i64>,
    pub(super) stage_name: String,
    pub(super) phase: &'static str,
    pub(super) started_message: &'static str,
    pub(super) repair_attempt_number: Option<i64>,
    pub(super) request_discriminator: Option<String>,
    pub(super) run_cancellation_token: Option<CancellationToken>,
}

impl RunCompletionRuntime {
    pub(super) async fn model_context(&self) -> AppResult<CompletionModelContext> {
        match self {
            Self::Api {
                profile,
                model_override,
            } => {
                let effective_model = resolve_effective_model(profile, model_override.as_deref())?;
                let model_output_limit =
                    resolve_model_output_token_limit_for_backend(profile, &effective_model).await;
                Ok(CompletionModelContext {
                    profile_id: Some(profile.profile_id.clone()),
                    model_override: model_override.clone(),
                    model_output_limit,
                })
            }
            Self::GeminiBrowser { .. } => Ok(CompletionModelContext {
                profile_id: None,
                model_override: None,
                model_output_limit: None,
            }),
        }
    }

    pub(super) async fn execute(
        self,
        handle: AppHandle,
        pool: SqlitePool,
        request: StageCompletionRequest,
    ) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
        match self {
            Self::Api { profile, .. } => run_api_llm_request(handle, profile, request).await,
            Self::GeminiBrowser {
                browser_provider_config,
            } => run_browser_llm_request(handle, pool, browser_provider_config, request).await,
        }
    }
}

pub(super) fn llm_chat_request_to_browser_prompt(request: &LlmChatRequest) -> AppResult<String> {
    let mut sections = Vec::with_capacity(request.messages.len());
    for message in &request.messages {
        let label = match message.role.as_str() {
            "system" => "System",
            "user" => "User",
            other => {
                return Err(AppError::validation(format!(
                    "Unsupported Browser Provider prompt message role: {other}"
                )));
            }
        };
        sections.push(format!("{label}:\n{}", message.content));
    }
    let prompt = sections.join("\n\n");
    if prompt.trim().is_empty() {
        return Err(AppError::validation(
            "Browser Provider prompt cannot be empty",
        ));
    }
    Ok(prompt)
}

pub(super) fn browser_run_id_for_stage(
    run_id: i64,
    stage_run_id: i64,
    repair_attempt_number: Option<i64>,
    request_discriminator: Option<&str>,
) -> String {
    match (request_discriminator, repair_attempt_number) {
        (Some(discriminator), _) => {
            format!("prompt-pack-{run_id}-stage-{stage_run_id}-{discriminator}")
        }
        (None, Some(attempt_number)) => {
            format!("prompt-pack-{run_id}-stage-{stage_run_id}-repair-{attempt_number}")
        }
        (None, None) => format!("prompt-pack-{run_id}-stage-{stage_run_id}"),
    }
}

pub(super) fn browser_run_source_for_stage(
    run_id: i64,
    stage_run_id: i64,
    stage_name: &str,
    request_discriminator: Option<&str>,
) -> String {
    let base =
        format!("prompt_pack:youtube_summary:{stage_name}:run:{run_id}:stage:{stage_run_id}");
    match request_discriminator {
        Some(discriminator) => format!("{base}:{discriminator}"),
        None => base,
    }
}

pub(super) fn browser_stage_completion_from_result(
    result: crate::gemini_browser::GeminiBrowserRunResult,
) -> AppResult<PromptPackLlmCompletion> {
    let latency_ms = result.elapsed_ms as i64;
    let text = super::gemini_browser_stage::browser_result_to_completion_text(result)?;
    Ok(PromptPackLlmCompletion {
        text,
        input_tokens: None,
        output_tokens: None,
        latency_ms,
    })
}

async fn run_api_llm_request(
    handle: AppHandle,
    profile: ResolvedLlmProfile,
    request: StageCompletionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let StageCompletionRequest {
        llm_request,
        run_id,
        stage_run_id,
        source_snapshot_id,
        stage_name,
        phase,
        started_message,
        run_cancellation_token,
        ..
    } = request;
    let request_id = llm_request.request_id.clone();
    let provider = profile.provider.as_str().to_string();
    let scheduler = handle.state::<LlmSchedulerState>();
    let queued_handle = handle.clone();
    let started_handle = handle.clone();
    let queued_request_id = request_id.clone();
    let started_request_id = request_id.clone();
    let queued_stage_name = stage_name.clone();
    let started_stage_name = stage_name;
    let queued_phase = phase.to_string();
    let started_phase = queued_phase.clone();
    let scheduled_request = llm_request.clone();
    let scheduled_profile = profile.clone();
    let stage_cancellation_token = run_cancellation_token.clone();

    match scheduler
        .run_request(
            LlmRequestMetadata {
                request_id: request_id.clone(),
                profile_id: profile.profile_id.clone(),
                provider,
                kind: LlmRequestKind::PromptPackStage,
                priority: LlmRequestPriority::Background,
                owner_run_id: Some(run_id),
            },
            move |position| {
                let queued_message = if phase == "repair" {
                    format!("JSON repair queued at position {position}")
                } else {
                    format!("LLM request queued at position {position}")
                };
                let _ = queued_handle.emit(
                    PROMPT_PACK_RUN_EVENT,
                    PromptPackRunEvent {
                        run_id,
                        request_id: queued_request_id.clone(),
                        kind: "queued".to_string(),
                        run_status: "running".to_string(),
                        phase: queued_phase.clone(),
                        stage_run_id: Some(stage_run_id),
                        stage_name: Some(queued_stage_name.clone()),
                        source_snapshot_id,
                        queue_position: Some(position as i64),
                        progress_current: None,
                        progress_total: None,
                        message: Some(queued_message),
                        error: None,
                    },
                );
            },
            move |control| async move {
                let _ = started_handle.emit(
                    PROMPT_PACK_RUN_EVENT,
                    PromptPackRunEvent {
                        run_id,
                        request_id: started_request_id,
                        kind: "started".to_string(),
                        run_status: "running".to_string(),
                        phase: started_phase,
                        stage_run_id: Some(stage_run_id),
                        stage_name: Some(started_stage_name),
                        source_snapshot_id,
                        queue_position: None,
                        progress_current: None,
                        progress_total: None,
                        message: Some(started_message.to_string()),
                        error: None,
                    },
                );
                let started_at = Instant::now();
                let completion = run_with_prompt_pack_run_cancellation(
                    stage_cancellation_token,
                    control.run_cancellable(run_llm_collect_with_profile(
                        &scheduled_request,
                        &scheduled_profile,
                    )),
                )
                .await?;
                Ok((completion, started_at.elapsed().as_millis() as i64))
            },
        )
        .await
    {
        Ok((completion, latency_ms)) => Ok(PromptPackLlmCompletion {
            text: completion.text,
            input_tokens: completion
                .usage
                .as_ref()
                .and_then(|usage| usage.input_tokens),
            output_tokens: completion
                .usage
                .as_ref()
                .and_then(|usage| usage.output_tokens),
            latency_ms,
        }),
        Err(LlmRequestError::Cancelled) => Err(YoutubeSummaryStageExecutionError::Cancelled),
        Err(LlmRequestError::Failed(error)) => {
            Err(YoutubeSummaryStageExecutionError::Failed(error))
        }
    }
}

async fn run_browser_llm_request(
    handle: AppHandle,
    pool: SqlitePool,
    browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
    request: StageCompletionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let StageCompletionRequest {
        llm_request,
        run_id,
        stage_run_id,
        source_snapshot_id,
        stage_name,
        phase,
        started_message,
        repair_attempt_number,
        request_discriminator,
        run_cancellation_token,
    } = request;
    let request_discriminator = request_discriminator.as_deref();
    let browser_run_id = browser_run_id_for_stage(
        run_id,
        stage_run_id,
        repair_attempt_number,
        request_discriminator,
    );
    if run_cancellation_token
        .as_ref()
        .is_some_and(CancellationToken::is_cancelled)
    {
        crate::gemini_browser::cancel_gemini_browser_job(&handle, &browser_run_id)
            .await
            .map_err(YoutubeSummaryStageExecutionError::Failed)?;
        return Err(YoutubeSummaryStageExecutionError::Cancelled);
    }

    let prompt = llm_chat_request_to_browser_prompt(&llm_request)?;
    let source =
        browser_run_source_for_stage(run_id, stage_run_id, &stage_name, request_discriminator);
    let queued_handle = handle.clone();
    let started_handle = handle.clone();
    let request_id = llm_request.request_id.clone();
    let started_request_id = request_id.clone();
    let queued_stage_name = stage_name.clone();
    let started_stage_name = stage_name;
    let queued_phase = phase.to_string();
    let started_phase = queued_phase.clone();
    let run_cancellation_for_stop = run_cancellation_token.clone();
    let browser_state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
    let browser_run_id_for_cancel = browser_run_id.clone();

    let _ = queued_handle.emit(
        PROMPT_PACK_RUN_EVENT,
        PromptPackRunEvent {
            run_id,
            request_id: request_id.clone(),
            kind: "queued".to_string(),
            run_status: "running".to_string(),
            phase: queued_phase,
            stage_run_id: Some(stage_run_id),
            stage_name: Some(queued_stage_name),
            source_snapshot_id,
            queue_position: None,
            progress_current: None,
            progress_total: None,
            message: Some("Browser Provider request queued".to_string()),
            error: None,
        },
    );

    let browser_future = async {
        let _ = started_handle.emit(
            PROMPT_PACK_RUN_EVENT,
            PromptPackRunEvent {
                run_id,
                request_id: started_request_id,
                kind: "started".to_string(),
                run_status: "running".to_string(),
                phase: started_phase,
                stage_run_id: Some(stage_run_id),
                stage_name: Some(started_stage_name),
                source_snapshot_id,
                queue_position: None,
                progress_current: None,
                progress_total: None,
                message: Some(started_message.to_string()),
                error: None,
            },
        );
        crate::gemini_browser::send_single_prompt(
            &handle,
            &browser_state,
            browser_run_id,
            prompt,
            Some(source),
            Some("reduced".to_string()),
            browser_provider_config,
        )
        .await
        .map_err(LlmRequestError::Failed)
    };

    let cancel_handle = handle.clone();
    let result = run_browser_stage_result_with_cancellation(
        run_cancellation_token,
        browser_future,
        move || async move {
            crate::gemini_browser::cancel_gemini_browser_job(
                &cancel_handle,
                &browser_run_id_for_cancel,
            )
            .await
        },
    )
    .await?;

    if run_cancellation_for_stop
        .as_ref()
        .is_some_and(CancellationToken::is_cancelled)
    {
        return Err(YoutubeSummaryStageExecutionError::Cancelled);
    }

    persist_browser_stage_provenance(&pool, stage_run_id, &result)
        .await
        .map_err(YoutubeSummaryStageExecutionError::Failed)?;

    browser_stage_completion_from_result(result).map_err(YoutubeSummaryStageExecutionError::from)
}

pub(super) async fn run_browser_stage_result_with_cancellation<
    BrowserFuture,
    CancelBrowser,
    CancelFuture,
>(
    run_cancellation_token: Option<CancellationToken>,
    browser_future: BrowserFuture,
    cancel_browser_job: CancelBrowser,
) -> Result<crate::gemini_browser::GeminiBrowserRunResult, YoutubeSummaryStageExecutionError>
where
    BrowserFuture:
        Future<Output = Result<crate::gemini_browser::GeminiBrowserRunResult, LlmRequestError>>,
    CancelBrowser: FnOnce() -> CancelFuture,
    CancelFuture: Future<Output = AppResult<()>>,
{
    match run_with_prompt_pack_run_cancellation(run_cancellation_token, browser_future).await {
        Ok(result) => Ok(result),
        Err(LlmRequestError::Cancelled) => {
            cancel_browser_job()
                .await
                .map_err(YoutubeSummaryStageExecutionError::Failed)?;
            Err(YoutubeSummaryStageExecutionError::Cancelled)
        }
        Err(LlmRequestError::Failed(error)) => {
            Err(YoutubeSummaryStageExecutionError::Failed(error))
        }
    }
}

pub(super) async fn persist_browser_stage_provenance(
    pool: &SqlitePool,
    stage_run_id: i64,
    result: &crate::gemini_browser::GeminiBrowserRunResult,
) -> AppResult<()> {
    let completion_reason = result
        .debug_summary
        .as_ref()
        .and_then(|summary| serde_json::to_value(&summary.answer_completion_reason).ok())
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .and_then(non_empty_string);
    let provider_mode = result
        .debug_summary
        .as_ref()
        .and_then(|summary| serde_json::to_value(&summary.mode).ok())
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .and_then(non_empty_string);
    let run_status = serde_json::to_value(&result.status)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .and_then(non_empty_string);
    let run_message = result.message.clone().and_then(non_empty_string);

    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET browser_run_id = ?,
             browser_run_status = ?,
             browser_completion_reason = ?,
             browser_provider_mode = ?,
             browser_run_message = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(&result.run_id)
    .bind(run_status)
    .bind(completion_reason)
    .bind(provider_mode)
    .bind(run_message)
    .bind(crate::time::now_rfc3339_utc())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn non_empty_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::RunCompletionRuntime;
    use crate::llm::{ProviderKind, ResolvedLlmProfile};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    async fn start_model_metadata_server() -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind model metadata server");
        let base_url = format!("http://{}", listener.local_addr().expect("model endpoint"));
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept model request");
            let mut request = Vec::new();
            let mut chunk = [0_u8; 1024];
            loop {
                let read = socket.read(&mut chunk).await.expect("read model request");
                assert!(read > 0, "model request ended before headers");
                request.extend_from_slice(&chunk[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8_lossy(&request);
            assert!(request.starts_with("GET /models "));

            let body = r#"{"data":[{"id":"override-model","object":"model","owned_by":"test","context_length":32768,"max_output_tokens":8192}]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body,
            );
            socket
                .write_all(response.as_bytes())
                .await
                .expect("write model response");
        });
        (base_url, server)
    }

    #[tokio::test]
    async fn browser_model_context_has_no_api_fields() {
        let runtime = RunCompletionRuntime::GeminiBrowser {
            browser_provider_config: None,
        };

        let context = runtime.model_context().await.expect("browser context");

        assert_eq!(context.profile_id, None);
        assert_eq!(context.model_override, None);
        assert_eq!(context.model_output_limit, None);
    }

    #[tokio::test]
    async fn api_model_context_retains_profile_and_override() {
        let (base_url, server) = start_model_metadata_server().await;
        let runtime = RunCompletionRuntime::Api {
            profile: ResolvedLlmProfile {
                profile_id: "profile-7".to_string(),
                provider: ProviderKind::OpenAiCompatible,
                default_model: "default-model".to_string(),
                api_key: "test-api-key".to_string().into(),
                base_url,
            },
            model_override: Some("override-model".to_string()),
        };

        let context = runtime.model_context().await.expect("api context");
        tokio::time::timeout(std::time::Duration::from_secs(2), server)
            .await
            .expect("model metadata server timeout")
            .expect("model metadata server");

        assert_eq!(context.profile_id.as_deref(), Some("profile-7"));
        assert_eq!(context.model_override.as_deref(), Some("override-model"));
        assert_eq!(context.model_output_limit, Some(8_192));
    }
}
