use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager, State};

use super::completion_transport::RunCompletionRuntime;
use super::dto::{
    PromptPackRunEvent, PromptPackRunSummaryDto, PromptPackRuntimeProvider, PromptPackStageRunDto,
    StartYoutubeSummaryRunOutcomeDto, StartYoutubeSummaryRunRequest,
    YoutubeSummaryPreflightFailure,
};
pub use super::run_control::PromptPackRunState;
use super::run_store::{
    delete_prompt_pack_run_in_pool, list_prompt_pack_run_stages_in_pool,
    list_prompt_pack_runs_in_pool, load_run_summary_optional, update_prompt_pack_run_in_pool,
};
use super::runtime_config::{load_run_runtime_config, RunRuntimeProvider};
use super::stage_execution::{
    run_gem_analysis_part_repair_request, run_gem_analysis_part_stage_request,
    run_json_repair_stage_request, run_synthesis_stage_request,
    run_transcript_analysis_stage_request,
};
use super::stage_request_policy::{
    gem_input_cap, transcript_analysis_stage_max_prompt_token_budget,
};
use super::youtube_summary::{
    execute_youtube_summary_run_with_stage_executor_with_options,
    load_youtube_summary_run_by_client_request_id_in_pool, model_budget_for_runtime,
    preflight_youtube_summary_in_pool, start_youtube_summary_run_in_pool,
    start_youtube_summary_run_with_preflight_failures_in_pool, GemAnalysisInputBudget,
    YoutubeSummaryExecutionOptions, YoutubeSummaryRunExecutionOutcome,
    YoutubeSummaryStageExecutionRequest,
};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::{
    resolve_effective_model, resolve_model_input_token_limit_for_backend,
    resolve_profile_for_backend, LlmSchedulerState,
};

pub use super::dto::PROMPT_PACK_RUN_EVENT;
#[cfg(dev)]
const PROMPT_PACK_CANCELLATION_SMOKE_FIXTURE_LABEL: &str =
    "__prompt_pack_cancellation_smoke_fixture__";

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
        super::dto::PreflightYoutubeSummaryRunRequest::new(
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
        ),
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
    let request = StartYoutubeSummaryRunRequest::new(
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
    );
    let outcome = if request.client_request_id().trim().is_empty() {
        start_youtube_summary_run_in_pool(&pool, request).await?
    } else if let Some(run) =
        load_youtube_summary_run_by_client_request_id_in_pool(&pool, request.client_request_id())
            .await?
    {
        StartYoutubeSummaryRunOutcomeDto::Started { run }
    } else {
        let runtime_failures =
            browser_runtime_start_failures_for_request(&handle, &request).await?;
        start_youtube_summary_run_with_preflight_failures_in_pool(&pool, request, runtime_failures)
            .await?
    };
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

async fn browser_runtime_start_failures_for_request(
    handle: &AppHandle,
    request: &StartYoutubeSummaryRunRequest,
) -> AppResult<Vec<YoutubeSummaryPreflightFailure>> {
    if request.runtime_provider() != PromptPackRuntimeProvider::GeminiBrowser {
        return Ok(Vec::new());
    }

    let browser_state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
    let status = crate::gemini_browser::provider_status(
        handle,
        &browser_state,
        request.browser_provider_config.clone(),
    )
    .await?;

    Ok(browser_runtime_start_blocking_failure(&status)
        .into_iter()
        .collect())
}

fn browser_runtime_start_blocking_failure(
    status: &crate::gemini_browser::GeminiBrowserProviderStatus,
) -> Option<YoutubeSummaryPreflightFailure> {
    if status.status == crate::gemini_browser::GeminiBrowserProviderStatusKind::Ready {
        return None;
    }

    let status_label = format!("{:?}", status.status);
    let detail = status
        .latest_message
        .as_deref()
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .unwrap_or(status_label.as_str());
    Some(YoutubeSummaryPreflightFailure {
        source_id: None,
        reason: "browser_provider_not_ready".to_string(),
        message: Some(format!("Gemini Browser Provider is not ready: {detail}")),
    })
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
    let config = load_run_runtime_config(&pool, run_id).await?;
    let (completion_runtime, model_input_limit) = match config.runtime_provider {
        RunRuntimeProvider::Api => {
            let profile =
                resolve_profile_for_backend(&handle, config.profile_id.as_deref()).await?;
            let effective_model =
                resolve_effective_model(&profile, config.model_override.as_deref())?;
            let model_input_limit =
                resolve_model_input_token_limit_for_backend(&profile, &effective_model).await;
            (
                RunCompletionRuntime::Api {
                    profile,
                    model_override: config.model_override.clone(),
                },
                model_input_limit,
            )
        }
        RunRuntimeProvider::GeminiBrowser => (
            RunCompletionRuntime::GeminiBrowser {
                browser_provider_config: config.browser_provider_config.clone(),
            },
            None,
        ),
    };
    let prompt_budget = transcript_analysis_stage_max_prompt_token_budget()?;
    let execution_options = YoutubeSummaryExecutionOptions {
        gem_input_budget: GemAnalysisInputBudget {
            max_input_tokens: gem_input_cap(model_input_limit, prompt_budget),
        },
    };
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

    let stage_pool = pool.clone();
    execute_youtube_summary_run_with_stage_executor_with_options(
        &pool,
        run_id,
        execution_options,
        move |stage_request| {
            let handle = handle.clone();
            let pool = stage_pool.clone();
            let completion_runtime = completion_runtime.clone();
            let run_cancellation_token = run_cancellation_token.clone();
            async move {
                match stage_request {
                    YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => {
                        run_transcript_analysis_stage_request(
                            handle,
                            pool,
                            completion_runtime,
                            run_cancellation_token,
                            request,
                        )
                        .await
                    }
                    YoutubeSummaryStageExecutionRequest::Synthesis(request) => {
                        run_synthesis_stage_request(
                            handle,
                            pool,
                            completion_runtime,
                            run_cancellation_token,
                            request,
                        )
                        .await
                    }
                    YoutubeSummaryStageExecutionRequest::JsonRepair(request) => {
                        run_json_repair_stage_request(
                            handle,
                            pool,
                            completion_runtime,
                            run_cancellation_token,
                            request,
                        )
                        .await
                    }
                    YoutubeSummaryStageExecutionRequest::GemAnalysisPart(request) => {
                        run_gem_analysis_part_stage_request(
                            handle,
                            pool,
                            completion_runtime,
                            run_cancellation_token,
                            request,
                        )
                        .await
                    }
                    YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(request) => {
                        run_gem_analysis_part_repair_request(
                            handle,
                            pool,
                            completion_runtime,
                            run_cancellation_token,
                            request,
                        )
                        .await
                    }
                }
            }
        },
        |_| {},
    )
    .await
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
        super::dto::ListPromptPackRunsRequest::new(project_id, limit),
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

#[cfg(dev)]
#[tauri::command]
pub async fn seed_prompt_pack_cancellation_smoke_fixture(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<PromptPackRunSummaryDto> {
    let pool = get_pool(&handle).await?;
    seed_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, state.inner()).await
}

#[cfg(dev)]
#[tauri::command]
pub async fn clear_prompt_pack_cancellation_smoke_fixture(
    handle: AppHandle,
    state: State<'_, PromptPackRunState>,
) -> AppResult<i64> {
    let pool = get_pool(&handle).await?;
    clear_prompt_pack_cancellation_smoke_fixture_in_pool(&pool, state.inner()).await
}

#[cfg(dev)]
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

#[cfg(dev)]
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

#[cfg(dev)]
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

async fn emit_prompt_pack_run_event(
    handle: &AppHandle,
    state: &PromptPackRunState,
    event: PromptPackRunEvent,
) {
    state.apply_event(event.clone()).await;
    let _ = handle.emit(PROMPT_PACK_RUN_EVENT, event);
}

fn now_string() -> String {
    crate::time::now_rfc3339_utc()
}

#[cfg(test)]
mod tests {
    use super::super::completion_transport::{
        browser_run_id_for_stage, browser_run_source_for_stage,
        browser_stage_completion_from_result, llm_chat_request_to_browser_prompt,
        persist_browser_stage_provenance, run_browser_stage_result_with_cancellation,
    };
    use super::super::run_control::run_with_prompt_pack_run_cancellation;
    use super::super::run_store::{
        delete_prompt_pack_run_in_pool, list_prompt_pack_run_stages_in_pool,
        list_prompt_pack_runs_in_pool, update_prompt_pack_run_in_pool,
    };
    use super::super::runtime_config::{load_run_runtime_config, RunRuntimeProvider};
    use super::super::stage_request_policy::{
        build_gem_analysis_part_llm_request, build_gem_analysis_part_repair_llm_request,
        build_synthesis_llm_request, build_transcript_analysis_llm_request, gem_input_cap,
        synthesis_stage_max_output_token_budget, transcript_analysis_max_output_tokens,
        transcript_analysis_stage_max_output_token_budget,
        transcript_analysis_stage_max_output_token_budget_for_control_preset,
        transcript_analysis_stage_max_prompt_token_budget, DETAILED_REPORT_CONTROL_PRESET,
    };
    use super::{
        browser_runtime_start_blocking_failure, cleanup_interrupted_prompt_pack_runs_in_pool,
        clear_prompt_pack_cancellation_smoke_fixture_in_pool, now_string,
        seed_prompt_pack_cancellation_smoke_fixture_in_pool, PromptPackRunState,
    };
    use crate::gemini_browser::{GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind};
    use crate::llm::{LlmChatRequest, LlmMessage, LlmRequestError};
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::prompt_packs::dto::{ListPromptPackRunsRequest, PromptPackRunEvent};
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;
    use crate::prompt_packs::youtube_summary::{
        GemAnalysisPart, GemAnalysisPartRepairRequest, GemAnalysisPartStageExecutionRequest,
        TranscriptAnalysisStageExecutionRequest, YoutubeSummaryStageExecutionError,
    };
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    };
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
    async fn browser_cancellation_completes_before_terminal_persistence_and_event_follow_up() {
        let token = CancellationToken::new();
        let order = Arc::new(Mutex::new(Vec::new()));
        let cancellation_order = order.clone();
        let stage_result = run_browser_stage_result_with_cancellation(
            Some(token.clone()),
            std::future::pending(),
            move || async move {
                cancellation_order
                    .lock()
                    .expect("cancellation order")
                    .push("browser_cancel");
                Ok(())
            },
        );

        token.cancel();
        let stage_result = tokio::time::timeout(std::time::Duration::from_secs(1), stage_result)
            .await
            .expect("stage cancellation returned");
        order
            .lock()
            .expect("terminal persistence order")
            .push("terminal_persistence");
        order
            .lock()
            .expect("terminal event order")
            .push("terminal_event");

        assert!(matches!(
            stage_result,
            Err(YoutubeSummaryStageExecutionError::Cancelled)
        ));
        assert_eq!(
            *order.lock().expect("complete order"),
            ["browser_cancel", "terminal_persistence", "terminal_event"]
        );
    }

    #[test]
    fn start_source_applies_queued_state_and_event_before_spawned_profile_resolution() {
        let source = include_str!("runtime.rs");
        let start_begin = source
            .find("pub async fn start_youtube_summary_run(")
            .expect("start command");
        let start_end = source[start_begin..]
            .find("async fn browser_runtime_start_failures_for_request(")
            .map(|offset| start_begin + offset)
            .expect("start command end");
        let start = &source[start_begin..start_end];
        let event = start
            .find("emit_prompt_pack_run_event(")
            .expect("queued event publication");
        let spawn = start
            .find("spawn_youtube_summary_execution(")
            .expect("spawn directive");
        assert!(event < spawn);
        assert!(!start.contains("resolve_profile_for_backend"));

        let emitter_begin = source
            .find("async fn emit_prompt_pack_run_event(")
            .expect("event helper");
        let emitter = &source[emitter_begin..];
        let apply_state = emitter
            .find("state.apply_event(event.clone()).await")
            .expect("state transition");
        let publish = emitter
            .find("handle.emit(PROMPT_PACK_RUN_EVENT, event)")
            .expect("event emission");
        assert!(apply_state < publish);

        let spawn_begin = source
            .find("fn spawn_youtube_summary_execution(")
            .expect("spawn helper");
        let execution_begin = source
            .find("async fn execute_youtube_summary_run(")
            .expect("execution helper");
        let spawn_body = &source[spawn_begin..execution_begin];
        assert!(spawn_body.contains("tauri::async_runtime::spawn(async move"));
        assert!(spawn_body.contains("execute_youtube_summary_run(handle.clone(), run_id).await"));
        assert!(!spawn_body.contains("resolve_profile_for_backend"));

        let execution = &source[execution_begin..];
        assert!(execution.contains("resolve_profile_for_backend(&handle"));
    }

    #[tokio::test]
    async fn prompt_pack_browser_stage_cancelled_while_queued_cancels_browser_job() {
        let pool =
            test_pool_with_prompt_pack_runs([(41, Some(7), "running", "2026-06-22T10:00:00Z")])
                .await;
        let stage_run_id = 1001;
        insert_prompt_pack_browser_stage(&pool, 41, stage_run_id).await;
        let browser_run_id = browser_run_id_for_stage(41, stage_run_id, None, None);
        let runs_dir = tempfile::tempdir().expect("runs dir");
        crate::gemini_browser::create_queued_run(
            runs_dir.path(),
            &browser_run_id,
            "prompt_pack:youtube_summary:transcript_analysis",
            "Summarize",
        )
        .expect("queued browser run");
        let token = CancellationToken::new();
        let cancel_calls = Arc::new(AtomicUsize::new(0));
        let cancel_calls_for_hook = cancel_calls.clone();
        let browser_run_id_for_hook = browser_run_id.clone();
        let runs_root = runs_dir.path().to_path_buf();

        let stage_result = run_browser_stage_result_with_cancellation(
            Some(token.clone()),
            std::future::pending(),
            move || async move {
                cancel_calls_for_hook.fetch_add(1, Ordering::SeqCst);
                crate::gemini_browser::finish_run(
                    &runs_root,
                    &browser_run_id_for_hook,
                    cancelled_browser_result(&browser_run_id_for_hook),
                )
                .map_err(|error| crate::error::AppError::internal(error.to_string()))?;
                Ok(())
            },
        );

        token.cancel();
        let stage_result = tokio::time::timeout(std::time::Duration::from_secs(1), stage_result)
            .await
            .expect("stage cancellation returned");

        assert!(matches!(
            stage_result,
            Err(YoutubeSummaryStageExecutionError::Cancelled)
        ));
        assert_eq!(cancel_calls.load(Ordering::SeqCst), 1);
        let browser_run = crate::gemini_browser::list_runs(runs_dir.path(), 10)
            .expect("browser runs")
            .runs
            .into_iter()
            .find(|run| run.run_id == browser_run_id)
            .expect("browser run");
        assert_eq!(
            browser_run.status,
            crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
        );
        assert_browser_stage_has_no_success_provenance(&pool, stage_run_id).await;
    }

    #[tokio::test]
    async fn prompt_pack_browser_stage_cancelled_while_active_stops_sidecar() {
        let pool =
            test_pool_with_prompt_pack_runs([(42, Some(7), "running", "2026-06-22T10:00:00Z")])
                .await;
        let stage_run_id = 1002;
        insert_prompt_pack_browser_stage(&pool, 42, stage_run_id).await;
        let browser_run_id = browser_run_id_for_stage(42, stage_run_id, None, None);
        let runs_dir = tempfile::tempdir().expect("runs dir");
        crate::gemini_browser::create_queued_run(
            runs_dir.path(),
            &browser_run_id,
            "prompt_pack:youtube_summary:transcript_analysis",
            "Summarize",
        )
        .expect("queued browser run");
        crate::gemini_browser::mark_running(runs_dir.path(), &browser_run_id)
            .expect("mark browser run active");
        let token = CancellationToken::new();
        let active_browser_token = CancellationToken::new();
        let stop_requested = Arc::new(AtomicBool::new(false));
        let stop_requested_for_hook = stop_requested.clone();
        let active_browser_token_for_hook = active_browser_token.clone();
        let browser_run_id_for_hook = browser_run_id.clone();
        let runs_root = runs_dir.path().to_path_buf();

        let stage_result = run_browser_stage_result_with_cancellation(
            Some(token.clone()),
            std::future::pending(),
            move || async move {
                active_browser_token_for_hook.cancel();
                stop_requested_for_hook.store(true, Ordering::SeqCst);
                crate::gemini_browser::finish_run(
                    &runs_root,
                    &browser_run_id_for_hook,
                    cancelled_browser_result(&browser_run_id_for_hook),
                )
                .map_err(|error| crate::error::AppError::internal(error.to_string()))?;
                Ok(())
            },
        );

        token.cancel();
        let stage_result = tokio::time::timeout(std::time::Duration::from_secs(1), stage_result)
            .await
            .expect("stage cancellation returned");

        assert!(matches!(
            stage_result,
            Err(YoutubeSummaryStageExecutionError::Cancelled)
        ));
        assert!(active_browser_token.is_cancelled());
        assert!(stop_requested.load(Ordering::SeqCst));
        let browser_run = crate::gemini_browser::list_runs(runs_dir.path(), 10)
            .expect("browser runs")
            .runs
            .into_iter()
            .find(|run| run.run_id == browser_run_id)
            .expect("browser run");
        assert_eq!(
            browser_run.status,
            crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn cancelled_browser_stage_does_not_persist_success_provenance() {
        let pool =
            test_pool_with_prompt_pack_runs([(43, Some(7), "running", "2026-06-22T10:00:00Z")])
                .await;
        let stage_run_id = 1003;
        insert_prompt_pack_browser_stage(&pool, 43, stage_run_id).await;
        let token = CancellationToken::new();

        let stage_result = run_browser_stage_result_with_cancellation(
            Some(token.clone()),
            std::future::pending(),
            || async { Ok(()) },
        );

        token.cancel();
        let stage_result = tokio::time::timeout(std::time::Duration::from_secs(1), stage_result)
            .await
            .expect("stage cancellation returned");

        assert!(matches!(
            stage_result,
            Err(YoutubeSummaryStageExecutionError::Cancelled)
        ));
        assert_browser_stage_has_no_success_provenance(&pool, stage_run_id).await;
    }

    #[tokio::test]
    async fn prompt_pack_browser_stage_cancelled_before_enqueue_is_tolerated() {
        let pool =
            test_pool_with_prompt_pack_runs([(44, Some(7), "running", "2026-06-22T10:00:00Z")])
                .await;
        let stage_run_id = 1004;
        insert_prompt_pack_browser_stage(&pool, 44, stage_run_id).await;
        let token = CancellationToken::new();
        token.cancel();
        let cancel_calls = Arc::new(AtomicUsize::new(0));
        let cancel_calls_for_hook = cancel_calls.clone();

        let stage_result = run_browser_stage_result_with_cancellation(
            Some(token),
            std::future::pending(),
            move || async move {
                cancel_calls_for_hook.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        )
        .await;

        assert!(matches!(
            stage_result,
            Err(YoutubeSummaryStageExecutionError::Cancelled)
        ));
        assert_eq!(cancel_calls.load(Ordering::SeqCst), 1);
        assert_browser_stage_has_no_success_provenance(&pool, stage_run_id).await;
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

    #[test]
    fn browser_prompt_formatter_preserves_role_order_and_content() {
        let request = LlmChatRequest {
            request_id: "req-browser-format".to_string(),
            profile_id: None,
            model_override: None,
            max_output_tokens: None,
            messages: vec![
                LlmMessage {
                    role: "system".to_string(),
                    content: "Return strict JSON.".to_string(),
                },
                LlmMessage {
                    role: "user".to_string(),
                    content: "Analyze this transcript.".to_string(),
                },
                LlmMessage {
                    role: "user".to_string(),
                    content: "Use source_ref_1 only.".to_string(),
                },
            ],
        };

        let prompt = llm_chat_request_to_browser_prompt(&request).expect("format prompt");

        assert_eq!(
            prompt,
            "System:\nReturn strict JSON.\n\nUser:\nAnalyze this transcript.\n\nUser:\nUse source_ref_1 only."
        );
    }

    #[test]
    fn browser_prompt_formatter_rejects_unsupported_roles() {
        let request = LlmChatRequest {
            request_id: "req-browser-format".to_string(),
            profile_id: None,
            model_override: None,
            max_output_tokens: None,
            messages: vec![LlmMessage {
                role: "assistant".to_string(),
                content: "previous answer".to_string(),
            }],
        };

        let error = llm_chat_request_to_browser_prompt(&request).expect_err("unsupported role");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("assistant"));
    }

    #[test]
    fn browser_run_identity_includes_repair_attempt_when_present() {
        assert_eq!(
            browser_run_id_for_stage(42, 1001, None, None),
            "prompt-pack-42-stage-1001"
        );
        assert_eq!(
            browser_run_id_for_stage(42, 1001, Some(2), None),
            "prompt-pack-42-stage-1001-repair-2"
        );
        assert_eq!(
            browser_run_source_for_stage(42, 1001, "youtube_summary/transcript_analysis", None),
            "prompt_pack:youtube_summary:youtube_summary/transcript_analysis:run:42:stage:1001"
        );
    }

    #[test]
    fn browser_run_id_accepts_optional_gem_discriminator() {
        assert_eq!(
            browser_run_id_for_stage(42, 1001, None, None),
            "prompt-pack-42-stage-1001"
        );
        assert_eq!(
            browser_run_id_for_stage(42, 1001, Some(2), None),
            "prompt-pack-42-stage-1001-repair-2"
        );
        assert_eq!(
            browser_run_id_for_stage(42, 1001, None, Some("gem-passport")),
            "prompt-pack-42-stage-1001-gem-passport"
        );
        assert_eq!(
            browser_run_id_for_stage(42, 1001, None, Some("gem-deep-recap-repair-1")),
            "prompt-pack-42-stage-1001-gem-deep-recap-repair-1"
        );
    }

    #[test]
    fn browser_stage_result_maps_to_prompt_pack_completion_without_tokens() {
        let result = crate::gemini_browser::GeminiBrowserRunResult {
            run_id: "prompt-pack-42-stage-1001".to_string(),
            status: crate::gemini_browser::GeminiBrowserRunStatus::Ok,
            text: Some("answer".to_string()),
            message: None,
            manual_action: None,
            artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 321,
            debug_summary: None,
        };

        let completion = browser_stage_completion_from_result(result).expect("completion");

        assert_eq!(completion.text, "answer");
        assert_eq!(completion.input_tokens, None);
        assert_eq!(completion.output_tokens, None);
        assert_eq!(completion.latency_ms, 321);
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
    async fn load_run_runtime_config_reads_api_and_browser_rows() {
        let pool = test_pool_with_prompt_pack_runs([]).await;
        sqlx::query(
            "INSERT INTO prompt_pack_runs (
                id, project_id, pack_version_id, pack_id, pack_version,
                schema_version, run_status, result_status, provider_profile_id, model,
                runtime_provider, browser_provider_config_json, output_language,
                control_preset, evidence_mode, include_comments, latest_message,
                created_at, updated_at
             )
             VALUES
                (101, NULL, 1, 'youtube_summary', '1.0.0', '1.0',
                 'queued', 'none', 'profile-1', 'model-1', 'api', NULL,
                 'en', 'standard', 'standard', 0, 'Queued', '2026-06-21T00:00:00Z', '2026-06-21T00:00:00Z'),
                (102, NULL, 1, 'youtube_summary', '1.0.0', '1.0',
                 'queued', 'none', NULL, NULL, 'gemini_browser',
                 '{\"mode\":\"cdp_attach\",\"cdp_endpoint\":\"http://127.0.0.1:9222\"}',
                 'en', 'standard', 'standard', 0, 'Queued', '2026-06-21T00:00:00Z', '2026-06-21T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert runtime rows");

        let api = load_run_runtime_config(&pool, 101)
            .await
            .expect("api config");
        assert_eq!(api.runtime_provider, RunRuntimeProvider::Api);
        assert_eq!(api.profile_id.as_deref(), Some("profile-1"));
        assert_eq!(api.model_override.as_deref(), Some("model-1"));

        let browser = load_run_runtime_config(&pool, 102)
            .await
            .expect("browser config");
        assert_eq!(browser.runtime_provider, RunRuntimeProvider::GeminiBrowser);
        let browser_config = browser.browser_provider_config.expect("browser config");
        assert_eq!(
            browser_config.cdp_endpoint.as_deref(),
            Some("http://127.0.0.1:9222")
        );
    }

    #[tokio::test]
    async fn load_run_runtime_config_rejects_unsupported_provider() {
        let pool =
            test_pool_with_prompt_pack_runs([(103, None, "queued", "2026-06-21T00:00:00Z")]).await;
        let mut connection = pool.acquire().await.expect("acquire test connection");
        sqlx::query("PRAGMA ignore_check_constraints = ON")
            .execute(&mut *connection)
            .await
            .expect("allow corrupted runtime provider fixture");
        sqlx::query(
            "UPDATE prompt_pack_runs
             SET runtime_provider = 'unsupported'
             WHERE id = 103",
        )
        .execute(&mut *connection)
        .await
        .expect("set unsupported runtime provider");
        drop(connection);

        let error = load_run_runtime_config(&pool, 103)
            .await
            .expect_err("unsupported provider");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert_eq!(
            error.message,
            "Unsupported prompt-pack runtime provider: unsupported"
        );
    }

    #[tokio::test]
    async fn load_run_runtime_config_rejects_malformed_browser_config() {
        let pool =
            test_pool_with_prompt_pack_runs([(104, None, "queued", "2026-06-21T00:00:00Z")]).await;
        sqlx::query(
            "UPDATE prompt_pack_runs
             SET runtime_provider = 'gemini_browser',
                 browser_provider_config_json = '{not-json'
             WHERE id = 104",
        )
        .execute(&pool)
        .await
        .expect("set malformed browser config");

        let error = load_run_runtime_config(&pool, 104)
            .await
            .expect_err("malformed browser config");

        assert_eq!(error.kind, crate::error::AppErrorKind::Internal);
        assert!(
            error
                .message
                .starts_with("parse Browser Provider config snapshot:"),
            "unexpected error message: {}",
            error.message
        );
    }

    #[tokio::test]
    async fn list_prompt_pack_runs_returns_recent_runs_for_project() {
        let pool = test_pool_with_prompt_pack_runs([
            (41, Some(7), "complete", "2026-06-14T10:00:00Z"),
            (42, Some(7), "running", "2026-06-14T11:00:00Z"),
            (43, Some(8), "complete", "2026-06-14T12:00:00Z"),
        ])
        .await;

        let runs =
            list_prompt_pack_runs_in_pool(&pool, ListPromptPackRunsRequest::new(Some(7), Some(20)))
                .await
                .expect("recent runs");

        assert_eq!(
            runs.iter().map(|run| run.run_id).collect::<Vec<_>>(),
            vec![42, 41]
        );
        assert!(runs.iter().all(|run| run.project_id == Some(7)));
    }

    #[test]
    fn browser_runtime_start_gate_maps_unready_status_to_preflight_failure() {
        let status = GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::NeedsLogin,
            manual_action: None,
            active_run_id: None,
            queue_depth: 0,
            browser_profile_dir: "profile".to_string(),
            latest_message: Some("Login required".to_string()),
        };

        let failure = browser_runtime_start_blocking_failure(&status)
            .expect("needs_login should block browser runtime start");

        assert_eq!(failure.source_id, None);
        assert_eq!(failure.reason, "browser_provider_not_ready");
        assert!(failure
            .message
            .as_deref()
            .expect("message")
            .contains("Login required"));
    }

    #[test]
    fn browser_runtime_start_gate_allows_ready_status() {
        let status = GeminiBrowserProviderStatus {
            status: GeminiBrowserProviderStatusKind::Ready,
            manual_action: None,
            active_run_id: None,
            queue_depth: 0,
            browser_profile_dir: "profile".to_string(),
            latest_message: Some("Ready".to_string()),
        };

        assert_eq!(browser_runtime_start_blocking_failure(&status), None);
    }

    #[tokio::test]
    async fn list_prompt_pack_run_stages_returns_browser_provenance() {
        let pool =
            test_pool_with_prompt_pack_runs([(41, Some(7), "complete", "2026-06-14T10:00:00Z")])
                .await;
        sqlx::query(
            "INSERT INTO prompt_pack_stage_runs (
                id, run_id, stage_name, stage_order, stage_status,
                browser_run_id, browser_run_status, browser_completion_reason,
                browser_provider_mode, browser_run_message, created_at, updated_at
             )
             VALUES (
                1001, 41, 'youtube_summary/transcript_analysis', 20, 'succeeded',
                'prompt-pack-41-stage-1001', 'ok', 'stable',
                'cdp_attach', 'Browser answer accepted', '2026-06-14T10:00:01Z',
                '2026-06-14T10:00:02Z'
             )",
        )
        .execute(&pool)
        .await
        .expect("insert browser stage");

        let stages = list_prompt_pack_run_stages_in_pool(&pool, 41)
            .await
            .expect("stage list");

        assert_eq!(stages.len(), 1);
        let stage = &stages[0];
        assert_eq!(
            stage.browser_run_id.as_deref(),
            Some("prompt-pack-41-stage-1001")
        );
        assert_eq!(stage.browser_run_status.as_deref(), Some("ok"));
        assert_eq!(stage.browser_completion_reason.as_deref(), Some("stable"));
        assert_eq!(stage.browser_provider_mode.as_deref(), Some("cdp_attach"));
        assert_eq!(
            stage.browser_run_message.as_deref(),
            Some("Browser answer accepted")
        );
    }

    #[tokio::test]
    async fn persist_browser_stage_provenance_records_result_identity() {
        let pool =
            test_pool_with_prompt_pack_runs([(41, Some(7), "running", "2026-06-14T10:00:00Z")])
                .await;
        sqlx::query(
            "INSERT INTO prompt_pack_stage_runs (
                id, run_id, stage_name, stage_order, stage_status, created_at, updated_at
             )
             VALUES (
                1001, 41, 'youtube_summary/transcript_analysis', 20, 'running',
                '2026-06-14T10:00:01Z', '2026-06-14T10:00:01Z'
             )",
        )
        .execute(&pool)
        .await
        .expect("insert browser stage");

        persist_browser_stage_provenance(
            &pool,
            1001,
            &crate::gemini_browser::GeminiBrowserRunResult {
                run_id: "prompt-pack-41-stage-1001".to_string(),
                status: crate::gemini_browser::GeminiBrowserRunStatus::Ok,
                text: Some("answer".to_string()),
                message: Some("   ".to_string()),
                manual_action: None,
                artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
                elapsed_ms: 321,
                debug_summary: Some(crate::gemini_browser::GeminiBrowserRunDebugSummary {
                    mode: crate::gemini_browser::GeminiBrowserProviderMode::CdpAttach,
                    composer_found: true,
                    send_button_found: true,
                    generation_busy_observed: false,
                    answer_found: true,
                    answer_selector: Some("message-content".to_string()),
                    waited_for_send_ms: 0,
                    waited_for_answer_ms: 1200,
                    answer_stable_ms: 800,
                    answer_completion_reason:
                        crate::gemini_browser::GeminiBrowserAnswerCompletionReason::Stable,
                    final_text_length: 6,
                    error_stage: None,
                    extraction: None,
                }),
            },
        )
        .await
        .expect("persist browser provenance");

        let stage = list_prompt_pack_run_stages_in_pool(&pool, 41)
            .await
            .expect("stage list")
            .pop()
            .expect("stage");
        assert_eq!(
            stage.browser_run_id.as_deref(),
            Some("prompt-pack-41-stage-1001")
        );
        assert_eq!(stage.browser_run_status.as_deref(), Some("ok"));
        assert_eq!(stage.browser_completion_reason.as_deref(), Some("stable"));
        assert_eq!(stage.browser_provider_mode.as_deref(), Some("cdp_attach"));
        assert_eq!(stage.browser_run_message.as_deref(), None);
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
    fn gem_analysis_part_llm_request_preserves_part_and_frozen_input() {
        let request = build_gem_analysis_part_llm_request(
            &GemAnalysisPartStageExecutionRequest {
                run_id: 42,
                stage_run_id: 1001,
                source_snapshot_id: 501,
                source_ref_id: "source_ref_1".to_string(),
                part: GemAnalysisPart::Passport,
                prompt_input_json: "{\"frozen_input\":\"passport-source-material\"}".to_string(),
            },
            Some("profile-1".to_string()),
            Some("model-1".to_string()),
            Some(8_192),
        );

        assert_eq!(
            request.request_id,
            "prompt-pack-run-42-stage-1001-gem-passport"
        );
        assert_eq!(request.profile_id.as_deref(), Some("profile-1"));
        assert_eq!(request.model_override.as_deref(), Some("model-1"));
        assert_eq!(request.max_output_tokens, Some(8_192));
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
        assert!(request.messages[1]
            .content
            .contains("\"part\": \"passport\""));
        assert!(request.messages[1]
            .content
            .contains("{\"frozen_input\":\"passport-source-material\"}"));
    }

    #[test]
    fn gem_analysis_part_repair_llm_request_preserves_attempt_and_repair_context() {
        let request = build_gem_analysis_part_repair_llm_request(
            &GemAnalysisPartRepairRequest {
                run_id: 42,
                stage_run_id: 1002,
                source_snapshot_id: 501,
                source_ref_id: "source_ref_1".to_string(),
                part: GemAnalysisPart::Comments,
                attempt_number: 2,
                prompt_input_json: "{\"frozen_input\":\"comments-source-material\"}".to_string(),
                raw_output: "{invalid-provider-output".to_string(),
                error_message: "parser-sentinel: missing closing brace".to_string(),
            },
            Some("profile-1".to_string()),
            Some("model-1".to_string()),
            Some(4_096),
        );

        assert_eq!(
            request.request_id,
            "prompt-pack-run-42-stage-1002-gem-comments-repair-2"
        );
        assert_eq!(request.profile_id.as_deref(), Some("profile-1"));
        assert_eq!(request.model_override.as_deref(), Some("model-1"));
        assert_eq!(request.max_output_tokens, Some(4_096));
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
        assert!(request.messages[1]
            .content
            .contains("Repair the invalid Gem analysis part output for part `comments`"));
        assert!(request.messages[1]
            .content
            .contains("parser-sentinel: missing closing brace"));
        assert!(request.messages[1]
            .content
            .contains("{\"frozen_input\":\"comments-source-material\"}"));
        assert!(request.messages[1]
            .content
            .contains("{invalid-provider-output"));
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
    fn transcript_analysis_stage_max_prompt_token_budget_reads_runtime_config() {
        assert_eq!(
            transcript_analysis_stage_max_prompt_token_budget().expect("prompt budget"),
            24_000
        );
    }

    #[test]
    fn gem_input_budget_uses_lower_known_model_limit() {
        assert_eq!(gem_input_cap(Some(8_000), 24_000), 8_000);
        assert_eq!(gem_input_cap(Some(64_000), 24_000), 24_000);
        assert_eq!(gem_input_cap(None, 24_000), 24_000);
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

    async fn insert_prompt_pack_browser_stage(
        pool: &sqlx::SqlitePool,
        run_id: i64,
        stage_run_id: i64,
    ) {
        sqlx::query(
            "INSERT INTO prompt_pack_stage_runs (
                id, run_id, stage_name, stage_order, stage_status, created_at, updated_at
             )
             VALUES (
                ?, ?, 'youtube_summary/transcript_analysis', 20, 'running',
                '2026-06-22T10:00:01Z', '2026-06-22T10:00:01Z'
             )",
        )
        .bind(stage_run_id)
        .bind(run_id)
        .execute(pool)
        .await
        .expect("insert prompt pack browser stage");
    }

    fn cancelled_browser_result(run_id: &str) -> crate::gemini_browser::GeminiBrowserRunResult {
        crate::gemini_browser::GeminiBrowserRunResult {
            run_id: run_id.to_string(),
            status: crate::gemini_browser::GeminiBrowserRunStatus::Cancelled,
            text: None,
            message: Some("Cancelled".to_string()),
            manual_action: None,
            artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 0,
            debug_summary: None,
        }
    }

    async fn assert_browser_stage_has_no_success_provenance(
        pool: &sqlx::SqlitePool,
        stage_run_id: i64,
    ) {
        let status: Option<String> = sqlx::query_scalar(
            "SELECT browser_run_status FROM prompt_pack_stage_runs WHERE id = ?",
        )
        .bind(stage_run_id)
        .fetch_one(pool)
        .await
        .expect("read browser provenance");

        assert!(
            !matches!(status.as_deref(), Some("ok") | Some("ready")),
            "cancelled browser stage persisted success provenance: {status:?}"
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
