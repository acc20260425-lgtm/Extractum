use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::{
    resolve_effective_model, resolve_model_input_token_limit_for_backend,
    resolve_profile_for_backend, run_llm_collect_with_profile, run_llm_stream_with_profile,
    LlmCompletion, LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority,
    LlmSchedulerState, ResolvedLlmProfile,
};

use super::corpus::{
    load_corpus_messages, preflight_analysis_run, preflight_limit_error, resolve_analysis_sources,
    AnalysisRunPreflight, AnalysisRunPreflightLimits, AnalysisSourceResolutionError,
    CorpusLoadRequest, YoutubeCorpusMode,
};
use super::events::emit_analysis_event;
use super::models::{
    AnalysisChunkSummaryEvent, AnalysisPromptTemplate, AnalysisRunEvent, ChunkSummary,
    CorpusMessage,
};
use super::store::{
    capture_run_snapshot, fetch_prompt_template, fetch_run_row, fetch_source_group,
    find_active_duplicate_run, insert_analysis_run, mark_run_capture_failed,
    sanitize_provider_error, sanitize_snapshot_error, set_run_status, AnalysisRunInsert,
    DuplicateRunLookup,
};
use super::trace::{build_trace_data, compress_trace_data};
use super::{
    now_secs, AnalysisState, ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_COMPLETED,
    ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING, TEMPLATE_KIND_REPORT,
};

mod requests;

use self::requests::{
    build_map_request, build_reduce_request, chunk_messages,
    chunk_target_chars_for_model_input_limit, parse_chunk_summary, ReduceRequestParams,
};

const INTERRUPTED_RUN_MESSAGE: &str = "Analysis run was interrupted when the app was restarted.";
const CANCELLED_RUN_MESSAGE: &str = "Analysis run cancelled.";
const SNAPSHOT_CAPTURE_FAILED_MESSAGE: &str = "Snapshot capture failed";

pub(crate) struct StartAnalysisReportRequest {
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: String,
    pub(crate) prompt_template_id: i64,
    pub(crate) model_override: Option<String>,
    pub(crate) profile_id: Option<String>,
    pub(crate) youtube_corpus_mode: Option<String>,
    pub(crate) include_migrated_history: bool,
}

pub(crate) fn resolve_analysis_telegram_history_scope(
    include_migrated_history: bool,
    source_type: &str,
) -> AppResult<(&'static str, bool)> {
    if include_migrated_history && source_type != crate::sources::TELEGRAM_SOURCE_TYPE {
        return Err(AppError::validation(
            "Migrated historical scope can be included only for Telegram analysis",
        ));
    }
    if include_migrated_history {
        return Ok((
            crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED,
            true,
        ));
    }
    Ok((
        crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT,
        false,
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReportRunError {
    Failed(String),
    CaptureFailed(String),
    Cancelled(String),
}

async fn capture_report_corpus(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    request: &CorpusLoadRequest,
) -> Result<Vec<CorpusMessage>, ReportRunError> {
    let corpus = load_corpus_messages(pool, request).await.map_err(|error| {
        ReportRunError::CaptureFailed(sanitize_snapshot_error(
            "Corpus preload failed",
            &error.to_string(),
        ))
    })?;

    if corpus.is_empty() {
        return Err(ReportRunError::CaptureFailed(
            SNAPSHOT_CAPTURE_FAILED_MESSAGE.to_string(),
        ));
    }

    capture_run_snapshot(pool, run_id, scope_label, &corpus)
        .await
        .map_err(|error| {
            ReportRunError::CaptureFailed(sanitize_snapshot_error(
                SNAPSHOT_CAPTURE_FAILED_MESSAGE,
                &error.to_string(),
            ))
        })
}

struct RunEvent {
    event: AnalysisRunEvent,
}

impl RunEvent {
    fn new(run_id: i64, kind: &str, phase: &str) -> Self {
        Self {
            event: AnalysisRunEvent {
                run_id,
                request_id: None,
                kind: kind.to_string(),
                phase: phase.to_string(),
                queue_position: None,
                message: None,
                progress_current: None,
                progress_total: None,
                delta: None,
                chunk_summary: None,
                error: None,
            },
        }
    }

    fn request_id(mut self, request_id: String) -> Self {
        self.event.request_id = Some(request_id);
        self
    }

    fn queue_position(mut self, queue_position: usize) -> Self {
        self.event.queue_position = Some(queue_position);
        self
    }

    fn message(mut self, message: String) -> Self {
        self.event.message = Some(message);
        self
    }

    fn progress(mut self, current: i64, total: i64) -> Self {
        self.event.progress_current = Some(current);
        self.event.progress_total = Some(total);
        self
    }

    fn delta(mut self, delta: String) -> Self {
        self.event.delta = Some(delta);
        self
    }

    fn chunk_summary(mut self, chunk_summary: AnalysisChunkSummaryEvent) -> Self {
        self.event.chunk_summary = Some(chunk_summary);
        self
    }

    fn error(mut self, error: String) -> Self {
        self.event.error = Some(error);
        self
    }

    fn emit(self, handle: &AppHandle) {
        emit_analysis_event(handle, &self.event);
    }
}

fn finish_map_phase(
    ordered_summaries: Vec<Option<ChunkSummary>>,
    first_error: Option<ReportRunError>,
) -> Result<Vec<ChunkSummary>, ReportRunError> {
    if let Some(error) = first_error {
        return Err(error);
    }

    ordered_summaries
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| {
            ReportRunError::Failed("Some chunk summaries were not collected".to_string())
        })
}

struct ReportRunInput {
    run_id: i64,
    scope_label: String,
    corpus_request: CorpusLoadRequest,
    period_from: i64,
    period_to: i64,
    output_language: String,
    prompt_template: AnalysisPromptTemplate,
    model_override: Option<String>,
    resolved_profile: ResolvedLlmProfile,
    chunk_target_chars: usize,
    preflight: AnalysisRunPreflight,
}

fn validate_report_preflight(preflight: &AnalysisRunPreflight) -> AppResult<()> {
    if preflight.message_count == 0 {
        return Err(AppError::validation(
            "No synced source documents were found for the selected analysis scope and period",
        ));
    }

    if let Some(error) = preflight_limit_error(preflight) {
        return Err(AppError::validation(error));
    }

    Ok(())
}

struct ReportPipelineContext {
    handle: AppHandle,
    pool: Pool<Sqlite>,
    resolved_profile: ResolvedLlmProfile,
    run_id: i64,
}

impl ReportPipelineContext {
    async fn ensure_not_cancelled(&self) -> Result<(), ReportRunError> {
        if self
            .handle
            .state::<AnalysisState>()
            .is_report_run_cancelled(self.run_id)
            .await
        {
            return Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()));
        }

        Ok(())
    }

    async fn cancel_children(&self) {
        self.handle
            .state::<LlmSchedulerState>()
            .cancel_run_requests(self.run_id)
            .await;
    }

    fn emit(&self, event: RunEvent) {
        event.emit(&self.handle);
    }
}

struct ReducePhaseResult {
    request_id: String,
    completion: LlmCompletion,
}

async fn run_analysis_step_with_cancel<Fut, T>(
    cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> Result<T, LlmRequestError>
where
    Fut: Future<Output = Result<T, LlmRequestError>>,
{
    let Some(cancellation_token) = cancellation_token else {
        return future.await;
    };

    if cancellation_token.is_cancelled() {
        return Err(LlmRequestError::Cancelled);
    }

    tokio::select! {
        result = future => result,
        _ = cancellation_token.cancelled() => Err(LlmRequestError::Cancelled),
    }
}

async fn run_map_phase(
    ctx: &ReportPipelineContext,
    chunks: Vec<Vec<CorpusMessage>>,
) -> Result<Vec<ChunkSummary>, ReportRunError> {
    ctx.emit(
        RunEvent::new(ctx.run_id, "progress", "map")
            .message(format!(
                "Dispatching {} chunk analysis request{}...",
                chunks.len(),
                if chunks.len() == 1 { "" } else { "s" }
            ))
            .progress(0, chunks.len() as i64),
    );

    let completed_chunks = Arc::new(AtomicUsize::new(0));
    let mut join_set = JoinSet::new();
    let total_chunks = chunks.len();
    for (index, chunk) in chunks.into_iter().enumerate() {
        let task_handle = ctx.handle.clone();
        let task_profile = ctx.resolved_profile.clone();
        let task_profile_id = ctx.resolved_profile.profile_id.clone();
        let chunk_request =
            build_map_request(ctx.run_id, task_profile_id, index + 1, total_chunks, &chunk);
        let chunk_request_id = chunk_request.request_id.clone();
        let chunk_provider = task_profile.provider.as_str().to_string();
        let chunk_counter = completed_chunks.clone();
        let chunk_message_count = chunk.len() as i64;
        let run_id = ctx.run_id;
        let cancellation_token = ctx
            .handle
            .state::<AnalysisState>()
            .report_run_child_token(ctx.run_id)
            .await;

        join_set.spawn(async move {
            let scheduler = task_handle.state::<LlmSchedulerState>();
            let request_meta = LlmRequestMetadata {
                request_id: chunk_request.request_id.clone(),
                profile_id: task_profile.profile_id.clone(),
                provider: chunk_provider.clone(),
                kind: LlmRequestKind::AnalysisReportMap,
                priority: LlmRequestPriority::Background,
                owner_run_id: Some(run_id),
            };
            let queued_handle = task_handle.clone();
            let started_handle = task_handle.clone();
            let failed_handle = task_handle.clone();
            let cancelled_handle = task_handle.clone();
            let queued_counter = chunk_counter.clone();
            let started_counter = chunk_counter.clone();
            let queued_request_id = chunk_request_id.clone();
            let started_request_id = chunk_request_id.clone();
            let failed_request_id = chunk_request_id.clone();
            let cancelled_request_id = chunk_request_id.clone();
            let scheduled_request = chunk_request.clone();
            let scheduled_profile = task_profile.clone();
            let step_cancellation_token = cancellation_token.clone();

            match scheduler
                .run_request(
                    request_meta,
                    move |position| {
                        RunEvent::new(run_id, "queued", "map")
                            .request_id(queued_request_id.clone())
                            .queue_position(position)
                            .message(format!(
                                "Chunk {} of {} queued at position {}...",
                                index + 1,
                                total_chunks,
                                position
                            ))
                            .progress(
                                queued_counter.load(Ordering::SeqCst) as i64,
                                total_chunks as i64,
                            )
                            .emit(&queued_handle);
                    },
                    move |control| async move {
                        RunEvent::new(run_id, "started", "map")
                            .request_id(started_request_id)
                            .message(format!(
                                "Analyzing chunk {} of {}...",
                                index + 1,
                                total_chunks
                            ))
                            .progress(
                                started_counter.load(Ordering::SeqCst) as i64,
                                total_chunks as i64,
                            )
                            .emit(&started_handle);

                        run_analysis_step_with_cancel(
                            step_cancellation_token,
                            control.run_cancellable(run_llm_collect_with_profile(
                                &scheduled_request,
                                &scheduled_profile,
                            )),
                        )
                        .await
                    },
                )
                .await
            {
                Ok(completion) => {
                    let summary =
                        parse_chunk_summary(&completion.text).map_err(ReportRunError::Failed)?;
                    let completed = chunk_counter.fetch_add(1, Ordering::SeqCst) + 1;
                    RunEvent::new(run_id, "progress", "map")
                        .request_id(chunk_request_id.clone())
                        .message(format!(
                            "Chunk {} of {} summarized.",
                            index + 1,
                            total_chunks
                        ))
                        .progress(completed as i64, total_chunks as i64)
                        .chunk_summary(AnalysisChunkSummaryEvent {
                            index: (index + 1) as i64,
                            total: total_chunks as i64,
                            message_count: chunk_message_count,
                            summary: summary.summary.clone(),
                            topics: summary.topics.clone(),
                            notable_points: summary.notable_points.clone(),
                            candidate_refs: summary.candidate_refs.clone(),
                        })
                        .emit(&task_handle);
                    Ok::<(usize, ChunkSummary), ReportRunError>((index, summary))
                }
                Err(LlmRequestError::Failed(error)) => {
                    let error = error.to_string();
                    RunEvent::new(run_id, "failed", "map")
                        .request_id(failed_request_id)
                        .message(format!("Chunk {} of {} failed.", index + 1, total_chunks))
                        .progress(
                            chunk_counter.load(Ordering::SeqCst) as i64,
                            total_chunks as i64,
                        )
                        .error(error.clone())
                        .emit(&failed_handle);
                    Err(ReportRunError::Failed(error))
                }
                Err(LlmRequestError::Cancelled) => {
                    RunEvent::new(run_id, "cancelled", "map")
                        .request_id(cancelled_request_id)
                        .message(format!(
                            "Chunk {} of {} cancelled.",
                            index + 1,
                            total_chunks
                        ))
                        .progress(
                            chunk_counter.load(Ordering::SeqCst) as i64,
                            total_chunks as i64,
                        )
                        .emit(&cancelled_handle);
                    Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()))
                }
            }
        });
    }

    let mut ordered_summaries = vec![None; total_chunks];
    let mut first_error: Option<ReportRunError> = None;
    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok(Ok((index, summary))) => {
                ordered_summaries[index] = Some(summary);
            }
            Ok(Err(error)) => {
                if first_error.is_none() {
                    first_error = Some(error.clone());
                    ctx.cancel_children().await;
                }
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(ReportRunError::Failed(format!(
                        "Chunk worker crashed: {error}"
                    )));
                    ctx.cancel_children().await;
                }
            }
        }
    }

    finish_map_phase(ordered_summaries, first_error)
}

async fn run_reduce_phase(
    ctx: &ReportPipelineContext,
    input: &ReportRunInput,
    chunk_summaries: &[ChunkSummary],
) -> Result<ReducePhaseResult, ReportRunError> {
    ctx.emit(
        RunEvent::new(ctx.run_id, "progress", "reduce")
            .message("Writing final report...".to_string()),
    );

    let reduce_request = build_reduce_request(ReduceRequestParams {
        run_id: ctx.run_id,
        profile_id: ctx.resolved_profile.profile_id.clone(),
        scope_label: &input.scope_label,
        output_language: &input.output_language,
        prompt_template: &input.prompt_template,
        period_from: input.period_from,
        period_to: input.period_to,
        chunk_summaries,
        model_override: input.model_override.clone(),
    });
    let reduce_request_id = reduce_request.request_id.clone();
    let reduce_provider = ctx.resolved_profile.provider.as_str().to_string();
    let scheduler = ctx.handle.state::<LlmSchedulerState>();
    let queued_handle = ctx.handle.clone();
    let started_handle = ctx.handle.clone();
    let delta_handle = ctx.handle.clone();
    let failed_handle = ctx.handle.clone();
    let cancelled_handle = ctx.handle.clone();
    let queued_request_id = reduce_request_id.clone();
    let started_request_id = reduce_request_id.clone();
    let delta_request_id = reduce_request_id.clone();
    let failed_request_id = reduce_request_id.clone();
    let cancelled_request_id = reduce_request_id.clone();
    let reduce_request_for_stream = reduce_request.clone();
    let reduce_profile = ctx.resolved_profile.clone();
    let run_id = ctx.run_id;
    let cancellation_token = ctx
        .handle
        .state::<AnalysisState>()
        .report_run_child_token(ctx.run_id)
        .await;
    let completion = match scheduler
        .run_request(
            LlmRequestMetadata {
                request_id: reduce_request.request_id.clone(),
                profile_id: ctx.resolved_profile.profile_id.clone(),
                provider: reduce_provider.clone(),
                kind: LlmRequestKind::AnalysisReportReduce,
                priority: LlmRequestPriority::Background,
                owner_run_id: Some(ctx.run_id),
            },
            move |position| {
                RunEvent::new(run_id, "queued", "reduce")
                    .request_id(queued_request_id.clone())
                    .queue_position(position)
                    .message(format!("Final report queued at position {}...", position))
                    .emit(&queued_handle);
            },
            move |control| async move {
                RunEvent::new(run_id, "started", "reduce")
                    .request_id(started_request_id)
                    .message("Writing final report...".to_string())
                    .emit(&started_handle);

                run_analysis_step_with_cancel(
                    cancellation_token,
                    control.run_cancellable(run_llm_stream_with_profile(
                        &reduce_request_for_stream,
                        &reduce_profile,
                        |delta| {
                            RunEvent::new(run_id, "delta", "reduce")
                                .request_id(delta_request_id.clone())
                                .delta(delta.to_string())
                                .emit(&delta_handle);
                        },
                    )),
                )
                .await
            },
        )
        .await
    {
        Ok(completion) => completion,
        Err(LlmRequestError::Failed(error)) => {
            let error = error.to_string();
            RunEvent::new(run_id, "failed", "reduce")
                .request_id(failed_request_id)
                .message("Final report generation failed.".to_string())
                .error(error.clone())
                .emit(&failed_handle);
            return Err(ReportRunError::Failed(error));
        }
        Err(LlmRequestError::Cancelled) => {
            RunEvent::new(run_id, "cancelled", "reduce")
                .request_id(cancelled_request_id)
                .message("Final report generation cancelled.".to_string())
                .emit(&cancelled_handle);
            return Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()));
        }
    };

    Ok(ReducePhaseResult {
        request_id: reduce_request_id,
        completion,
    })
}

async fn run_report_pipeline(
    handle: AppHandle,
    input: ReportRunInput,
) -> Result<(), ReportRunError> {
    let run_id = input.run_id;

    if handle
        .state::<AnalysisState>()
        .is_report_run_cancelled(run_id)
        .await
    {
        return Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()));
    }

    let pool = get_pool(&handle)
        .await
        .map_err(|error| ReportRunError::Failed(error.to_string()))?;
    set_run_status(
        &pool,
        run_id,
        ANALYSIS_STATUS_RUNNING,
        None,
        None,
        None,
        None,
    )
    .await
    .map_err(|error| ReportRunError::Failed(error.to_string()))?;

    RunEvent::new(run_id, "started", "load_items")
        .message(format!(
            "Preflight passed: {} documents, {} estimated chunks, {} estimated input characters.",
            input.preflight.message_count,
            input.preflight.estimated_chunks,
            input.preflight.estimated_input_chars
        ))
        .emit(&handle);

    let corpus =
        capture_report_corpus(&pool, run_id, &input.scope_label, &input.corpus_request).await?;
    if handle
        .state::<AnalysisState>()
        .is_report_run_cancelled(run_id)
        .await
    {
        return Err(ReportRunError::Cancelled(CANCELLED_RUN_MESSAGE.to_string()));
    }

    RunEvent::new(run_id, "progress", "chunking")
        .message(format!(
            "Loaded {} source documents. Preparing chunks...",
            corpus.len()
        ))
        .emit(&handle);

    let chunks = chunk_messages(&corpus, input.chunk_target_chars);
    let ctx = ReportPipelineContext {
        handle,
        pool,
        resolved_profile: input.resolved_profile.clone(),
        run_id,
    };

    ctx.ensure_not_cancelled().await?;
    let chunk_summaries = run_map_phase(&ctx, chunks).await?;
    ctx.ensure_not_cancelled().await?;

    let reduce_result = run_reduce_phase(&ctx, &input, &chunk_summaries).await?;
    ctx.ensure_not_cancelled().await?;
    let trace_data = build_trace_data(&reduce_result.completion.text, &corpus);
    let compressed_trace = compress_trace_data(&trace_data)
        .map_err(|error| ReportRunError::Failed(error.to_string()))?;

    ctx.emit(
        RunEvent::new(run_id, "progress", "persist")
            .request_id(reduce_result.request_id.clone())
            .message("Saving report...".to_string()),
    );

    set_run_status(
        &ctx.pool,
        run_id,
        ANALYSIS_STATUS_COMPLETED,
        Some(&reduce_result.completion.text),
        Some(&compressed_trace),
        None,
        Some(now_secs()),
    )
    .await
    .map_err(|error| ReportRunError::Failed(error.to_string()))?;

    ctx.emit(
        RunEvent::new(run_id, "completed", "persist")
            .request_id(reduce_result.request_id)
            .message(format!(
                "Report completed with {} cited references.",
                trace_data.refs.len()
            )),
    );

    Ok(())
}

async fn fail_run(handle: &AppHandle, run_id: i64, error: String) {
    let sanitized_error = sanitize_provider_error("Report run failed", &error);
    if let Ok(pool) = get_pool(handle).await {
        let _ = set_run_status(
            &pool,
            run_id,
            ANALYSIS_STATUS_FAILED,
            None,
            None,
            Some(&sanitized_error),
            Some(now_secs()),
        )
        .await;
    }

    RunEvent::new(run_id, "failed", "persist")
        .message("Report run failed.".to_string())
        .error(sanitized_error)
        .emit(handle);
}

async fn fail_capture_run(handle: &AppHandle, run_id: i64, error: String) {
    if let Ok(pool) = get_pool(handle).await {
        let _ = mark_run_capture_failed(&pool, run_id, &error, now_secs()).await;
    }

    RunEvent::new(run_id, "failed", "persist")
        .message("Report run failed before snapshot capture completed.".to_string())
        .error(error)
        .emit(handle);
}

async fn cancel_run(handle: &AppHandle, run_id: i64, message: String) {
    if let Ok(pool) = get_pool(handle).await {
        let _ = set_run_status(
            &pool,
            run_id,
            ANALYSIS_STATUS_CANCELLED,
            None,
            None,
            Some(&message),
            Some(now_secs()),
        )
        .await;
    }

    RunEvent::new(run_id, "cancelled", "persist")
        .message(message)
        .emit(handle);
}

pub(crate) async fn mark_interrupted_analysis_runs(pool: &Pool<Sqlite>) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET status = ?, error = ?, completed_at = ?
        WHERE status IN (?, ?)
        "#,
    )
    .bind(ANALYSIS_STATUS_CANCELLED)
    .bind(INTERRUPTED_RUN_MESSAGE)
    .bind(now_secs())
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(ANALYSIS_STATUS_RUNNING)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub async fn cleanup_interrupted_analysis_runs(handle: AppHandle) {
    if let Ok(pool) = get_pool(&handle).await {
        let _ = mark_interrupted_analysis_runs(&pool).await;
    }
}

async fn request_analysis_run_cancel_for_pool(
    pool: &Pool<Sqlite>,
    state: &AnalysisState,
    scheduler: &LlmSchedulerState,
    run_id: i64,
) -> AppResult<String> {
    let run = fetch_run_row(pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;

    if run.status != ANALYSIS_STATUS_QUEUED && run.status != ANALYSIS_STATUS_RUNNING {
        return Err(AppError::conflict(format!(
            "Analysis run {run_id} is not queued or running"
        )));
    }

    let requested = state.request_report_run_cancel(run_id).await;
    let cancelled_requests = scheduler.cancel_run_requests(run_id).await;
    if !requested && cancelled_requests == 0 {
        return Err(AppError::conflict(format!(
            "Analysis run {run_id} is no longer active"
        )));
    }

    Ok(run.status)
}

pub(crate) async fn request_analysis_run_cancel(
    handle: &AppHandle,
    state: &AnalysisState,
    scheduler: &LlmSchedulerState,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(handle).await?;
    let status = request_analysis_run_cancel_for_pool(&pool, state, scheduler, run_id).await?;

    RunEvent::new(run_id, "progress", &status)
        .message("Cancelling analysis run...".to_string())
        .emit(handle);

    Ok(())
}

pub(crate) async fn start_analysis_report_run(
    handle: AppHandle,
    state: &AnalysisState,
    request: StartAnalysisReportRequest,
) -> AppResult<i64> {
    let StartAnalysisReportRequest {
        source_id,
        source_group_id,
        project_id,
        period_from,
        period_to,
        output_language,
        prompt_template_id,
        model_override,
        profile_id,
        youtube_corpus_mode,
        include_migrated_history,
    } = request;

    if period_from > period_to {
        return Err(AppError::validation(
            "period_from must be less than or equal to period_to",
        ));
    }

    let output_language = output_language.trim().to_string();
    if output_language.is_empty() {
        return Err(AppError::validation("Output language cannot be empty"));
    }

    let selected_count = [
        source_id.is_some(),
        source_group_id.is_some(),
        project_id.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if selected_count != 1 {
        return Err(AppError::validation("Select exactly one analysis scope"));
    }

    let pool = get_pool(&handle).await?;
    let prompt_template = fetch_prompt_template(&pool, prompt_template_id).await?;
    if prompt_template.template_kind != TEMPLATE_KIND_REPORT {
        return Err(AppError::validation(
            "Selected prompt template is not a report template",
        ));
    }

    let resolved_profile = resolve_profile_for_backend(&handle, profile_id.as_deref()).await?;
    let effective_model = resolve_effective_model(&resolved_profile, model_override.as_deref())?;
    let model_input_token_limit =
        resolve_model_input_token_limit_for_backend(&resolved_profile, &effective_model).await;
    let chunk_target_chars = chunk_target_chars_for_model_input_limit(model_input_token_limit);
    let youtube_corpus_mode = YoutubeCorpusMode::from_wire(youtube_corpus_mode.as_deref())
        .map_err(AppError::validation)?;

    let (scope_type, resolved_source_id, resolved_group_id, resolved_project_id, scope_label) =
        if let Some(source_id) = source_id {
            let source_exists =
                sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM sources WHERE id = ?)")
                    .bind(source_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(AppError::database)?;
            if source_exists == 0 {
                return Err(AppError::not_found(format!("Source {source_id} not found")));
            }

            let source_title =
                sqlx::query_scalar::<_, Option<String>>("SELECT title FROM sources WHERE id = ?")
                    .bind(source_id)
                    .fetch_optional(&pool)
                    .await
                    .map_err(AppError::database)?
                    .flatten()
                    .filter(|title| !title.trim().is_empty())
                    .unwrap_or_else(|| format!("Source {source_id}"));

            (
                ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
                Some(source_id),
                None,
                None,
                source_title,
            )
        } else if let Some(group_id) = source_group_id {
            let group = fetch_source_group(&pool, group_id).await?.ok_or_else(|| {
                AppError::not_found(format!("Analysis source group {group_id} not found"))
            })?;

            if group.members.is_empty() {
                return Err(AppError::validation(
                    "The selected source group does not contain any sources",
                ));
            }

            (
                ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
                None,
                Some(group.id),
                None,
                group.name.clone(),
            )
        } else {
            let project_id = project_id.expect("validated project_id");
            let project = crate::projects::get_project_in_pool(&pool, project_id)
                .await?
                .ok_or_else(|| AppError::not_found(format!("Project {project_id} not found")))?;
            let source_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM project_sources WHERE project_id = ?")
                    .bind(project_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(AppError::database)?;
            if source_count == 0 {
                return Err(AppError::validation("Project does not contain any sources"));
            }

            (
                ANALYSIS_SCOPE_TYPE_PROJECT,
                None,
                None,
                Some(project.id),
                project.name.clone(),
            )
        };

    let resolved_sources = resolve_analysis_sources(
        &pool,
        resolved_source_id,
        resolved_group_id,
        resolved_project_id,
    )
    .await
    .map_err(AnalysisSourceResolutionError::into_app_error)?;
    let (telegram_history_scope, include_migrated_history) =
        resolve_analysis_telegram_history_scope(
            include_migrated_history,
            &resolved_sources.source_type,
        )?;
    let corpus_request = CorpusLoadRequest {
        source_type: resolved_sources.source_type.clone(),
        source_ids: resolved_sources.source_ids.clone(),
        period_from,
        period_to,
        youtube_corpus_mode,
        include_migrated_history,
    };

    let preflight = preflight_analysis_run(
        &pool,
        &corpus_request,
        chunk_target_chars,
        AnalysisRunPreflightLimits::default(),
    )
    .await?;

    validate_report_preflight(&preflight)?;

    if let Some(existing_run_id) = find_active_duplicate_run(
        &pool,
        &DuplicateRunLookup {
            scope_type,
            source_id: resolved_source_id,
            source_group_id: resolved_group_id,
            project_id: resolved_project_id,
            period_from,
            period_to,
            output_language: &output_language,
            prompt_template_id: prompt_template.id,
            provider_profile: &resolved_profile.profile_id,
            model: &effective_model,
            youtube_corpus_mode,
            telegram_history_scope,
        },
    )
    .await?
    {
        let active_run_ids = state.active_report_run_ids().await;
        if active_run_ids.contains(&existing_run_id) {
            return Err(AppError::conflict(format!(
                "An identical analysis report is already queued or running (run {existing_run_id})"
            )));
        }

        set_run_status(
            &pool,
            existing_run_id,
            ANALYSIS_STATUS_CANCELLED,
            None,
            None,
            Some(INTERRUPTED_RUN_MESSAGE),
            Some(now_secs()),
        )
        .await?;
    }

    let run_id = insert_analysis_run(
        &pool,
        &AnalysisRunInsert {
            scope_type,
            source_id: resolved_source_id,
            source_group_id: resolved_group_id,
            project_id: resolved_project_id,
            period_from,
            period_to,
            output_language: &output_language,
            prompt_template: &prompt_template,
            provider_profile: &resolved_profile.profile_id,
            provider: resolved_profile.provider.as_str(),
            model: &effective_model,
            youtube_corpus_mode,
            telegram_history_scope,
            scope_label_snapshot: Some(&scope_label),
        },
    )
    .await?;

    state.insert_active_report_run(run_id).await;

    let app_handle = handle.clone();
    tokio::spawn(async move {
        match run_report_pipeline(
            app_handle.clone(),
            ReportRunInput {
                run_id,
                scope_label,
                corpus_request,
                period_from,
                period_to,
                output_language,
                prompt_template,
                model_override,
                resolved_profile,
                chunk_target_chars,
                preflight,
            },
        )
        .await
        {
            Ok(()) => {}
            Err(ReportRunError::Failed(error)) => fail_run(&app_handle, run_id, error).await,
            Err(ReportRunError::CaptureFailed(error)) => {
                fail_capture_run(&app_handle, run_id, error).await
            }
            Err(ReportRunError::Cancelled(message)) => {
                cancel_run(&app_handle, run_id, message).await
            }
        }
        app_handle
            .state::<AnalysisState>()
            .remove_active_report_run(run_id)
            .await;
    });

    Ok(run_id)
}

#[cfg(test)]
mod tests {
    use super::requests::extract_json_payload;
    use super::{
        build_map_request, build_reduce_request, capture_report_corpus,
        chunk_target_chars_for_model_input_limit, finish_map_phase, mark_interrupted_analysis_runs,
        parse_chunk_summary, request_analysis_run_cancel_for_pool,
        resolve_analysis_telegram_history_scope, run_analysis_step_with_cancel,
        validate_report_preflight, ReduceRequestParams, ReportRunError, ReportRunInput,
        StartAnalysisReportRequest,
    };
    use crate::analysis::corpus::{
        AnalysisRunPreflight, AnalysisRunPreflightLimits, CorpusLoadRequest, YoutubeCorpusMode,
    };
    use crate::analysis::models::{AnalysisPromptTemplate, ChunkSummary, CorpusMessage};
    use crate::error::AppErrorKind;
    use crate::llm::{LlmRequestError, LlmSchedulerState, ProviderKind, ResolvedLlmProfile};
    use sqlx::SqlitePool;
    use tokio_util::sync::CancellationToken;

    const SAMPLE_JSON: &str = r#"{"summary":"Brief","topics":["sync"],"notable_points":["Point"],"candidate_refs":["s1-i2"]}"#;

    fn sample_chunk_summary(label: &str) -> ChunkSummary {
        ChunkSummary {
            summary: label.to_string(),
            topics: vec![format!("{label}-topic")],
            notable_points: vec![format!("{label}-point")],
            candidate_refs: vec![format!("{label}-ref")],
        }
    }

    fn sample_prompt_template() -> AnalysisPromptTemplate {
        AnalysisPromptTemplate {
            id: 7,
            name: "Report".to_string(),
            template_kind: "report".to_string(),
            body: "Write a concise report.".to_string(),
            version: 3,
            is_builtin: false,
            created_at: 1,
            updated_at: 1,
        }
    }

    fn sample_corpus_message() -> CorpusMessage {
        CorpusMessage {
            item_id: 1,
            source_id: 2,
            external_id: "42".to_string(),
            published_at: 1_700_000_000,
            author: Some("analyst".to_string()),
            content: "Important update from the source".to_string(),
            r#ref: "s2-i1".to_string(),
            item_kind: Some("telegram_message".to_string()),
            source_type: Some("telegram".to_string()),
            source_subtype: Some("channel".to_string()),
            metadata_zstd: None,
        }
    }

    fn sample_resolved_profile() -> ResolvedLlmProfile {
        ResolvedLlmProfile {
            profile_id: "research".to_string(),
            provider: ProviderKind::Gemini,
            default_model: "gemini-2.5-flash".to_string(),
            api_key: "secret-key".to_string().into(),
            base_url: String::new(),
        }
    }

    async fn request_cancel_pool_with_runs() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        sqlx::query(
            "CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY,
                run_type TEXT NOT NULL DEFAULT 'report',
                scope_type TEXT NOT NULL DEFAULT 'single_source',
                source_id INTEGER,
                source_group_id INTEGER,
                project_id INTEGER,
                period_from INTEGER NOT NULL DEFAULT 0,
                period_to INTEGER NOT NULL DEFAULT 0,
                output_language TEXT NOT NULL DEFAULT 'English',
                prompt_template_id INTEGER NOT NULL DEFAULT 1,
                prompt_template_version INTEGER NOT NULL DEFAULT 1,
                provider_profile TEXT NOT NULL DEFAULT 'research',
                provider TEXT NOT NULL DEFAULT 'gemini',
                model TEXT NOT NULL DEFAULT 'gemini-2.5-flash',
                youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
                telegram_history_scope TEXT,
                status TEXT NOT NULL,
                result_markdown TEXT,
                trace_data_zstd BLOB,
                scope_label_snapshot TEXT,
                snapshot_captured_at TEXT,
                snapshot_error TEXT,
                error TEXT,
                created_at INTEGER NOT NULL DEFAULT 1,
                completed_at INTEGER
            )",
        )
        .execute(&pool)
        .await
        .expect("create analysis_runs");

        sqlx::query("CREATE TABLE sources (id INTEGER PRIMARY KEY, title TEXT)")
            .execute(&pool)
            .await
            .expect("create sources");
        sqlx::query("CREATE TABLE analysis_source_groups (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&pool)
            .await
            .expect("create groups");
        sqlx::query("CREATE TABLE projects (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&pool)
            .await
            .expect("create projects");
        sqlx::query("CREATE TABLE analysis_prompt_templates (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&pool)
            .await
            .expect("create templates");
        sqlx::query("CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL)")
            .execute(&pool)
            .await
            .expect("create run messages");

        pool
    }

    async fn insert_cancel_request_run(pool: &SqlitePool, run_id: i64, status: &str) {
        sqlx::query(
            "INSERT INTO analysis_runs (
                id, run_type, scope_type, status, period_from, period_to, output_language,
                prompt_template_id, prompt_template_version, provider_profile, provider, model,
                youtube_corpus_mode, created_at
            ) VALUES (
                ?, 'report', 'single_source', ?, 1, 2, 'English', 1, 1,
                'research', 'gemini', 'gemini-2.5-flash', 'transcript_description', 1
            )",
        )
        .bind(run_id)
        .bind(status)
        .execute(pool)
        .await
        .expect("insert analysis run");
    }

    #[test]
    fn report_run_input_carries_resolved_profile_snapshot() {
        let input = ReportRunInput {
            run_id: 9,
            scope_label: "Source".to_string(),
            corpus_request: CorpusLoadRequest {
                source_type: crate::sources::TELEGRAM_SOURCE_TYPE.to_string(),
                source_ids: vec![2],
                period_from: 10,
                period_to: 20,
                youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
                include_migrated_history: false,
            },
            period_from: 10,
            period_to: 20,
            output_language: "English".to_string(),
            prompt_template: sample_prompt_template(),
            model_override: Some("gemini-2.5-pro".to_string()),
            resolved_profile: sample_resolved_profile(),
            chunk_target_chars: 16_000,
            preflight: AnalysisRunPreflight {
                source_ids: vec![2],
                message_count: 1,
                estimated_input_chars: 500,
                estimated_chunks: 1,
                limits: AnalysisRunPreflightLimits::default(),
            },
        };

        assert_eq!(input.resolved_profile.profile_id, "research");
        assert_eq!(input.resolved_profile.default_model, "gemini-2.5-flash");
    }

    #[test]
    fn telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match() {
        let (scope, include_migrated_history) =
            resolve_analysis_telegram_history_scope(true, "telegram")
                .expect("resolve Telegram opt-in");

        assert!(include_migrated_history);
        assert_eq!(
            scope,
            crate::sources::ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED
        );
    }

    #[test]
    fn migrated_history_opt_in_rejects_non_telegram_analysis() {
        let error = resolve_analysis_telegram_history_scope(true, "youtube")
            .expect_err("reject non-Telegram opt-in");
        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    }

    #[test]
    fn report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape() {
        let request = StartAnalysisReportRequest {
            source_id: Some(1),
            source_group_id: None,
            project_id: None,
            period_from: 1,
            period_to: 2,
            output_language: "Russian".to_string(),
            prompt_template_id: 1,
            model_override: None,
            profile_id: None,
            youtube_corpus_mode: None,
            include_migrated_history: true,
        };

        assert!(request.include_migrated_history);
    }

    #[test]
    fn chunk_target_chars_are_derived_from_model_input_limit_with_fallback() {
        assert_eq!(chunk_target_chars_for_model_input_limit(None), 16_000);
        assert_eq!(
            chunk_target_chars_for_model_input_limit(Some(8_192)),
            11_259
        );
        assert!(chunk_target_chars_for_model_input_limit(Some(32_768)) > 16_000);
    }

    #[tokio::test]
    async fn capture_report_corpus_returns_reloaded_snapshot_before_provider_phases() {
        let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;
        crate::sources::test_support::create_analysis_documents_table(&pool).await;
        crate::sources::test_support::create_youtube_typed_source_tables(&pool).await;
        sqlx::query(
            "CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY,
                scope_label_snapshot TEXT,
                snapshot_captured_at TEXT,
                snapshot_error TEXT
            )",
        )
        .execute(&pool)
        .await
        .expect("create runs");
        sqlx::query(
            "CREATE TABLE analysis_run_messages (
                run_id INTEGER NOT NULL,
                item_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                external_id TEXT NOT NULL,
                author TEXT,
                published_at INTEGER NOT NULL,
                ref TEXT NOT NULL,
                content_zstd BLOB NOT NULL,
                item_kind TEXT,
                source_type TEXT,
                source_subtype TEXT,
                metadata_zstd BLOB,
                PRIMARY KEY (run_id, ref)
            )",
        )
        .execute(&pool)
        .await
        .expect("create run messages");
        sqlx::query("INSERT INTO analysis_runs (id) VALUES (1)")
            .execute(&pool)
            .await
            .expect("insert run");
        sqlx::query(
            "CREATE TABLE youtube_transcript_segments (
                item_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                segment_index INTEGER NOT NULL,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER,
                text TEXT NOT NULL,
                chapter_index INTEGER,
                caption_language TEXT,
                caption_track_kind TEXT,
                is_auto_generated INTEGER NOT NULL DEFAULT 0,
                metadata_zstd BLOB,
                UNIQUE(item_id, segment_index)
            )",
        )
        .execute(&pool)
        .await
        .expect("create youtube transcript segments");
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
             VALUES (2, 'telegram', 'channel', 'tg2', 'Telegram 2', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, ingested_at, content_kind, has_media, content_zstd)
             VALUES (10, 2, '10', 'telegram_message', 'Alice', 100, 100, 'text_only', 0, ?)",
        )
        .bind(crate::compression::compress_text("captured text").expect("compress"))
        .execute(&pool)
        .await
        .expect("insert item");
        crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 2)
            .await
            .expect("rebuild docs");

        let request = CorpusLoadRequest {
            source_type: "telegram".to_string(),
            source_ids: vec![2],
            period_from: 1,
            period_to: 1_000,
            youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescription,
            include_migrated_history: false,
        };

        let captured = capture_report_corpus(&pool, 1, "Frozen source", &request)
            .await
            .expect("capture report corpus");

        sqlx::query("DELETE FROM analysis_documents WHERE source_id = 2")
            .execute(&pool)
            .await
            .expect("delete live docs after capture");

        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].content, "captured text");
    }

    #[tokio::test]
    async fn interrupted_cleanup_preserves_captured_snapshot_state_marker() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            "CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY,
                status TEXT NOT NULL,
                error TEXT,
                completed_at INTEGER,
                snapshot_captured_at TEXT,
                snapshot_error TEXT
            )",
        )
        .execute(&pool)
        .await
        .expect("create runs");
        sqlx::query(
            "INSERT INTO analysis_runs (id, status, snapshot_captured_at, snapshot_error)
             VALUES (1, 'running', '2026-05-18T10:00:00Z', NULL)",
        )
        .execute(&pool)
        .await
        .expect("insert running captured run");

        mark_interrupted_analysis_runs(&pool)
            .await
            .expect("mark interrupted");

        let row: (String, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT status, snapshot_captured_at, snapshot_error FROM analysis_runs WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("load run");

        assert_eq!(row.0, crate::analysis::ANALYSIS_STATUS_CANCELLED);
        assert_eq!(row.1.as_deref(), Some("2026-05-18T10:00:00Z"));
        assert_eq!(row.2, None);
    }

    #[tokio::test]
    async fn request_analysis_run_cancel_missing_run_keeps_not_found_message() {
        let pool = request_cancel_pool_with_runs().await;
        let state = crate::analysis::AnalysisState::new();
        let scheduler = LlmSchedulerState::new();
        let run_id = 404;

        let error = request_analysis_run_cancel_for_pool(&pool, &state, &scheduler, run_id)
            .await
            .expect_err("missing run should fail");

        assert_eq!(error.kind, AppErrorKind::NotFound);
        assert_eq!(error.message, format!("Analysis run {run_id} not found"));
    }

    #[tokio::test]
    async fn request_analysis_run_cancel_completed_run_keeps_conflict_message() {
        let pool = request_cancel_pool_with_runs().await;
        insert_cancel_request_run(&pool, 405, crate::analysis::ANALYSIS_STATUS_COMPLETED).await;
        let state = crate::analysis::AnalysisState::new();
        let scheduler = LlmSchedulerState::new();
        let run_id = 405;

        let error = request_analysis_run_cancel_for_pool(&pool, &state, &scheduler, run_id)
            .await
            .expect_err("completed run should fail");

        assert_eq!(error.kind, AppErrorKind::Conflict);
        assert_eq!(
            error.message,
            format!("Analysis run {run_id} is not queued or running")
        );
    }

    #[tokio::test]
    async fn request_analysis_run_cancel_running_but_inactive_keeps_conflict_message() {
        let pool = request_cancel_pool_with_runs().await;
        insert_cancel_request_run(&pool, 406, crate::analysis::ANALYSIS_STATUS_RUNNING).await;
        let state = crate::analysis::AnalysisState::new();
        let scheduler = LlmSchedulerState::new();
        let run_id = 406;

        let error = request_analysis_run_cancel_for_pool(&pool, &state, &scheduler, run_id)
            .await
            .expect_err("inactive running run should fail");

        assert_eq!(error.kind, AppErrorKind::Conflict);
        assert_eq!(
            error.message,
            format!("Analysis run {run_id} is no longer active")
        );
    }

    #[test]
    fn extracts_json_with_text_before_and_after() {
        let response = format!("Preface\n{SAMPLE_JSON}\nTail");
        let payload = extract_json_payload(&response).expect("extract payload");

        assert_eq!(payload, SAMPLE_JSON);
    }

    #[test]
    fn extracts_json_inside_markdown_fence() {
        let response = format!("```json\n{SAMPLE_JSON}\n```");
        let payload = extract_json_payload(&response).expect("extract fenced payload");

        assert_eq!(payload, SAMPLE_JSON);
    }

    #[tokio::test]
    async fn analysis_step_cancel_wrapper_allows_completed_future() {
        let result =
            run_analysis_step_with_cancel(None, async { Ok::<_, LlmRequestError>("done") })
                .await
                .expect("step result");

        assert_eq!(result, "done");
    }

    #[tokio::test]
    async fn analysis_step_cancel_wrapper_interrupts_pending_future() {
        let token = CancellationToken::new();
        token.cancel();

        let result: Result<(), LlmRequestError> =
            run_analysis_step_with_cancel(Some(token), std::future::pending()).await;

        assert!(matches!(result, Err(LlmRequestError::Cancelled)));
    }

    #[test]
    fn parse_chunk_summary_ignores_non_json_prefix_with_braces() {
        let summary = parse_chunk_summary(&format!("Note {{not json}}\n{SAMPLE_JSON}"))
            .expect("parse summary");

        assert_eq!(summary.summary, "Brief");
        assert_eq!(summary.topics, vec!["sync".to_string()]);
    }

    #[test]
    fn parse_chunk_summary_rejects_malformed_payload() {
        let error = parse_chunk_summary("```json\n{\"summary\": }\n```")
            .expect_err("malformed payload should fail");

        assert!(
            error.contains("Failed to parse chunk summary JSON")
                || error.contains("malformed JSON")
                || error.contains("valid JSON object")
        );
    }

    #[test]
    fn finish_map_phase_preserves_chunk_order_by_original_index() {
        let ordered = vec![
            Some(sample_chunk_summary("first")),
            Some(sample_chunk_summary("second")),
            Some(sample_chunk_summary("third")),
        ];

        let collected = finish_map_phase(ordered, None).expect("collect summaries");

        assert_eq!(collected[0].summary, "first");
        assert_eq!(collected[1].summary, "second");
        assert_eq!(collected[2].summary, "third");
    }

    #[test]
    fn finish_map_phase_rejects_missing_chunk_before_reduce() {
        let ordered = vec![Some(sample_chunk_summary("first")), None];

        let error = finish_map_phase(ordered, None).expect_err("missing chunk should fail");

        assert_eq!(
            error,
            ReportRunError::Failed("Some chunk summaries were not collected".to_string())
        );
    }

    #[test]
    fn finish_map_phase_propagates_map_error_without_starting_reduce() {
        let ordered = vec![Some(sample_chunk_summary("first"))];

        let error = finish_map_phase(
            ordered,
            Some(ReportRunError::Cancelled(
                "Analysis run cancelled.".to_string(),
            )),
        )
        .expect_err("map cancellation should stop reduce");

        assert_eq!(
            error,
            ReportRunError::Cancelled("Analysis run cancelled.".to_string())
        );
    }

    #[test]
    fn build_map_request_keeps_run_scoped_request_and_profile() {
        let request =
            build_map_request(55, "default".to_string(), 2, 4, &[sample_corpus_message()]);

        assert!(request.request_id.starts_with("analysis-map-55-2-"));
        assert_eq!(request.profile_id.as_deref(), Some("default"));
        assert!(request.messages[0]
            .content
            .contains("source document excerpts"));
        assert!(request.messages[1].content.contains("Chunk 2 of 4."));
        assert!(request.messages[1].content.contains("Documents:"));
    }

    #[test]
    fn build_reduce_request_keeps_run_scoped_request_and_profile() {
        let prompt_template = sample_prompt_template();
        let chunk_summaries = vec![sample_chunk_summary("alpha"), sample_chunk_summary("beta")];
        let request = build_reduce_request(ReduceRequestParams {
            run_id: 77,
            profile_id: "profile-a".to_string(),
            scope_label: "My scope",
            output_language: "Russian",
            prompt_template: &prompt_template,
            period_from: 10,
            period_to: 20,
            chunk_summaries: &chunk_summaries,
            model_override: Some("model-x".to_string()),
        });

        assert!(request.request_id.starts_with("analysis-reduce-77-"));
        assert_eq!(request.profile_id.as_deref(), Some("profile-a"));
        assert_eq!(request.model_override.as_deref(), Some("model-x"));
        assert!(request.messages[0].content.contains("[s12-i845]"));
        assert!(request.messages[1].content.contains("Chunk 1 summary"));
        assert!(request.messages[1].content.contains("Chunk 2 summary"));
    }

    #[test]
    fn validate_report_preflight_rejects_empty_corpus() {
        let error = validate_report_preflight(&AnalysisRunPreflight {
            source_ids: vec![1],
            message_count: 0,
            estimated_input_chars: 0,
            estimated_chunks: 0,
            limits: AnalysisRunPreflightLimits::default(),
        })
        .expect_err("empty corpus should fail");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert_eq!(
            error.message,
            "No synced source documents were found for the selected analysis scope and period"
        );
    }

    #[test]
    fn validate_report_preflight_rejects_oversized_runs() {
        let error = validate_report_preflight(&AnalysisRunPreflight {
            source_ids: vec![1],
            message_count: 10_001,
            estimated_input_chars: 100_000,
            estimated_chunks: 10,
            limits: AnalysisRunPreflightLimits::default(),
        })
        .expect_err("oversized corpus should fail");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert!(error.message.contains("Analysis scope is too large"));
    }

    #[test]
    fn validate_report_preflight_allows_runs_within_limits() {
        validate_report_preflight(&AnalysisRunPreflight {
            source_ids: vec![1],
            message_count: 100,
            estimated_input_chars: 50_000,
            estimated_chunks: 4,
            limits: AnalysisRunPreflightLimits::default(),
        })
        .expect("preflight should pass");
    }

    #[test]
    fn analysis_report_workflow_file_has_no_tauri_command_adapters() {
        let source = std::fs::read_to_string("src/analysis/report.rs").expect("read report.rs");
        let command_attribute = ["#[tauri", "::command]"].join("");

        assert!(
            !source.contains(&command_attribute),
            "Analysis report command adapters should live outside src/analysis/report.rs"
        );
    }
}
