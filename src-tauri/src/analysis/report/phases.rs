use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, Manager};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::llm::{
    run_llm_collect_with_profile, run_llm_stream_with_profile, LlmCompletion, LlmRequestError,
    LlmRequestKind, LlmRequestMetadata, LlmRequestPriority, LlmSchedulerState, ResolvedLlmProfile,
};

use super::super::models::{AnalysisChunkSummaryEvent, ChunkSummary, CorpusMessage};
use super::super::state::AnalysisState;
use super::requests::{
    build_map_request, build_reduce_request, parse_chunk_summary, ReduceRequestParams,
};
use super::{ReportRunError, ReportRunInput, RunEvent, CANCELLED_RUN_MESSAGE};

pub(super) fn finish_map_phase(
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

pub(super) struct ReportPipelineContext {
    pub(super) handle: AppHandle,
    pub(super) pool: Pool<Sqlite>,
    pub(super) resolved_profile: ResolvedLlmProfile,
    pub(super) run_id: i64,
}

impl ReportPipelineContext {
    pub(super) async fn ensure_not_cancelled(&self) -> Result<(), ReportRunError> {
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

    pub(super) fn emit(&self, event: RunEvent) {
        event.emit(&self.handle);
    }
}

pub(super) struct ReducePhaseResult {
    pub(super) request_id: String,
    pub(super) completion: LlmCompletion,
}

pub(super) async fn run_analysis_step_with_cancel<Fut, T>(
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

pub(super) async fn run_map_phase(
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

pub(super) async fn run_reduce_phase(
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
