use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::time::Instant;

use serde::Deserialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::dto::{
    PromptPackRunEvent, PromptPackRunSummaryDto, PromptPackRuntimeProvider,
    PromptPackStageRunDto, StartYoutubeSummaryRunOutcomeDto,
};
use super::json_repair::JsonRepairStageExecutionRequest;
use super::youtube_summary::{
    execute_youtube_summary_run_with_stage_executor, model_budget_for_runtime,
    preflight_youtube_summary_in_pool, start_youtube_summary_run_in_pool,
    LlmCompletion as PromptPackLlmCompletion, SynthesisStageExecutionRequest,
    TranscriptAnalysisStageExecutionRequest,
    YoutubeSummaryRunExecutionOutcome, YoutubeSummaryStageExecutionError,
    YoutubeSummaryStageExecutionRequest,
};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::{
    resolve_effective_model, resolve_model_output_token_limit_for_backend,
    resolve_profile_for_backend, run_llm_collect_with_profile, LlmChatRequest, LlmMessage,
    LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState,
    ResolvedLlmProfile,
};

pub const PROMPT_PACK_RUN_EVENT: &str = "prompt-pack-run-event";
const TRANSCRIPT_ANALYSIS_STAGE_JSON: &str =
    include_str!("../../prompt-packs/youtube_summary/1.0.0/runtime/transcript_analysis.json");
const SYNTHESIS_STAGE_JSON: &str =
    include_str!("../../prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json");
#[cfg(debug_assertions)]
const PROMPT_PACK_CANCELLATION_SMOKE_FIXTURE_LABEL: &str =
    "__prompt_pack_cancellation_smoke_fixture__";

#[derive(Deserialize)]
struct StageRuntimeConfigAsset {
    runtime_configuration: Option<StageRuntimeConfiguration>,
}

#[derive(Deserialize)]
struct StageRuntimeConfiguration {
    budget_limits: Option<StageBudgetLimits>,
}

#[derive(Deserialize)]
struct StageBudgetLimits {
    max_output_tokens: Option<i64>,
}

#[derive(Default)]
pub struct PromptPackRunState {
    active: Mutex<HashSet<i64>>,
    cancellation_tokens: Mutex<HashMap<i64, CancellationToken>>,
}

impl PromptPackRunState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn track(&self, run_id: i64) -> AppResult<()> {
        self.active.lock().await.insert(run_id);
        self.ensure_cancellation_token(run_id).await;
        Ok(())
    }

    pub async fn track_if_absent(&self, run_id: i64) -> AppResult<bool> {
        let inserted = self.active.lock().await.insert(run_id);
        self.ensure_cancellation_token(run_id).await;
        Ok(inserted)
    }

    pub async fn request_cancel(&self, run_id: i64) -> AppResult<()> {
        self.ensure_cancellation_token(run_id).await.cancel();
        Ok(())
    }

    pub async fn child_token(&self, run_id: i64) -> Option<CancellationToken> {
        self.cancellation_tokens
            .lock()
            .await
            .get(&run_id)
            .map(CancellationToken::child_token)
    }

    pub async fn finish(&self, run_id: i64) {
        self.active.lock().await.remove(&run_id);
        self.cancellation_tokens.lock().await.remove(&run_id);
    }

    pub async fn active_run_ids(&self) -> Vec<i64> {
        let mut ids = self.active.lock().await.iter().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    pub async fn apply_event(&self, event: PromptPackRunEvent) {
        if matches!(
            event.kind.as_str(),
            "completed" | "partial" | "failed" | "cancelled" | "interrupted"
        ) {
            self.finish(event.run_id).await;
        }
    }

    async fn ensure_cancellation_token(&self, run_id: i64) -> CancellationToken {
        self.cancellation_tokens
            .lock()
            .await
            .entry(run_id)
            .or_insert_with(CancellationToken::new)
            .clone()
    }
}

#[tauri::command]
pub async fn preflight_youtube_summary_run(
    handle: AppHandle,
    project_id: Option<i64>,
    source_ids: Vec<i64>,
    profile_id: Option<String>,
    model_override: Option<String>,
    runtime_provider: Option<PromptPackRuntimeProvider>,
    browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
    output_language: String,
    control_preset: String,
    evidence_mode: String,
    include_comments: bool,
) -> AppResult<super::dto::YoutubeSummaryPreflightResponse> {
    let pool = get_pool(&handle).await?;
    let runtime_provider = runtime_provider.unwrap_or_default();
    preflight_youtube_summary_in_pool(
        &pool,
        super::dto::PreflightYoutubeSummaryRunRequest {
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
        },
        model_budget_for_runtime(runtime_provider),
    )
    .await
}

#[tauri::command]
pub async fn start_youtube_summary_run(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
    client_request_id: String,
    project_id: Option<i64>,
    source_ids: Vec<i64>,
    profile_id: Option<String>,
    model_override: Option<String>,
    runtime_provider: Option<PromptPackRuntimeProvider>,
    browser_provider_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
    output_language: String,
    control_preset: String,
    evidence_mode: String,
    include_comments: bool,
) -> AppResult<StartYoutubeSummaryRunOutcomeDto> {
    let pool = get_pool(&handle).await?;
    let runtime_provider = runtime_provider.unwrap_or_default();
    let outcome = start_youtube_summary_run_in_pool(
        &pool,
        super::dto::StartYoutubeSummaryRunRequest {
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
        },
    )
    .await?;
    if let StartYoutubeSummaryRunOutcomeDto::Started { run } = &outcome {
        let should_spawn = run.run_status == "queued" && state.track_if_absent(run.run_id).await?;
        if should_spawn {
            emit_prompt_pack_run_event(
                &handle,
                &state,
                PromptPackRunEvent {
                    run_id: run.run_id,
                    request_id: format!("run-{}", run.run_id),
                    kind: "queued".to_string(),
                    run_status: run.run_status.clone(),
                    phase: "snapshot".to_string(),
                    stage_run_id: None,
                    stage_name: None,
                    source_snapshot_id: None,
                    queue_position: run.queue_position,
                    progress_current: run.progress_current,
                    progress_total: run.progress_total,
                    message: run.latest_message.clone(),
                    error: None,
                },
            )
            .await;
            spawn_youtube_summary_execution(handle.clone(), run.run_id);
        }
    }
    Ok(outcome)
}

#[tauri::command]
pub async fn cancel_prompt_pack_run(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
    scheduler: State<'_, LlmSchedulerState>,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    state.request_cancel(run_id).await?;
    scheduler.cancel_run_requests(run_id).await;
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = 'cancelled', completed_at = COALESCE(completed_at, ?), updated_at = ?
         WHERE id = ? AND run_status IN ('queued', 'running')",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(run_id)
    .execute(&pool)
    .await
    .map_err(AppError::database)?;
    emit_prompt_pack_run_event(
        &handle,
        &state,
        PromptPackRunEvent {
            run_id,
            request_id: format!("cancel-{run_id}"),
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
    )
    .await;
    Ok(())
}

#[tauri::command]
pub async fn update_prompt_pack_run(
    handle: AppHandle,
    run_id: i64,
    run_label: Option<String>,
) -> AppResult<PromptPackRunSummaryDto> {
    let pool = get_pool(&handle).await?;
    update_prompt_pack_run_in_pool(&pool, run_id, run_label).await
}

#[tauri::command]
pub async fn delete_prompt_pack_run(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    delete_prompt_pack_run_in_pool(&pool, run_id).await?;
    state.finish(run_id).await;
    Ok(())
}

fn spawn_youtube_summary_execution(handle: AppHandle, run_id: i64) {
    tauri::async_runtime::spawn(async move {
        let result = execute_youtube_summary_run(handle.clone(), run_id).await;
        match result {
            Ok(outcome) => emit_youtube_summary_terminal_event(&handle, outcome).await,
            Err(error) => {
                if let Err(mark_error) =
                    mark_prompt_pack_run_failed(&handle, run_id, &error.message).await
                {
                    eprintln!("Prompt Pack run {run_id} failed and could not be marked failed: {mark_error}");
                }
                emit_youtube_summary_terminal_event(
                    &handle,
                    YoutubeSummaryRunExecutionOutcome {
                        run_id,
                        run_status: "failed".to_string(),
                        progress_current: 0,
                        progress_total: 0,
                        message: error.message,
                    },
                )
                .await;
            }
        }
    });
}

async fn execute_youtube_summary_run(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<YoutubeSummaryRunExecutionOutcome> {
    let pool = get_pool(&handle).await?;
    let config = load_run_llm_config(&pool, run_id).await?;
    let resolved_profile =
        resolve_profile_for_backend(&handle, config.profile_id.as_deref()).await?;
    let run_cancellation_token = handle
        .state::<PromptPackRunState>()
        .child_token(run_id)
        .await;
    emit_prompt_pack_run_event(
        &handle,
        &handle.state::<PromptPackRunState>(),
        PromptPackRunEvent {
            run_id,
            request_id: format!("run-{run_id}-started"),
            kind: "started".to_string(),
            run_status: "running".to_string(),
            phase: "execution".to_string(),
            stage_run_id: None,
            stage_name: None,
            source_snapshot_id: None,
            queue_position: None,
            progress_current: Some(0),
            progress_total: None,
            message: Some("Running".to_string()),
            error: None,
        },
    )
    .await;

    execute_youtube_summary_run_with_stage_executor(&pool, run_id, move |stage_request| {
        let handle = handle.clone();
        let profile = resolved_profile.clone();
        let model_override = config.model_override.clone();
        let run_cancellation_token = run_cancellation_token.clone();
        async move {
            match stage_request {
                YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => {
                    run_transcript_analysis_stage_request(
                        handle,
                        profile,
                        model_override,
                        run_cancellation_token,
                        request,
                    )
                    .await
                }
                YoutubeSummaryStageExecutionRequest::Synthesis(request) => {
                    run_synthesis_stage_request(
                        handle,
                        profile,
                        model_override,
                        run_cancellation_token,
                        request,
                    )
                    .await
                }
                YoutubeSummaryStageExecutionRequest::JsonRepair(request) => {
                    run_json_repair_stage_request(
                        handle,
                        profile,
                        model_override,
                        run_cancellation_token,
                        request,
                    )
                    .await
                }
            }
        }
    })
    .await
}

#[derive(Clone, Debug)]
struct RunLlmConfig {
    profile_id: Option<String>,
    model_override: Option<String>,
}

async fn load_run_llm_config(pool: &SqlitePool, run_id: i64) -> AppResult<RunLlmConfig> {
    sqlx::query_as::<_, (Option<String>, Option<String>)>(
        "SELECT provider_profile_id, model FROM prompt_pack_runs WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map(|(profile_id, model_override)| RunLlmConfig {
        profile_id,
        model_override,
    })
    .map_err(AppError::database)
}

async fn run_transcript_analysis_stage_request(
    handle: AppHandle,
    profile: ResolvedLlmProfile,
    model_override: Option<String>,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: TranscriptAnalysisStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    // Gemini Browser Provider completion routing will call
    // gemini_browser_stage::browser_result_to_completion_text after the provider command returns a
    // successful single-prompt result. The default Prompt Pack path remains API-backed until a run
    // request explicitly selects the browser provider.
    let effective_model = resolve_effective_model(&profile, model_override.as_deref())?;
    let model_output_limit =
        resolve_model_output_token_limit_for_backend(&profile, &effective_model).await;
    let control_preset = transcript_analysis_control_preset(&stage_request.prompt_input_json);
    let stage_output_budget =
        transcript_analysis_stage_max_output_token_budget_for_control_preset(&control_preset)?;
    let max_output_tokens =
        transcript_analysis_max_output_tokens(stage_output_budget, model_output_limit);
    let llm_request = build_transcript_analysis_llm_request(
        &stage_request,
        Some(profile.profile_id.clone()),
        model_override,
        max_output_tokens,
    );
    let request_id = llm_request.request_id.clone();
    let provider = profile.provider.as_str().to_string();
    let scheduler = handle.state::<LlmSchedulerState>();
    let queued_handle = handle.clone();
    let started_handle = handle.clone();
    let queued_request_id = request_id.clone();
    let started_request_id = request_id.clone();
    let queued_stage_name = "youtube_summary/transcript_analysis".to_string();
    let started_stage_name = queued_stage_name.clone();
    let queued_stage_run_id = stage_request.stage_run_id;
    let started_stage_run_id = stage_request.stage_run_id;
    let queued_source_snapshot_id = stage_request.source_snapshot_id;
    let started_source_snapshot_id = stage_request.source_snapshot_id;
    let run_id = stage_request.run_id;
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
                owner_run_id: Some(stage_request.run_id),
            },
            move |position| {
                let _ = queued_handle.emit(
                    PROMPT_PACK_RUN_EVENT,
                    PromptPackRunEvent {
                        run_id,
                        request_id: queued_request_id.clone(),
                        kind: "queued".to_string(),
                        run_status: "running".to_string(),
                        phase: "transcript_analysis".to_string(),
                        stage_run_id: Some(queued_stage_run_id),
                        stage_name: Some(queued_stage_name.clone()),
                        source_snapshot_id: Some(queued_source_snapshot_id),
                        queue_position: Some(position as i64),
                        progress_current: None,
                        progress_total: None,
                        message: Some(format!("LLM request queued at position {position}")),
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
                        phase: "transcript_analysis".to_string(),
                        stage_run_id: Some(started_stage_run_id),
                        stage_name: Some(started_stage_name),
                        source_snapshot_id: Some(started_source_snapshot_id),
                        queue_position: None,
                        progress_current: None,
                        progress_total: None,
                        message: Some("Analyzing transcript".to_string()),
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

async fn run_synthesis_stage_request(
    handle: AppHandle,
    profile: ResolvedLlmProfile,
    model_override: Option<String>,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: SynthesisStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let effective_model = resolve_effective_model(&profile, model_override.as_deref())?;
    let model_output_limit =
        resolve_model_output_token_limit_for_backend(&profile, &effective_model).await;
    let stage_output_budget = synthesis_stage_max_output_token_budget()?;
    let max_output_tokens =
        transcript_analysis_max_output_tokens(stage_output_budget, model_output_limit);
    let llm_request = build_synthesis_llm_request(
        stage_request.run_id,
        stage_request.stage_run_id,
        stage_request.prompt_input_json.clone(),
        Some(profile.profile_id.clone()),
        model_override,
        max_output_tokens,
    );
    let request_id = llm_request.request_id.clone();
    let provider = profile.provider.as_str().to_string();
    let scheduler = handle.state::<LlmSchedulerState>();
    let queued_handle = handle.clone();
    let started_handle = handle.clone();
    let queued_request_id = request_id.clone();
    let started_request_id = request_id.clone();
    let stage_run_id = stage_request.stage_run_id;
    let run_id = stage_request.run_id;
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
                owner_run_id: Some(stage_request.run_id),
            },
            move |position| {
                let _ = queued_handle.emit(
                    PROMPT_PACK_RUN_EVENT,
                    PromptPackRunEvent {
                        run_id,
                        request_id: queued_request_id.clone(),
                        kind: "queued".to_string(),
                        run_status: "running".to_string(),
                        phase: "synthesis".to_string(),
                        stage_run_id: Some(stage_run_id),
                        stage_name: Some("youtube_summary/synthesis".to_string()),
                        source_snapshot_id: None,
                        queue_position: Some(position as i64),
                        progress_current: None,
                        progress_total: None,
                        message: Some(format!("LLM request queued at position {position}")),
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
                        phase: "synthesis".to_string(),
                        stage_run_id: Some(stage_run_id),
                        stage_name: Some("youtube_summary/synthesis".to_string()),
                        source_snapshot_id: None,
                        queue_position: None,
                        progress_current: None,
                        progress_total: None,
                        message: Some("Synthesizing videos".to_string()),
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

async fn run_json_repair_stage_request(
    handle: AppHandle,
    profile: ResolvedLlmProfile,
    model_override: Option<String>,
    run_cancellation_token: Option<CancellationToken>,
    stage_request: JsonRepairStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let effective_model = resolve_effective_model(&profile, model_override.as_deref())?;
    let model_output_limit =
        resolve_model_output_token_limit_for_backend(&profile, &effective_model).await;
    let stage_output_budget = if stage_request.stage_name == "youtube_summary/synthesis" {
        synthesis_stage_max_output_token_budget()?
    } else if stage_request.stage_name == "youtube_summary/transcript_analysis" {
        let control_preset = transcript_analysis_control_preset(&stage_request.prompt_input_json);
        transcript_analysis_stage_max_output_token_budget_for_control_preset(&control_preset)?
    } else {
        transcript_analysis_stage_max_output_token_budget()?
    };
    let max_output_tokens =
        transcript_analysis_max_output_tokens(stage_output_budget, model_output_limit);
    let llm_request = build_json_repair_llm_request(
        &stage_request,
        Some(profile.profile_id.clone()),
        model_override,
        max_output_tokens,
    );
    let request_id = llm_request.request_id.clone();
    let provider = profile.provider.as_str().to_string();
    let scheduler = handle.state::<LlmSchedulerState>();
    let queued_handle = handle.clone();
    let started_handle = handle.clone();
    let queued_request_id = request_id.clone();
    let started_request_id = request_id.clone();
    let stage_name = stage_request.stage_name.clone();
    let queued_stage_name = stage_name.clone();
    let started_stage_name = stage_name;
    let stage_run_id = stage_request.stage_run_id;
    let run_id = stage_request.run_id;
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
                owner_run_id: Some(stage_request.run_id),
            },
            move |position| {
                let _ = queued_handle.emit(
                    PROMPT_PACK_RUN_EVENT,
                    PromptPackRunEvent {
                        run_id,
                        request_id: queued_request_id.clone(),
                        kind: "queued".to_string(),
                        run_status: "running".to_string(),
                        phase: "repair".to_string(),
                        stage_run_id: Some(stage_run_id),
                        stage_name: Some(queued_stage_name.clone()),
                        source_snapshot_id: None,
                        queue_position: Some(position as i64),
                        progress_current: None,
                        progress_total: None,
                        message: Some(format!("JSON repair queued at position {position}")),
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
                        phase: "repair".to_string(),
                        stage_run_id: Some(stage_run_id),
                        stage_name: Some(started_stage_name),
                        source_snapshot_id: None,
                        queue_position: None,
                        progress_current: None,
                        progress_total: None,
                        message: Some("Repairing provider JSON".to_string()),
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

async fn run_with_prompt_pack_run_cancellation<Fut, T>(
    run_cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> Result<T, LlmRequestError>
where
    Fut: Future<Output = Result<T, LlmRequestError>>,
{
    let Some(run_cancellation_token) = run_cancellation_token else {
        return future.await;
    };

    if run_cancellation_token.is_cancelled() {
        return Err(LlmRequestError::Cancelled);
    }

    tokio::select! {
        result = future => result,
        _ = run_cancellation_token.cancelled() => Err(LlmRequestError::Cancelled),
    }
}

const DETAILED_REPORT_CONTROL_PRESET: &str = "detailed_report";

const STANDARD_VIDEO_SUMMARY_PROMPT: &str = "Write 2 to 4 paragraphs in the requested output_language, covering the main argument, important context, and practical takeaways. Keep it grounded in the frozen transcript; do not copy long transcript passages.";

const DETAILED_VIDEO_SUMMARY_PROMPT: &str = r#"Put the full Markdown report inside video_candidate.summary_text. This must be the full report, not a short abstract. Minimum length: 800 words when the transcript has enough substance. Keep the response as strict JSON; escape Markdown as a JSON string. Use only the frozen transcript and provided metadata/material refs. Do not claim external verification unless the frozen input contains it.

**Системная роль:**

Вы — ведущий аналитик видеоконтента и эксперт по структурированию знаний. Ваша специализация — деконструкция сложных видео (обучение, лекции, интервью) в атомарные инструкции и глубокие аналитические отчеты.

### 1. Цели и задачи:**

* Предоставлять глубокий технический и смысловой анализ YouTube-видео.
* Создавать структурированные отчеты, включающие метаданные, эссенцию, пошаговые руководства и интерактивные пересказы.
* Использовать внешние ресурсы для проверки фактов и контекста.

---

### 2. Структура ответа

#### I. Метаданные и Контекст

* **Тип контента:** [Обучение / Новости / Интервью / Аналитика]
* **Наличие пошаговых инструкций:** [Да / Нет] (укажите сразу, содержит ли видео четкий алгоритм действий).
* **Целевая аудитория:** Кому и почему это полезно.
* **Инфо-карта:** Название видео (гиперссылка)| Автор (название канала), подписчики| Метрики: [Длительность, Дата, Охват].
* **Таймлайн:** Список ключевых этапов видео с таймкодами.

#### II. Эссенция (Суть)

* **Main Idea:** Главная мысль одним емким предложением.
* **Ключевые тезисы:** 3–5 пунктов с итоговыми выводами (факты, советы, цитаты).
* **Action Plan:** 2-3 конкретных шага: что сделать пользователю сразу после просмотра.

#### III. Пошаговое руководство (How-to) — НОВЫЙ БЛОК

*Этот блок обязателен, если в видео есть процесс (настройка ПО, рецепт, стратегия).*

* **Цель инструкции:** Какой результат получит пользователь.
* **Инструменты:** Что понадобится (сервисы, софт, ингредиенты).
* **Алгоритм:** Детальный нумерованный список. Каждый шаг включает:

1. **Действие:** Что делать.
2. **Таймкод:** `[MM:SS]` как ссылка.
3. **Нюанс:** Важное замечание от автора (чего избегать).

#### IV. Адаптивный модуль (Выполняется в зависимости от типа)

* **Для Обучения:** Глоссарий сложных терминов + Практическое задание для закрепления.
* **Для Новостей:** Список действующих лиц + Исторический/политический контекст (предыстория).
* **Для видео > 20 минут:** Раздел «FAQ: Часто задаваемые вопросы» (5 пар вопрос-ответ на основе видео).

#### V. Глубокий интерактивный пересказ

* **Объем:** Минимум 800-1000 слов. Никакой «воды», только плотный концентрат информации.
* **Структура:** Разбейте на главы с осмысленными заголовками.
* **Навигация:** Каждому важному факту или мысли ОБЯЗАТЕЛЬНО должен сопутствовать таймкод в формате `[ММ:СС]`, являющийся ссылкой.
* **Форматирование:** Используйте таблицы для сравнения характеристик, списки для перечисления.
* **Математика и Код:** Если в видео есть формулы — используйте LaTeX (например, $E=mc^2$). Если код — используйте блоки кода с указанием языка.

---

### 3. Правила оформления и Тон

1. **Язык:** Строго русский.
2. **Стиль:** Профессиональный, аналитический, без «воды».
3. **Визуальный стиль:**

* Заголовки `##` и `###`.
* Разделители `---` между крупными блоками.
* **Жирный шрифт** для ключевых понятий и определений.
* Цитаты `> ` для прямых высказываний автора.

4. **Запрет:** Не использовать фразы «В этом видео говорится...», «Автор рассказывает...». Сразу переходите к сути: «Метод X заключается в...»."#;

fn transcript_analysis_control_preset(prompt_input_json: &str) -> String {
    serde_json::from_str::<serde_json::Value>(prompt_input_json)
        .ok()
        .and_then(|input| {
            input
                .get("controlPreset")
                .or_else(|| input.get("control_preset"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "standard".to_string())
}

fn transcript_analysis_summary_prompt(control_preset: &str) -> &'static str {
    if control_preset == DETAILED_REPORT_CONTROL_PRESET {
        DETAILED_VIDEO_SUMMARY_PROMPT
    } else {
        STANDARD_VIDEO_SUMMARY_PROMPT
    }
}

fn build_transcript_analysis_llm_request(
    request: &TranscriptAnalysisStageExecutionRequest,
    profile_id: Option<String>,
    model_override: Option<String>,
    max_output_tokens: Option<i64>,
) -> LlmChatRequest {
    let control_preset = transcript_analysis_control_preset(&request.prompt_input_json);
    let summary_prompt = transcript_analysis_summary_prompt(&control_preset);
    LlmChatRequest {
        request_id: format!(
            "prompt-pack-run-{}-stage-{}",
            request.run_id, request.stage_run_id
        ),
        profile_id,
        model_override,
        max_output_tokens,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "Return strict JSON for the YouTube Summary transcript analysis stage. Use only refs from the provided registries.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Analyze the frozen transcript and return exactly one strict JSON object matching this shape:\n\
                     {{\n\
                     \"stage_io_version\": \"1.0\",\n\
                     \"schema_version\": \"1.0\",\n\
                     \"stage\": \"youtube_summary/transcript_analysis\",\n\
                     \"video_candidate\": {{\n\
                     \"summary_text\": \"readable narrative summary\",\n\
                     \"segment_candidates\": [],\n\
                     \"key_point_candidates\": [{{ \"text\": \"point\", \"segment_candidate_index\": 0, \"material_refs\": [\"allowed material ref\"] }}],\n\
                     \"quote_candidates\": [{{ \"text\": \"short quote\", \"segment_candidate_index\": 0, \"material_refs\": [\"allowed material ref\"] }}],\n\
                     \"action_item_candidates\": [],\n\
                     \"open_question_candidates\": []\n\
                     }},\n\
                     \"claim_candidates\": [{{ \"text\": \"claim\", \"material_refs\": [\"allowed material ref\"] }}],\n\
                     \"evidence_fragment_candidates\": [{{ \"text\": \"evidence quote or paraphrase\", \"quote_candidate_index\": 0, \"material_refs\": [\"allowed material ref\"] }}],\n\
                     \"warning_candidates\": []\n\
                     }}\n\n\
                     summary_text must be a readable narrative summary of the video, not a terse label. {}\n\n\
                     Do not include backend-owned refs or IDs such as segment_ref, key_point_ref, quote_ref, claim_id, evidence_id, source_ref_id, segment_id, key_point_id, quote_id, action_item_id, or open_question_id. For optional candidate-to-candidate linkage, use only zero-based segment_candidate_index and quote_candidate_index. Omit candidate index fields when no clear candidate link exists. Use material_refs only from allowed_material_refs in the frozen input. Do not rename fields. Do not wrap the JSON in Markdown.\n\n\
                     Frozen stage input JSON:\n{}",
                    summary_prompt,
                    request.prompt_input_json
                ),
            },
        ],
    }
}

fn build_synthesis_llm_request(
    run_id: i64,
    stage_run_id: i64,
    prompt_input_json: String,
    profile_id: Option<String>,
    model_override: Option<String>,
    max_output_tokens: Option<i64>,
) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!("prompt-pack-run-{run_id}-stage-{stage_run_id}"),
        profile_id,
        model_override,
        max_output_tokens,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "Return strict JSON for the YouTube Summary synthesis stage. Produce a synthesis_candidate only; the backend assigns canonical IDs and traversal fields.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Synthesize the transcript-analysis candidates into one strict JSON object with stage_io_version, schema_version, stage, synthesis_candidate, limitations, and warning_candidates.\n\nRequired synthesis_candidate shape:\n{{\n  \"summary_text\": \"readable synthesis summary\",\n  \"cross_video_themes\": [{{ \"theme_text\": \"theme\", \"source_refs\": [\"source_ref_1\"], \"claim_refs\": [], \"evidence_refs\": [] }}],\n  \"common_claims\": [],\n  \"contradictions_across_videos\": []\n}}\n\nsummary_text must be a readable synthesis summary, not a terse label. Write 3 to 5 paragraphs in the requested output_language, explaining the shared themes, meaningful differences, and combined takeaway across the analyzed videos. Keep it grounded in the transcript-analysis candidates and canonical_graph; do not copy long transcript passages.\n\nThe input wrapper field source_ref_id may be used only for reasoning. Do not copy the key source_ref_id into the output. Use only source_refs from allowed_refs.source_refs, claim_refs from allowed_refs.claim_refs, and evidence_refs from allowed_refs.evidence_refs. You may use segment_refs, key_point_refs, and quote_refs from allowed_refs only for reasoning over canonical_graph. Do not emit segment_refs, key_point_refs, or quote_refs in the output. Leave claim_refs or evidence_refs empty when no supporting allowed ref exists. Do not include backend-owned IDs or keys such as source_ref_id, theme_id, common_claim_id, contradiction_id, claim_id, evidence_id, video_id, section_id, or synthesis_item_id. Do not wrap the JSON in Markdown.\n\nSynthesis input JSON:\n{}",
                    prompt_input_json
                ),
            },
        ],
    }
}

fn build_json_repair_llm_request(
    request: &JsonRepairStageExecutionRequest,
    profile_id: Option<String>,
    model_override: Option<String>,
    max_output_tokens: Option<i64>,
) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!(
            "prompt-pack-run-{}-stage-{}-repair-{}",
            request.run_id, request.stage_run_id, request.attempt_number
        ),
        profile_id,
        model_override,
        max_output_tokens,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "Repair invalid provider JSON for a YouTube Summary pipeline stage. Return exactly one strict JSON object. Do not add Markdown, prose, comments, or backend-owned IDs.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Repair the provider output for stage `{}`.\n\n\
                     Parser/validator error:\n{}\n\n\
                     Original frozen stage input JSON:\n{}\n\n\
                     Invalid provider output:\n{}\n\n\
                     Return only the corrected JSON object for the same stage, schema_version, and stage_io_version. Preserve useful candidate text from the invalid output when possible. If the original output is truncated, complete only the missing JSON structure using the frozen input as context. Do not include backend-owned IDs.",
                    request.stage_name,
                    request.error_message,
                    request.prompt_input_json,
                    request.raw_output
                ),
            },
        ],
    }
}

fn transcript_analysis_stage_max_output_token_budget() -> AppResult<i64> {
    stage_max_output_token_budget(TRANSCRIPT_ANALYSIS_STAGE_JSON, "transcript-analysis")
}

fn transcript_analysis_stage_max_output_token_budget_for_control_preset(
    control_preset: &str,
) -> AppResult<i64> {
    let standard_budget = transcript_analysis_stage_max_output_token_budget()?;
    if control_preset == DETAILED_REPORT_CONTROL_PRESET {
        Ok(standard_budget.max(8_192))
    } else {
        Ok(standard_budget)
    }
}

fn synthesis_stage_max_output_token_budget() -> AppResult<i64> {
    stage_max_output_token_budget(SYNTHESIS_STAGE_JSON, "synthesis")
}

fn stage_max_output_token_budget(asset_json: &str, label: &str) -> AppResult<i64> {
    let asset = serde_json::from_str::<StageRuntimeConfigAsset>(asset_json).map_err(|error| {
        AppError::internal(format!(
            "Parse bundled {label} runtime configuration: {error}"
        ))
    })?;
    asset
        .runtime_configuration
        .and_then(|runtime| runtime.budget_limits)
        .and_then(|budget| budget.max_output_tokens)
        .filter(|max_output_tokens| *max_output_tokens > 0)
        .ok_or_else(|| {
            AppError::internal(format!(
                "Bundled {label} runtime configuration is missing positive max_output_tokens"
            ))
        })
}

fn transcript_analysis_max_output_tokens(
    stage_output_budget: i64,
    model_output_limit: Option<i64>,
) -> Option<i64> {
    Some(match model_output_limit.filter(|limit| *limit > 0) {
        Some(limit) => stage_output_budget.min(limit),
        None => stage_output_budget,
    })
}

async fn mark_prompt_pack_run_failed(
    handle: &AppHandle,
    run_id: i64,
    message: &str,
) -> AppResult<()> {
    let pool = get_pool(handle).await?;
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = 'failed',
             result_status = 'failed',
             latest_message = ?,
             completed_at = COALESCE(completed_at, ?),
             updated_at = ?
         WHERE id = ? AND run_status IN ('queued', 'running')",
    )
    .bind(message)
    .bind(now_string())
    .bind(now_string())
    .bind(run_id)
    .execute(&pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn emit_youtube_summary_terminal_event(
    handle: &AppHandle,
    outcome: YoutubeSummaryRunExecutionOutcome,
) {
    let state = handle.state::<PromptPackRunState>();
    let event_kind = match outcome.run_status.as_str() {
        "complete" => "completed",
        other => other,
    };
    emit_prompt_pack_run_event(
        handle,
        &state,
        PromptPackRunEvent {
            run_id: outcome.run_id,
            request_id: format!("run-{}-terminal", outcome.run_id),
            kind: event_kind.to_string(),
            run_status: outcome.run_status,
            phase: "terminal".to_string(),
            stage_run_id: None,
            stage_name: None,
            source_snapshot_id: None,
            queue_position: None,
            progress_current: Some(outcome.progress_current),
            progress_total: Some(outcome.progress_total),
            message: Some(outcome.message),
            error: None,
        },
    )
    .await;
}

#[tauri::command]
pub async fn list_prompt_pack_runs(
    handle: AppHandle,
    project_id: Option<i64>,
    limit: Option<i64>,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let pool = get_pool(&handle).await?;
    list_prompt_pack_runs_in_pool(
        &pool,
        super::dto::ListPromptPackRunsRequest { project_id, limit },
    )
    .await
}

#[tauri::command]
pub async fn list_active_prompt_pack_runs(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let pool = get_pool(&handle).await?;
    let ids = state.active_run_ids().await;
    let mut runs = Vec::new();
    for run_id in ids {
        if let Some(run) = load_run_summary_optional(&pool, run_id).await? {
            runs.push(run);
        }
    }
    Ok(runs)
}

#[tauri::command]
pub async fn list_prompt_pack_run_stages(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<Vec<PromptPackStageRunDto>> {
    let pool = get_pool(&handle).await?;
    list_prompt_pack_run_stages_in_pool(&pool, run_id).await
}

pub(crate) async fn cleanup_interrupted_prompt_pack_runs_in_pool(
    pool: &SqlitePool,
    state: &PromptPackRunState,
) -> AppResult<()> {
    let now = now_string();
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = 'interrupted', completed_at = COALESCE(completed_at, ?), updated_at = ?,
             latest_message = 'Interrupted during app shutdown'
         WHERE run_status IN ('queued', 'running')",
    )
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    for run_id in state.active_run_ids().await {
        state.finish(run_id).await;
    }
    Ok(())
}

pub async fn cleanup_interrupted_prompt_pack_runs(handle: AppHandle) {
    match get_pool(&handle).await {
        Ok(pool) => {
            let state = handle.state::<PromptPackRunState>();
            if let Err(error) = cleanup_interrupted_prompt_pack_runs_in_pool(&pool, &state).await {
                eprintln!("Prompt Pack cleanup failed: {error}");
            }
        }
        Err(error) => eprintln!("Prompt Pack cleanup skipped: {error}"),
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
pub async fn seed_prompt_pack_cancellation_smoke_fixture(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<PromptPackRunSummaryDto> {
    let pool = get_pool(&handle).await?;
    seed_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, state.inner()).await
}

#[cfg(debug_assertions)]
#[tauri::command]
pub async fn clear_prompt_pack_cancellation_smoke_fixture(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<i64> {
    let pool = get_pool(&handle).await?;
    clear_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, state.inner()).await
}

#[cfg(debug_assertions)]
async fn seed_prompt_pack_cancellation_smoke_fixture_in_pool(
    pool: &SqlitePool,
    state: &PromptPackRunState,
) -> AppResult<PromptPackRunSummaryDto> {
    clear_prompt_pack_cancellation_smoke_fixture_in_pool(pool, state).await?;
    let pack_version_id = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM prompt_pack_versions
         WHERE pack_id = 'youtube_summary' AND pack_version = '1.0.0' AND schema_version = '1.0'
         LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    let now = now_string();
    let run_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO prompt_pack_runs (
            pack_version_id, pack_id, pack_version, schema_version, run_status,
            result_status, provider_profile_id, model, output_language, control_preset,
            evidence_mode, include_comments, latest_message, progress_current,
            progress_total, created_at, started_at, updated_at, run_label
         )
         VALUES (
            ?, 'youtube_summary', '1.0.0', '1.0', 'running',
            'none', '__prompt_pack_cancellation_smoke_fixture__', 'smoke-model',
            'en', 'standard', 'standard', 0, 'Prompt Pack cancellation smoke fixture running',
            0, 1, ?, ?, ?, ?
         )
         RETURNING id",
    )
    .bind(pack_version_id)
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .bind(PROMPT_PACK_CANCELLATION_SMOKE_FIXTURE_LABEL)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    state.track(run_id).await?;
    load_run_summary_optional(pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Prompt Pack run {run_id} not found")))
}

#[cfg(debug_assertions)]
async fn clear_prompt_pack_cancellation_smoke_fixture_in_pool(
    pool: &SqlitePool,
    state: &PromptPackRunState,
) -> AppResult<i64> {
    let run_ids = prompt_pack_cancellation_smoke_fixture_run_ids(pool).await?;
    for run_id in &run_ids {
        state.request_cancel(*run_id).await?;
        state.finish(*run_id).await;
    }
    if run_ids.is_empty() {
        return Ok(0);
    }
    let mut deleted = 0;
    for run_id in run_ids {
        let result = sqlx::query("DELETE FROM prompt_pack_runs WHERE id = ?")
            .bind(run_id)
            .execute(pool)
            .await
            .map_err(AppError::database)?;
        deleted += result.rows_affected() as i64;
    }
    Ok(deleted)
}

#[cfg(debug_assertions)]
async fn prompt_pack_cancellation_smoke_fixture_run_ids(pool: &SqlitePool) -> AppResult<Vec<i64>> {
    sqlx::query_scalar(
        "SELECT id FROM prompt_pack_runs
         WHERE run_label = ?
         ORDER BY id",
    )
    .bind(PROMPT_PACK_CANCELLATION_SMOKE_FIXTURE_LABEL)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn list_prompt_pack_runs_in_pool(
    pool: &SqlitePool,
    request: super::dto::ListPromptPackRunsRequest,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let limit = request.limit.unwrap_or(20).clamp(1, 100);
    let rows = if let Some(project_id) = request.project_id {
        sqlx::query_as::<_, RunSummaryRow>(
            "SELECT id, project_id, run_label, runtime_provider, pack_id, pack_version,
                    run_status, result_status, created_at, started_at, completed_at,
                    latest_message, progress_current, progress_total, queue_position
             FROM prompt_pack_runs
             WHERE project_id = ?
             ORDER BY created_at DESC, id DESC
             LIMIT ?",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?
    } else {
        sqlx::query_as::<_, RunSummaryRow>(
            "SELECT id, project_id, run_label, runtime_provider, pack_id, pack_version,
                    run_status, result_status, created_at, started_at, completed_at,
                    latest_message, progress_current, progress_total, queue_position
             FROM prompt_pack_runs
             ORDER BY created_at DESC, id DESC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?
    };
    Ok(rows.into_iter().map(Into::into).collect())
}

pub(crate) async fn update_prompt_pack_run_in_pool(
    pool: &SqlitePool,
    run_id: i64,
    run_label: Option<String>,
) -> AppResult<PromptPackRunSummaryDto> {
    let normalized_label = normalize_prompt_pack_run_label(run_label);
    let result = sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_label = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&normalized_label)
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!(
            "Prompt Pack run {run_id} not found"
        )));
    }

    load_run_summary_optional(pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Prompt Pack run {run_id} not found")))
}

pub(crate) async fn delete_prompt_pack_run_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<()> {
    let status =
        sqlx::query_scalar::<_, String>("SELECT run_status FROM prompt_pack_runs WHERE id = ?")
            .bind(run_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?
            .ok_or_else(|| AppError::not_found(format!("Prompt Pack run {run_id} not found")))?;

    if status == "queued" || status == "running" {
        return Err(AppError::conflict(
            "Queued or running Prompt Pack runs cannot be deleted",
        ));
    }

    sqlx::query("DELETE FROM prompt_pack_runs WHERE id = ?")
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

fn normalize_prompt_pack_run_label(label: Option<String>) -> Option<String> {
    label
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn list_prompt_pack_run_stages_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<PromptPackStageRunDto>> {
    sqlx::query_as::<_, (i64, i64, Option<i64>, String, i64, String, Option<String>)>(
        "SELECT id, run_id, source_snapshot_id, stage_name, stage_order,
                stage_status, latest_message
         FROM prompt_pack_stage_runs
         WHERE run_id = ?
         ORDER BY stage_order ASC, id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(
                    stage_run_id,
                    run_id,
                    source_snapshot_id,
                    stage_name,
                    stage_order,
                    stage_status,
                    latest_message,
                )| PromptPackStageRunDto {
                    stage_run_id,
                    run_id,
                    source_snapshot_id,
                    stage_name,
                    stage_order,
                    stage_status,
                    latest_message,
                },
            )
            .collect()
    })
    .map_err(AppError::database)
}

async fn load_run_summary_optional(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Option<PromptPackRunSummaryDto>> {
    sqlx::query_as::<_, RunSummaryRow>(
        "SELECT id, project_id, run_label, runtime_provider, pack_id, pack_version,
                run_status, result_status, created_at, started_at, completed_at,
                latest_message, progress_current, progress_total, queue_position
         FROM prompt_pack_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(Into::into))
    .map_err(AppError::database)
}

async fn emit_prompt_pack_run_event(
    handle: &AppHandle,
    state: &PromptPackRunState,
    event: PromptPackRunEvent,
) {
    state.apply_event(event.clone()).await;
    let _ = handle.emit(PROMPT_PACK_RUN_EVENT, event);
}

#[derive(sqlx::FromRow)]
struct RunSummaryRow {
    id: i64,
    project_id: Option<i64>,
    run_label: Option<String>,
    runtime_provider: String,
    pack_id: String,
    pack_version: String,
    run_status: String,
    result_status: String,
    created_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    latest_message: Option<String>,
    progress_current: Option<i64>,
    progress_total: Option<i64>,
    queue_position: Option<i64>,
}

impl From<RunSummaryRow> for PromptPackRunSummaryDto {
    fn from(row: RunSummaryRow) -> Self {
        Self {
            run_id: row.id,
            project_id: row.project_id,
            run_label: row.run_label,
            runtime_provider: row.runtime_provider,
            pack_id: row.pack_id,
            pack_version: row.pack_version,
            run_status: row.run_status,
            result_status: row.result_status,
            created_at: row.created_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
            latest_message: row.latest_message,
            progress_current: row.progress_current,
            progress_total: row.progress_total,
            queue_position: row.queue_position,
        }
    }
}

fn now_string() -> String {
    crate::time::now_rfc3339_utc()
}

#[cfg(test)]
mod tests {
    use super::{
        build_synthesis_llm_request, build_transcript_analysis_llm_request,
        cleanup_interrupted_prompt_pack_runs_in_pool,
        clear_prompt_pack_cancellation_smoke_fixture_in_pool, delete_prompt_pack_run_in_pool,
        list_prompt_pack_runs_in_pool, now_string, run_with_prompt_pack_run_cancellation,
        seed_prompt_pack_cancellation_smoke_fixture_in_pool,
        synthesis_stage_max_output_token_budget, transcript_analysis_max_output_tokens,
        transcript_analysis_stage_max_output_token_budget,
        transcript_analysis_stage_max_output_token_budget_for_control_preset,
        update_prompt_pack_run_in_pool, PromptPackRunState, DETAILED_REPORT_CONTROL_PRESET,
    };
    use crate::llm::LlmRequestError;
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::prompt_packs::dto::{ListPromptPackRunsRequest, PromptPackRunEvent};
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;
    use crate::prompt_packs::youtube_summary::TranscriptAnalysisStageExecutionRequest;
    use tokio_util::sync::CancellationToken;

    #[test]
    fn now_string_uses_current_utc_time() {
        use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

        let before = OffsetDateTime::now_utc() - Duration::seconds(5);
        let value = now_string();
        let after = OffsetDateTime::now_utc() + Duration::seconds(5);
        let parsed = OffsetDateTime::parse(&value, &Rfc3339).expect("parse runtime timestamp");

        assert_ne!(value, "2026-06-14T00:00:00Z");
        assert!(
            parsed >= before && parsed <= after,
            "expected {value} to be between {before} and {after}"
        );
    }

    #[tokio::test]
    async fn prompt_pack_run_state_tracks_active_and_cancel_requested_runs() {
        let state = PromptPackRunState::new();

        assert!(state.track_if_absent(42).await.expect("first track"));
        assert!(!state.track_if_absent(42).await.expect("duplicate track"));
        state.track(43).await.expect("track second");
        assert!(state.active_run_ids().await.contains(&42));

        state.request_cancel(42).await.expect("cancel");

        state.finish(42).await;
        assert!(!state.active_run_ids().await.contains(&42));
        assert!(state.active_run_ids().await.contains(&43));
    }

    #[tokio::test]
    async fn prompt_pack_run_state_cancels_child_tokens() {
        let state = PromptPackRunState::new();

        state.track(42).await.expect("track");
        let child = state.child_token(42).await.expect("child token");
        assert!(!child.is_cancelled());

        state.request_cancel(42).await.expect("cancel");

        tokio::time::timeout(std::time::Duration::from_secs(1), child.cancelled())
            .await
            .expect("child token cancelled");

        state.finish(42).await;
        assert!(state.child_token(42).await.is_none());
    }

    #[tokio::test]
    async fn prompt_pack_cancellation_smoke_fixture_tracks_active_run() {
        let pool = test_pool_with_prompt_pack_runs([]).await;
        let state = PromptPackRunState::new();

        let run = seed_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, &state)
            .await
            .expect("seed smoke fixture");

        assert_eq!(run.run_status, "running");
        assert_eq!(
            run.run_label.as_deref(),
            Some(super::PROMPT_PACK_CANCELLATION_SMOKE_FIXTURE_LABEL)
        );
        assert!(state.active_run_ids().await.contains(&run.run_id));
        assert!(state.child_token(run.run_id).await.is_some());
    }

    #[tokio::test]
    async fn prompt_pack_cancellation_smoke_fixture_clear_cancels_tokens_and_deletes_rows() {
        let pool = test_pool_with_prompt_pack_runs([]).await;
        let state = PromptPackRunState::new();
        let run = seed_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, &state)
            .await
            .expect("seed smoke fixture");
        let child = state.child_token(run.run_id).await.expect("child token");

        let deleted = clear_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, &state)
            .await
            .expect("clear smoke fixture");

        assert_eq!(deleted, 1);
        tokio::time::timeout(std::time::Duration::from_secs(1), child.cancelled())
            .await
            .expect("fixture child token cancelled");
        assert!(!state.active_run_ids().await.contains(&run.run_id));
        let row_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_runs WHERE run_label = ?")
                .bind(super::PROMPT_PACK_CANCELLATION_SMOKE_FIXTURE_LABEL)
                .fetch_one(&pool)
                .await
                .expect("count smoke rows");
        assert_eq!(row_count, 0);
    }

    #[tokio::test]
    async fn prompt_pack_run_cancellation_allows_completed_stage_future() {
        let result = run_with_prompt_pack_run_cancellation(None, async {
            Ok::<_, LlmRequestError>("completed")
        })
        .await;

        assert_eq!(result.expect("stage future"), "completed");
    }

    #[tokio::test]
    async fn prompt_pack_run_cancellation_interrupts_stage_future() {
        let token = CancellationToken::new();
        token.cancel();

        let result: Result<(), LlmRequestError> =
            run_with_prompt_pack_run_cancellation(Some(token), std::future::pending()).await;

        assert!(matches!(result, Err(LlmRequestError::Cancelled)));
    }

    #[tokio::test]
    async fn terminal_event_removes_run_from_active_state() {
        let state = PromptPackRunState::new();

        state.track(42).await.expect("track");
        state
            .apply_event(PromptPackRunEvent {
                run_id: 42,
                request_id: "req-42".to_string(),
                kind: "completed".to_string(),
                run_status: "complete".to_string(),
                phase: "terminal".to_string(),
                stage_run_id: None,
                stage_name: None,
                source_snapshot_id: None,
                queue_position: None,
                progress_current: Some(1),
                progress_total: Some(1),
                message: Some("Completed".to_string()),
                error: None,
            })
            .await;

        assert!(!state.active_run_ids().await.contains(&42));
    }

    #[tokio::test]
    async fn cleanup_interrupted_prompt_pack_runs_marks_stale_active_rows_interrupted() {
        let pool = test_pool_with_prompt_pack_runs([
            (41, None, "queued", "2026-06-14T10:00:00Z"),
            (42, None, "running", "2026-06-14T11:00:00Z"),
            (43, None, "complete", "2026-06-14T12:00:00Z"),
        ])
        .await;
        let state = PromptPackRunState::new();

        cleanup_interrupted_prompt_pack_runs_in_pool(&pool, &state)
            .await
            .expect("cleanup");

        let statuses = list_run_statuses(&pool).await;
        assert_eq!(statuses.get(&41).map(String::as_str), Some("interrupted"));
        assert_eq!(statuses.get(&42).map(String::as_str), Some("interrupted"));
        assert_eq!(statuses.get(&43).map(String::as_str), Some("complete"));
    }

    #[tokio::test]
    async fn list_prompt_pack_runs_returns_recent_runs_for_project() {
        let pool = test_pool_with_prompt_pack_runs([
            (41, Some(7), "complete", "2026-06-14T10:00:00Z"),
            (42, Some(7), "running", "2026-06-14T11:00:00Z"),
            (43, Some(8), "complete", "2026-06-14T12:00:00Z"),
        ])
        .await;

        let runs = list_prompt_pack_runs_in_pool(
            &pool,
            ListPromptPackRunsRequest {
                project_id: Some(7),
                limit: Some(20),
            },
        )
        .await
        .expect("recent runs");

        assert_eq!(
            runs.iter().map(|run| run.run_id).collect::<Vec<_>>(),
            vec![42, 41]
        );
        assert!(runs.iter().all(|run| run.project_id == Some(7)));
    }

    #[tokio::test]
    async fn update_prompt_pack_run_updates_user_label_only() {
        let pool =
            test_pool_with_prompt_pack_runs([(41, Some(7), "complete", "2026-06-14T10:00:00Z")])
                .await;

        let run = update_prompt_pack_run_in_pool(&pool, 41, Some("  June summary  ".to_string()))
            .await
            .expect("update label");

        assert_eq!(run.run_label.as_deref(), Some("June summary"));
        let status: String =
            sqlx::query_scalar("SELECT run_status FROM prompt_pack_runs WHERE id = 41")
                .fetch_one(&pool)
                .await
                .expect("status");
        assert_eq!(status, "complete");
    }

    #[tokio::test]
    async fn delete_prompt_pack_run_rejects_active_runs() {
        let pool = test_pool_with_prompt_pack_runs([
            (41, Some(7), "running", "2026-06-14T10:00:00Z"),
            (42, Some(7), "complete", "2026-06-14T11:00:00Z"),
        ])
        .await;

        let active_error = delete_prompt_pack_run_in_pool(&pool, 41)
            .await
            .expect_err("active run delete rejected");
        assert_eq!(active_error.kind, crate::error::AppErrorKind::Conflict);

        delete_prompt_pack_run_in_pool(&pool, 42)
            .await
            .expect("delete complete run");
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_runs WHERE id = 42")
            .fetch_one(&pool)
            .await
            .expect("count deleted run");
        assert_eq!(count, 0);
    }

    #[test]
    fn transcript_analysis_llm_request_embeds_frozen_stage_input() {
        let request = build_transcript_analysis_llm_request(
            &TranscriptAnalysisStageExecutionRequest {
                run_id: 42,
                stage_run_id: 1001,
                source_snapshot_id: 501,
                source_ref_id: "source_ref_1".to_string(),
                prompt_input_json:
                    "{\"stage\":\"youtube_summary/transcript_analysis\",\"controlPreset\":\"standard\"}"
                        .to_string(),
            },
            Some("profile-1".to_string()),
            Some("model-1".to_string()),
            transcript_analysis_max_output_tokens(
                transcript_analysis_stage_max_output_token_budget().expect("stage budget"),
                None,
            ),
        );

        assert_eq!(request.request_id, "prompt-pack-run-42-stage-1001");
        assert_eq!(request.profile_id.as_deref(), Some("profile-1"));
        assert_eq!(request.model_override.as_deref(), Some("model-1"));
        assert_eq!(request.max_output_tokens, Some(4096));
        assert_eq!(request.messages[0].role, "system");
        assert!(request.messages[0].content.contains("Return strict JSON"));
        assert_eq!(request.messages[1].role, "user");
        assert!(request.messages[1]
            .content
            .contains("Analyze the frozen transcript"));
        assert!(request.messages[1].content.contains("stage_io_version"));
        assert!(request.messages[1].content.contains("summary_text"));
        assert!(request.messages[1]
            .content
            .contains("summary_text must be a readable narrative summary"));
        assert!(request.messages[1].content.contains("2 to 4 paragraphs"));
        assert!(!request.messages[1]
            .content
            .contains("Put the full Markdown report inside video_candidate.summary_text"));
        assert!(!request.messages[1].content.contains("Системная роль"));
        assert!(!request.messages[1]
            .content
            .contains("Минимум 800-1000 слов"));
        assert!(!request.messages[1].content.contains("concise summary"));
        assert!(request.messages[1]
            .content
            .contains("Do not include backend-owned refs or IDs"));
        assert!(request.messages[1]
            .content
            .contains("\"stage\":\"youtube_summary/transcript_analysis\""));
    }

    #[test]
    fn transcript_analysis_llm_request_uses_detailed_report_prompt_for_control_preset() {
        let request = build_transcript_analysis_llm_request(
            &TranscriptAnalysisStageExecutionRequest {
                run_id: 42,
                stage_run_id: 1001,
                source_snapshot_id: 501,
                source_ref_id: "source_ref_1".to_string(),
                prompt_input_json: "{\"stage\":\"youtube_summary/transcript_analysis\",\"controlPreset\":\"detailed_report\"}"
                    .to_string(),
            },
            Some("profile-1".to_string()),
            Some("model-1".to_string()),
            transcript_analysis_max_output_tokens(
                transcript_analysis_stage_max_output_token_budget().expect("stage budget"),
                None,
            ),
        );

        assert!(request.messages[1]
            .content
            .contains("Put the full Markdown report inside video_candidate.summary_text"));
        assert!(request.messages[1].content.contains("Системная роль"));
        assert!(request.messages[1]
            .content
            .contains("Минимум 800-1000 слов"));
        assert!(!request.messages[1].content.contains("concise summary"));
        assert!(request.messages[1]
            .content
            .contains("Do not include backend-owned refs or IDs"));
        assert!(request.messages[1]
            .content
            .contains("\"stage\":\"youtube_summary/transcript_analysis\""));
    }

    #[test]
    fn transcript_analysis_llm_request_describes_candidate_indexes_and_forbids_backend_refs() {
        let request = build_transcript_analysis_llm_request(
            &TranscriptAnalysisStageExecutionRequest {
                run_id: 42,
                stage_run_id: 1001,
                source_snapshot_id: 501,
                source_ref_id: "source_ref_1".to_string(),
                prompt_input_json: "{\"stage\":\"youtube_summary/transcript_analysis\"}"
                    .to_string(),
            },
            None,
            Some("model".to_string()),
            Some(1024),
        );
        let prompt = &request.messages[1].content;

        assert!(prompt.contains("segment_candidate_index"));
        assert!(prompt.contains("quote_candidate_index"));
        assert!(prompt.contains("zero-based"));
        assert!(prompt.contains("Do not include backend-owned refs or IDs"));
        assert!(prompt.contains("segment_ref"));
        assert!(prompt.contains("quote_ref"));
        assert!(prompt.contains("source_ref_id"));
    }

    #[test]
    fn synthesis_llm_request_describes_allowed_refs_and_forbids_direct_intermediate_refs() {
        let request = build_synthesis_llm_request(
            42,
            2001,
            "{\"allowed_refs\":{}}".to_string(),
            None,
            Some("model".to_string()),
            Some(1024),
        );
        let prompt = &request.messages[1].content;

        assert!(prompt.contains("allowed_refs.source_refs"));
        assert!(prompt.contains("allowed_refs.claim_refs"));
        assert!(prompt.contains("allowed_refs.evidence_refs"));
        assert!(prompt.contains("Do not emit segment_refs"));
        assert!(prompt.contains("key_point_refs"));
        assert!(prompt.contains("quote_refs"));
        assert!(prompt.contains("summary_text must be a readable synthesis summary"));
        assert!(prompt.contains("3 to 5 paragraphs"));
        assert!(!prompt.contains("combined readable summary"));
    }

    #[test]
    fn transcript_analysis_output_budget_is_clamped_to_model_limit() {
        assert_eq!(
            transcript_analysis_max_output_tokens(4_096, Some(2_048)),
            Some(2_048)
        );
        assert_eq!(
            transcript_analysis_max_output_tokens(4_096, Some(8_192)),
            Some(4_096)
        );
        assert_eq!(
            transcript_analysis_max_output_tokens(4_096, None),
            Some(4_096)
        );
    }

    #[test]
    fn transcript_analysis_output_budget_comes_from_stage_runtime_config() {
        assert_eq!(
            transcript_analysis_stage_max_output_token_budget().expect("load stage budget"),
            4_096
        );
    }

    #[test]
    fn detailed_report_control_preset_uses_larger_transcript_analysis_output_budget() {
        assert_eq!(
            transcript_analysis_stage_max_output_token_budget_for_control_preset(
                DETAILED_REPORT_CONTROL_PRESET
            )
            .expect("load detailed report budget"),
            8_192
        );
        assert_eq!(
            transcript_analysis_stage_max_output_token_budget_for_control_preset("standard")
                .expect("load standard budget"),
            4_096
        );
    }

    #[test]
    fn synthesis_output_budget_comes_from_stage_runtime_config() {
        assert_eq!(
            synthesis_stage_max_output_token_budget().expect("load synthesis budget"),
            6_144
        );
    }

    async fn test_pool_with_prompt_pack_runs<const N: usize>(
        rows: [(i64, Option<i64>, &str, &str); N],
    ) -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed");
        for (run_id, project_id, status, created_at) in rows {
            if let Some(project_id) = project_id {
                sqlx::query(
                    "INSERT OR IGNORE INTO projects (id, name, created_at, updated_at)
                     VALUES (?, ?, 1, 1)",
                )
                .bind(project_id)
                .bind(format!("Project {project_id}"))
                .execute(&pool)
                .await
                .expect("insert project");
            }
            sqlx::query(
                "INSERT INTO prompt_pack_runs (
                    id, project_id, pack_version_id, pack_id, pack_version,
                    schema_version, run_status, result_status, output_language,
                    control_preset, evidence_mode, include_comments,
                    latest_message, created_at, updated_at
                 )
                 VALUES (?, ?, 1, 'youtube_summary', '1.0.0', '1.0',
                    ?, 'none', 'en', 'standard', 'standard', 0,
                    'Test run', ?, ?)",
            )
            .bind(run_id)
            .bind(project_id)
            .bind(status)
            .bind(created_at)
            .bind(created_at)
            .execute(&pool)
            .await
            .expect("insert run");
        }
        pool
    }

    async fn list_run_statuses(pool: &sqlx::SqlitePool) -> std::collections::HashMap<i64, String> {
        sqlx::query_as::<_, (i64, String)>(
            "SELECT id, run_status FROM prompt_pack_runs ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .expect("statuses")
        .into_iter()
        .collect()
    }
}
