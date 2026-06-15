use std::collections::HashSet;
use std::time::Instant;

use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;

use super::dto::{
    PromptPackRunEvent, PromptPackRunSummaryDto, PromptPackStageRunDto,
    StartYoutubeSummaryRunOutcomeDto,
};
use super::youtube_summary::{
    execute_youtube_summary_run_with_stage_executor, preflight_youtube_summary_in_pool,
    start_youtube_summary_run_in_pool, LlmCompletion as PromptPackLlmCompletion, ModelBudget,
    TranscriptAnalysisStageExecutionRequest, YoutubeSummaryRunExecutionOutcome,
    YoutubeSummaryStageExecutionError,
};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::{
    resolve_profile_for_backend, run_llm_collect_with_profile, LlmChatRequest, LlmMessage,
    LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState,
    ResolvedLlmProfile,
};

pub const PROMPT_PACK_RUN_EVENT: &str = "prompt-pack-run-event";

#[derive(Default)]
pub struct PromptPackRunState {
    active: Mutex<HashSet<i64>>,
    cancel_requested: Mutex<HashSet<i64>>,
}

impl PromptPackRunState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn track(&self, run_id: i64) -> AppResult<()> {
        self.active.lock().await.insert(run_id);
        Ok(())
    }

    pub async fn track_if_absent(&self, run_id: i64) -> AppResult<bool> {
        Ok(self.active.lock().await.insert(run_id))
    }

    pub async fn request_cancel(&self, run_id: i64) -> AppResult<()> {
        self.cancel_requested.lock().await.insert(run_id);
        Ok(())
    }

    pub async fn is_cancel_requested(&self, run_id: i64) -> bool {
        self.cancel_requested.lock().await.contains(&run_id)
    }

    pub async fn finish(&self, run_id: i64) {
        self.active.lock().await.remove(&run_id);
        self.cancel_requested.lock().await.remove(&run_id);
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
}

#[tauri::command]
pub async fn preflight_youtube_summary_run(
    handle: AppHandle,
    project_id: Option<i64>,
    source_ids: Vec<i64>,
    profile_id: Option<String>,
    model_override: Option<String>,
    output_language: String,
    control_preset: String,
    evidence_mode: String,
    include_comments: bool,
) -> AppResult<super::dto::YoutubeSummaryPreflightResponse> {
    let pool = get_pool(&handle).await?;
    preflight_youtube_summary_in_pool(
        &pool,
        super::dto::PreflightYoutubeSummaryRunRequest {
            project_id,
            source_ids,
            profile_id,
            model_override,
            output_language,
            control_preset,
            evidence_mode,
            include_comments,
        },
        ModelBudget {
            input_token_limit: Some(32_000),
        },
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
    output_language: String,
    control_preset: String,
    evidence_mode: String,
    include_comments: bool,
) -> AppResult<StartYoutubeSummaryRunOutcomeDto> {
    let pool = get_pool(&handle).await?;
    let outcome = start_youtube_summary_run_in_pool(
        &pool,
        super::dto::StartYoutubeSummaryRunRequest {
            client_request_id,
            project_id,
            source_ids,
            profile_id,
            model_override,
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
        async move {
            run_transcript_analysis_stage_request(handle, profile, model_override, stage_request)
                .await
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
    stage_request: TranscriptAnalysisStageExecutionRequest,
) -> Result<PromptPackLlmCompletion, YoutubeSummaryStageExecutionError> {
    let llm_request = build_transcript_analysis_llm_request(
        &stage_request,
        Some(profile.profile_id.clone()),
        model_override,
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
                let completion = control
                    .run_cancellable(run_llm_collect_with_profile(
                        &scheduled_request,
                        &scheduled_profile,
                    ))
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

fn build_transcript_analysis_llm_request(
    request: &TranscriptAnalysisStageExecutionRequest,
    profile_id: Option<String>,
    model_override: Option<String>,
) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!(
            "prompt-pack-run-{}-stage-{}",
            request.run_id, request.stage_run_id
        ),
        profile_id,
        model_override,
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
                     \"summary_text\": \"concise summary\",\n\
                     \"segment_candidates\": [],\n\
                     \"key_point_candidates\": [],\n\
                     \"quote_candidates\": [],\n\
                     \"action_item_candidates\": [],\n\
                     \"open_question_candidates\": []\n\
                     }},\n\
                     \"claim_candidates\": [{{ \"text\": \"claim\", \"material_refs\": [\"allowed material ref\"] }}],\n\
                     \"evidence_fragment_candidates\": [{{ \"text\": \"evidence quote or paraphrase\", \"material_refs\": [\"allowed material ref\"] }}],\n\
                     \"warning_candidates\": []\n\
                     }}\n\n\
                     Do not include backend-owned IDs such as claim_id, evidence_id, source_ref_id, segment_id, key_point_id, quote_id, action_item_id, or open_question_id. Use material_refs only from allowed_material_refs in the frozen input. Do not rename fields. Do not wrap the JSON in Markdown.\n\n\
                     Frozen stage input JSON:\n{}",
                    request.prompt_input_json
                ),
            },
        ],
    }
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

pub(crate) async fn list_prompt_pack_runs_in_pool(
    pool: &SqlitePool,
    request: super::dto::ListPromptPackRunsRequest,
) -> AppResult<Vec<PromptPackRunSummaryDto>> {
    let limit = request.limit.unwrap_or(20).clamp(1, 100);
    let rows = if let Some(project_id) = request.project_id {
        sqlx::query_as::<_, RunSummaryRow>(
            "SELECT id, project_id, run_label, pack_id, pack_version, run_status, result_status,
                    created_at, started_at, completed_at, latest_message,
                    progress_current, progress_total, queue_position
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
            "SELECT id, project_id, run_label, pack_id, pack_version, run_status, result_status,
                    created_at, started_at, completed_at, latest_message,
                    progress_current, progress_total, queue_position
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
    let status = sqlx::query_scalar::<_, String>(
        "SELECT run_status FROM prompt_pack_runs WHERE id = ?",
    )
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
        "SELECT id, project_id, run_label, pack_id, pack_version, run_status, result_status,
                created_at, started_at, completed_at, latest_message,
                progress_current, progress_total, queue_position
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
    "2026-06-14T00:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        build_transcript_analysis_llm_request, cleanup_interrupted_prompt_pack_runs_in_pool,
        delete_prompt_pack_run_in_pool, list_prompt_pack_runs_in_pool,
        update_prompt_pack_run_in_pool, PromptPackRunState,
    };
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::prompt_packs::dto::{ListPromptPackRunsRequest, PromptPackRunEvent};
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;
    use crate::prompt_packs::youtube_summary::TranscriptAnalysisStageExecutionRequest;

    #[tokio::test]
    async fn prompt_pack_run_state_tracks_active_and_cancel_requested_runs() {
        let state = PromptPackRunState::new();

        assert!(state.track_if_absent(42).await.expect("first track"));
        assert!(!state.track_if_absent(42).await.expect("duplicate track"));
        state.track(43).await.expect("track second");
        assert!(state.active_run_ids().await.contains(&42));

        state.request_cancel(42).await.expect("cancel");
        assert!(state.is_cancel_requested(42).await);

        state.finish(42).await;
        assert!(!state.active_run_ids().await.contains(&42));
        assert!(state.active_run_ids().await.contains(&43));
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
        let pool = test_pool_with_prompt_pack_runs([(
            41,
            Some(7),
            "complete",
            "2026-06-14T10:00:00Z",
        )])
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
                prompt_input_json: "{\"stage\":\"youtube_summary/transcript_analysis\"}"
                    .to_string(),
            },
            Some("profile-1".to_string()),
            Some("model-1".to_string()),
        );

        assert_eq!(request.request_id, "prompt-pack-run-42-stage-1001");
        assert_eq!(request.profile_id.as_deref(), Some("profile-1"));
        assert_eq!(request.model_override.as_deref(), Some("model-1"));
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
            .contains("Do not include backend-owned IDs"));
        assert!(request.messages[1]
            .content
            .contains("\"stage\":\"youtube_summary/transcript_analysis\""));
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
